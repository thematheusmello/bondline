//! One actual CLI boundary case: invalid resource queries do not harden CDX Open.
use super::{cli, support};
use crate::registry_fixture;
use bondline_commands::Engine;
use serde_json::{Value, json};
use std::{fs, process::Command, sync::Arc};
#[path = "../../../../xtask/oracles/cdx_resource_support.rs"]
#[allow(
    dead_code,
    reason = "reuse independent raw-resource source and saved-byte reader"
)]
mod f;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn request(command: &str, params: Value) -> Value {
    json!({"command":command,"params":params})
}

#[test]
fn actual_cli_invalid_resource_queries_keep49_unchanged_saves_and_old_authority() -> Result {
    let name = "duplicate_font_table";
    assert_eq!(f::facts(name)["query"]["ok"], false);
    let directory = support::SdfTestDirectory::new()?;
    let input = directory.path("resource-cli-原-λ.cdx");
    let output = directory.path("resource-cli-保存-λ.cdx");
    let sentinel = directory.path("resource-cli-preserved.cdx");
    fs::write(&input, f::source(name))?;
    fs::write(&sentinel, b"keep destination\n")?;
    let listed = Command::new(env!("CARGO_BIN_EXE_bondline-cli"))
        .arg("commands")
        .output()?;
    assert!(
        listed.status.success(),
        "{}",
        String::from_utf8_lossy(&listed.stderr)
    );
    let previous = registry_fixture::with_cdx_caption_edit(include_str!(
        "../../../io/tests/fixtures/cdx-literal-metadata/old49.json"
    ));
    let prefix = previous.strip_suffix("]\n").ok_or("frozen array end")?;
    assert!(
        listed.stdout.starts_with(format!("{prefix},").as_bytes()),
        "the previous 49 wire records must remain byte-for-byte exact"
    );
    let rows: Vec<Value> = serde_json::from_slice(&listed.stdout)?;
    assert_eq!(rows.len(), 53);
    assert_eq!(rows[49]["name"], "ui.caption");
    let opened = cli(&[request("document.open_cdx", json!({"path":input}))])?;
    assert_eq!(opened["revision"], 1);
    let reopened = cli(&[
        request("document.open_cdx", json!({"path":input})),
        request(
            "page.set_size",
            json!({"page_id":1,"width_pt":612,"height_pt":792}),
        ),
        request("history.undo", json!({})),
        request("document.save_cdx", json!({"path":output})),
        request("document.open_cdx", json!({"path":output})),
        request("document.inspect", json!({})),
    ])?;
    assert_eq!(reopened["document"], opened["document"]);
    assert_eq!(reopened["revision"], 4);
    assert_eq!(reopened["file"]["format"], "cdx");
    assert_eq!(reopened["file"]["dirty"], false);
    assert_eq!(reopened["file"]["path"], json!(fs::canonicalize(&output)?));
    assert_eq!(
        reopened["source"],
        json!({"format":"cdx","byte_length":f::source(name).len(),"modified":false})
    );
    for path in [&input, &output] {
        let actual = fs::read(path)?;
        assert_eq!(actual, f::source(name));
        assert_eq!(f::read(&actual)?, f::facts(name));
    }
    // Existing public engine APIs keep the refusal state observable after errors;
    // the CLI intentionally exits on its first rejected command.
    let mut engine = Engine::default();
    engine.dispatch("document.open_cdx", json!({"path":input}))?;
    assert!(!engine.stored_document()?.is_fully_mapped());
    assert!(!engine.stored_document()?.has_display_projection());
    let before = serde_json::to_value(engine.state())?;
    let model = engine.stored_document()?.model_snapshot();
    let owner = engine.capture_stamp();
    let file = engine.ui_file_context_stamp();
    assert!(
        !engine
            .dispatch(
                "page.set_size",
                json!({"page_id":1,"width_pt":210.0*72.0/25.4,"height_pt":297.0*72.0/25.4})
            )?
            .changed
    );
    for (command, params) in [
        (
            "bond.create",
            json!({"page_id":1,"begin_x_pt":300,"begin_y_pt":300,"end_x_pt":348,"end_y_pt":300}),
        ),
        (
            "render.capture",
            json!({"page_id":1,"width_px":1024,"height_px":1024,"backend":"cpu"}),
        ),
        (
            "document.save_cdxml",
            json!({"path":sentinel,"overwrite":true}),
        ),
        (
            "document.save_mol",
            json!({"path":sentinel,"overwrite":true}),
        ),
    ] {
        assert_eq!(
            engine
                .dispatch(command, params)
                .err()
                .ok_or("authority widened")?
                .info()
                .code,
            "unsupported_content"
        );
        assert_eq!(serde_json::to_value(engine.state())?, before);
        assert_eq!(fs::read(&sentinel)?, b"keep destination\n");
    }
    assert!(Arc::ptr_eq(
        &model,
        &engine.stored_document()?.model_snapshot()
    ));
    assert!(engine.capture_stamp_matches(&owner));
    assert!(engine.ui_file_context_stamp_matches(&file));
    engine.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":612,"height_pt":792}),
    )?;
    assert!(engine.state().file.dirty);
    let dirty = serde_json::to_value(engine.state())?;
    assert_eq!(
        engine
            .dispatch(
                "document.save_cdx",
                json!({"path":sentinel,"overwrite":true})
            )
            .err()
            .ok_or("drift saved")?
            .info()
            .code,
        "unsupported_content"
    );
    assert_eq!(serde_json::to_value(engine.state())?, dirty);
    assert_eq!(fs::read(&sentinel)?, b"keep destination\n");
    engine.dispatch("history.undo", json!({}))?;
    assert!(!engine.state().file.dirty);
    assert_eq!(
        engine.stored_document()?.to_cdx()?.as_ref(),
        f::source(name)
    );
    assert!(!engine.stored_document()?.has_display_projection());
    Ok(())
}
