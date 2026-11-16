//! Independent central controls and read-only display authority for SMILES projections.
use bondline_commands::Engine;
use serde_json::{Value, json};
use std::fs;

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse the unchanged isolated filesystem owner")]
mod support;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn controls(engine: &Engine) -> TestResult<Value> {
    Ok(serde_json::to_value(engine.controls())?)
}
fn control<'a>(rows: &'a Value, id: &str) -> TestResult<&'a Value> {
    rows.as_array()
        .ok_or("controls array")?
        .iter()
        .find(|row| row["id"] == id)
        .ok_or_else(|| format!("missing {id}").into())
}

#[test]
fn import_binding_is_one_typed_file_presentation_without_a_second_registry_command() -> TestResult {
    let mut engine = Engine::default();
    let rows = controls(&engine)?;
    assert_eq!(rows.as_array().ok_or("controls")?.len(), 20);
    let import = control(&rows, "file.import_smiles_records")?;
    assert_eq!(import["label"], "Import SMILES records…");
    assert_eq!(import["menu"], "File");
    assert!(import["shortcut"].is_null());
    assert_eq!(import["enabled"], false);
    assert_eq!(import["disabled_reason"], "ui_unavailable");
    assert_eq!(
        import["command"],
        json!({"command":"ui.file","params":{"expected_revision":0,"action":"import_smiles_records"}})
    );
    assert_eq!(bondline_commands::registry().len(), 32);
    engine.set_ui_file_host(true);
    let rows = controls(&engine)?;
    let import = control(&rows, "file.import_smiles_records")?;
    assert_eq!(import["enabled"], false);
    assert_eq!(import["disabled_reason"], "unsupported_content");
    Ok(())
}

#[test]
fn projected_smiles_exposes_view_and_order_modes_without_granting_generic_source_authority()
-> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("projected-controls-λ.smi");
    let source = b"CC\tPRIVATE\0\xff\r\n";
    fs::write(&input, source)?;
    let mut engine = Engine::default();
    engine.set_ui_file_host(true);
    engine.dispatch("document.open_smiles", json!({"path":input}))?;
    let raw = controls(&engine)?;
    assert_eq!(raw.as_array().ok_or("raw controls")?.len(), 10);
    assert_eq!(
        control(&raw, "file.import_smiles_records")?["enabled"],
        true
    );
    assert!(control(&raw, "view.actual").is_err());
    assert!(control(&raw, "tool.bond.single").is_err());
    let before = serde_json::to_value(engine.state())?;
    assert!(
        engine
            .dispatch("ui.view", json!({"action":"actual"}))
            .is_err()
    );
    assert_eq!(serde_json::to_value(engine.state())?, before);
    engine.dispatch("document.import_smiles_records",json!({"expected_revision":1,"selection":{"mode":"all"},
        "bond_length_pt":64,"origin_x_pt":96,"origin_y_pt":96,"page_width_pt":612,"page_height_pt":792}))?;
    assert!(
        !engine.stored_document()?.is_fully_mapped(),
        "projection does not grant a complete foreign-format mapping"
    );
    let owner = engine.content().clone();
    let stamp = engine.capture_stamp();
    let before = serde_json::to_value(engine.state())?;
    engine.set_view_canvas_size(1024, 1024)?;
    let rows = controls(&engine)?;
    assert_eq!(rows.as_array().ok_or("projected controls")?.len(), 20);
    for id in [
        "view.actual",
        "view.fit",
        "view.magnify",
        "view.reduce",
        "tool.bond.single",
        "tool.bond.double",
        "tool.bond.triple",
    ] {
        assert_eq!(control(&rows, id)?["enabled"], true, "{id}");
    }
    for id in [
        "page.a4",
        "page.letter",
        "page.rotate",
        "file.import_smiles_records",
        "edit.paste_smiles",
    ] {
        let row = control(&rows, id)?;
        assert_eq!(row["enabled"], false, "{id}");
        assert_eq!(row["disabled_reason"], "unsupported_content", "{id}");
    }
    engine.dispatch("ui.view", json!({"action":"set_zoom","percent":175.5}))?;
    engine.dispatch("ui.bond_tool", json!({"order":"double"}))?;
    assert_eq!(engine.view().zoom_percent, Some(175.5));
    let rendered=engine.dispatch("render.capture",json!({"page_id":1,"width_px":1024,"height_px":1024,"backend":"cpu",
        "view":{"logical_width":1024,"logical_height":1024,"expected_view_epoch":engine.view().epoch}}))?;
    assert!(rendered.capture.is_some());
    assert_eq!(engine.content(), &owner);
    assert!(engine.capture_stamp_matches(&stamp));
    assert_eq!(serde_json::to_value(engine.state())?, before);
    let output = directory.path("no-metadata-loss.cdxml");
    assert_eq!(
        engine
            .dispatch("document.save_cdxml", json!({"path":output}))
            .err()
            .ok_or("crossformat Save")?
            .info()
            .code,
        "unsupported_content"
    );
    assert!(!output.exists());
    assert_eq!(fs::read(input)?, source);
    Ok(())
}

#[test]
fn sdf_retains_its_own_controls_and_raw_import_obeys_host_and_close_guards() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let sdf_path = directory.path("controls.sdf");
    fs::write(
        &sdf_path,
        include_bytes!("../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf"),
    )?;
    let mut sdf = Engine::default();
    sdf.set_ui_file_host(true);
    sdf.dispatch("document.open_sdf", json!({"path":sdf_path}))?;
    let rows = controls(&sdf)?;
    assert_eq!(rows.as_array().ok_or("SDF controls")?.len(), 14);
    assert_eq!(
        control(&rows, "file.import_smiles_records")?["enabled"],
        false
    );
    assert_eq!(
        control(&rows, "file.import_smiles_records")?["disabled_reason"],
        "unsupported_content"
    );
    let source = directory.path("close.smi");
    fs::write(&source, b"CC\n")?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_smiles", json!({"path":source}))?;
    let before = serde_json::to_value(engine.state())?;
    assert_eq!(
        engine
            .dispatch(
                "ui.file",
                json!({"expected_revision":1,"action":"import_smiles_records"})
            )
            .err()
            .ok_or("headless import")?
            .info()
            .code,
        "ui_unavailable"
    );
    assert_eq!(serde_json::to_value(engine.state())?, before);
    engine.set_ui_file_host(true);
    engine.dispatch(
        "document.close",
        json!({"expected_revision":1,"action":"request"}),
    )?;
    let rows = controls(&engine)?;
    assert_eq!(
        control(&rows, "file.import_smiles_records")?["enabled"],
        false
    );
    assert_eq!(
        control(&rows, "file.import_smiles_records")?["disabled_reason"],
        "close_pending"
    );
    let before = serde_json::to_value(engine.state())?;
    assert_eq!(
        engine
            .dispatch(
                "ui.file",
                json!({"expected_revision":1,"action":"import_smiles_records"})
            )
            .err()
            .ok_or("close import")?
            .info()
            .code,
        "ui_busy"
    );
    assert_eq!(serde_json::to_value(engine.state())?, before);
    Ok(())
}
