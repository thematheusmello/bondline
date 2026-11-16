//! Independent regular-polygon precision controls. No renderer/minimum-length policy.
use bondline_commands::Engine;
use bondline_io::StoredDocument;
use bondline_model::PageSize;
use serde_json::json;
use std::sync::Arc;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const NATIVE: &[u8] = include_bytes!("../../io/tests/fixtures/bondline-v0.cdxml");
const RELATIVE_TOL: f64 = 1e-12;
const ANGLE_TOL: f64 = 1e-10;

fn retained_source_with_redo_and_selection() -> TestResult<(Engine, bondline_model::Document)> {
    let stored = StoredDocument::from_cdxml(Arc::<[u8]>::from(NATIVE), PageSize::A4)?;
    assert!(stored.is_fully_mapped());
    let mut engine = Engine::from_document(stored);
    // Spend IDs through a real public edit; Undo must retain those session floors.
    engine.dispatch(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":300,"begin_y_pt":400,"end_x_pt":348,"end_y_pt":400}),
    )?;
    let redo_document = engine.document()?.clone();
    engine.dispatch("history.undo", json!({}))?;
    engine.dispatch("ui.selection",json!({"action":"set","expected_revision":engine.state().revision,"page_id":1,"atom_ids":[1,2]}))?;
    assert_eq!(engine.state().redo_depth, 1);
    assert_eq!(engine.state().undo_depth, 0);
    assert!(!engine.state().file.dirty);
    assert_eq!(engine.stored_document()?.export_cdxml()?, NATIVE);
    Ok((engine, redo_document))
}
fn precision_refusal(a: [f64; 2], b: [f64; 2]) -> TestResult {
    let (mut engine, redo_document) = retained_source_with_redo_and_selection()?;
    let before = serde_json::to_value(engine.state())?;
    let content = engine.content().clone();
    let stamp = engine.capture_stamp();
    let model = engine.stored_document()?.model_snapshot();
    let selection = engine.selection_context()?;
    let file = engine.ui_file_context_stamp();
    let view = engine.view();
    let generation = engine.session_generation();
    let error = engine
        .dispatch(
            "ring.create",
            json!({"expected_revision":engine.state().revision,"page_id":1,"size":3,
        "hydrogen_policy":"rectify_neutral_carbon","placement":{"mode":"free",
        "begin_x_pt":a[0],"begin_y_pt":a[1],"end_x_pt":b[0],"end_y_pt":b[1],"side":"positive"}}),
        )
        .err()
        .ok_or("a rounded nonregular polygon was accepted")?;
    assert_eq!(error.info().code, "invalid_params");
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert_eq!(engine.content(), &content);
    assert_eq!(engine.capture_stamp(), stamp);
    assert!(Arc::ptr_eq(
        &model,
        &engine.stored_document()?.model_snapshot()
    ));
    assert!(selection.is_current(&engine));
    assert!(engine.ui_file_context_stamp_matches(&file));
    assert_eq!(engine.view(), view);
    assert_eq!(engine.session_generation(), generation);
    assert_eq!(engine.stored_document()?.export_cdxml()?, NATIVE);
    // Public history behavior checks that the saved baseline and redo owner survived.
    engine.dispatch("history.redo", json!({}))?;
    assert_eq!(engine.document()?, &redo_document);
    assert!(engine.state().file.dirty);
    engine.dispatch("history.undo", json!({}))?;
    assert!(!engine.state().file.dirty);
    assert_eq!(engine.stored_document()?.export_cdxml()?, NATIVE);
    // Public allocation proves the rejected candidate did not spend private identity floors.
    engine.dispatch(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":400,"begin_y_pt":500,"end_x_pt":448,"end_y_pt":500}),
    )?;
    let fragments = engine.document()?.pages()[0].fragments();
    assert_eq!(fragments.len(), 2);
    assert_eq!(
        serde_json::to_value(fragments[1].as_ref())?,
        json!({"id":3,"atoms":[
        {"id":5,"element":6,"position":{"x_pt":400.0,"y_pt":500.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":3},
        {"id":6,"element":6,"position":{"x_pt":448.0,"y_pt":500.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":3}],
        "bonds":[{"id":3,"begin":5,"end":6,"order":"single"}]})
    );
    Ok(())
}

#[test]
fn one_ulp_triangle_at_unit_origin_refuses_distinct_but_nonregular_rounded_geometry_atomically()
-> TestResult {
    let a = [1.0, 1.0];
    let b = [1.0 + f64::EPSILON, 1.0];
    assert!(b[0] > a[0]);
    assert_eq!(b[0] - a[0], f64::EPSILON);
    // The draft's repeated-point-only check can keep three distinct right-triangle corners.
    // This is the independently declared hypothesis; Root must observe its actual result.
    precision_refusal(a, b)
}

#[test]
fn a_tiny_origin_triangle_with_well_conditioned_relative_geometry_is_not_rejected_by_length()
-> TestResult {
    let length = 1e-200_f64;
    assert!(length.is_normal());
    assert_eq!(
        length * length,
        0.0,
        "the control must defeat a naive squared-length check"
    );
    let mut engine = Engine::default();
    let before = engine.state();
    let result = engine.dispatch(
        "ring.create",
        json!({"expected_revision":before.revision,"page_id":1,"size":3,
        "hydrogen_policy":"rectify_neutral_carbon","placement":{"mode":"free",
        "begin_x_pt":0,"begin_y_pt":0,"end_x_pt":length,"end_y_pt":0,"side":"positive"}}),
    )?;
    assert!(result.changed);
    assert_eq!(result.revision, before.revision + 1);
    assert_eq!(result.undo_depth, before.undo_depth + 1);
    assert_eq!(result.redo_depth, 0);
    assert!(result.source.is_none());
    assert!(result.file.dirty && result.file.path.is_none());
    let fragments = engine.document()?.pages()[0].fragments();
    assert_eq!(fragments.len(), 1);
    let fragment = &fragments[0];
    assert_eq!(fragment.id().get(), 1);
    assert_eq!(fragment.atoms().len(), 3);
    assert_eq!(fragment.bonds().len(), 3);
    let points = fragment
        .atoms()
        .iter()
        .map(|a| {
            let p = a.position.ok_or("position")?;
            Ok([p.x_pt(), p.y_pt()])
        })
        .collect::<TestResult<Vec<_>>>()?;
    assert_eq!(points[0], [0.0, 0.0]);
    assert_eq!(points[1], [length, 0.0]);
    assert!((points[2][0] / length - 0.5).abs() <= RELATIVE_TOL);
    assert!((points[2][1] / length - 3.0_f64.sqrt() / 2.0).abs() <= RELATIVE_TOL);
    for i in 0..3 {
        assert_eq!(fragment.atoms()[i].id.get(), u32::try_from(i + 1)?);
        assert_eq!(fragment.atoms()[i].element.get(), 6);
        assert_eq!(fragment.atoms()[i].explicit_hydrogens, Some(2));
        assert_eq!(fragment.atoms()[i].isotope, None);
        assert_eq!(fragment.atoms()[i].charge, 0);
        assert_eq!(fragment.atoms()[i].radical, bondline_model::Radical::None);
        let a = points[(i + 2) % 3];
        let b = points[i];
        let c = points[(i + 1) % 3];
        // Divide before products: no tiny cross/dot/squared values can underflow here.
        let u = [(a[0] - b[0]) / length, (a[1] - b[1]) / length];
        let v = [(c[0] - b[0]) / length, (c[1] - b[1]) / length];
        let ul = u[0].hypot(u[1]);
        let vl = v[0].hypot(v[1]);
        assert!((ul - 1.0).abs() <= RELATIVE_TOL);
        assert!((vl - 1.0).abs() <= RELATIVE_TOL);
        let u = [u[0] / ul, u[1] / ul];
        let v = [v[0] / vl, v[1] / vl];
        let angle = (u[0] * v[1] - u[1] * v[0])
            .abs()
            .atan2(u[0] * v[0] + u[1] * v[1]);
        assert!((angle - std::f64::consts::PI / 3.0).abs() <= ANGLE_TOL);
        assert_eq!(
            serde_json::to_value(fragment.bonds()[i])?,
            json!({"id":i+1,"begin":i+1,"end":if i==2 {1}else{i+2},"order":"single"})
        );
    }
    // Core authoring fidelity only: microscopic glyph/ink/raster readability is not claimed.
    Ok(())
}

#[test]
fn minimum_subnormal_triangle_refuses_quantized_nonregular_geometry_separately() -> TestResult {
    let length = f64::from_bits(1);
    assert!(length > 0.0 && length.is_subnormal());
    precision_refusal([0.0, 0.0], [length, 0.0])
}
