//! Existing XML projection API: independent literal source and expected atom data.
use bondline_io::cdxml::ElementalGraph;
use serde_json::json;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const SOURCE: &[u8] = br#"<CDXML><page><fragment id="71"><n id="90" Element="6" p="72 96" NumHydrogens="3" ShowTerminalCarbonLabels="no" ShowNonTerminalCarbonLabels="yes"/><n id="10" Element="6" p="120 96" NumHydrogens="2"/><n id="700" Element="8" p="168 96" NumHydrogens="1"/><b id="800" B="90" E="10" Order="1"/><b id="900" B="700" E="10" Order="1"/></fragment></page><!--original terminal data--></CDXML>"#;

#[test]
fn literal_terminal_no_maps_independently_while_the_complete_xml_stays_exact() -> TestResult {
    let graph = ElementalGraph::parse(SOURCE)?;
    assert_eq!(graph.as_bytes(), SOURCE);
    assert_eq!(graph.to_bytes(), SOURCE);
    let fragment = graph.fragments().first().ok_or("fragment")?;
    assert_eq!((fragment.atoms().len(), fragment.bonds().len()), (3, 2));
    assert_eq!(
        serde_json::to_value(&fragment.atoms()[0])?,
        json!({
            "id":1,"element":6,"position":{"x_pt":72.0,"y_pt":96.0},
            "isotope":null,"charge":0,"radical":"none","explicit_hydrogens":3,
            "show_terminal_carbon_labels":false,"show_non_terminal_carbon_labels":true
        })
    );
    assert_eq!(
        serde_json::to_value(fragment.bonds())?,
        json!([
            {"id":1,"begin":1,"end":2,"order":"single"},
            {"id":2,"begin":3,"end":2,"order":"single"}
        ])
    );
    Ok(())
}

// Expanded cases preserve the exact first1,489-byte XML runtime declaration.
use bondline_io::{
    DocumentError, StoredDocument,
    cdxml::{MappingError, write_document},
    mol::{self, MolTransform, Version, v3000},
    smiles::{RecordSelection, SmilesFileDocument},
};
use bondline_model::{Atom, Bond, Fragment, FragmentId, PageId, PageSize, Point2};
use bondline_render::Cancellation;
use serde_json::Value;
use std::{
    borrow::Cow,
    fs,
    io::Write as _,
    process::{Command, Output, Stdio},
    sync::Arc,
};
#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(
    dead_code,
    reason = "reuse only the existing isolated file-directory helper"
)]
mod support;
const LEGACY: &[u8] = include_bytes!("fixtures/bondline-v0.cdxml");

fn page() -> TestResult<PageId> {
    PageId::new(1).ok_or_else(|| "page".into())
}
fn frame() -> TestResult<MolTransform> {
    Ok(MolTransform::new(1.0, Point2::new(0.0, 0.0)?)?)
}

fn flags(atom: &Atom, terminal: Option<bool>, nonterminal: Option<bool>) -> TestResult<Atom> {
    let mut value = serde_json::to_value(atom)?;
    for (name, flag) in [
        ("show_terminal_carbon_labels", terminal),
        ("show_non_terminal_carbon_labels", nonterminal),
    ] {
        match flag {
            Some(flag) => value[name] = json!(flag),
            None => {
                value.as_object_mut().ok_or("atom")?.remove(name);
            }
        }
    }
    Ok(serde_json::from_value(value)?)
}

fn fragment(
    terminal: Option<bool>,
    nonterminal: Option<bool>,
    element: u8,
) -> TestResult<Fragment> {
    let h = match element {
        6 => 3,
        8 => 1,
        1 => 0,
        _ => return Err("declared element".into()),
    };
    let first: Atom = serde_json::from_value(json!({"id":1,"element":element,
        "position":{"x_pt":72.0,"y_pt":96.0},"isotope":null,"charge":0,
        "radical":"none","explicit_hydrogens":h}))?;
    let second: Atom = serde_json::from_value(json!({"id":2,"element":6,
        "position":{"x_pt":120.0,"y_pt":96.0},"isotope":null,"charge":0,
        "radical":"none","explicit_hydrogens":3}))?;
    Ok(Fragment::new(
        FragmentId::new(1).ok_or("fragment")?,
        vec![flags(&first, terminal, nonterminal)?, second],
        vec![serde_json::from_value::<Bond>(
            json!({"id":1,"begin":2,"end":1,"order":"single"}),
        )?],
    )?)
}

fn stored(fragment: Fragment) -> TestResult<StoredDocument> {
    let mut value = StoredDocument::default();
    value
        .model_mut()
        .append_fragments(page()?, vec![fragment])?;
    Ok(value)
}

fn canonical(terminal: Option<bool>, nonterminal: Option<bool>) -> Vec<u8> {
    let mut attrs = String::new();
    if let Some(flag) = terminal {
        attrs.push_str(if flag {
            " ShowTerminalCarbonLabels=\"yes\""
        } else {
            " ShowTerminalCarbonLabels=\"no\""
        });
    }
    if let Some(flag) = nonterminal {
        attrs.push_str(if flag {
            " ShowNonTerminalCarbonLabels=\"yes\""
        } else {
            " ShowNonTerminalCarbonLabels=\"no\""
        });
    }
    format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?><CDXML CreationProgram=\"Bondline 0.1.0\"><page id=\"1\" DrawingSpace=\"Poster\" Width=\"612\" Height=\"792\"><fragment id=\"2\"><n id=\"3\" NodeType=\"Element\" Element=\"6\" Charge=\"0\" Radical=\"None\" p=\"72 96\" NumHydrogens=\"3\"{attrs}/><n id=\"4\" NodeType=\"Element\" Element=\"6\" Charge=\"0\" Radical=\"None\" p=\"120 96\" NumHydrogens=\"3\"/><b id=\"5\" B=\"4\" E=\"3\" Order=\"1\"/></fragment></page></CDXML>").into_bytes()
}

fn expected_xml(terminal: Option<bool>, nonterminal: Option<bool>) -> Value {
    let mut first = json!({"id":"3","NodeType":"Element","Element":"6","Charge":"0",
        "Radical":"None","p":"72 96","NumHydrogens":"3"});
    if let Some(flag) = terminal {
        first["ShowTerminalCarbonLabels"] = json!(if flag { "yes" } else { "no" });
    }
    if let Some(flag) = nonterminal {
        first["ShowNonTerminalCarbonLabels"] = json!(if flag { "yes" } else { "no" });
    }
    json!([
        ["CDXML",{"CreationProgram":"Bondline 0.1.0"}],
        ["page",{"id":"1","DrawingSpace":"Poster","Width":"612","Height":"792"}],
        ["fragment",{"id":"2"}], ["n",first],
        ["n",{"id":"4","NodeType":"Element","Element":"6","Charge":"0","Radical":"None","p":"120 96","NumHydrogens":"3"}],
        ["b",{"id":"5","B":"4","E":"3","Order":"1"}]
    ])
}

// Original, narrowly scoped property checker: no Bondline parser/writer import.
// Its manual positive/negative controls execute before any generated-file check below.
const PROPERTIES: &str = r#"
import decimal,json,sys,xml.parsers.expat as expat
expected=json.loads(sys.argv[1]); actual=[]
def start(name,attrs):
 for key,value in attrs.items():
  if key in ('id','B','E','Element','Charge','NumHydrogens'):
   n=decimal.Decimal(value)
   assert n.is_finite() and n==n.to_integral_value()
  elif key in ('Width','Height','Order'):
   assert decimal.Decimal(value).is_finite()
  elif key=='p':
   xy=value.split()
   assert len(xy)==2 and all(decimal.Decimal(v).is_finite() for v in xy)
  elif key in ('ShowTerminalCarbonLabels','ShowNonTerminalCarbonLabels'):
   assert value in ('yes','no')
 if 'ShowTerminalCarbonLabels' in attrs and 'ShowNonTerminalCarbonLabels' in attrs:
  assert list(attrs).index('ShowTerminalCarbonLabels')<list(attrs).index('ShowNonTerminalCarbonLabels')
 actual.append([name,attrs])
p=expat.ParserCreate(namespace_separator='|')
p.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
p.StartElementHandler=start
p.Parse(sys.stdin.buffer.read(),True)
assert actual==expected,(actual,expected)
print(json.dumps({'exact_properties':True,'elements':len(actual)},separators=(',',':')))
"#;

fn python(args: &[String], bytes: &[u8]) -> TestResult<Output> {
    let mut child =
        Command::new(std::env::var_os("BONDLINE_PYTHON").ok_or("pinned BONDLINE_PYTHON")?)
            .arg("-B")
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
    child
        .stdin
        .take()
        .ok_or("consumer stdin")?
        .write_all(bytes)?;
    Ok(child.wait_with_output()?)
}

fn properties(bytes: &[u8], expected: &Value) -> TestResult<Output> {
    python(
        &["-c".into(), PROPERTIES.into(), expected.to_string()],
        bytes,
    )
}

fn exact_properties(bytes: &[u8], expected: &Value) -> TestResult {
    let result = properties(bytes, expected)?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&result.stdout)?,
        json!({"exact_properties":true,"elements":6})
    );
    Ok(())
}

fn unchanged_chemical_consumers(bytes: &[u8]) -> TestResult {
    let graph = python(
        &[format!(
            "{}/../../xtask/oracles/cdxml_graph.py",
            env!("CARGO_MANIFEST_DIR")
        )],
        bytes,
    )?;
    assert!(
        graph.status.success(),
        "{}",
        String::from_utf8_lossy(&graph.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&graph.stdout)?,
        json!([{"id":1,"atoms":[
        {"id":1,"element":6,"position":{"x_pt":72.0,"y_pt":96.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":3},
        {"id":2,"element":6,"position":{"x_pt":120.0,"y_pt":96.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":3}],
        "bonds":[{"id":1,"begin":2,"end":1,"order":"single"}]}])
    );
    let chemical = python(
        &[format!(
            "{}/../../xtask/oracles/native_bond_rdkit.py",
            env!("CARGO_MANIFEST_DIR")
        )],
        bytes,
    )?;
    assert!(
        chemical.status.success(),
        "{}",
        String::from_utf8_lossy(&chemical.stderr)
    );
    Ok(())
}

#[test]
fn exact_native_presence_and_attribute_order_do_not_open_opaque_root_text_or_legacy_source()
-> TestResult {
    // Qualify the original checker on manually authored bytes before touching a product writer.
    for terminal in [None, Some(false), Some(true)] {
        for nonterminal in [None, Some(false), Some(true)] {
            exact_properties(
                &canonical(terminal, nonterminal),
                &expected_xml(terminal, nonterminal),
            )?;
        }
    }
    let manual = String::from_utf8(canonical(Some(false), Some(true)))?;
    let expected = expected_xml(Some(false), Some(true));
    for (from, to) in [
        (
            "ShowTerminalCarbonLabels=\"no\"",
            "ShowTerminalCarbonLabels=\"yes\"",
        ),
        (" ShowTerminalCarbonLabels=\"no\"", ""),
        (
            "ShowTerminalCarbonLabels=\"no\"",
            "ShowTerminalCarbonLabels=\"false\"",
        ),
        (
            "ShowTerminalCarbonLabels=\"no\" ShowNonTerminalCarbonLabels=\"yes\"",
            "ShowNonTerminalCarbonLabels=\"yes\" ShowTerminalCarbonLabels=\"no\"",
        ),
        ("NumHydrogens=\"3\"", "NumHydrogens=\"2\""),
        ("p=\"72 96\"", "p=\"72.125 96\""),
        ("B=\"4\" E=\"3\"", "B=\"3\" E=\"4\""),
        ("<CDXML ", "<CDXML Future=\"opaque\" "),
        ("/>", "><t><s>opaque label</s></t></n>"),
    ] {
        assert!(manual.contains(from));
        assert!(
            !properties(manual.replacen(from, to, 1).as_bytes(), &expected)?
                .status
                .success(),
            "property checker accepted {from:?}->{to:?}"
        );
    }

    let legacy = StoredDocument::from_cdxml(LEGACY.into(), PageSize::A4)?;
    assert!(legacy.is_fully_mapped());
    assert_eq!(write_document(legacy.model())?, LEGACY);
    assert!(matches!(legacy.to_cdxml()?, Cow::Borrowed(_)));
    assert_eq!(legacy.to_cdxml()?.as_ref(), LEGACY);
    assert!(
        legacy.model().pages()[0].fragments()[0]
            .atoms()
            .iter()
            .all(|atom| serde_json::to_value(atom)
                .is_ok_and(|value| value.get("show_terminal_carbon_labels").is_none()
                    && value.get("show_non_terminal_carbon_labels").is_none()))
    );

    let directory = support::FileTestDirectory::new()?;
    for terminal in [None, Some(false), Some(true)] {
        for nonterminal in [None, Some(false), Some(true)] {
            let raw = canonical(terminal, nonterminal);
            let mut native = stored(fragment(terminal, nonterminal, 6)?)?;
            native
                .model_mut()
                .set_page_size(page()?, PageSize::new(612.0, 792.0)?)?;
            let before = native.clone();
            let snapshot = native.model_snapshot();
            let actual = native.to_cdxml()?.into_owned();
            assert_eq!(
                actual, raw,
                "canonical byte profile includes prolog/order and no trailing newline"
            );
            exact_properties(&actual, &expected_xml(terminal, nonterminal))?;
            unchanged_chemical_consumers(&actual)?;
            let path = directory.path(&format!("terminal-{terminal:?}-{nonterminal:?}-炭-λ.cdxml"));
            bondline_io::files::save_cdxml_file(&path, &native, false, &Cancellation::default())?;
            assert_eq!(fs::read(&path)?, raw);
            let loaded = StoredDocument::from_cdxml(fs::read(path)?.into(), PageSize::A4)?;
            assert!(loaded.is_fully_mapped());
            assert_eq!(loaded.model(), native.model());
            assert!(matches!(loaded.to_cdxml()?, Cow::Borrowed(_)));
            assert_eq!(loaded.to_cdxml()?.as_ref(), raw);
            assert_eq!(native, before);
            assert!(Arc::ptr_eq(&snapshot, &native.model_snapshot()));
        }
    }
    for invalid in [
        "", "true", "false", "Yes", "NO", "0", "1", " no", "yes ", "future",
    ] {
        let source = String::from_utf8(SOURCE.to_vec())?
            .replacen(
                "ShowTerminalCarbonLabels=\"no\"",
                &format!("ShowTerminalCarbonLabels=\"{invalid}\""),
                1,
            )
            .into_bytes();
        let syntax = bondline_cdxml::Document::parse(&source)?;
        assert_eq!(syntax.as_bytes(), source);
        assert_eq!(syntax.to_bytes(), source);
        assert!(
            matches!(
                ElementalGraph::parse(&source),
                Err(MappingError::InvalidProperty("ShowTerminalCarbonLabels"))
            ),
            "invalid terminal XML value {invalid:?}"
        );
    }
    // Root properties are retained, not silently inherited into every node.
    for flag in ["yes", "no"] {
        let raw = String::from_utf8(canonical(None, None))?
            .replacen(
                "<CDXML ",
                &format!("<CDXML ShowTerminalCarbonLabels=\"{flag}\" "),
                1,
            )
            .into_bytes();
        let root = StoredDocument::from_cdxml(raw.clone().into(), PageSize::A4)?;
        assert!(!root.is_fully_mapped());
        assert_eq!(root.to_cdxml()?.as_ref(), raw);
        assert!(
            root.model().pages()[0].fragments()[0]
                .atoms()
                .iter()
                .all(|atom| serde_json::to_value(atom)
                    .is_ok_and(|value| value.get("show_terminal_carbon_labels").is_none()))
        );
    }
    for raw in [
        manual.replacen("<CDXML ", "<CDXML ShowTerminalCarbonLabels=\"yes\" ", 1),
        manual.replacen("<n ", "<n Future=\"preserve\" ", 1),
        manual.replacen("/>", "><t><s>KEEP ORIGINAL LABEL</s></t></n>", 1),
    ] {
        let bytes = raw.into_bytes();
        let mut guarded = StoredDocument::from_cdxml(bytes.clone().into(), PageSize::A4)?;
        assert!(!guarded.is_fully_mapped());
        assert_eq!(guarded.to_cdxml()?.as_ref(), bytes);
        let previous = guarded.model().pages()[0].fragments()[0].as_ref();
        let mut atoms = previous.atoms().to_vec();
        atoms[0] = flags(&atoms[0], Some(true), Some(true))?;
        let replacement = Fragment::new(previous.id(), atoms, previous.bonds().to_vec())?;
        guarded.model_mut().replace_fragment(page()?, replacement)?;
        assert!(matches!(
            guarded.to_cdxml(),
            Err(DocumentError::UnsupportedEdit)
        ));
        assert_eq!(
            serde_json::to_value(&guarded.model().pages()[0].fragments()[0].atoms()[0])?["explicit_hydrogens"],
            3
        );
    }
    let label_only = b"<CDXML><page><fragment><n id='1' ShowTerminalCarbonLabels='no'><t><s>N</s></t></n></fragment></page></CDXML>";
    assert!(matches!(
        ElementalGraph::parse(label_only),
        Err(MappingError::UnsupportedLabelOnlyAtom)
    ));
    Ok(())
}

#[test]
fn terminal_metadata_cannot_disappear_through_mol_or_retained_smiles_edit_and_save_paths()
-> TestResult {
    const V2: &[u8] = include_bytes!("fixtures/mol/ethane-v2000.mol");
    const V3: &[u8] = include_bytes!("fixtures/v3000/noncontiguous-cc.mol");
    const WHOLE: &[u8] = b"CC opaque\t\0\xffmetadata\r\nCC untouched\n";
    let source = SmilesFileDocument::parse(Arc::from(WHOLE))?;
    let source_owner = source.clone();
    let projected = StoredDocument::from_smiles_records(
        source,
        RecordSelection::All,
        48.0,
        Point2::new(72.0, 72.0)?,
        PageSize::A4,
        &Cancellation::default(),
    )?;
    let before = projected.clone();
    let snapshot = projected.model_snapshot();
    let previous = projected.model().pages()[0].fragments()[0].as_ref();
    let mut probe_atoms = previous.atoms().to_vec();
    probe_atoms[0] = flags(&probe_atoms[0], Some(false), None)?;
    let replacement = Fragment::new(previous.id(), probe_atoms, previous.bonds().to_vec())?;
    let marked = fragment(Some(false), None, 6)?;
    let mol_source = StoredDocument::from_mol(V2.into(), PageSize::A4, frame()?)?;
    let mol_previous = mol_source.model().pages()[0].fragments()[0].as_ref();
    let mut mol_atoms = mol_previous.atoms().to_vec();
    mol_atoms[0] = flags(&mol_atoms[0], Some(false), None)?;
    let mut mol_drift = mol_source.clone();
    mol_drift.model_mut().replace_fragment(
        page()?,
        Fragment::new(mol_previous.id(), mol_atoms, mol_previous.bonds().to_vec())?,
    )?;
    // Evaluate every immutable no-loss seam before asserting, so the baseline exposes
    // both writer and retained-source acceptance without an early writer failure masking it.
    let refusals = (
        mol::write_v2000(&marked, "terminal probe", frame()?)
            .err()
            .map(|e| e.code()),
        v3000::write_v3000(&marked, "terminal probe", frame()?)
            .err()
            .map(|e| e.code()),
        projected
            .with_smiles_fragment(page()?, replacement, &Cancellation::default())
            .err()
            .map(|e| e.code()),
        mol_drift
            .to_mol_as(Version::V2000, frame()?)
            .err()
            .map(|e| e.code()),
    );
    assert_eq!(
        refusals,
        (
            Some("unsupported_content"),
            Some("unsupported_content"),
            Some("unsupported_content"),
            Some("unsupported_content")
        ),
        "V2000/V3000/retained-SMILES/MOL-regeneration must each refuse flag-only loss"
    );
    assert_eq!(projected, before);
    assert!(Arc::ptr_eq(&snapshot, &projected.model_snapshot()));
    assert_eq!(projected.smiles_file()?.as_bytes(), WHOLE);
    assert_eq!(mol_source.export_mol()?, V2);
    let directory = support::FileTestDirectory::new()?;
    let destination = directory.path("retained-destination-炭.mol");
    let absent = directory.path("must-stay-absent-λ.mol");
    const SENTINEL: &[u8] = b"unchanged original destination\0";
    fs::write(&destination, SENTINEL)?;
    for element in [6, 8, 1] {
        let plain = fragment(None, None, element)?;
        assert!(!mol::write_v2000(&plain, "old profile", frame()?)?.is_empty());
        assert!(!v3000::write_v3000(&plain, "old profile", frame()?)?.is_empty());
        for flag in [false, true] {
            let marked = fragment(Some(flag), None, element)?;
            let before = marked.clone();
            let v2 = mol::write_v2000(&marked, "cannot lose terminal datum", frame()?)
                .err()
                .ok_or("V2000 lost terminal flag")?;
            assert_eq!(v2.code(), "unsupported_content");
            assert!(matches!(v2,mol::MolError::Unsupported{feature,..}
                if feature.contains("terminal")&&!feature.contains("non-terminal")));
            let v3 = v3000::write_v3000(&marked, "cannot lose terminal datum", frame()?)
                .err()
                .ok_or("V3000 lost terminal flag")?;
            assert_eq!(v3.code(), "unsupported_content");
            assert!(matches!(v3,v3000::Error::Unsupported{feature,..}
                if feature.contains("terminal")&&!feature.contains("non-terminal")));
            assert_eq!(marked, before);
            let native = stored(marked)?;
            let original = native.clone();
            let owner = native.model_snapshot();
            for version in [Version::V2000, Version::V3000] {
                assert_eq!(
                    native
                        .to_mol_as(version, frame()?)
                        .err()
                        .ok_or("MOL stored export lost terminal flag")?
                        .code(),
                    "unsupported_content"
                );
                for path in [&destination, &absent] {
                    let failure = bondline_io::files::save_mol_file_as(
                        path,
                        &native,
                        true,
                        version,
                        frame()?,
                        &Cancellation::default(),
                    )
                    .err()
                    .ok_or("MOL publisher lost terminal flag")?;
                    assert_eq!(failure.code(), "unsupported_content");
                    assert!(!failure.published());
                    assert_eq!(fs::read(&destination)?, SENTINEL);
                    assert!(!absent.exists());
                    assert_eq!(
                        fs::read_dir(destination.parent().ok_or("directory")?)?.count(),
                        1
                    );
                    assert_eq!(native, original);
                    assert!(Arc::ptr_eq(&owner, &native.model_snapshot()));
                }
            }
        }
    }
    assert_eq!(mol::MolFile::parse(V2)?.as_bytes(), V2);
    assert_eq!(v3000::MolFile::parse(V3)?.as_bytes(), V3);
    for (bytes, version) in [(V2, Version::V2000), (V3, Version::V3000)] {
        let original = StoredDocument::from_mol(bytes.into(), PageSize::A4, frame()?)?;
        let snapshot = original.model_snapshot();
        assert_eq!(original.export_mol()?, bytes);
        assert!(matches!(
            original.to_mol_as(version, frame()?)?,
            Cow::Borrowed(_)
        ));
        for flag in [false, true] {
            let previous = original.model().pages()[0].fragments()[0].as_ref();
            let mut atoms = previous.atoms().to_vec();
            atoms[0] = flags(&atoms[0], Some(flag), None)?;
            let mut drift = original.clone();
            drift.model_mut().replace_fragment(
                page()?,
                Fragment::new(previous.id(), atoms, previous.bonds().to_vec())?,
            )?;
            assert!(drift.source_info().ok_or("source")?.modified);
            assert!(drift.export_mol().is_err());
            assert_eq!(
                drift
                    .to_mol_as(version, frame()?)
                    .err()
                    .ok_or("MOL flag-only drift returned stale source")?
                    .code(),
                "unsupported_content"
            );
            assert_eq!(original.export_mol()?, bytes);
            assert!(Arc::ptr_eq(&snapshot, &original.model_snapshot()));
        }
    }

    let noop =
        projected.with_smiles_fragment(page()?, previous.clone(), &Cancellation::default())?;
    assert_eq!(noop, projected);
    assert!(Arc::ptr_eq(&snapshot, &noop.model_snapshot()));
    assert_eq!(
        noop.smiles_file()?.as_bytes().as_ptr(),
        projected.smiles_file()?.as_bytes().as_ptr()
    );
    for flag in [false, true] {
        let mut atoms = previous.atoms().to_vec();
        atoms[0] = flags(&atoms[0], Some(flag), None)?;
        let replacement = Fragment::new(previous.id(), atoms, previous.bonds().to_vec())?;
        assert_eq!(
            projected
                .with_smiles_fragment(page()?, replacement.clone(), &Cancellation::default())
                .err()
                .ok_or("notation cannot encode terminal label metadata")?
                .code(),
            "unsupported_content"
        );
        assert_eq!(projected, before);
        assert!(Arc::ptr_eq(&snapshot, &projected.model_snapshot()));
        assert_eq!(projected.smiles_file()?.as_bytes(), WHOLE);
        assert_eq!(
            projected.smiles_file()?.as_bytes().as_ptr(),
            source_owner.as_bytes().as_ptr()
        );
        let mut drift = projected.clone();
        drift.model_mut().replace_fragment(page()?, replacement)?;
        assert!(!drift.has_smiles_projection());
        assert_eq!(
            drift
                .smiles_file()
                .err()
                .ok_or("drift gained whole-file Save authority")?
                .code(),
            "unsupported_content"
        );
        assert!(matches!(
            bondline_io::display::drawing_page_scene(
                1024,
                1024,
                &drift,
                page()?,
                &Cancellation::default()
            ),
            Err(bondline_io::display::DisplayError::IncompleteSource)
        ));
        assert_eq!(projected.smiles_file()?.as_bytes(), WHOLE);
        assert_eq!(source_owner.as_bytes(), WHOLE);
    }
    Ok(())
}
