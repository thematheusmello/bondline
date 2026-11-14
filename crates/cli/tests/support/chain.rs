use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};

pub fn create() -> Value {
    json!({"page_id":1,"begin_x_pt":100.0,"begin_y_pt":200.0,"end_x_pt":140.0,"end_y_pt":200.0})
}

pub fn extend(revision: u64, atom: u32, x: f64, y: f64) -> Value {
    json!({"expected_revision":revision,"page_id":1,"fragment_id":1,"begin_atom_id":atom,
        "end_x_pt":x,"end_y_pt":y,"hydrogen_policy":"rectify_terminal_carbon"})
}

pub fn independently_read(path: &Path, count: u32) -> Result<(), Box<dyn std::error::Error>> {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let output = Command::new(python)
        .arg("-B")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../commands/tests/terminal_chain_oracle.py"
        ))
        .args([count.to_string(), "a4".to_owned()])
        .stdin(fs::File::open(path)?)
        .output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}
