//! Explicit run typography and immutable scalar metadata are separate source facts.
#[path = "../../../xtask/oracles/registry_fixture_support.rs"]
mod registry_fixture;
use bondline_commands::{Command, Engine};
use bondline_io::StoredDocument;
use bondline_model::PageSize;
use serde_json::json;
use std::{fs, sync::Arc};
#[path = "../../../xtask/oracles/created_label_support.rs"]
#[allow(
    dead_code,
    reason = "already frozen original graph and exact edit sources"
)]
mod existing;
#[path = "../../../xtask/oracles/label_metadata_support.rs"]
#[allow(
    dead_code,
    reason = "independent immutable metadata sources and file facts"
)]
mod f;
#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "existing isolated real-file directory")]
mod files;
type TestResult<T = ()> = existing::Result<T>;

fn metadata(raw: &str) -> String {
    assert!(raw.starts_with("<CDXML "));
    raw.replacen(
        "<CDXML ",
        "<CDXML LabelFont='42' LabelSize='18.000' LabelFace='96' ",
        1,
    )
}
fn engine(name: &str) -> existing::Result<Engine> {
    Ok(Engine::from_document(StoredDocument::from_cdxml(
        metadata(existing::source(name)).as_bytes().into(),
        PageSize::A4,
    )?))
}

#[test]
fn explicit_runs_beside_checked_defaults_render_without_using_the_default_size_or_face()
-> TestResult {
    let e = engine("created_3_black")?;
    let plain = StoredDocument::from_cdxml(
        existing::source("created_3_black").as_bytes().into(),
        PageSize::A4,
    )?;
    assert_eq!(e.stored_document()?.model(), plain.model());
    assert!(e.stored_document()?.has_display_projection());
    assert!(!e.stored_document()?.is_fully_mapped());
    let page = plain.model().pages()[0].id();
    let a = bondline_io::display::drawing_page_scene(
        1088,
        1088,
        e.stored_document()?,
        page,
        &Default::default(),
    )?;
    let b =
        bondline_io::display::drawing_page_scene(1088, 1088, &plain, page, &Default::default())?;
    assert_eq!(
        bondline_render::cpu_scene(a.scene())?,
        bondline_render::cpu_scene(b.scene())?
    );
    Ok(())
}

#[test]
fn command48_preserves_every_unused_default_lexeme_while_editing_its_literal_label() -> TestResult {
    let mut e = engine("created_3_black")?;
    let original = e.content().clone();
    assert!(
        e.dispatch(
            "atom.set_authored_label",
            existing::label_params(0, 1, 7, 2)
        )?
        .changed
    );
    assert_eq!(
        e.stored_document()?.to_cdxml()?.as_ref(),
        metadata(existing::source("endpoint_edited_created_3")).as_bytes()
    );
    assert_eq!(e.state().undo_depth, 1);
    e.dispatch("history.undo", json!({}))?;
    assert_eq!(e.content(), &original);
    assert_eq!(
        e.stored_document()?.to_cdxml()?.as_ref(),
        metadata(existing::source("created_3_black")).as_bytes()
    );
    Ok(())
}

#[test]
fn command49_declared_style_does_not_borrow_or_rewrite_unused_default_metadata() -> TestResult {
    let mut e = engine("base")?;
    assert!(
        e.dispatch(
            "atom.create_authored_label",
            existing::params(0, 3, 6, 2, existing::black_style())
        )?
        .changed
    );
    assert_eq!(
        e.stored_document()?.to_cdxml()?.as_ref(),
        metadata(existing::source("created_3_black")).as_bytes()
    );
    assert_eq!(e.state().undo_depth, 1);
    Ok(())
}

#[test]
fn identical_models_with_different_source_defaults_retire_old_gesture_form_and_save_owners()
-> TestResult {
    let directory = files::FileTestDirectory::new()?;
    let a = directory.path("defaults-A-原.cdxml");
    let b = directory.path("defaults-B-λ.cdxml");
    let saved_a = directory.path("captured-A-保存.cdxml");
    let raw_a = metadata(existing::source("base"));
    let raw_b = raw_a.replacen("LabelSize='18.000'", "LabelSize='19.000'", 1);
    fs::write(&a, &raw_a)?;
    fs::write(&b, &raw_b)?;
    let mut e = Engine::default();
    e.dispatch("document.open_cdxml", json!({"path":a}))?;
    e.dispatch("ui.selection", json!({"action":"set","expected_revision":e.state().revision,"page_id":1,"atom_ids":[1,2,3,4,5]}))?;
    let model_a = e.stored_document()?.model_snapshot();
    let stamp_a = e.capture_stamp();
    let gesture_a = e.selection_context()?;
    assert!(gesture_a.is_current(&e));
    e.dispatch("document.open_cdxml", json!({"path":b}))?;
    assert_eq!(e.stored_document()?.model(), model_a.as_ref());
    assert!(!e.capture_stamp_matches(&stamp_a));
    assert!(
        !gesture_a.is_current(&e),
        "Open B alone retires A's gesture"
    );
    e.dispatch("document.open_cdxml", json!({"path":a}))?;
    e.dispatch("ui.selection", json!({"action":"set","expected_revision":e.state().revision,"page_id":1,"atom_ids":[1,2,3,4,5]}))?;
    assert!(
        !gesture_a.is_current(&e),
        "equal-model A→B→A cannot revive it"
    );
    assert!(e.selection_context()?.is_current(&e));
    let stamp_a = e.capture_stamp();
    e.set_ui_file_host(true);
    e.dispatch("ui.atom_label", json!({"action":"open","expected_revision":e.state().revision,"page_id":1,"fragment_id":1,"atom_id":3}))?;
    let request_a = e.ui_atom_label_request().ok_or("A form")?;
    assert!(request_a.current);
    let save = e
        .prepare(Command::parse(
            "document.save_cdxml",
            json!({"path":saved_a}),
        )?)?
        .complete()?;
    e.dispatch("document.open_cdxml", json!({"path":b}))?;
    assert_eq!(e.stored_document()?.model(), model_a.as_ref());
    assert!(!e.capture_stamp_matches(&stamp_a));
    assert!(!gesture_a.is_current(&e));
    assert!(!e.ui_atom_label_request_current(request_a.id));
    let before = serde_json::to_value(e.state())?;
    assert_eq!(
        e.apply_ui_atom_label(request_a.id, 7, 1)
            .err()
            .ok_or("stale form applied")?
            .info()
            .code,
        "ui_stale_request"
    );
    assert_eq!(serde_json::to_value(e.state())?, before);
    let published = serde_json::to_value(e.finish(save)?)?;
    assert_eq!(
        published["file_operation"]["path"],
        json!(fs::canonicalize(&saved_a)?)
    );
    assert_eq!(
        serde_json::to_value(e.state())?,
        before,
        "old publication cannot associate/acknowledge equal-model source B"
    );
    assert_eq!(fs::read(&saved_a)?, raw_a.as_bytes());
    assert_eq!(e.stored_document()?.to_cdxml()?.as_ref(), raw_b.as_bytes());
    assert_eq!(fs::read(&a)?, raw_a.as_bytes());
    assert_eq!(fs::read(&b)?, raw_b.as_bytes());
    Ok(())
}

#[test]
fn certified_label_noop_keeps_model_sharing_redo_selection_and_capture_identity() -> TestResult {
    let mut e = engine("created_3_black")?;
    e.dispatch(
        "atom.set_authored_label",
        existing::label_params(0, 1, 7, 2),
    )?;
    e.dispatch("history.undo", json!({}))?;
    e.dispatch("ui.selection",json!({"action":"set","expected_revision":e.state().revision,"page_id":1,"atom_ids":[1,2,3,4,5]}))?;
    let state = serde_json::to_value(e.state())?;
    let model = e.stored_document()?.model_snapshot();
    let stamp = e.capture_stamp();
    let gesture = e.selection_context()?;
    assert!(
        !e.dispatch(
            "atom.set_authored_label",
            existing::label_params(e.state().revision, 1, 8, 1)
        )?
        .changed
    );
    assert_eq!(serde_json::to_value(e.state())?, state);
    assert!(Arc::ptr_eq(&model, &e.stored_document()?.model_snapshot()));
    assert!(e.capture_stamp_matches(&stamp));
    assert!(gesture.is_current(&e));
    assert_eq!(
        e.stored_document()?.to_cdxml()?.as_ref(),
        metadata(existing::source("created_3_black")).as_bytes()
    );
    e.dispatch("history.redo", json!({}))?;
    assert_eq!(
        e.stored_document()?.to_cdxml()?.as_ref(),
        metadata(existing::source("endpoint_edited_created_3")).as_bytes()
    );
    Ok(())
}

fn original(name: &str) -> TestResult<Engine> {
    Ok(Engine::from_document(StoredDocument::from_cdxml(
        f::source(name).as_bytes().into(),
        PageSize::A4,
    )?))
}
fn exact(e: &Engine, before: &serde_json::Value, name: &str) -> TestResult {
    let document = e.stored_document()?;
    assert!(document.has_label_defaults_projection());
    assert!(document.has_display_projection());
    assert!(!document.is_fully_mapped());
    assert!(!document.has_boxed_atom_labels());
    assert!(!document.has_connected_atom_labels());
    assert!(!document.has_mixed_atom_labels());
    assert!(!document.has_flagged_atom_labels());
    let bytes = document.to_cdxml()?;
    assert_eq!(bytes.as_ref(), f::source(name).as_bytes());
    assert_eq!(f::read(bytes.as_ref())?, f::facts(name));
    assert_eq!(
        serde_json::to_value(document.model())?,
        f::model_for_case(before, name)?
    );
    Ok(())
}

#[test]
fn frozen_metadata_sources_compose_explicit_edits_movement_history_and_real_file_continuation()
-> TestResult {
    let old: serde_json::Value =
        serde_json::from_str(&registry_fixture::with_cdx_caption_edit(f::METADATA49))?;
    let commands = bondline_commands::registry();
    assert_eq!(commands.len(), 54);
    assert_eq!(commands[49].name, "ui.caption");
    assert_eq!(
        serde_json::to_value(&commands[..49])?,
        old,
        "all49 records/schema/order unchanged"
    );
    let mut e = original("mixed_a")?;
    let before = serde_json::to_value(e.stored_document()?.model())?;
    let mut seed = 0x2805_7341_u32;
    for _ in 0..8 {
        e.dispatch(
            "atom.set_authored_label",
            f::label_params(e.state().revision, 1, 7, 2),
        )?;
        exact(&e, &before, "edited_endpoint")?;
        e.dispatch(
            "atom.create_authored_label",
            f::create_params(e.state().revision, 3, 6, 2, f::black_style()),
        )?;
        e.dispatch("ui.selection",json!({"action":"set","expected_revision":e.state().revision,"page_id":1,"atom_ids":[1,2,3,4,5]}))?;
        let p = f::motion(e.state().revision, e.selection().epoch, 24., -12.);
        e.dispatch("selection.translate", p)?;
        exact(&e, &before, "compound")?;
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        for _ in 0..1 + seed % 3 {
            for _ in 0..3 {
                e.dispatch("history.undo", json!({}))?;
            }
            exact(&e, &before, "mixed_a")?;
            for _ in 0..3 {
                e.dispatch("history.redo", json!({}))?;
            }
            exact(&e, &before, "compound")?;
        }
        for _ in 0..3 {
            e.dispatch("history.undo", json!({}))?;
        }
        assert_eq!(e.state().undo_depth, 0);
    }
    let directory = files::FileTestDirectory::new()?;
    let partial = directory.path("metadata-partial-保存-λ.cdxml");
    let all = directory.path("metadata-all-保存-λ.cdxml");
    let mut e = original("mixed_a")?;
    e.dispatch(
        "atom.create_authored_label",
        f::create_params(0, 3, 6, 2, f::black_style()),
    )?;
    exact(&e, &before, "created_3")?;
    e.dispatch("document.save_cdxml", json!({"path":partial}))?;
    assert_eq!(fs::read(&partial)?, f::source("created_3").as_bytes());
    e.dispatch("document.open_cdxml", json!({"path":partial}))?;
    for id in [2, 4] {
        e.dispatch(
            "atom.create_authored_label",
            f::create_params(e.state().revision, id, 6, 2, f::black_style()),
        )?;
    }
    exact(&e, &before, "all_printed")?;
    e.dispatch("document.save_cdxml", json!({"path":all}))?;
    assert_eq!(f::read(&fs::read(&all)?)?, f::facts("all_printed"));
    e.dispatch("document.open_cdxml", json!({"path":all}))?;
    e.dispatch("ui.selection",json!({"action":"set","expected_revision":e.state().revision,"page_id":1,"atom_ids":[1,2,3,4,5]}))?;
    let p = f::motion(e.state().revision, e.selection().epoch, 24., -12.);
    e.dispatch("selection.translate", p)?;
    exact(&e, &before, "all_moved")?;
    Ok(())
}

#[test]
fn precise_metadata_permission_never_authorizes_other_chemical_page_style_or_lossy_writers()
-> TestResult {
    let mut e = original("caption_owners")?;
    let directory = files::FileTestDirectory::new()?;
    for (name, p) in [
        ("atom.set_label", f::label_params(0, 1, 8, 1)),
        (
            "page.set_size",
            json!({"page_id":1,"width_pt":512,"height_pt":512}),
        ),
        (
            "document.set_drawing_settings",
            json!({"expected_revision":0,"bond_length_pt":48,"chain_angle_degrees":120}),
        ),
        (
            "bond.create",
            json!({"page_id":1,"begin_x_pt":400,"begin_y_pt":400,"end_x_pt":448,"end_y_pt":400}),
        ),
        (
            "document.save_mol",
            json!({"path":directory.path("refused.mol")}),
        ),
        (
            "document.save_cdx",
            json!({"path":directory.path("refused.cdx")}),
        ),
    ] {
        let before = serde_json::to_value(e.state())?;
        let owner = e.capture_stamp();
        let snapshot = e.stored_document()?.model_snapshot();
        assert!(e.dispatch(name, p).is_err(), "{name} must refuse");
        assert_eq!(serde_json::to_value(e.state())?, before);
        assert!(e.capture_stamp_matches(&owner));
        assert!(Arc::ptr_eq(
            &snapshot,
            &e.stored_document()?.model_snapshot()
        ));
        assert_eq!(
            e.stored_document()?.to_cdxml()?.as_ref(),
            f::source("caption_owners").as_bytes()
        );
    }
    assert!(!directory.path("refused.mol").exists());
    assert!(!directory.path("refused.cdx").exists());
    Ok(())
}
