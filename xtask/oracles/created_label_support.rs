//! Frozen original created-label vectors and independent actual-file reader; tests only.
use serde_json::{Value, json};
use std::{
    io::Write,
    process::{Command, Stdio},
};
pub type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
pub const METADATA48: &str =
    include_str!("../../crates/cli/tests/support/connected-label-old48.json");
pub fn source(name: &str) -> &'static str {
    let value = match name {
        "base" => include_str!("../../crates/io/tests/fixtures/created-labels/base.cdxml"),
        "created_3_black" => {
            include_str!("../../crates/io/tests/fixtures/created-labels/created_3_black.cdxml")
        }
        "created_3_2_black" => {
            include_str!("../../crates/io/tests/fixtures/created-labels/created_3_2_black.cdxml")
        }
        "created_all_black" => {
            include_str!("../../crates/io/tests/fixtures/created-labels/created_all_black.cdxml")
        }
        "created_2_black" => {
            include_str!("../../crates/io/tests/fixtures/created-labels/created_2_black.cdxml")
        }
        "created_3_blue14" => {
            include_str!("../../crates/io/tests/fixtures/created-labels/created_3_blue14.cdxml")
        }
        "created_3_nh" => {
            include_str!("../../crates/io/tests/fixtures/created-labels/created_3_nh.cdxml")
        }
        "created_3_old48_nh" => {
            include_str!("../../crates/io/tests/fixtures/created-labels/created_3_old48_nh.cdxml")
        }
        "endpoint_edited_created_3" => include_str!(
            "../../crates/io/tests/fixtures/created-labels/endpoint_edited_created_3.cdxml"
        ),
        "created_3_all_p24_m12" => include_str!(
            "../../crates/io/tests/fixtures/created-labels/created_3_all_p24_m12.cdxml"
        ),
        "created_3_selected_p12_5_m7_25" => include_str!(
            "../../crates/io/tests/fixtures/created-labels/created_3_selected_p12_5_m7_25.cdxml"
        ),
        "created_all_p24_m12" => {
            include_str!("../../crates/io/tests/fixtures/created-labels/created_all_p24_m12.cdxml")
        }
        "paired_base" => {
            include_str!("../../crates/io/tests/fixtures/created-labels/paired_base.cdxml")
        }
        "paired_created_3" => {
            include_str!("../../crates/io/tests/fixtures/created-labels/paired_created_3.cdxml")
        }
        "dense_base" => {
            include_str!("../../crates/io/tests/fixtures/created-labels/dense_base.cdxml")
        }
        "dense_created_3" => {
            include_str!("../../crates/io/tests/fixtures/created-labels/dense_created_3.cdxml")
        }
        "dense_created_3_2" => {
            include_str!("../../crates/io/tests/fixtures/created-labels/dense_created_3_2.cdxml")
        }
        "dense_created_all" => {
            include_str!("../../crates/io/tests/fixtures/created-labels/dense_created_all.cdxml")
        }
        "caption_base" => {
            include_str!("../../crates/io/tests/fixtures/created-labels/caption_base.cdxml")
        }
        "caption_created_3" => {
            include_str!("../../crates/io/tests/fixtures/created-labels/caption_created_3.cdxml")
        }
        _ => "",
    };
    assert!(!value.is_empty(), "declared original created-label source");
    value
}
pub fn facts(name: &str) -> Value {
    let all: Value = serde_json::from_str(include_str!(
        "../../crates/io/tests/fixtures/created-labels/creation-facts.json"
    ))
    .unwrap_or(Value::Null);
    assert!(all.is_object(), "frozen creation facts object");
    let value = all.get(name).cloned().unwrap_or(Value::Null);
    assert!(value.is_object(), "declared original created-label facts");
    value
}
pub fn black_style() -> Value {
    json!({"font_family":"Roboto","size_pt":12.0,"face":0,"rgb":[0.0,0.0,0.0]})
}
pub fn blue_style() -> Value {
    json!({"font_family":"Roboto","size_pt":14.0,"face":0,"rgb":[0.2,0.4,0.8]})
}
pub fn params(revision: u64, atom: u32, element: u8, h: u16, style: Value) -> Value {
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
            include_str!("../../crates/io/tests/fixtures/created-labels/created_label_oracle.py"),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("created reader stdin")?
        .write_all(raw)?;
    let out = child.wait_with_output()?;
    if !out.status.success() {
        return Err(format!(
            "strict independent created-label reader: {}",
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
            assert_eq!(atom[field], n["flags"][xml], "creation preserves flags");
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
