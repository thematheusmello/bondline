#![cfg(not(target_arch = "wasm32"))]
//! Sequential chemical and caption edits must stay attached to the exact current CDX.
use bondline_commands::Engine;
use serde_json::{Value, json};
use std::fs;

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse only the isolated file directory")]
mod files;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
macro_rules! fixture {
    ($name:literal) => {
        include_bytes!(concat!("../../io/tests/fixtures/", $name)).as_slice()
    };
}

const BOXED: &[u8] = fixture!("cdx-label-preview/boxed.cdx");
const CREATED: &[u8] = fixture!("cdx-chemical-caption-create/chemical_first.cdx");
const OH: &[u8] = fixture!("cdx-mixed-label-edit/chemical_first_oh.cdx");
const BOTH: &[u8] = fixture!("cdx-mixed-label-edit/chemical_first_both.cdx");
const BODY: &str = "Phase β & q";

fn label(revision: u64, fragment: u32, atom: u32, element: u8, hydrogens: u16) -> Value {
    json!({"expected_revision":revision,"page_id":1,"fragment_id":fragment,"atom_id":atom,
           "element":element,"explicit_hydrogens":hydrogens})
}

fn run(revision: u64, body: &str) -> Value {
    json!({"expected_revision":revision,"page_id":1,"text_id":1,"run_index":0,"text":body})
}

fn refuse(engine: &mut Engine, command: &str, params: Value) -> Result {
    let state = serde_json::to_value(engine.state())?;
    let saved = engine.ui_file_context_stamp();
    let stamp = engine.capture_stamp();
    let error = engine
        .dispatch(command, params)
        .err()
        .ok_or("command unexpectedly succeeded")?;
    assert_eq!(
        error.info().code,
        "unsupported_content",
        "{command}: {error}"
    );
    assert_eq!(serde_json::to_value(engine.state())?, state, "{command}");
    assert!(engine.ui_file_context_stamp_matches(&saved) && engine.capture_stamp_matches(&stamp));
    Ok(())
}

#[test]
fn sequential_aliased_label_edits_and_caption_run_keep_four_distinct_history_entries() -> Result {
    let files = files::FileTestDirectory::new()?;
    let input = files.path("two-fragments-原.cdx");
    let output = files.path("two-labels-caption-保存-β.cdx");
    fs::write(&input, BOXED)?;
    let mut engine = Engine::default();
    let opened = engine.dispatch("document.open_cdx", json!({"path":input}))?;
    let created = engine.dispatch(
        "text.create",
        json!({"expected_revision":opened.revision,"page_id":1,
                "x_pt":144.25,"y_pt":216.75,"text":BODY}),
    )?;
    assert_eq!(engine.stored_document()?.export_cdx()?, CREATED);
    let first = engine.dispatch(
        "atom.set_authored_label",
        label(created.revision, 1, 1, 8, 1),
    )?;
    assert_eq!(engine.stored_document()?.export_cdx()?, OH);
    let second = engine.dispatch(
        "atom.set_authored_label",
        label(first.revision, 2, 2, 35, 0),
    )?;
    assert_eq!(engine.stored_document()?.export_cdx()?, BOTH);

    let oracle: Value = serde_json::from_str(include_str!(
        "../../io/tests/fixtures/cdx-label-edit/cases.json"
    ))?;
    let mut model = serde_json::to_value(&created.document)?;
    for (name, index, actual) in [
        ("boxed_nh2_to_oh", 0, &first.document),
        ("boxed_cl_to_br", 1, &second.document),
    ] {
        model["pages"][0]["fragments"][index] =
            oracle["cases"][name]["after_model"]["pages"][0]["fragments"][index].clone();
        assert_eq!(serde_json::to_value(actual)?, model);
    }
    // A same-length substitution changes precisely one byte in the separate Page Text.
    let mut bodies = BOTH
        .windows(BODY.len())
        .enumerate()
        .filter(|(_, bytes)| *bytes == BODY.as_bytes());
    let (start, _) = bodies.next().ok_or("fixture has no caption body")?;
    assert!(
        bodies.next().is_none(),
        "fixture has ambiguous caption bytes"
    );
    let mut final_bytes = BOTH.to_vec();
    final_bytes[start + BODY.len() - 1] = b'r';
    let changed = engine.dispatch("text.set_run", run(second.revision, "Phase β & r"))?;
    model["pages"][0]["texts"][0]["runs"][0]["text"] = json!("Phase β & r");
    assert_eq!(serde_json::to_value(&changed.document)?, model);
    assert_eq!((changed.undo_depth, changed.redo_depth), (4, 0));
    assert_eq!(engine.stored_document()?.export_cdx()?, final_bytes);
    assert!(!engine.selection_available());
    for (command, params) in [
        (
            "ui.selection",
            json!({"action":"all","expected_revision":changed.revision,"page_id":1}),
        ),
        (
            "bond.create",
            json!({"page_id":1,"begin_x_pt":32,"begin_y_pt":32,"end_x_pt":64,"end_y_pt":32}),
        ),
        ("atom.create_authored_label", {
            let mut params = label(changed.revision, 1, 1, 8, 1);
            params["style"] =
                json!({"font_family":"Roboto","size_pt":12.0,"face":0,"rgb":[0.0,0.0,0.0]});
            params
        }),
    ] {
        refuse(&mut engine, command, params)?;
    }

    let states = [
        (BOXED, &opened.document),
        (CREATED, &created.document),
        (OH, &first.document),
        (BOTH, &second.document),
        (final_bytes.as_slice(), &changed.document),
    ];
    for (command, depth) in (0..4)
        .rev()
        .map(|d| ("history.undo", d))
        .chain((1..=4).map(|d| ("history.redo", d)))
    {
        let snapshot = engine.dispatch(command, json!({}))?;
        assert_eq!(
            (snapshot.undo_depth, snapshot.redo_depth),
            (depth, 4 - depth)
        );
        assert_eq!(&snapshot.document, states[depth].1);
        assert_eq!(snapshot.file.dirty, depth != 0);
        let origin = snapshot.source.ok_or("missing original source")?;
        assert_eq!(
            (origin.byte_length, origin.modified),
            (BOXED.len(), depth != 0)
        );
        assert_eq!(engine.stored_document()?.export_cdx()?, states[depth].0);
    }
    let revision = engine.state().revision;
    let saved = engine.dispatch("document.save_cdx", json!({"path":output}))?;
    assert!(!saved.file.dirty);
    let receipt = saved.file_operation.ok_or("missing Save As receipt")?;
    assert_eq!(
        (receipt.input_revision, receipt.byte_length),
        (revision, final_bytes.len())
    );
    assert_eq!(fs::read(&input)?, BOXED);
    assert_eq!(fs::read(&output)?, final_bytes);
    let reopened = engine.dispatch("document.open_cdx", json!({"path":output}))?;
    assert_eq!(serde_json::to_value(reopened.document)?, model);
    assert_eq!(
        (
            reopened.undo_depth,
            reopened.redo_depth,
            reopened.file.dirty
        ),
        (0, 0, false)
    );
    let source = reopened.source.ok_or("reopened source missing")?;
    assert_eq!(
        (source.byte_length, source.modified),
        (final_bytes.len(), false)
    );
    engine.dispatch("text.set_run", run(reopened.revision, BODY))?;
    assert_eq!(engine.stored_document()?.export_cdx()?, BOTH);
    Ok(())
}

#[test]
fn off_paper_new_label_and_unmapped_group_preserve_their_current_sources() -> Result {
    let edge = fixture!("cdx-mixed-label-edit/edge_cl_before.cdx");
    let files = files::FileTestDirectory::new()?;
    let mut engine = Engine::default();
    let path = files.path("edge-左.cdx");
    fs::write(&path, edge)?;
    let opened = engine.dispatch("document.open_cdx", json!({"path":path}))?;
    engine.dispatch(
        "atom.set_authored_label",
        label(opened.revision, 1, 1, 8, 1),
    )?;
    let undo = engine.dispatch("history.undo", json!({}))?;
    assert_eq!((undo.undo_depth, undo.redo_depth), (0, 1));
    // The independent Cl box starts at +2.888671875; Mg would extend to -3.404296875 pt.
    refuse(
        &mut engine,
        "atom.set_authored_label",
        label(undo.revision, 2, 2, 12, 0),
    )?;
    assert_eq!(engine.stored_document()?.export_cdx()?, edge);
    let redo = engine.dispatch("history.redo", json!({}))?;
    assert_eq!((redo.undo_depth, redo.redo_depth), (1, 0));
    assert!(redo.file.dirty);
    assert!(!engine.dispatch("history.undo", json!({}))?.file.dirty);
    assert_eq!(engine.stored_document()?.export_cdx()?, edge);
    assert_eq!(fs::read(&path)?, edge);

    let group = fixture!("cdx-label-preview/group_peer.cdx");
    let path = files.path("group-未対応.cdx");
    fs::write(&path, group)?;
    let unmapped = engine.dispatch("document.open_cdx", json!({"path":path}))?;
    assert!(!engine.stored_document()?.has_cdx_mixed_preview());
    refuse(
        &mut engine,
        "atom.set_authored_label",
        label(unmapped.revision, 1, 1, 8, 1),
    )?;
    assert_eq!(engine.stored_document()?.export_cdx()?, group);
    Ok(())
}
