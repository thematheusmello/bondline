//! Existing public APIs only; freeze and observe runtime RED before command48 routing changes.
use bondline_commands::{Command, Engine};
use bondline_io::StoredDocument;
use bondline_model::PageSize;
use serde_json::{Value, json};
use std::{borrow::Cow, sync::Arc};
#[path = "../../../xtask/oracles/connected_label_support.rs"]
#[allow(dead_code, reason = "shared original source/file vectors")]
mod connected;
type TestResult = connected::Result;
fn engine(raw: &str) -> connected::Result<Engine> {
    Ok(Engine::from_document(StoredDocument::from_cdxml(
        raw.as_bytes().into(),
        PageSize::A4,
    )?))
}
fn refused(e: &mut Engine, name: &str, p: Value, code: &str) -> TestResult {
    let state = serde_json::to_value(e.state())?;
    let content = e.content().clone();
    let model = e.stored_document()?.model_snapshot();
    let stamp = e.capture_stamp();
    let file = e.ui_file_context_stamp();
    let error = e
        .dispatch(name, p)
        .err()
        .ok_or_else(|| format!("{name} gained unsupported authoring"))?;
    assert_eq!(error.info().code, code, "{error}");
    assert_eq!(serde_json::to_value(e.state())?, state);
    assert_eq!(e.content(), &content);
    assert!(Arc::ptr_eq(&model, &e.stored_document()?.model_snapshot()));
    assert_eq!(e.capture_stamp(), stamp);
    assert!(e.ui_file_context_stamp_matches(&file));
    Ok(())
}
#[test]
fn connected_command48_edits_numeric_body_box_and_clearance_in_one_exact_source_history_step()
-> TestResult {
    let frozen: Vec<Value> = serde_json::from_str(connected::METADATA48)?;
    assert_eq!(
        serde_json::to_value(bondline_commands::registry())?,
        json!(frozen)
    );
    let mut e = engine(connected::SOURCE)?;
    assert!(
        !e.stored_document()?.has_boxed_atom_labels(),
        "old Boxed means all-labelled/no-bond foreign source"
    );
    let before = e.content().clone();
    let value = e.state().document.clone();
    // First independent caller runtime RED: current command48 refuses this foreign connected owner.
    let changed = e.dispatch("atom.set_authored_label", connected::params(0, 7, 1))?;
    assert!(changed.changed);
    assert_eq!(
        (changed.revision, changed.undo_depth, changed.redo_depth),
        (1, 1, 0)
    );
    let owner = e.stored_document()?;
    assert!(!owner.is_fully_mapped());
    assert!(!owner.has_boxed_atom_labels());
    assert!(owner.has_display_projection());
    assert_eq!(owner.to_cdxml()?.as_ref(), connected::EDITED.as_bytes());
    assert_eq!(
        connected::read(owner.to_cdxml()?.as_ref())?,
        connected::facts(7, 1)
    );
    let after = serde_json::to_value(e.state().document.clone())?;
    let original = serde_json::to_value(value)?;
    assert_eq!(
        after["pages"][0]["fragments"][0]["bonds"],
        original["pages"][0]["fragments"][0]["bonds"]
    );
    assert_eq!(after["drawing_style"], original["drawing_style"]);
    assert_eq!(after["drawing_settings"], original["drawing_settings"]);
    let saved = StoredDocument::from_cdxml(connected::EDITED.as_bytes().into(), PageSize::A4)?;
    assert_eq!(saved.model(), owner.model());
    assert!(!saved.has_boxed_atom_labels());
    assert!(saved.has_display_projection());
    let changed_owner = e.content().clone();
    e.dispatch("history.undo", json!({}))?;
    assert_eq!(e.content(), &before);
    assert_eq!(
        e.stored_document()?.to_cdxml()?.as_ref(),
        connected::SOURCE.as_bytes()
    );
    e.dispatch("history.redo", json!({}))?;
    assert_eq!(e.content(), &changed_owner);
    assert_eq!(
        e.stored_document()?.to_cdxml()?.as_ref(),
        connected::EDITED.as_bytes()
    );
    Ok(())
}
#[test]
fn connected_numeric_noop_preserves_arcs_redo_weak_owners_and_exact_style_source_lexemes()
-> TestResult {
    let mut e = engine(connected::SOURCE)?;
    let snapshot = e.stored_document()?.model_snapshot();
    let stamp = e.capture_stamp();
    let file = e.ui_file_context_stamp();
    let stable = serde_json::to_value(e.state())?;
    let no = e.dispatch("atom.set_authored_label", connected::params(0, 8, 0))?;
    assert!(!no.changed);
    assert_eq!(serde_json::to_value(e.state())?, stable);
    assert!(Arc::ptr_eq(
        &snapshot,
        &e.stored_document()?.model_snapshot()
    ));
    assert_eq!(e.capture_stamp(), stamp);
    assert!(e.ui_file_context_stamp_matches(&file));
    assert!(matches!(e.stored_document()?.to_cdxml()?, Cow::Borrowed(_)));
    e.dispatch("atom.set_authored_label", connected::params(0, 7, 1))?;
    e.dispatch("history.undo", json!({}))?;
    assert_eq!(e.state().redo_depth, 1);
    let content = e.content().clone();
    let stamp = e.capture_stamp();
    assert!(
        !e.dispatch("atom.set_authored_label", connected::params(2, 8, 0))?
            .changed
    );
    assert_eq!(e.state().redo_depth, 1);
    assert_eq!(e.content(), &content);
    assert_eq!(e.capture_stamp(), stamp);
    e.dispatch("history.redo", json!({}))?;
    assert_eq!(
        e.stored_document()?.to_cdxml()?.as_ref(),
        connected::EDITED.as_bytes()
    );
    Ok(())
}
#[test]
fn connected_grant_never_enables_old_writers_or_unsupported_source_noops() -> TestResult {
    let mut e = engine(connected::SOURCE)?;
    for (name, p) in [
        ("atom.set_label", connected::params(0, 8, 0)),
        (
            "page.set_size",
            json!({"page_id":1,"width_pt":512,"height_pt":512}),
        ),
        (
            "bond.set_order",
            json!({"expected_revision":0,"page_id":1,"fragment_id":1,"bond_id":1,"order":"single","hydrogen_policy":"rectify_neutral_carbon"}),
        ),
        (
            "document.set_document_settings",
            json!({"expected_revision":0,"bond_length_pt":40,"chain_angle_degrees":null,"line_width_pt":1.5,"bond_spacing_percent":15}),
        ),
        (
            "text.set_run",
            json!({"expected_revision":0,"page_id":1,"text_id":1,"run_index":0,"text":"O"}),
        ),
    ] {
        refused(&mut e, name, p, "unsupported_content")?;
    }
    for source in [
        connected::SOURCE.replace(" MarginWidth='2.000'", ""),
        connected::SOURCE.replace(" LineWidth='1.50'", ""),
        connected::SOURCE.replace("<CDXML ", "<CDXML Unknown='retain' "),
        connected::SOURCE.replace("<b id='900'", "<b MarginWidth='2' id='900'"),
        connected::SOURCE.replace("color r='0' g='0' b='0'", "color r='1' g='0' b='0'"),
    ] {
        let mut opaque = engine(&source)?;
        assert!(!opaque.stored_document()?.has_display_projection());
        refused(
            &mut opaque,
            "atom.set_authored_label",
            connected::params(0, 8, 0),
            "unsupported_content",
        )?;
        assert_eq!(
            opaque.stored_document()?.to_cdxml()?.as_ref(),
            source.as_bytes()
        );
    }
    refused(
        &mut e,
        "atom.set_authored_label",
        connected::params(1, 7, 1),
        "stale_revision",
    )?;
    Ok(())
}
#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse existing isolated actual-file directory")]
mod files;
#[test]
fn captured_connected_save_does_not_acknowledge_newer_label_or_margin_source_owners() -> TestResult
{
    let dir = files::FileTestDirectory::new()?;
    let input = dir.path("connected-original-λ.cdxml");
    let output = dir.path("connected-captured-保存.cdxml");
    std::fs::write(&input, connected::SOURCE)?;
    let mut e = Engine::default();
    e.dispatch("document.open_cdxml", json!({"path":input}))?;
    let pending = e.prepare(Command::parse(
        "document.save_cdxml",
        json!({"path":output}),
    )?)?;
    e.dispatch("atom.set_authored_label", connected::params(1, 7, 1))?;
    let current = e.stored_document()?.model_snapshot();
    let completed = pending.complete()?;
    assert_eq!(std::fs::read(&output)?, connected::SOURCE.as_bytes());
    assert_eq!(
        connected::read(&std::fs::read(&output)?)?,
        connected::facts(8, 0)
    );
    let done = e.finish(completed)?;
    assert!(!done.changed && done.file.dirty && done.source.ok_or("source")?.modified);
    assert!(Arc::ptr_eq(
        &current,
        &e.stored_document()?.model_snapshot()
    ));
    assert_eq!(
        e.stored_document()?.to_cdxml()?.as_ref(),
        connected::EDITED.as_bytes()
    );
    e.dispatch("atom.set_authored_label", connected::params(2, 8, 0))?;
    assert!(!e.state().file.dirty);
    assert_eq!(std::fs::read(input)?, connected::SOURCE.as_bytes());
    Ok(())
}

#[test]
fn generated_connected_label_history_restores_every_exact_source_owner() -> TestResult {
    let mut e = engine(connected::SOURCE)?;
    let mut states = vec![e.content().clone()];
    let mut seed = 671_308_429_u64;
    for _ in 0..24 {
        seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        let (element, hydrogens, source) = if seed & (1 << 32) == 0 {
            (8, 0, connected::SOURCE)
        } else {
            (7, 1, connected::EDITED)
        };
        let outcome = e.dispatch(
            "atom.set_authored_label",
            connected::params(e.state().revision, element, hydrogens),
        )?;
        let encoded = e.stored_document()?.to_cdxml()?;
        assert_eq!(encoded.as_ref(), source.as_bytes());
        assert_eq!(
            connected::read(encoded.as_ref())?,
            connected::facts(element, hydrogens)
        );
        if outcome.changed {
            states.push(e.content().clone());
        }
    }
    assert!(
        states.len() > 8,
        "the fixed seed exercises real edits and no-ops"
    );
    for owner in states.iter().rev().skip(1) {
        e.dispatch("history.undo", json!({}))?;
        assert_eq!(e.content(), owner);
    }
    for owner in states.iter().skip(1) {
        e.dispatch("history.redo", json!({}))?;
        assert_eq!(e.content(), owner);
    }
    Ok(())
}
