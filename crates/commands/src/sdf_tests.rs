use super::*;
use bondline_io::sdf::{RecordId, SdfDocument};
use serde_json::json;
use std::fs;

#[path = "../../../xtask/oracles/sdf_test_support.rs"]
mod support;

type TestResult = Result<(), Box<dyn std::error::Error>>;
const SOURCE: &[u8] = include_bytes!("../../io/tests/fixtures/sdf/ordered-fields-v2000.sdf");

fn contents() -> Result<(StoredContent, StoredContent), Box<dyn std::error::Error>> {
    let original = SdfDocument::parse(SOURCE.into())?;
    let changed = original.with_field_value(
        RecordId::new(1).ok_or("record")?,
        5,
        &["revised", "multiline"],
    )?;
    Ok((
        StoredContent::from_sdf(original.clone()),
        StoredContent::Sdf {
            original,
            current: changed,
        },
    ))
}

#[test]
fn whole_collection_history_separates_current_original_and_saved_baselines() -> TestResult {
    let (original, changed) = contents()?;
    let mut engine = Engine::from_content(original.clone());
    assert_eq!(
        serde_json::to_value(engine.state())?["collection"],
        support::expected_summary(false)
    );
    assert!(engine.document().is_err());
    assert!(engine.stored_document().is_err());
    assert!(engine.commit(changed.clone())?);
    assert_eq!(engine.content(), &changed);
    assert!(engine.state().file.dirty);
    assert!(engine.state().source.ok_or("source")?.modified);
    assert_eq!((engine.state().revision, engine.state().undo_depth), (1, 1));
    assert!(!engine.commit(changed.clone())?);
    engine.dispatch("history.undo", json!({}))?;
    assert_eq!(engine.content(), &original);
    assert!(!engine.state().file.dirty);
    assert!(!engine.state().source.ok_or("source")?.modified);
    engine.dispatch("history.redo", json!({}))?;
    assert_eq!(engine.content(), &changed);
    assert!(engine.state().file.dirty);
    assert_eq!(engine.content().sdf()?.as_bytes().len(), 704);
    Ok(())
}

#[test]
fn captured_whole_sdf_save_holds_the_single_slot_through_publication_and_finishes_truthfully()
-> TestResult {
    let directory = support::SdfTestDirectory::new()?;
    let (original, changed) = contents()?;
    let mut engine = Engine::from_content(original.clone());
    let path = directory.path("captured.sdf");
    let plan = engine.prepare(Command::parse("document.save_sdf", json!({"path":path}))?)?;
    assert!(matches!(
        engine.prepare(Command::parse(
            "document.save_sdf",
            json!({"path":directory.path("busy.sdf")})
        )?),
        Err(Error::SaveBusy)
    ));
    engine.commit(changed.clone())?;
    let completed = plan.complete()?;
    support::independently_read_sdf(&path, SOURCE, false)?;
    assert!(matches!(
        engine.prepare(Command::parse(
            "document.save_sdf",
            json!({"path":directory.path("still-busy.sdf")})
        )?),
        Err(Error::SaveBusy)
    ));
    let receipt = engine.finish(completed)?;
    assert_eq!(receipt.file_operation.ok_or("receipt")?.byte_length, 690);
    assert_eq!(engine.content(), &changed);
    assert!(engine.state().file.dirty);
    assert!(engine.state().source.ok_or("source")?.modified);
    let saved = engine.dispatch("document.save_sdf", json!({"path":path,"overwrite":true}))?;
    assert_eq!(saved.file_operation.ok_or("receipt")?.byte_length, 704);
    assert_eq!(fs::read(&path)?.len(), 704);
    assert_eq!(
        fs::read(&path)?,
        [&SOURCE[..383], b"revised\nmultiline\n", &SOURCE[387..]].concat()
    );
    let external = support::describe_source("sdf_oracle.py", &fs::read(&path)?)?;
    assert_eq!(external["records"][0]["range"], json!([0, 438]));
    assert_eq!(external["records"][1]["range"], json!([438, 704]));
    assert_eq!(
        external["records"][0]["fields"][4]["value_lines"],
        json!(["one"])
    );
    assert_eq!(
        external["records"][0]["fields"][5]["value_lines"],
        json!(["revised", "multiline"])
    );
    assert_eq!(external["records"][0]["rdkit"]["smiles"], "CC");
    assert_eq!(external["records"][1]["rdkit"]["smiles"], "CC");
    assert_eq!(
        serde_json::to_value(engine.state())?["collection"]["byte_length"],
        704
    );
    assert_eq!(
        engine.state().source,
        Some(bondline_io::SourceInfo {
            format: "sdf",
            byte_length: 690,
            modified: true
        })
    );
    assert!(!engine.state().file.dirty);
    engine.dispatch("history.undo", json!({}))?;
    assert_eq!(engine.content(), &original);
    assert!(engine.state().file.dirty);
    engine.dispatch("history.redo", json!({}))?;
    assert_eq!(engine.content(), &changed);
    assert!(!engine.state().file.dirty);
    Ok(())
}

#[test]
fn drawing_edits_cross_format_saves_and_unreadable_captures_refuse_without_changing_sdf()
-> TestResult {
    let directory = support::SdfTestDirectory::new()?;
    let (original, _) = contents()?;
    let mut engine = Engine::from_content(original.clone());
    let before = serde_json::to_value(engine.state())?;
    let missing = directory.path("no-such-directory/output");
    for (name, params) in [
        (
            "page.set_size",
            json!({"page_id":1,"width_pt":612,"height_pt":792}),
        ),
        (
            "bond.create",
            json!({"page_id":1,"begin_x_pt":100,"begin_y_pt":200,"end_x_pt":140,"end_y_pt":200}),
        ),
        (
            "bond.extend",
            json!({"expected_revision":0,"page_id":1,"fragment_id":1,"begin_atom_id":2,"end_x_pt":180,"end_y_pt":240,"hydrogen_policy":"rectify_terminal_carbon"}),
        ),
        (
            "render.capture",
            json!({"page_id":1,"width_px":640,"height_px":480,"backend":"cpu"}),
        ),
        ("document.save_cdxml", json!({"path":missing})),
        ("document.save_cdx", json!({"path":missing})),
        ("document.save_mol", json!({"path":missing})),
    ] {
        assert_eq!(
            engine
                .dispatch(name, params)
                .err()
                .ok_or("wrong kind")?
                .info()
                .code,
            if name == "render.capture" {
                "unsupported_scale"
            } else {
                "unsupported_content"
            },
            "{name}"
        );
        assert_eq!(engine.content(), &original);
        assert_eq!(serde_json::to_value(engine.state())?, before);
        assert_eq!(fs::read_dir(directory.path("."))?.count(), 0);
    }
    let mut drawing = Engine::default();
    let old = serde_json::to_value(drawing.state())?;
    assert_eq!(
        drawing
            .dispatch("document.save_sdf", json!({"path":missing}))
            .err()
            .ok_or("drawing to SDF")?
            .info()
            .code,
        "unsupported_content"
    );
    assert_eq!(serde_json::to_value(drawing.state())?, old);
    assert_eq!(fs::read_dir(directory.path("."))?.count(), 0);
    Ok(())
}

#[test]
fn sdf_source_only_changes_and_distinct_originals_remain_whole_history_and_storage() -> TestResult {
    let original = SdfDocument::parse(SOURCE.into())?;
    let current = original.with_field_value(RecordId::new(1).ok_or("record")?, 5, &["TWO"])?;
    let other_original =
        original.with_field_value(RecordId::new(1).ok_or("record")?, 0, &["FIRST"])?;
    let first = StoredContent::Sdf {
        original: original.clone(),
        current: current.clone(),
    };
    let second = StoredContent::Sdf {
        original: other_original,
        current,
    };
    let mut engine = Engine::from_content(first.clone());
    assert!(engine.commit(second.clone())?);
    assert!(engine.state().file.dirty);
    assert_eq!(engine.content().collection(), first.collection());
    let pool = StoredContent::retained_bytes(
        std::iter::once(engine.document.as_ref())
            .chain(std::iter::once(engine.saved.as_ref()))
            .chain(engine.undo.iter().map(Arc::as_ref)),
    );
    assert_eq!(
        pool,
        2 * size_of::<StoredContent>()
            + 3 * 690
            + 3 * 2 * size_of::<bondline_io::sdf::RecordInfo>()
    );
    engine.trim_history(pool);
    assert_eq!(engine.state().undo_depth, 1);
    engine.dispatch("history.undo", json!({}))?;
    assert_eq!(engine.content(), &first);
    assert!(!engine.state().file.dirty);
    engine.dispatch("history.redo", json!({}))?;
    assert_eq!(engine.content(), &second);
    assert!(engine.state().file.dirty);
    assert_eq!(original.as_bytes(), SOURCE);
    Ok(())
}

#[test]
fn old_sdf_saves_and_opens_cannot_relabel_new_generations_or_newer_sdf_values() -> TestResult {
    let directory = support::SdfTestDirectory::new()?;
    let (original, changed) = contents()?;
    let mut engine = Engine::from_content(original.clone());
    let output = directory.path("old-generation.sdf");
    let save = engine.prepare(Command::parse("document.save_sdf", json!({"path":output}))?)?;
    let native = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../io/tests/fixtures/bondline-v0.cdxml"
    );
    engine.dispatch("document.open_cdxml", json!({"path":native}))?;
    let before = serde_json::to_value(engine.state())?;
    let receipt = engine.finish(save.complete()?)?;
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert_eq!(
        receipt.file_operation.ok_or("old save receipt")?.format,
        FileFormat::Sdf
    );
    support::independently_read_sdf(&output, SOURCE, false)?;
    assert!(engine.document().is_ok());
    assert!(engine.state().collection.is_none());
    let input = directory.path("candidate.sdf");
    fs::write(&input, SOURCE)?;
    engine.dispatch("document.open_sdf", json!({"path":input}))?;
    let plan = engine.prepare(Command::parse("document.open_sdf", json!({"path":input}))?)?;
    engine.commit(changed.clone())?;
    let before = serde_json::to_value(engine.state())?;
    assert!(matches!(
        engine.finish(plan.complete()?),
        Err(Error::StaleRevision)
    ));
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert_eq!(engine.content(), &changed);
    Ok(())
}

#[test]
fn malformed_later_sdf_open_keeps_dirty_whole_content_saved_baseline_and_history() -> TestResult {
    let directory = support::SdfTestDirectory::new()?;
    let (original, changed) = contents()?;
    let mut engine = Engine::from_content(original.clone());
    engine.commit(changed.clone())?;
    let path = directory.path("invalid-second.sdf");
    fs::write(&path, [&SOURCE[..424], b"invalid second record\n"].concat())?;
    let before = serde_json::to_value(engine.state())?;
    assert_eq!(
        engine
            .dispatch("document.open_sdf", json!({"path":path}))
            .err()
            .ok_or("dirty guard")?
            .info()
            .code,
        "unsaved_changes"
    );
    assert_eq!(
        engine
            .dispatch(
                "document.open_sdf",
                json!({"path":path,"discard_unsaved":true})
            )
            .err()
            .ok_or("bad later record")?
            .info()
            .code,
        "invalid_sdf"
    );
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert_eq!(engine.content(), &changed);
    assert_eq!(engine.saved.as_ref(), &original);
    engine.dispatch("history.undo", json!({}))?;
    assert_eq!(engine.content(), &original);
    assert!(!engine.state().file.dirty);
    engine.dispatch("history.redo", json!({}))?;
    assert_eq!(engine.content(), &changed);
    assert!(engine.state().file.dirty);
    Ok(())
}

#[test]
fn sdf_save_reservation_releases_on_drop_cancellation_and_refusal_without_saving_state()
-> TestResult {
    let directory = support::SdfTestDirectory::new()?;
    let (original, _) = contents()?;
    let mut engine = Engine::from_content(original);
    let path = directory.path("reserved.sdf");
    let before = serde_json::to_value(engine.state())?;
    let plan = engine.prepare(Command::parse("document.save_sdf", json!({"path":path}))?)?;
    drop(plan);
    let plan = engine.prepare(Command::parse("document.save_sdf", json!({"path":path}))?)?;
    let cancel = bondline_render::Cancellation::default();
    cancel.cancel();
    assert_eq!(
        plan.complete_with_cancel(&cancel)
            .err()
            .ok_or("cancelled save")?
            .info()
            .code,
        "cancelled"
    );
    assert!(!path.exists());
    fs::write(&path, b"existing")?;
    let plan = engine.prepare(Command::parse("document.save_sdf", json!({"path":path}))?)?;
    assert_eq!(
        plan.complete().err().ok_or("no clobber")?.info().code,
        "already_exists"
    );
    assert_eq!(fs::read(&path)?, b"existing");
    let completed = engine
        .prepare(Command::parse(
            "document.save_sdf",
            json!({"path":path,"overwrite":true}),
        )?)?
        .complete()?;
    assert!(matches!(
        engine.prepare(Command::parse(
            "document.save_sdf",
            json!({"path":path,"overwrite":true})
        )?),
        Err(Error::SaveBusy)
    ));
    drop(completed);
    assert_eq!(
        serde_json::to_value(engine.state())?,
        before,
        "discarded receipt does not claim an applied save"
    );
    let saved = engine.dispatch("document.save_sdf", json!({"path":path,"overwrite":true}))?;
    assert!(saved.file_operation.is_some());
    assert!(!saved.file.dirty);
    support::independently_read_sdf(&path, SOURCE, false)?;
    Ok(())
}

#[test]
fn sdf_close_save_uses_current_trusted_receipt_and_kind_or_revision_aba_cannot_restore_ack()
-> TestResult {
    let directory = support::SdfTestDirectory::new()?;
    let (original, changed) = contents()?;
    let path = directory.path("close.sdf");
    for mode in 0..3 {
        let mut engine = Engine::from_content(original.clone());
        engine.dispatch("document.save_sdf", json!({"path":path,"overwrite":true}))?;
        engine.commit(changed.clone())?;
        engine.set_ui_file_host(true);
        engine.dispatch(
            "document.close",
            json!({"expected_revision":1,"action":"request"}),
        )?;
        let epoch = engine.state().close_epoch;
        engine.dispatch(
            "ui.file",
            json!({"expected_revision":1,"action":"save","close_epoch":epoch}),
        )?;
        let id = engine
            .state()
            .ui_file
            .request
            .ok_or("close Save request")?
            .id;
        assert!(matches!(
            engine.claim_ui_file(id)?,
            UiFilePlan::SaveExisting {
                format: FileFormat::Sdf,
                ..
            }
        ));
        let prepared = engine.prepare_ui_file(
            id,
            Command::parse(
                "document.save_sdf",
                json!({"path":path.canonicalize()?,"overwrite":true}),
            )?,
        )?;
        let result = engine.finish(prepared.complete()?)?;
        assert_eq!(
            result.file_operation.ok_or("receipt")?.ui_request_current,
            Some(true)
        );
        assert!(!engine.complete_close());
        if mode == 0 {
            assert!(engine.finish_ui_file(id, false));
            engine.dispatch(
                "document.close",
                json!({"expected_revision":1,"action":"request"}),
            )?;
            assert!(engine.complete_close());
            assert_eq!(fs::read(&path)?.len(), 704);
        } else if mode == 1 {
            engine.commit(original.clone())?;
            engine.dispatch("history.undo", json!({}))?;
            assert_eq!(engine.content(), &changed);
            assert!(!engine.state().file.dirty);
            assert!(!engine.finish_ui_file(id, false));
            assert!(!engine.complete_close());
        } else {
            engine.dispatch(
                "document.close",
                json!({"expected_revision":1,"action":"cancel"}),
            )?;
            engine.dispatch("document.open_cdxml",json!({"path":concat!(env!("CARGO_MANIFEST_DIR"),"/../io/tests/fixtures/bondline-v0.cdxml")}))?;
            engine.dispatch("document.open_sdf", json!({"path":path}))?;
            assert_eq!(engine.state().file.format, FileFormat::Sdf);
            assert_eq!(
                engine.state().file.path.as_deref(),
                path.canonicalize()?.to_str()
            );
            assert!(!engine.finish_ui_file(id, false));
            assert!(!engine.complete_close());
        }
    }
    Ok(())
}
