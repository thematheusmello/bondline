//! The user-facing file Open, Analysis window and formula commands through the actual CLI.
use serde_json::{Value, json};
use std::{path::Path, process::Command};

type Test<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn open_and_analyze(relative_source: &str, atom: Option<u32>) -> Test<Value> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../io/tests/fixtures")
        .join(relative_source);
    let mut commands = vec![
        json!({"command":"document.open_cdx","params":{"path":path}}),
        json!({"command":"ui.analysis","params":{"expected_revision":1,"visible":true}}),
    ];
    if let Some(id) = atom {
        commands.push(json!({"command":"ui.selection","params":{"action":"set","expected_revision":1,"page_id":1,"atom_ids":[id]}}));
    }
    commands.push(json!({"command":"analysis.formula","params":{"expected_revision":1,"expected_selection_epoch":u64::from(atom.is_some())}}));
    let mut cli = Command::new(env!("CARGO_BIN_EXE_bondline-cli"));
    cli.arg("run");
    for command in commands {
        cli.arg("--cmd").arg(command.to_string());
    }
    let out = cli.output()?;
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let result: Value = serde_json::from_slice(&out.stdout)?;
    assert_eq!(result["source"]["format"], "cdx");
    assert_eq!(result["source"]["modified"], false);
    assert_eq!(result["analysis_window_open"], true);
    assert_eq!(result["changed"], false);
    assert_eq!(result["undo_depth"], 0);
    assert_eq!(result["redo_depth"], 0);
    assert_eq!(result["file"]["dirty"], false);
    Ok(result["analysis"].clone())
}

#[test]
fn skeletal_propane_analysis_resolves_h_on_the_complete_graph_before_selection() -> Test {
    assert_eq!(
        open_and_analyze("cdx-skeletal/propane.cdx", None)?,
        json!({"scope":"document","status":"available","atom_count":3,"fragment_count":1,"partial_fragments":false,"formula":"C3H8"})
    );
    assert_eq!(
        open_and_analyze("cdx-skeletal/missing_h.cdx", Some(1))?,
        json!({"scope":"selection","status":"available","atom_count":1,"fragment_count":1,"partial_fragments":true,"formula":"CH3"})
    );
    assert_eq!(
        open_and_analyze("cdx-skeletal/all_h_missing.cdx", None)?,
        json!({"scope":"document","status":"available","atom_count":3,"fragment_count":1,"partial_fragments":false,"formula":"C3H8","defaulted_hydrogen_atoms":3})
    );
    assert_eq!(
        open_and_analyze("cdx-skeletal/all_h_missing.cdx", Some(2))?,
        json!({"scope":"selection","status":"available","atom_count":1,"fragment_count":1,"partial_fragments":true,"formula":"CH2","defaulted_hydrogen_atoms":1})
    );
    assert_eq!(
        open_and_analyze("cdx-skeletal-cycle/cycle.cdx", None)?,
        json!({"scope":"document","status":"available","atom_count":4,"fragment_count":1,"partial_fragments":false,"formula":"C4H8","defaulted_hydrogen_atoms":4})
    );
    Ok(())
}

#[test]
fn open_mixed_cdx_distinguishes_written_from_unspecified_hydrogens() -> Test {
    assert_eq!(
        open_and_analyze("cdx-mixed-skeleton/literal_h.cdx", None)?,
        json!({"scope":"document","status":"available","atom_count":2,"fragment_count":1,"partial_fragments":false,"formula":"CH4O"})
    );
    for (name, formula, count, defaulted) in [
        ("methanol", "CH4O", 2, 1),
        ("ethanol", "C2H6O", 3, 2),
        ("amino_alcohol", "C3H9NO", 5, 3),
    ] {
        let path = format!("cdx-mixed-skeleton/{name}.cdx");
        assert_eq!(
            open_and_analyze(&path, None)?,
            json!({"scope":"document","status":"available","atom_count":count,"fragment_count":1,"partial_fragments":false,"formula":formula,"defaulted_hydrogen_atoms":defaulted})
        );
    }
    Ok(())
}
