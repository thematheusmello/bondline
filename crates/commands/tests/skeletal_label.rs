//! Original bent propane source; same-count explicit labeling must restore its display datum.
use bondline_commands::Engine;
use bondline_io::StoredDocument;
use bondline_model::PageSize;
use serde_json::json;

const NATIVE_V: &[u8] = br#"<?xml version="1.0" encoding="UTF-8"?><CDXML CreationProgram="Bondline 0.1.0"><page id="1" DrawingSpace="Poster" Width="612" Height="792"><fragment id="2"><n id="3" NodeType="Element" Element="6" Charge="0" Radical="None" p="120 120" NumHydrogens="2" ShowNonTerminalCarbonLabels="no"/><n id="4" NodeType="Element" Element="6" Charge="0" Radical="None" p="72 168" NumHydrogens="3"/><n id="5" NodeType="Element" Element="6" Charge="0" Radical="None" p="168 168" NumHydrogens="3"/><b id="6" B="4" E="3" Order="1"/><b id="7" B="3" E="5" Order="1"/></fragment></page></CDXML>"#;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn same_literal_label_restores_explicit_false_as_one_authored_edit() -> TestResult {
    let stored = StoredDocument::from_cdxml(NATIVE_V.into(), PageSize::A4)?;
    assert!(
        stored.is_fully_mapped(),
        "original input must be canonical native source"
    );
    assert_eq!(stored.to_cdxml()?.as_ref(), NATIVE_V);
    let mut engine = Engine::from_document(stored);
    let before = serde_json::to_value(engine.document()?)?;
    let mut expected = before.clone();
    expected["pages"][0]["fragments"][0]["atoms"][0]["show_non_terminal_carbon_labels"] =
        json!(true);
    let result = engine.dispatch(
        "atom.set_label",
        json!({"expected_revision":0,
        "page_id":1,"fragment_id":1,"atom_id":1,"element":6,"explicit_hydrogens":2}),
    )?;
    assert!(
        result.changed,
        "same-count false-to-true restoration is a real edit, not a no-op"
    );
    assert_eq!(engine.state().revision, 1);
    assert_eq!(
        (engine.state().undo_depth, engine.state().redo_depth),
        (1, 0)
    );
    assert!(engine.state().file.dirty);
    assert_eq!(serde_json::to_value(engine.document()?)?, expected);
    assert_ne!(expected, before);
    Ok(())
}

// Additive command/authority declarations; the corrected first declaration stays byte-exact.
use bondline_commands::CloseState;
use serde_json::Value;
use std::{
    fs,
    path::Path,
    process::{Command as Process, Stdio},
    sync::Arc,
};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(
    dead_code,
    reason = "reuse only the existing confined temporary directory"
)]
mod support;

type Case<T> = Result<T, Box<dyn std::error::Error>>;
const NAME: &str = "atom.set_label";

fn native(flag: Option<bool>) -> Vec<u8> {
    let source = String::from_utf8_lossy(NATIVE_V);
    match flag {
        None => source
            .replace(" ShowNonTerminalCarbonLabels=\"no\"", "")
            .into_bytes(),
        Some(false) => NATIVE_V.to_vec(),
        Some(true) => source
            .replace(
                "ShowNonTerminalCarbonLabels=\"no\"",
                "ShowNonTerminalCarbonLabels=\"yes\"",
            )
            .into_bytes(),
    }
}
fn seeded(flag: Option<bool>) -> Case<Engine> {
    let source = native(flag);
    let stored = StoredDocument::from_cdxml(source.clone().into(), PageSize::A4)?;
    assert!(stored.is_fully_mapped());
    assert_eq!(stored.to_cdxml()?.as_ref(), source);
    Ok(Engine::from_document(stored))
}
fn label(engine: &Engine, element: u8, h: u16) -> Value {
    json!({"expected_revision":engine.state().revision,"page_id":1,
        "fragment_id":1,"atom_id":1,"element":element,"explicit_hydrogens":h})
}
fn model(engine: &Engine) -> Case<Value> {
    Ok(serde_json::to_value(engine.document()?)?)
}
fn fail(engine: &mut Engine, params: Value, code: &str) -> TestResult {
    let before = serde_json::to_value(engine.state())?;
    let content = engine.content().clone();
    let stamp = engine.capture_stamp();
    let file = engine.ui_file_context_stamp();
    let owner = engine
        .stored_document()
        .ok()
        .map(StoredDocument::model_snapshot);
    let error = engine
        .dispatch(NAME, params)
        .err()
        .ok_or("invalid label restoration accepted")?;
    assert_eq!(error.info().code, code, "{error}");
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert_eq!(engine.content(), &content);
    assert!(engine.capture_stamp_matches(&stamp));
    assert!(engine.ui_file_context_stamp_matches(&file));
    if let Some(owner) = owner {
        assert!(Arc::ptr_eq(
            &owner,
            &engine.stored_document()?.model_snapshot()
        ));
    }
    Ok(())
}

#[test]
fn legacy_none_and_explicit_true_noops_preserve_redo_close_source_and_arc_after_validation()
-> TestResult {
    let registry = bondline_commands::registry();
    assert_eq!(registry.len(), 55, "restoration adds no command");
    assert_eq!(
        registry
            .iter()
            .map(|v| v.name)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        registry.len()
    );
    let schema = serde_json::to_value(
        &registry
            .iter()
            .find(|v| v.name == NAME)
            .ok_or("label schema")?
            .input_schema,
    )?;
    let fields = [
        "expected_revision",
        "page_id",
        "fragment_id",
        "atom_id",
        "element",
        "explicit_hydrogens",
    ]
    .into_iter()
    .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(schema["additionalProperties"], false);
    assert_eq!(
        schema["properties"]
            .as_object()
            .ok_or("properties")?
            .keys()
            .map(String::as_str)
            .collect::<std::collections::BTreeSet<_>>(),
        fields
    );
    assert_eq!(
        schema["required"]
            .as_array()
            .ok_or("required")?
            .iter()
            .map(|v| v.as_str().ok_or("field"))
            .collect::<Result<std::collections::BTreeSet<_>, _>>()?,
        fields
    );
    for flag in [None, Some(true)] {
        let mut engine = seeded(flag)?;
        engine.dispatch(NAME, json!({"expected_revision":0,"page_id":1,"fragment_id":1,"atom_id":3,"element":8,"explicit_hydrogens":1}))?;
        engine.dispatch("history.undo", json!({}))?;
        assert_eq!(engine.state().redo_depth, 1);
        engine.dispatch(
            "document.close",
            json!({"expected_revision":engine.state().revision,"action":"request"}),
        )?;
        assert_eq!(engine.state().close, CloseState::Approved);
        let before = serde_json::to_value(engine.state())?;
        let source = engine.stored_document()?.export_cdxml()?.as_ptr();
        let owner = engine.stored_document()?.model_snapshot();
        let stamp = engine.capture_stamp();
        let file = engine.ui_file_context_stamp();
        let result = engine.dispatch(NAME, label(&engine, 6, 2))?;
        assert!(!result.changed);
        assert_eq!(serde_json::to_value(engine.state())?, before);
        assert_eq!(engine.stored_document()?.export_cdxml()?.as_ptr(), source);
        assert!(Arc::ptr_eq(
            &owner,
            &engine.stored_document()?.model_snapshot()
        ));
        assert!(engine.capture_stamp_matches(&stamp));
        assert!(engine.ui_file_context_stamp_matches(&file));
        let target = &model(&engine)?["pages"][0]["fragments"][0]["atoms"][0];
        match flag {
            None => assert!(target.get("show_non_terminal_carbon_labels").is_none()),
            Some(v) => assert_eq!(target["show_non_terminal_carbon_labels"], v),
        }
        fail(
            &mut engine,
            json!({"expected_revision":0,"page_id":0,"fragment_id":0,"atom_id":0,"element":0,"explicit_hydrogens":2}),
            "stale_revision",
        )?;
        let mut invalid = label(&engine, 6, 2);
        invalid["show_non_terminal_carbon_labels"] = json!(false);
        fail(&mut engine, invalid, "invalid_params")?;
        assert_eq!(engine.state().redo_depth, 1);
    }
    Ok(())
}

#[test]
fn restoration_history_and_heteroatom_entry_keep_other_flags_topology_coordinates_and_literal_h_exact()
-> TestResult {
    let source = String::from_utf8(NATIVE_V.to_vec())?
        .replacen(
            "NumHydrogens=\"3\"/>",
            "NumHydrogens=\"3\" ShowNonTerminalCarbonLabels=\"no\"/>",
            1,
        )
        .replacen(
            "p=\"168 168\" NumHydrogens=\"3\"/>",
            "p=\"168 168\" NumHydrogens=\"3\" ShowNonTerminalCarbonLabels=\"yes\"/>",
            1,
        );
    let stored = StoredDocument::from_cdxml(source.as_bytes().into(), PageSize::A4)?;
    assert!(stored.is_fully_mapped());
    let mut engine = Engine::from_document(stored);
    let original = engine.stored_document()?.clone();
    let original_owner = engine.stored_document()?.model_snapshot();
    let mut expected = model(&engine)?;
    expected["pages"][0]["fragments"][0]["atoms"][0]["show_non_terminal_carbon_labels"] =
        json!(true);
    let changed = engine.dispatch(NAME, label(&engine, 6, 2))?;
    assert!(changed.changed);
    assert_eq!(
        (changed.revision, changed.undo_depth, changed.redo_depth),
        (1, 1, 0)
    );
    assert_eq!(model(&engine)?, expected);
    let restored_owner = engine.stored_document()?.model_snapshot();
    engine.dispatch("history.undo", json!({}))?;
    assert_eq!(engine.stored_document()?, &original);
    assert!(Arc::ptr_eq(
        &original_owner,
        &engine.stored_document()?.model_snapshot()
    ));
    assert_eq!(
        engine.stored_document()?.to_cdxml()?.as_ref(),
        source.as_bytes()
    );
    engine.dispatch("history.redo", json!({}))?;
    assert_eq!(model(&engine)?, expected);
    assert!(Arc::ptr_eq(
        &restored_owner,
        &engine.stored_document()?.model_snapshot()
    ));
    let before = serde_json::to_value(engine.state())?;
    let owner = engine.stored_document()?.model_snapshot();
    assert!(!engine.dispatch(NAME, label(&engine, 6, 2))?.changed);
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert!(Arc::ptr_eq(
        &owner,
        &engine.stored_document()?.model_snapshot()
    ));
    expected["pages"][0]["fragments"][0]["atoms"][0]["element"] = json!(8);
    expected["pages"][0]["fragments"][0]["atoms"][0]["explicit_hydrogens"] = json!(0);
    assert!(engine.dispatch(NAME, label(&engine, 8, 0))?.changed);
    assert_eq!(
        model(&engine)?,
        expected,
        "true persists as dormant oxygen metadata; no neighbor repair"
    );
    assert_eq!(engine.state().undo_depth, 2);
    Ok(())
}

#[test]
fn false_restoration_keeps_existing_error_priority_metadata_and_committed_close_guards()
-> TestResult {
    let mut engine = seeded(Some(false))?;
    engine.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":700,"height_pt":800}),
    )?;
    engine.dispatch("history.undo", json!({}))?;
    for (field, value, code) in [
        ("expected_revision", json!(0), "stale_revision"),
        ("page_id", json!(0), "invalid_params"),
        ("fragment_id", json!(0), "invalid_params"),
        ("atom_id", json!(0), "invalid_params"),
        ("page_id", json!(9), "not_found"),
        ("fragment_id", json!(9), "not_found"),
        ("atom_id", json!(9), "not_found"),
        ("element", json!(0), "invalid_params"),
        ("element", json!(119), "invalid_params"),
        ("explicit_hydrogens", json!(65536), "invalid_params"),
        ("extra", json!(true), "invalid_params"),
    ] {
        let mut params = label(&engine, 6, 2);
        params[field] = value;
        fail(&mut engine, params, code)?;
    }
    let params = label(&engine, 1, 1);
    fail(&mut engine, params, "invalid_params")?;
    assert_eq!(engine.state().redo_depth, 1);
    for (from, to) in [
        ("Charge=\"0\"", "Charge=\"1\""),
        ("Radical=\"None\"", "Radical=\"Doublet\""),
        (
            "p=\"120 120\" NumHydrogens",
            "p=\"120 120\" Isotope=\"13\" NumHydrogens",
        ),
    ] {
        let source = String::from_utf8(NATIVE_V.to_vec())?.replacen(from, to, 1);
        let stored = StoredDocument::from_cdxml(source.as_bytes().into(), PageSize::A4)?;
        assert!(
            stored.is_fully_mapped(),
            "metadata vector must reach target guard"
        );
        let mut guarded = Engine::from_document(stored);
        let params = label(&guarded, 6, 2);
        fail(&mut guarded, params, "unsupported_content")?;
        assert_eq!(
            guarded.stored_document()?.to_cdxml()?.as_ref(),
            source.as_bytes()
        );
    }
    let mut committed = seeded(Some(false))?;
    committed.dispatch(
        "document.close",
        json!({"expected_revision":0,"action":"request"}),
    )?;
    assert!(committed.complete_close());
    fail(
        &mut committed,
        json!({"expected_revision":0,"page_id":0,"fragment_id":0,"atom_id":0,"element":0,"explicit_hydrogens":2}),
        "close_committed",
    )?;
    Ok(())
}

#[test]
fn label_restoration_never_grants_opaque_foreign_or_retained_smiles_source_authority() -> TestResult
{
    let directory = support::FileTestDirectory::new()?;
    let opaque = String::from_utf8(NATIVE_V.to_vec())?.replacen(
        "<CDXML ",
        "<CDXML Future=\"keep-whole-source\" ",
        1,
    );
    let mut engine = Engine::from_document(StoredDocument::from_cdxml(
        opaque.as_bytes().into(),
        PageSize::A4,
    )?);
    assert!(!engine.stored_document()?.is_fully_mapped());
    let params = label(&engine, 6, 2);
    fail(&mut engine, params, "unsupported_content")?;
    assert_eq!(engine.stored_document()?.export_cdxml()?, opaque.as_bytes());
    for (name, bytes, command) in [
        (
            "source.mol",
            include_bytes!("../../io/tests/fixtures/mol/ethane-v2000.mol").as_slice(),
            "document.open_mol",
        ),
        (
            "source.cdx",
            include_bytes!("../../io/tests/fixtures/cdx/metadata-explicit.cdx").as_slice(),
            "document.open_cdx",
        ),
        (
            "source.sdf",
            include_bytes!("../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf").as_slice(),
            "document.open_sdf",
        ),
        (
            "source.smi",
            b"CCC whole\t\0\xffmetadata\r\nCC keep\n".as_slice(),
            "document.open_smiles",
        ),
    ] {
        let path = directory.path(name);
        fs::write(&path, bytes)?;
        let mut foreign = Engine::default();
        foreign.dispatch(command, json!({"path":path}))?;
        let params = label(&foreign, 6, 2);
        fail(&mut foreign, params, "unsupported_content")?;
        assert_eq!(fs::read(&path)?, bytes);
        if command == "document.open_smiles" {
            foreign.dispatch("document.import_smiles_records",json!({"expected_revision":foreign.state().revision,"selection":{"mode":"record","record_id":1},"bond_length_pt":64,"origin_x_pt":96,"origin_y_pt":96,"page_width_pt":612,"page_height_pt":792}))?;
            assert!(foreign.stored_document()?.has_smiles_projection());
            let params = label(&foreign, 6, 3);
            fail(&mut foreign, params, "unsupported_content")?;
            assert_eq!(foreign.stored_document()?.smiles_file()?.as_bytes(), bytes);
        }
    }
    Ok(())
}

fn capture(engine: &mut Engine, backend: &str) -> Case<String> {
    let before = serde_json::to_value(engine.state())?;
    let content = engine.content().clone();
    let owner = engine.stored_document()?.model_snapshot();
    let stamp = engine.capture_stamp();
    let result = engine.dispatch(
        "render.capture",
        json!({"page_id":1,"width_px":1024,"height_px":1024,"backend":backend}),
    )?;
    assert!(!result.changed);
    let capture = result.capture.ok_or("capture missing")?;
    if backend == "gpu" {
        let adapter = capture.adapter.as_ref().ok_or("actual GPU adapter")?;
        assert!(!adapter.name.is_empty() && !adapter.backend.is_empty());
    } else {
        assert!(capture.adapter.is_none());
    }
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert_eq!(engine.content(), &content);
    assert!(engine.capture_stamp_matches(&stamp));
    assert!(Arc::ptr_eq(
        &owner,
        &engine.stored_document()?.model_snapshot()
    ));
    Ok(capture.png_base64)
}
fn consume_file(path: &Path, flag: bool) -> TestResult {
    let output=Process::new(std::env::var_os("BONDLINE_PYTHON").ok_or("pinned BONDLINE_PYTHON")?).args(["-B","-c",r#"
import decimal,json,subprocess,sys,xml.parsers.expat as expat
from rdkit import Chem,rdBase
from rdkit.Chem import rdMolDescriptors
assert rdBase.rdkitVersion=='2026.09.2'
raw=open(sys.argv[1],'rb').read(1048577)
assert 0<len(raw)<=1048576
flag='yes' if sys.argv[2]=='true' else 'no'
records=[]
p=expat.ParserCreate(namespace_separator='|');p.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
p.StartElementHandler=lambda name,attrs:records.append((name,attrs))
p.Parse(raw,True)
assert [n for n,_ in records]==['CDXML','page','fragment','n','n','n','b','b']
nodes=[a for n,a in records if n=='n'];bonds=[a for n,a in records if n=='b']
expected=[{'id':str(i),'NodeType':'Element','Element':'6','Charge':'0','Radical':'None','p':p,'NumHydrogens':h} for i,p,h in [(3,'120 120','2'),(4,'72 168','3'),(5,'168 168','3')]]
expected[0]['ShowNonTerminalCarbonLabels']=flag
assert nodes==expected,(nodes,expected)
assert bonds==[{'id':'6','B':'4','E':'3','Order':'1'},{'id':'7','B':'3','E':'5','Order':'1'}]
for a in nodes: assert all(decimal.Decimal(v).is_finite() for v in a['p'].split())
external=subprocess.run([sys.executable,'-B',sys.argv[3]],input=raw,stdout=subprocess.PIPE,stderr=subprocess.PIPE,check=True)
assert json.loads(external.stdout)==[{'id':1,'atoms':[{'id':i,'element':6,'position':{'x_pt':x,'y_pt':y},'isotope':None,'charge':0,'radical':'none','explicit_hydrogens':h} for i,x,y,h in [(1,120.0,120.0,2),(2,72.0,168.0,3),(3,168.0,168.0,3)]],'bonds':[{'id':1,'begin':2,'end':1,'order':'single'},{'id':2,'begin':1,'end':3,'order':'single'}]}]
mols=Chem.MolsFromCDXML(raw.decode('utf8'),sanitize=True,removeHs=False)
assert len(mols)==1
m=mols[0]
assert m.GetNumAtoms()==3 and m.GetNumBonds()==2
assert [a.GetAtomicNum() for a in m.GetAtoms()]==[6,6,6]
assert [a.GetTotalNumHs(includeNeighbors=True) for a in m.GetAtoms()]==[2,3,3]
assert [(b.GetBeginAtomIdx(),b.GetEndAtomIdx(),str(b.GetBondType())) for b in m.GetBonds()]==[(1,0,'SINGLE'),(0,2,'SINGLE')]
assert all(a.GetFormalCharge()==0 and a.GetIsotope()==0 and a.GetNumRadicalElectrons()==0 and a.GetChiralTag()==Chem.ChiralType.CHI_UNSPECIFIED for a in m.GetAtoms())
assert all(b.GetStereo()==Chem.BondStereo.STEREONONE for b in m.GetBonds())
assert rdMolDescriptors.CalcMolFormula(m)=='C3H8' and Chem.MolToSmiles(m)=='CCC'
"#]).arg(path).arg(flag.to_string()).arg(concat!(env!("CARGO_MANIFEST_DIR"),"/../../xtask/oracles/cdxml_graph.py")).stdout(Stdio::piped()).stderr(Stdio::piped()).output()?;
    assert!(
        output.status.success(),
        "actual-file flag/Expat/Decimal/unchanged graph/RDKit checks: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

#[test]
fn unicode_open_capture_restore_save_history_and_reopen_preserve_whole_source_and_chemistry()
-> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("original-骨格-λ.cdxml");
    let output = directory.path("restored-骨格-λ.cdxml");
    fs::write(&input, NATIVE_V)?;
    consume_file(&input, false)?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_cdxml", json!({"path":input}))?;
    let original = engine.stored_document()?.clone();
    let before = model(&engine)?;
    let hidden = capture(&mut engine, "cpu")?;
    assert_eq!(capture(&mut engine, "gpu")?, hidden);
    let result = engine.dispatch(NAME, label(&engine, 6, 2))?;
    assert!(result.changed);
    assert_eq!(
        (result.revision, result.undo_depth, result.redo_depth),
        (2, 1, 0)
    );
    let mut expected = before;
    expected["pages"][0]["fragments"][0]["atoms"][0]["show_non_terminal_carbon_labels"] =
        json!(true);
    assert_eq!(model(&engine)?, expected);
    let labeled = capture(&mut engine, "cpu")?;
    assert_eq!(capture(&mut engine, "gpu")?, labeled);
    assert_ne!(
        hidden, labeled,
        "actual central glyph/mask change must change image ink"
    );
    engine.dispatch("document.save_cdxml", json!({"path":output}))?;
    consume_file(&output, true)?;
    let saved = fs::read(&output)?;
    assert_ne!(saved, NATIVE_V);
    assert_eq!(fs::read(&input)?, NATIVE_V);
    assert!(!engine.state().file.dirty);
    assert_eq!(
        engine.state().file.path.as_deref(),
        fs::canonicalize(&output)?.to_str()
    );
    engine.dispatch("history.undo", json!({}))?;
    assert_eq!(engine.stored_document()?, &original);
    assert!(engine.state().file.dirty);
    engine.dispatch("history.redo", json!({}))?;
    assert_eq!(model(&engine)?, expected);
    assert!(!engine.state().file.dirty);
    engine.dispatch("document.open_cdxml", json!({"path":output}))?;
    assert_eq!(model(&engine)?, expected);
    assert!(!engine.state().file.dirty);
    assert_eq!(
        (engine.state().undo_depth, engine.state().redo_depth),
        (0, 0)
    );
    assert!(
        !engine
            .stored_document()?
            .source_info()
            .ok_or("source")?
            .modified
    );
    assert_eq!(engine.stored_document()?.to_cdxml()?.as_ref(), saved);
    assert_eq!(capture(&mut engine, "cpu")?, labeled);
    assert_eq!(fs::read(&input)?, NATIVE_V);
    Ok(())
}

#[test]
fn explicit_label_can_repair_unknown_or_inconsistent_h_without_a_render_or_c4_precondition()
-> TestResult {
    for old_h in [None, Some(1)] {
        for (element, h) in [(6, 2), (8, 1)] {
            let text = String::from_utf8(NATIVE_V.to_vec())?;
            let source = match old_h {
                None => text.replacen(" NumHydrogens=\"2\"", "", 1),
                Some(value) => text.replacen(
                    "NumHydrogens=\"2\"",
                    &format!("NumHydrogens=\"{value}\""),
                    1,
                ),
            };
            let stored = StoredDocument::from_cdxml(source.as_bytes().into(), PageSize::A4)?;
            assert!(stored.is_fully_mapped());
            let mut engine = Engine::from_document(stored);
            let mut expected = model(&engine)?;
            expected["pages"][0]["fragments"][0]["atoms"][0]["element"] = json!(element);
            expected["pages"][0]["fragments"][0]["atoms"][0]["explicit_hydrogens"] = json!(h);
            expected["pages"][0]["fragments"][0]["atoms"][0]["show_non_terminal_carbon_labels"] =
                json!(true);
            let result = engine.dispatch(NAME, label(&engine, element, h))?;
            assert!(result.changed);
            assert_eq!(
                (result.revision, result.undo_depth, result.redo_depth),
                (1, 1, 0)
            );
            assert_eq!(model(&engine)?, expected);
            // These are literal model expectations, not claims that every chosen H has valid valence.
        }
    }
    Ok(())
}
