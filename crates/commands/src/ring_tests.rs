//! Independent private counter/typed-input declarations; Root owns the cfg bridge.
use super::*;
use crate::{Command, Engine, OrderHydrogenPolicy};
use serde_json::json;
use std::sync::Arc;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn selected_pair() -> TestResult<Engine> {
    let mut engine = Engine::default();
    engine.dispatch(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":72,"begin_y_pt":72,
        "end_x_pt":120,"end_y_pt":72}),
    )?;
    engine.dispatch(
        "ui.selection",
        json!({"action":"set","expected_revision":1,
        "page_id":1,"atom_ids":[1,2]}),
    )?;
    Ok(engine)
}

fn request(engine: &Engine, placement: RingPlacement) -> RingCreate {
    RingCreate {
        expected_revision: engine.state().revision,
        page_id: 1,
        size: 6,
        hydrogen_policy: OrderHydrogenPolicy::RectifyNeutralCarbon,
        placement,
    }
}

fn free() -> RingPlacement {
    RingPlacement::Free {
        begin_x_pt: 120.0,
        begin_y_pt: 168.0,
        end_x_pt: 120.0,
        end_y_pt: 120.0,
        side: RingSide::Positive,
    }
}

fn atomic_refusal(engine: &mut Engine, params: RingCreate, code: &str) -> TestResult {
    let before = serde_json::to_value(engine.state())?;
    let content = engine.content().clone();
    let stamp = engine.capture_stamp();
    let model = engine.stored_document()?.model_snapshot();
    let current = Arc::clone(&engine.document);
    let saved = Arc::clone(&engine.saved);
    let ids = (
        engine.identities.atom,
        engine.identities.bond,
        engine.identities.fragment,
    );
    let tool = engine.bond_tool();
    let view = engine.view();
    let generation = engine.session_generation();
    let file = engine.ui_file_context_stamp();
    let error = engine
        .execute(Command::RingCreate(params.into()))
        .err()
        .ok_or("invalid ring accepted")?;
    assert_eq!(error.info().code, code, "{error}");
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert_eq!(engine.content(), &content);
    assert_eq!(engine.capture_stamp(), stamp);
    assert!(Arc::ptr_eq(&current, &engine.document));
    assert!(Arc::ptr_eq(&saved, &engine.saved));
    assert!(Arc::ptr_eq(
        &model,
        &engine.stored_document()?.model_snapshot()
    ));
    assert_eq!(
        (
            engine.identities.atom,
            engine.identities.bond,
            engine.identities.fragment
        ),
        ids
    );
    assert_eq!(engine.bond_tool(), tool);
    assert_eq!(engine.view(), view);
    assert_eq!(engine.session_generation(), generation);
    assert!(engine.ui_file_context_stamp_matches(&file));
    Ok(())
}

#[test]
fn typed_nonfinite_ring_coordinates_are_atomic_in_every_coordinate_field() -> TestResult {
    let mut engine = selected_pair()?;
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for field in 0..4 {
            let mut values = [300.0, 168.0, 300.0, 120.0];
            values[field] = bad;
            let params = request(
                &engine,
                RingPlacement::Free {
                    begin_x_pt: values[0],
                    begin_y_pt: values[1],
                    end_x_pt: values[2],
                    end_y_pt: values[3],
                    side: RingSide::Positive,
                },
            );
            atomic_refusal(&mut engine, params, "invalid_params")?;
        }
        for field in 0..2 {
            let mut values = [144.0, 72.0 - 24.0 * 3.0_f64.sqrt()];
            values[field] = bad;
            let params = request(
                &engine,
                RingPlacement::Atom {
                    fragment_id: 1,
                    atom_id: 2,
                    end_x_pt: values[0],
                    end_y_pt: values[1],
                    side: RingSide::Positive,
                },
            );
            atomic_refusal(&mut engine, params, "invalid_params")?;
        }
    }
    Ok(())
}

#[test]
fn exhausted_ring_counters_are_atomic_and_attachments_never_spend_a_fragment_id() -> TestResult {
    for counter in [
        "revision",
        "atom",
        "bond",
        "fragment",
        "atom-near",
        "bond-near",
        "selection",
    ] {
        let mut engine = selected_pair()?;
        let code = match counter {
            "revision" => {
                engine.revision = u64::MAX;
                "revision_exhausted"
            }
            "atom" => {
                engine.identities.atom = u32::MAX;
                "identity_exhausted"
            }
            "bond" => {
                engine.identities.bond = u32::MAX;
                "identity_exhausted"
            }
            "fragment" => {
                engine.identities.fragment = u32::MAX;
                "identity_exhausted"
            }
            "atom-near" => {
                engine.identities.atom = u32::MAX - 5;
                "identity_exhausted"
            }
            "bond-near" => {
                engine.identities.bond = u32::MAX - 5;
                "identity_exhausted"
            }
            _ => {
                engine.selection.state.epoch = u64::MAX;
                "ui_selection_epoch_exhausted"
            }
        };
        let params = request(&engine, free());
        atomic_refusal(&mut engine, params, code)?;
    }
    for placement in [
        RingPlacement::Atom {
            fragment_id: 1,
            atom_id: 2,
            end_x_pt: 144.0,
            end_y_pt: 72.0 - 24.0 * 3.0_f64.sqrt(),
            side: RingSide::Positive,
        },
        RingPlacement::Bond {
            fragment_id: 1,
            bond_id: 1,
            side: RingSide::Positive,
        },
    ] {
        let mut engine = selected_pair()?;
        engine.selection.state.epoch = u64::MAX;
        let params = request(&engine, placement);
        atomic_refusal(&mut engine, params, "ui_selection_epoch_exhausted")?;
    }
    for kind in ["atom", "bond"] {
        let mut engine = selected_pair()?;
        engine.identities.fragment = u32::MAX;
        let placement = if kind == "atom" {
            RingPlacement::Atom {
                fragment_id: 1,
                atom_id: 2,
                end_x_pt: 144.0,
                end_y_pt: 72.0 - 24.0 * 3.0_f64.sqrt(),
                side: RingSide::Positive,
            }
        } else {
            RingPlacement::Bond {
                fragment_id: 1,
                bond_id: 1,
                side: RingSide::Positive,
            }
        };
        let params = request(&engine, placement);
        let result = engine.execute(Command::RingCreate(params.into()))?;
        assert!(result.changed);
        assert_eq!(engine.identities.fragment, u32::MAX);
        let f = &engine.document()?.pages()[0].fragments()[0];
        assert_eq!(f.id().get(), 1);
        assert_eq!(
            (f.atoms().len(), f.bonds().len()),
            if kind == "atom" { (7, 7) } else { (6, 6) }
        );
        assert_eq!(engine.identities.atom, if kind == "atom" { 7 } else { 6 });
        assert_eq!(engine.identities.bond, if kind == "atom" { 7 } else { 6 });
    }
    Ok(())
}
