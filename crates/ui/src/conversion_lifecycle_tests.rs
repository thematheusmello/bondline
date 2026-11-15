//! Native conversion intent and real registered-worker lifecycle checks.
use super::*;
use bondline_commands::ConvertSmilesToCdxml;
use serde_json::json;
use std::fs;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn params(
    revision: u64,
    input: &std::path::Path,
    output: &std::path::Path,
) -> ConvertSmilesToCdxml {
    ConvertSmilesToCdxml {
        profile: Default::default(),
        expected_revision: revision,
        input_path: input.to_string_lossy().into_owned(),
        output_path: output.to_string_lossy().into_owned(),
        bond_length_pt: 48.0,
        origin_x_pt: 72.0,
        origin_y_pt: 72.0,
        page_width_pt: 210.0 * 72.0 / 25.4,
        page_height_pt: 297.0 * 72.0 / 25.4,
    }
}

#[test]
fn queued_conversion_is_busy_and_a_replaced_source_cannot_spawn_or_publish() -> TestResult {
    let directory = close_tests::Directory::new()?;
    let input = directory.0.join("registered-input-化学.smi");
    let output = directory.0.join("registered-output-化学.cdxml");
    fs::write(&input, b"CCC\n")?;
    let mut engine = Engine::default();
    engine.set_ui_file_host(true);
    engine.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":640,"height_pt":480}),
    )?;
    let mut workflow = FileWorkflow::new(directory.0.clone());
    engine.dispatch(
        "ui.file",
        json!({"expected_revision":1,"action":"convert_smiles"}),
    )?;
    assert!(workflow.adopt_request(&mut engine, &mut None).is_none());
    let context = egui::Context::default();
    workflow.queue_conversion(params(1, &input, &output), &engine, &context);
    assert!(workflow.is_busy() && workflow.blocks_authoring(&engine.state()));
    let replacement = directory.0.join("replacement.sdf");
    let source = include_bytes!("../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf");
    fs::write(&replacement, source)?;
    engine.dispatch(
        "document.open_sdf",
        json!({"path":replacement,"discard_unsaved":true}),
    )?;
    let before = serde_json::to_value(engine.state())?;
    let engine = std::sync::Arc::new(std::sync::Mutex::new(engine));
    let executor = CommandExecutor::new(std::sync::Arc::clone(&engine));
    workflow.start_pending_conversion(&executor, &context);
    assert!(!workflow.is_busy() && !executor.has_active_work());
    assert!(!output.exists());
    let guard = engine.lock().map_err(|_| "engine")?;
    assert_eq!(serde_json::to_value(guard.state())?, before);
    assert_eq!(guard.content().sdf()?.as_bytes(), source);
    assert_eq!(fs::read(input)?, b"CCC\n");
    Ok(())
}
