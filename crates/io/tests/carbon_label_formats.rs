use bondline_io::cdxml::ElementalGraph;
use serde_json::json;

type TestResult = Result<(), Box<dyn std::error::Error>>;

// Original three-carbon literal, with a degree-two target and non-dense source IDs.
// Explicit no must remain present data, independent of future skeletal rendering.
const EXPLICIT_NO: &[u8] = br#"<CDXML Future="keep"><page><fragment id="71"><n id="90" Element="6" p="100 100" NumHydrogens="2" ShowNonTerminalCarbonLabels="no"/><n id="10" Element="6" p="60 100" NumHydrogens="3"/><n id="700" Element="6" p="140 100" NumHydrogens="3"/><b id="800" B="10" E="90" Order="1"/><b id="900" B="90" E="700" Order="1"/></fragment></page><!--original--></CDXML>"#;

#[test]
fn literal_node_explicit_no_is_typed_false_while_complete_source_stays_exact() -> TestResult {
    let graph = ElementalGraph::parse(EXPLICIT_NO)?;
    assert_eq!(graph.as_bytes(), EXPLICIT_NO);
    assert_eq!(graph.to_bytes(), EXPLICIT_NO);
    let fragment = graph.fragments().first().ok_or("fragment")?;
    assert_eq!(fragment.id().get(), 1);
    assert_eq!((fragment.atoms().len(), fragment.bonds().len()), (3, 2));
    let target = fragment.atoms().first().ok_or("target")?;
    assert_eq!(
        serde_json::to_value(target)?,
        json!({
            "id":1,"element":6,"position":{"x_pt":100.0,"y_pt":100.0},
            "isotope":null,"charge":0,"radical":"none","explicit_hydrogens":2,
            "show_non_terminal_carbon_labels":false
        })
    );
    Ok(())
}

// Additive independent declarations preserve the complete initial runtime vector.
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
            "bondline-carbon-label-data-{}-{}",
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
fn with_flag(atom: &Atom, flag: bool) -> Out<Atom> {
    let mut value = serde_json::to_value(atom)?;
    value["show_non_terminal_carbon_labels"] = json!(flag);
    Ok(serde_json::from_value(value)?)
}
fn fragment(flag: Option<bool>, terminal: bool, target_element: u8) -> Out<Fragment> {
    let mut atoms = Vec::new();
    for (id, x, h) in [(1, 100.0, 2), (2, 60.0, 3), (3, 140.0, 3)] {
        let mut atom = Atom::new(
            AtomId::new(id).ok_or("atom")?,
            Element::try_from(6)?,
            Some(Point2::new(x, 100.0)?),
        );
        atom.explicit_hydrogens = Some(h);
        atoms.push(atom);
    }
    let target = if terminal { 1 } else { 0 };
    atoms[target].element = Element::try_from(target_element)?;
    atoms[target].explicit_hydrogens = Some(match target_element {
        6 => {
            if terminal {
                3
            } else {
                2
            }
        }
        8 => {
            if terminal {
                1
            } else {
                0
            }
        }
        1 => 0,
        _ => return Err("declared target element".into()),
    });
    if let Some(flag) = flag {
        atoms[target] = with_flag(&atoms[target], flag)?;
    }
    let bonds = [[2, 1], [1, 3]]
        .into_iter()
        .enumerate()
        .map(|(index, [begin, end])| {
            serde_json::from_value::<Bond>(
                json!({"id":index+1,"begin":begin,"end":end,"order":"single"}),
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Fragment::new(
        FragmentId::new(1).ok_or("fragment")?,
        atoms,
        bonds,
    )?)
}
fn stored(fragment: Fragment) -> Out<StoredDocument> {
    let mut value = StoredDocument::default();
    value
        .model_mut()
        .append_fragments(page()?, vec![fragment])?;
    Ok(value)
}
fn literal_source(flag: Option<&str>, root_extra: &str, node_extra: &str, text: bool) -> Vec<u8> {
    let flag = flag.map_or_else(String::new, |flag| {
        format!(" ShowNonTerminalCarbonLabels='{flag}'")
    });
    let middle = if text {
        format!(
            "<n id='90' Element='6' p='100 100' NumHydrogens='2'{flag}{node_extra}><t><s>KEEP THIS LABEL</s></t></n>"
        )
    } else {
        format!("<n id='90' Element='6' p='100 100' NumHydrogens='2'{flag}{node_extra}/>")
    };
    format!("<CDXML{root_extra}><page><fragment id='71'>{middle}<n id='10' Element='6' p='60 100' NumHydrogens='3'/><n id='700' Element='6' p='140 100' NumHydrogens='3'/><b id='800' B='10' E='90' Order='1'/><b id='900' B='90' E='700' Order='1'/></fragment></page><!--retain--></CDXML>").into_bytes()
}
const EXPAT: &str = r#"
import decimal,json,sys,xml.parsers.expat as expat
expected=json.loads(sys.argv[1])
actual=[]
def start(name,attrs):
 for key,value in attrs.items():
  if key in ('id','B','E','Element','Charge','NumHydrogens'):
   d=decimal.Decimal(value)
   assert d.is_finite() and d==d.to_integral_value()
  elif key in ('Width','Height','Order'):
   assert decimal.Decimal(value).is_finite()
  elif key=='p':
   v=value.split()
   assert len(v)==2 and all(decimal.Decimal(n).is_finite() for n in v)
  elif key=='ShowNonTerminalCarbonLabels':
   assert value in ('yes','no')
 actual.append([name,attrs])
p=expat.ParserCreate(namespace_separator='|')
p.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
p.StartElementHandler=start
p.Parse(sys.stdin.buffer.read(),True)
assert actual==expected,(actual,expected)
print(json.dumps({'exact_attributes':True,'elements':len(actual)},separators=(',',':')))
"#;
fn expected_xml(flag: Option<bool>) -> Value {
    let mut target = json!({"id":"3","NodeType":"Element","Element":"6","Charge":"0","Radical":"None","p":"100 100","NumHydrogens":"2"});
    if let Some(flag) = flag {
        target["ShowNonTerminalCarbonLabels"] = json!(if flag { "yes" } else { "no" });
    }
    json!([
        ["CDXML",{"CreationProgram":"Bondline 0.1.0"}],
        ["page",{"id":"1","DrawingSpace":"Poster","Width":"612","Height":"792"}],
        ["fragment",{"id":"2"}],
        ["n",target],
        ["n",{"id":"4","NodeType":"Element","Element":"6","Charge":"0","Radical":"None","p":"60 100","NumHydrogens":"3"}],
        ["n",{"id":"5","NodeType":"Element","Element":"6","Charge":"0","Radical":"None","p":"140 100","NumHydrogens":"3"}],
        ["b",{"id":"6","B":"4","E":"3","Order":"1"}],
        ["b",{"id":"7","B":"3","E":"5","Order":"1"}]
    ])
}
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
fn checked_expat(bytes: &[u8], expected: &Value) -> TestResult {
    let output = expat(bytes, expected)?;
    assert!(
        output.status.success(),
        "independent Expat/Decimal: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout)?,
        json!({"exact_attributes":true,"elements":8})
    );
    Ok(())
}

#[test]
fn exact_node_yes_no_and_absence_preserve_presence_chemical_counts_and_directed_topology()
-> TestResult {
    for (flag, expected) in [
        (None, None),
        (Some("no"), Some(false)),
        (Some("yes"), Some(true)),
    ] {
        let source = literal_source(flag, "", "", false);
        let graph = ElementalGraph::parse(&source)?;
        assert_eq!(graph.as_bytes(), source);
        assert_eq!(graph.to_bytes(), source);
        let fragment = &graph.fragments()[0];
        let atoms = serde_json::to_value(fragment.atoms())?;
        let mut target = json!({"id":1,"element":6,"position":{"x_pt":100.0,"y_pt":100.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":2});
        if let Some(expected) = expected {
            target["show_non_terminal_carbon_labels"] = json!(expected);
        }
        assert_eq!(atoms[0], target);
        assert!(atoms[1].get("show_non_terminal_carbon_labels").is_none());
        assert!(atoms[2].get("show_non_terminal_carbon_labels").is_none());
        assert_eq!(atoms[1]["explicit_hydrogens"], 3);
        assert_eq!(atoms[2]["explicit_hydrogens"], 3);
        assert_eq!(
            serde_json::to_value(fragment.bonds())?,
            json!([
                {"id":1,"begin":2,"end":1,"order":"single"},
                {"id":2,"begin":1,"end":3,"order":"single"}
            ])
        );
    }
    // Presence is retained even where the flag is dormant; this is data, not display.
    for element in [1, 8] {
        let source = String::from_utf8(literal_source(Some("no"), "", "", false))?
            .replacen("Element='6'", &format!("Element='{element}'"), 1)
            .into_bytes();
        let graph = ElementalGraph::parse(&source)?;
        let atom = serde_json::to_value(&graph.fragments()[0].atoms()[0])?;
        assert_eq!(atom["element"], element);
        assert_eq!(
            atom["explicit_hydrogens"], 2,
            "dormant data must not repair literal H"
        );
        assert_eq!(atom["show_non_terminal_carbon_labels"], false);
        assert_eq!(graph.to_bytes(), source);
    }
    Ok(())
}

#[test]
fn malformed_node_values_refuse_typed_mapping_but_standalone_xml_retains_every_byte() -> TestResult
{
    for flag in [
        "", "false", "true", "No", "YES", "0", "1", "no ", " yes", "future",
    ] {
        let source = literal_source(Some(flag), "", "", false);
        let syntax = bondline_cdxml::Document::parse(&source)?;
        assert_eq!(syntax.as_bytes(), source);
        assert_eq!(syntax.to_bytes(), source);
        assert!(
            matches!(
                ElementalGraph::parse(&source),
                Err(MappingError::InvalidProperty("ShowNonTerminalCarbonLabels"))
            ),
            "flag {flag:?}"
        );
    }
    Ok(())
}

#[test]
fn root_unknown_and_explicit_text_remain_backed_without_inheritance_or_new_regeneration_authority()
-> TestResult {
    for root in [
        " ShowNonTerminalCarbonLabels='yes'",
        " ShowNonTerminalCarbonLabels='no'",
    ] {
        let source = literal_source(None, root, "", false);
        let graph = ElementalGraph::parse(&source)?;
        assert!(graph.fragments()[0].atoms().iter().all(|atom| {
            serde_json::to_value(atom)
                .is_ok_and(|value| value.get("show_non_terminal_carbon_labels").is_none())
        }));
        assert_eq!(graph.to_bytes(), source);
    }
    let label_only=b"<CDXML><page><fragment><n id='1' ShowNonTerminalCarbonLabels='no'><t><s>N</s></t></n></fragment></page></CDXML>";
    assert!(matches!(
        ElementalGraph::parse(label_only),
        Err(MappingError::UnsupportedLabelOnlyAtom)
    ));
    for flag in [false, true] {
        let mut native = stored(fragment(Some(flag), false, 6)?)?;
        native
            .model_mut()
            .set_page_size(page()?, PageSize::new(612.0, 792.0)?)?;
        let canonical = String::from_utf8(write_document(native.model())?)?;
        for source in [
            canonical.replacen("<CDXML ", "<CDXML ShowNonTerminalCarbonLabels=\"yes\" ", 1),
            canonical.replacen("<n ", "<n Future=\"preserve\" ", 1),
            canonical.replacen("/>", "><t><s>KEEP THIS LABEL</s></t></n>", 1),
        ] {
            let bytes = source.into_bytes();
            let mut guarded = StoredDocument::from_cdxml(bytes.clone().into(), PageSize::A4)?;
            assert!(!guarded.is_fully_mapped());
            assert_eq!(guarded.to_cdxml()?.as_ref(), bytes);
            assert_eq!(
                serde_json::to_value(&guarded.model().pages()[0].fragments()[0].atoms()[0])?["show_non_terminal_carbon_labels"],
                flag
            );
            guarded
                .model_mut()
                .set_page_size(page()?, PageSize::new(700.0, 800.0)?)?;
            assert!(matches!(
                guarded.to_cdxml(),
                Err(bondline_io::DocumentError::UnsupportedEdit)
            ));
        }
    }
    Ok(())
}

#[test]
fn native_node_attribute_writer_proof_and_old_absent_profile_are_independently_exact() -> TestResult
{
    let old = StoredDocument::from_cdxml(LEGACY.into(), PageSize::A4)?;
    assert!(old.is_fully_mapped());
    assert_eq!(write_document(old.model())?, LEGACY);
    assert_eq!(old.to_cdxml()?.as_ref(), LEGACY);
    for flag in [None, Some(false), Some(true)] {
        let mut native = stored(fragment(flag, false, 6)?)?;
        native
            .model_mut()
            .set_page_size(page()?, PageSize::new(612.0, 792.0)?)?;
        let before = native.clone();
        let bytes = native.to_cdxml()?.into_owned();
        checked_expat(&bytes, &expected_xml(flag))?;
        if let Some(flag) = flag {
            let marker = format!(
                " NumHydrogens=\"2\" ShowNonTerminalCarbonLabels=\"{}\"/>",
                if flag { "yes" } else { "no" }
            );
            assert!(
                std::str::from_utf8(&bytes)?.contains(&marker),
                "new attribute follows all existing atom attributes"
            );
        }
        let loaded = StoredDocument::from_cdxml(bytes.clone().into(), PageSize::A4)?;
        assert!(loaded.is_fully_mapped());
        assert_eq!(loaded.model(), native.model());
        assert!(matches!(loaded.to_cdxml()?, Cow::Borrowed(_)));
        assert_eq!(loaded.to_cdxml()?.as_ref(), bytes);
        assert_eq!(native, before);
    }
    Ok(())
}

#[test]
fn exact_attribute_consumer_is_qualified_on_original_manual_xml_and_semantic_negatives()
-> TestResult {
    let manual = r#"<CDXML CreationProgram="Bondline 0.1.0"><page id="1" DrawingSpace="Poster" Width="612" Height="792"><fragment id="2"><n id="3" NodeType="Element" Element="6" Charge="0" Radical="None" p="100 100" NumHydrogens="2" ShowNonTerminalCarbonLabels="no"/><n id="4" NodeType="Element" Element="6" Charge="0" Radical="None" p="60 100" NumHydrogens="3"/><n id="5" NodeType="Element" Element="6" Charge="0" Radical="None" p="140 100" NumHydrogens="3"/><b id="6" B="4" E="3" Order="1"/><b id="7" B="3" E="5" Order="1"/></fragment></page></CDXML>"#;
    checked_expat(manual.as_bytes(), &expected_xml(Some(false)))?;
    checked_expat(
        manual
            .replace(
                "ShowNonTerminalCarbonLabels=\"no\"",
                "ShowNonTerminalCarbonLabels=\"yes\"",
            )
            .as_bytes(),
        &expected_xml(Some(true)),
    )?;
    checked_expat(
        manual
            .replace(" ShowNonTerminalCarbonLabels=\"no\"", "")
            .as_bytes(),
        &expected_xml(None),
    )?;
    let expected = expected_xml(Some(false));
    for (from, to) in [
        (
            "ShowNonTerminalCarbonLabels=\"no\"",
            "ShowNonTerminalCarbonLabels=\"yes\"",
        ),
        (" ShowNonTerminalCarbonLabels=\"no\"", ""),
        (
            "ShowNonTerminalCarbonLabels=\"no\"",
            "ShowNonTerminalCarbonLabels=\"false\"",
        ),
        ("NumHydrogens=\"2\"", "NumHydrogens=\"1\""),
        ("100 100", "100.125 100"),
        ("B=\"4\" E=\"3\"", "B=\"3\" E=\"4\""),
        ("<CDXML ", "<CDXML Future=\"hidden\" "),
        ("/>", "><t><s>invented</s></t></n>"),
    ] {
        assert!(manual.contains(from));
        let bytes = manual.replacen(from, to, 1).into_bytes();
        assert!(
            !expat(&bytes, &expected)?.status.success(),
            "consumer accepted mutation {from:?}->{to:?}"
        );
    }
    Ok(())
}

#[test]
fn specified_yes_no_and_dormant_flags_refuse_mol_model_and_atomic_file_loss() -> TestResult {
    let directory = Directory::new()?;
    let target = directory.0.join("keep-炭-λ.mol");
    let absent = directory.0.join("absent-炭-λ.mol");
    const SENTINEL: &[u8] = b"original destination\0unchanged";
    fs::write(&target, SENTINEL)?;
    for (terminal, element) in [(false, 6), (true, 6), (true, 8), (true, 1)] {
        let plain = fragment(None, terminal, element)?;
        assert!(!mol::write_v2000(&plain, "legacy control", frame()?)?.is_empty());
        assert!(!v3000::write_v3000(&plain, "legacy control", frame()?)?.is_empty());
        for flag in [false, true] {
            let value = fragment(Some(flag), terminal, element)?;
            let before = value.clone();
            assert_eq!(
                mol::write_v2000(&value, "must preserve flag", frame()?)
                    .err()
                    .ok_or("V2000 lost flag")?
                    .code(),
                "unsupported_content"
            );
            assert_eq!(
                v3000::write_v3000(&value, "must preserve flag", frame()?)
                    .err()
                    .ok_or("V3000 lost flag")?
                    .code(),
                "unsupported_content"
            );
            assert_eq!(value, before);
            let native = stored(value)?;
            let before = native.clone();
            let snapshot = native.model_snapshot();
            for version in [Version::V2000, Version::V3000] {
                assert_eq!(
                    native
                        .to_mol_as(version, frame()?)
                        .err()
                        .ok_or("stored MOL lost flag")?
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
                    .ok_or("file MOL lost flag")?;
                    assert_eq!(error.code(), "unsupported_content");
                    assert!(!error.published());
                    assert_eq!(fs::read(&target)?, SENTINEL);
                    assert!(!absent.exists());
                    assert_eq!(fs::read_dir(&directory.0)?.count(), 1);
                    assert_eq!(native, before);
                    assert!(Arc::ptr_eq(&snapshot, &native.model_snapshot()));
                }
            }
        }
    }
    // An imported source cannot take an unchanged-source shortcut after flag-only drift.
    const MOL: &[u8] = include_bytes!("fixtures/mol/ethane-v2000.mol");
    let original = StoredDocument::from_mol(MOL.into(), PageSize::A4, frame()?)?;
    assert_eq!(original.export_mol()?, MOL);
    for flag in [false, true] {
        let previous = original.model().pages()[0].fragments()[0].as_ref();
        let mut atoms = previous.atoms().to_vec();
        atoms[0] = with_flag(&atoms[0], flag)?;
        let replacement = Fragment::new(previous.id(), atoms, previous.bonds().to_vec())?;
        let mut drift = original.clone();
        drift.model_mut().replace_fragment(page()?, replacement)?;
        assert!(drift.source_info().ok_or("source info")?.modified);
        assert!(drift.export_mol().is_err());
        assert_eq!(
            drift
                .to_mol_as(Version::V2000, frame()?)
                .err()
                .ok_or("old MOL source returned stale bytes")?
                .code(),
            "unsupported_content"
        );
        assert_eq!(original.export_mol()?, MOL);
    }
    Ok(())
}

#[test]
fn retained_smiles_replacement_and_detached_drift_cannot_synchronize_unencoded_flag_data()
-> TestResult {
    use bondline_io::smiles::{RecordSelection, SmilesFileDocument};
    const WHOLE: &[u8] = b"CCC opaque\t\0\xffmetadata\r\nCC untouched\n";
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
    for flag in [false, true] {
        let mut atoms = previous.atoms().to_vec();
        atoms[1] = with_flag(&atoms[1], flag)?;
        let replacement = Fragment::new(previous.id(), atoms, previous.bonds().to_vec())?;
        assert_eq!(
            native
                .with_smiles_fragment(page()?, replacement.clone(), &Cancellation::default())
                .err()
                .ok_or("retained SMILES lost flag")?
                .code(),
            "unsupported_content"
        );
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
        assert_eq!(native.smiles_file()?.as_bytes(), WHOLE);
    }
    Ok(())
}

#[test]
fn raw_cdx_implied_presence_and_sdf_rxn_payloads_remain_separate_opaque_controls() -> TestResult {
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
    // Original syntax-only root/Atom table; an implied flag has a zero-byte payload.
    // Neither a molecule nor an IO projection/automatic-vs-manual bool mapping is claimed.
    for present in [false, true] {
        let mut source = b"VjCD0100\x04\x03\x02\x01".to_vec();
        source.resize(28, 0);
        source.extend_from_slice(&[0, 128, 0, 0, 0, 0, 4, 128, 1, 0, 0, 0]);
        if present {
            source.extend_from_slice(&[0x43, 0x04, 0, 0]);
        }
        source.extend_from_slice(&[0, 0, 0, 0]);
        let syntax = bondline_cdx::Document::parse(&source)?;
        let properties = syntax
            .records()
            .iter()
            .filter(|record| {
                matches!(
                    record.kind(),
                    bondline_cdx::RecordKind::Property { tag: 0x0443, .. }
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(properties.len(), usize::from(present));
        if let Some(property) = properties.first() {
            assert!(syntax.payload(property)?.is_empty());
        }
        assert_eq!(syntax.as_bytes(), source);
        assert_eq!(syntax.to_bytes(), source);
        assert!(bondline_io::cdx::ElementalGraph::parse(&source).is_err());
    }
    Ok(())
}
