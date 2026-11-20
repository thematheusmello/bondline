#![cfg(not(target_arch = "wasm32"))]

use bondline_commands::Engine;
use serde_json::{Value, json};
use std::fs;

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse the isolated file directory")]
mod files;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const BEFORE: &[u8] = include_bytes!("../../io/tests/fixtures/cdx-caption-edit/single_before.cdx");
const OLD_EDIT: &[u8] = include_bytes!("../../io/tests/fixtures/cdx-caption-edit/single_after.cdx");
const FIRST: &[u8] = include_bytes!("../../io/tests/fixtures/cdx-caption-create/first.cdx");
const SPARSE: &[u8] = include_bytes!("../../io/tests/fixtures/cdx-caption-create/after_undo.cdx");
const WITNESSES: &str = include_str!("../../io/tests/fixtures/cdx-caption-edit/cases.json");
const BODY: &str = "Phase β & q";

fn create(revision: u64, text: &str) -> Value {
    json!({"expected_revision":revision,"page_id":1,"x_pt":144.25,"y_pt":216.75,"text":text})
}

fn edit(revision: u64, id: u32, text: &str) -> Value {
    json!({"expected_revision":revision,"page_id":1,"text_id":id,"run_index":0,"text":text})
}

fn reject(engine: &mut Engine, command: &str, params: Value, code: &str) -> TestResult {
    let before = serde_json::to_value(engine.state())?;
    let stamp = engine.capture_stamp();
    let file = engine.ui_file_context_stamp();
    let refusal = engine
        .dispatch(command, params)
        .err()
        .ok_or("unexpected edit")?;
    assert_eq!(refusal.info().code, code, "{refusal}");
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert!(engine.capture_stamp_matches(&stamp));
    assert!(engine.ui_file_context_stamp_matches(&file));
    Ok(())
}

#[test]
fn cdx_creation_keeps_exact_history_sparse_identity_and_both_run_edits() -> TestResult {
    let directory = files::FileTestDirectory::new()?;
    let input = directory.path("caption-原-λ.cdx");
    let output = directory.path("caption-保存-β.cdx");
    fs::write(&input, BEFORE)?;
    let manifest: Value = serde_json::from_str(WITNESSES)?;
    let witness = &manifest["cases"]["single_grow"];

    let mut engine = Engine::default();
    let opened = engine.dispatch("document.open_cdx", json!({"path":input}))?;
    assert_eq!(
        serde_json::to_value(opened.document)?,
        witness["before_model"]
    );
    let original = engine.content().clone();
    let first = engine.dispatch("text.create", create(1, BODY))?;
    assert!(first.changed && first.file.dirty);
    assert_eq!(
        (first.revision, first.undo_depth, first.redo_depth),
        (2, 1, 0)
    );
    let mut model = witness["before_model"].clone();
    let texts = model["pages"][0]["texts"]
        .as_array_mut()
        .ok_or("original captions")?;
    texts.push(json!({"id":2,"position":{"x_pt":144.25,"y_pt":216.75},
                     "runs":[{"text":BODY,"style":{"font_family":"Roboto","size_pt":12.0,
                                                  "face":0,"rgb":[0.0,0.0,0.0]}}]}));
    assert_eq!(serde_json::to_value(first.document)?, model);
    assert_eq!(engine.stored_document()?.export_cdx()?, FIRST);
    assert!(!engine.stored_document()?.is_fully_mapped());
    assert!(!engine.selection_available());

    let undo = engine.dispatch("history.undo", json!({}))?;
    assert_eq!(engine.content(), &original);
    assert!(!undo.file.dirty);
    assert_eq!(engine.stored_document()?.export_cdx()?, BEFORE);
    let redo = engine.dispatch("history.redo", json!({}))?;
    assert!(redo.file.dirty);
    assert_eq!(engine.stored_document()?.export_cdx()?, FIRST);
    engine.dispatch("history.undo", json!({}))?;
    reject(&mut engine, "text.create", create(5, ""), "invalid_params")?;
    let branched = engine.dispatch("text.create", create(5, BODY))?;
    assert_eq!(
        (branched.revision, branched.undo_depth, branched.redo_depth),
        (6, 1, 0)
    );
    model["pages"][0]["texts"][1]["id"] = json!(3);
    assert_eq!(serde_json::to_value(branched.document)?, model);
    assert_eq!(engine.stored_document()?.export_cdx()?, SPARSE);
    assert!(branched.file.dirty);

    let crossed = directory.path("would-lose-source.cdxml");
    reject(
        &mut engine,
        "document.save_cdxml",
        json!({"path":crossed}),
        "unsupported_content",
    )?;
    assert!(!crossed.exists());
    let saved = engine.dispatch("document.save_cdx", json!({"path":output}))?;
    assert!(!saved.file.dirty);
    let source = saved.source.ok_or("retained original CDX")?;
    assert_eq!((source.byte_length, source.modified), (BEFORE.len(), true));
    let receipt = saved.file_operation.ok_or("CDX save receipt")?;
    assert_eq!(
        (receipt.input_revision, receipt.byte_length),
        (6, SPARSE.len())
    );
    assert_eq!(fs::read(&input)?, BEFORE);
    assert_eq!(fs::read(&output)?, SPARSE);
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
    let source = reopened.source.ok_or("reopened sparse CDX")?;
    assert_eq!((source.byte_length, source.modified), (SPARSE.len(), false));
    reject(
        &mut engine,
        "text.set_run",
        edit(reopened.revision, 2, "Gap"),
        "invalid_document",
    )?;

    // Two frozen oracles give the original caption rewrite and the splice before the Page end.
    let insertion = SPARSE
        .get(BEFORE.len() - 4..)
        .ok_or("frozen inserted caption")?;
    let old_prefix = OLD_EDIT
        .get(..OLD_EDIT.len() - 4)
        .ok_or("frozen old caption rewrite")?;
    let mut expected = [old_prefix, insertion].concat();
    let old_text = witness["params"]["text"].as_str().ok_or("old edit text")?;
    let old_changed = engine.dispatch("text.set_run", edit(reopened.revision, 1, old_text))?;
    model["pages"][0]["texts"][0] = witness["after_model"]["pages"][0]["texts"][0].clone();
    assert_eq!(serde_json::to_value(old_changed.document)?, model);
    assert_eq!(engine.stored_document()?.export_cdx()?, expected);

    let replacement = "Phase β & r";
    let mut bodies = expected
        .windows(BODY.len())
        .enumerate()
        .filter(|(_, bytes)| *bytes == BODY.as_bytes());
    let (start, _) = bodies.next().ok_or("new caption body")?;
    assert!(bodies.next().is_none());
    expected[start..start + BODY.len()].copy_from_slice(replacement.as_bytes());
    let new_changed =
        engine.dispatch("text.set_run", edit(old_changed.revision, 3, replacement))?;
    model["pages"][0]["texts"][1]["runs"][0]["text"] = json!(replacement);
    assert_eq!(serde_json::to_value(new_changed.document)?, model);
    assert_eq!(engine.stored_document()?.export_cdx()?, expected);
    assert_eq!(fs::read(output)?, SPARSE);
    Ok(())
}

#[test]
fn empty_cdx_creates_its_first_caption_and_reopens_the_sparse_undo_branch() -> TestResult {
    let before = include_bytes!("../../io/tests/fixtures/cdx-caption-empty/implicit_before.cdx");
    let first = include_bytes!("../../io/tests/fixtures/cdx-caption-empty/implicit_first.cdx");
    let sparse =
        include_bytes!("../../io/tests/fixtures/cdx-caption-empty/implicit_after_undo.cdx");
    let unsupported_font =
        include_bytes!("../../io/tests/fixtures/cdx-caption-empty/font_not_utf8.cdx");
    let witnesses: Value = serde_json::from_str(include_str!(
        "../../io/tests/fixtures/cdx-caption-empty/cases.json"
    ))?;
    let case = &witnesses["cases"]["implicit"];
    let directory = files::FileTestDirectory::new()?;
    let input = directory.path("empty-原-λ.cdx");
    let output = directory.path("first-caption-保存-β.cdx");
    fs::write(&input, before)?;
    let mut engine = Engine::default();
    let opened = engine.dispatch("document.open_cdx", json!({"path":input}))?;
    let original = engine.content().clone();
    let stored = engine.stored_document()?;
    assert_eq!(stored.export_cdx()?, before);
    assert!(stored.model().pages()[0].texts().is_empty());
    let created = engine.dispatch("text.create", create(opened.revision, BODY))?;
    assert_eq!(
        serde_json::to_value(opened.document)?["pages"][0]["size"],
        case["before"]["model"]["pages"][0]["size"]
    );
    assert!(created.changed && created.file.dirty);
    assert_eq!(
        (created.revision, created.undo_depth, created.redo_depth),
        (2, 1, 0)
    );
    assert_eq!(
        serde_json::to_value(created.document)?,
        case["first"]["model"]
    );
    assert_eq!(engine.stored_document()?.export_cdx()?, first);
    assert!(!engine.stored_document()?.is_fully_mapped());
    assert!(!engine.selection_available());
    reject(
        &mut engine,
        "bond.create",
        json!({"page_id":1,"begin_x_pt":80,"begin_y_pt":80,"end_x_pt":120,"end_y_pt":80}),
        "unsupported_content",
    )?;

    let undo = engine.dispatch("history.undo", json!({}))?;
    assert!(!undo.file.dirty);
    assert_eq!(engine.content(), &original);
    assert_eq!(engine.stored_document()?.export_cdx()?, before);
    let branched = engine.dispatch("text.create", create(undo.revision, BODY))?;
    assert_eq!(
        (branched.revision, branched.undo_depth, branched.redo_depth),
        (4, 1, 0)
    );
    assert_eq!(
        serde_json::to_value(branched.document)?,
        case["after_undo"]["model"]
    );
    assert_eq!(engine.stored_document()?.export_cdx()?, sparse);

    let saved = engine.dispatch("document.save_cdx", json!({"path":output}))?;
    assert!(!saved.file.dirty);
    let source = saved.source.ok_or("original empty CDX source")?;
    assert_eq!((source.byte_length, source.modified), (before.len(), true));
    assert_eq!(fs::read(&input)?, before);
    assert_eq!(fs::read(&output)?, sparse);
    let reopened = engine.dispatch("document.open_cdx", json!({"path":output}))?;
    assert_eq!(
        serde_json::to_value(reopened.document)?,
        case["after_undo"]["model"]
    );
    assert_eq!(
        (
            reopened.undo_depth,
            reopened.redo_depth,
            reopened.file.dirty
        ),
        (0, 0, false)
    );
    assert_eq!(engine.stored_document()?.export_cdx()?, sparse);

    let incompatible = directory.path("unsupported-font.cdx");
    fs::write(&incompatible, unsupported_font)?;
    let refused_source = engine.dispatch("document.open_cdx", json!({"path":incompatible}))?;
    reject(
        &mut engine,
        "text.create",
        create(refused_source.revision, BODY),
        "unsupported_content",
    )?;
    assert_eq!(engine.stored_document()?.export_cdx()?, unsupported_font);
    Ok(())
}

#[test]
fn chemistry_survives_new_cdx_caption_branch_save_and_reopen() -> TestResult {
    let directory = files::FileTestDirectory::new()?;
    let geometry: Value = serde_json::from_str(include_str!(
        "../../io/tests/fixtures/cdx-label-preview/cases.json"
    ))?;
    let chemical = &geometry["renderable"]["boxed"]["model"]["pages"][0];
    for (name, before, first, sparse, next_id) in [
        (
            "chemical",
            include_bytes!("../../io/tests/fixtures/cdx-label-preview/boxed.cdx").as_slice(),
            include_bytes!(
                "../../io/tests/fixtures/cdx-chemical-caption-create/chemical_first.cdx"
            )
            .as_slice(),
            include_bytes!(
                "../../io/tests/fixtures/cdx-chemical-caption-create/chemical_after_undo.cdx"
            )
            .as_slice(),
            1,
        ),
        (
            "mixed",
            include_bytes!("../../io/tests/fixtures/cdx-mixed-caption-edit/before.cdx").as_slice(),
            include_bytes!("../../io/tests/fixtures/cdx-chemical-caption-create/mixed_first.cdx")
                .as_slice(),
            include_bytes!(
                "../../io/tests/fixtures/cdx-chemical-caption-create/mixed_after_undo.cdx"
            )
            .as_slice(),
            20,
        ),
    ] {
        let input = directory.path(&format!("{name}-原.cdx"));
        let output = directory.path(&format!("{name}-保存-β.cdx"));
        fs::write(&input, before)?;
        let mut engine = Engine::default();
        let opened = engine.dispatch("document.open_cdx", json!({"path":input}))?;
        let original = engine.content().clone();
        let baseline = serde_json::to_value(opened.document)?;
        assert_eq!(baseline["pages"][0]["fragments"], chemical["fragments"]);
        let old_texts = baseline["pages"][0]["texts"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        let old_ids: Vec<_> = old_texts.iter().map(|t| &t["id"]).collect();
        assert_eq!(
            json!(old_ids),
            if name == "mixed" {
                json!([1, 19])
            } else {
                json!([])
            }
        );
        let expected = |id| {
            let mut texts = old_texts.clone();
            texts.push(json!({"id":id,"position":{"x_pt":144.25,"y_pt":216.75},
                "runs":[{"text":BODY,"style":{"font_family":"Roboto","size_pt":12.0,"face":0,"rgb":[0.0,0.0,0.0]}}]}));
            let mut document = baseline.clone();
            document["pages"][0]["texts"] = json!(texts);
            document
        };
        let created = engine.dispatch("text.create", create(opened.revision, BODY))?;
        assert!(created.changed && created.file.dirty);
        assert_eq!((created.undo_depth, created.redo_depth), (1, 0));
        assert_eq!(serde_json::to_value(created.document)?, expected(next_id));
        assert_eq!(engine.stored_document()?.export_cdx()?, first);
        assert!(engine.stored_document()?.has_cdx_mixed_preview());
        assert!(!engine.selection_available());

        let undo = engine.dispatch("history.undo", json!({}))?;
        assert!(!undo.file.dirty);
        assert_eq!(engine.content(), &original);
        assert_eq!(engine.stored_document()?.export_cdx()?, before);
        engine.dispatch("history.redo", json!({}))?;
        assert_eq!(engine.stored_document()?.export_cdx()?, first);
        let undo = engine.dispatch("history.undo", json!({}))?;
        reject(
            &mut engine,
            "text.create",
            create(undo.revision, "   "),
            "invalid_params",
        )?;
        let branch = engine.dispatch("text.create", create(undo.revision, BODY))?;
        assert_eq!((branch.undo_depth, branch.redo_depth), (1, 0));
        assert_eq!(
            serde_json::to_value(branch.document)?,
            expected(next_id + 1)
        );
        assert_eq!(engine.stored_document()?.export_cdx()?, sparse);
        let lost = directory.path(&format!("{name}-loss.cdxml"));
        reject(
            &mut engine,
            "document.save_cdxml",
            json!({"path":lost}),
            "unsupported_content",
        )?;
        assert!(!lost.exists());
        let saved = engine.dispatch("document.save_cdx", json!({"path":output}))?;
        assert!(!saved.file.dirty);
        let source = saved.source.ok_or("retained chemical source")?;
        assert_eq!((source.byte_length, source.modified), (before.len(), true));
        let receipt = saved.file_operation.ok_or("Save As receipt")?;
        assert_eq!(
            (receipt.byte_length, receipt.input_revision),
            (sparse.len(), branch.revision)
        );
        assert_eq!(fs::read(&input)?, before);
        assert_eq!(fs::read(&output)?, sparse);
        let reopened = engine.dispatch("document.open_cdx", json!({"path":output}))?;
        assert_eq!(
            serde_json::to_value(reopened.document)?,
            expected(next_id + 1)
        );
        assert_eq!(
            (
                reopened.undo_depth,
                reopened.redo_depth,
                reopened.file.dirty
            ),
            (0, 0, false)
        );
        assert_eq!(engine.stored_document()?.export_cdx()?, sparse);
        reject(
            &mut engine,
            "text.set_run",
            edit(reopened.revision, next_id, "Gap"),
            "invalid_document",
        )?;
        let editing = engine.dispatch(
            "text.set_run",
            edit(reopened.revision, next_id + 1, "Phase β & r"),
        )?;
        assert!(editing.changed);
        engine.dispatch("history.undo", json!({}))?;
        assert_eq!(engine.stored_document()?.export_cdx()?, sparse);
    }
    Ok(())
}

#[test]
fn unsupported_chemical_ink_and_exhausted_mixed_raw_id_cannot_create_captions() -> TestResult {
    let directory = files::FileTestDirectory::new()?;
    for (name, source, mixed_preview) in [
        (
            "contradictory-label",
            include_bytes!("../../io/tests/fixtures/cdx-label-preview/contradictory_ink_box.cdx")
                .as_slice(),
            false,
        ),
        (
            "raw-id-exhausted",
            include_bytes!("../../io/tests/fixtures/cdx-mixed-caption/boxed_after.cdx").as_slice(),
            true,
        ),
    ] {
        let path = directory.path(&format!("{name}.cdx"));
        fs::write(&path, source)?;
        let mut engine = Engine::default();
        let opened = engine.dispatch("document.open_cdx", json!({"path":path}))?;
        assert_eq!(
            engine.stored_document()?.has_cdx_mixed_preview(),
            mixed_preview
        );
        reject(
            &mut engine,
            "text.create",
            create(opened.revision, BODY),
            "unsupported_content",
        )?;
        assert_eq!(engine.stored_document()?.export_cdx()?, source);
        assert_eq!(fs::read(&path)?, source);
    }
    Ok(())
}
