//! Independent original dependent-label fixtures and actual-file consumer, tests only.
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
        "boxed" => include_str!("../../crates/io/tests/fixtures/dependent-labels/boxed.cdxml"),
        "boxed_pair" => {
            include_str!("../../crates/io/tests/fixtures/dependent-labels/boxed_pair.cdxml")
        }
        "boxed_rounding_boundary" => include_str!(
            "../../crates/io/tests/fixtures/dependent-labels/boxed_rounding_boundary.cdxml"
        ),
        "boxed_tolerated_raw" => include_str!(
            "../../crates/io/tests/fixtures/dependent-labels/boxed_tolerated_raw.cdxml"
        ),
        "boxed_sub_p12_5_m7_25" => include_str!(
            "../../crates/io/tests/fixtures/dependent-labels/boxed_sub_p12_5_m7_25.cdxml"
        ),
        "boxed_all_p24_m12" => {
            include_str!("../../crates/io/tests/fixtures/dependent-labels/boxed_all_p24_m12.cdxml")
        }
        "boxed_nudge_p25_m2" => {
            include_str!("../../crates/io/tests/fixtures/dependent-labels/boxed_nudge_p25_m2.cdxml")
        }
        "boxed_tolerated_sub_p12_5_m7_25" => include_str!(
            "../../crates/io/tests/fixtures/dependent-labels/boxed_tolerated_sub_p12_5_m7_25.cdxml"
        ),
        "boxed_pair_all_p24_m12" => include_str!(
            "../../crates/io/tests/fixtures/dependent-labels/boxed_pair_all_p24_m12.cdxml"
        ),
        "connected" => {
            include_str!("../../crates/io/tests/fixtures/dependent-labels/connected.cdxml")
        }
        "connected_nh" => {
            include_str!("../../crates/io/tests/fixtures/dependent-labels/connected_nh.cdxml")
        }
        "connected_component" => include_str!(
            "../../crates/io/tests/fixtures/dependent-labels/connected_component.cdxml"
        ),
        "connected_crossing_base" => include_str!(
            "../../crates/io/tests/fixtures/dependent-labels/connected_crossing_base.cdxml"
        ),
        "connected_sub_p12_5_m7_25" => include_str!(
            "../../crates/io/tests/fixtures/dependent-labels/connected_sub_p12_5_m7_25.cdxml"
        ),
        "connected_all_p24_m12" => include_str!(
            "../../crates/io/tests/fixtures/dependent-labels/connected_all_p24_m12.cdxml"
        ),
        "connected_nh_sub_p12_5_m7_25" => include_str!(
            "../../crates/io/tests/fixtures/dependent-labels/connected_nh_sub_p12_5_m7_25.cdxml"
        ),
        "connected_drag_p24_m12" => include_str!(
            "../../crates/io/tests/fixtures/dependent-labels/connected_drag_p24_m12.cdxml"
        ),
        "connected_nudge_p25_m2" => include_str!(
            "../../crates/io/tests/fixtures/dependent-labels/connected_nudge_p25_m2.cdxml"
        ),
        "connected_component_p24_m12" => include_str!(
            "../../crates/io/tests/fixtures/dependent-labels/connected_component_p24_m12.cdxml"
        ),
        _ => "",
    };
    assert!(
        !value.is_empty(),
        "declared independent dependent-label fixture"
    );
    value
}
pub fn facts(name: &str) -> Value {
    let all: Value = serde_json::from_str(include_str!(
        "../../crates/io/tests/fixtures/dependent-labels/dependent-label-facts.json"
    ))
    .unwrap_or(Value::Null);
    assert!(all.is_object(), "frozen facts must parse as an object");
    let value = all.get(name).cloned().unwrap_or(Value::Null);
    assert!(value.is_object(), "declared facts object");
    value
}
pub fn params(revision: u64, epoch: u64, x: f64, y: f64) -> Value {
    json!({"expected_revision":revision,"expected_selection_epoch":epoch,"delta_x_pt":x,"delta_y_pt":y})
}
pub fn model_after(before: &Value, ids: &[u32], dx: f64, dy: f64) -> Result<Value> {
    let mut result = before.clone();
    for fragment in result["pages"][0]["fragments"]
        .as_array_mut()
        .ok_or("fragments")?
    {
        for atom in fragment["atoms"].as_array_mut().ok_or("atoms")? {
            let id = u32::try_from(atom["id"].as_u64().ok_or("atom id")?)?;
            if !ids.contains(&id) {
                continue;
            }
            let x = atom["position"]["x_pt"].as_f64().ok_or("x")?;
            let y = atom["position"]["y_pt"].as_f64().ok_or("y")?;
            atom["position"] = json!({"x_pt":x+dx,"y_pt":y+dy});
            let values = atom["authored_label"]["bounding_box"]
                .as_array()
                .ok_or("box")?;
            let mut bbox = Vec::new();
            for (i, v) in values.iter().enumerate() {
                bbox.push(v.as_f64().ok_or("box number")? + if i % 2 == 0 { dx } else { dy });
            }
            atom["authored_label"]["bounding_box"] = json!(bbox);
        }
    }
    Ok(result)
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
            include_str!(
                "../../crates/io/tests/fixtures/dependent-labels/dependent_label_oracle.py"
            ),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child.stdin.take().ok_or("oracle stdin")?.write_all(raw)?;
    let result = child.wait_with_output()?;
    if !result.status.success() {
        return Err(format!(
            "independent dependent-label reader: {}",
            String::from_utf8_lossy(&result.stderr)
        )
        .into());
    }
    Ok(serde_json::from_slice(&result.stdout)?)
}
