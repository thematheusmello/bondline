//! New public type compile qualification; do not count a missing-type compile failure as JSON RED.
use bondline_commands::{
    NamedRingCreate, OrderHydrogenPolicy, RingCreate, RingCreateRequest, RingKind, RingPhase,
    RingPlacement, RingSide,
};
use serde_json::json;

type TestResult = Result<(), Box<dyn std::error::Error>>;
fn placement() -> RingPlacement {
    RingPlacement::Free {
        begin_x_pt: 120.0,
        begin_y_pt: 168.0,
        end_x_pt: 120.0,
        end_y_pt: 120.0,
        side: RingSide::Positive,
    }
}
#[test]
fn legacy_conversion_is_byte_exact_and_named_defaults_getters_and_patterns_are_literal()
-> TestResult {
    let legacy = RingCreate {
        expected_revision: 7,
        page_id: 1,
        size: 6,
        hydrogen_policy: OrderHydrogenPolicy::RectifyNeutralCarbon,
        placement: placement(),
    };
    let expected = r#"{"expected_revision":7,"page_id":1,"size":6,"hydrogen_policy":"rectify_neutral_carbon","placement":{"mode":"free","begin_x_pt":120.0,"begin_y_pt":168.0,"end_x_pt":120.0,"end_y_pt":120.0,"side":"positive"}}"#;
    assert_eq!(serde_json::to_string(&legacy)?, expected);
    let request: RingCreateRequest = legacy.into();
    assert!(matches!(&request, RingCreateRequest::Sized(_)));
    assert_eq!(request.expected_revision(), 7);
    assert_eq!(serde_json::to_string(&request)?, expected);
    for (kind, size, weights, name) in [
        (RingKind::Benzene, 6, vec![2_u16, 1, 2, 1, 2, 1], "benzene"),
        (
            RingKind::Cyclopentadiene,
            5,
            vec![2_u16, 1, 2, 1, 1],
            "cyclopentadiene",
        ),
    ] {
        assert_eq!(kind.size(), size);
        assert_eq!(kind.base_weights(), weights.as_slice());
        let named = NamedRingCreate {
            expected_revision: 42,
            page_id: 1,
            kind,
            hydrogen_policy: OrderHydrogenPolicy::RectifyNeutralCarbon,
            placement: placement(),
            phase: None,
        };
        let omitted = serde_json::to_value(&named)?;
        assert_eq!(omitted["kind"], name);
        assert!(omitted.get("size").is_none() && omitted.get("phase").is_none());
        let request: RingCreateRequest = named.into();
        assert_eq!(request.expected_revision(), 42);
        assert_eq!(serde_json::to_value(&request)?, omitted);
        let read: NamedRingCreate = serde_json::from_value(omitted.clone())?;
        assert!(read.phase.is_none());
        assert_eq!(read.kind, kind);
        for (phase, text) in [
            (RingPhase::DoubleFirst, "double_first"),
            (RingPhase::SingleFirst, "single_first"),
        ] {
            let mut value = omitted.clone();
            value["phase"] = json!(text);
            let parsed: NamedRingCreate = serde_json::from_value(value.clone())?;
            assert_eq!(parsed.phase, Some(phase));
            assert_eq!(
                serde_json::to_value(RingCreateRequest::from(parsed))?,
                value
            );
        }
        let mut null = omitted;
        null["phase"] = json!(null);
        assert!(serde_json::from_value::<NamedRingCreate>(null).is_err());
    }
    Ok(())
}
