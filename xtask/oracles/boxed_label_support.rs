//! Original shared fixture and independent actual-file reader; no Bondline producer oracle.
use serde_json::{Value, json};
use std::{
    io::Write,
    process::{Command, Stdio},
};
pub type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
pub const SOURCE: &str = r#"<CDXML CreationProgram="Original box-label oracle"><fonttable><font id="42" name="Roboto"/></fonttable><colortable><color r="1" g="1" b="1"/><color r="0" g="0" b="0"/><color r="0.2" g="0.4" b="0.8"/></colortable><page id="90" DrawingSpace="Poster" Width="512" Height="512"><fragment id="51"><n id="73" NodeType="Element" Element="8" Charge="0" Radical="None" NumHydrogens="1" p="84.1220703125 95.734375"><t id="21" BoundingBox="80.69140625 91.3515625 95.796875 100.1171875" Justification="Left" LabelJustification="Left"><s font="42" size="12" face="0" color="4">OH</s></t></n></fragment></page></CDXML>"#;
pub const POINT: [f64; 2] = [84.1220703125, 95.734375];
pub const OH_BOX: [f64; 4] = [80.69140625, 91.3515625, 95.796875, 100.1171875];
pub const NH2_BOX: [f64; 4] = [80.84375, 91.3515625, 103.26171875, 100.0];
pub fn params(revision: u64, element: u8, h: u16) -> Value {
    json!({"expected_revision":revision,"page_id":1,"fragment_id":1,"atom_id":1,"element":element,"explicit_hydrogens":h})
}
pub fn edited() -> String {
    SOURCE
        .replace("Element=\"8\"", "Element=\"7\"")
        .replace("NumHydrogens=\"1\"", "NumHydrogens=\"2\"")
        .replace(
            "80.69140625 91.3515625 95.796875 100.1171875",
            "80.84375 91.3515625 103.26171875 100",
        )
        .replace(">OH</s>", ">NH2</s>")
}
pub fn label(element: u8, h: u16) -> Value {
    assert!(
        matches!((element, h), (8, 1) | (7, 2)),
        "declared test vector"
    );
    let (text, bounds) = if element == 8 {
        ("OH", OH_BOX)
    } else {
        ("NH2", NH2_BOX)
    };
    json!({"position":null,"bounding_box":bounds,"justification":"left","label_justification":"left","label_alignment":null,
        "runs":[{"text":text,"style":{"font_family":"Roboto","size_pt":12.0,"face":0,"rgb":[0.2,0.4,0.8]}}]})
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
            include_str!("boxed_label_oracle.py"),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("boxed oracle stdin")?
        .write_all(raw)?;
    let result = child.wait_with_output()?;
    if !result.status.success() {
        return Err(format!(
            "independent boxed-label consumer: {}",
            String::from_utf8_lossy(&result.stderr)
        )
        .into());
    }
    Ok(serde_json::from_slice(&result.stdout)?)
}
pub fn facts(element: u8, h: u16) -> Value {
    assert!(
        matches!((element, h), (8, 1) | (7, 2)),
        "declared test vector"
    );
    let (text, bounds, formula) = if element == 8 {
        ("OH", OH_BOX, "HO")
    } else {
        ("NH2", NH2_BOX, "H2N")
    };
    json!({"element":element,"hydrogens":h,"point":POINT,"bounding_box":bounds,"text":text,"font":"Roboto","size":12.0,"face":0,"rgb":[0.2,0.4,0.8],"numeric_formula":formula})
}
