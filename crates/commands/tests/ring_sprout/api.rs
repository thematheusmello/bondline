//! Separate future-type/preview compile qualification; child of ring_sprout.rs.
use super::*;
use bondline_commands::{OrderHydrogenPolicy, RingPhase, RingSide, RingSprout, SproutRing};

fn typed(e: &Engine, ring: SproutRing) -> RingSprout {
    RingSprout {
        expected_revision: e.state().revision,
        page_id: 1,
        fragment_id: 1,
        atom_id: 2,
        hydrogen_policy: OrderHydrogenPolicy::RectifyNeutralCarbon,
        ring,
        ring_begin_x_pt: 168.0,
        ring_begin_y_pt: 144.0,
        ring_end_x_pt: 168.0,
        ring_end_y_pt: 96.0,
        side: RingSide::Positive,
    }
}
#[test]
fn typed_variants_are_closed_and_preview_is_pure_equivalent_without_spending_ids_or_save_authority()
-> TestResult {
    let mut e = pair()?;
    for (kind, value) in [
        (
            SproutRing::Polygon { size: 6 },
            json!({"kind":"polygon","size":6}),
        ),
        (
            SproutRing::Benzene { phase: None },
            json!({"kind":"benzene"}),
        ),
        (
            SproutRing::Benzene {
                phase: Some(RingPhase::SingleFirst),
            },
            json!({"kind":"benzene","phase":"single_first"}),
        ),
        (
            SproutRing::Cyclopentadiene {
                phase: Some(RingPhase::DoubleFirst),
            },
            json!({"kind":"cyclopentadiene","phase":"double_first"}),
        ),
        (
            SproutRing::CyclohexaneChair1 {},
            json!({"kind":"cyclohexane_chair_1"}),
        ),
        (
            SproutRing::CyclohexaneChair2 {},
            json!({"kind":"cyclohexane_chair_2"}),
        ),
    ] {
        let p = typed(&e, kind);
        let want = params(&e, value, [168.0, 144.0], [168.0, 96.0], "positive");
        assert_eq!(serde_json::to_value(&p)?, want);
        assert_eq!(
            serde_json::to_value(Command::RingSprout(p))?,
            json!({"command":"ring.sprout","params":want})
        );
    }
    let p = typed(&e, SproutRing::Polygon { size: 6 });
    let before = serde_json::to_value(e.state())?;
    let whole = e.content().clone();
    let stamp = e.capture_stamp();
    let model = e.stored_document()?.model_snapshot();
    let file = e.ui_file_context_stamp();
    let view = e.view();
    let tool = e.effective_tool();
    let preview = e.preview_ring_sprout(&p)?;
    let again = e.preview_ring_sprout(&p)?;
    assert_eq!(preview, again);
    let ps = polygon(6, [168.0, 144.0], [168.0, 96.0], "positive");
    let want = expected(
        ethane(),
        1,
        2,
        &ps,
        &[1, 2, 2, 2, 2, 2],
        &[1; 6],
        "positive",
        2,
        1,
    )?;
    graph_eq(
        &serde_json::to_value(preview.model())?["pages"][0]["fragments"][0],
        &want,
    )?;
    for variant in 0..4 {
        let mut invalid = p.clone();
        match variant {
            0 => invalid.ring_begin_x_pt = f64::NAN,
            1 => invalid.ring_end_y_pt = f64::INFINITY,
            2 => invalid.ring = SproutRing::Polygon { size: 9 },
            _ => invalid.page_id = 0,
        };
        assert_eq!(
            e.preview_ring_sprout(&invalid)
                .err()
                .ok_or("invalid typed preview accepted")?
                .info()
                .code,
            "invalid_params"
        );
    }
    assert_eq!(serde_json::to_value(e.state())?, before);
    assert_eq!(e.content(), &whole);
    assert!(e.capture_stamp_matches(&stamp));
    assert!(Arc::ptr_eq(&model, &e.stored_document()?.model_snapshot()));
    assert!(e.ui_file_context_stamp_matches(&file));
    assert_eq!(e.view(), view);
    assert_eq!(e.effective_tool(), tool);
    let prepared = e.prepare(Command::RingSprout(p))?;
    assert!(!prepared.is_deferred());
    let result = e.finish(prepared.complete()?)?;
    assert!(result.changed);
    assert!(result.file_operation.is_none() && result.ui_file_operation.is_none());
    assert_eq!(e.stored_document()?, &preview);
    assert_eq!(g(&e)?["atoms"][2]["id"], 3);
    assert_eq!(g(&e)?["bonds"][1]["id"], 2);
    Ok(())
}
