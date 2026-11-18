use super::*;

const SOURCE: &[u8] =
    include_bytes!("../../../io/tests/fixtures/sdf-preview/valence-absent-mixed.sdf");
const PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../io/tests/fixtures/sdf-preview/valence-absent-mixed.sdf"
);

#[test]
fn successful_current_sdf_extent_wakes_host_once_without_disclosing_internal_metadata() -> TestResult
{
    let mut initial = Engine::default();
    initial.dispatch("document.open_sdf", json!({"path":PATH}))?;
    initial.dispatch("ui.view", json!({"action":"actual"}))?;
    let before = serde_json::to_value(initial.state())?;
    let stamp = initial.capture_stamp();
    let engine: SharedEngine = Arc::new(Mutex::new(initial));
    let count = Arc::new(AtomicUsize::new(0));
    let on_change = Arc::clone(&count);
    let server = LoopbackServer::bind(0, Arc::clone(&engine), move || {
        on_change.fetch_add(1, Ordering::SeqCst);
    })?;
    let mut client = authenticated(&server)?;
    let inspect = query(&mut client, json!({"action":"inspect"}))?;
    assert!(
        inspect["result"]["ui_view_operation"]
            .get("prepared_sdf_extent")
            .is_none()
    );
    // Legacy whole-page capture has no view/cache effect, even after explicit Actual.
    let legacy = call(
        &mut client,
        json!({"command":"render.capture","params":{
            "page_id":1,"width_px":1024,"height_px":1024,"backend":"cpu"
        }}),
    )?;
    assert!(legacy.get("error").is_none(), "{legacy}");
    assert!(legacy["result"]["capture"]["png_base64"].is_string());
    let inspect = query(&mut client, json!({"action":"inspect"}))?;
    assert!(
        inspect["result"]["ui_view_operation"]
            .get("prepared_sdf_extent")
            .is_none()
    );
    assert_eq!(count.load(Ordering::SeqCst), 0);

    let params = json!({"page_id":1,"width_px":848,"height_px":651,"backend":"cpu",
        "view":{"logical_width":848,"logical_height":651,"expected_view_epoch":1}});
    let first = call(
        &mut client,
        json!({"command":"render.capture","params":params}),
    )?;
    assert!(first.get("error").is_none(), "{first}");
    assert!(first["result"]["capture"]["png_base64"].is_string());
    assert_eq!(first["result"]["changed"], false);
    assert!(first["result"].get("view_target_prepared").is_none());
    let inspect = query(&mut client, json!({"action":"inspect"}))?;
    let extent = &inspect["result"]["ui_view_operation"]["prepared_sdf_extent"];
    assert!((extent["width_pt"].as_f64().ok_or("width")? - 210.0 / 25.4 * 72.0).abs() < 1e-9);
    assert!((extent["height_pt"].as_f64().ok_or("height")? - 297.0 / 25.4 * 72.0).abs() < 1e-9);
    wait_for_count(&count, 1);
    let second = call(
        &mut client,
        json!({"command":"render.capture","params":params}),
    )?;
    assert_eq!(
        first, second,
        "cache preparation is absent from public capture JSON"
    );
    let state = call(
        &mut client,
        json!({"command":"document.inspect","params":{}}),
    )?;
    assert_eq!(state["result"], before);
    assert_eq!(
        count.load(Ordering::SeqCst),
        1,
        "an unchanged extent is not a new view change"
    );
    {
        let current = engine.lock().map_err(|_| "engine")?;
        assert_eq!(current.capture_stamp(), stamp);
        assert_eq!(current.content().sdf()?.as_bytes(), SOURCE);
        assert!(current.document().is_err());
    }
    let panned = query(
        &mut client,
        json!({"action":"pan","center_x_pt":100,"center_y_pt":144}),
    )?;
    assert!(panned.get("error").is_none(), "{panned}");
    let state = call(
        &mut client,
        json!({"command":"document.inspect","params":{}}),
    )?;
    assert_eq!(state["result"], before);
    wait_for_count(&count, 2);
    server.shutdown()?;
    Ok(())
}

#[test]
fn native_current_capture_rejects_an_obsolete_close_epoch_before_reserving_work() -> TestResult {
    use bondline_commands::{Cancellation, CaptureView, RenderBackend, RenderCapture};
    use bondline_mcp::CommandExecutor;

    let mut initial = Engine::default();
    initial.dispatch("document.open_sdf", json!({"path":PATH}))?;
    initial.dispatch("ui.view", json!({"action":"actual"}))?;
    let stamp = initial.capture_stamp();
    let close_epoch = initial.state().close_epoch;
    for action in ["request", "cancel"] {
        initial.dispatch(
            "document.close",
            json!({
                "expected_revision":initial.state().revision,"action":action
            }),
        )?;
    }
    assert!(initial.capture_stamp_matches(&stamp));
    assert_eq!(initial.state().close_epoch, close_epoch + 2);
    let before = serde_json::to_value(initial.state())?;
    let current_epoch = initial.state().close_epoch;
    let engine: SharedEngine = Arc::new(Mutex::new(initial));
    let executor = CommandExecutor::new(Arc::clone(&engine));
    let params = || RenderCapture {
        page_id: 1,
        width_px: 848,
        height_px: 651,
        backend: RenderBackend::Cpu,
        view: Some(CaptureView {
            logical_width: 848,
            logical_height: 651,
            expected_view_epoch: 1,
        }),
    };
    let error = executor
        .prepare_ui_capture_for_close(&stamp, close_epoch, params())
        .err()
        .ok_or("obsolete native close context was reserved")?;
    assert_eq!(error.code, "ui_stale_request");
    assert!(!executor.has_active_work());
    assert_eq!(
        serde_json::to_value(engine.lock().map_err(|_| "engine")?.state())?,
        before
    );
    assert!(
        engine
            .lock()
            .map_err(|_| "engine")?
            .view_page_size()
            .is_err()
    );
    let work = executor
        .prepare_ui_capture_for_close(&stamp, current_epoch, params())
        .map_err(|error| error.message)?;
    assert!(executor.has_active_work());
    let result = work
        .complete_blocking(&Cancellation::default())
        .map_err(|error| error.message)?;
    assert!(result.capture.is_some());
    assert!(result.view_target_prepared);
    assert!(!executor.has_active_work());
    let current = engine.lock().map_err(|_| "engine")?;
    assert_eq!(serde_json::to_value(current.state())?, before);
    assert_eq!(current.capture_stamp(), stamp);
    assert_eq!(current.content().sdf()?.as_bytes(), SOURCE);
    assert!(current.view_page_size().is_ok());
    Ok(())
}
