use bondline_io::cdxml::ElementalGraph;
use serde_json::json;

type TestResult = Result<(), Box<dyn std::error::Error>>;

// Original literal SDK-name vector, not native writer output. Source identifiers
// deliberately differ from the fresh dense application IDs.
const RIGHT_SOURCE: &[u8] = br#"<CDXML Future="retain"><page><fragment id="7"><n id="10" Element="6" p="72 72" NumHydrogens="2"/><n id="20" Element="6" p="168 72" NumHydrogens="2"/><b id="30" B="10" E="20" Order="2" DoublePosition="Right"/></fragment></page><!--opaque--></CDXML>"#;

#[test]
fn literal_sdk_right_position_is_typed_and_all_original_bytes_remain_authoritative() -> TestResult {
    let graph = ElementalGraph::parse(RIGHT_SOURCE)?;
    assert_eq!(graph.as_bytes(), RIGHT_SOURCE);
    assert_eq!(graph.to_bytes(), RIGHT_SOURCE);
    let fragment = graph.fragments().first().ok_or("fragment")?;
    assert_eq!(fragment.id().get(), 1);
    assert_eq!(fragment.atoms().len(), 2);
    let bond = fragment.bonds().first().ok_or("bond")?;
    assert_eq!(
        serde_json::to_value(bond)?,
        json!({
            "id": 1, "begin": 1, "end": 2, "order": "double",
            "double_position": "right"
        })
    );
    Ok(())
}

// Additive declarations below preserve the initial 1,179-byte runtime-RED vector.
use bondline_io::{
    StoredDocument,
    cdxml::{MappingError, write_document},
    mol::{self, MolTransform, Version, v3000},
};
use bondline_model::{Atom, AtomId, Bond, Element, Fragment, FragmentId, PageId, PageSize, Point2};
use bondline_render::Cancellation;
use serde_json::Value;
use std::{
    borrow::Cow,
    fs,
    io::Write as _,
    path::PathBuf,
    process::{Command as Process, Output, Stdio},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

type Out<T> = Result<T, Box<dyn std::error::Error>>;
const LEGACY: &[u8] = include_bytes!("fixtures/bondline-v0.cdxml");
static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

struct Directory(PathBuf);
impl Directory {
    fn new() -> Out<Self> {
        let path = std::env::temp_dir().join(format!(
            "bondline-sided-double-{}-{}",
            std::process::id(),
            NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path)?;
        Ok(Self(path))
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn page() -> Out<PageId> {
    PageId::new(1).ok_or_else(|| "page".into())
}
fn frame() -> Out<MolTransform> {
    Ok(MolTransform::new(1.0, Point2::new(0.0, 0.0)?)?)
}

fn fragment(order: &str, position: Option<&str>, reverse: bool) -> Out<Fragment> {
    let h = match order {
        "single" => 3,
        "double" => 2,
        "triple" => 1,
        _ => return Err("declared order".into()),
    };
    let mut atoms = Vec::new();
    for (id, x) in [(1, 72.125), (2, 168.5)] {
        let mut atom = Atom::new(
            AtomId::new(id).ok_or("atom")?,
            Element::try_from(6)?,
            Some(Point2::new(x, 96.25)?),
        );
        atom.explicit_hydrogens = Some(h);
        atoms.push(atom);
    }
    let mut value =
        json!({"id":1,"begin":if reverse {2} else {1},"end":if reverse {1} else {2},"order":order});
    if let Some(position) = position {
        value["double_position"] = json!(position);
    }
    let bond: Bond = serde_json::from_value(value)?;
    Ok(Fragment::new(
        FragmentId::new(1).ok_or("fragment")?,
        atoms,
        vec![bond],
    )?)
}
fn stored(fragment: Fragment) -> Out<StoredDocument> {
    let mut stored = StoredDocument::default();
    stored
        .model_mut()
        .append_fragments(page()?, vec![fragment])?;
    Ok(stored)
}
fn literal_source(position: Option<&str>, reverse: bool, order: &str) -> Vec<u8> {
    let property = position.map_or_else(String::new, |value| format!(" DoublePosition='{value}'"));
    format!("<CDXML Future='opaque'><page><fragment id='700'><n id='10' Element='6' p='72.125 96.25' NumHydrogens='2'/><n id='20' Element='6' p='168.5 96.25' NumHydrogens='2'/><b id='30' B='{}' E='{}' Order='{order}'{property}/></fragment></page><!--retain--></CDXML>", if reverse {20} else {10}, if reverse {10} else {20}).into_bytes()
}

// Expat compares all attributes as a complete literal table. Decimal checks every
// numeric token independently; no Bondline parser/writer or Rust f64 JSON bridge.
const EXPAT: &str = r#"
import decimal,json,sys,xml.parsers.expat as expat
expected=json.loads(sys.argv[1])
assert isinstance(expected,list) and expected
actual=[]
def start(name,attrs):
    for key,value in attrs.items():
        if key in ('id','B','E','Element','Charge','NumHydrogens'):
            v=decimal.Decimal(value)
            assert v.is_finite() and v==v.to_integral_value()
        elif key in ('Width','Height','Order'):
            assert decimal.Decimal(value).is_finite()
        elif key=='p':
            xy=value.split()
            assert len(xy)==2 and all(decimal.Decimal(v).is_finite() for v in xy)
        elif key=='DoublePosition':
            assert value in ('Center','Right','Left')
    actual.append([name,attrs])
p=expat.ParserCreate(namespace_separator='|')
p.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
p.StartElementHandler=start
p.Parse(sys.stdin.buffer.read(),True)
assert actual==expected,(actual,expected)
print(json.dumps({'exact_attributes':True,'elements':len(actual)},separators=(',',':')))
"#;
fn expat(bytes: &[u8], expected: &Value) -> Out<Output> {
    let mut child =
        Process::new(std::env::var_os("BONDLINE_PYTHON").ok_or("pinned BONDLINE_PYTHON")?)
            .args(["-B", "-c", EXPAT, &expected.to_string()])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
    child.stdin.take().ok_or("Expat stdin")?.write_all(bytes)?;
    Ok(child.wait_with_output()?)
}
fn expected_xml(position: Option<&str>, order: &str, reverse: bool) -> Value {
    let h = match order {
        "1" => "3",
        "2" => "2",
        "3" => "1",
        _ => "undeclared",
    };
    let mut bond = json!({"id":"5","B":if reverse {"4"} else {"3"},"E":if reverse {"3"} else {"4"},"Order":order});
    if let Some(position) = position {
        bond["DoublePosition"] = json!(position);
    }
    json!([
        ["CDXML",{"CreationProgram":"Bondline 0.1.0"}],
        ["page",{"id":"1","DrawingSpace":"Poster","Width":"612","Height":"792"}],
        ["fragment",{"id":"2"}],
        ["n",{"id":"3","NodeType":"Element","Element":"6","Charge":"0","Radical":"None","p":"72.125 96.25","NumHydrogens":h}],
        ["n",{"id":"4","NodeType":"Element","Element":"6","Charge":"0","Radical":"None","p":"168.5 96.25","NumHydrogens":h}],
        ["b",bond]
    ])
}
fn checked_expat(bytes: &[u8], expected: &Value) -> TestResult {
    let output = expat(bytes, expected)?;
    assert!(
        output.status.success(),
        "independent exact Expat/Decimal: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout)?,
        json!({"exact_attributes":true,"elements":6})
    );
    Ok(())
}

#[test]
fn sdk_names_presence_direction_and_dormant_metadata_are_distinct_without_rewriting_source()
-> TestResult {
    for (xml, model) in [
        (Some("Center"), Some("center")),
        (Some("Right"), Some("right")),
        (Some("Left"), Some("left")),
        (None, None),
    ] {
        for reverse in [false, true] {
            for order in ["1", "2"] {
                let source = literal_source(xml, reverse, order);
                let graph = ElementalGraph::parse(&source)?;
                assert_eq!(graph.as_bytes(), source);
                assert_eq!(graph.to_bytes(), source);
                let item = graph.fragments().first().ok_or("fragment")?;
                let mut expected = json!({"id":1,"begin":if reverse {2} else {1},"end":if reverse {1} else {2},"order":if order=="1" {"single"} else {"double"}});
                if let Some(model) = model {
                    expected["double_position"] = json!(model);
                }
                assert_eq!(
                    serde_json::to_value(item.bonds().first().ok_or("bond")?)?,
                    expected
                );
                assert_eq!(item.atoms()[0].position, Some(Point2::new(72.125, 96.25)?));
                assert_eq!(item.atoms()[1].position, Some(Point2::new(168.5, 96.25)?));
                assert!(
                    item.atoms()
                        .iter()
                        .all(|atom| atom.explicit_hydrogens == Some(2))
                );
            }
        }
    }
    Ok(())
}

#[test]
fn invalid_double_position_is_a_typed_mapping_error_while_standalone_syntax_retains_every_byte()
-> TestResult {
    for name in [
        "",
        "right",
        "RIGHT",
        "1",
        "256",
        "Automatic",
        "Unspecified",
        "Left ",
        "Future",
    ] {
        let source = literal_source(Some(name), false, "2");
        let syntax = bondline_cdxml::Document::parse(&source)?;
        assert_eq!(syntax.as_bytes(), source);
        assert_eq!(syntax.to_bytes(), source);
        assert!(
            matches!(
                ElementalGraph::parse(&source),
                Err(MappingError::InvalidProperty("DoublePosition"))
            ),
            "invalid XML name: {name:?}"
        );
    }
    Ok(())
}

#[test]
fn native_writer_properties_and_source_completeness_match_exact_independent_attributes()
-> TestResult {
    let legacy = StoredDocument::from_cdxml(LEGACY.into(), PageSize::A4)?;
    assert!(legacy.is_fully_mapped());
    assert_eq!(write_document(legacy.model())?, LEGACY);
    assert_eq!(legacy.to_cdxml()?.as_ref(), LEGACY);
    for (model, xml) in [
        (None, None),
        (Some("center"), Some("Center")),
        (Some("right"), Some("Right")),
        (Some("left"), Some("Left")),
    ] {
        for reverse in [false, true] {
            let mut native = stored(fragment("double", model, reverse)?)?;
            native
                .model_mut()
                .set_page_size(page()?, PageSize::new(612.0, 792.0)?)?;
            let before = native.clone();
            let bytes = native.to_cdxml()?.into_owned();
            checked_expat(&bytes, &expected_xml(xml, "2", reverse))?;
            assert_eq!(native, before);
            let loaded = StoredDocument::from_cdxml(bytes.clone().into(), PageSize::A4)?;
            assert!(loaded.is_fully_mapped());
            assert_eq!(loaded.model(), native.model());
            assert!(matches!(loaded.to_cdxml()?, Cow::Borrowed(_)));
            assert_eq!(loaded.to_cdxml()?.as_ref(), bytes);
            let foreign = String::from_utf8(bytes)?
                .replacen("<CDXML ", "<CDXML Future='preserve' ", 1)
                .into_bytes();
            let mut guarded = StoredDocument::from_cdxml(foreign.clone().into(), PageSize::A4)?;
            assert!(!guarded.is_fully_mapped());
            assert_eq!(guarded.to_cdxml()?.as_ref(), foreign);
            guarded
                .model_mut()
                .set_page_size(page()?, PageSize::new(700.0, 800.0)?)?;
            assert!(matches!(
                guarded.to_cdxml(),
                Err(bondline_io::DocumentError::UnsupportedEdit)
            ));
            assert_eq!(loaded.model(), native.model());
        }
    }
    Ok(())
}

#[test]
fn independent_attribute_reader_is_qualified_by_original_manual_xml_and_semantic_mutations()
-> TestResult {
    let source = r#"<CDXML CreationProgram="Bondline 0.1.0"><page id="1" DrawingSpace="Poster" Width="612" Height="792"><fragment id="2"><n id="3" NodeType="Element" Element="6" Charge="0" Radical="None" p="72.125 96.25" NumHydrogens="2"/><n id="4" NodeType="Element" Element="6" Charge="0" Radical="None" p="168.5 96.25" NumHydrogens="2"/><b id="5" B="3" E="4" Order="2" DoublePosition="Right"/></fragment></page></CDXML>"#;
    for name in ["Right", "Center", "Left"] {
        let positive = source.replace(
            "DoublePosition=\"Right\"",
            &format!("DoublePosition=\"{name}\""),
        );
        checked_expat(positive.as_bytes(), &expected_xml(Some(name), "2", false))?;
    }
    let expected = expected_xml(Some("Right"), "2", false);
    for (from, to) in [
        ("DoublePosition=\"Right\"", "DoublePosition=\"Left\""),
        ("DoublePosition=\"Right\"", ""),
        ("DoublePosition=\"Right\"", "DoublePosition=\"Automatic\""),
        ("B=\"3\" E=\"4\"", "B=\"4\" E=\"3\""),
        ("Order=\"2\"", "Order=\"1\""),
        ("NumHydrogens=\"2\"", "NumHydrogens=\"3\""),
        ("72.125 96.25", "72.25 96.25"),
        ("<CDXML ", "<CDXML Future=\"hidden\" "),
    ] {
        assert!(source.contains(from));
        let mutation = source.replace(from, to);
        let output = expat(mutation.as_bytes(), &expected)?;
        assert!(
            !output.status.success(),
            "attribute oracle accepted literal mutation {from:?}->{to:?}"
        );
    }
    Ok(())
}

#[test]
fn every_specified_position_including_center_and_dormant_metadata_refuses_both_mol_model_writers()
-> TestResult {
    for order in ["single", "double", "triple"] {
        let plain = fragment(order, None, false)?;
        assert!(!mol::write_v2000(&plain, "literal absence", frame()?)?.is_empty());
        assert!(!v3000::write_v3000(&plain, "literal absence", frame()?)?.is_empty());
        for position in ["center", "right", "left"] {
            let styled = fragment(order, Some(position), false)?;
            let before = styled.clone();
            assert_eq!(
                mol::write_v2000(&styled, "cannot lose placement", frame()?)
                    .err()
                    .ok_or("V2000 dropped placement")?
                    .code(),
                "unsupported_content"
            );
            assert_eq!(
                v3000::write_v3000(&styled, "cannot lose placement", frame()?)
                    .err()
                    .ok_or("V3000 dropped placement")?
                    .code(),
                "unsupported_content"
            );
            assert_eq!(styled, before);
        }
    }
    Ok(())
}

#[test]
fn stored_mol_encoding_and_actual_atomic_files_refuse_before_destination_or_snapshot_changes()
-> TestResult {
    let directory = Directory::new()?;
    let target = directory.0.join("preserved-λ.mol");
    let absent = directory.0.join("absent-λ.mol");
    const SENTINEL: &[u8] = b"original destination bytes\0untouched";
    fs::write(&target, SENTINEL)?;
    for order in ["single", "double"] {
        for position in ["center", "right", "left"] {
            let native = stored(fragment(order, Some(position), false)?)?;
            let before = native.clone();
            let snapshot = native.model_snapshot();
            for version in [Version::V2000, Version::V3000] {
                assert_eq!(
                    native
                        .to_mol_as(version, frame()?)
                        .err()
                        .ok_or("stored encoding dropped metadata")?
                        .code(),
                    "unsupported_content"
                );
                for path in [&target, &absent] {
                    let error = bondline_io::files::save_mol_file_as(
                        path,
                        &native,
                        true,
                        version,
                        frame()?,
                        &Cancellation::default(),
                    )
                    .err()
                    .ok_or("file encoding dropped metadata")?;
                    assert_eq!(error.code(), "unsupported_content");
                    assert!(!error.published());
                    assert_eq!(fs::read(&target)?, SENTINEL);
                    assert!(!absent.exists());
                    assert_eq!(
                        fs::read_dir(&directory.0)?.count(),
                        1,
                        "no partial temporary output"
                    );
                    assert_eq!(native, before);
                    assert!(Arc::ptr_eq(&snapshot, &native.model_snapshot()));
                }
            }
        }
    }
    Ok(())
}

#[test]
fn placement_is_not_an_authorized_retained_smiles_replacement_and_raw_drift_loses_save_display_authority()
-> TestResult {
    use bondline_io::smiles::{RecordSelection, SmilesFileDocument};
    const WHOLE: &[u8] = b"C=C opaque\t\0\xffmetadata\r\nCC untouched\n";
    let file = SmilesFileDocument::parse(Arc::from(WHOLE))?;
    let native = StoredDocument::from_smiles_records(
        file,
        RecordSelection::All,
        64.0,
        Point2::new(96.0, 96.0)?,
        PageSize::A4,
        &Cancellation::default(),
    )?;
    assert!(native.has_smiles_projection());
    let before = native.clone();
    let snapshot = native.model_snapshot();
    let previous = native.model().pages()[0].fragments()[0].as_ref();
    for position in ["center", "right", "left"] {
        let mut bonds = previous.bonds().to_vec();
        let mut value = serde_json::to_value(bonds.first().ok_or("mapped bond")?)?;
        value["double_position"] = json!(position);
        bonds[0] = serde_json::from_value(value)?;
        let replacement = Fragment::new(previous.id(), previous.atoms().to_vec(), bonds)?;
        let error = native
            .with_smiles_fragment(page()?, replacement.clone(), &Cancellation::default())
            .err()
            .ok_or("SMILES replacement silently lost placement")?;
        assert_eq!(error.code(), "unsupported_content");
        assert_eq!(native, before);
        assert!(Arc::ptr_eq(&snapshot, &native.model_snapshot()));
        assert_eq!(native.smiles_file()?.as_bytes(), WHOLE);
        let mut drift = native.clone();
        drift.model_mut().replace_fragment(page()?, replacement)?;
        assert!(!drift.has_smiles_projection());
        assert_eq!(
            drift
                .smiles_file()
                .err()
                .ok_or("raw drift gained Save authority")?
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
        assert_eq!(native.smiles_file()?.as_bytes(), WHOLE);
        assert!(Arc::ptr_eq(&snapshot, &native.model_snapshot()));
    }
    Ok(())
}

#[test]
fn raw_sdf_rxn_wrappers_and_cdx_auto_manual_payloads_remain_opaque_source_controls() -> TestResult {
    use bondline_io::{
        rxn::{self, ComponentInput, Role, RxnFile},
        sdf::{self, SdfFile, SdfRecordInput},
    };
    const MOL: &[u8] = include_bytes!("fixtures/mol/ethane-v2000.mol");
    const SDF: &[u8] = include_bytes!("fixtures/sdf/mixed-v2000-v3000.sdf");
    const RXN: &[u8] = include_bytes!("fixtures/rxn/mixed-mol-v2000-wrapper.rxn");
    assert_eq!(SdfFile::parse(SDF)?.as_bytes(), SDF);
    assert_eq!(RxnFile::parse(RXN)?.as_bytes(), RXN);
    let wrapped = sdf::write(&[SdfRecordInput {
        mol_bytes: MOL,
        items: &[],
    }])?;
    assert_eq!(wrapped, [MOL, b"$$$$\n"].concat());
    assert_eq!(SdfFile::parse(&wrapped)?.records()[0].mol_bytes(), MOL);
    let reaction = rxn::write_v2000(
        ["Original literal wrapper", "Opaque payload", ""],
        &[ComponentInput {
            role: Role::Reactant,
            mol_bytes: MOL,
        }],
    )?;
    assert_eq!(
        reaction,
        [
            b"$RXN\nOriginal literal wrapper\nOpaque payload\n\n  1  0  0\n$MOL\n".as_slice(),
            MOL
        ]
        .concat()
    );
    assert_eq!(RxnFile::parse(&reaction)?.components()[0].mol_bytes(), MOL);
    // Literal syntax-only object/property table. No molecule/model authority is claimed.
    for value in [0_u16, 1, 2, 256, 257, 258] {
        let mut source = b"VjCD0100\x04\x03\x02\x01".to_vec();
        source.resize(28, 0);
        source.extend_from_slice(&[0, 128, 0, 0, 0, 0, 5, 128, 1, 0, 0, 0, 3, 6, 2, 0]);
        source.extend_from_slice(&value.to_le_bytes());
        source.extend_from_slice(&[0, 0, 0, 0]);
        let syntax = bondline_cdx::Document::parse(&source)?;
        let property = syntax
            .records()
            .iter()
            .find(|record| {
                matches!(
                    record.kind(),
                    bondline_cdx::RecordKind::Property { tag: 0x0603, .. }
                )
            })
            .ok_or("opaque placement property")?;
        assert_eq!(syntax.payload(property)?, value.to_le_bytes());
        assert_eq!(syntax.as_bytes(), source);
        assert_eq!(syntax.to_bytes(), source);
        assert!(
            bondline_io::cdx::ElementalGraph::parse(&source).is_err(),
            "syntax-only metadata is not new CDX chemistry authority"
        );
    }
    Ok(())
}
