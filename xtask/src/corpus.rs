use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    ffi::OsString,
    fs,
    io::{Read, Write},
    path::Path,
    process::{Command, Stdio},
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    version: u8,
    repository: String,
    revision: String,
    notice: String,
    files: Vec<Fixture>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    name: String,
    path: String,
    format: Format,
    size: usize,
    sha256: String,
    license: String,
    provenance: String,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Format {
    Cdxml,
    Mol,
}

fn normal_component(name: &str) -> bool {
    !name.is_empty()
        && !matches!(name, "." | "..")
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

fn validate(manifest: &Manifest) -> Result<(), Box<dyn Error>> {
    let repository: Vec<_> = manifest.repository.split('/').collect();
    if manifest.version != 1
        || manifest.files.is_empty()
        || manifest.files.len() > 1024
        || repository.len() != 2
        || !repository.iter().all(|part| normal_component(part))
        || manifest.revision.len() != 40
        || !manifest.revision.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err("corpus requires a supported version, files and a pinned GitHub commit".into());
    }
    let mut names = BTreeSet::new();
    for file in &manifest.files {
        let extension = match file.format {
            Format::Cdxml => ".cdxml",
            Format::Mol => ".mol",
        };
        if !normal_component(&file.name)
            || !file.name.ends_with(extension)
            || !names.insert(&file.name)
            || !file.path.split('/').all(normal_component)
            || !file.path.ends_with(extension)
            || file.size == 0
            || file.size > 8 * 1024 * 1024
            || file.sha256.len() != 64
            || !file.sha256.bytes().all(|b| b.is_ascii_hexdigit())
            || !super::licenses::allowed(&file.license)
            || file.provenance.trim().is_empty()
        {
            return Err(format!("unreviewed or invalid corpus entry: {}", file.name).into());
        }
    }
    Ok(())
}

fn verify(file: &Fixture, bytes: &[u8]) -> Result<(), Box<dyn Error>> {
    if bytes.len() != file.size || format!("{:x}", Sha256::digest(bytes)) != file.sha256 {
        return Err(format!("corpus length/SHA-256 mismatch: {}", file.name).into());
    }
    Ok(())
}

fn python() -> OsString {
    std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    })
}

fn download(manifest: &Manifest, file: &Fixture) -> Result<Vec<u8>, Box<dyn Error>> {
    let url = format!(
        "https://raw.githubusercontent.com/{}/{}/{}",
        manifest.repository, manifest.revision, file.path
    );
    let result = Command::new(python())
        .args([
            "-c",
            r#"
import sys,urllib.request
with urllib.request.urlopen(sys.argv[1],timeout=15) as response:
    if response.status!=200: raise RuntimeError('corpus HTTP status is not 200')
    sys.stdout.buffer.write(response.read(int(sys.argv[2])+1))
"#,
            &url,
            &file.size.to_string(),
        ])
        .output()?;
    if !result.status.success() {
        return Err(String::from_utf8_lossy(&result.stderr).into_owned().into());
    }
    verify(file, &result.stdout)?;
    Ok(result.stdout)
}

fn expanded_name(namespace: Option<&str>, local: &str) -> String {
    namespace.map_or_else(|| local.to_owned(), |uri| format!("{uri}|{local}"))
}

fn compare_xml(
    bytes: &[u8],
    document: &bondline_cdxml::Document<'_>,
) -> Result<(), Box<dyn Error>> {
    let elements: Vec<_> = document
        .xml()
        .descendants()
        .filter(|node| node.is_element())
        .map(|node| {
            let tag = node.tag_name();
            let attributes: BTreeMap<_, _> = node
                .attributes()
                .map(|attribute| {
                    (
                        expanded_name(attribute.namespace(), attribute.name()),
                        attribute.value(),
                    )
                })
                .collect();
            json!({"tag":expanded_name(tag.namespace(), tag.name()),"attributes":attributes})
        })
        .collect();
    let text: String = document
        .xml()
        .descendants()
        .filter(|node| node.is_text())
        .filter_map(|node| node.text())
        .collect();
    let expected = json!({"elements":elements,"text":text});
    let independent = xml_oracle(
        r#"
import json,sys,xml.parsers.expat as expat
parser=expat.ParserCreate(namespace_separator='|')
elements=[]
text=[]
parser.StartElementHandler=lambda tag,attributes: elements.append({'tag':tag,'attributes':attributes})
parser.CharacterDataHandler=text.append
parser.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
parser.Parse(sys.stdin.buffer.read(),True)
json.dump({'elements':elements,'text':''.join(text)},sys.stdout)
"#,
        bytes,
    )?;
    if independent != expected {
        return Err("CDXML semantic view differs from independent Expat".into());
    }
    Ok(())
}

fn xml_oracle(script: &str, bytes: &[u8]) -> Result<Value, Box<dyn Error>> {
    let mut oracle = Command::new(python())
        .args(["-c", script])
        .env("PYTHONIOENCODING", "ascii")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    oracle
        .stdin
        .take()
        .ok_or("missing Expat stdin")?
        .write_all(bytes)?;
    let result = oracle.wait_with_output()?;
    if !result.status.success() {
        return Err(String::from_utf8_lossy(&result.stderr).into_owned().into());
    }
    Ok(serde_json::from_slice(&result.stdout)?)
}

fn compare_graph(bytes: &[u8], projection: &Value) -> Result<(), Box<dyn Error>> {
    let independent = xml_oracle(include_str!("../oracles/cdxml_graph.py"), bytes)?;
    if independent != *projection {
        return Err("CDXML elemental projection differs from independent Expat/Decimal".into());
    }
    Ok(())
}

pub fn check(offline: bool) -> Result<(), Box<dyn Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("missing workspace root")?;
    let manifest: Manifest =
        serde_json::from_slice(&fs::read(root.join("docs/corpus/manifest.json"))?)?;
    validate(&manifest)?;
    let notice = super::licenses::confined_path(root, &manifest.notice)?;
    if !notice.is_file() || notice.metadata()?.len() == 0 {
        return Err("corpus notice is empty".into());
    }
    let cache = root.join(".cache/corpus");
    fs::create_dir_all(&cache)?;
    if !cache.canonicalize()?.starts_with(root.canonicalize()?) {
        return Err("corpus cache escapes workspace".into());
    }
    let mut round_trips = 0;
    let mut pending = Vec::new();
    for file in &manifest.files {
        let path = cache.join(&file.name);
        let bytes = if path.try_exists()? {
            let path = super::licenses::confined_path(&cache, &file.name)?;
            let mut bytes = Vec::new();
            fs::File::open(path)?
                .take(file.size as u64 + 1)
                .read_to_end(&mut bytes)?;
            verify(file, &bytes)?;
            bytes
        } else {
            if offline {
                return Err(format!("offline corpus entry is missing: {}", file.name).into());
            }
            let bytes = download(&manifest, file)?;
            let staged = cache.join(format!("{}.{}.part", file.name, std::process::id()));
            let mut target = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&staged)?;
            target.write_all(&bytes)?;
            target.sync_all()?;
            drop(target);
            fs::rename(staged, path)?;
            bytes
        };
        match file.format {
            Format::Cdxml => {
                let document = bondline_cdxml::Document::parse(&bytes)?;
                let output = document.to_bytes();
                if output != bytes {
                    return Err(format!("CDXML byte loss: {}", file.name).into());
                }
                compare_xml(&output, &document)?;
                drop(document);
                let stored = bondline_io::StoredDocument::from_cdxml(
                    std::sync::Arc::from(bytes),
                    bondline_model::PageSize::A4,
                )?;
                if stored.export_cdxml()? != output {
                    return Err(format!("source-backed CDXML byte loss: {}", file.name).into());
                }
                let fragments: Vec<_> = stored
                    .model()
                    .pages()
                    .iter()
                    .flat_map(|page| page.fragments().iter().map(std::sync::Arc::as_ref))
                    .collect();
                compare_graph(stored.export_cdxml()?, &serde_json::to_value(fragments)?)?;
                round_trips += 1;
            }
            Format::Mol => pending.push(&file.name),
        }
    }
    writeln!(
        std::io::stdout(),
        "corpus integrity: {} pinned licensed files; CDXML byte/Expat checks: {round_trips}; elemental projections: {round_trips}; source-backed snapshots: {round_trips}",
        manifest.files.len()
    )?;
    for name in pending {
        writeln!(
            std::io::stdout(),
            "{name}: integrity only; MOL parser/chemistry checks MISSING"
        )?;
    }
    writeln!(
        std::io::stdout(),
        "chemistry calculations (formula/mass/aromaticity/stereo/SMILES/InChI) MISSING"
    )?;
    if round_trips == 0 {
        return Err("corpus has no implemented parser coverage".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Manifest, validate, verify};

    #[test]
    fn manifest_pins_provenance_and_rejects_unlicensed_or_unconfined_sources()
    -> Result<(), Box<dyn std::error::Error>> {
        let original = include_bytes!("../../docs/corpus/manifest.json");
        let manifest: Manifest = serde_json::from_slice(original)?;
        validate(&manifest)?;
        assert_eq!(manifest.files.len(), 2);
        for field in ["license", "name", "path", "sha256", "provenance"] {
            let mut data: serde_json::Value = serde_json::from_slice(original)?;
            data["files"][0][field] = match field {
                "license" => "ISC",
                "name" => "../escape.cdxml",
                "path" => "../secret",
                "sha256" => "unverified",
                _ => "",
            }
            .into();
            assert!(
                validate(&serde_json::from_value(data)?).is_err(),
                "accepted {field}"
            );
        }
        let mut manifest: Manifest = serde_json::from_slice(original)?;
        manifest.revision = "main".into();
        assert!(validate(&manifest).is_err());
        Ok(())
    }

    #[test]
    fn exact_length_and_sha256_are_required_even_for_cached_files()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut manifest: Manifest =
            serde_json::from_slice(include_bytes!("../../docs/corpus/manifest.json"))?;
        let fixture = manifest.files.first_mut().ok_or("missing fixture")?;
        fixture.size = 3;
        // Published SHA-256 "abc" vector, independent of the downloaded fixture.
        fixture.sha256 = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad".into();
        verify(fixture, b"abc")?;
        assert!(verify(fixture, b"abd").is_err());
        assert!(verify(fixture, b"abcd").is_err());
        Ok(())
    }

    #[test]
    fn independent_xml_comparison_detects_a_semantic_disagreement()
    -> Result<(), Box<dyn std::error::Error>> {
        let bytes = include_bytes!("../../crates/cdxml/tests/fixtures/unknown.cdxml");
        let document = bondline_cdxml::Document::parse(bytes)?;
        super::compare_xml(bytes, &document)?;
        let changed = std::str::from_utf8(bytes)?.replace("Order='2.00'", "Order='1.00'");
        assert!(super::compare_xml(changed.as_bytes(), &document).is_err());
        Ok(())
    }

    #[test]
    fn independent_elemental_projection_detects_mapping_disagreements()
    -> Result<(), Box<dyn std::error::Error>> {
        let bytes = include_bytes!("../../crates/cdxml/tests/fixtures/unknown.cdxml");
        let graph = bondline_io::cdxml::ElementalGraph::parse(bytes)?;
        let mut projection = serde_json::to_value(graph.fragments())?;
        super::compare_graph(bytes, &projection)?;
        projection[0]["atoms"][0]["element"] = serde_json::json!(8);
        assert!(super::compare_graph(bytes, &projection).is_err());
        Ok(())
    }

    #[test]
    fn projection_oracle_checks_scoped_references_and_explicit_metadata()
    -> Result<(), Box<dyn std::error::Error>> {
        let bytes=br#"<CDXML><page><fragment id="11"><n id="29.00" Isotope="13" Charge="-1" Radical="Doublet" NumHydrogens="0" p="72 144"/><n id="30" Element="8"/><b B="30" E="29" Order="dative"/></fragment><group><fragment id="11"><n id="29"/><n id="30" Element="7" Radical="Singlet" NumHydrogens="3"/><b B="29" E="30" Order="1.500"/></fragment></group></page></CDXML>"#;
        let graph = bondline_io::cdxml::ElementalGraph::parse(bytes)?;
        super::compare_graph(bytes, &serde_json::to_value(graph.fragments())?)?;
        Ok(())
    }
}
