use bondline_commands::Engine;
use serde_json::json;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn complete_settings_accept_explicit_spacing_as_one_edit() -> TestResult {
    let mut engine = Engine::default();
    let result = engine.dispatch(
        "document.set_document_settings",
        json!({
            "expected_revision":0,"bond_length_pt":40.0,"chain_angle_degrees":120.0,
            "line_width_pt":0.6,"bond_spacing_percent":18.0
        }),
    )?;
    assert!(result.changed && result.file.dirty);
    assert_eq!(
        (result.revision, result.undo_depth, result.redo_depth),
        (1, 1, 0)
    );
    assert_eq!(
        serde_json::to_value(engine.document()?)?["drawing_style"],
        json!({"line_width_pt":0.6,"bond_spacing_percent":18.0})
    );
    Ok(())
}

// Independent additive acceptance; the initial existing-JSON declaration above is exact.
use bondline_commands::{Command, DocumentSettingsSet};
use bondline_io::StoredDocument;
use bondline_model::PageSize;
use serde_json::Value;
use std::{collections::BTreeSet, sync::Arc};

const SETTINGS: &str = "document.set_document_settings";
const NATIVE: &[u8] = include_bytes!("../../io/tests/fixtures/bondline-v0.cdxml");
fn native() -> TestResult<Engine> {
    Ok(Engine::from_document(StoredDocument::from_cdxml(
        NATIVE.into(),
        PageSize::A4,
    )?))
}
fn patch(
    e: &Engine,
    length: Option<f64>,
    angle: Option<f64>,
    width: Option<f64>,
    spacing: Option<Option<f64>>,
) -> Value {
    let mut value = json!({"expected_revision":e.state().revision,
        "bond_length_pt":length,"chain_angle_degrees":angle,"line_width_pt":width});
    if let Some(spacing) = spacing {
        value["bond_spacing_percent"] = json!(spacing);
    }
    value
}
fn rejected(e: &mut Engine, params: Value, code: &str) -> TestResult {
    let before = serde_json::to_value(e.state())?;
    let content = e.content().clone();
    let model = e.stored_document()?.model_snapshot();
    let stamp = e.capture_stamp();
    let file = e.ui_file_context_stamp();
    let error = e
        .dispatch(SETTINGS, params)
        .err()
        .ok_or("invalid settings accepted")?;
    assert_eq!(error.info().code, code, "{error}");
    assert_eq!(serde_json::to_value(e.state())?, before);
    assert_eq!(e.content(), &content);
    assert!(Arc::ptr_eq(&model, &e.stored_document()?.model_snapshot()));
    assert!(e.capture_stamp_matches(&stamp) && e.ui_file_context_stamp_matches(&file));
    Ok(())
}
fn open_form(e: &mut Engine) -> TestResult<u64> {
    e.dispatch(
        "ui.drawing_settings",
        json!({"action":"open","expected_revision":e.state().revision}),
    )?;
    Ok(e.ui_drawing_settings_request()
        .ok_or("settings request")?
        .id)
}

#[test]
fn optional_spacing_preserves_clears_and_noops_without_losing_redo_or_source() -> TestResult {
    let mut e = native()?;
    let original = e.stored_document()?.model_snapshot();
    let pages = serde_json::to_value(e.document()?)?["pages"].clone();
    e.dispatch(
        SETTINGS,
        patch(&e, Some(40.0), Some(120.0), Some(0.6), Some(Some(18.0))),
    )?;
    let eighteen = e.stored_document()?.model_snapshot();
    e.dispatch(
        SETTINGS,
        patch(&e, Some(40.0), Some(120.0), Some(0.6), Some(Some(20.0))),
    )?;
    e.dispatch("history.undo", json!({}))?;
    assert!(Arc::ptr_eq(
        &eighteen,
        &e.stored_document()?.model_snapshot()
    ));
    assert_eq!(e.state().redo_depth, 1);
    for spacing in [None, Some(Some(18.0))] {
        let before = serde_json::to_value(e.state())?;
        let stamp = e.capture_stamp();
        let result = e.dispatch(
            SETTINGS,
            patch(&e, Some(40.0), Some(120.0), Some(0.6), spacing),
        )?;
        assert!(
            !result.changed
                && result.file_operation.is_none()
                && result.ui_file_operation.is_none()
        );
        assert_eq!(serde_json::to_value(e.state())?, before);
        assert!(Arc::ptr_eq(
            &eighteen,
            &e.stored_document()?.model_snapshot()
        ));
        assert!(e.capture_stamp_matches(&stamp));
    }
    e.dispatch("history.redo", json!({}))?;
    assert_eq!(
        serde_json::to_value(e.document()?)?["drawing_style"],
        json!({"line_width_pt":0.6,"bond_spacing_percent":20.0})
    );
    let cleared = e.dispatch(
        SETTINGS,
        patch(&e, Some(40.0), Some(120.0), Some(0.6), Some(None)),
    )?;
    assert!(cleared.changed && cleared.file.dirty && cleared.source.ok_or("source")?.modified);
    assert!(cleared.file_operation.is_none() && cleared.ui_file_operation.is_none());
    assert_eq!(
        serde_json::to_value(e.document()?)?["drawing_style"],
        json!({"line_width_pt":0.6})
    );
    assert_eq!(serde_json::to_value(e.document()?)?["pages"], pages);
    e.dispatch("history.undo", json!({}))?;
    assert_eq!(
        serde_json::to_value(e.document()?)?["drawing_style"]["bond_spacing_percent"],
        20.0
    );
    // Undo both actual settings edits back to the complete original owner.
    e.dispatch("history.undo", json!({}))?;
    e.dispatch("history.undo", json!({}))?;
    assert!(Arc::ptr_eq(
        &original,
        &e.stored_document()?.model_snapshot()
    ));
    assert!(!e.state().file.dirty);
    assert_eq!(e.stored_document()?.export_cdxml()?, NATIVE);
    Ok(())
}

#[test]
fn legacy_pair_and_native_apply_three_preserve_unaddressed_spacing() -> TestResult {
    let mut e = native()?;
    e.dispatch(
        SETTINGS,
        patch(&e, Some(40.0), Some(120.0), Some(0.6), Some(Some(18.0))),
    )?;
    let pages = serde_json::to_value(e.document()?)?["pages"].clone();
    e.dispatch(
        "document.set_drawing_settings",
        json!({"expected_revision":e.state().revision,
        "bond_length_pt":50.0,"chain_angle_degrees":90.0}),
    )?;
    assert_eq!(
        serde_json::to_value(e.document()?)?["drawing_style"],
        json!({"line_width_pt":0.6,"bond_spacing_percent":18.0})
    );
    e.dispatch(
        SETTINGS,
        patch(&e, Some(48.0), Some(120.0), Some(2.0), None),
    )?;
    assert_eq!(
        serde_json::to_value(e.document()?)?["drawing_style"],
        json!({"line_width_pt":2.0,"bond_spacing_percent":18.0})
    );
    e.set_ui_file_host(true);
    let id = open_form(&mut e)?;
    let result = e.apply_ui_document_settings(id, Some(40.0), Some(120.0), Some(0.6))?;
    assert!(
        result.changed && result.file_operation.is_none() && result.ui_file_operation.is_none()
    );
    assert_eq!(
        serde_json::to_value(e.document()?)?["drawing_style"],
        json!({"line_width_pt":0.6,"bond_spacing_percent":18.0})
    );
    assert!(e.ui_drawing_settings_request().is_none());
    let id = open_form(&mut e)?;
    e.apply_ui_drawing_settings(id, Some(50.0), Some(90.0))?;
    assert_eq!(
        serde_json::to_value(e.document()?)?["drawing_style"],
        json!({"line_width_pt":0.6,"bond_spacing_percent":18.0})
    );
    assert_eq!(serde_json::to_value(e.document()?)?["pages"], pages);
    Ok(())
}

#[test]
fn final_settings_are_certified_once_and_preferred_only_changes_can_close_the_gap() -> TestResult {
    let mut e = Engine::default();
    e.dispatch(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":144,"begin_y_pt":300,
        "end_x_pt":224,"end_y_pt":300,"order":"double"}),
    )?;
    let pages = serde_json::to_value(e.document()?)?["pages"].clone();
    e.dispatch(
        SETTINGS,
        patch(&e, Some(20.0), Some(120.0), Some(4.0), Some(Some(40.0))),
    )?;
    // Old and final separation are both 8pt. The unwanted intermediate L10/p40/w4
    // would have zero clear gap; it is not a candidate that the user submitted.
    let before = e.state();
    let result = e.dispatch(
        SETTINGS,
        patch(&e, Some(10.0), Some(90.0), Some(2.0), Some(Some(80.0))),
    )?;
    assert!(result.changed);
    assert_eq!(
        (result.revision, result.undo_depth),
        (before.revision + 1, before.undo_depth + 1)
    );
    assert_eq!(
        serde_json::to_value(e.document()?)?["drawing_settings"],
        json!({"bond_length_pt":10.0,"chain_angle_degrees":90.0})
    );
    assert_eq!(
        serde_json::to_value(e.document()?)?["drawing_style"],
        json!({"line_width_pt":2.0,"bond_spacing_percent":80.0})
    );
    assert_eq!(serde_json::to_value(e.document()?)?["pages"], pages);
    let collapsed = patch(&e, Some(2.0), Some(120.0), Some(2.0), None);
    rejected(&mut e, collapsed, "unsupported_scale")?;
    let before = serde_json::to_value(e.state())?;
    let model = e.stored_document()?.model_snapshot();
    let error = e
        .dispatch(
            "document.set_drawing_settings",
            json!({"expected_revision":e.state().revision,
        "bond_length_pt":2.0,"chain_angle_degrees":120.0}),
        )
        .err()
        .ok_or("preferred-only pair ignored spacing")?;
    assert_eq!(error.info().code, "unsupported_scale");
    assert_eq!(serde_json::to_value(e.state())?, before);
    assert!(Arc::ptr_eq(&model, &e.stored_document()?.model_snapshot()));
    let collapsed = patch(&e, Some(10.0), Some(90.0), Some(2.0), Some(Some(1.0)));
    rejected(&mut e, collapsed, "unsupported_scale")?;
    Ok(())
}

#[test]
fn optional_nullable_schema_domain_and_source_errors_are_closed_and_atomic() -> TestResult {
    let rows = bondline_commands::registry();
    assert_eq!(rows.len(), 50);
    let row = rows.get(45).ok_or("combined settings row46")?;
    assert_eq!(row.name, SETTINGS);
    let schema = row.input_schema.as_value();
    assert_eq!(schema["type"], "object");
    assert_eq!(schema["additionalProperties"], false);
    let required = schema["required"]
        .as_array()
        .ok_or("required")?
        .iter()
        .map(|v| v.as_str().ok_or("name"))
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
    let spacing = &schema["properties"]["bond_spacing_percent"];
    let kinds = spacing["type"].as_array().ok_or("nullable spacing")?;
    assert_eq!(kinds.len(), 2);
    assert!(kinds.contains(&json!("number")) && kinds.contains(&json!("null")));
    assert_eq!(
        (spacing["minimum"].as_f64(), spacing["maximum"].as_f64()),
        (Some(1.0), Some(100.0))
    );
    let mut e = Engine::default();
    for bad in [
        json!(0),
        json!(-1),
        json!(100.000_1),
        json!("18"),
        json!(true),
        json!({}),
        json!([]),
    ] {
        let mut p = patch(&e, None, None, None, None);
        p["bond_spacing_percent"] = bad;
        rejected(&mut e, p, "invalid_params")?;
    }
    let mut extra = patch(&e, None, None, None, Some(Some(18.0)));
    extra["spacing"] = json!(18);
    rejected(&mut e, extra, "invalid_params")?;
    for value in [1.0, 100.0] {
        e.dispatch(SETTINGS, patch(&e, None, None, None, Some(Some(value))))?;
    }
    let mut stale = patch(&e, None, None, None, Some(Some(0.0)));
    stale["expected_revision"] = json!(0);
    rejected(&mut e, stale, "stale_revision")?;
    let source =
        String::from_utf8(NATIVE.to_vec())?.replace("<CDXML ", "<CDXML Private=\"retain\" ");
    let mut opaque = Engine::from_document(StoredDocument::from_cdxml(
        source.as_bytes().into(),
        PageSize::A4,
    )?);
    assert!(!opaque.stored_document()?.is_fully_mapped());
    let params = patch(&opaque, None, None, None, Some(Some(18.0)));
    rejected(&mut opaque, params, "unsupported_content")?;
    assert_eq!(opaque.stored_document()?.export_cdxml()?, source.as_bytes());
    Ok(())
}

#[test]
fn typed_tristate_serialization_and_four_field_apply_keep_the_same_weak_draft() -> TestResult {
    let mut e = native()?;
    for (spacing, expected) in [
        (None, None),
        (Some(None), Some(Value::Null)),
        (Some(Some(18.0)), Some(json!(18.0))),
    ] {
        let dto: DocumentSettingsSet =
            serde_json::from_value(patch(&e, None, None, None, spacing))?;
        assert_eq!(dto.bond_spacing_percent, spacing);
        let encoded = serde_json::to_value(dto)?;
        assert_eq!(encoded.get("bond_spacing_percent").cloned(), expected);
    }
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut dto: DocumentSettingsSet =
            serde_json::from_value(patch(&e, None, None, None, None))?;
        dto.bond_spacing_percent = Some(Some(bad));
        assert!(serde_json::to_value(Command::DocumentSettingsSet(dto.clone())).is_err());
        let before = serde_json::to_value(e.state())?;
        let stamp = e.capture_stamp();
        let error = e
            .execute(Command::DocumentSettingsSet(dto))
            .err()
            .ok_or("typed nonfinite accepted")?;
        assert_eq!(error.info().code, "invalid_params");
        assert_eq!(serde_json::to_value(e.state())?, before);
        assert!(e.capture_stamp_matches(&stamp));
    }
    e.dispatch(
        SETTINGS,
        patch(&e, Some(40.0), Some(120.0), Some(0.6), Some(Some(18.0))),
    )?;
    e.set_ui_file_host(true);
    let id = open_form(&mut e)?;
    let request = serde_json::to_value(e.ui_drawing_settings_request().ok_or("request")?)?;
    assert_eq!(request["bond_spacing_percent"], 18.0);
    let before = serde_json::to_value(e.state())?;
    let error = e
        .apply_ui_document_settings_with_spacing(id, Some(40.0), Some(120.0), Some(0.6), Some(0.0))
        .err()
        .ok_or("out-of-domain form accepted")?;
    assert_eq!(error.info().code, "invalid_params");
    assert_eq!(serde_json::to_value(e.state())?, before);
    assert!(e.ui_drawing_settings_request_current(id));
    let model = e.stored_document()?.model_snapshot();
    let result = e.apply_ui_document_settings_with_spacing(
        id,
        Some(40.0),
        Some(120.0),
        Some(0.6),
        Some(18.0),
    )?;
    assert!(
        !result.changed && result.file_operation.is_none() && result.ui_file_operation.is_none()
    );
    assert!(Arc::ptr_eq(&model, &e.stored_document()?.model_snapshot()));
    assert!(e.ui_drawing_settings_request().is_none());
    let id = open_form(&mut e)?;
    e.dispatch("ui.view", json!({"action":"actual"}))?;
    e.dispatch("ui.view", json!({"action":"fit"}))?;
    assert!(!e.ui_drawing_settings_request_current(id));
    let before = serde_json::to_value(e.state())?;
    let error = e
        .apply_ui_document_settings_with_spacing(id, None, None, None, None)
        .err()
        .ok_or("stale form applied")?;
    assert_eq!(error.info().code, "ui_stale_request");
    assert_eq!(serde_json::to_value(e.state())?, before);
    e.dispatch(
        "ui.drawing_settings",
        json!({"action":"cancel","expected_revision":e.state().revision,"request_id":id}),
    )?;
    assert!(e.ui_drawing_settings_request().is_none());
    Ok(())
}

#[test]
fn actual_capture_uses_current_preferred_length_and_independent_width_on_both_backends()
-> TestResult {
    use std::process::{Command as Process, Stdio};
    let mut e = Engine::default();
    e.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":256,"height_pt":256}),
    )?;
    e.dispatch(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":32,"begin_y_pt":64.125,
        "end_x_pt":112,"end_y_pt":64.125,"order":"double"}),
    )?;
    let pages = serde_json::to_value(e.document()?)?["pages"].clone();
    let mut images = Vec::new();
    for (preferred, width, centers, ink_width) in [
        (40.0, 0.6, [153.05, 167.45], 1.2),
        (80.0, 0.6, [145.85, 174.65], 1.2),
        (80.0, 1.2, [145.85, 174.65], 2.4),
    ] {
        e.dispatch(
            SETTINGS,
            patch(&e, Some(preferred), None, Some(width), Some(Some(18.0))),
        )?;
        let before = serde_json::to_value(e.state())?;
        let stamp = e.capture_stamp();
        let view = e.page_view(576, 576, bondline_model::PageId::new(1).ok_or("page")?)?;
        let graph = view.graph_scene(
            &e.document()?.pages()[0],
            &bondline_render::Cancellation::default(),
        )?;
        let strokes = graph
            .bond_strokes(
                bondline_model::FragmentId::new(1).ok_or("fragment")?,
                bondline_model::BondId::new(1).ok_or("bond")?,
            )
            .collect::<Vec<_>>();
        assert_eq!(strokes.len(), 2);
        for (stroke, expected) in strokes.iter().zip(centers) {
            assert!((f64::from(stroke.begin[1]) - expected).abs() < 0.0001);
            assert!((f64::from(stroke.width) - ink_width).abs() < 0.0001);
        }
        for current in [false, true] {
            let mut pair = Vec::new();
            for backend in ["cpu", "gpu"] {
                let mut params =
                    json!({"page_id":1,"width_px":576,"height_px":576,"backend":backend});
                if current {
                    params["view"] = json!({"logical_width":576,"logical_height":576,"expected_view_epoch":e.view().epoch});
                }
                pair.push(
                    e.dispatch("render.capture", params)?
                        .capture
                        .ok_or("capture")?
                        .png_base64,
                );
                assert_eq!(serde_json::to_value(e.state())?, before);
                assert!(e.capture_stamp_matches(&stamp));
                assert_eq!(serde_json::to_value(e.document()?)?["pages"], pages);
            }
            images.push(json!({"centers":centers,"width":ink_width,"pngs":pair}));
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
records=json.load(sys.stdin)
assert len(records)==6
for row in records:
 images=[Image.open(io.BytesIO(base64.b64decode(p,validate=True))).convert('RGBA') for p in row['pngs']]
 assert len(images)==2 and images[0].size==images[1].size==(576,576)
 assert images[0].tobytes()==images[1].tobytes()
 for y in range(135,185):
  for x in range(144,208):
   black=any(abs(y+0.5-center)<=row['width']/2 for center in row['centers'])
   assert images[0].getpixel((x,y))==((0,0,0,255) if black else (255,255,255,255)),(row,x,y)
print(json.dumps({'captures':12,'analytic_strips':6}))
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
        json!({"captures":12,"analytic_strips":6})
    );
    Ok(())
}
