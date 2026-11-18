//! Literal binary metadata crosses existing file/history owners without granting drawing authority.
use bondline_commands::{Command, Engine};
use serde_json::{Value, json};
use std::{fs, sync::Arc};
#[path = "../../../xtask/oracles/cdx_literal_support.rs"]
#[allow(
    dead_code,
    reason = "independent immutable source/model/file expectations"
)]
mod f;
#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "existing isolated real-file directory")]
mod files;
type TestResult<T = ()> = f::Result<T>;

fn exact(e: &Engine, name: &str) -> TestResult {
    assert_eq!(serde_json::to_value(e.document()?)?, f::model(name)?);
    let stored = e.stored_document()?;
    assert!(!stored.is_fully_mapped());
    assert!(!stored.has_display_projection());
    assert!(!stored.has_boxed_atom_labels());
    assert!(!stored.has_connected_atom_labels());
    assert!(!stored.has_mixed_atom_labels());
    assert!(!stored.has_flagged_atom_labels());
    assert!(!stored.has_label_defaults_projection());
    assert_eq!(stored.to_cdx()?.as_ref(), f::source(name));
    Ok(())
}

#[test]
fn literal_cdx_atoms_survive_registered_open_unchanged_unicode_save_and_reopen() -> TestResult {
    assert_eq!(
        serde_json::to_value(bondline_commands::registry())?,
        serde_json::from_str::<Value>(f::METADATA49)?
    );
    for name in [
        "methylammonium_explicit",
        "methylammonium_extended",
        "methylammonium_embedded",
        "methyl_doublet",
        "deuteron",
        "charge4_low_one",
        "charge4_low_noise_max",
    ] {
        let directory = files::FileTestDirectory::new()?;
        let input = directory.path("literal-原-λ.cdx");
        let output = directory.path("literal-保存-λ.cdx");
        fs::write(&input, f::source(name))?;
        let mut e = Engine::default();
        e.dispatch("document.open_cdx", json!({"path":input}))?;
        exact(&e, name)?;
        let before = serde_json::to_value(e.state())?;
        e.dispatch("document.save_cdx", json!({"path":output}))?;
        assert_eq!(fs::read(&output)?, f::source(name));
        assert_eq!(f::read(&fs::read(&output)?)?, f::facts(name));
        e.dispatch("document.open_cdx", json!({"path":output}))?;
        exact(&e, name)?;
        assert_eq!(
            serde_json::to_value(e.state())?["document"],
            before["document"]
        );
        assert_eq!((e.state().undo_depth, e.state().redo_depth), (0, 0));
        assert!(!e.state().file.dirty);
        assert_eq!(fs::read(&input)?, f::source(name));
    }
    Ok(())
}

#[test]
fn equal_model_sources_keep_distinct_capture_and_save_generations() -> TestResult {
    let directory = files::FileTestDirectory::new()?;
    let a = directory.path("absent-原.cdx");
    let b = directory.path("explicit-zero-λ.cdx");
    let captured = directory.path("captured-保存.cdx");
    fs::write(&a, f::source("absent_implied"))?;
    fs::write(&b, f::source("scalar_zero_implied"))?;
    let mut e = Engine::default();
    e.dispatch("document.open_cdx", json!({"path":a}))?;
    let model = e.stored_document()?.model_snapshot();
    let stamp = e.capture_stamp();
    let file = e.ui_file_context_stamp();
    let save = e
        .prepare(Command::parse(
            "document.save_cdx",
            json!({"path":captured}),
        )?)?
        .complete()?;
    e.dispatch("document.open_cdx", json!({"path":b}))?;
    assert_eq!(e.document()?, model.as_ref());
    assert!(!e.capture_stamp_matches(&stamp));
    assert!(!e.ui_file_context_stamp_matches(&file));
    exact(&e, "scalar_zero_implied")?;
    let before = serde_json::to_value(e.state())?;
    e.finish(save)?;
    assert_eq!(
        serde_json::to_value(e.state())?,
        before,
        "old Save publication cannot associate equal-model B"
    );
    assert_eq!(fs::read(&captured)?, f::source("absent_implied"));
    assert_eq!(f::read(&fs::read(&captured)?)?, f::facts("absent_implied"));
    assert_eq!(fs::read(&b)?, f::source("scalar_zero_implied"));
    Ok(())
}

#[test]
fn literal_metadata_keeps_refusals_noop_ownership_and_detached_page_history() -> TestResult {
    let directory = files::FileTestDirectory::new()?;
    let input = directory.path("metadata.cdx");
    let refused = directory.path("refused.cdx");
    fs::write(&input, f::source("methylammonium_explicit"))?;
    fs::write(&refused, b"preserve unrelated bytes")?;
    let mut e = Engine::default();
    e.dispatch("document.open_cdx", json!({"path":input}))?;
    for (command, p) in [
        (
            "atom.set_label",
            json!({"expected_revision":1,"page_id":1,"fragment_id":1,"atom_id":1,"element":6,"explicit_hydrogens":3}),
        ),
        (
            "bond.create",
            json!({"page_id":1,"begin_x_pt":300,"begin_y_pt":300,"end_x_pt":348,"end_y_pt":300}),
        ),
        (
            "ui.selection",
            json!({"action":"set","expected_revision":1,"page_id":1,"atom_ids":[1]}),
        ),
        (
            "render.capture",
            json!({"page_id":1,"width_px":1088,"height_px":1088,"backend":"cpu"}),
        ),
        (
            "document.save_cdxml",
            json!({"path":refused,"overwrite":true}),
        ),
        (
            "document.save_mol",
            json!({"path":refused,"overwrite":true}),
        ),
    ] {
        let before = serde_json::to_value(e.state())?;
        let model = e.stored_document()?.model_snapshot();
        let stamp = e.capture_stamp();
        assert!(
            e.dispatch(command, p).is_err(),
            "{command} cannot gain authority from metadata"
        );
        assert_eq!(serde_json::to_value(e.state())?, before);
        assert!(e.capture_stamp_matches(&stamp));
        assert!(Arc::ptr_eq(&model, &e.stored_document()?.model_snapshot()));
        assert_eq!(fs::read(&refused)?, b"preserve unrelated bytes");
    }
    let before = serde_json::to_value(e.state())?;
    let model = e.stored_document()?.model_snapshot();
    let stamp = e.capture_stamp();
    assert!(
        !e.dispatch(
            "page.set_size",
            json!({"page_id":1,"width_pt":210.0*72.0/25.4,"height_pt":297.0*72.0/25.4})
        )?
        .changed
    );
    assert_eq!(serde_json::to_value(e.state())?, before);
    assert!(Arc::ptr_eq(&model, &e.stored_document()?.model_snapshot()));
    assert!(e.capture_stamp_matches(&stamp));
    e.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":612,"height_pt":792}),
    )?;
    assert!(e.state().file.dirty);
    assert_eq!(e.state().undo_depth, 1);
    assert!(
        e.dispatch(
            "document.save_cdx",
            json!({"path":refused,"overwrite":true})
        )
        .is_err()
    );
    assert_eq!(fs::read(&refused)?, b"preserve unrelated bytes");
    e.dispatch("history.undo", json!({}))?;
    exact(&e, "methylammonium_explicit")?;
    assert!(!e.state().file.dirty);
    e.dispatch("history.redo", json!({}))?;
    assert!(e.state().file.dirty);
    e.dispatch("history.undo", json!({}))?;
    exact(&e, "methylammonium_explicit")?;
    Ok(())
}
