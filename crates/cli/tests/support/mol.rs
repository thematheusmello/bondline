use serde_json::Value;
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

pub const METHYL: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../io/tests/fixtures/mol/rdkit-CH3.mol"
));
pub const V3000_SOURCE: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../io/tests/fixtures/v3000/noncontiguous-cc.mol"
));

pub fn assert_v3000_state(result: &Value) -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(result["file"]["format"], "mol_v3000");
    assert_eq!(result["file"]["dirty"], false);
    assert_eq!(
        result["source"],
        serde_json::json!({"format":"mol","byte_length":344,"modified":false})
    );
    let fragments = result["document"]["pages"][0]["fragments"]
        .as_array()
        .ok_or("fragments")?;
    assert_eq!(fragments.len(), 1);
    let fragment = fragments.first().ok_or("fragment")?;
    let atoms = fragment["atoms"].as_array().ok_or("atoms")?;
    assert_eq!(atoms.len(), 2);
    for (atom, x) in atoms.iter().zip([42.0, 78.0]) {
        assert_eq!(atom["element"], 6);
        assert_eq!(atom["explicit_hydrogens"], 3);
        assert_eq!(atom["radical"], "none");
        assert_eq!(atom["position"], serde_json::json!({"x_pt":x,"y_pt":60.0}));
    }
    assert_eq!(
        fragment["bonds"],
        serde_json::json!([{"id":1,"begin":1,"end":2,"order":"single"}])
    );
    assert!(
        !result
            .to_string()
            .contains("Original specification-derived vector")
    );
    Ok(())
}

pub fn assert_methyl_state(
    result: &Value,
    x_pt: f64,
    y_pt: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(
        result["source"],
        serde_json::json!({"format":"mol","byte_length":170,"modified":false})
    );
    let fragments = result["document"]["pages"][0]["fragments"]
        .as_array()
        .ok_or("fragments")?;
    assert_eq!(fragments.len(), 1);
    let fragment = fragments.first().ok_or("fragment")?;
    assert_eq!(fragment["atoms"].as_array().ok_or("atoms")?.len(), 1);
    assert_eq!(fragment["bonds"], serde_json::json!([]));
    let atom = &fragment["atoms"][0];
    assert_eq!(atom["element"], 6);
    assert_eq!(atom["charge"], 0);
    assert_eq!(atom["isotope"], Value::Null);
    assert_eq!(atom["radical"], "doublet");
    assert_eq!(atom["explicit_hydrogens"], 3);
    for (axis, expected) in [("x_pt", x_pt), ("y_pt", y_pt)] {
        assert!((atom["position"][axis].as_f64().ok_or("point")? - expected).abs() < 1e-10);
    }
    let serialized = result.to_string();
    assert!(!serialized.contains("Mrv0541"));
    assert!(!serialized.contains("M  END"));
    Ok(())
}

pub fn independently_read_mol(path: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    read_with_oracle(path, "mol_oracle.py")
}

pub fn independently_read_v3000(path: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    read_with_oracle(path, "v3000_oracle.py")
}

fn read_with_oracle(path: &Path, script: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child = Command::new(python)
        .arg("-B")
        .arg(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../io/tests")
                .join(script),
        )
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("MOL oracle stdin")?
        .write_all(&std::fs::read(path)?)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(serde_json::from_slice(&output.stdout)?)
}
