use bondline_commands::{Cancellation, Command, Engine};
use serde_json::{Value, json};
use std::{fs, path::Path, process::Command as ProcessCommand};

#[path = "../../../xtask/oracles/files_test_support.rs"]
mod file_support;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn bond(engine: &mut Engine) -> TestResult {
    engine.dispatch(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":100,"begin_y_pt":200,"end_x_pt":140,"end_y_pt":200}),
    )?;
    Ok(())
}

fn save(name: &str, path: &Path, overwrite: bool) -> Result<Command, bondline_commands::Error> {
    Command::parse(name, json!({"path":path,"overwrite":overwrite}))
}

fn snapshot(engine: &Engine) -> Result<Value, serde_json::Error> {
    serde_json::to_value(engine.state())
}

fn independently_read_mol(path: &Path) -> TestResult {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let output = ProcessCommand::new(python)
        .arg("-B")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../io/tests/mol_oracle.py"
        ))
        .stdin(fs::File::open(path)?)
        .output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let graph: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(graph["smiles"], "CC");
    assert_eq!(graph["formula"], "C2H6");
    assert_eq!(graph["atoms"].as_array().ok_or("atoms")?.len(), 2);
    assert_eq!(graph["bonds"], json!([{"begin":0,"end":1,"order":1.0}]));
    Ok(())
}

#[test]
fn published_but_unfinished_save_blocks_another_format_before_it_can_publish() -> TestResult {
    let directory = file_support::FileTestDirectory::new()?;
    let path = directory.path("same-path.cdxml");
    let other = directory.path("other.mol");
    let mut engine = Engine::default();
    bond(&mut engine)?;
    let prepared = engine.prepare(save("document.save_cdxml", &path, false)?)?;
    let completed = prepared.complete()?;
    let published = fs::read(&path)?;
    file_support::independently_read_file(&path, 1)?;
    let before = snapshot(&engine)?;
    for (name, destination) in [
        ("document.save_mol", &path),
        ("document.save_mol", &other),
        ("document.save_cdxml", &other),
    ] {
        let error = engine
            .prepare(save(name, destination, true)?)
            .err()
            .ok_or("a second save was accepted before the published receipt was applied")?;
        assert_eq!(error.info().code, "busy");
        assert_eq!(snapshot(&engine)?, before);
        assert_eq!(fs::read(&path)?, published);
        assert!(!other.try_exists()?);
    }
    let receipt = engine.finish(completed)?;
    assert_eq!(receipt.file.format, bondline_commands::FileFormat::Cdxml);
    assert!(!receipt.file.dirty);
    assert_eq!(receipt.revision, 1);
    assert_eq!(receipt.undo_depth, 1);
    assert_eq!(fs::read(&path)?, published);
    let next = engine.prepare(save("document.save_mol", &other, false)?)?;
    let completed = next.complete()?;
    engine.finish(completed)?;
    assert!(
        fs::read_to_string(&other)?
            .lines()
            .nth(3)
            .ok_or("MOL counts")?
            .ends_with("V2000")
    );
    independently_read_mol(&other)?;
    assert_eq!(
        engine.state().file.format,
        bondline_commands::FileFormat::MolV2000
    );
    Ok(())
}

#[test]
fn a_save_blocks_only_other_saves_and_keeps_a_newly_opened_session_intact() -> TestResult {
    let directory = file_support::FileTestDirectory::new()?;
    let old_path = directory.path("captured.cdxml");
    let opened_path = directory.path("new-session.cdxml");
    let next_path = directory.path("next.cdxml");
    let independent_path = directory.path("independent.mol");
    const NATIVE: &[u8] = include_bytes!("../../io/tests/fixtures/bondline-v0.cdxml");
    fs::write(&opened_path, NATIVE)?;
    let mut engine = Engine::default();
    bond(&mut engine)?;
    let pending = engine.prepare(save("document.save_cdxml", &old_path, false)?)?;
    let mut independent = Engine::default();
    let other = independent.prepare(save("document.save_mol", &independent_path, false)?)?;
    drop(other);
    assert!(!independent_path.try_exists()?);
    assert_eq!(engine.dispatch("document.inspect", json!({}))?.revision, 1);
    engine.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":612,"height_pt":792}),
    )?;
    let capture = engine.dispatch(
        "render.capture",
        json!({"page_id":1,"width_px":1024,"height_px":1024,"backend":"cpu"}),
    )?;
    assert!(capture.capture.is_some());
    assert_eq!((capture.revision, capture.undo_depth), (2, 2));
    engine.dispatch(
        "document.open_cdxml",
        json!({"path":opened_path,"discard_unsaved":true}),
    )?;
    let before = snapshot(&engine)?;
    let error = engine
        .prepare(save("document.save_mol", &next_path, false)?)
        .err()
        .ok_or("a new generation escaped an outstanding save")?;
    assert_eq!(error.info().code, "busy");
    assert_eq!(snapshot(&engine)?, before);
    assert!(!next_path.try_exists()?);
    let receipt = engine.finish(pending.complete()?)?;
    assert_eq!(receipt.file_operation.ok_or("receipt")?.input_revision, 1);
    assert_eq!(snapshot(&engine)?, before);
    assert_eq!(fs::read(&opened_path)?, NATIVE);
    file_support::independently_read_file(&old_path, 1)?;
    engine.dispatch("document.save_cdxml", json!({"path":next_path}))?;
    assert_eq!(fs::read(&next_path)?, NATIVE);
    assert_eq!(engine.state().revision, 3);
    assert_eq!(
        (engine.state().undo_depth, engine.state().redo_depth),
        (0, 0)
    );
    Ok(())
}

#[test]
fn dropped_prepared_and_published_handles_release_without_inventing_an_applied_save() -> TestResult
{
    for (name, extension) in [
        ("document.save_cdxml", "cdxml"),
        ("document.save_mol", "mol"),
    ] {
        for published in [false, true] {
            let directory = file_support::FileTestDirectory::new()?;
            let path = directory.path(&format!("abandoned.{extension}"));
            let next = directory.path(&format!("applied.{extension}"));
            let mut engine = Engine::default();
            bond(&mut engine)?;
            let before = snapshot(&engine)?;
            let prepared = engine.prepare(save(name, &path, false)?)?;
            if published {
                let completed = prepared.complete()?;
                assert_eq!(snapshot(&engine)?, before);
                drop(completed);
                if extension == "cdxml" {
                    file_support::independently_read_file(&path, 1)?;
                } else {
                    independently_read_mol(&path)?;
                }
            } else {
                drop(prepared);
                assert!(!path.try_exists()?);
            }
            assert_eq!(snapshot(&engine)?, before);
            assert!(engine.state().file.path.is_none());
            assert!(engine.state().file.dirty);
            let applied = engine.dispatch(name, json!({"path":next}))?;
            assert_eq!(applied.revision, 1);
            assert_eq!(applied.undo_depth, 1);
            assert!(!applied.file.dirty);
            if published {
                assert_eq!(fs::read(&path)?, fs::read(&next)?);
            }
            assert!(fs::metadata(&next)?.len() > 0);
        }
    }
    Ok(())
}

#[test]
fn cancellation_and_publication_refusal_release_the_save_and_preserve_existing_bytes() -> TestResult
{
    for cancelled in [false, true] {
        let directory = file_support::FileTestDirectory::new()?;
        let occupied = directory.path("unrelated.cdxml");
        let next = directory.path("retry.cdxml");
        fs::write(&occupied, b"unrelated original bytes")?;
        let mut engine = Engine::default();
        bond(&mut engine)?;
        let before = snapshot(&engine)?;
        let prepared = engine.prepare(save("document.save_cdxml", &occupied, false)?)?;
        let cancel = Cancellation::default();
        if cancelled {
            cancel.cancel();
        }
        let error = prepared
            .complete_with_cancel(&cancel)
            .err()
            .ok_or("save unexpectedly succeeded")?;
        assert_eq!(
            error.info().code,
            if cancelled {
                "cancelled"
            } else {
                "already_exists"
            }
        );
        assert_eq!(snapshot(&engine)?, before);
        assert_eq!(fs::read(&occupied)?, b"unrelated original bytes");
        engine.dispatch("document.save_cdxml", json!({"path":next}))?;
        assert!(!engine.state().file.dirty);
        file_support::independently_read_file(&next, 1)?;
    }
    Ok(())
}

#[test]
fn refused_source_regeneration_releases_the_save_without_losing_opaque_data() -> TestResult {
    const OPAQUE: &[u8] = include_bytes!("../../cdxml/tests/fixtures/unknown.cdxml");
    let directory = file_support::FileTestDirectory::new()?;
    let source = directory.path("opaque.cdxml");
    let output = directory.path("opaque-copy.cdxml");
    fs::write(&source, OPAQUE)?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_cdxml", json!({"path":source}))?;
    engine.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":400,"height_pt":600}),
    )?;
    let before = snapshot(&engine)?;
    let error = engine
        .prepare(save("document.save_cdxml", &output, false)?)?
        .complete()
        .err()
        .ok_or("opaque source was discarded")?;
    assert_eq!(error.info().code, "unsupported_content");
    assert_eq!(snapshot(&engine)?, before);
    assert!(!output.try_exists()?);
    assert_eq!(fs::read(&source)?, OPAQUE);
    engine.dispatch("history.undo", json!({}))?;
    let saved = engine.dispatch("document.save_cdxml", json!({"path":output}))?;
    assert_eq!(saved.revision, 3);
    assert_eq!((saved.undo_depth, saved.redo_depth), (0, 1));
    assert!(!saved.file.dirty);
    assert_eq!(fs::read(&output)?, OPAQUE);
    Ok(())
}

#[test]
fn cancellation_after_publication_keeps_the_save_reserved_until_its_truthful_receipt() -> TestResult
{
    let directory = file_support::FileTestDirectory::new()?;
    let path = directory.path("published.cdxml");
    let next = directory.path("next.cdxml");
    let mut engine = Engine::default();
    bond(&mut engine)?;
    let cancel = Cancellation::default();
    let completed = engine
        .prepare(save("document.save_cdxml", &path, false)?)?
        .complete_with_cancel(&cancel)?;
    let bytes = fs::read(&path)?;
    cancel.cancel();
    let error = engine
        .prepare(save("document.save_cdxml", &next, false)?)
        .err()
        .ok_or("cancellation released a published receipt too early")?;
    assert_eq!(error.info().code, "busy");
    assert!(!next.try_exists()?);
    let result = engine.finish(completed)?;
    assert!(!result.file.dirty);
    assert_eq!(
        result
            .file_operation
            .ok_or("lost published receipt")?
            .byte_length,
        bytes.len()
    );
    assert_eq!(fs::read(&path)?, bytes);
    file_support::independently_read_file(&path, 1)?;
    drop(engine.prepare(save("document.save_cdxml", &next, false)?)?);
    assert!(!next.try_exists()?);
    Ok(())
}
