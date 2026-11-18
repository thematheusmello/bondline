//! Original CDX resources and independent raw/query facts; test-only helper.
use serde_json::Value;
use std::{
    io::Write,
    process::{Command, Stdio},
};
pub type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
pub fn source(name: &str) -> &'static [u8] {
    let raw: &[u8] = match name {
        "tables_implicit" => {
            include_bytes!("../../crates/cdx/tests/fixtures/resources/tables_implicit.cdx")
        }
        "tables_explicit" => {
            include_bytes!("../../crates/cdx/tests/fixtures/resources/tables_explicit.cdx")
        }
        "tables_header" => {
            include_bytes!("../../crates/cdx/tests/fixtures/resources/tables_header.cdx")
        }
        "tables_implicit_file_end" => {
            include_bytes!("../../crates/cdx/tests/fixtures/resources/tables_implicit_file_end.cdx")
        }
        "tables_explicit_file_end" => {
            include_bytes!("../../crates/cdx/tests/fixtures/resources/tables_explicit_file_end.cdx")
        }
        "tables_reordered" => {
            include_bytes!("../../crates/cdx/tests/fixtures/resources/tables_reordered.cdx")
        }
        "tables_extended" => {
            include_bytes!("../../crates/cdx/tests/fixtures/resources/tables_extended.cdx")
        }
        "absent" => include_bytes!("../../crates/cdx/tests/fixtures/resources/absent.cdx"),
        "empty_tables" => {
            include_bytes!("../../crates/cdx/tests/fixtures/resources/empty_tables.cdx")
        }
        "font_empty_mac" => {
            include_bytes!("../../crates/cdx/tests/fixtures/resources/font_empty_mac.cdx")
        }
        "palette_empty" => {
            include_bytes!("../../crates/cdx/tests/fixtures/resources/palette_empty.cdx")
        }
        "palette_one" => {
            include_bytes!("../../crates/cdx/tests/fixtures/resources/palette_one.cdx")
        }
        "palette_nondefault" => {
            include_bytes!("../../crates/cdx/tests/fixtures/resources/palette_nondefault.cdx")
        }
        "max_name" => include_bytes!("../../crates/cdx/tests/fixtures/resources/max_name.cdx"),
        "user_tags_only" => {
            include_bytes!("../../crates/cdx/tests/fixtures/resources/user_tags_only.cdx")
        }
        "max_fonts" => include_bytes!("../../crates/cdx/tests/fixtures/resources/max_fonts.cdx"),
        "max_colors" => include_bytes!("../../crates/cdx/tests/fixtures/resources/max_colors.cdx"),
        "invalid_font_header" => {
            include_bytes!("../../crates/cdx/tests/fixtures/resources/invalid_font_header.cdx")
        }
        "invalid_font_count" => {
            include_bytes!("../../crates/cdx/tests/fixtures/resources/invalid_font_count.cdx")
        }
        "invalid_font_name" => {
            include_bytes!("../../crates/cdx/tests/fixtures/resources/invalid_font_name.cdx")
        }
        "invalid_font_trailing" => {
            include_bytes!("../../crates/cdx/tests/fixtures/resources/invalid_font_trailing.cdx")
        }
        "invalid_font_max_count" => {
            include_bytes!("../../crates/cdx/tests/fixtures/resources/invalid_font_max_count.cdx")
        }
        "invalid_color_header" => {
            include_bytes!("../../crates/cdx/tests/fixtures/resources/invalid_color_header.cdx")
        }
        "invalid_color_count" => {
            include_bytes!("../../crates/cdx/tests/fixtures/resources/invalid_color_count.cdx")
        }
        "invalid_color_trailing" => {
            include_bytes!("../../crates/cdx/tests/fixtures/resources/invalid_color_trailing.cdx")
        }
        "invalid_color_max_count" => {
            include_bytes!("../../crates/cdx/tests/fixtures/resources/invalid_color_max_count.cdx")
        }
        "unsupported_platform_2" => {
            include_bytes!("../../crates/cdx/tests/fixtures/resources/unsupported_platform_2.cdx")
        }
        "unsupported_platform_65535" => include_bytes!(
            "../../crates/cdx/tests/fixtures/resources/unsupported_platform_65535.cdx"
        ),
        "duplicate_font_id" => {
            include_bytes!("../../crates/cdx/tests/fixtures/resources/duplicate_font_id.cdx")
        }
        "duplicate_font_late" => {
            include_bytes!("../../crates/cdx/tests/fixtures/resources/duplicate_font_late.cdx")
        }
        "duplicate_font_table" => {
            include_bytes!("../../crates/cdx/tests/fixtures/resources/duplicate_font_table.cdx")
        }
        "duplicate_color_table" => {
            include_bytes!("../../crates/cdx/tests/fixtures/resources/duplicate_color_table.cdx")
        }
        "duplicate_before_invalid_font" => include_bytes!(
            "../../crates/cdx/tests/fixtures/resources/duplicate_before_invalid_font.cdx"
        ),
        "owner_before_invalid_font" => include_bytes!(
            "../../crates/cdx/tests/fixtures/resources/owner_before_invalid_font.cdx"
        ),
        "owner_font_page" => {
            include_bytes!("../../crates/cdx/tests/fixtures/resources/owner_font_page.cdx")
        }
        "owner_font_group" => {
            include_bytes!("../../crates/cdx/tests/fixtures/resources/owner_font_group.cdx")
        }
        "owner_font_fragment" => {
            include_bytes!("../../crates/cdx/tests/fixtures/resources/owner_font_fragment.cdx")
        }
        "owner_font_node" => {
            include_bytes!("../../crates/cdx/tests/fixtures/resources/owner_font_node.cdx")
        }
        "owner_font_text" => {
            include_bytes!("../../crates/cdx/tests/fixtures/resources/owner_font_text.cdx")
        }
        "owner_font_custom" => {
            include_bytes!("../../crates/cdx/tests/fixtures/resources/owner_font_custom.cdx")
        }
        "owner_font_nested_document" => include_bytes!(
            "../../crates/cdx/tests/fixtures/resources/owner_font_nested_document.cdx"
        ),
        "owner_color_page" => {
            include_bytes!("../../crates/cdx/tests/fixtures/resources/owner_color_page.cdx")
        }
        "owner_color_group" => {
            include_bytes!("../../crates/cdx/tests/fixtures/resources/owner_color_group.cdx")
        }
        "owner_color_fragment" => {
            include_bytes!("../../crates/cdx/tests/fixtures/resources/owner_color_fragment.cdx")
        }
        "owner_color_node" => {
            include_bytes!("../../crates/cdx/tests/fixtures/resources/owner_color_node.cdx")
        }
        "owner_color_text" => {
            include_bytes!("../../crates/cdx/tests/fixtures/resources/owner_color_text.cdx")
        }
        "owner_color_custom" => {
            include_bytes!("../../crates/cdx/tests/fixtures/resources/owner_color_custom.cdx")
        }
        "owner_color_nested_document" => include_bytes!(
            "../../crates/cdx/tests/fixtures/resources/owner_color_nested_document.cdx"
        ),
        _ => &[],
    };
    assert!(!raw.is_empty(), "declared original CDX resource source");
    raw
}
pub fn facts(name: &str) -> Value {
    let all: Value = serde_json::from_str(include_str!(
        "../../crates/cdx/tests/fixtures/resources/resource-facts.json"
    ))
    .unwrap_or(Value::Null);
    assert!(all.is_object(), "frozen resource facts object");
    let row = all.get(name).cloned().unwrap_or(Value::Null);
    assert!(row.is_object(), "declared original resource facts");
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
            include_str!("../../crates/cdx/tests/fixtures/resources/resource_oracle.py"),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("resource reader stdin")?
        .write_all(raw)?;
    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(format!(
            "strict independent resource raw reader: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(serde_json::from_slice(&output.stdout)?)
}
