//! Independent transient view contract; no new production Rust types are required.
use bondline_commands::Engine;
use serde_json::{Value, json};
use std::sync::Arc;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn registered_view_modes_centres_and_noops_preserve_complete_authored_state_and_capture_identity()
-> TestResult {
    let mut engine = Engine::default();
    let before = serde_json::to_value(engine.state())?;
    let content = engine.content().clone();
    let model = engine.stored_document()?.model_snapshot();
    let stamp = engine.capture_stamp();
    for (params, zoom, center, epoch, changed) in [
        (
            json!({"action":"inspect"}),
            Value::Null,
            Value::Null,
            0,
            false,
        ),
        (
            json!({"action":"set_zoom","percent":125}),
            json!(125.0),
            Value::Null,
            1,
            true,
        ),
        (
            json!({"action":"set_zoom","percent":125}),
            json!(125.0),
            Value::Null,
            1,
            false,
        ),
        (
            json!({"action":"pan","center_x_pt":72,"center_y_pt":144}),
            json!(125.0),
            json!([72.0, 144.0]),
            2,
            true,
        ),
        (
            json!({"action":"set_zoom","percent":200}),
            json!(200.0),
            json!([72.0, 144.0]),
            3,
            true,
        ),
        (
            json!({"action":"actual"}),
            json!(100.0),
            json!([72.0, 144.0]),
            4,
            true,
        ),
        (json!({"action":"fit"}), Value::Null, Value::Null, 5, true),
        (json!({"action":"fit"}), Value::Null, Value::Null, 5, false),
        (
            json!({"action":"inspect"}),
            Value::Null,
            Value::Null,
            5,
            false,
        ),
    ] {
        let mut reply = serde_json::to_value(engine.dispatch("ui.view", params.clone())?)?;
        let operation = reply
            .as_object_mut()
            .ok_or("reply")?
            .remove("ui_view_operation")
            .ok_or("explicit view operation missing")?;
        assert_eq!(
            operation,
            json!({
                "zoom_percent":zoom,
                "center_x_pt":center.get(0).cloned().unwrap_or(Value::Null),
                "center_y_pt":center.get(1).cloned().unwrap_or(Value::Null),
                "epoch":epoch,"changed":changed
            }),
            "typed view transition {params}"
        );
        assert_eq!(
            reply, before,
            "explicit reply contains unchanged authored state"
        );
        assert_eq!(serde_json::to_value(engine.state())?, before);
        assert_eq!(
            engine.content(),
            &content,
            "complete backing must remain unchanged"
        );
        assert!(Arc::ptr_eq(
            &engine.stored_document()?.model_snapshot(),
            &model
        ));
        assert!(
            engine.capture_stamp_matches(&stamp),
            "whole-page capture identity ignores window view"
        );
    }
    Ok(())
}

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(
    dead_code,
    reason = "reuse only the existing confined temporary directory"
)]
mod support;

fn view(engine: &mut Engine, params: Value) -> TestResult<Value> {
    let reply = serde_json::to_value(engine.dispatch("ui.view", params)?)?;
    assert_eq!(reply["changed"], false);
    reply
        .get("ui_view_operation")
        .cloned()
        .ok_or_else(|| "view operation missing".into())
}

fn refusal(engine: &mut Engine, params: Value, code: &str) -> TestResult {
    let state = serde_json::to_value(engine.state())?;
    let prior = view(engine, json!({"action":"inspect"}))?;
    let error = engine
        .dispatch("ui.view", params.clone())
        .err()
        .ok_or("invalid view accepted")?;
    assert_eq!(error.info().code, code, "{params}");
    assert_eq!(serde_json::to_value(engine.state())?, state);
    assert_eq!(view(engine, json!({"action":"inspect"}))?, prior);
    Ok(())
}

#[test]
fn closed_view_parameters_and_busy_guards_are_checked_before_noop() -> TestResult {
    let mut engine = Engine::default();
    view(&mut engine, json!({"action":"actual"}))?;
    for params in [
        json!({}),
        json!({"action":"rotate"}),
        json!({"action":"inspect","percent":100}),
        json!({"action":"actual","width_px":100}),
        json!({"action":"set_zoom"}),
        json!({"action":"set_zoom","percent":null}),
        json!({"action":"set_zoom","percent":"100"}),
        json!({"action":"set_zoom","percent":0.999}),
        json!({"action":"set_zoom","percent":999.001}),
        json!({"action":"set_zoom","percent":1e300}),
        json!({"action":"magnify","width_px":0,"height_px":100}),
        json!({"action":"magnify","width_px":100}),
        json!({"action":"reduce","width_px":100001,"height_px":100}),
        json!({"action":"reduce","width_px":100,"height_px":1.5}),
        json!({"action":"pan","center_x_pt":-1,"center_y_pt":0}),
        json!({"action":"pan","center_x_pt":0}),
    ] {
        refusal(&mut engine, params, "invalid_params")?;
    }
    view(
        &mut engine,
        json!({"action":"pan","center_x_pt":0,"center_y_pt":0}),
    )?;
    let page = engine.document()?.pages().first().ok_or("page")?.size();
    view(
        &mut engine,
        json!({"action":"pan","center_x_pt":page.width_pt(),"center_y_pt":page.height_pt()}),
    )?;
    refusal(
        &mut engine,
        json!({"action":"pan","center_x_pt":page.width_pt()+0.001,"center_y_pt":0}),
        "invalid_params",
    )?;
    view(&mut engine, json!({"action":"fit"}))?;
    refusal(
        &mut engine,
        json!({"action":"pan","center_x_pt":0,"center_y_pt":0}),
        "unsupported_content",
    )?;
    view(&mut engine, json!({"action":"actual"}))?;
    engine.dispatch(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":100,"begin_y_pt":200,"end_x_pt":180,"end_y_pt":200}),
    )?;
    engine.dispatch(
        "document.close",
        json!({"expected_revision":engine.state().revision,"action":"request"}),
    )?;
    refusal(&mut engine, json!({"action":"actual"}), "ui_busy")?;
    engine.dispatch(
        "document.close",
        json!({"expected_revision":engine.state().revision,"action":"cancel"}),
    )?;
    engine.set_ui_file_host(true);
    engine.dispatch(
        "ui.file",
        json!({"expected_revision":engine.state().revision,"action":"open"}),
    )?;
    refusal(&mut engine, json!({"action":"actual"}), "ui_busy")?;
    Ok(())
}

#[test]
fn relative_zoom_uses_declared_fit_scale_and_never_reverses_direction_at_limits() -> TestResult {
    let mut engine = Engine::default();
    engine.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":500,"height_pt":500}),
    )?;
    let state = serde_json::to_value(engine.state())?;
    // Rounded legacy paper is936square at1000square, so Fit=187.2%.
    let magnified = view(
        &mut engine,
        json!({"action":"magnify","width_px":1000,"height_px":1000}),
    )?;
    assert!((magnified["zoom_percent"].as_f64().ok_or("zoom")? - 234.0).abs() < 1e-9);
    assert_eq!(magnified["center_x_pt"], Value::Null);
    view(&mut engine, json!({"action":"fit"}))?;
    let reduced = view(
        &mut engine,
        json!({"action":"reduce","width_px":1000,"height_px":1000}),
    )?;
    assert!((reduced["zoom_percent"].as_f64().ok_or("zoom")? - 149.76).abs() < 1e-9);
    for (percent, action) in [(999, "magnify"), (1, "reduce")] {
        view(&mut engine, json!({"action":"set_zoom","percent":percent}))?;
        let before = view(&mut engine, json!({"action":"inspect"}))?;
        assert_eq!(
            view(
                &mut engine,
                json!({"action":action,"width_px":1000,"height_px":1000})
            )?,
            before
        );
    }
    assert_eq!(serde_json::to_value(engine.state())?, state);
    for (size, dimension, action) in [(1, 100000, "magnify"), (100000, 1, "reduce")] {
        engine.dispatch(
            "page.set_size",
            json!({"page_id":1,"width_pt":size,"height_pt":size}),
        )?;
        view(&mut engine, json!({"action":"fit"}))?;
        let before = view(&mut engine, json!({"action":"inspect"}))?;
        assert_eq!(
            view(
                &mut engine,
                json!({"action":action,"width_px":dimension,"height_px":dimension})
            )?,
            before,
            "relative limit must preserve Fit rather than move in the opposite direction"
        );
    }
    Ok(())
}

#[test]
fn magnify_local_focus_and_window_view_have_distinct_history_and_open_lifetimes() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let mut engine = Engine::default();
    engine.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":500,"height_pt":500}),
    )?;
    view(&mut engine, json!({"action":"actual"}))?;
    view(
        &mut engine,
        json!({"action":"pan","center_x_pt":200,"center_y_pt":250}),
    )?;
    engine.dispatch(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":100,"begin_y_pt":100,"end_x_pt":140,"end_y_pt":140}),
    )?;
    let focus = view(
        &mut engine,
        json!({"action":"magnify","width_px":1000,"height_px":1000}),
    )?;
    assert_eq!(
        (
            focus["zoom_percent"].clone(),
            focus["center_x_pt"].clone(),
            focus["center_y_pt"].clone()
        ),
        (json!(125.0), json!(120.0), json!(120.0))
    );
    let reduce = view(
        &mut engine,
        json!({"action":"reduce","width_px":1000,"height_px":1000}),
    )?;
    assert_eq!(
        (
            reduce["zoom_percent"].clone(),
            reduce["center_x_pt"].clone()
        ),
        (json!(100.0), json!(120.0))
    );
    view(
        &mut engine,
        json!({"action":"pan","center_x_pt":200,"center_y_pt":250}),
    )?;
    engine.dispatch("bond.set_order",json!({"expected_revision":engine.state().revision,"page_id":1,"fragment_id":1,"bond_id":1,"order":"double","hydrogen_policy":"rectify_neutral_carbon"}))?;
    let focus = view(
        &mut engine,
        json!({"action":"magnify","width_px":1000,"height_px":1000}),
    )?;
    assert_eq!(
        (focus["center_x_pt"].clone(), focus["center_y_pt"].clone()),
        (json!(120.0), json!(120.0)),
        "position-preserving own order edit retains local draw focus"
    );
    view(
        &mut engine,
        json!({"action":"pan","center_x_pt":200,"center_y_pt":250}),
    )?;
    let before = view(&mut engine, json!({"action":"inspect"}))?;
    engine.dispatch("history.undo", json!({}))?;
    assert_eq!(
        view(&mut engine, json!({"action":"inspect"}))?,
        before,
        "authoring history cannot undo window view"
    );
    let after = view(
        &mut engine,
        json!({"action":"magnify","width_px":1000,"height_px":1000}),
    )?;
    assert_eq!(
        (after["center_x_pt"].clone(), after["center_y_pt"].clone()),
        (json!(200.0), json!(250.0)),
        "history invalidates last-drawn intent despite unchanged bond coordinates"
    );
    view(
        &mut engine,
        json!({"action":"pan","center_x_pt":400,"center_y_pt":450}),
    )?;
    let mut small = Engine::default();
    small.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":100,"height_pt":100}),
    )?;
    let path = directory.path("small-native.cdxml");
    small.dispatch("document.save_cdxml", json!({"path":path}))?;
    let before = view(&mut engine, json!({"action":"inspect"}))?;
    engine.dispatch(
        "document.open_cdxml",
        json!({"path":path,"discard_unsaved":true}),
    )?;
    assert_eq!(
        view(&mut engine, json!({"action":"inspect"}))?,
        before,
        "Open preserves stored centre and epoch; projection alone clamps to the new page"
    );
    let after = view(
        &mut engine,
        json!({"action":"magnify","width_px":1000,"height_px":1000}),
    )?;
    assert_eq!(
        (after["center_x_pt"].clone(), after["center_y_pt"].clone()),
        (json!(400.0), json!(450.0)),
        "import order is never last-drawn focus"
    );
    Ok(())
}

#[test]
fn source_guards_and_whole_page_capture_remain_independent_of_window_view() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let mut engine = Engine::default();
    engine.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":500,"height_pt":500}),
    )?;
    engine.dispatch(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":100,"begin_y_pt":200,"end_x_pt":200,"end_y_pt":200}),
    )?;
    let original = directory.path("complete-native.cdxml");
    engine.dispatch("document.save_cdxml", json!({"path":original}))?;
    engine.dispatch("document.open_cdxml", json!({"path":original}))?;
    let capture = json!({"page_id":1,"width_px":1024,"height_px":1024,"backend":"cpu"});
    let first = engine
        .dispatch("render.capture", capture.clone())?
        .capture
        .ok_or("PNG")?
        .png_bytes()?;
    let state = serde_json::to_value(engine.state())?;
    view(&mut engine, json!({"action":"set_zoom","percent":200}))?;
    view(
        &mut engine,
        json!({"action":"pan","center_x_pt":72,"center_y_pt":144}),
    )?;
    assert_eq!(
        engine
            .dispatch("render.capture", capture)?
            .capture
            .ok_or("PNG")?
            .png_bytes()?,
        first,
        "registered capture remains whole-page"
    );
    assert_eq!(serde_json::to_value(engine.state())?, state);
    let copy = directory.path("native-copy.cdxml");
    engine.dispatch("document.save_cdxml", json!({"path":copy}))?;
    assert_eq!(std::fs::read(copy)?, std::fs::read(&original)?);
    let mol = directory.path("read-only.mol");
    let raw = include_bytes!("../../io/tests/fixtures/mol/ethane-v2000.mol");
    std::fs::write(&mol, raw)?;
    engine.dispatch("document.open_mol", json!({"path":mol}))?;
    let state = serde_json::to_value(engine.state())?;
    view(&mut engine, json!({"action":"actual"}))?;
    assert_eq!(
        serde_json::to_value(engine.state())?,
        state,
        "view is allowed on complete read-only MOL display without gaining edit authority"
    );
    let copy = directory.path("mol-copy.mol");
    engine.dispatch("document.save_mol", json!({"path":copy}))?;
    assert_eq!(std::fs::read(copy)?, raw);
    let sdf = directory.path("whole.sdf");
    std::fs::write(
        &sdf,
        include_bytes!("../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf"),
    )?;
    engine.dispatch("document.open_sdf", json!({"path":sdf}))?;
    let state = serde_json::to_value(engine.state())?;
    let content = engine.content().clone();
    let stamp = engine.capture_stamp();
    // SDF now permits camera-only Actual without becoming an editable Drawing.
    let actual = view(&mut engine, json!({"action":"actual"}))?;
    assert_eq!(actual["zoom_percent"], 100.0);
    assert_eq!(serde_json::to_value(engine.state())?, state);
    assert_eq!(engine.content(), &content);
    assert_eq!(engine.capture_stamp(), stamp);
    assert!(engine.document().is_err());
    refusal(
        &mut engine,
        json!({"action":"pan","center_x_pt":100,"center_y_pt":144}),
        "viewport_unavailable",
    )?;
    let copy = directory.path("sdf-copy.sdf");
    engine.dispatch("document.save_sdf", json!({"path":copy}))?;
    assert_eq!(std::fs::read(copy)?, content.sdf()?.as_bytes());
    let extended = directory.path("opaque.cdxml");
    let text = String::from_utf8(std::fs::read(&original)?)?.replacen(
        "<CDXML",
        "<CDXML OpaqueViewTest=\"preserve\"",
        1,
    );
    std::fs::write(&extended, text.as_bytes())?;
    engine.dispatch("document.open_cdxml", json!({"path":extended}))?;
    refusal(
        &mut engine,
        json!({"action":"actual"}),
        "unsupported_content",
    )?;
    let copy = directory.path("opaque-copy.cdxml");
    engine.dispatch("document.save_cdxml", json!({"path":copy}))?;
    assert_eq!(std::fs::read(copy)?, text.as_bytes());
    Ok(())
}

#[test]
fn empty_actions_refuse_every_unused_field_before_any_state_or_epoch_change() -> TestResult {
    let mut engine = Engine::default();
    view(&mut engine, json!({"action":"actual"}))?;
    for action in ["inspect", "fit", "actual"] {
        for extra in [
            json!({"percent":100}),
            json!({"center_x_pt":0}),
            json!({"width_px":100,"height_px":100}),
            json!({"unused":true}),
        ] {
            let mut params = json!({"action":action});
            params
                .as_object_mut()
                .ok_or("params")?
                .extend(extra.as_object().ok_or("extra")?.clone());
            refusal(&mut engine, params, "invalid_params")?;
        }
    }
    Ok(())
}
