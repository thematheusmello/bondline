use bondline_commands::Engine;
use serde_json::json;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn initial(order: &str) -> Result<Engine, Box<dyn std::error::Error>> {
    let mut engine = Engine::default();
    engine.dispatch("bond.create",json!({"page_id":1,"begin_x_pt":72.0,"begin_y_pt":144.0,"end_x_pt":120.0,"end_y_pt":144.0,"order":order}))?;
    Ok(engine)
}

fn extend(order: &str) -> serde_json::Value {
    json!({"expected_revision":1,"page_id":1,"fragment_id":1,"begin_atom_id":2,"end_x_pt":168.0,"end_y_pt":144.0,"hydrogen_policy":"rectify_terminal_carbon","order":order})
}

#[test]
fn consistent_unsaturated_terminal_can_continue_when_requested_order_fits_literal_h() -> TestResult
{
    for (old, new, hydrogens) in [
        ("double", "single", [2, 1, 3]),
        ("triple", "single", [1, 0, 3]),
        ("double", "double", [2, 0, 2]),
    ] {
        let mut engine = initial(old)?;
        let before = engine.stored_document()?.clone();
        engine.dispatch("bond.extend", extend(new))?;
        let fragment = &engine.document()?.pages()[0].fragments()[0];
        assert_eq!(
            fragment
                .atoms()
                .iter()
                .map(|a| a.explicit_hydrogens)
                .collect::<Vec<_>>(),
            hydrogens.map(Some)
        );
        assert_eq!(
            fragment
                .atoms()
                .iter()
                .map(|a| a.id.get())
                .collect::<Vec<_>>(),
            [1, 2, 3]
        );
        assert_eq!(
            serde_json::to_value(fragment.bonds())?,
            json!([{"id":1,"begin":1,"end":2,"order":old},{"id":2,"begin":2,"end":3,"order":new}])
        );
        let after = engine.stored_document()?.clone();
        engine.dispatch("history.undo", json!({}))?;
        assert_eq!(engine.stored_document()?, &before);
        engine.dispatch("history.redo", json!({}))?;
        assert_eq!(engine.stored_document()?, &after);
    }
    Ok(())
}

#[test]
fn insufficient_literal_h_refuses_without_advancing_history_or_identity_floors() -> TestResult {
    for (old, new) in [
        ("double", "triple"),
        ("triple", "double"),
        ("triple", "triple"),
    ] {
        let mut engine = initial(old)?;
        let before = serde_json::to_value(engine.state())?;
        assert_eq!(
            engine
                .dispatch("bond.extend", extend(new))
                .err()
                .ok_or("overvalent request accepted")?
                .info()
                .code,
            "unsupported_content"
        );
        assert_eq!(serde_json::to_value(engine.state())?, before);
        engine.dispatch("bond.extend", extend("single"))?;
        let fragment = &engine.document()?.pages()[0].fragments()[0];
        assert_eq!(fragment.atoms().last().ok_or("new atom")?.id.get(), 3);
        assert_eq!(fragment.bonds().last().ok_or("new bond")?.id.get(), 2);
    }
    Ok(())
}
