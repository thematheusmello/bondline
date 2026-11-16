//! Public new point/request API and pure previews; separate from the first JSON runtime.
use bondline_commands::{
    ChainCreate, ChainCreatePath, ChainPathPlacement, ChainPathPoint, ChainPlacement, ChainSide,
    Command, Engine, OrderHydrogenPolicy,
};
use bondline_io::StoredDocument;
use bondline_model::{PageSize, Point2};
use serde_json::{Value, json};
use std::sync::Arc;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const NATIVE: &[u8] = include_bytes!("../../io/tests/fixtures/bondline-v0.cdxml");

fn path(revision: u64) -> ChainCreatePath {
    ChainCreatePath {
        expected_revision: revision,
        page_id: 1,
        hydrogen_policy: OrderHydrogenPolicy::RectifyNeutralCarbon,
        placement: ChainPathPlacement::Free {
            points: vec![
                ChainPathPoint {
                    x_pt: 64.0,
                    y_pt: 112.0,
                },
                ChainPathPoint {
                    x_pt: 112.0,
                    y_pt: 112.0,
                },
                ChainPathPoint {
                    x_pt: 112.0,
                    y_pt: 160.0,
                },
            ],
        },
    }
}
fn fixed(revision: u64) -> ChainCreate {
    ChainCreate {
        expected_revision: revision,
        page_id: 1,
        count: 3,
        angle_degrees: 90.0,
        hydrogen_policy: OrderHydrogenPolicy::RectifyNeutralCarbon,
        placement: ChainPlacement::Free {
            begin_x_pt: 400.0,
            begin_y_pt: 400.0,
            end_x_pt: 448.0,
            end_y_pt: 400.0,
            side: ChainSide::Positive,
        },
    }
}

#[test]
fn public_path_points_are_strict_objects_and_new_request_does_not_change_fixed_wire() -> TestResult
{
    let request = path(0);
    let value = serde_json::to_value(&request)?;
    assert_eq!(
        value,
        json!({"expected_revision":0,"page_id":1,"hydrogen_policy":"rectify_neutral_carbon",
        "placement":{"mode":"free","points":[{"x_pt":64.0,"y_pt":112.0},
        {"x_pt":112.0,"y_pt":112.0},{"x_pt":112.0,"y_pt":160.0}]}})
    );
    assert_eq!(
        serde_json::to_value(serde_json::from_value::<ChainCreatePath>(value.clone())?)?,
        value
    );
    let attached = ChainPathPlacement::Atom {
        fragment_id: 17,
        atom_id: 29,
        new_points: vec![ChainPathPoint {
            x_pt: 192.0,
            y_pt: 128.0,
        }],
    };
    assert_eq!(
        serde_json::to_value(attached)?,
        json!({"mode":"atom","fragment_id":17,"atom_id":29,
        "new_points":[{"x_pt":192.0,"y_pt":128.0}]})
    );
    for p in [
        json!([1, 2]),
        Value::Null,
        json!({"x_pt":1}),
        json!({"x_pt":1,"y_pt":2,"z_pt":3}),
        json!({"x_pt":"1","y_pt":2}),
    ] {
        assert!(serde_json::from_value::<ChainPathPoint>(p).is_err());
    }
    let mut array = value;
    array["placement"]["points"][0] = json!([64, 112]);
    assert!(serde_json::from_value::<ChainCreatePath>(array).is_err());
    assert_eq!(
        serde_json::to_value(fixed(0))?,
        json!({"expected_revision":0,"page_id":1,"count":3,
        "angle_degrees":90.0,"hydrogen_policy":"rectify_neutral_carbon","placement":{"mode":"free",
        "begin_x_pt":400.0,"begin_y_pt":400.0,"end_x_pt":448.0,"end_y_pt":400.0,"side":"positive"}})
    );
    // Existing Point2 map serialization is unchanged; no new schema dependency/ABI is required.
    assert_eq!(
        serde_json::to_value(Point2::new(1.0, 2.0)?)?,
        json!({"x_pt":1.0,"y_pt":2.0})
    );
    Ok(())
}

#[test]
fn typed_nonfinite_and_out_of_model_bounds_validate_before_any_publication() -> TestResult {
    let mut engine = Engine::default();
    let before = serde_json::to_value(engine.state())?;
    let stamp = engine.capture_stamp();
    let owner = engine.stored_document()?.model_snapshot();
    for bad in [
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        100001.0,
        -100001.0,
    ] {
        for axis in [0, 1] {
            let mut request = path(0);
            let ChainPathPlacement::Free { points } = &mut request.placement else {
                return Err("free fixture".into());
            };
            if axis == 0 {
                points[1].x_pt = bad;
            } else {
                points[1].y_pt = bad;
            }
            let error = engine
                .execute(Command::ChainCreatePath(request))
                .err()
                .ok_or("invalid typed point accepted")?;
            assert_eq!(error.info().code, "invalid_params");
            assert_eq!(serde_json::to_value(engine.state())?, before);
            assert!(engine.capture_stamp_matches(&stamp));
            assert!(Arc::ptr_eq(
                &owner,
                &engine.stored_document()?.model_snapshot()
            ));
        }
    }
    Ok(())
}

#[test]
fn both_previews_preserve_complete_context_and_floors_then_equal_the_real_commit() -> TestResult {
    for kind in ["fixed", "path"] {
        let mut engine =
            Engine::from_document(StoredDocument::from_cdxml(NATIVE.into(), PageSize::A4)?);
        engine.dispatch(
            "bond.create",
            json!({"page_id":1,"begin_x_pt":300,"begin_y_pt":400,"end_x_pt":348,"end_y_pt":400}),
        )?;
        engine.dispatch("history.undo", json!({}))?;
        engine.dispatch(
            "ui.selection",
            json!({"action":"set","expected_revision":2,"page_id":1,"atom_ids":[1]}),
        )?;
        engine.dispatch("ui.view", json!({"action":"set_zoom","percent":175.5}))?;
        let mut request = path(2);
        request.placement = ChainPathPlacement::Atom {
            fragment_id: 1,
            atom_id: 1,
            new_points: vec![
                ChainPathPoint {
                    x_pt: 100.0,
                    y_pt: 152.0,
                },
                ChainPathPoint {
                    x_pt: 124.0,
                    y_pt: 152.0,
                },
            ],
        };
        let fixed = fixed(2);
        let before = serde_json::to_value(engine.state())?;
        let whole = engine.content().clone();
        let stamp = engine.capture_stamp();
        let owner = engine.stored_document()?.model_snapshot();
        let file = engine.ui_file_context_stamp();
        let tool = engine.bond_tool();
        let view = engine.view();
        let generation = engine.session_generation();
        let candidate = if kind == "fixed" {
            engine.preview_chain_create(&fixed)?
        } else {
            engine.preview_chain_path(&request)?
        };
        let again = if kind == "fixed" {
            engine.preview_chain_create(&fixed)?
        } else {
            engine.preview_chain_path(&request)?
        };
        assert_eq!(again, candidate);
        assert_eq!(serde_json::to_value(engine.state())?, before);
        assert_eq!(engine.content(), &whole);
        assert!(engine.capture_stamp_matches(&stamp));
        assert!(Arc::ptr_eq(
            &owner,
            &engine.stored_document()?.model_snapshot()
        ));
        assert!(engine.ui_file_context_stamp_matches(&file));
        assert_eq!(engine.bond_tool(), tool);
        assert_eq!(engine.view(), view);
        assert_eq!(engine.session_generation(), generation);
        assert_eq!(engine.stored_document()?.export_cdxml()?, NATIVE);
        if kind == "fixed" {
            let added = &candidate.model().pages()[0].fragments()[1];
            assert_eq!(added.id().get(), 3);
            assert_eq!(
                added.atoms().iter().map(|a| a.id.get()).collect::<Vec<_>>(),
                [5, 6, 7]
            );
        } else {
            assert_eq!(
                candidate.model().pages()[0].fragments()[0]
                    .atoms()
                    .iter()
                    .map(|a| a.id.get())
                    .collect::<Vec<_>>(),
                [1, 2, 5, 6]
            );
        }
        let result = if kind == "fixed" {
            engine.dispatch("chain.create", serde_json::to_value(&fixed)?)?
        } else {
            engine.dispatch("chain.create_path", serde_json::to_value(&request)?)?
        };
        assert_eq!(
            (
                result.changed,
                result.revision,
                result.undo_depth,
                result.redo_depth
            ),
            (true, 3, 1, 0)
        );
        assert_eq!(engine.stored_document()?, &candidate);
        assert_eq!(engine.view(), view);
        assert!(engine.selection().atom_ids.is_empty());
        let stable = serde_json::to_value(engine.state())?;
        let error = if kind == "fixed" {
            engine.preview_chain_create(&fixed).err()
        } else {
            engine.preview_chain_path(&request).err()
        }
        .ok_or("stale preview accepted")?;
        assert_eq!(error.info().code, "stale_revision");
        assert_eq!(serde_json::to_value(engine.state())?, stable);
    }
    let mut closed = Engine::default();
    closed.dispatch(
        "document.close",
        json!({"expected_revision":0,"action":"request"}),
    )?;
    assert!(closed.complete_close());
    let before = serde_json::to_value(closed.state())?;
    assert_eq!(
        closed
            .preview_chain_create(&fixed(0))
            .err()
            .ok_or("closed fixed preview")?
            .info()
            .code,
        "close_committed"
    );
    assert_eq!(
        closed
            .preview_chain_path(&path(0))
            .err()
            .ok_or("closed path preview")?
            .info()
            .code,
        "close_committed"
    );
    assert_eq!(serde_json::to_value(closed.state())?, before);
    Ok(())
}
