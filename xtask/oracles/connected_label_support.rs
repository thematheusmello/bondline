//! Original connected sources plus frozen independent saved-file facts.
use serde_json::{Value, json};
use std::{
    io::Write,
    process::{Command, Stdio},
};
pub type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
pub const SOURCE: &str =
    include_str!("../../crates/io/tests/fixtures/connected-labels/water.cdxml");
pub const EDITED: &str =
    include_str!("../../crates/io/tests/fixtures/connected-labels/ammonia_after_label_edit.cdxml");
pub const METADATA48: &str =
    include_str!("../../crates/cli/tests/support/connected-label-old48.json");
pub const POINT: [f64; 2] = [160.0, 160.0];
pub fn params(revision: u64, element: u8, h: u16) -> Value {
    json!({"expected_revision":revision,"page_id":1,"fragment_id":1,"atom_id":1,"element":element,"explicit_hydrogens":h})
}
pub fn edited() -> String {
    EDITED.to_owned()
}
pub fn label(element: u8, h: u16) -> Value {
    assert!(
        matches!((element, h), (8, 0) | (7, 1)),
        "declared original test vector"
    );
    let (text, bounds) = if element == 8 {
        (
            "O",
            [156.5693359375, 155.6171875, 163.4306640625, 164.3828125],
        )
    } else {
        (
            "NH",
            [156.7216796875, 155.734375, 171.8330078125, 164.265625],
        )
    };
    json!({"position":null,"bounding_box":bounds,"justification":"left","label_justification":"left","label_alignment":null,
        "runs":[{"text":text,"style":{"font_family":"Roboto","size_pt":12.0,"face":0,"rgb":[0.2,0.4,0.8]}}]})
}
pub fn facts(element: u8, h: u16) -> Value {
    assert!(
        matches!((element, h), (8, 0) | (7, 1)),
        "declared original test vector"
    );
    let frozen = if element == 8 {
        include_str!("connected-label-water-facts.json")
    } else {
        include_str!("connected-label-ammonia-facts.json")
    };
    let parsed = serde_json::from_str(frozen).unwrap_or(Value::Null);
    assert!(
        parsed.is_object(),
        "frozen independent facts must parse to an object"
    );
    parsed
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
            include_str!("connected_label_oracle.py"),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("connected oracle stdin")?
        .write_all(raw)?;
    let result = child.wait_with_output()?;
    if !result.status.success() {
        return Err(format!(
            "independent connected source reader: {}",
            String::from_utf8_lossy(&result.stderr)
        )
        .into());
    }
    Ok(serde_json::from_slice(&result.stdout)?)
}
