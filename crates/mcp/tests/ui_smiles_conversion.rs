use bondline_commands::{
    Cancellation, Command, ConvertSmilesToCdxml, Engine, RenderBackend, RenderCapture,
};
use bondline_mcp::{CommandExecutor, SharedEngine};
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    sync::{Arc, Mutex},
};

#[path = "../../cli/tests/support/conversion.rs"]
#[allow(
    dead_code,
    reason = "reuse the unchanged external actual-output consumer"
)]
mod conversion_support;
#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(
    dead_code,
    reason = "reuse the existing bounded temporary-directory owner"
)]
mod support;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn conversion(revision: u64, input: &Path, output: &Path) -> TestResult<ConvertSmilesToCdxml> {
    Ok(serde_json::from_value(conversion_support::params(
        revision, input, output,
    ))?)
}
fn capture() -> RenderCapture {
    RenderCapture {
        page_id: 1,
        width_px: 1024,
        height_px: 1024,
        backend: RenderBackend::Cpu,
        view: None,
    }
}
fn state(engine: &SharedEngine) -> TestResult<Value> {
    Ok(serde_json::to_value(
        engine.lock().map_err(|_| "engine")?.state(),
    )?)
}
fn request(engine: &SharedEngine) -> TestResult<u64> {
    let mut engine = engine.lock().map_err(|_| "engine")?;
    let revision = engine.state().revision;
    engine.dispatch(
        "ui.file",
        json!({"expected_revision":revision,"action":"convert_smiles"}),
    )?;
    let id = engine
        .state()
        .ui_file
        .request
        .ok_or("conversion intent")?
        .id;
    engine.claim_ui_file(id)?;
    Ok(id)
}
fn origin() -> (SharedEngine, CommandExecutor) {
    let mut engine = Engine::default();
    engine.set_ui_file_host(true);
    let shared = Arc::new(Mutex::new(engine));
    (Arc::clone(&shared), CommandExecutor::new(shared))
}

#[test]
fn native_conversion_reserves_the_shared_budget_before_consuming_the_accepted_intent() -> TestResult
{
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("input-λ.smi");
    let output = directory.path("output-化学.cdxml");
    fs::write(&input, b"CC\n")?;
    let (engine, executor) = origin();
    let stamp = engine.lock().map_err(|_| "engine")?.capture_stamp();
    let one = executor
        .prepare_ui_capture(&stamp, capture())
        .map_err(|e| e.message)?;
    let two = executor
        .prepare_ui_capture(&stamp, capture())
        .map_err(|e| e.message)?;
    let id = request(&engine)?;
    let accepted = state(&engine)?;
    assert_eq!(
        executor
            .prepare_ui_smiles_conversion(id, conversion(0, &input, &output)?)
            .err()
            .ok_or("full worker budget accepted conversion")?
            .code,
        "busy"
    );
    assert_eq!(state(&engine)?, accepted);
    assert!(!output.try_exists()?);
    drop(one);
    assert_eq!(
        executor
            .prepare_ui_file(
                id,
                Command::ConvertSmilesToCdxml(conversion(0, &input, &output)?)
            )
            .err()
            .ok_or("generic UI file binder accepted conversion")?
            .code,
        "invalid_params"
    );
    assert_eq!(state(&engine)?, accepted);
    let work = executor
        .clone()
        .prepare_ui_smiles_conversion(id, conversion(0, &input, &output)?)
        .map_err(|e| e.message)?;
    assert!(Arc::ptr_eq(executor.engine(), &engine));
    assert!(
        engine.try_lock().is_ok(),
        "preparation retained the origin mutex"
    );
    let mut consumed = accepted;
    consumed["ui_file"]["request"] = Value::Null;
    assert_eq!(state(&engine)?, consumed);
    assert!(executor.has_active_work());
    drop(two);
    assert!(executor.clone().has_active_work());
    // A real completed conversion keeps origin identity even if the active session changes.
    engine
        .lock()
        .map_err(|_| "engine")?
        .dispatch("document.new", json!({"expected_revision":0}))?;
    let latest = state(&engine)?;
    let result = std::thread::spawn(move || work.complete_blocking(&Cancellation::default()))
        .join()
        .map_err(|_| "conversion worker panicked")?
        .map_err(|e| e.message)?;
    let mut wire = serde_json::to_value(result)?;
    let receipt = wire
        .as_object_mut()
        .ok_or("result")?
        .remove("conversion")
        .ok_or("conversion receipt")?;
    assert_eq!(receipt["input_revision"], 0);
    assert_eq!(receipt["session_generation"], 0);
    assert_eq!(receipt["output"]["path"], json!(fs::canonicalize(&output)?));
    assert_eq!(receipt["output"]["byte_length"], fs::read(&output)?.len());
    assert_eq!(wire, latest);
    assert_eq!(state(&engine)?, latest);
    assert!(!executor.has_active_work());
    assert!(
        !engine
            .lock()
            .map_err(|_| "engine")?
            .finish_ui_file(id, false)
    );
    assert_eq!(state(&engine)?, latest);
    conversion_support::independent(&output, 2)?;
    assert_eq!(fs::read(input)?, b"CC\n");
    Ok(())
}

#[test]
fn failed_or_dropped_native_conversion_releases_workers_and_save_ownership_without_an_ack()
-> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("not-read.smi");
    let output = directory.path("not-created.cdxml");
    let (engine, executor) = origin();
    let stamp = engine.lock().map_err(|_| "engine")?.capture_stamp();
    let held = executor
        .prepare_ui_capture(&stamp, capture())
        .map_err(|e| e.message)?;
    let id = request(&engine)?;
    let accepted = state(&engine)?;
    for invalid in 0..6 {
        let mut params = conversion(if invalid % 2 == 0 { 1 } else { 0 }, &input, &output)?;
        if invalid % 2 == 1 {
            params.bond_length_pt = 0.0;
        }
        assert_eq!(
            executor
                .prepare_ui_smiles_conversion(id, params)
                .err()
                .ok_or("bad preparation accepted")?
                .code,
            if invalid % 2 == 0 {
                "stale_revision"
            } else {
                "invalid_params"
            }
        );
        assert_eq!(state(&engine)?, accepted);
    }
    // A leaked permit from any refusal would exhaust the single remaining slot.
    let work = executor
        .prepare_ui_smiles_conversion(id, conversion(0, &input, &output)?)
        .map_err(|e| e.message)?;
    assert!(executor.has_active_work());
    drop(work);
    let mut consumed = accepted;
    consumed["ui_file"]["request"] = Value::Null;
    assert_eq!(state(&engine)?, consumed);
    assert!(
        !engine
            .lock()
            .map_err(|_| "engine")?
            .finish_ui_file(id, false)
    );
    drop(held);
    assert!(!executor.has_active_work());
    assert!(!input.try_exists()? && !output.try_exists()?);
    fs::write(&input, b"CC\n")?;
    let retry = request(&engine)?;
    let work = executor
        .prepare_ui_smiles_conversion(retry, conversion(0, &input, &output)?)
        .map_err(|e| e.message)?;
    let before = state(&engine)?;
    let cancel = Cancellation::default();
    cancel.cancel();
    assert_eq!(
        work.complete_blocking(&cancel)
            .err()
            .ok_or("cancelled conversion published")?
            .code,
        "cancelled"
    );
    assert_eq!(state(&engine)?, before);
    assert!(!executor.has_active_work());
    assert!(!output.try_exists()?);
    assert!(
        !engine
            .lock()
            .map_err(|_| "engine")?
            .finish_ui_file(retry, false)
    );
    assert_eq!(state(&engine)?, before);
    // Success after both Drop and cancellation proves the same Engine Save lease was released.
    let final_id = request(&engine)?;
    let final_work = executor
        .prepare_ui_smiles_conversion(final_id, conversion(0, &input, &output)?)
        .map_err(|e| e.message)?;
    final_work
        .complete_blocking(&Cancellation::default())
        .map_err(|e| e.message)?;
    assert!(!executor.has_active_work());
    conversion_support::independent(&output, 2)?;
    Ok(())
}
