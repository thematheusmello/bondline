use bondline_commands::Engine;
use bondline_io::StoredDocument;
use bondline_model::PageSize;
use serde_json::{Value, json};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const BEFORE: &[u8] = include_bytes!("../../io/tests/fixtures/cdx-caption-edit/multi_before.cdx");
const AFTER: &[u8] = include_bytes!("../../io/tests/fixtures/cdx-caption-edit/multi_after.cdx");
const ATOM_LABELS: &[u8] = include_bytes!("../../io/tests/fixtures/cdx-label-preview/water.cdx");

fn engine(bytes: &[u8]) -> TestResult<Engine> {
    Ok(Engine::from_document(StoredDocument::from_cdx(
        bytes.into(),
        PageSize::A4,
    )?))
}

fn open(expected_revision: u64, run_index: usize) -> Value {
    json!({"action":"open","expected_revision":expected_revision,"page_id":1,"text_id":2,"run_index":run_index})
}

fn apply(expected_revision: u64, request_id: u64, text: &str) -> Value {
    json!({"action":"apply","expected_revision":expected_revision,"request_id":request_id,"text":text})
}

fn reject(engine: &mut Engine, params: Value, code: &str) -> TestResult {
    let before = serde_json::to_value(engine.state())?;
    let stamp = engine.capture_stamp();
    let owner = engine.ui_file_context_stamp();
    let tool = engine.bond_tool();
    let failure = engine
        .dispatch("ui.caption", params)
        .err()
        .ok_or("native caption request was accepted")?;
    assert_eq!(failure.info().code, code, "{failure}");
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert!(engine.capture_stamp_matches(&stamp));
    assert!(engine.ui_file_context_stamp_matches(&owner));
    assert_eq!(engine.bond_tool(), tool);
    Ok(())
}

fn request(engine: &mut Engine, revision: u64) -> TestResult<u64> {
    let result = serde_json::to_value(engine.dispatch("ui.caption", open(revision, 0))?)?;
    let view = &result["ui_caption"];
    let id = view["id"].as_u64().ok_or("caption request ID")?;
    assert_ne!(id, 0);
    assert_eq!(
        view,
        &json!({"id":id,"input_revision":revision,"page_id":1,"text_id":2,
          "run_index":0,"run_count":3,"text":"AV","current":true})
    );
    assert_eq!(result["changed"], false);
    assert_eq!(result["ui_caption_operation"], id);
    Ok(id)
}

#[test]
fn registry_appends_creation_and_keeps_the_existing_caption_action_schemas() -> TestResult {
    let rows = serde_json::to_value(bondline_commands::registry())?;
    let rows = rows.as_array().ok_or("registry")?;
    assert_eq!(rows.len(), 51);
    let caption = &rows[49];
    assert_eq!(caption["name"], "ui.caption");
    assert_eq!(caption["read_only"], false);
    assert_eq!(
        caption["description"],
        "Select Text or open, apply, or cancel a plain caption draft on an attached native host. Apply uses the shared caption authoring commands."
    );
    let mut existing = caption["input_schema"].clone();
    let actions = existing["oneOf"].as_array_mut().ok_or("caption actions")?;
    let open_new = actions.pop().ok_or("new caption action")?;
    assert_eq!(open_new["properties"]["action"]["const"], "open_new");
    assert_eq!(open_new["additionalProperties"], false);
    assert_eq!(
        open_new["required"],
        json!(["action", "expected_revision", "page_id", "x_pt", "y_pt"])
    );

    // The four original alternatives are pinned from the verified registry at ad2cde9.
    let revision = json!({"type":"integer","format":"uint64","minimum":0});
    let object_id = json!({"type":"integer","format":"uint32","minimum":1});
    let request_id = json!({"type":"integer","format":"uint64","minimum":1});
    assert_eq!(
        existing,
        json!({
            "$schema":"https://json-schema.org/draft/2020-12/schema",
            "title":"UiCaption","type":"object","unevaluatedProperties":false,
            "oneOf":[
                {"type":"object","additionalProperties":false,
                 "properties":{"action":{"type":"string","const":"select"}},
                 "required":["action"]},
                {"type":"object","additionalProperties":false,
                 "properties":{"action":{"type":"string","const":"open"},
                   "expected_revision":revision,"page_id":object_id,"text_id":object_id,
                   "run_index":{"type":"integer","format":"uint","minimum":0}},
                 "required":["action","expected_revision","page_id","text_id","run_index"]},
                {"type":"object","additionalProperties":false,
                 "properties":{"action":{"type":"string","const":"apply"},
                   "expected_revision":revision,"request_id":request_id,
                   "text":{"description":"Replacement UTF-8, at most 16384 bytes.","type":"string"}},
                 "required":["action","expected_revision","request_id","text"]},
                {"type":"object","additionalProperties":false,
                 "properties":{"action":{"type":"string","const":"cancel"},
                   "expected_revision":revision,"request_id":request_id},
                 "required":["action","expected_revision","request_id"]}
            ]
        })
    );
    let create = &rows[50];
    assert_eq!(create["name"], "text.create");
    assert_eq!(create["read_only"], false);
    assert_eq!(create["input_schema"]["additionalProperties"], false);
    assert_eq!(
        create["input_schema"]["required"],
        json!(["expected_revision", "page_id", "x_pt", "y_pt", "text"])
    );
    Ok(())
}

#[test]
fn presentation_requires_host_and_exact_caption_tool_and_source() -> TestResult {
    let mut current = engine(BEFORE)?;
    reject(&mut current, json!({"action":"select"}), "ui_unavailable")?;
    current.set_ui_file_host(true);
    reject(&mut current, open(0, 0), "unsupported_content")?;
    reject(
        &mut current,
        json!({"action":"select","text_id":2}),
        "invalid_params",
    )?;
    let original = current.capture_stamp();
    let selection = current.dispatch("ui.caption", json!({"action":"select"}))?;
    assert_eq!((selection.changed, selection.revision), (false, 0));
    let controls = serde_json::to_value(current.controls())?;
    let tool = controls
        .as_array()
        .ok_or("controls")?
        .iter()
        .find(|control| control["id"] == "tool.caption")
        .ok_or("Text control")?;
    assert_eq!(
        (tool["label"].as_str(), &tool["enabled"], &tool["selected"]),
        (Some("Text"), &json!(true), &json!(true))
    );
    reject(&mut current, open(1, 0), "stale_revision")?;
    reject(&mut current, open(0, 3), "not_found")?;
    let id = request(&mut current, 0)?;
    reject(&mut current, open(0, 0), "ui_busy")?;
    reject(&mut current, apply(0, id + 1, "AVAV"), "ui_stale_request")?;
    assert!(current.capture_stamp_matches(&original));
    assert_eq!(current.stored_document()?.export_cdx()?, BEFORE);

    let mut labels = engine(ATOM_LABELS)?;
    labels.set_ui_file_host(true);
    reject(
        &mut labels,
        json!({"action":"select"}),
        "unsupported_content",
    )?;
    assert_eq!(labels.stored_document()?.export_cdx()?, ATOM_LABELS);
    Ok(())
}

#[test]
fn apply_performs_one_certified_run_edit_and_preserves_invalid_draft_owner() -> TestResult {
    let mut current = engine(BEFORE)?;
    current.set_ui_file_host(true);
    current.dispatch("ui.caption", json!({"action":"select"}))?;
    let id = request(&mut current, 0)?;
    reject(
        &mut current,
        apply(0, id, &"X".repeat(16_385)),
        "resource_limit",
    )?;
    let bad = current.dispatch("ui.caption", apply(0, id, "λ")).err();
    assert!(bad.is_some(), "UTF-8 in an early CDX run is ambiguous");
    assert_eq!(
        serde_json::to_value(current.state())?["ui_caption"]["id"],
        id
    );
    assert_eq!(current.stored_document()?.export_cdx()?, BEFORE);
    let unchanged = current.dispatch("ui.caption", apply(0, id, "AV"))?;
    assert_eq!(
        (unchanged.changed, unchanged.revision, unchanged.undo_depth),
        (false, 0, 0)
    );
    assert!(serde_json::to_value(unchanged)?.get("ui_caption").is_none());
    let next = request(&mut current, 0)?;
    assert!(next > id);
    let result = current.dispatch("ui.caption", apply(0, next, "AVAV"))?;
    assert_eq!(
        (result.changed, result.revision, result.undo_depth),
        (true, 1, 1)
    );
    assert_eq!(current.stored_document()?.export_cdx()?, AFTER);
    current.dispatch("history.undo", json!({}))?;
    assert_eq!(current.stored_document()?.export_cdx()?, BEFORE);
    current.dispatch("history.redo", json!({}))?;
    assert_eq!(current.stored_document()?.export_cdx()?, AFTER);
    Ok(())
}

#[test]
fn hidden_file_open_cancel_and_host_replacement_retire_old_request_authority() -> TestResult {
    let mut current = engine(BEFORE)?;
    current.set_ui_file_host(true);
    current.dispatch("ui.caption", json!({"action":"select"}))?;
    let id = request(&mut current, 0)?;
    let file = current.dispatch("ui.file", json!({"expected_revision":0,"action":"open"}))?;
    let file_id = file.ui_file_operation.ok_or("file request")?;
    current.dispatch(
        "ui.file",
        json!({"expected_revision":0,"action":"cancel","request_id":file_id}),
    )?;
    assert_eq!(
        serde_json::to_value(current.state())?["ui_caption"]["current"],
        false
    );
    reject(&mut current, apply(0, id, "AVAV"), "ui_stale_request")?;
    current.dispatch(
        "ui.caption",
        json!({"action":"cancel","expected_revision":0,"request_id":id}),
    )?;
    let next = request(&mut current, 0)?;
    current.set_ui_file_host(false);
    current.set_ui_file_host(true);
    reject(&mut current, apply(0, next, "AVAV"), "ui_stale_request")?;
    assert_eq!(current.stored_document()?.export_cdx()?, BEFORE);
    Ok(())
}
