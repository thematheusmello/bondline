//! The chemistry belongs to the requested SDF record and is independent of its data fields.
use serde_json::{Value, json};
use std::{path::Path, process::Command};

#[test]
fn mixed_file_analyzes_each_record_without_changing_collection_or_history()
-> Result<(), Box<dyn std::error::Error>> {
    let input = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../io/tests/fixtures/sdf-preview/valence-absent-mixed.sdf");
    let source = std::fs::read(&input)?;
    for (record, formula, exact, defaulted) in
        [(1, "C2H6O", 46.041865, 3), (2, "CH5N", 31.042199, 2)]
    {
        let ops = [
            json!({"command":"document.open_sdf","params":{"path":input}}),
            json!({"command":"analysis.properties","params":{"expected_revision":1,"expected_selection_epoch":0,"sdf_record_id":record,"include_elemental_analysis":true}}),
        ];
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_bondline-cli"));
        cmd.arg("run");
        for op in ops {
            cmd.arg("--cmd").arg(op.to_string());
        }
        let out = cmd.output()?;
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let state: Value = serde_json::from_slice(&out.stdout)?;
        let result = &state["analysis"];
        assert_eq!(result["scope"], "sdf_record");
        assert_eq!(result["record_id"], record);
        assert_eq!(result["formula"], formula);
        assert_eq!(result["defaulted_hydrogen_atoms"], defaulted);
        assert!(
            (result["masses"]["exact_mass_da"]
                .as_f64()
                .ok_or("exact mass")?
                - exact)
                .abs()
                < 0.000001
        );
        assert_eq!(
            result["elemental_analysis"]["elements"]
                .as_array()
                .ok_or("elements")?
                .len(),
            3
        );
        assert_eq!(state["file"]["dirty"], false);
        assert_eq!(state["undo_depth"], 0);
        assert_eq!(state["revision"], 1);
        assert_eq!(std::fs::read(&input)?, source);
    }
    Ok(())
}

#[test]
fn omitted_and_invalid_record_ids_preserve_the_collection_boundary()
-> Result<(), Box<dyn std::error::Error>> {
    let input = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../io/tests/fixtures/sdf-preview/valence-absent-mixed.sdf");
    let open = json!({"command":"document.open_sdf","params":{"path":input}});
    let remove =
        json!({"command":"sdf.record.remove","params":{"expected_revision":1,"record_id":1}});
    let query = |rev, id: Option<u32>| {
        let mut params = json!({"expected_revision":rev,"expected_selection_epoch":0});
        if let Some(value) = id {
            params["sdf_record_id"] = json!(value);
        }
        json!({"command":"analysis.properties","params":params})
    };
    let run = |ops: &[Value]| -> Result<(bool, Value), Box<dyn std::error::Error>> {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_bondline-cli"));
        cmd.arg("run");
        for op in ops {
            cmd.arg("--cmd").arg(op.to_string());
        }
        let out = cmd.output()?;
        Ok((
            out.status.success(),
            serde_json::from_slice(if out.status.success() {
                &out.stdout
            } else {
                &out.stderr
            })?,
        ))
    };
    let (ok, whole) = run(&[open.clone(), query(1, None)])?;
    assert!(ok);
    assert_eq!(
        whole["analysis"],
        json!({"scope":"document","status":"unsupported_source"})
    );
    for (ops, reason) in [
        (vec![open.clone(), query(1, Some(0))], "invalid_params"),
        (vec![open.clone(), query(1, Some(3))], "not_found"),
        (
            vec![open.clone(), remove.clone(), query(1, Some(1))],
            "stale_revision",
        ),
    ] {
        let (ok, result) = run(&ops)?;
        assert!(!ok);
        assert_eq!(result["error"]["code"], reason);
    }
    let (ok, current) = run(&[open, remove, query(2, Some(1))])?;
    assert!(ok);
    assert_eq!(current["analysis"]["formula"], "CH5N");
    assert_eq!(current["undo_depth"], 1);
    Ok(())
}

#[test]
fn five_molecules_and_a_bounded_record_compare_with_an_independent_sdf_reader()
-> Result<(), Box<dyn std::error::Error>> {
    let out = Command::new(std::env::var_os("BONDLINE_PYTHON").ok_or("pinned RDKit environment")?)
        .arg("-B")
        .arg(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../xtask/oracles/sdf_record_analysis.py"),
        )
        .arg(env!("CARGO_BIN_EXE_bondline-cli"))
        .output()?;
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    Ok(())
}
