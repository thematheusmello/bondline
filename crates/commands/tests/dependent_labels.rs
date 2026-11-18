//! Existing APIs only, compiled against9cf7103 before selection/source producers.
use bondline_commands::{Engine, SelectionTranslate};
use bondline_io::StoredDocument;
use bondline_model::PageSize;
use serde_json::{Value, json};
use std::sync::Arc;
type TestResult = Result<(), Box<dyn std::error::Error>>;
#[path = "../../../xtask/oracles/boxed_label_support.rs"]
#[allow(dead_code, reason = "reuse frozen explicit Boxed source")]
mod boxed;
#[path = "../../../xtask/oracles/connected_label_support.rs"]
#[allow(dead_code, reason = "reuse frozen explicit Connected source")]
mod connected;

fn selection_preview(raw: &str, old_point: [f64; 2], old_box: [f64; 4]) -> TestResult {
    let old: Vec<Value> = serde_json::from_str(connected::METADATA48)?;
    assert_eq!(
        serde_json::to_value(bondline_commands::registry())?,
        json!(old)
    );
    let mut e = Engine::from_document(StoredDocument::from_cdxml(
        raw.as_bytes().into(),
        PageSize::A4,
    )?);
    assert!(!e.stored_document()?.is_fully_mapped());
    let model = e.stored_document()?.model_snapshot();
    let content = e.content().clone();
    let stamp = e.capture_stamp();
    // Real runtime RED: selection source authority currently excludes authored labels.
    let selected = e.dispatch(
        "ui.selection",
        json!({"action":"set","expected_revision":0,"page_id":1,"atom_ids":[1]}),
    )?;
    assert!(!selected.changed);
    assert_eq!(e.selection().atom_ids, vec![1]);
    assert_eq!(e.content(), &content);
    assert_eq!(e.capture_stamp(), stamp);
    assert!(Arc::ptr_eq(&model, &e.stored_document()?.model_snapshot()));
    let preview = e.preview_selection_translation(&SelectionTranslate {
        expected_revision: 0,
        expected_selection_epoch: e.selection().epoch,
        delta_x_pt: 12.5,
        delta_y_pt: -7.25,
    })?;
    let v = serde_json::to_value(preview.model())?;
    let atom = &v["pages"][0]["fragments"][0]["atoms"][0];
    assert_eq!(
        atom["position"],
        json!({"x_pt":old_point[0]+12.5,"y_pt":old_point[1]-7.25})
    );
    assert_eq!(
        atom["authored_label"]["bounding_box"],
        json!([
            old_box[0] + 12.5,
            old_box[1] - 7.25,
            old_box[2] + 12.5,
            old_box[3] - 7.25
        ])
    );
    assert!(!preview.is_fully_mapped());
    assert!(preview.has_display_projection());
    assert!(
        preview.to_cdxml().is_ok(),
        "preview must certify a complete Save representation"
    );
    assert_eq!(e.content(), &content);
    assert_eq!(e.capture_stamp(), stamp);
    assert!(Arc::ptr_eq(&model, &e.stored_document()?.model_snapshot()));
    assert_eq!(e.state().revision, 0);
    assert_eq!(e.state().undo_depth, 0);
    assert_eq!(e.state().redo_depth, 0);
    assert_eq!(e.stored_document()?.to_cdxml()?.as_ref(), raw.as_bytes());
    Ok(())
}
#[test]
fn boxed_selection_builds_a_synchronized_point_and_current_box_preview_without_authoring()
-> TestResult {
    selection_preview(boxed::SOURCE, boxed::POINT, boxed::OH_BOX)
}
#[test]
fn connected_selection_builds_a_synchronized_point_and_current_box_preview_without_authoring()
-> TestResult {
    selection_preview(
        connected::SOURCE,
        connected::POINT,
        [156.5693359375, 155.6171875, 163.4306640625, 164.3828125],
    )
}
