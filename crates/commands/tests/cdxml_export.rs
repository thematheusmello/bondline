use bondline_commands::Engine;
use serde_json::json;
use std::{
    io::Write,
    process::{Command, Stdio},
};

#[test]
fn registered_bond_creation_exports_as_ethane_in_external_rdkit()
-> Result<(), Box<dyn std::error::Error>> {
    let mut engine = Engine::default();
    engine.dispatch("bond.create",json!({"page_id":1,"begin_x_pt":100.0,"begin_y_pt":200.0,"end_x_pt":140.0,"end_y_pt":200.0}))?;
    let before = serde_json::to_vec(&engine.state())?;
    let bytes = engine.stored_document().to_cdxml()?;
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child = Command::new(python)
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../xtask/oracles/native_bond_rdkit.py"
        ))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("oracle stdin")?
        .write_all(&bytes)?;
    let result = child.wait_with_output()?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(
        serde_json::to_vec(&engine.state())?,
        before,
        "encoding is a read-only library operation"
    );
    Ok(())
}
