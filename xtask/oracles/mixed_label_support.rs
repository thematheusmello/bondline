//! Frozen original mixed sources and independent actual-file consumption, tests only.
use serde_json::{Value, json};
use std::{
    io::Write,
    process::{Command, Stdio},
};
pub type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
pub const METADATA48: &str =
    include_str!("../../crates/cli/tests/support/connected-label-old48.json");
pub fn source(name: &str) -> &'static str {
    let raw = match name {
        "amino_alcohol" => {
            include_str!("../../crates/io/tests/fixtures/mixed-labels/amino_alcohol.cdxml")
        }
        "amino_diamine" => {
            include_str!("../../crates/io/tests/fixtures/mixed-labels/amino_diamine.cdxml")
        }
        "hidden_middle_p12_5_m7_25" => include_str!(
            "../../crates/io/tests/fixtures/mixed-labels/hidden_middle_p12_5_m7_25.cdxml"
        ),
        "all_p24_m12" => {
            include_str!("../../crates/io/tests/fixtures/mixed-labels/all_p24_m12.cdxml")
        }
        "subset_1_3_5_p12_5_m7_25" => include_str!(
            "../../crates/io/tests/fixtures/mixed-labels/subset_1_3_5_p12_5_m7_25.cdxml"
        ),
        "edited_all_p24_m12" => {
            include_str!("../../crates/io/tests/fixtures/mixed-labels/edited_all_p24_m12.cdxml")
        }
        "methanol" => include_str!("../../crates/io/tests/fixtures/mixed-labels/methanol.cdxml"),
        "methanol_bad_c2" => {
            include_str!("../../crates/io/tests/fixtures/mixed-labels/methanol_bad_c2.cdxml")
        }
        "mandatory_collinear" => {
            include_str!("../../crates/io/tests/fixtures/mixed-labels/mandatory_collinear.cdxml")
        }
        _ => "",
    };
    assert!(!raw.is_empty(), "declared frozen original mixed source");
    raw
}
pub fn facts(name: &str) -> Value {
    let v: Value = serde_json::from_str(include_str!(
        "../../crates/io/tests/fixtures/mixed-labels/mixed-label-facts.json"
    ))
    .unwrap_or(Value::Null);
    let value = v.get(name).cloned().unwrap_or(Value::Null);
    assert!(value.is_object(), "declared frozen mixed facts");
    value
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
            include_str!("../../crates/io/tests/fixtures/mixed-labels/mixed_label_oracle.py"),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("mixed oracle stdin")?
        .write_all(raw)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(serde_json::from_slice(&output.stdout)?)
}
pub fn label_params(revision: u64, atom: u32, element: u8, h: u16) -> Value {
    json!({"expected_revision":revision,"page_id":1,"fragment_id":1,"atom_id":atom,"element":element,"explicit_hydrogens":h})
}
pub fn motion(revision: u64, epoch: u64, x: f64, y: f64) -> Value {
    json!({"expected_revision":revision,"expected_selection_epoch":epoch,"delta_x_pt":x,"delta_y_pt":y})
}
/// Apply frozen source facts to the stated old model, retaining every unrelated field.
pub fn model_for_case(before: &Value, name: &str) -> Result<Value> {
    let facts = facts(name);
    let expected = facts["nodes"].as_array().ok_or("frozen nodes")?;
    let mut model = before.clone();
    let atoms = model["pages"][0]["fragments"][0]["atoms"]
        .as_array_mut()
        .ok_or("model atoms")?;
    assert_eq!(atoms.len(), expected.len());
    for atom in atoms {
        let node = expected
            .iter()
            .find(|n| n["model_id"] == atom["id"])
            .ok_or("frozen application atom")?;
        atom["element"] = node["element"].clone();
        atom["explicit_hydrogens"] = node["hydrogens"].clone();
        atom["position"] = json!({"x_pt":node["point"][0],"y_pt":node["point"][1]});
        if node["label"].is_null() {
            assert!(atom.get("authored_label").is_none_or(Value::is_null));
        } else {
            assert!(atom["authored_label"].is_object());
            atom["authored_label"]["bounding_box"] = node["label"]["bounding_box"].clone();
            atom["authored_label"]["runs"][0]["text"] = node["label"]["text"].clone();
        }
    }
    Ok(model)
}
