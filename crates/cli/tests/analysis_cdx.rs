//! The user-facing file Open, Analysis window and formula commands through the actual CLI.
use serde_json::{Value, json};
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

type Test<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn open_and_analyze(relative_source: &str, atom: Option<u32>) -> Test<Value> {
    analyze(relative_source, atom, "analysis.formula")
}

fn analyze(relative_source: &str, atom: Option<u32>, command: &str) -> Test<Value> {
    analyze_options(relative_source, atom, command, false)
}

fn analyze_options(
    relative_source: &str,
    atom: Option<u32>,
    command: &str,
    elemental: bool,
) -> Test<Value> {
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
    let mut params =
        json!({"expected_revision":1,"expected_selection_epoch":u64::from(atom.is_some())});
    if elemental {
        params["include_elemental_analysis"] = json!(true);
    }
    commands.push(json!({"command":command,"params":params}));
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

#[test]
fn open_file_returns_mass_and_weight_through_the_real_cli() -> Test {
    let entire = analyze(
        "cdx-mixed-skeleton/ethanol.cdx",
        None,
        "analysis.properties",
    )?;
    assert_eq!(entire["formula"], "C2H6O");
    assert!(
        (entire["masses"]["exact_mass_da"].as_f64().ok_or("mass")? - 46.04186481295).abs() < 1e-8
    );
    assert_eq!(entire["masses"]["molecular_weight"], 46.069);
    let partial = analyze(
        "cdx-skeletal/all_h_missing.cdx",
        Some(2),
        "analysis.properties",
    )?;
    assert_eq!(partial["formula"], "CH2");
    assert_eq!(partial["partial_fragments"], true);
    assert!(
        (partial["masses"]["molecular_weight"]
            .as_f64()
            .ok_or("weight")?
            - 14.027)
            .abs()
            < 1e-8
    );
    Ok(())
}

#[test]
fn elemental_analysis_of_imported_ethanol_and_an_uncapped_selection_agrees_with_rdkit() -> Test {
    for (source, atom, notation) in [
        ("cdx-mixed-skeleton/ethanol.cdx", None, "CCO"),
        ("cdx-skeletal/all_h_missing.cdx", Some(2), "[CH2]"),
    ] {
        let report = analyze_options(source, atom, "analysis.properties", true)?;
        assert_eq!(report["partial_fragments"], atom.is_some());
        let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
            if cfg!(windows) {
                "python".into()
            } else {
                "python3".into()
            }
        });
        let mut child = Command::new(python)
            .arg(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../xtask/oracles/analysis_elemental.py"
            ))
            .arg(notation)
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        child
            .stdin
            .take()
            .ok_or("stdin")?
            .write_all(&serde_json::to_vec(&report)?)?;
        let output = child.wait_with_output()?;
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}

#[test]
fn printed_skeletal_label_edits_reopen_with_the_exact_source_and_independent_formula() -> Test {
    let oracle =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../xtask/oracles/cdx_skeletal_label.py");
    let out = Command::new(std::env::var_os("BONDLINE_PYTHON").ok_or("pinned RDKit Python")?)
        .arg("-B")
        .arg(oracle)
        .arg(env!("CARGO_BIN_EXE_bondline-cli"))
        .output()?;
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    Ok(())
}
