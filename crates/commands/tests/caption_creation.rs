//! Register the creation workflow through the actual JSON boundary first.
use bondline_commands::Engine;
use bondline_io::StoredDocument;
use bondline_model::PageSize;
use serde_json::{Value, json};
use std::fs;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse the isolated file directory")]
mod files;

const BLANK: &[u8] = include_bytes!("../../io/tests/fixtures/caption-create/blank.cdxml");
const FIRST: &[u8] = include_bytes!("../../io/tests/fixtures/caption-create/first.cdxml");
const AFTER_UNDO: &[u8] = include_bytes!("../../io/tests/fixtures/caption-create/after_undo.cdxml");
const MIXED_BEFORE: &[u8] =
    include_bytes!("../../io/tests/fixtures/caption-create/mixed_before.cdxml");
const MIXED_AFTER: &[u8] =
    include_bytes!("../../io/tests/fixtures/caption-create/mixed_after.cdxml");
const FOREIGN: &[u8] =
    include_bytes!("../../io/tests/fixtures/caption-create/mixed_noncanonical.cdxml");
const CDX: &[u8] = include_bytes!("../../io/tests/fixtures/cdx-caption/caption_ascii.cdx");
const WITNESSES: &str = include_str!("../../io/tests/fixtures/caption-create/cases.json");

fn create(expected_revision: u64, text: &str) -> Value {
    json!({"expected_revision":expected_revision,"page_id":1,
           "x_pt":144.25,"y_pt":216.75,"text":text})
}

fn blank() -> TestResult<Engine> {
    let mut stored = StoredDocument::default();
    let page = stored.model().pages().first().ok_or("blank page")?.id();
    stored
        .model_mut()
        .set_page_size(page, PageSize::new(512.0, 512.0)?)?;
    Ok(Engine::from_document(stored))
}

#[test]
fn deliberate_creation_adds_one_literal_baseline_and_one_history_edit() -> TestResult {
    let mut engine = blank()?;
    let original = engine.content().clone();
    assert_eq!(engine.stored_document()?.to_cdxml()?.as_ref(), BLANK);
    let created = engine.dispatch("text.create", create(0, "Phase β & q"))?;
    assert!(created.changed && created.file.dirty);
    assert_eq!(
        (created.revision, created.undo_depth, created.redo_depth),
        (1, 1, 0)
    );
    let model = serde_json::to_value(created.document)?;
    assert_eq!(
        model,
        json!({"pages":[{"id":1,"size":{"width_pt":512.0,"height_pt":512.0},"fragments":[],
            "texts":[{"id":1,"position":{"x_pt":144.25,"y_pt":216.75},
            "runs":[{"text":"Phase β & q","style":{"font_family":"Roboto","size_pt":12.0,
               "face":0,"rgb":[0.0,0.0,0.0]}}]}]}]})
    );
    let saved = engine.stored_document()?.to_cdxml()?.into_owned();
    assert_eq!(saved, FIRST);
    engine.dispatch("history.undo", json!({}))?;
    assert_eq!(engine.content(), &original);
    assert!(!engine.state().file.dirty);
    engine.dispatch("history.redo", json!({}))?;
    assert_eq!(engine.stored_document()?.to_cdxml()?.as_ref(), saved);
    Ok(())
}

fn reject(engine: &mut Engine, params: Value, code: &str) -> TestResult {
    let before = serde_json::to_value(engine.state())?;
    let owner = engine.capture_stamp();
    let file = engine.ui_file_context_stamp();
    let error = engine
        .dispatch("text.create", params)
        .err()
        .ok_or("invalid creation was accepted")?;
    assert_eq!(error.info().code, code, "{error}");
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert!(engine.capture_stamp_matches(&owner) && engine.ui_file_context_stamp_matches(&file));
    Ok(())
}

#[test]
fn rejected_creation_spends_no_id_and_reopen_of_undo_branch_keeps_the_gap() -> TestResult {
    let directory = files::FileTestDirectory::new()?;
    let path = directory.path("caption-保存-β.cdxml");
    let mut engine = blank()?;
    for (params, code) in [
        (create(1, "Phase A"), "stale_revision"),
        (create(0, " \u{a0} "), "invalid_params"),
        (create(0, "Phase\nA"), "unsupported_content"),
        (create(0, "Phase 🧪"), "unsupported_content"),
        (create(0, &"λ".repeat(8_193)), "resource_limit"),
        (
            json!({"expected_revision":0,"page_id":1,"x_pt":0,"y_pt":100,"text":"Phase A"}),
            "invalid_params",
        ),
        (
            json!({"expected_revision":0,"page_id":1,"x_pt":511.5,"y_pt":100,"text":"Phase A"}),
            "unsupported_content",
        ),
        (
            json!({"expected_revision":0,"page_id":9,"x_pt":100,"y_pt":100,"text":"Phase A"}),
            "not_found",
        ),
        (
            json!({"expected_revision":0,"page_id":1,"x_pt":100,"y_pt":100,"text":"Phase A","font":"other"}),
            "invalid_params",
        ),
    ] {
        reject(&mut engine, params, code)?;
    }
    engine.dispatch("text.create", create(0, "Phase β & q"))?;
    assert_eq!(engine.stored_document()?.to_cdxml()?.as_ref(), FIRST);
    engine.dispatch("history.undo", json!({}))?;
    reject(&mut engine, create(2, ""), "invalid_params")?;
    let branched = engine.dispatch("text.create", create(2, "Phase β & q"))?;
    assert_eq!(
        (branched.revision, branched.undo_depth, branched.redo_depth),
        (3, 1, 0)
    );
    assert_eq!(engine.stored_document()?.to_cdxml()?.as_ref(), AFTER_UNDO);
    assert_eq!(
        serde_json::to_value(&branched.document)?["pages"][0]["texts"][0]["id"],
        2
    );
    engine.dispatch("document.save_cdxml", json!({"path":path}))?;
    assert_eq!(fs::read(&path)?, AFTER_UNDO);
    let reopened = engine.dispatch("document.open_cdxml", json!({"path":path}))?;
    assert_eq!(reopened.document, branched.document);
    assert!(!reopened.file.dirty);
    let edited = engine.dispatch(
        "text.set_run",
        json!({"expected_revision":reopened.revision,"page_id":1,"text_id":2,"run_index":0,"text":"Phase C"}),
    )?;
    assert!(edited.changed);
    assert!(
        String::from_utf8_lossy(&engine.stored_document()?.to_cdxml()?)
            .contains("<t id=\"3\" p=\"144.25 216.75\"")
    );
    Ok(())
}

#[test]
fn native_mixed_creation_appends_without_rewriting_existing_data() -> TestResult {
    let cases: Value = serde_json::from_str(WITNESSES)?;
    let case = &cases["cases"]["mixed_append"];
    let directory = files::FileTestDirectory::new()?;
    let input = directory.path("before-原.cdxml");
    let output = directory.path("caption-新.cdxml");
    fs::write(&input, MIXED_BEFORE)?;
    let mut engine = Engine::default();
    let opened = engine.dispatch("document.open_cdxml", json!({"path":input}))?;
    assert_eq!(
        serde_json::to_value(&opened.document)?,
        case["before_model"]
    );
    let original = engine.capture_stamp();
    let created = engine.dispatch("text.create", create(opened.revision, "Phase β & q"))?;
    assert!(created.changed && created.file.dirty);
    assert_eq!(
        serde_json::to_value(&created.document)?,
        case["after_model"]
    );
    assert!(!engine.capture_stamp_matches(&original));
    assert_eq!(engine.stored_document()?.to_cdxml()?.as_ref(), MIXED_AFTER);
    engine.dispatch("document.save_cdxml", json!({"path":output}))?;
    assert_eq!(fs::read(&input)?, MIXED_BEFORE);
    assert_eq!(fs::read(&output)?, MIXED_AFTER);
    let reopened = engine.dispatch("document.open_cdxml", json!({"path":output}))?;
    assert_eq!(reopened.document, created.document);
    assert_eq!(
        (
            reopened.undo_depth,
            reopened.redo_depth,
            reopened.file.dirty
        ),
        (0, 0, false)
    );
    Ok(())
}

#[test]
fn retained_foreign_cdxml_and_detached_cdx_refuse_creation_even_with_owned_caption_data()
-> TestResult {
    let mut xml = Engine::from_document(StoredDocument::from_cdxml(FOREIGN.into(), PageSize::A4)?);
    let mut cdx = Engine::from_document(StoredDocument::from_cdx(CDX.into(), PageSize::A4)?);
    assert!(!xml.document()?.pages()[0].texts().is_empty());
    reject(&mut xml, create(0, "Phase A"), "unsupported_content")?;
    assert!(cdx.stored_document()?.has_cdx_caption_preview());
    let resized = cdx.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":612.0,"height_pt":792.0}),
    )?;
    assert!(resized.changed);
    assert!(!cdx.document()?.pages()[0].texts().is_empty());
    assert!(!cdx.stored_document()?.has_cdx_caption_preview());
    reject(
        &mut cdx,
        create(resized.revision, "Phase A"),
        "unsupported_content",
    )?;
    cdx.dispatch("history.undo", json!({}))?;
    assert_eq!(xml.stored_document()?.to_cdxml()?.as_ref(), FOREIGN);
    assert_eq!(cdx.stored_document()?.export_cdx()?, CDX);
    Ok(())
}

#[test]
fn blank_paper_draft_is_attached_and_exact_then_uses_one_registered_edit() -> TestResult {
    let mut engine = blank()?;
    engine.set_ui_file_host(true);
    let before = engine.capture_stamp();
    engine.dispatch("ui.caption", json!({"action":"select"}))?;
    let open = engine.dispatch(
        "ui.caption",
        json!({"action":"open_new","expected_revision":0,"page_id":1,"x_pt":144.25,"y_pt":216.75}),
    )?;
    let request = open.ui_caption_operation.ok_or("caption request")?;
    assert_eq!(
        (open.changed, open.revision, open.undo_depth),
        (false, 0, 0)
    );
    assert_eq!(
        serde_json::to_value(&open)?["ui_caption"],
        json!({"id":request,"input_revision":0,"page_id":1,"position":{"x_pt":144.25,
               "y_pt":216.75},"text":"","current":true})
    );
    assert!(engine.capture_stamp_matches(&before));
    let empty = engine.dispatch(
        "ui.caption",
        json!({"action":"apply","expected_revision":0,"request_id":request,"text":""}),
    );
    let failure = empty.err().ok_or("empty creation draft was applied")?;
    assert_eq!(failure.info().code, "invalid_params");
    assert!(engine.capture_stamp_matches(&before));
    let changed = engine.dispatch(
        "ui.caption",
        json!({"action":"apply","expected_revision":0,"request_id":request,"text":"Phase β & q"}),
    )?;
    assert_eq!(
        (changed.changed, changed.revision, changed.undo_depth),
        (true, 1, 1)
    );
    assert!(changed.ui_caption.is_none());
    assert_eq!(engine.stored_document()?.to_cdxml()?.as_ref(), FIRST);
    Ok(())
}
