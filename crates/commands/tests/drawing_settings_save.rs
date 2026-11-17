use bondline_commands::{Command, Engine};
use serde_json::json;
use std::{fs, sync::Arc};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(
    dead_code,
    reason = "reuse only the unchanged isolated-directory helper"
)]
mod support;
const NATIVE: &[u8] = include_bytes!("../../io/tests/fixtures/bondline-v0.cdxml");

#[test]
fn captured_save_keeps_its_old_baseline_and_lease_without_cleaning_a_new_settings_edit()
-> Result<(), Box<dyn std::error::Error>> {
    for edited in [false, true] {
        let directory = support::FileTestDirectory::new()?;
        let input = directory.path("original-設定-λ.cdxml");
        let output = directory.path("captured-設定-λ.cdxml");
        let unused = directory.path("not-published.cdxml");
        fs::write(&input, NATIVE)?;
        let mut engine = Engine::default();
        engine.dispatch("document.open_cdxml", json!({"path":input}))?;
        let pending = engine.prepare(Command::parse(
            "document.save_cdxml",
            json!({"path":output}),
        )?)?;
        let changed = engine.dispatch(
            "document.set_drawing_settings",
            json!({"expected_revision":1,
                "bond_length_pt":edited.then_some(24.5),
                "chain_angle_degrees":edited.then_some(120.5)}),
        )?;
        assert_eq!(changed.changed, edited);
        let model = engine.stored_document()?.model_snapshot();
        let request = || Command::parse("document.save_cdxml", json!({"path":unused}));
        let error = engine.prepare(request()?).err().ok_or("save lease lost")?;
        assert_eq!(error.info().code, "busy");
        let completed = pending.complete()?;
        assert_eq!(
            fs::read(&output)?,
            NATIVE,
            "publication uses the captured original"
        );
        let error = engine
            .prepare(request()?)
            .err()
            .ok_or("lease ended before finish")?;
        assert_eq!(error.info().code, "busy");
        let result = engine.finish(completed)?;
        assert!(!result.changed);
        assert_eq!(result.revision, if edited { 2 } else { 1 });
        assert_eq!(result.file.dirty, edited);
        assert_eq!(
            result.file.path.as_deref(),
            fs::canonicalize(&output)?.to_str()
        );
        assert_eq!(result.source.ok_or("source")?.modified, edited);
        let receipt = result.file_operation.ok_or("save receipt")?;
        assert_eq!(receipt.input_revision, 1);
        assert_eq!(receipt.byte_length, NATIVE.len());
        assert!(receipt.ui_request_id.is_none() && receipt.ui_request_current.is_none());
        assert!(Arc::ptr_eq(
            &engine.stored_document()?.model_snapshot(),
            &model
        ));
        assert_eq!(
            engine.document()?.drawing_settings().bond_length_pt(),
            edited.then_some(24.5)
        );
        assert_eq!(
            engine.document()?.drawing_settings().chain_angle_degrees(),
            edited.then_some(120.5)
        );
        assert_eq!(fs::read(&input)?, NATIVE);
        if edited {
            engine.dispatch(
                "document.set_drawing_settings",
                json!({
                "expected_revision":2,"bond_length_pt":null,"chain_angle_degrees":null}),
            )?;
            assert!(
                !engine.state().file.dirty,
                "the saved baseline is the old captured pair"
            );
            assert_eq!(engine.stored_document()?.export_cdxml()?, NATIVE);
        }
        drop(engine.prepare(request()?)?);
        assert!(!unused.exists(), "dropped pending work cannot publish");
    }
    Ok(())
}
