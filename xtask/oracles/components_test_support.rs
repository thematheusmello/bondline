//! Shared literal mixed-component acceptance consumer; test-only.
use serde_json::{Value, json};
use std::{fs, path::Path, process::Command as Process};

type TestResult = Result<(), Box<dyn std::error::Error>>;

pub fn consume(path: &Path, edited: bool) -> TestResult {
    let h = if edited { 2 } else { 3 };
    let order = if edited { 2 } else { 1 };
    let formula = if edited { "C8H18O" } else { "C8H20O" };
    let expected = json!({
        "atoms":[[6,h],[6,h],[8,2],[6,2],[6,2],[6,2],[6,2],[6,2],[6,2]],
        "bonds":[[1,2,order],[4,5,1],[5,6,1],[6,7,1],[7,8,1],[8,9,1],[4,9,1]],
        "components":[[1,2],[3],[4,5,6,7,8,9]],"formula":formula
    });
    let result = Process::new(std::env::var_os("BONDLINE_PYTHON").ok_or("BONDLINE_PYTHON")?)
        .arg("-B")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../io/tests/smiles_components_oracle.py"
        ))
        .arg(if edited {
            "C=C.O.C1CCCCC1"
        } else {
            "CC.O.C1CCCCC1"
        })
        .arg(expected.to_string())
        .stdin(fs::File::open(path)?)
        .output()?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let verified: Value = serde_json::from_slice(&result.stdout)?;
    assert_eq!(verified["atoms"], 9);
    assert_eq!(verified["bonds"], 7);
    assert_eq!(verified["components"], 3);
    assert_eq!(verified["formula"], formula);
    assert_eq!(verified["no_assigned_stereo"], true);
    for key in ["atoms", "bonds", "components"] {
        assert_eq!(verified["raw_graph"][key], expected[key], "{key}");
    }
    Ok(())
}
