use bondline_commands::Engine;
use serde_json::{Value, json};

#[test]
fn selecting_chain_modes_and_constraints_only_changes_the_transient_shared_tool_epoch()
-> Result<(), Box<dyn std::error::Error>> {
    let mut engine = Engine::default();
    engine.dispatch("ui.bond_tool", json!({"order":"triple"}))?;
    let before = serde_json::to_value(engine.state())?;
    for (params, operation) in [
        (
            json!({"action":"select","kind":"acyclic"}),
            json!({"kind":"acyclic","fixed_lengths":true,"fixed_angles":true,"epoch":2,"changed":true}),
        ),
        (
            json!({"action":"constraints","fixed_lengths":false,"fixed_angles":true}),
            json!({"kind":"acyclic","fixed_lengths":false,"fixed_angles":true,"epoch":3,"changed":true}),
        ),
        (
            json!({"action":"constraints","fixed_lengths":false,"fixed_angles":true}),
            json!({"kind":"acyclic","fixed_lengths":false,"fixed_angles":true,"epoch":3,"changed":false}),
        ),
        (
            json!({"action":"select","kind":"snaking"}),
            json!({"kind":"snaking","fixed_lengths":false,"fixed_angles":true,"epoch":4,"changed":true}),
        ),
    ] {
        let result = serde_json::to_value(engine.dispatch("ui.chain_tool", params)?)?;
        assert_eq!(result["ui_chain_tool_operation"], operation);
        assert_eq!(result["changed"], false);
        for key in [
            "document",
            "revision",
            "undo_depth",
            "redo_depth",
            "file",
            "source",
        ] {
            assert_eq!(result[key], before[key], "chain tool changed {key}");
        }
        for key in ["file_operation", "ui_add_chain", "ui_add_chain_operation"] {
            assert_eq!(result[key], Value::Null);
        }
        assert_eq!(
            engine.bond_tool().order,
            bondline_commands::BondDrawOrder::Triple
        );
    }
    Ok(())
}
