//! The opt-in table and old responses share one guarded document and selection.
use bondline_commands::Engine;
use serde_json::{Value, json};
type Test<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn read(e: &mut Engine) -> Test<Value> {
    let before = serde_json::to_value(e.state())?;
    let stamp = e.capture_stamp();
    let req = serde_json::to_value(e.analysis_request())?;
    let old = serde_json::to_value(e.dispatch("analysis.properties", req.clone())?)?;
    let mut off = req.clone();
    off["include_elemental_analysis"] = json!(false);
    assert_eq!(
        serde_json::to_value(e.dispatch("analysis.properties", off)?)?,
        old
    );
    let mut on = req;
    on["include_elemental_analysis"] = json!(true);
    let mut result = serde_json::to_value(e.dispatch("analysis.properties", on.clone())?)?;
    let rows = result["analysis"]
        .as_object_mut()
        .ok_or("report")?
        .remove("elemental_analysis")
        .unwrap_or(Value::Null);
    assert_eq!(result, old);
    assert!(e.dispatch("analysis.formula", on).is_err());
    assert_eq!(serde_json::to_value(e.state())?, before);
    assert_eq!(e.capture_stamp(), stamp);
    Ok(rows)
}

fn row(report: &Value, index: usize, symbol: &str, weight: f64, percent: f64) -> Test {
    let r = &report["elements"][index];
    assert_eq!(r["symbol"], symbol);
    assert!((r["weight_contribution"].as_f64().ok_or("weight")? - weight).abs() < 1e-10);
    assert!((r["percent_by_weight"].as_f64().ok_or("percent")? - percent).abs() < 1e-9);
    Ok(())
}

#[test]
fn whole_partial_undo_stale_and_missing_weights_never_show_an_incomplete_denominator() -> Test {
    let mut e = Engine::default();
    assert!(read(&mut e)?.is_null());
    e.dispatch(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":72,"begin_y_pt":72,"end_x_pt":120,"end_y_pt":72}),
    )?;
    e.dispatch("atom.set_label", json!({"expected_revision":1,"page_id":1,"fragment_id":1,"atom_id":2,"element":8,"explicit_hydrogens":1}))?;
    let whole = read(&mut e)?;
    row(&whole, 0, "H", 4.032, 12.583484177018)?;
    row(&whole, 1, "C", 12.011, 37.485175706885)?;
    row(&whole, 2, "O", 15.999, 49.931340116098)?;
    let mut stale = serde_json::to_value(e.analysis_request())?;
    stale["include_elemental_analysis"] = json!(true);
    e.dispatch(
        "ui.selection",
        json!({"action":"set","expected_revision":2,"page_id":1,"atom_ids":[2]}),
    )?;
    let selected = read(&mut e)?;
    assert_eq!(selected["elements"].as_array().ok_or("elements")?.len(), 2);
    row(&selected, 0, "H", 1.008, 5.926971247134)?;
    row(&selected, 1, "O", 15.999, 94.073028752866)?;
    assert!(matches!(
        e.dispatch("analysis.properties", stale),
        Err(bondline_commands::Error::StaleSelection)
    ));
    e.dispatch(
        "ui.selection",
        json!({"action":"clear","expected_revision":2}),
    )?;
    e.dispatch("atom.set_label", json!({"expected_revision":2,"page_id":1,"fragment_id":1,"atom_id":2,"element":43,"explicit_hydrogens":0}))?;
    assert_eq!(
        read(&mut e)?,
        json!({"reason":"no_standard_atomic_weight","atomic_number":43})
    );
    e.dispatch("history.undo", json!({}))?;
    assert_eq!(read(&mut e)?, whole);
    Ok(())
}
