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
        assert_eq!(preview.export_cdx()?.as_ref(), *moved);
        assert_eq!(e.stored_document()?.export_cdx()?.as_ref(), *raw);
        assert!(e.capture_stamp_matches(&stamp));
        e.dispatch("selection.translate", serde_json::to_value(&params)?)?;
        assert_eq!(e.stored_document()?.export_cdx()?.as_ref(), *moved);
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
        assert_eq!(e.stored_document()?.export_cdx()?.as_ref(), *raw);
        e.dispatch("history.redo", json!({}))?;
        assert_eq!(e.stored_document()?.export_cdx()?.as_ref(), *moved);
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
        assert_eq!(e.stored_document()?.export_cdx()?.as_ref(), raw);
    }
    Ok(())
}
