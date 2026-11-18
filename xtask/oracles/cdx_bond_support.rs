//! Independent original plain CDX bond sources, whole models and actual-file reader.
use serde_json::Value;
use std::{
    io::Write,
    process::{Command, Stdio},
};
pub type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
pub const METADATA49: &str =
    include_str!("../../crates/io/tests/fixtures/cdx-literal-metadata/old49.json");
pub fn source(name: &str) -> &'static [u8] {
    let raw: &[u8] = match name {
        "single_absent" => {
            include_bytes!("../../crates/io/tests/fixtures/cdx-plain-bonds/single_absent.cdx")
        }
        "single_solid" => {
            include_bytes!("../../crates/io/tests/fixtures/cdx-plain-bonds/single_solid.cdx")
        }
        "ethene" => include_bytes!("../../crates/io/tests/fixtures/cdx-plain-bonds/ethene.cdx"),
        "ethyne" => include_bytes!("../../crates/io/tests/fixtures/cdx-plain-bonds/ethyne.cdx"),
        "auto_center" => {
            include_bytes!("../../crates/io/tests/fixtures/cdx-plain-bonds/auto_center.cdx")
        }
        "auto_right" => {
            include_bytes!("../../crates/io/tests/fixtures/cdx-plain-bonds/auto_right.cdx")
        }
        "auto_left" => {
            include_bytes!("../../crates/io/tests/fixtures/cdx-plain-bonds/auto_left.cdx")
        }
        "manual_center" => {
            include_bytes!("../../crates/io/tests/fixtures/cdx-plain-bonds/manual_center.cdx")
        }
        "manual_right" => {
            include_bytes!("../../crates/io/tests/fixtures/cdx-plain-bonds/manual_right.cdx")
        }
        "manual_left" => {
            include_bytes!("../../crates/io/tests/fixtures/cdx-plain-bonds/manual_left.cdx")
        }
        "reversed_right" => {
            include_bytes!("../../crates/io/tests/fixtures/cdx-plain-bonds/reversed_right.cdx")
        }
        "reordered_right" => {
            include_bytes!("../../crates/io/tests/fixtures/cdx-plain-bonds/reordered_right.cdx")
        }
        "extended_right" => {
            include_bytes!("../../crates/io/tests/fixtures/cdx-plain-bonds/extended_right.cdx")
        }
        "embedded_right" => {
            include_bytes!("../../crates/io/tests/fixtures/cdx-plain-bonds/embedded_right.cdx")
        }
        "implied_right" => {
            include_bytes!("../../crates/io/tests/fixtures/cdx-plain-bonds/implied_right.cdx")
        }
        "ethene_unknown_h" => {
            include_bytes!("../../crates/io/tests/fixtures/cdx-plain-bonds/ethene_unknown_h.cdx")
        }
        _ => &[],
    };
    assert!(!raw.is_empty(), "declared original plain CDX bond source");
    raw
}
pub fn facts(name: &str) -> Value {
    let all: Value = serde_json::from_str(include_str!(
        "../../crates/io/tests/fixtures/cdx-plain-bonds/bond-facts.json"
    ))
    .unwrap_or(Value::Null);
    assert!(all.is_object(), "frozen plain CDX bond facts object");
    let row = all.get(name).cloned().unwrap_or(Value::Null);
    assert!(row.is_object(), "declared original plain CDX bond facts");
    row
}
pub fn model(name: &str) -> Result<Value> {
    let model = facts(name)
        .get("model")
        .cloned()
        .ok_or("frozen complete model")?;
    if !model.is_object() {
        return Err("frozen complete model must be an object".into());
    }
    Ok(model)
}
pub fn read(raw: &[u8]) -> Result<Value> {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child = Command::new(python)
        .args([
            "-X",
            "utf8",
            "-B",
            "-c",
            include_str!("../../crates/io/tests/fixtures/cdx-plain-bonds/bond_oracle.py"),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("plain CDX bond reader stdin")?
        .write_all(raw)?;
    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(format!(
            "strict independent plain CDX bond reader: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(serde_json::from_slice(&output.stdout)?)
}
