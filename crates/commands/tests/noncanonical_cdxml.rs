#![cfg(not(target_arch = "wasm32"))]
use bondline_commands::Engine;
use serde_json::json;
use std::fs;

#[test]
fn preserved_explicit_fragment_smiles_export_declares_context_loss_without_saving_the_session()
-> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("preserved-fragment-λ.cdxml");
    let output = directory.path("preserved-fragment-λ.smi");
    fs::write(&input, SOURCE)?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_cdxml", json!({"path":input}))?;
    let before = serde_json::to_value(engine.state())?;
    let stamp = engine.capture_stamp();
    let result = engine.dispatch(
        "file.export_smiles_fragment",
        json!({
            "expected_revision":1,"page_id":1,"fragment_id":1,"output_path":output
        }),
    )?;
    let report = result
        .smiles_export
        .ok_or("explicit graph export receipt")?;
    assert_eq!(report.scope, "selected_fragment_chemical_graph");
    assert_eq!(report.not_encoded, ["geometry", "document_context"]);
    assert_eq!(report.input_revision, 1);
    assert_eq!(fs::read(output)?, b"[CH3][CH3]\n");
    assert_eq!(fs::read(input)?, SOURCE);
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert_eq!(engine.capture_stamp(), stamp);
    assert_eq!(engine.stored_document()?.to_cdxml()?.as_ref(), SOURCE);
    Ok(())
}
#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(
    dead_code,
    reason = "reuse the existing isolated native-file directory"
)]
mod support;
type TestResult = Result<(), Box<dyn std::error::Error>>;

const SOURCE: &[u8] = br#"<?xml version="1.0" encoding="UTF-8"?>
<CDXML CreationProgram="Bondline 0.1.0">
 <page id='91' Width='612.00' Height="792.000" DrawingSpace='Poster'>
  <fragment id="400">
   <b id='900' E='207' Order='1' B='101' />
   <n Radical='None' id='101' p='100.00 200.000' NumHydrogens='3' Charge='0' Element='6' NodeType='Element'/>
   <n Element="6" NumHydrogens="3" NodeType="Element" p="140 200" id="207" Radical="None" Charge="0"></n>
  </fragment>
 </page>
</CDXML>
"#;
const DOUBLE: &[u8] = br#"<?xml version="1.0" encoding="UTF-8"?>
<CDXML CreationProgram="Bondline 0.1.0">
 <page id='91' Width='612.00' Height="792.000" DrawingSpace='Poster'>
  <fragment id="400">
   <b id='900' E='207' Order='2' B='101' />
   <n Radical='None' id='101' p='100.00 200.000' NumHydrogens='2' Charge='0' Element='6' NodeType='Element'/>
   <n Element="6" NumHydrogens="2" NodeType="Element" p="140 200" id="207" Radical="None" Charge="0"></n>
  </fragment>
 </page>
</CDXML>
"#;

#[test]
fn noncanonical_order_edit_preserves_original_ids_and_lexemes_through_real_save_history_reopen()
-> TestResult {
    assert_eq!(SOURCE.len(), 464);
    assert_eq!(DOUBLE.len(), 464);
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("noncanonical-original-λ.cdxml");
    let output = directory.path("noncanonical-double-化学.cdxml");
    fs::write(&input, SOURCE)?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_cdxml", json!({"path":input}))?;
    let original = engine.content().clone();
    assert_eq!(engine.state().revision, 1);
    assert_eq!(engine.state().undo_depth, 0);
    assert!(!engine.state().file.dirty);
    assert_eq!(engine.stored_document()?.to_cdxml()?.as_ref(), SOURCE);
    // Existing public API: old profile guard must refuse here, not at a future method.
    let changed = engine.dispatch("bond.set_order",json!({"expected_revision":1,"page_id":1,"fragment_id":1,"bond_id":1,"order":"double","hydrogen_policy":"rectify_neutral_carbon"}))?;
    assert!(changed.changed);
    assert_eq!(changed.revision, 2);
    assert_eq!(changed.undo_depth, 1);
    assert!(changed.file.dirty);
    assert!(changed.source.as_ref().ok_or("source")?.modified);
    assert_eq!(engine.stored_document()?.to_cdxml()?.as_ref(), DOUBLE);
    assert_eq!(fs::read(&input)?, SOURCE);
    let edited = engine.content().clone();
    let edited_model = serde_json::to_value(engine.state())?["document"].clone();
    let graph = &edited_model["pages"][0]["fragments"][0];
    assert_eq!(graph["id"], 1);
    assert_eq!(graph["atoms"][0]["id"], 1);
    assert_eq!(graph["atoms"][1]["id"], 2);
    assert_eq!(graph["atoms"][0]["explicit_hydrogens"], 2);
    assert_eq!(graph["atoms"][1]["explicit_hydrogens"], 2);
    assert_eq!(
        graph["bonds"],
        json!([{"id":1,"begin":1,"end":2,"order":"double"}])
    );
    engine.dispatch("history.undo", json!({}))?;
    assert_eq!(engine.content(), &original);
    assert_eq!(engine.stored_document()?.to_cdxml()?.as_ref(), SOURCE);
    assert!(!engine.state().file.dirty);
    engine.dispatch("history.redo", json!({}))?;
    assert_eq!(engine.content(), &edited);
    assert_eq!(engine.stored_document()?.to_cdxml()?.as_ref(), DOUBLE);
    let saved = engine.dispatch("document.save_cdxml", json!({"path":output}))?;
    assert_eq!(saved.revision, 4);
    assert!(!saved.file.dirty);
    assert!(saved.source.as_ref().ok_or("original source")?.modified);
    assert_eq!(fs::read(&output)?, DOUBLE);
    assert_eq!(fs::read(&input)?, SOURCE);
    engine.dispatch("document.open_cdxml", json!({"path":output}))?;
    assert_eq!(engine.state().revision, 5);
    assert_eq!(engine.state().undo_depth, 0);
    assert_eq!(engine.state().redo_depth, 0);
    assert!(!engine.state().file.dirty);
    assert!(
        !engine
            .state()
            .source
            .as_ref()
            .ok_or("reopened source")?
            .modified
    );
    assert_eq!(
        serde_json::to_value(engine.state())?["document"],
        edited_model
    );
    assert_eq!(engine.stored_document()?.to_cdxml()?.as_ref(), DOUBLE);
    Ok(())
}

use serde_json::Value;
use std::{
    io::Write,
    path::Path,
    process::{Command as Consumer, Stdio},
};

fn consume_profile(
    path: &Path,
    raw: &[u8],
    h: u8,
    order: &str,
    smiles: &str,
    formula: &str,
) -> TestResult {
    // Raw preservation is checked before a chemical reader which ignores formatting.
    assert_eq!(fs::read(path)?, raw);
    let graph = json!([{"id":1,"atoms":[
        {"id":1,"element":6,"position":{"x_pt":100.0,"y_pt":200.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":h},
        {"id":2,"element":6,"position":{"x_pt":140.0,"y_pt":200.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":h}],
        "bonds":[{"id":1,"begin":1,"end":2,"order":order}]}]);
    let args = serde_json::to_string(
        &json!({"graph":graph,"smiles":smiles,"formula":formula,"flags":[null,null],"double_positions":[null]}),
    )?;
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut process = Consumer::new(python)
        .args([
            "-B",
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/ring_create/named_ring_oracle.py"
            ),
            &args,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    process
        .stdin
        .take()
        .ok_or("independent consumer stdin")?
        .write_all(raw)?;
    let result = process.wait_with_output()?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let facts: Value = serde_json::from_slice(&result.stdout)?;
    assert_eq!(
        facts,
        json!({"atoms":2,"bonds":1,"formula":formula,"literal_integer_orders":true,"literal_direction":true,"literal_flags_and_double_positions":true,"no_assigned_stereo":true})
    );
    Ok(())
}

#[test]
fn manual_vectors_and_actual_saved_order_edit_pass_the_existing_independent_consumer() -> TestResult
{
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("manual-single-λ.cdxml");
    let literal_double = directory.path("manual-double.cdxml");
    let actual = directory.path("actual-double-化学.cdxml");
    fs::write(&input, SOURCE)?;
    fs::write(&literal_double, DOUBLE)?;
    // Qualify the unchanged reader on both independent literal inputs BEFORE our output.
    consume_profile(&input, SOURCE, 3, "single", "CC", "C2H6")?;
    consume_profile(&literal_double, DOUBLE, 2, "double", "C=C", "C2H4")?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_cdxml", json!({"path":input}))?;
    engine.dispatch("bond.set_order",json!({"expected_revision":1,"page_id":1,"fragment_id":1,"bond_id":1,"order":"double","hydrogen_policy":"rectify_neutral_carbon"}))?;
    engine.dispatch("document.save_cdxml", json!({"path":actual}))?;
    consume_profile(&actual, DOUBLE, 2, "double", "C=C", "C2H4")?;
    assert_eq!(fs::read(&input)?, SOURCE);
    Ok(())
}

#[test]
fn opaque_source_refuses_edits_atomically_and_preserved_whole_mol_is_not_canonical_authority()
-> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("preserved-λ.cdxml");
    let opaque = directory.path("opaque.cdxml");
    let copy = directory.path("opaque-copy.cdxml");
    let sentinel = directory.path("keep-unchanged.mol");
    let canonical = directory.path("canonical-化学.cdxml");
    let allowed_mol = directory.path("canonical-λ.mol");
    let unknown = std::str::from_utf8(SOURCE)?.replacen(
        "<CDXML CreationProgram=",
        "<CDXML Future='retained &amp; unknown' CreationProgram=",
        1,
    );
    fs::write(&opaque, unknown.as_bytes())?;
    fs::write(&input, SOURCE)?;
    fs::write(
        &sentinel,
        b"do not clobber this destination
",
    )?;
    let mut blocked = Engine::default();
    blocked.dispatch("document.open_cdxml", json!({"path":opaque}))?;
    assert!(!blocked.stored_document()?.is_fully_mapped());
    let before = serde_json::to_value(blocked.state())?;
    let owner = blocked.content().clone();
    let stamp = blocked.capture_stamp();
    for (name, params) in [
        (
            "bond.set_order",
            json!({"expected_revision":1,"page_id":1,"fragment_id":1,"bond_id":1,"order":"double","hydrogen_policy":"rectify_neutral_carbon"}),
        ),
        (
            "document.save_mol",
            json!({"path":sentinel,"overwrite":true}),
        ),
    ] {
        let error = blocked
            .dispatch(name, params)
            .err()
            .ok_or("opaque source was authorized")?;
        assert_eq!(error.info().code, "unsupported_content");
        assert_eq!(serde_json::to_value(blocked.state())?, before);
        assert_eq!(blocked.content(), &owner);
        assert!(blocked.capture_stamp_matches(&stamp));
        assert_eq!(
            fs::read(&sentinel)?,
            b"do not clobber this destination
"
        );
        assert_eq!(fs::read(&opaque)?, unknown.as_bytes());
    }
    blocked.dispatch("document.save_cdxml", json!({"path":copy}))?;
    assert_eq!(fs::read(&copy)?, unknown.as_bytes());
    // Eliminate old page-layout refusal: this is one ordinary unstyled C2 fragment on A4.
    let mut preserved = Engine::default();
    preserved.dispatch("document.open_cdxml", json!({"path":input}))?;
    let a4 = bondline_model::PageSize::A4;
    preserved.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":a4.width_pt(),"height_pt":a4.height_pt()}),
    )?;
    assert!(preserved.stored_document()?.is_fully_mapped());
    let before = serde_json::to_value(preserved.state())?;
    let owner = preserved.content().clone();
    let stamp = preserved.capture_stamp();
    assert_eq!(
        preserved
            .dispatch(
                "document.save_mol",
                json!({"path":sentinel,"overwrite":true})
            )
            .err()
            .ok_or("Preserved source gained whole-MOL authority")?
            .info()
            .code,
        "unsupported_content"
    );
    assert_eq!(serde_json::to_value(preserved.state())?, before);
    assert_eq!(preserved.content(), &owner);
    assert!(preserved.capture_stamp_matches(&stamp));
    assert_eq!(
        fs::read(&sentinel)?,
        b"do not clobber this destination
"
    );
    // A real save/reopen gives Canonical backing rather than merely a source-free control.
    let mut native = Engine::default();
    native.dispatch(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":100,"begin_y_pt":200,"end_x_pt":140,"end_y_pt":200}),
    )?;
    native.dispatch("document.save_cdxml", json!({"path":canonical}))?;
    native.dispatch("document.open_cdxml", json!({"path":canonical}))?;
    assert!(native.stored_document()?.is_fully_mapped());
    assert_eq!(
        native
            .state()
            .source
            .as_ref()
            .ok_or("canonical backing")?
            .format,
        "cdxml"
    );
    let saved = native.dispatch("document.save_mol", json!({"path":allowed_mol}))?;
    let wire = serde_json::to_value(saved)?;
    assert_eq!(wire["file"]["format"], "mol_v2000");
    assert_eq!(wire["file"]["dirty"], false);
    assert!(fs::read(&allowed_mol)?.windows(5).any(|w| w == b"V2000"));
    assert_eq!(fs::read(&input)?, SOURCE);
    Ok(())
}
