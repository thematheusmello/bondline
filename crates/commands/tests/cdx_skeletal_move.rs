//! Test the real selection registry against independently predicted CDX bytes.
use bondline_commands::Engine;
use bondline_io::StoredDocument;
use bondline_model::PageSize;
use serde_json::json;
use std::sync::Arc;
type Result = std::result::Result<(), Box<dyn std::error::Error>>;

fn engine(raw: &[u8]) -> std::result::Result<Engine, Box<dyn std::error::Error>> {
    Ok(Engine::from_document(StoredDocument::from_cdx(
        Arc::from(raw),
        PageSize::A4,
    )?))
}
fn select(e: &mut Engine, ids: &[u32]) -> std::result::Result<u64, Box<dyn std::error::Error>> {
    e.dispatch(
        "ui.selection",
        json!({"action":"set","expected_revision":e.state().revision,"page_id":1,"atom_ids":ids}),
    )?;
    Ok(e.selection().epoch)
}

#[test]
fn skeletal_move_uses_only_the_owned_positions_and_preserves_every_header() -> Result {
    let inputs = [
        (
            include_bytes!("../../io/tests/fixtures/cdx-skeletal/propane.cdx").as_slice(),
            include_bytes!("../../io/tests/fixtures/cdx-skeletal-move/propane_center.cdx")
                .as_slice(),
        ),
        (
            include_bytes!("../../io/tests/fixtures/cdx-skeletal/all_h_missing.cdx").as_slice(),
            include_bytes!("../../io/tests/fixtures/cdx-skeletal-move/all_h_missing_center.cdx")
                .as_slice(),
        ),
        (
            include_bytes!("../../io/tests/fixtures/cdx-skeletal/missing_h.cdx").as_slice(),
            include_bytes!("../../io/tests/fixtures/cdx-skeletal-move/missing_h_center.cdx")
                .as_slice(),
        ),
        (
            include_bytes!("../../io/tests/fixtures/cdx-skeletal/dormant_true.cdx").as_slice(),
            include_bytes!("../../io/tests/fixtures/cdx-skeletal-move/dormant_true_center.cdx")
                .as_slice(),
        ),
        (
            include_bytes!("../../io/tests/fixtures/cdx-skeletal/dormant_nonterminal.cdx")
                .as_slice(),
            include_bytes!(
                "../../io/tests/fixtures/cdx-skeletal-move/dormant_nonterminal_center.cdx"
            )
            .as_slice(),
        ),
        (
            include_bytes!("../../io/tests/fixtures/cdx-skeletal-move/aliased_extended.cdx")
                .as_slice(),
            include_bytes!("../../io/tests/fixtures/cdx-skeletal-move/aliased_moved.cdx")
                .as_slice(),
        ),
    ];
    for (index, (raw, moved)) in inputs.iter().enumerate() {
        let mut e = engine(raw)?;
        let (ids, dx, dy) = if index == 5 {
            (&[4, 2][..], -5.125, 3.25)
        } else {
            (&[2][..], 8.25, -5.125)
        };
        let stamp = e.capture_stamp();
        let epoch = select(&mut e, ids)?;
        let params = bondline_commands::SelectionTranslate {
            expected_revision: e.state().revision,
            expected_selection_epoch: epoch,
            delta_x_pt: dx,
            delta_y_pt: dy,
        };
        let preview = e.preview_selection_translation(&params)?;
        assert_eq!(preview.export_cdx()?, *moved);
        assert_eq!(e.stored_document()?.export_cdx()?, *raw);
        assert!(e.capture_stamp_matches(&stamp));
        e.dispatch("selection.translate", serde_json::to_value(&params)?)?;
        assert_eq!(e.stored_document()?.export_cdx()?, *moved);
        assert_eq!(e.state().undo_depth, 1);
        e.dispatch("history.undo", json!({}))?;
        let epoch = select(&mut e, ids)?;
        let model = e.stored_document()?.model_snapshot();
        let stamp = e.capture_stamp();
        let revision = e.state().revision;
        for delta in [0.0, 0.5 / 65536., -0.5 / 65536.] {
            let result=e.dispatch("selection.translate",json!({"expected_revision":revision,"expected_selection_epoch":epoch,"delta_x_pt":delta,"delta_y_pt":-delta}))?;
            assert!(!result.changed);
            assert_eq!(result.redo_depth, 1);
            assert!(e.capture_stamp_matches(&stamp));
            assert!(Arc::ptr_eq(&model, &e.stored_document()?.model_snapshot()));
        }
        assert_eq!(e.stored_document()?.export_cdx()?, *raw);
        e.dispatch("history.redo", json!({}))?;
        assert_eq!(e.stored_document()?.export_cdx()?, *moved);
    }
    Ok(())
}

#[test]
fn skeletal_move_rejects_out_of_paper_overflow_and_new_mandatory_carbon_ink() -> Result {
    let raw = include_bytes!("../../io/tests/fixtures/cdx-skeletal/all_h_missing.cdx");
    let mut e = engine(raw)?;
    let epoch = select(&mut e, &[2])?;
    let stamp = e.capture_stamp();
    let model = e.stored_document()?.model_snapshot();
    for (dx, dy) in [
        (0., 48.),
        (0., 500.),
        (0., 65536.),
        (1e300, 0.),
        (-300., -200.),
    ] {
        let q = json!({"expected_revision":e.state().revision,"expected_selection_epoch":epoch,"delta_x_pt":dx,"delta_y_pt":dy});
        assert!(e.dispatch("selection.translate", q).is_err(), "{dx},{dy}");
        assert!(e.capture_stamp_matches(&stamp));
        assert!(Arc::ptr_eq(&model, &e.stored_document()?.model_snapshot()));
        assert_eq!(e.state().undo_depth, 0);
        assert_eq!(e.stored_document()?.export_cdx()?, raw);
    }
    Ok(())
}

#[test]
fn mixed_move_certifies_the_preview_then_commits_one_history_entry() -> Result {
    let raw = include_bytes!("../../io/tests/fixtures/cdx-mixed-skeleton/amino_alcohol.cdx");
    let end = include_bytes!("../../io/tests/fixtures/cdx-mixed-skeletal-move/combined.cdx");
    let mut e = engine(raw)?;
    let epoch = select(&mut e, &[5, 1, 3])?;
    let p = bondline_commands::SelectionTranslate {
        expected_revision: e.state().revision,
        expected_selection_epoch: epoch,
        delta_x_pt: 8.25,
        delta_y_pt: -5.125,
    };
    let q = serde_json::to_value(&p)?;
    let preview = e.preview_selection_translation(&p)?;
    assert_eq!(preview.export_cdx()?, end);
    assert_eq!(e.stored_document()?.export_cdx()?, raw);
    e.dispatch("selection.translate", q.clone())?;
    assert_eq!(e.stored_document()?.export_cdx()?, end);
    assert_eq!(e.state().undo_depth, 1);
    assert!(e.dispatch("selection.translate", q).is_err());
    e.dispatch("history.undo", json!({}))?;
    let epoch = select(&mut e, &[3, 1, 5])?;
    let stamp = e.capture_stamp();
    for delta in [0.0, 0.5 / 65536., -0.5 / 65536., 1024.] {
        let r=e.dispatch("selection.translate",json!({"expected_revision":e.state().revision,"expected_selection_epoch":epoch,"delta_x_pt":delta,"delta_y_pt":-delta}));
        if delta == 1024. {
            assert!(r.is_err())
        } else {
            assert!(!r?.changed);
        }
        assert_eq!(e.state().redo_depth, 1);
        assert!(e.capture_stamp_matches(&stamp));
        assert_eq!(e.stored_document()?.export_cdx()?, raw);
    }
    e.dispatch("history.redo", json!({}))?;
    assert_eq!(e.stored_document()?.export_cdx()?, end);
    Ok(())
}

#[test]
fn editing_a_printed_atom_preserves_hidden_neighbors_and_rejected_edits_preserve_redo() -> Result {
    let input = include_bytes!("../../io/tests/fixtures/cdx-mixed-skeletal-move/twins.cdx");
    let expected = include_bytes!("../../io/tests/fixtures/cdx-skeletal-label-edit/twins.cdx");
    let mut engine = engine(input)?;
    let change = |revision, fragment, atom, element, h| {
        json!({
            "expected_revision":revision,"page_id":1,"fragment_id":fragment,"atom_id":atom,
            "element":element,"explicit_hydrogens":h
        })
    };
    let start = engine.state().revision;
    engine.dispatch("atom.set_authored_label", change(start, 2, 4, 7, 2))?;
    assert_eq!(engine.stored_document()?.export_cdx()?, expected);
    engine.dispatch("history.undo", json!({}))?;
    let revision = engine.state().revision;
    let stamp = engine.capture_stamp();
    for invalid in [
        change(start, 2, 4, 7, 2),    // stale edit
        change(revision, 1, 4, 7, 2), // other fragment
        change(revision, 2, 3, 7, 2), // hidden carbon has no owned Text
        change(revision, 2, 4, 6, 3), // printed carbon would invalidate the drawing profile
    ] {
        assert!(engine.dispatch("atom.set_authored_label", invalid).is_err());
    }
    assert!(
        !engine
            .dispatch("atom.set_authored_label", change(revision, 2, 4, 8, 1))?
            .changed
    );
    assert!(engine.capture_stamp_matches(&stamp));
    assert_eq!(engine.state().redo_depth, 1);
    assert_eq!(engine.stored_document()?.export_cdx()?, input);
    engine.dispatch("history.redo", json!({}))?;
    assert_eq!(engine.stored_document()?.export_cdx()?, expected);
    Ok(())
}
