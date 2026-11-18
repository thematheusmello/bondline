//! Frozen original label-metadata vectors and independent actual-file reader; tests only.
use serde_json::{Value, json};
use std::{
    io::Write,
    process::{Command, Stdio},
};
pub type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
pub const METADATA49: &str =
    include_str!("../../crates/io/tests/fixtures/label-metadata/old49.json");
pub fn source(name: &str) -> &'static str {
    let value = match name {
        "mixed_a" => include_str!("../../crates/io/tests/fixtures/label-metadata/mixed_a.cdxml"),
        "mixed_b" => include_str!("../../crates/io/tests/fixtures/label-metadata/mixed_b.cdxml"),
        "edited_endpoint" => {
            include_str!("../../crates/io/tests/fixtures/label-metadata/edited_endpoint.cdxml")
        }
        "created_3" => {
            include_str!("../../crates/io/tests/fixtures/label-metadata/created_3.cdxml")
        }
        "all_printed" => {
            include_str!("../../crates/io/tests/fixtures/label-metadata/all_printed.cdxml")
        }
        "mixed_moved" => {
            include_str!("../../crates/io/tests/fixtures/label-metadata/mixed_moved.cdxml")
        }
        "compound" => include_str!("../../crates/io/tests/fixtures/label-metadata/compound.cdxml"),
        "all_moved" => {
            include_str!("../../crates/io/tests/fixtures/label-metadata/all_moved.cdxml")
        }
        "caption_owners" => {
            include_str!("../../crates/io/tests/fixtures/label-metadata/caption_owners.cdxml")
        }
        "unsigned_actual_font" => {
            include_str!("../../crates/io/tests/fixtures/label-metadata/unsigned_actual_font.cdxml")
        }
        "partial" => include_str!("../../crates/io/tests/fixtures/label-metadata/partial.cdxml"),
        "boxed" => include_str!("../../crates/io/tests/fixtures/label-metadata/boxed.cdxml"),
        _ => "",
    };
    assert!(!value.is_empty(), "declared original label-metadata source");
    value
}
pub fn facts(name: &str) -> Value {
    let all: Value = serde_json::from_str(include_str!(
        "../../crates/io/tests/fixtures/label-metadata/metadata-facts.json"
    ))
    .unwrap_or(Value::Null);
    assert!(all.is_object(), "frozen metadata facts object");
    let value = all.get(name).cloned().unwrap_or(Value::Null);
    assert!(value.is_object(), "declared original label-metadata facts");
    value
}
pub fn black_style() -> Value {
    json!({"font_family":"Roboto","size_pt":12.0,"face":0,"rgb":[0.0,0.0,0.0]})
}
pub fn create_params(revision: u64, atom: u32, element: u8, h: u16, style: Value) -> Value {
    json!({"expected_revision":revision,"page_id":1,"fragment_id":1,"atom_id":atom,"element":element,"explicit_hydrogens":h,"style":style})
}
pub fn label_params(revision: u64, atom: u32, element: u8, h: u16) -> Value {
    json!({"expected_revision":revision,"page_id":1,"fragment_id":1,"atom_id":atom,"element":element,"explicit_hydrogens":h})
}
pub fn motion(revision: u64, epoch: u64, x: f64, y: f64) -> Value {
    json!({"expected_revision":revision,"expected_selection_epoch":epoch,"delta_x_pt":x,"delta_y_pt":y})
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
            include_str!("../../crates/io/tests/fixtures/label-metadata/label_metadata_oracle.py"),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("metadata reader stdin")?
        .write_all(raw)?;
    let out = child.wait_with_output()?;
    if !out.status.success() {
        return Err(format!(
            "strict independent label-metadata reader: {}",
            String::from_utf8_lossy(&out.stderr)
        )
        .into());
    }
    Ok(serde_json::from_slice(&out.stdout)?)
}
/// Expected authored values from frozen independent facts; all unrelated baseline fields remain.
pub fn model_for_case(before: &Value, name: &str) -> Result<Value> {
    let facts = facts(name);
    let nodes = facts["nodes"].as_array().ok_or("frozen nodes")?;
    let mut model = before.clone();
    let atoms = model["pages"][0]["fragments"][0]["atoms"]
        .as_array_mut()
        .ok_or("model atoms")?;
    assert_eq!(atoms.len(), nodes.len());
    for atom in atoms {
        let n = nodes
            .iter()
            .find(|n| n["model_id"] == atom["id"])
            .ok_or("application atom")?;
        for (xml, field) in [
            ("ShowTerminalCarbonLabels", "show_terminal_carbon_labels"),
            (
                "ShowNonTerminalCarbonLabels",
                "show_non_terminal_carbon_labels",
            ),
        ] {
            assert_eq!(
                atom[field], n["flags"][xml],
                "metadata workflow preserves flags"
            );
        }
        atom["element"] = n["element"].clone();
        atom["explicit_hydrogens"] = n["hydrogens"].clone();
        atom["position"] = json!({"x_pt":n["point"][0],"y_pt":n["point"][1]});
        let label = &n["label"];
        if label.is_null() {
            atom.as_object_mut().ok_or("atom")?.remove("authored_label");
        } else {
            atom["authored_label"] = json!({"position":null,"justification":"left","label_justification":"left","label_alignment":null,
                "bounding_box":label["bounding_box"],"runs":[{"text":label["text"],"style":{"font_family":label["font"],"size_pt":label["size"],"face":label["face"],"rgb":label["rgb"]}}]});
        }
    }
    Ok(model)
}
