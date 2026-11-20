//! Initial existing-JSON acceptance: no new Rust type is required for the intended RED.
use bondline_commands::Engine;
use serde_json::json;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn explicit_document_settings_commit_once_without_moving_the_graph() -> TestResult {
    let mut engine = Engine::default();
    engine.dispatch(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":144,
        "begin_y_pt":144,"end_x_pt":192,"end_y_pt":144}),
    )?;
    let before = serde_json::to_value(engine.state())?;
    let result = engine.dispatch(
        "document.set_document_settings",
        json!({
        "expected_revision":1,"bond_length_pt":24.5,"chain_angle_degrees":120.5,
        "line_width_pt":2.0}),
    )?;
    assert!(result.changed);
    assert_eq!(
        (result.revision, result.undo_depth, result.redo_depth),
        (2, 2, 0)
    );
    assert!(result.file.dirty && result.source.is_none());
    assert!(result.file_operation.is_none() && result.ui_file_operation.is_none());
    let actual = serde_json::to_value(result)?;
    assert_eq!(actual["document"]["pages"], before["document"]["pages"]);
    assert_eq!(
        actual["document"]["drawing_settings"],
        json!({"bond_length_pt":24.5,"chain_angle_degrees":120.5})
    );
    Ok(())
}

// Expanded independent acceptance; the initial1,227 bytes above remain unchanged.
use bondline_commands::Command;
use bondline_io::StoredDocument;
use bondline_model::PageSize;
use serde_json::Value;
use std::{collections::BTreeSet, fs, sync::Arc};
#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(
    dead_code,
    reason = "reuse only the unchanged isolated-directory helper"
)]
mod support;
const COMBINED: &str = "document.set_document_settings";
const PAIR: &str = "document.set_drawing_settings";
const NATIVE: &[u8] = include_bytes!("../../io/tests/fixtures/bondline-v0.cdxml");
fn native() -> TestResult<Engine> {
    Ok(Engine::from_document(StoredDocument::from_cdxml(
        NATIVE.into(),
        PageSize::A4,
    )?))
}
fn triple(e: &Engine, length: Option<f64>, angle: Option<f64>, width: Option<f64>) -> Value {
    json!({"expected_revision":e.state().revision,"bond_length_pt":length,
        "chain_angle_degrees":angle,"line_width_pt":width})
}
fn reject(e: &mut Engine, params: Value, code: &str) -> TestResult {
    let before = serde_json::to_value(e.state())?;
    let content = e.content().clone();
    let stamp = e.capture_stamp();
    let file = e.ui_file_context_stamp();
    let view = e.view();
    let tool = (e.bond_tool(), e.effective_tool());
    let generation = e.session_generation();
    let error = e
        .dispatch(COMBINED, params)
        .err()
        .ok_or("invalid triple accepted")?;
    assert_eq!(error.info().code, code, "{error}");
    assert_eq!(serde_json::to_value(e.state())?, before);
    assert_eq!(e.content(), &content);
    assert!(e.capture_stamp_matches(&stamp));
    assert!(e.ui_file_context_stamp_matches(&file));
    assert_eq!(
        (
            e.view(),
            e.bond_tool(),
            e.effective_tool(),
            e.session_generation()
        ),
        (view, tool.0, tool.1, generation)
    );
    Ok(())
}
#[test]
fn triple_history_and_legacy_pair_keep_all_unrelated_model_data() -> TestResult {
    let mut e = native()?;
    let original = e.stored_document()?.model_snapshot();
    let before = serde_json::to_value(e.document()?)?;
    let view = e.view();
    let generation = e.session_generation();
    let result = e.dispatch(COMBINED, triple(&e, Some(24.5), Some(120.5), Some(2.0)))?;
    assert_eq!(
        (
            result.changed,
            result.revision,
            result.undo_depth,
            result.redo_depth
        ),
        (true, 1, 1, 0)
    );
    assert!(result.file.dirty && result.source.ok_or("source")?.modified);
    assert!(result.file_operation.is_none() && result.ui_file_operation.is_none());
    let expected = json!({"bond_length_pt":24.5,"chain_angle_degrees":120.5});
    let edited = serde_json::to_value(e.document()?)?;
    assert_eq!(edited["drawing_settings"], expected);
    assert_eq!(edited["drawing_style"], json!({"line_width_pt":2.0}));
    assert_eq!(edited["pages"], before["pages"]);
    assert_eq!((e.view(), e.session_generation()), (view, generation));
    let changed = e.stored_document()?.model_snapshot();
    e.dispatch("history.undo", json!({}))?;
    assert!(Arc::ptr_eq(
        &original,
        &e.stored_document()?.model_snapshot()
    ));
    assert!(!e.state().file.dirty);
    assert_eq!(e.stored_document()?.export_cdxml()?, NATIVE);
    e.dispatch("history.redo", json!({}))?;
    assert!(Arc::ptr_eq(
        &changed,
        &e.stored_document()?.model_snapshot()
    ));
    e.dispatch(
        PAIR,
        json!({"expected_revision":3,"bond_length_pt":30,"chain_angle_degrees":90}),
    )?;
    assert_eq!(
        serde_json::to_value(e.document()?)?["drawing_style"],
        json!({"line_width_pt":2.0})
    );
    let result = e.dispatch(COMBINED, triple(&e, None, None, None))?;
    assert_eq!(
        (result.changed, result.revision, result.undo_depth),
        (true, 5, 3)
    );
    assert_eq!(serde_json::to_value(e.document()?)?, before);
    assert!(
        !result.file.dirty,
        "clearing all values restores the saved complete snapshot"
    );
    assert_eq!(e.stored_document()?.to_cdxml()?.as_ref(), NATIVE);
    Ok(())
}
#[test]
fn validated_noop_preserves_redo_selection_file_request_and_all_weak_contexts() -> TestResult {
    let mut e = native()?;
    e.dispatch(COMBINED, triple(&e, Some(24.5), Some(120.5), Some(2.0)))?;
    e.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":400,"height_pt":600}),
    )?;
    e.dispatch("history.undo", json!({}))?;
    e.set_ui_file_host(true);
    e.dispatch("ui.bond_tool", json!({"order":"double"}))?;
    e.dispatch("ui.view", json!({"action":"actual"}))?;
    e.dispatch(
        "ui.selection",
        json!({"action":"set","expected_revision":3,"page_id":1,"atom_ids":[1,2]}),
    )?;
    let selection = e.selection_context()?;
    let before = serde_json::to_value(e.state())?;
    let stamp = e.capture_stamp();
    let file = e.ui_file_context_stamp();
    let model = e.stored_document()?.model_snapshot();
    let result = e.dispatch(COMBINED, triple(&e, Some(24.5), Some(120.5), Some(2.0)))?;
    assert!(
        !result.changed && result.file_operation.is_none() && result.ui_file_operation.is_none()
    );
    assert_eq!(serde_json::to_value(e.state())?, before);
    assert_eq!(e.state().redo_depth, 1);
    assert!(selection.is_current(&e) && e.capture_stamp_matches(&stamp));
    assert!(e.ui_file_context_stamp_matches(&file));
    assert!(Arc::ptr_eq(&model, &e.stored_document()?.model_snapshot()));
    e.dispatch("ui.file", json!({"action":"save","expected_revision":3}))?;
    let pending = serde_json::to_value(e.state())?;
    let pending_file = e.ui_file_context_stamp();
    let result = e.dispatch(COMBINED, triple(&e, Some(24.5), Some(120.5), Some(2.0)))?;
    assert!(
        !result.changed && result.file_operation.is_none() && result.ui_file_operation.is_none()
    );
    assert_eq!(serde_json::to_value(e.state())?, pending);
    assert!(e.ui_file_context_stamp_matches(&pending_file));
    assert!(Arc::ptr_eq(&model, &e.stored_document()?.model_snapshot()));
    Ok(())
}
#[test]
fn closed_required_nullable_triple_validates_before_any_partial_commit() -> TestResult {
    let rows = bondline_commands::registry();
    assert_eq!(rows.len(), 53);
    assert_eq!(rows.get(45).ok_or("settings row46")?.name, COMBINED);
    let row = rows.get(45).ok_or("settings row46")?;
    assert!(!row.read_only);
    let schema = row.input_schema.as_value();
    assert_eq!(schema["type"], "object");
    assert_eq!(schema["additionalProperties"], false);
    let required = schema["required"]
        .as_array()
        .ok_or("required")?
        .iter()
        .map(|v| v.as_str().ok_or("required name"))
        .collect::<Result<BTreeSet<_>, _>>()?;
    assert_eq!(
        required,
        BTreeSet::from([
            "expected_revision",
            "bond_length_pt",
            "chain_angle_degrees",
            "line_width_pt"
        ])
    );
    assert_eq!(
        schema["properties"].as_object().ok_or("properties")?.len(),
        5
    );
    for key in [
        "bond_length_pt",
        "chain_angle_degrees",
        "line_width_pt",
        "bond_spacing_percent",
    ] {
        let types = schema["properties"][key]["type"]
            .as_array()
            .ok_or("nullable types")?;
        assert_eq!(types.len(), 2);
        assert!(types.contains(&json!("number")) && types.contains(&json!("null")));
    }
    for key in ["bond_length_pt", "line_width_pt"] {
        assert_eq!(schema["properties"][key]["exclusiveMinimum"], 0.0);
        assert_eq!(schema["properties"][key]["maximum"], 100_000.0);
    }
    assert_eq!(schema["properties"]["chain_angle_degrees"]["minimum"], 1.0);
    assert_eq!(schema["properties"]["bond_spacing_percent"]["minimum"], 1.0);
    assert_eq!(
        schema["properties"]["bond_spacing_percent"]["maximum"],
        100.0
    );
    assert_eq!(
        schema["properties"]["chain_angle_degrees"]["maximum"],
        179.0
    );
    let old = rows.get(40).ok_or("old pair row")?.input_schema.as_value();
    assert_eq!(rows[40].name, PAIR);
    assert_eq!(
        old["properties"].as_object().ok_or("old properties")?.len(),
        3
    );
    assert!(old["properties"].get("line_width_pt").is_none());
    let mut e = Engine::default();
    for key in [
        "expected_revision",
        "bond_length_pt",
        "chain_angle_degrees",
        "line_width_pt",
    ] {
        let mut p = triple(&e, Some(24.5), Some(120.5), Some(2.0));
        p.as_object_mut().ok_or("params")?.remove(key);
        reject(&mut e, p, "invalid_params")?;
    }
    for (key, value) in [
        ("line_width_pt", json!(0)),
        ("line_width_pt", json!(-1)),
        ("line_width_pt", json!(100_000.000_1)),
        ("line_width_pt", json!("2")),
        ("line_width_pt", json!(true)),
        ("bond_length_pt", json!(0)),
        ("chain_angle_degrees", json!(179.001)),
        ("expected_revision", json!(-1)),
        ("unexpected", json!(true)),
    ] {
        let mut p = triple(&e, Some(24.5), Some(120.5), Some(2.0));
        p[key] = value;
        reject(&mut e, p, "invalid_params")?;
    }
    for p in [Value::Null, json!([0, 24.5, 120.5, 2])] {
        reject(&mut e, p, "invalid_params")?;
    }
    e.dispatch(
        COMBINED,
        triple(&e, Some(100_000.0), Some(1.0), Some(100_000.0)),
    )?;
    assert_eq!(
        serde_json::to_value(e.document()?)?["drawing_style"]["line_width_pt"],
        100_000.0
    );
    e.dispatch(COMBINED, triple(&e, None, Some(179.0), None))?;
    assert!(
        serde_json::to_value(e.document()?)?
            .get("drawing_style")
            .is_none()
    );
    Ok(())
}
#[test]
fn source_revision_and_committed_close_guards_precede_noop_or_domain_checks() -> TestResult {
    let source =
        String::from_utf8(NATIVE.to_vec())?.replace("<CDXML ", "<CDXML Private=\"retain\" ");
    let mut opaque = Engine::from_document(StoredDocument::from_cdxml(
        source.as_bytes().into(),
        PageSize::A4,
    )?);
    assert!(!opaque.stored_document()?.is_fully_mapped());
    for p in [
        triple(&opaque, None, None, None),
        triple(&opaque, Some(0.0), Some(180.0), Some(0.0)),
    ] {
        reject(&mut opaque, p, "unsupported_content")?;
    }
    assert_eq!(opaque.stored_document()?.export_cdxml()?, source.as_bytes());
    let mut e = native()?;
    e.dispatch(COMBINED, triple(&e, None, None, Some(2.0)))?;
    reject(
        &mut e,
        json!({"expected_revision":0,"bond_length_pt":0,
        "chain_angle_degrees":180,"line_width_pt":0}),
        "stale_revision",
    )?;
    let mut closed = native()?;
    closed.dispatch(
        "document.close",
        json!({"expected_revision":0,"action":"request"}),
    )?;
    assert!(closed.complete_close());
    reject(
        &mut closed,
        json!({"expected_revision":99,"bond_length_pt":0,
        "chain_angle_degrees":180,"line_width_pt":0}),
        "close_committed",
    )?;
    Ok(())
}
#[test]
fn whole_page_ink_refusals_leave_both_creation_preferences_and_width_unchanged() -> TestResult {
    let mut e = Engine::default();
    e.dispatch(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":144,"begin_y_pt":300,
        "end_x_pt":224,"end_y_pt":300,"order":"double"}),
    )?;
    e.dispatch(COMBINED, triple(&e, Some(24.5), Some(120.5), Some(2.0)))?;
    let gap = triple(&e, Some(48.0), Some(90.0), Some(3.0));
    reject(&mut e, gap, "unsupported_scale")?;
    let tiny = triple(&e, Some(48.0), Some(90.0), Some(1.0e-10));
    reject(&mut e, tiny, "unsupported_scale")?;
    let result = e.dispatch(
        COMBINED,
        triple(&e, Some(100_000.0), Some(179.0), Some(2.0)),
    )?;
    assert!(
        result.changed,
        "unchanged width retains existing pair behavior"
    );
    assert_eq!(
        serde_json::to_value(e.document()?)?["drawing_style"],
        json!({"line_width_pt":2.0})
    );
    Ok(())
}
#[test]
fn captured_save_keeps_its_old_snapshot_lease_while_the_triple_stays_dirty() -> TestResult {
    let dir = support::FileTestDirectory::new()?;
    let input = dir.path("settings-original-λ.cdxml");
    let output = dir.path("settings-captured-化学.cdxml");
    let blocked = dir.path("never-published.cdxml");
    fs::write(&input, NATIVE)?;
    let mut e = Engine::default();
    e.dispatch("document.open_cdxml", json!({"path":input}))?;
    let work = e.prepare(Command::parse(
        "document.save_cdxml",
        json!({"path":output}),
    )?)?;
    let result = e.dispatch(COMBINED, triple(&e, Some(24.5), Some(120.5), Some(2.0)))?;
    assert_eq!((result.revision, result.undo_depth), (2, 1));
    let second = || Command::parse("document.save_cdxml", json!({"path":blocked}));
    assert_eq!(
        e.prepare(second()?).err().ok_or("save lease")?.info().code,
        "busy"
    );
    let done = work.complete()?;
    assert_eq!(fs::read(&output)?, NATIVE);
    assert_eq!(
        e.prepare(second()?)
            .err()
            .ok_or("publication released lease")?
            .info()
            .code,
        "busy"
    );
    let result = e.finish(done)?;
    assert!(!result.changed && result.file.dirty && result.source.ok_or("source")?.modified);
    assert_eq!(result.file_operation.ok_or("receipt")?.input_revision, 1);
    assert_eq!(
        serde_json::to_value(e.document()?)?["drawing_style"],
        json!({"line_width_pt":2.0})
    );
    assert_eq!(fs::read(&input)?, NATIVE);
    assert!(!blocked.exists());
    e.dispatch("history.undo", json!({}))?;
    assert!(!e.state().file.dirty);
    assert_eq!(e.stored_document()?.export_cdxml()?, NATIVE);
    drop(e.prepare(second()?)?);
    Ok(())
}

#[test]
fn actual_registry_capture_and_native_camera_use_authored_ink_without_authoring() -> TestResult {
    use std::process::{Command as Process, Stdio};
    let mut e = Engine::default();
    e.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":256,"height_pt":256}),
    )?;
    e.dispatch(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":32,"begin_y_pt":64.125,"end_x_pt":96,"end_y_pt":64.125}),
    )?;
    let pages = serde_json::to_value(e.document()?)?["pages"].clone();
    let mut images = Vec::new();
    for width in [0.5, 2.0] {
        e.dispatch(COMBINED, triple(&e, None, None, Some(width)))?;
        let before = serde_json::to_value(e.state())?;
        let stamp = e.capture_stamp();
        let view = e.page_view(576, 576, bondline_model::PageId::new(1).ok_or("page")?)?;
        assert_eq!(view.stroke_width(), 2.0);
        assert_eq!(view.ink_width(), (width * 2.0) as f32);
        for current in [false, true] {
            let mut pair = Vec::new();
            for backend in ["cpu", "gpu"] {
                let mut params =
                    json!({"page_id":1,"width_px":576,"height_px":576,"backend":backend});
                if current {
                    params["view"] = json!({"logical_width":576,"logical_height":576,"expected_view_epoch":e.view().epoch});
                }
                let capture = e
                    .dispatch("render.capture", params)?
                    .capture
                    .ok_or("capture")?;
                pair.push(capture.png_base64);
                assert_eq!(serde_json::to_value(e.state())?, before);
                assert!(e.capture_stamp_matches(&stamp));
                assert_eq!(serde_json::to_value(e.document()?)?["pages"], pages);
            }
            images.push(json!({"width_pt":width,"pngs":pair}));
        }
    }
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child=Process::new(python).args(["-X","utf8","-B","-c",r#"
import base64,io,json,sys
from PIL import Image
rows=json.load(sys.stdin)
assert len(rows)==4
for row in rows:
 images=[Image.open(io.BytesIO(base64.b64decode(p,validate=True))).convert('RGBA') for p in row['pngs']]
 assert len(images)==2 and images[0].size==images[1].size==(576,576)
 assert images[0].tobytes()==images[1].tobytes()
 w=row['width_pt']*2
 for y in range(154,168):
  for x in range(130,191):
   black=abs((y+0.5)-160.25)<=w/2
   assert images[0].getpixel((x,y))==((0,0,0,255) if black else (255,255,255,255)),(row['width_pt'],x,y)
print(json.dumps({'captures':8,'strip_cases':4}))
"#]).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
    serde_json::to_writer(child.stdin.take().ok_or("Pillow stdin")?, &images)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout)?,
        json!({"captures":8,"strip_cases":4})
    );
    Ok(())
}
