//! Separate public-type/request-preview compile qualification, not the JSON runtime RED.
use bondline_commands::{
    Command, EffectiveTool, Engine, NamedRingCreate, OrderHydrogenPolicy, RingCreate,
    RingCreateRequest, RingKind, RingPhase, RingPlacement, RingSide, SelectNamedRingTool,
    SelectRingTool, SelectRingToolRequest,
};
use serde_json::json;
use std::sync::Arc;
type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn typed_named_mode_and_request_preview_keep_legacy_wrapper_and_never_author_before_commit()
-> TestResult {
    let legacy = SelectRingTool { size: 6 };
    let legacy_request: SelectRingToolRequest = legacy.into();
    assert_eq!(serde_json::to_value(legacy_request)?, json!({"size":6}));
    let mut engine = Engine::default();
    engine.execute(Command::UiRingToolSelect(legacy_request))?;
    assert_eq!(engine.effective_tool(), EffectiveTool::Ring { size: 6 });
    let named = SelectNamedRingTool {
        kind: RingKind::Benzene,
    };
    let request: SelectRingToolRequest = named.into();
    assert_eq!(serde_json::to_value(request)?, json!({"kind":"benzene"}));
    engine.execute(Command::UiRingToolSelect(request))?;
    assert_eq!(
        engine.effective_tool(),
        EffectiveTool::NamedRing {
            kind: RingKind::Benzene
        }
    );
    let proof = engine.ring_context(1)?;
    let before = serde_json::to_value(engine.state())?;
    let stamp = engine.capture_stamp();
    let model = engine.stored_document()?.model_snapshot();
    let tool = engine.bond_tool();
    let mode = engine.effective_tool();
    let placement = RingPlacement::Free {
        begin_x_pt: 120.0,
        begin_y_pt: 168.0,
        end_x_pt: 120.0,
        end_y_pt: 120.0,
        side: RingSide::Positive,
    };
    let legacy = RingCreate {
        expected_revision: 0,
        page_id: 1,
        size: 6,
        hydrogen_policy: OrderHydrogenPolicy::RectifyNeutralCarbon,
        placement: placement.clone(),
    };
    let legacy_preview = engine.preview_ring_create(&legacy)?;
    let request: RingCreateRequest = legacy.clone().into();
    assert_eq!(
        engine.preview_ring_request(&request)?,
        legacy_preview,
        "the original sized preview wrapper remains exact"
    );
    let named = NamedRingCreate {
        expected_revision: 0,
        page_id: 1,
        kind: RingKind::Benzene,
        hydrogen_policy: OrderHydrogenPolicy::RectifyNeutralCarbon,
        placement,
        phase: Some(RingPhase::SingleFirst),
    };
    let request: RingCreateRequest = named.clone().into();
    let preview = engine.preview_ring_request(&request)?;
    let fragment = &preview.model().pages()[0].fragments()[0];
    assert_eq!(fragment.id().get(), 1);
    assert_eq!(
        fragment
            .atoms()
            .iter()
            .map(|a| (
                a.id.get(),
                a.element.get(),
                a.explicit_hydrogens,
                a.show_non_terminal_carbon_labels
            ))
            .collect::<Vec<_>>(),
        vec![
            (1, 6, Some(1), Some(false)),
            (2, 6, Some(1), Some(false)),
            (3, 6, Some(1), Some(false)),
            (4, 6, Some(1), Some(false)),
            (5, 6, Some(1), Some(false)),
            (6, 6, Some(1), Some(false))
        ]
    );
    assert_eq!(
        serde_json::to_value(fragment.bonds())?,
        json!([
        {"id":1,"begin":1,"end":2,"order":"single"},
        {"id":2,"begin":2,"end":3,"order":"double","double_position":"right"},
        {"id":3,"begin":3,"end":4,"order":"single"},
        {"id":4,"begin":4,"end":5,"order":"double","double_position":"right"},
        {"id":5,"begin":5,"end":6,"order":"single"},
        {"id":6,"begin":6,"end":1,"order":"double","double_position":"right"}])
    );
    let mut stale = named;
    stale.expected_revision = 9;
    let error = engine
        .preview_ring_request(&stale.into())
        .err()
        .ok_or("stale preview accepted")?;
    assert_eq!(error.info().code, "stale_revision");
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert!(engine.capture_stamp_matches(&stamp));
    assert!(Arc::ptr_eq(
        &model,
        &engine.stored_document()?.model_snapshot()
    ));
    assert_eq!(engine.bond_tool(), tool);
    assert_eq!(engine.effective_tool(), mode);
    assert!(proof.is_current(&engine));
    engine.execute(Command::RingCreate(request))?;
    assert_eq!(engine.document()?, preview.model());
    assert_eq!(
        engine.document()?.pages()[0].fragments()[0].atoms()[0]
            .id
            .get(),
        1,
        "all previews/errors must leave the first actual ID unspent"
    );
    assert!(!proof.is_current(&engine));
    Ok(())
}
