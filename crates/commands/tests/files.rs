use bondline_commands::{Cancellation, Command, Engine};
use serde_json::{Value, json};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command as ProcessCommand, Stdio},
    sync::atomic::{AtomicU64, Ordering},
};

type TestResult = Result<(), Box<dyn std::error::Error>>;
const NATIVE: &[u8] = include_bytes!("../../io/tests/fixtures/bondline-v0.cdxml");
const OPAQUE: &[u8] = include_bytes!("../../cdxml/tests/fixtures/unknown.cdxml");
static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new() -> Result<Self, std::io::Error> {
        let target = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target");
        fs::create_dir_all(&target)?;
        loop {
            let path = target.join(format!(
                "file-verification-{}-{}",
                std::process::id(),
                NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self(path)),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            }
        }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn bond(engine: &mut Engine, y: f64) -> TestResult {
    engine.dispatch(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":100.0,"begin_y_pt":y,"end_x_pt":140.0,"end_y_pt":y}),
    )?;
    Ok(())
}

fn file_state(engine: &Engine, path: Option<&Path>, dirty: bool) -> TestResult {
    let path = path.map(fs::canonicalize).transpose()?;
    assert_eq!(
        serde_json::to_value(engine.state())?.get("file"),
        Some(&json!({"path":path,"dirty":dirty})),
        "every result must expose the current saved path and exact dirty state"
    );
    Ok(())
}

fn snapshot(engine: &Engine) -> Result<Vec<u8>, serde_json::Error> {
    serde_json::to_vec(&engine.state())
}

fn independently_read(bytes: &[u8], fragments: usize) -> TestResult {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child = ProcessCommand::new(python)
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../xtask/oracles/files_rdkit.py"
        ))
        .arg(fragments.to_string())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child.stdin.take().ok_or("oracle stdin")?.write_all(bytes)?;
    let result = child.wait_with_output()?;
    assert!(
        result.status.success(),
        "independent Expat/RDKit consumer: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    let reported: Value = serde_json::from_slice(&result.stdout)?;
    assert_eq!(
        reported,
        json!({"fragments":fragments,"molecules":fragments})
    );
    Ok(())
}

#[test]
fn registered_file_commands_are_closed_typed_parameter_objects() -> TestResult {
    for name in ["document.open_cdxml", "document.save_cdxml"] {
        assert!(Command::parse(name, json!({"path":"drawing.cdxml"})).is_ok());
        for invalid in [
            json!({}),
            json!({"path":42}),
            json!({"path":"drawing.cdxml","unexpected":true}),
        ] {
            let error = Command::parse(name, invalid)
                .err()
                .ok_or("closed typed file params")?;
            assert_eq!(error.info().code, "invalid_params");
        }
    }
    assert_eq!(
        Command::parse(
            "document.open_cdxml",
            json!({"path":"drawing.cdxml","discard_unsaved":"true"})
        )
        .err()
        .ok_or("discard must be boolean")?
        .info()
        .code,
        "invalid_params"
    );
    assert_eq!(
        Command::parse(
            "document.save_cdxml",
            json!({"path":"drawing.cdxml","overwrite":"true"})
        )
        .err()
        .ok_or("overwrite must be boolean")?
        .info()
        .code,
        "invalid_params"
    );
    Ok(())
}

#[test]
fn draw_save_open_edit_save_and_reopen_is_independently_readable() -> TestResult {
    let directory = TestDirectory::new()?;
    let path = directory.path("drawing.cdxml");
    let mut engine = Engine::default();
    file_state(&engine, None, false)?;
    bond(&mut engine, 200.0)?;
    file_state(&engine, None, true)?;
    let before_save = engine.state();
    let saved = engine.dispatch("document.save_cdxml", json!({"path":path}))?;
    assert_eq!(saved.revision, before_save.revision);
    assert_eq!(saved.undo_depth, before_save.undo_depth);
    assert_eq!(saved.redo_depth, before_save.redo_depth);
    file_state(&engine, Some(&path), false)?;
    let first_bytes = fs::read(&path)?;
    independently_read(&first_bytes, 1)?;
    let receipt = serde_json::to_value(saved)?;
    assert_eq!(receipt["file_operation"]["operation"], "save");
    assert_eq!(
        receipt["file_operation"]["path"],
        json!(fs::canonicalize(&path)?)
    );
    assert_eq!(
        receipt["file_operation"]["input_revision"],
        before_save.revision
    );
    assert_eq!(receipt["file_operation"]["byte_length"], first_bytes.len());
    assert!(receipt["file_operation"]["directory_synced"].is_boolean());

    let mut reopened = Engine::default();
    let opened = reopened.dispatch("document.open_cdxml", json!({"path":path}))?;
    assert_eq!(
        (opened.revision, opened.undo_depth, opened.redo_depth),
        (1, 0, 0)
    );
    let opening_receipt = serde_json::to_value(&opened)?;
    assert_eq!(opening_receipt["file_operation"]["operation"], "open");
    assert_eq!(opening_receipt["file_operation"]["input_revision"], 0);
    assert_eq!(
        opening_receipt["file_operation"]["byte_length"],
        first_bytes.len()
    );
    assert_eq!(reopened.document(), engine.document());
    file_state(&reopened, Some(&path), false)?;
    bond(&mut reopened, 300.0)?;
    file_state(&reopened, Some(&path), true)?;
    let before_no_clobber = snapshot(&reopened)?;
    let no_clobber = reopened
        .dispatch("document.save_cdxml", json!({"path":path}))
        .err()
        .ok_or("save defaults to no-clobber")?;
    assert_eq!(no_clobber.info().code, "already_exists");
    assert_eq!(snapshot(&reopened)?, before_no_clobber);
    assert_eq!(fs::read(&path)?, first_bytes);
    let before_replace = reopened.state();
    reopened.dispatch("document.save_cdxml", json!({"path":path,"overwrite":true}))?;
    assert_eq!(reopened.state().revision, before_replace.revision);
    assert_eq!(reopened.state().undo_depth, before_replace.undo_depth);
    file_state(&reopened, Some(&path), false)?;
    let second_bytes = fs::read(&path)?;
    independently_read(&second_bytes, 2)?;
    let edited = reopened.document().clone();
    reopened.dispatch("history.undo", json!({}))?;
    file_state(&reopened, Some(&path), true)?;
    assert_eq!(
        fs::read(&path)?,
        second_bytes,
        "Undo is an authoring operation"
    );
    reopened.dispatch("history.redo", json!({}))?;
    file_state(&reopened, Some(&path), false)?;
    assert_eq!(reopened.document(), &edited);
    let mut again = Engine::default();
    again.dispatch("document.open_cdxml", json!({"path":path}))?;
    assert_eq!(again.document(), &edited);
    Ok(())
}

#[test]
fn opaque_source_saves_exactly_and_refuses_lossy_edits() -> TestResult {
    let directory = TestDirectory::new()?;
    let original = directory.path("foreign.cdxml");
    let copy = directory.path("copy.cdxml");
    fs::write(&original, OPAQUE)?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_cdxml", json!({"path":original}))?;
    assert!(!engine.stored_document().is_fully_mapped());
    engine.dispatch("document.save_cdxml", json!({"path":copy}))?;
    assert_eq!(fs::read(&copy)?, OPAQUE);
    file_state(&engine, Some(&copy), false)?;
    let before = snapshot(&engine)?;
    let error = engine.dispatch("bond.create", json!({"page_id":1,"begin_x_pt":100.0,"begin_y_pt":200.0,"end_x_pt":140.0,"end_y_pt":200.0})).err().ok_or("unmapped source must not lose opaque data")?;
    assert_eq!(error.info().code, "unsupported_content");
    assert_eq!(snapshot(&engine)?, before);
    engine.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":700.0,"height_pt":800.0}),
    )?;
    file_state(&engine, Some(&copy), true)?;
    let edited = snapshot(&engine)?;
    let error = engine
        .dispatch("document.save_cdxml", json!({"path":copy,"overwrite":true}))
        .err()
        .ok_or("unmapped edited projection cannot replace source")?;
    assert_eq!(error.info().code, "unsupported_content");
    assert_eq!(snapshot(&engine)?, edited);
    assert_eq!(fs::read(&copy)?, OPAQUE);
    assert_eq!(fs::read(&original)?, OPAQUE);
    engine.dispatch("history.undo", json!({}))?;
    file_state(&engine, Some(&copy), false)?;
    assert_eq!(engine.stored_document().to_cdxml()?.as_ref(), OPAQUE);
    Ok(())
}

#[test]
fn failed_opens_retain_complete_document_source_history_and_file_state() -> TestResult {
    let directory = TestDirectory::new()?;
    let original = directory.path("original.cdxml");
    fs::write(&original, NATIVE)?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_cdxml", json!({"path":original}))?;
    bond(&mut engine, 300.0)?;
    bond(&mut engine, 400.0)?;
    engine.dispatch("history.undo", json!({}))?;
    let stored = engine.stored_document().clone();
    let before = snapshot(&engine)?;
    let malformed = directory.path("malformed.cdxml");
    fs::write(&malformed, b"<CDXML><page></CDXML>")?;
    let wrong_root = directory.path("wrong-root.cdxml");
    fs::write(&wrong_root, b"<other/>")?;
    let multi_page = directory.path("multi-page.cdxml");
    fs::write(&multi_page, b"<CDXML><page id='1'/><page id='2'/></CDXML>")?;
    let oversized = directory.path("oversized.cdxml");
    fs::File::create(&oversized)?.set_len(64 * 1024 * 1024 + 1)?;
    for path in [
        directory.path("missing.cdxml"),
        malformed,
        wrong_root,
        multi_page,
        oversized,
    ] {
        assert!(
            engine
                .dispatch(
                    "document.open_cdxml",
                    json!({"path":path,"discard_unsaved":true})
                )
                .is_err(),
            "invalid open must fail: {}",
            path.display()
        );
        assert_eq!(engine.stored_document(), &stored);
        assert_eq!(
            snapshot(&engine)?,
            before,
            "failed open changed state: {}",
            path.display()
        );
        assert_eq!(fs::read(&original)?, NATIVE);
    }
    engine.dispatch("history.redo", json!({}))?;
    assert_eq!(
        engine.document().pages()[0].fragments().len(),
        3,
        "failed open must retain usable redo"
    );
    Ok(())
}

#[test]
fn dirty_open_requires_explicit_discard_and_success_resets_history() -> TestResult {
    let directory = TestDirectory::new()?;
    let path = directory.path("native.cdxml");
    fs::write(&path, NATIVE)?;
    let mut engine = Engine::default();
    bond(&mut engine, 200.0)?;
    let before = snapshot(&engine)?;
    for params in [
        json!({"path":path}),
        json!({"path":path,"discard_unsaved":false}),
    ] {
        assert_eq!(
            engine
                .dispatch("document.open_cdxml", params)
                .err()
                .ok_or("unsaved authoring requires explicit discard")?
                .info()
                .code,
            "unsaved_changes"
        );
        assert_eq!(snapshot(&engine)?, before);
    }
    let opened = engine.dispatch(
        "document.open_cdxml",
        json!({"path":path,"discard_unsaved":true}),
    )?;
    assert_eq!(
        (opened.revision, opened.undo_depth, opened.redo_depth),
        (2, 0, 0)
    );
    file_state(&engine, Some(&path), false)?;
    assert_eq!(engine.stored_document().to_cdxml()?.as_ref(), NATIVE);
    assert_eq!(
        engine
            .dispatch("history.undo", json!({}))
            .err()
            .ok_or("previous session must not remain in Undo")?
            .info()
            .code,
        "empty_history"
    );
    Ok(())
}

#[test]
fn save_failure_and_noop_preserve_path_dirty_state_history_and_existing_bytes() -> TestResult {
    let directory = TestDirectory::new()?;
    let original = directory.path("original.cdxml");
    let occupied = directory.path("occupied.cdxml");
    fs::write(&occupied, b"unrelated existing bytes")?;
    let mut engine = Engine::default();
    bond(&mut engine, 200.0)?;
    engine.dispatch("document.save_cdxml", json!({"path":original}))?;
    let disk = fs::read(&original)?;
    bond(&mut engine, 300.0)?;
    engine.dispatch("history.undo", json!({}))?;
    file_state(&engine, Some(&original), false)?;
    let before = snapshot(&engine)?;
    let page = engine.document().pages().first().ok_or("page")?;
    let no_op = engine.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":page.size().width_pt(),"height_pt":page.size().height_pt()}),
    )?;
    assert!(!no_op.changed);
    assert_eq!(snapshot(&engine)?, before);
    for path in [
        occupied.clone(),
        directory.path("absent-parent/drawing.cdxml"),
        directory.0.clone(),
    ] {
        assert!(
            engine
                .dispatch("document.save_cdxml", json!({"path":path}))
                .is_err(),
            "save must fail: {}",
            path.display()
        );
        assert_eq!(snapshot(&engine)?, before);
        assert_eq!(fs::read(&original)?, disk);
        assert_eq!(fs::read(&occupied)?, b"unrelated existing bytes");
    }
    engine.dispatch("history.redo", json!({}))?;
    file_state(&engine, Some(&original), true)?;
    assert_eq!(engine.document().pages()[0].fragments().len(), 2);
    Ok(())
}

#[test]
fn completed_open_rejects_stale_revision_and_cancellation_without_applying() -> TestResult {
    let directory = TestDirectory::new()?;
    let path = directory.path("native.cdxml");
    fs::write(&path, NATIVE)?;
    let mut engine = Engine::default();
    let command = || {
        Command::parse(
            "document.open_cdxml",
            json!({"path":path,"discard_unsaved":true}),
        )
    };
    let work = engine.prepare(command()?)?;
    let completed = work.complete()?;
    bond(&mut engine, 300.0)?;
    let before = snapshot(&engine)?;
    let error = engine
        .finish(completed)
        .err()
        .ok_or("stale open was applied")?;
    assert_eq!(error.info().code, "stale_revision");
    assert_eq!(snapshot(&engine)?, before);

    let cancel = Cancellation::default();
    let completed = engine.prepare(command()?)?.complete_with_cancel(&cancel)?;
    cancel.cancel();
    let error = engine
        .finish(completed)
        .err()
        .ok_or("cancelled open was applied")?;
    assert_eq!(error.info().code, "cancelled");
    assert_eq!(snapshot(&engine)?, before);
    Ok(())
}

#[test]
fn save_of_an_older_revision_keeps_current_dirty_and_never_relabels_a_new_session() -> TestResult {
    let directory = TestDirectory::new()?;
    let old_path = directory.path("captured.cdxml");
    let next_path = directory.path("next-session.cdxml");
    fs::write(&next_path, NATIVE)?;
    let mut engine = Engine::default();
    bond(&mut engine, 200.0)?;
    let work = engine.prepare(Command::parse(
        "document.save_cdxml",
        json!({"path":old_path}),
    )?)?;
    bond(&mut engine, 300.0)?;
    let completed = work.complete()?;
    let saved = serde_json::to_value(engine.finish(completed)?)?;
    independently_read(&fs::read(&old_path)?, 1)?;
    assert_eq!(saved["file_operation"]["input_revision"], 1);
    assert_eq!(engine.state().revision, 2);
    file_state(&engine, Some(&old_path), true)?;
    engine.dispatch("history.undo", json!({}))?;
    file_state(&engine, Some(&old_path), false)?;

    let pending_path = directory.path("previous-session-save.cdxml");
    let work = engine.prepare(Command::parse(
        "document.save_cdxml",
        json!({"path":pending_path}),
    )?)?;
    let completed = work.complete()?;
    engine.dispatch("document.open_cdxml", json!({"path":next_path}))?;
    let before = snapshot(&engine)?;
    let saved = serde_json::to_value(engine.finish(completed)?)?;
    assert_eq!(
        saved["file_operation"]["path"],
        json!(fs::canonicalize(&pending_path)?)
    );
    assert_eq!(
        snapshot(&engine)?,
        before,
        "an older session's publication must not relabel the new document"
    );
    file_state(&engine, Some(&next_path), false)?;
    independently_read(&fs::read(&pending_path)?, 1)?;
    Ok(())
}

#[test]
fn open_rechecks_unsaved_policy_when_an_older_save_changes_the_baseline() -> TestResult {
    let directory = TestDirectory::new()?;
    let path = directory.path("other.cdxml");
    fs::write(&path, NATIVE)?;
    let mut engine = Engine::default();
    bond(&mut engine, 200.0)?;
    let older = engine.prepare(Command::parse(
        "document.save_cdxml",
        json!({"path":directory.path("older.cdxml")}),
    )?)?;
    bond(&mut engine, 300.0)?;
    engine.dispatch(
        "document.save_cdxml",
        json!({"path":directory.path("current.cdxml")}),
    )?;
    let opening = engine.prepare(Command::parse("document.open_cdxml", json!({"path":path}))?)?;
    engine.finish(older.complete()?)?;
    let before = snapshot(&engine)?;
    let error = engine
        .finish(opening.complete()?)
        .err()
        .ok_or("open discarded newly dirty data")?;
    assert_eq!(error.info().code, "unsaved_changes");
    assert_eq!(snapshot(&engine)?, before);
    Ok(())
}

#[cfg(target_os = "linux")]
#[test]
fn registry_save_rejects_non_utf8_canonical_parent_before_publication() -> TestResult {
    use std::{
        ffi::OsStr,
        os::unix::{ffi::OsStrExt, fs::symlink},
    };
    // Linux permits these raw bytes. APFS does not; this fixture must run on a native
    // Linux filesystem rather than a bind-mounted macOS source or target directory.
    let directory = std::env::temp_dir().join(format!(
        "bondline-command-path-{}-{}",
        std::process::id(),
        NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&directory)?;
    let directory = TestDirectory(directory);
    let native_parent = directory.0.join(OsStr::from_bytes(b"native-\xff"));
    fs::create_dir(&native_parent)?;
    let alias = directory.path("utf8-alias");
    symlink(&native_parent, &alias)?;
    let chosen = alias.join("drawing.cdxml");
    assert!(chosen.to_str().is_some());
    assert!(fs::canonicalize(&alias)?.to_str().is_none());
    let mut engine = Engine::default();
    bond(&mut engine, 200.0)?;
    let before = snapshot(&engine)?;
    let error = engine
        .dispatch("document.save_cdxml", json!({"path":chosen}))
        .err()
        .ok_or("unrepresentable canonical destination was published")?;
    assert_eq!(error.info().code, "invalid_path");
    assert_eq!(snapshot(&engine)?, before);
    assert!(!native_parent.join("drawing.cdxml").exists());
    assert_eq!(
        fs::read_dir(&native_parent)?.count(),
        0,
        "no temporary or published bytes may remain"
    );

    fs::write(native_parent.join("drawing.cdxml"), b"existing bytes")?;
    let error = engine
        .dispatch(
            "document.save_cdxml",
            json!({"path":chosen,"overwrite":true}),
        )
        .err()
        .ok_or("unrepresentable overwrite was published")?;
    assert_eq!(error.info().code, "invalid_path");
    assert_eq!(snapshot(&engine)?, before);
    assert_eq!(
        fs::read(native_parent.join("drawing.cdxml"))?,
        b"existing bytes"
    );

    // Native PathBuf callers retain their wider path contract.
    let receipt = bondline_io::files::save_cdxml_file(
        &native_parent.join("native.cdxml"),
        engine.stored_document(),
        false,
        &Cancellation::default(),
    )?;
    assert!(receipt.path.to_str().is_none());
    assert_eq!(
        fs::read(&receipt.path)?,
        engine.stored_document().to_cdxml()?.as_ref()
    );
    Ok(())
}
