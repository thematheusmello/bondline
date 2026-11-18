//! Original raw CDXString sources/facts; test-only, no product or codec implementation.
use serde_json::Value;
use std::{
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};
pub type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn python() -> std::ffi::OsString {
    std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    })
}
fn script(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../crates/cdx/tests/fixtures/raw-strings")
        .join(name)
}
pub fn source(name: &str) -> Result<Vec<u8>> {
    match name {
        "zero_empty" => Ok(include_bytes!(
            "../../crates/cdx/tests/fixtures/raw-strings/data/zero_empty.cdx"
        )
        .to_vec()),
        "zero_control_bytes" => Ok(include_bytes!(
            "../../crates/cdx/tests/fixtures/raw-strings/data/zero_control_bytes.cdx"
        )
        .to_vec()),
        "zero_all_bytes" => Ok(include_bytes!(
            "../../crates/cdx/tests/fixtures/raw-strings/data/zero_all_bytes.cdx"
        )
        .to_vec()),
        "single_empty" => Ok(include_bytes!(
            "../../crates/cdx/tests/fixtures/raw-strings/data/single_empty.cdx"
        )
        .to_vec()),
        "single_size_240" => Ok(include_bytes!(
            "../../crates/cdx/tests/fixtures/raw-strings/data/single_size_240.cdx"
        )
        .to_vec()),
        "single_size_241" => Ok(include_bytes!(
            "../../crates/cdx/tests/fixtures/raw-strings/data/single_size_241.cdx"
        )
        .to_vec()),
        "literal_zero_fields" => Ok(include_bytes!(
            "../../crates/cdx/tests/fixtures/raw-strings/data/literal_zero_fields.cdx"
        )
        .to_vec()),
        "full_uint16_fields" => Ok(include_bytes!(
            "../../crates/cdx/tests/fixtures/raw-strings/data/full_uint16_fields.cdx"
        )
        .to_vec()),
        "duplicate_starts" => Ok(include_bytes!(
            "../../crates/cdx/tests/fixtures/raw-strings/data/duplicate_starts.cdx"
        )
        .to_vec()),
        "decreasing_starts" => Ok(include_bytes!(
            "../../crates/cdx/tests/fixtures/raw-strings/data/decreasing_starts.cdx"
        )
        .to_vec()),
        "out_of_range_starts" => Ok(include_bytes!(
            "../../crates/cdx/tests/fixtures/raw-strings/data/out_of_range_starts.cdx"
        )
        .to_vec()),
        "scalar_byte_witness" => Ok(include_bytes!(
            "../../crates/cdx/tests/fixtures/raw-strings/data/scalar_byte_witness.cdx"
        )
        .to_vec()),
        "style_looking_remainder" => Ok(include_bytes!(
            "../../crates/cdx/tests/fixtures/raw-strings/data/style_looking_remainder.cdx"
        )
        .to_vec()),
        "invalid_empty_payload" => Ok(include_bytes!(
            "../../crates/cdx/tests/fixtures/raw-strings/data/invalid_empty_payload.cdx"
        )
        .to_vec()),
        "invalid_one_byte" => Ok(include_bytes!(
            "../../crates/cdx/tests/fixtures/raw-strings/data/invalid_one_byte.cdx"
        )
        .to_vec()),
        "invalid_one_missing" => Ok(include_bytes!(
            "../../crates/cdx/tests/fixtures/raw-strings/data/invalid_one_missing.cdx"
        )
        .to_vec()),
        "invalid_one_nine" => Ok(include_bytes!(
            "../../crates/cdx/tests/fixtures/raw-strings/data/invalid_one_nine.cdx"
        )
        .to_vec()),
        "invalid_three_29" => Ok(include_bytes!(
            "../../crates/cdx/tests/fixtures/raw-strings/data/invalid_three_29.cdx"
        )
        .to_vec()),
        "plain_implicit" => Ok(include_bytes!(
            "../../crates/cdx/tests/fixtures/raw-strings/data/plain_implicit.cdx"
        )
        .to_vec()),
        "plain_explicit" => Ok(include_bytes!(
            "../../crates/cdx/tests/fixtures/raw-strings/data/plain_explicit.cdx"
        )
        .to_vec()),
        "plain_header" => Ok(include_bytes!(
            "../../crates/cdx/tests/fixtures/raw-strings/data/plain_header.cdx"
        )
        .to_vec()),
        "plain_implicit_file_end" => Ok(include_bytes!(
            "../../crates/cdx/tests/fixtures/raw-strings/data/plain_implicit_file_end.cdx"
        )
        .to_vec()),
        "plain_explicit_file_end" => Ok(include_bytes!(
            "../../crates/cdx/tests/fixtures/raw-strings/data/plain_explicit_file_end.cdx"
        )
        .to_vec()),
        "extended_small" => Ok(include_bytes!(
            "../../crates/cdx/tests/fixtures/raw-strings/data/extended_small.cdx"
        )
        .to_vec()),
        "opaque_tag_0700" => Ok(include_bytes!(
            "../../crates/cdx/tests/fixtures/raw-strings/data/opaque_tag_0700.cdx"
        )
        .to_vec()),
        "opaque_tag_0805" => Ok(include_bytes!(
            "../../crates/cdx/tests/fixtures/raw-strings/data/opaque_tag_0805.cdx"
        )
        .to_vec()),
        "nested_user_object" => Ok(include_bytes!(
            "../../crates/cdx/tests/fixtures/raw-strings/data/nested_user_object.cdx"
        )
        .to_vec()),
        "invalid_extended_one_nine" => Ok(include_bytes!(
            "../../crates/cdx/tests/fixtures/raw-strings/data/invalid_extended_one_nine.cdx"
        )
        .to_vec()),
        "invalid_max_count_following_padding"
        | "long_text_65553"
        | "max_runs_65535_empty"
        | "max_runs_65535_text" => {
            let result = Command::new(python())
                .args(["-X", "utf8", "-B"])
                .arg(script("generate_strings.py"))
                .args(["--raw-case", name])
                .output()?;
            if !result.status.success() {
                return Err(format!(
                    "original string runtime vector: {}",
                    String::from_utf8_lossy(&result.stderr)
                )
                .into());
            }
            Ok(result.stdout)
        }
        _ => Err(format!("unknown original raw string case: {name}").into()),
    }
}
pub fn facts(name: &str) -> Result<Value> {
    let values: Value = serde_json::from_str(include_str!(
        "../../crates/cdx/tests/fixtures/raw-strings/data/string-facts.json"
    ))?;
    values
        .get(name)
        .cloned()
        .ok_or_else(|| format!("unknown original raw string facts: {name}").into())
}
pub fn read(raw: &[u8], record: usize) -> Result<Value> {
    let mut child = Command::new(python())
        .args(["-X", "utf8", "-B"])
        .arg(script("string_oracle.py"))
        .arg("--record")
        .arg(record.to_string())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("independent string reader stdin")?
        .write_all(raw)?;
    let result = child.wait_with_output()?;
    if !result.status.success() {
        return Err(format!(
            "independent string reader: {}",
            String::from_utf8_lossy(&result.stderr)
        )
        .into());
    }
    Ok(serde_json::from_slice(&result.stdout)?)
}
