use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    process::{Command, Stdio},
};

pub const SOURCE: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../io/tests/fixtures/cdx/metadata-header.cdx"
));

pub fn assert_state(result: &Value) -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(result["file"]["format"], "cdx");
    assert_eq!(result["file"]["dirty"], false);
    assert_eq!(
        result["source"],
        json!({"format":"cdx","byte_length":319,"modified":false})
    );
    let fragments = result["document"]["pages"][0]["fragments"]
        .as_array()
        .ok_or("fragments")?;
    assert_eq!(fragments.len(), 1);
    let fragment = fragments.first().ok_or("fragment")?;
    let atoms = fragment["atoms"].as_array().ok_or("atoms")?;
    assert_eq!(atoms.len(), 2);
    for (atom, (x, y)) in atoms.iter().zip([(10.25, -3.5), (26.25, 8.5)]) {
        assert_eq!(atom["element"], 6);
        assert_eq!(atom["position"], json!({"x_pt":x,"y_pt":y}));
        assert!(atom["explicit_hydrogens"].is_null());
    }
    assert_eq!(
        fragment["bonds"],
        json!([{"id":1,"begin":1,"end":2,"order":"single"}])
    );
    assert!(!result.to_string().contains("Original metadata advisory"));
    Ok(())
}

pub fn independently_read_saved(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(fs::read(path)?, SOURCE);
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let output = Command::new(python)
        .arg("-B")
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("../io/tests/cdx_metadata_oracle.py"))
        .args(["verify", "header"])
        .stdin(fs::File::open(path)?)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let read: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(
        read,
        json!({"bytes":319,"sha256":"9af210eb978d085b143fad97074d55aa6512c44e02370d4a41e0376d06176a39","external_smiles":"CC","atoms":2,"bonds":1})
    );
    Ok(())
}
