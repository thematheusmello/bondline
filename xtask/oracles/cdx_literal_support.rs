//! Original CDX bytes and independent struct facts; test-only external reader.
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
        "absent_implied" => {
            include_bytes!("../../crates/io/tests/fixtures/cdx-literal-metadata/absent_implied.cdx")
        }
        "scalar_zero_implied" => include_bytes!(
            "../../crates/io/tests/fixtures/cdx-literal-metadata/scalar_zero_implied.cdx"
        ),
        "zero_present_explicit" => include_bytes!(
            "../../crates/io/tests/fixtures/cdx-literal-metadata/zero_present_explicit.cdx"
        ),
        "methylammonium_explicit" => include_bytes!(
            "../../crates/io/tests/fixtures/cdx-literal-metadata/methylammonium_explicit.cdx"
        ),
        "methylammonium_extended" => include_bytes!(
            "../../crates/io/tests/fixtures/cdx-literal-metadata/methylammonium_extended.cdx"
        ),
        "methylammonium_embedded" => include_bytes!(
            "../../crates/io/tests/fixtures/cdx-literal-metadata/methylammonium_embedded.cdx"
        ),
        "methyl_doublet" => {
            include_bytes!("../../crates/io/tests/fixtures/cdx-literal-metadata/methyl_doublet.cdx")
        }
        "singlet" => {
            include_bytes!("../../crates/io/tests/fixtures/cdx-literal-metadata/singlet.cdx")
        }
        "triplet" => {
            include_bytes!("../../crates/io/tests/fixtures/cdx-literal-metadata/triplet.cdx")
        }
        "extrema" => {
            include_bytes!("../../crates/io/tests/fixtures/cdx-literal-metadata/extrema.cdx")
        }
        "reordered" => {
            include_bytes!("../../crates/io/tests/fixtures/cdx-literal-metadata/reordered.cdx")
        }
        "default_carbon_fields" => include_bytes!(
            "../../crates/io/tests/fixtures/cdx-literal-metadata/default_carbon_fields.cdx"
        ),
        "deuteron" => {
            include_bytes!("../../crates/io/tests/fixtures/cdx-literal-metadata/deuteron.cdx")
        }
        "charge4_low_one" => include_bytes!(
            "../../crates/io/tests/fixtures/cdx-literal-metadata/charge4_low_one.cdx"
        ),
        "charge4_high_one" => include_bytes!(
            "../../crates/io/tests/fixtures/cdx-literal-metadata/charge4_high_one.cdx"
        ),
        "charge4_high_minus_one" => include_bytes!(
            "../../crates/io/tests/fixtures/cdx-literal-metadata/charge4_high_minus_one.cdx"
        ),
        "charge4_low_noise_max" => include_bytes!(
            "../../crates/io/tests/fixtures/cdx-literal-metadata/charge4_low_noise_max.cdx"
        ),
        "charge4_min" => {
            include_bytes!("../../crates/io/tests/fixtures/cdx-literal-metadata/charge4_min.cdx")
        }
        _ => &[],
    };
    assert!(!raw.is_empty(), "declared original literal CDX source");
    raw
}
pub fn facts(name: &str) -> Value {
    let all: Value = serde_json::from_str(include_str!(
        "../../crates/io/tests/fixtures/cdx-literal-metadata/literal-facts.json"
    ))
    .unwrap_or(Value::Null);
    assert!(all.is_object(), "frozen literal CDX facts object");
    let row = all.get(name).cloned().unwrap_or(Value::Null);
    assert!(row.is_object(), "declared original literal CDX facts");
    row
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
            include_str!("../../crates/io/tests/fixtures/cdx-literal-metadata/literal_oracle.py"),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("literal CDX reader stdin")?
        .write_all(raw)?;
    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(format!(
            "strict independent literal CDX reader: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(serde_json::from_slice(&output.stdout)?)
}
