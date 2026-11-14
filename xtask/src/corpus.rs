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

mod v3000;

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
    Cdx,
    Mol,
    #[serde(rename = "mol_v3000")]
    MolV3000,
    Sdf,
}

const MANIFESTS: &[&str] = &[
    "docs/corpus/manifest.json",
    "docs/corpus/bondline-manifest.json",
    "docs/corpus/bondline-cdx-manifest.json",
    "docs/corpus/bondline-v3000-manifest.json",
];
const MAX_MANIFEST_BYTES: usize = 1024 * 1024;
const MAX_FIXTURES: usize = 1024;

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
            Format::Cdx => ".cdx",
            Format::Mol | Format::MolV3000 => ".mol",
            Format::Sdf => ".sdf",
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

fn load_manifests(root: &Path) -> Result<Vec<Manifest>, Box<dyn Error>> {
    let mut manifests = Vec::new();
    let mut names = BTreeSet::new();
    let mut count = 0_usize;
    for relative in MANIFESTS {
        let path = super::licenses::confined_path(root, relative)?;
        let mut bytes = Vec::new();
        fs::File::open(path)?
            .take(MAX_MANIFEST_BYTES as u64 + 1)
            .read_to_end(&mut bytes)?;
        if bytes.len() > MAX_MANIFEST_BYTES {
            return Err("corpus manifest exceeds 1 MiB".into());
        }
        let manifest: Manifest = serde_json::from_slice(&bytes)?;
        validate(&manifest)?;
        let notice = super::licenses::confined_path(root, &manifest.notice)?;
        if !notice.is_file() || notice.metadata()?.len() == 0 {
            return Err("corpus notice is empty".into());
        }
        for fixture in &manifest.files {
            count = count.checked_add(1).ok_or("corpus count overflow")?;
            if count > MAX_FIXTURES {
                return Err("corpus exceeds 1024 total fixtures".into());
            }
            // Cache names must remain distinct on case-insensitive native filesystems too.
            if !names.insert(fixture.name.to_ascii_lowercase()) {
                return Err(format!("duplicate corpus cache name: {}", fixture.name).into());
            }
        }
        manifests.push(manifest);
    }
    Ok(manifests)
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
    let independent = python_oracle(
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

fn python_oracle(script: &str, bytes: &[u8]) -> Result<Value, Box<dyn Error>> {
    Ok(serde_json::from_slice(&python_output(script, &[], bytes)?)?)
}

fn python_output(
    script: &str,
    arguments: &[&str],
    bytes: &[u8],
) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut oracle = Command::new(python())
        .args(["-B", "-c", script])
        .args(arguments)
        .env("PYTHONIOENCODING", "ascii")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    oracle
        .stdin
        .take()
        .ok_or("missing oracle stdin")?
        .write_all(bytes)?;
    let result = oracle.wait_with_output()?;
    if !result.status.success() {
        return Err(String::from_utf8_lossy(&result.stderr).into_owned().into());
    }
    Ok(result.stdout)
}

fn compare_graph(bytes: &[u8], projection: &Value) -> Result<(), Box<dyn Error>> {
    let independent = python_oracle(include_str!("../oracles/cdxml_graph.py"), bytes)?;
    if independent != *projection {
        return Err("CDXML elemental projection differs from independent Expat/Decimal".into());
    }
    Ok(())
}

fn compare_mol(
    bytes: &[u8],
    document: &bondline_io::mol::MolFile<'_>,
    projection: &Value,
) -> Result<(), Box<dyn Error>> {
    let raw_atoms: Vec<_> = document
        .atoms()
        .iter()
        .map(|atom| {
            json!({
                "symbol": atom.symbol,
                "element": atom.element.map(bondline_model::Element::get),
                "position": [atom.x, atom.y, atom.z],
                "mass_difference": atom.mass_difference,
                "isotope": atom.isotope.map(std::num::NonZeroU16::get),
                "charge": atom.charge,
                "radical": atom.radical,
                "declared_valence": atom.declared_valence,
            })
        })
        .collect();
    let raw_bonds: Vec<_> = document
        .bonds()
        .iter()
        .map(|bond| json!({"begin":bond.begin,"end":bond.end,"order":bond.order}))
        .collect();
    let expected = json!({
        "raw": {"header": document.header(), "atoms":raw_atoms,"bonds":raw_bonds},
        "fragment": projection,
    });
    let mut independent = python_oracle(include_str!("../oracles/mol_graph.py"), bytes)?;
    // Decimal supplies the reference; binary64 scaling may differ by a few ulps.
    for axis in ["x_pt", "y_pt"] {
        let path = format!("/fragment/atoms/0/position/{axis}");
        let observed = expected
            .pointer(&path)
            .and_then(Value::as_f64)
            .ok_or("MOL projection has no finite coordinate")?;
        let reference = independent
            .pointer(&path)
            .and_then(Value::as_f64)
            .ok_or("MOL Decimal oracle has no coordinate")?;
        if !observed.is_finite() || (observed - reference).abs() > 1e-10 {
            return Err("MOL coordinate differs from independent Decimal transform".into());
        }
        *independent
            .pointer_mut(&path)
            .ok_or("MOL Decimal oracle coordinate is missing")? = json!(observed);
    }
    if independent != expected {
        return Err("MOL raw/model projection differs from independent RDKit/Decimal".into());
    }
    Ok(())
}

fn check_mol(bytes: &[u8]) -> Result<(), Box<dyn Error>> {
    let document = bondline_io::mol::MolFile::parse(bytes)?;
    if document.as_bytes() != bytes {
        return Err("MOL source byte loss".into());
    }
    let transform =
        bondline_io::mol::MolTransform::new(24.0, bondline_model::Point2::new(0.0, 0.0)?)?;
    let projection = document.to_fragment(transform)?;
    compare_mol(bytes, &document, &serde_json::to_value(projection)?)
}

fn sdf_table(file: &bondline_io::sdf::SdfFile<'_>) -> Result<Value, Box<dyn Error>> {
    fn bounds(range: std::ops::Range<usize>) -> [usize; 2] {
        [range.start, range.end]
    }
    let transform =
        bondline_io::mol::MolTransform::new(24.0, bondline_model::Point2::new(0.0, 0.0)?)?;
    let records = file.records().iter().map(|record| {
        let fields = record.fields().iter().map(|field| Ok(json!({
            "range":bounds(field.range()),"header_range":bounds(field.header_range()),
            "value_range":bounds(field.value_range()),"header":field.header(),
            "name":field.name(),"dt_number":field.dt_number(),
            "raw_value":std::str::from_utf8(field.raw_value())?,"value_lines":field.value_lines(),
        }))).collect::<Result<Vec<_>,Box<dyn Error>>>()?;
        let fragment = bondline_io::mol::MolFile::parse(record.mol_bytes())?.to_fragment(transform)?;
        Ok(json!({"range":bounds(record.range()),"mol_range":bounds(record.mol_range()),
            "delimiter_range":bounds(record.delimiter_range()),"fields":fields,"projection":fragment}))
    }).collect::<Result<Vec<_>,Box<dyn Error>>>()?;
    Ok(json!(records))
}

fn sdf_reference_projection(graph: &Value) -> Result<Value, Box<dyn Error>> {
    let atoms = graph["atoms"].as_array().ok_or("missing RDKit atoms")?.iter().enumerate().map(|(index, atom)| {
        // Current SDF corpus authors explicit H counts and has no radical-spin ambiguity.
        if atom["radical_electrons"] != 0 || atom["position"][2] != 0.0 {
            return Err("SDF corpus radical/3D reference mapping is not verified".into());
        }
        let x = atom["position"][0].as_f64().ok_or("missing RDKit x")? * 24.0;
        let y = -atom["position"][1].as_f64().ok_or("missing RDKit y")? * 24.0;
        let isotope = if atom["isotope"] == 0 { Value::Null } else { atom["isotope"].clone() };
        Ok(json!({"id":index.checked_add(1).ok_or("atom ID overflow")?,"element":atom["element"],
            "position":{"x_pt":x,"y_pt":y},"isotope":isotope,"charge":atom["charge"],
            "radical":"none","explicit_hydrogens":atom["hydrogens"]}))
    }).collect::<Result<Vec<_>,Box<dyn Error>>>()?;
    let bonds = graph["bonds"].as_array().ok_or("missing RDKit bonds")?.iter().enumerate().map(|(index,bond)| {
        let order = match bond["order"].as_f64() {
            Some(1.0) => "single",Some(2.0) => "double",Some(3.0) => "triple",Some(1.5) => "aromatic",
            _ => return Err("unsupported RDKit corpus bond order".into()),
        };
        Ok(json!({"id":index.checked_add(1).ok_or("bond ID overflow")?,
            "begin":bond["begin"].as_u64().and_then(|id|id.checked_add(1)).ok_or("RDKit begin ID")?,
            "end":bond["end"].as_u64().and_then(|id|id.checked_add(1)).ok_or("RDKit end ID")?,"order":order}))
    }).collect::<Result<Vec<_>,Box<dyn Error>>>()?;
    Ok(json!({"id":1,"atoms":atoms,"bonds":bonds}))
}

fn compare_sdf(bytes: &[u8], projection: &Value) -> Result<(usize, usize), Box<dyn Error>> {
    let independent = python_oracle(include_str!("../../crates/io/tests/sdf_oracle.py"), bytes)?;
    if independent["source_sha256"] != format!("{:x}", Sha256::digest(bytes)) {
        return Err("SDF oracle source identity mismatch".into());
    }
    let mut records = independent["records"]
        .as_array()
        .ok_or("missing independent SDF records")?
        .clone();
    for record in &mut records {
        let object = record.as_object_mut().ok_or("invalid SDF oracle record")?;
        let graph = object.remove("rdkit").ok_or("missing direct RDKit graph")?;
        object.insert("projection".into(), sdf_reference_projection(&graph)?);
    }
    if json!(records) != *projection {
        return Err("SDF ordered spans/data/model differ from independent scanner/RDKit".into());
    }
    let consumer_entries = independent["rdkit_records"]
        .as_array()
        .ok_or("missing supplier diagnostic")?
        .len();
    Ok((records.len(), consumer_entries))
}

fn check_sdf(bytes: &[u8]) -> Result<(usize, usize), Box<dyn Error>> {
    let file = bondline_io::sdf::SdfFile::parse(bytes)?;
    if file.as_bytes() != bytes {
        return Err("SDF source byte loss".into());
    }
    compare_sdf(bytes, &sdf_table(&file)?)
}

fn cdx_table(document: &bondline_cdx::Document<'_>) -> String {
    fn index(index: Option<usize>) -> String {
        index.map_or_else(|| "-".to_owned(), |value| value.to_string())
    }
    let mut lines = Vec::new();
    for record in document.records() {
        let parent = index(record.parent());
        let range = record.range();
        let suffix = format!("{parent} {} {}", range.start, range.end);
        lines.push(match record.kind() {
            bondline_cdx::RecordKind::ObjectStart { tag, id } => {
                format!("start {tag} {id} {suffix}")
            }
            bondline_cdx::RecordKind::Property {
                tag,
                length,
                payload,
            } => {
                let encoding = match length {
                    bondline_cdx::LengthEncoding::Short => "short",
                    bondline_cdx::LengthEncoding::Extended => "extended",
                };
                format!(
                    "prop {tag} {encoding} {} {} {} {suffix}",
                    payload.len(),
                    payload.start,
                    payload.end
                )
            }
            bondline_cdx::RecordKind::ObjectEnd { start } => {
                format!("end {} {suffix}", index(*start))
            }
        });
    }
    lines.join("\n") + "\n"
}

fn compare_cdx(bytes: &[u8], table: &str) -> Result<(), Box<dyn Error>> {
    let script = include_str!("../oracles/cdx_oracle.py");
    if python_output(script, &["describe"], bytes)? != table.as_bytes() {
        return Err("CDX record spans/lengths/ownership differ from independent struct".into());
    }
    // Keep the original scene/opaque-byte guard: this is not a general CDX validator.
    python_output(script, &["verify", "1", "explicit"], bytes)?;
    Ok(())
}

fn check_cdx(bytes: &[u8]) -> Result<(), Box<dyn Error>> {
    let document = bondline_cdx::Document::parse(bytes)?;
    if document.framing() != bondline_cdx::Framing::ExplicitDocument {
        return Err("CDX corpus expects the original explicit-root field-vector scene".into());
    }
    if document.as_bytes() != bytes || document.to_bytes() != bytes {
        return Err("CDX source byte loss".into());
    }
    compare_cdx(bytes, &cdx_table(&document))
}

pub fn check(offline: bool) -> Result<(), Box<dyn Error>> {
    let root = super::workspace::root()?;
    check_at(&root, offline, &mut std::io::stdout().lock())
}

fn check_at(root: &Path, offline: bool, output: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let manifests = load_manifests(root)?;
    let fixture_count: usize = manifests.iter().map(|manifest| manifest.files.len()).sum();
    let cache = root.join(".cache/corpus");
    let confined_root = root.canonicalize()?;
    let mut ancestor = cache.as_path();
    loop {
        match fs::symlink_metadata(ancestor) {
            Ok(_) => break,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                ancestor = ancestor.parent().ok_or("missing corpus cache ancestor")?;
            }
            Err(error) => return Err(error.into()),
        }
    }
    if !ancestor.canonicalize()?.starts_with(&confined_root) {
        return Err("corpus cache ancestor escapes workspace".into());
    }
    fs::create_dir_all(&cache)?;
    if !cache.canonicalize()?.starts_with(&confined_root) {
        return Err("corpus cache escapes workspace".into());
    }
    let mut round_trips = 0;
    let mut mol_checks = 0;
    let mut v3000_checks = 0;
    let mut cdx_checks = 0;
    let mut sdf_checks = 0;
    let mut sdf_records = 0;
    let mut supplier_entries = 0;
    for manifest in &manifests {
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
                let bytes = download(manifest, file)?;
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
                Format::Mol => {
                    check_mol(&bytes)?;
                    mol_checks += 1;
                }
                Format::MolV3000 => {
                    v3000::check(&bytes)?;
                    v3000_checks += 1;
                }
                Format::Cdx => {
                    check_cdx(&bytes)?;
                    cdx_checks += 1;
                }
                Format::Sdf => {
                    let (records, entries) = check_sdf(&bytes)?;
                    sdf_checks += 1;
                    sdf_records += records;
                    supplier_entries += entries;
                }
            }
        }
    }
    writeln!(
        output,
        "corpus integrity: {fixture_count} pinned licensed files from {} source bundles; CDXML byte/Expat checks: {round_trips}; elemental projections: {round_trips}; source-backed snapshots: {round_trips}",
        manifests.len()
    )?;
    writeln!(
        output,
        "MOL V2000 byte/source/RDKit/Decimal checks: {mol_checks} (strict elemental input subset)"
    )?;
    writeln!(
        output,
        "MOL V3000 original-vector byte/source/literal/Decimal/RDKit checks: {v3000_checks} (standalone subset)"
    )?;
    writeln!(
        output,
        "SDF V2000 byte/ordered-field/per-record RDKit checks: {sdf_checks}; records: {sdf_records}"
    )?;
    writeln!(
        output,
        "SDF full-supplier entries: {supplier_entries} (diagnostic only; DT-only consumer loss is separate from raw fidelity)"
    )?;
    writeln!(
        output,
        "CDX original explicit field-vector byte/struct/RDKit checks: {cdx_checks}"
    )?;
    writeln!(
        output,
        "CDX foreign/general/application compatibility MISSING"
    )?;
    writeln!(
        output,
        "chemistry calculations (formula/mass/aromaticity/stereo/SMILES/InChI) MISSING"
    )?;
    if round_trips == 0
        && mol_checks == 0
        && v3000_checks == 0
        && sdf_checks == 0
        && cdx_checks == 0
    {
        return Err("corpus has no implemented parser coverage".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Manifest, validate, verify};
    use sha2::Digest;
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct CorpusDirectory(PathBuf);
    impl CorpusDirectory {
        fn new() -> Result<Self, Box<dyn std::error::Error>> {
            let path = std::env::temp_dir().join(format!(
                "bondline-corpus-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path)?;
            let directory = Self(path);
            fs::create_dir_all(directory.0.join("docs/corpus"))?;
            fs::write(
                directory.0.join(super::MANIFESTS[0]),
                include_bytes!("../../docs/corpus/manifest.json"),
            )?;
            fs::write(
                directory.0.join(super::MANIFESTS[1]),
                include_bytes!("../../docs/corpus/bondline-manifest.json"),
            )?;
            fs::write(
                directory.0.join(super::MANIFESTS[2]),
                include_bytes!("../../docs/corpus/bondline-cdx-manifest.json"),
            )?;
            fs::write(
                directory.0.join(super::MANIFESTS[3]),
                include_bytes!("../../docs/corpus/bondline-v3000-manifest.json"),
            )?;
            fs::write(
                directory.0.join("docs/corpus/rdkit-license.txt"),
                include_bytes!("../../docs/corpus/rdkit-license.txt"),
            )?;
            fs::write(
                directory.0.join("LICENSE-MIT"),
                include_bytes!("../../LICENSE-MIT"),
            )?;
            Ok(directory)
        }
        fn write_manifest(
            &self,
            index: usize,
            data: &serde_json::Value,
        ) -> Result<(), Box<dyn std::error::Error>> {
            fs::write(
                self.0
                    .join(super::MANIFESTS.get(index).ok_or("manifest index")?),
                serde_json::to_vec(data)?,
            )?;
            Ok(())
        }
    }
    impl Drop for CorpusDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn v3000_is_an_explicit_reviewed_corpus_format() -> Result<(), Box<dyn std::error::Error>> {
        let file: super::Fixture = serde_json::from_value(serde_json::json!({
            "name":"bondline-original-noncontiguous-v3000.mol",
            "path":"crates/io/tests/fixtures/v3000/noncontiguous-cc.mol",
            "format":"mol_v3000",
            "size":344,
            "sha256":"1867a5904e742030431b523a31167951ff998007a89b9c3d94ec88cf7d4c4380",
            "license":"MIT OR Apache-2.0",
            "provenance":"Original Bondline specification-derived two-carbon V3000 field vector."
        }))?;
        assert_eq!(file.size, 344);
        assert_eq!(file.license, "MIT OR Apache-2.0");
        assert!(matches!(file.format, super::Format::MolV3000));
        let manifest: Manifest = serde_json::from_slice(include_bytes!(
            "../../docs/corpus/bondline-v3000-manifest.json"
        ))?;
        validate(&manifest)?;
        assert_eq!(manifest.repository, "thematheusmello/bondline");
        assert_eq!(
            manifest.revision,
            "5caf6ce0ecf40badf2666e36dcb73dc8c4337b51"
        );
        assert_eq!(manifest.notice, "LICENSE-MIT");
        assert_eq!(manifest.files.len(), 1);
        let pinned = manifest.files.first().ok_or("V3000 fixture")?;
        assert_eq!(pinned.name, file.name);
        assert_eq!(pinned.path, file.path);
        assert_eq!(pinned.sha256, file.sha256);
        assert_eq!(pinned.size, file.size);
        assert_eq!(pinned.license, file.license);
        assert!(matches!(pinned.format, super::Format::MolV3000));
        verify(
            pinned,
            include_bytes!("../../crates/io/tests/fixtures/v3000/noncontiguous-cc.mol"),
        )?;
        for spelling in ["molv3000", "MOL_V3000", "v3000"] {
            let mut invalid = serde_json::json!({
                "name":file.name,"path":file.path,"format":spelling,"size":file.size,
                "sha256":file.sha256,"license":file.license,"provenance":file.provenance
            });
            assert!(serde_json::from_value::<super::Fixture>(invalid.clone()).is_err());
            invalid["format"] = serde_json::json!("mol_v3000");
            invalid["extra"] = serde_json::json!(true);
            assert!(serde_json::from_value::<super::Fixture>(invalid).is_err());
        }
        Ok(())
    }

    #[test]
    fn fourth_bundle_is_validated_globally_before_any_cache_io()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = CorpusDirectory::new()?;
        let original: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../docs/corpus/bondline-v3000-manifest.json"
        ))?;
        let mut alias = original.clone();
        alias["files"][0]["name"] = serde_json::json!("BONDLINE-ORIGINAL-NONCONTIGUOUS-V3000.mol");
        directory.write_manifest(0, &original)?;
        directory.write_manifest(3, &alias)?;
        assert!(
            super::check_at(&directory.0, true, &mut Vec::new())
                .err()
                .ok_or("fourth bundle bypassed global names")?
                .to_string()
                .contains("duplicate corpus cache name")
        );
        assert!(!directory.0.join(".cache").exists());
        fs::write(
            directory.0.join(super::MANIFESTS[0]),
            include_bytes!("../../docs/corpus/manifest.json"),
        )?;
        for (pointer, value) in [
            ("/notice", serde_json::json!("../LICENSE-MIT")),
            ("/files/0/license", serde_json::json!("ISC")),
            ("/files/0/size", serde_json::json!(0)),
            (
                "/files/0/path",
                serde_json::json!("../noncontiguous-cc.mol"),
            ),
            ("/files/0/name", serde_json::json!("wrong.sdf")),
            ("/files/0/format", serde_json::json!("molv3000")),
        ] {
            let mut invalid = original.clone();
            *invalid.pointer_mut(pointer).ok_or("invalid field")? = value;
            directory.write_manifest(3, &invalid)?;
            assert!(
                super::check_at(&directory.0, true, &mut Vec::new()).is_err(),
                "{pointer}"
            );
            assert!(!directory.0.join(".cache").exists());
        }
        let mut many = original.clone();
        many["files"] = serde_json::json!(
            (0..1021)
                .map(|index| {
                    let mut item = original["files"][0].clone();
                    item["name"] = serde_json::json!(format!("v3000-{index}.mol"));
                    item
                })
                .collect::<Vec<_>>()
        );
        directory.write_manifest(3, &many)?;
        assert!(
            super::check_at(&directory.0, true, &mut Vec::new())
                .err()
                .ok_or("fourth bundle bypassed total bounds")?
                .to_string()
                .contains("1024 total fixtures")
        );
        assert!(!directory.0.join(".cache").exists());
        fs::write(
            directory.0.join(super::MANIFESTS[3]),
            vec![b' '; super::MAX_MANIFEST_BYTES + 1],
        )?;
        assert!(
            super::check_at(&directory.0, true, &mut Vec::new())
                .err()
                .ok_or("unbounded fourth manifest")?
                .to_string()
                .contains("exceeds 1 MiB")
        );
        assert!(!directory.0.join(".cache").exists());
        Ok(())
    }

    #[test]
    fn explicit_v3000_only_inputs_count_as_implemented_parser_coverage()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = CorpusDirectory::new()?;
        let original: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../docs/corpus/bondline-v3000-manifest.json"
        ))?;
        let cache = directory.0.join(".cache/corpus");
        fs::create_dir_all(&cache)?;
        for index in 0..super::MANIFESTS.len() {
            let mut bundle = original.clone();
            let name = format!("only-v3000-{index}.mol");
            bundle["files"][0]["name"] = serde_json::json!(name);
            directory.write_manifest(index, &bundle)?;
            fs::write(
                cache.join(name),
                include_bytes!("../../crates/io/tests/fixtures/v3000/noncontiguous-cc.mol"),
            )?;
        }
        let mut output = Vec::new();
        super::check_at(&directory.0, true, &mut output)?;
        let output = String::from_utf8(output)?;
        assert!(
            output
                .contains("MOL V3000 original-vector byte/source/literal/Decimal/RDKit checks: 4")
        );
        assert!(output.contains("MOL V2000 byte/source/RDKit/Decimal checks: 0"));
        assert!(output.contains(
            "chemistry calculations (formula/mass/aromaticity/stereo/SMILES/InChI) MISSING"
        ));
        Ok(())
    }

    #[test]
    fn new_cdx_bundle_pins_the_public_original_chemical_scene()
    -> Result<(), Box<dyn std::error::Error>> {
        let manifest: Manifest = serde_json::from_slice(include_bytes!(
            "../../docs/corpus/bondline-cdx-manifest.json"
        ))?;
        validate(&manifest)?;
        assert_eq!(manifest.repository, "thematheusmello/bondline");
        assert_eq!(
            manifest.revision,
            "30ae30e73a4ee3b68d41069376fc5690faa99bc4"
        );
        assert_eq!(manifest.files.len(), 1);
        let fixture = manifest.files.first().ok_or("CDX fixture")?;
        assert_eq!(fixture.license, "MIT OR Apache-2.0");
        assert_eq!(fixture.size, 65750);
        verify(
            fixture,
            include_bytes!("../../crates/cdx/tests/fixtures/original-ethane-explicit.cdx"),
        )?;
        Ok(())
    }

    #[test]
    fn new_sdf_bundle_retains_original_license_and_real_published_source_pin()
    -> Result<(), Box<dyn std::error::Error>> {
        let manifest: Manifest =
            serde_json::from_slice(include_bytes!("../../docs/corpus/bondline-manifest.json"))?;
        validate(&manifest)?;
        assert_eq!(manifest.repository, "thematheusmello/bondline");
        assert_eq!(
            manifest.revision,
            "fb86832a0c9b7744fb3ef8f8115c7ed296164505"
        );
        assert_eq!(manifest.files.len(), 1);
        let fixture = manifest.files.first().ok_or("SDF fixture")?;
        assert_eq!(fixture.license, "MIT OR Apache-2.0");
        assert_eq!(fixture.size, 690);
        verify(
            fixture,
            include_bytes!("../../crates/io/tests/fixtures/sdf/ordered-fields-v2000.sdf"),
        )?;
        Ok(())
    }

    #[test]
    fn all_bundles_are_mandatory_and_all_validation_precedes_cache_io()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = CorpusDirectory::new()?;
        let manifests = super::load_manifests(&directory.0)?;
        assert_eq!(manifests.len(), 4);
        assert_eq!(
            manifests
                .iter()
                .map(|bundle| bundle.files.len())
                .sum::<usize>(),
            5
        );
        assert_eq!(
            format!(
                "{:x}",
                sha2::Sha256::digest(include_bytes!("../../docs/corpus/manifest.json"))
            ),
            "820739ca30312745b53df2839f0238b30d1470e73446430b4c954a142cb88ca0"
        );
        assert_eq!(
            format!(
                "{:x}",
                sha2::Sha256::digest(include_bytes!("../../docs/corpus/bondline-manifest.json"))
            ),
            "2a81637144941b3a64e407b07f6caf5c93bd02718059ac5548b96f11de9d77b9"
        );
        assert_eq!(
            format!(
                "{:x}",
                sha2::Sha256::digest(include_bytes!(
                    "../../docs/corpus/bondline-cdx-manifest.json"
                ))
            ),
            "6f8105c4c6cfc3d45e0ccaa406ba0d779240dd9ae8a66464a88b0512f08f2713"
        );
        let original: serde_json::Value =
            serde_json::from_slice(include_bytes!("../../docs/corpus/bondline-manifest.json"))?;
        for (field, value) in [
            ("revision", serde_json::json!("main")),
            ("notice", serde_json::json!("../LICENSE-MIT")),
        ] {
            let mut invalid = original.clone();
            invalid[field] = value;
            directory.write_manifest(1, &invalid)?;
            assert!(super::check_at(&directory.0, true, &mut Vec::new()).is_err());
            assert!(!directory.0.join(".cache").exists());
        }
        let mut invalid = original.clone();
        invalid["files"][0]["license"] = serde_json::json!("ISC");
        directory.write_manifest(1, &invalid)?;
        assert!(super::check_at(&directory.0, true, &mut Vec::new()).is_err());
        assert!(!directory.0.join(".cache").exists());
        directory.write_manifest(1, &original)?;
        fs::write(directory.0.join("LICENSE-MIT"), b"")?;
        assert!(super::check_at(&directory.0, true, &mut Vec::new()).is_err());
        assert!(!directory.0.join(".cache").exists());
        fs::write(
            directory.0.join("LICENSE-MIT"),
            include_bytes!("../../LICENSE-MIT"),
        )?;
        fs::write(
            directory.0.join(super::MANIFESTS[1]),
            vec![b' '; super::MAX_MANIFEST_BYTES + 1],
        )?;
        assert!(
            super::check_at(&directory.0, true, &mut Vec::new())
                .err()
                .ok_or("unbounded manifest")?
                .to_string()
                .contains("exceeds 1 MiB")
        );
        assert!(!directory.0.join(".cache").exists());
        fs::remove_file(directory.0.join(super::MANIFESTS[1]))?;
        assert!(
            super::check_at(&directory.0, true, &mut Vec::new()).is_err(),
            "missing second bundle cannot silently fall back"
        );
        assert!(!directory.0.join(".cache").exists());
        directory.write_manifest(1, &original)?;
        fs::remove_file(directory.0.join(super::MANIFESTS[2]))?;
        assert!(
            super::check_at(&directory.0, true, &mut Vec::new()).is_err(),
            "missing CDX bundle cannot silently fall back"
        );
        assert!(!directory.0.join(".cache").exists());
        fs::write(
            directory.0.join(super::MANIFESTS[2]),
            include_bytes!("../../docs/corpus/bondline-cdx-manifest.json"),
        )?;
        fs::remove_file(directory.0.join(super::MANIFESTS[3]))?;
        assert!(
            super::check_at(&directory.0, true, &mut Vec::new()).is_err(),
            "missing V3000 bundle cannot silently fall back"
        );
        assert!(!directory.0.join(".cache").exists());
        Ok(())
    }

    #[test]
    fn cross_bundle_cache_aliases_and_total_entry_overflow_are_rejected()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = CorpusDirectory::new()?;
        let original: serde_json::Value =
            serde_json::from_slice(include_bytes!("../../docs/corpus/bondline-manifest.json"))?;
        directory.write_manifest(0, &original)?;
        let mut alias = original.clone();
        alias["files"][0]["name"] = serde_json::json!("BONDLINE-ORDERED-FIELDS-V2000.sdf");
        directory.write_manifest(1, &alias)?;
        assert!(
            super::check_at(&directory.0, true, &mut Vec::new())
                .err()
                .ok_or("accepted case alias")?
                .to_string()
                .contains("duplicate corpus cache name")
        );
        assert!(!directory.0.join(".cache").exists());
        let mut many = original.clone();
        many["files"] = serde_json::json!(
            (0..1024)
                .map(|index| {
                    let mut entry = original["files"][0].clone();
                    entry["name"] = serde_json::json!(format!("original-{index}.sdf"));
                    entry
                })
                .collect::<Vec<_>>()
        );
        directory.write_manifest(0, &many)?;
        directory.write_manifest(1, &original)?;
        assert!(
            super::check_at(&directory.0, true, &mut Vec::new())
                .err()
                .ok_or("accepted total overflow")?
                .to_string()
                .contains("1024 total fixtures")
        );
        assert!(!directory.0.join(".cache").exists());
        Ok(())
    }

    #[test]
    fn third_bundle_names_and_limits_are_validated_before_cache_io()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = CorpusDirectory::new()?;
        let original: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../docs/corpus/bondline-cdx-manifest.json"
        ))?;
        directory.write_manifest(0, &original)?;
        let mut alias = original.clone();
        alias["files"][0]["name"] = serde_json::json!("BONDLINE-ORIGINAL-ETHANE-EXPLICIT.cdx");
        directory.write_manifest(2, &alias)?;
        assert!(
            super::check_at(&directory.0, true, &mut Vec::new())
                .err()
                .ok_or("accepted third-bundle alias")?
                .to_string()
                .contains("duplicate corpus cache name")
        );
        assert!(!directory.0.join(".cache").exists());
        fs::write(
            directory.0.join(super::MANIFESTS[0]),
            include_bytes!("../../docs/corpus/manifest.json"),
        )?;
        fs::write(
            directory.0.join(super::MANIFESTS[2]),
            vec![b' '; super::MAX_MANIFEST_BYTES + 1],
        )?;
        assert!(
            super::check_at(&directory.0, true, &mut Vec::new())
                .err()
                .ok_or("unbounded third manifest")?
                .to_string()
                .contains("exceeds 1 MiB")
        );
        assert!(!directory.0.join(".cache").exists());
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn corpus_manifest_and_notice_symlinks_cannot_escape_the_workspace()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = CorpusDirectory::new()?;
        let outside = CorpusDirectory::new()?;
        let second = directory.0.join(super::MANIFESTS[1]);
        fs::remove_file(&second)?;
        std::os::unix::fs::symlink(outside.0.join(super::MANIFESTS[1]), &second)?;
        assert!(super::check_at(&directory.0, true, &mut Vec::new()).is_err());
        assert!(!directory.0.join(".cache").exists());
        fs::remove_file(&second)?;
        fs::write(
            &second,
            include_bytes!("../../docs/corpus/bondline-manifest.json"),
        )?;
        let notice = directory.0.join("LICENSE-MIT");
        fs::remove_file(&notice)?;
        std::os::unix::fs::symlink(outside.0.join("LICENSE-MIT"), &notice)?;
        assert!(super::check_at(&directory.0, true, &mut Vec::new()).is_err());
        assert!(!directory.0.join(".cache").exists());
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn cache_parent_symlink_is_refused_before_creating_outside_directories()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = CorpusDirectory::new()?;
        let outside = CorpusDirectory::new()?;
        std::os::unix::fs::symlink(&outside.0, directory.0.join(".cache"))?;
        assert!(!outside.0.join("corpus").exists());
        assert!(super::check_at(&directory.0, true, &mut Vec::new()).is_err());
        assert!(
            !outside.0.join("corpus").exists(),
            "cache validation created an outside directory"
        );
        Ok(())
    }

    #[test]
    fn offline_runner_counts_sdf_cdx_and_rejects_missing_or_corrupt_cache()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = CorpusDirectory::new()?;
        // Select the same pinned, committed MOL entry so this unit test needs no network/CDXML cache.
        let mut rdkit: serde_json::Value =
            serde_json::from_slice(include_bytes!("../../docs/corpus/manifest.json"))?;
        rdkit["files"] = serde_json::json!([rdkit["files"][1].clone()]);
        directory.write_manifest(0, &rdkit)?;
        let cache = directory.0.join(".cache/corpus");
        fs::create_dir_all(&cache)?;
        fs::write(
            cache.join("rdkit-methyl-radical.mol"),
            include_bytes!("../../crates/io/tests/fixtures/mol/rdkit-CH3.mol"),
        )?;
        assert!(
            super::check_at(&directory.0, true, &mut Vec::new())
                .err()
                .ok_or("missing SDF ignored")?
                .to_string()
                .contains("offline corpus entry is missing")
        );
        let sdf_path = cache.join("bondline-ordered-fields-v2000.sdf");
        fs::write(&sdf_path, b"corrupt")?;
        assert!(
            super::check_at(&directory.0, true, &mut Vec::new())
                .err()
                .ok_or("corrupt SDF accepted")?
                .to_string()
                .contains("length/SHA-256 mismatch")
        );
        fs::write(
            &sdf_path,
            include_bytes!("../../crates/io/tests/fixtures/sdf/ordered-fields-v2000.sdf"),
        )?;
        assert!(
            super::check_at(&directory.0, true, &mut Vec::new())
                .err()
                .ok_or("missing CDX ignored")?
                .to_string()
                .contains("offline corpus entry is missing")
        );
        let cdx_path = cache.join("bondline-original-ethane-explicit.cdx");
        fs::write(&cdx_path, b"corrupt")?;
        assert!(
            super::check_at(&directory.0, true, &mut Vec::new())
                .err()
                .ok_or("corrupt CDX accepted")?
                .to_string()
                .contains("length/SHA-256 mismatch")
        );
        fs::write(
            &cdx_path,
            include_bytes!("../../crates/cdx/tests/fixtures/original-ethane-explicit.cdx"),
        )?;
        assert!(
            super::check_at(&directory.0, true, &mut Vec::new())
                .err()
                .ok_or("missing V3000 ignored")?
                .to_string()
                .contains("offline corpus entry is missing")
        );
        let v3000_path = cache.join("bondline-original-noncontiguous-v3000.mol");
        fs::write(&v3000_path, b"corrupt")?;
        assert!(
            super::check_at(&directory.0, true, &mut Vec::new())
                .err()
                .ok_or("corrupt V3000 accepted")?
                .to_string()
                .contains("length/SHA-256 mismatch")
        );
        fs::write(
            &v3000_path,
            include_bytes!("../../crates/io/tests/fixtures/v3000/noncontiguous-cc.mol"),
        )?;
        let mut output = Vec::new();
        super::check_at(&directory.0, true, &mut output)?;
        let output = String::from_utf8(output)?;
        assert!(output.contains("corpus integrity: 4 pinned licensed files from 4 source bundles"));
        assert!(output.contains("MOL V2000 byte/source/RDKit/Decimal checks: 1"));
        assert!(
            output
                .contains("MOL V3000 original-vector byte/source/literal/Decimal/RDKit checks: 1")
        );
        assert!(
            output.contains("SDF V2000 byte/ordered-field/per-record RDKit checks: 1; records: 2")
        );
        assert!(output.contains("CDX original explicit field-vector byte/struct/RDKit checks: 1"));
        assert!(output.contains("CDX foreign/general/application compatibility MISSING"));
        assert!(output.contains(
            "chemistry calculations (formula/mass/aromaticity/stereo/SMILES/InChI) MISSING"
        ));
        Ok(())
    }

    #[test]
    fn cdx_original_checker_rejects_changed_spans_opaque_payload_bond_and_explicit_scene()
    -> Result<(), Box<dyn std::error::Error>> {
        let source = include_bytes!("../../crates/cdx/tests/fixtures/original-ethane-explicit.cdx");
        let document = bondline_cdx::Document::parse(source)?;
        let table = super::cdx_table(&document);
        assert_eq!(document.records().len(), 29);
        super::check_cdx(source)?;
        let wrong_table = table.replacen("start 32768 0 - 28 34", "start 32768 0 - 29 34", 1);
        assert_ne!(wrong_table, table);
        assert!(super::compare_cdx(source, &wrong_table).is_err());
        for (tag, payload) in [
            (0x4003, b"atom\0\xfe".as_slice()),
            (0x0600, b"\x02\0".as_slice()),
        ] {
            let property=document.records().iter().find(|record|matches!(record.kind(),bondline_cdx::RecordKind::Property{tag:actual,..} if *actual==tag)).ok_or("target property")?;
            let altered = document.replace_property(property, payload)?;
            let altered_document = bondline_cdx::Document::parse(&altered)?;
            assert_eq!(
                super::cdx_table(&altered_document),
                table,
                "same-length changes still have the same valid binary record layout"
            );
            if tag == 0x0600 {
                super::python_output(
                    include_str!("../oracles/cdx_oracle.py"),
                    &["verify", "2", "explicit"],
                    &altered,
                )?;
            }
            assert!(
                super::compare_cdx(&altered, &table).is_err(),
                "original scene guard accepted changed tag{tag}"
            );
        }
        let mut different_explicit = source.to_vec();
        different_explicit
            .get_mut(30..34)
            .ok_or("root ID span")?
            .copy_from_slice(&1_u32.to_le_bytes());
        assert_eq!(
            bondline_cdx::Document::parse(&different_explicit)?.framing(),
            bondline_cdx::Framing::ExplicitDocument
        );
        assert!(super::check_cdx(&different_explicit).is_err());
        Ok(())
    }

    #[test]
    fn sdf_verifier_rejects_order_data_range_and_model_disagreements()
    -> Result<(), Box<dyn std::error::Error>> {
        let source = include_bytes!("../../crates/io/tests/fixtures/sdf/ordered-fields-v2000.sdf");
        let file = bondline_io::sdf::SdfFile::parse(source)?;
        let original = super::sdf_table(&file)?;
        assert_eq!(super::compare_sdf(source, &original)?, (2, 1));
        for pointer in [
            "/0/fields/4/value_lines/0",
            "/0/fields/3/dt_number",
            "/0/fields/2/range/0",
            "/1/projection/atoms/0/element",
        ] {
            let mut wrong = original.clone();
            *wrong
                .pointer_mut(pointer)
                .ok_or("expected source/model field")? = serde_json::json!("wrong");
            assert!(super::compare_sdf(source, &wrong).is_err(), "{pointer}");
        }
        let mut reordered = original.clone();
        reordered[0]["fields"]
            .as_array_mut()
            .ok_or("fields")?
            .swap(4, 5);
        assert!(super::compare_sdf(source, &reordered).is_err());
        let mut dropped = original.clone();
        dropped[0]["fields"]
            .as_array_mut()
            .ok_or("fields")?
            .remove(3);
        assert!(super::compare_sdf(source, &dropped).is_err());
        Ok(())
    }

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

    #[test]
    fn pinned_mol_source_and_projection_match_independent_rdkit_decimal()
    -> Result<(), Box<dyn std::error::Error>> {
        let bytes = include_bytes!("../../crates/io/tests/fixtures/mol/rdkit-CH3.mol");
        let manifest: Manifest =
            serde_json::from_slice(include_bytes!("../../docs/corpus/manifest.json"))?;
        let fixture = manifest
            .files
            .iter()
            .find(|file| matches!(file.format, super::Format::Mol))
            .ok_or("missing pinned MOL entry")?;
        assert_eq!(fixture.name, "rdkit-methyl-radical.mol");
        assert_eq!(fixture.size, 170);
        assert_eq!(
            fixture.sha256,
            "afd2352697ca20c0d580c4981fe8bd24da36c4e1661e9d2aee501a40364fd42d"
        );
        verify(fixture, bytes)?;
        super::check_mol(bytes)?;
        let document = bondline_io::mol::MolFile::parse(bytes)?;
        assert_eq!(document.as_bytes(), bytes);
        let transform =
            bondline_io::mol::MolTransform::new(24.0, bondline_model::Point2::new(0.0, 0.0)?)?;
        let fragment = document.to_fragment(transform)?;
        assert_eq!(fragment.atoms().len(), 1);
        assert!(fragment.bonds().is_empty());
        let atom = fragment.atoms().first().ok_or("missing carbon")?;
        assert_eq!(atom.element.get(), 6);
        assert_eq!(atom.explicit_hydrogens, Some(3));
        assert_eq!(atom.radical, bondline_model::Radical::Doublet);
        let point = atom.position.ok_or("missing point")?;
        assert!((point.x_pt() + 29.7).abs() < 1e-10);
        assert!((point.y_pt() + 51.6216).abs() < 1e-10);
        Ok(())
    }

    #[test]
    fn independent_mol_oracle_rejects_wrong_raw_or_model_metadata_and_coordinates()
    -> Result<(), Box<dyn std::error::Error>> {
        let bytes = include_bytes!("../../crates/io/tests/fixtures/mol/rdkit-CH3.mol");
        let document = bondline_io::mol::MolFile::parse(bytes)?;
        let transform =
            bondline_io::mol::MolTransform::new(24.0, bondline_model::Point2::new(0.0, 0.0)?)?;
        let fragment = document.to_fragment(transform)?;
        let original = serde_json::to_value(fragment)?;
        for (field, value) in [
            ("/atoms/0/element", serde_json::json!(8)),
            ("/atoms/0/radical", serde_json::json!("none")),
            ("/atoms/0/explicit_hydrogens", serde_json::json!(2)),
            ("/atoms/0/position/x_pt", serde_json::json!(-28.7)),
        ] {
            let mut wrong = original.clone();
            *wrong.pointer_mut(field).ok_or("missing model field")? = value;
            assert!(
                super::compare_mol(bytes, &document, &wrong).is_err(),
                "{field}"
            );
        }
        let different_source = std::str::from_utf8(bytes)?.replace("-1.2375", "-2.2375");
        let wrong_raw = bondline_io::mol::MolFile::parse(different_source.as_bytes())?;
        assert!(super::compare_mol(bytes, &wrong_raw, &original).is_err());
        Ok(())
    }
}
