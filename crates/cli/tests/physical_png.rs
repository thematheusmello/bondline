use std::{path::Path, process::Command};

#[test]
fn registered_export_writes_a_new_physical_page_without_saving_the_drawing()
-> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(std::env::var_os("BONDLINE_PYTHON").ok_or("pinned Pillow")?)
        .arg("-B")
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/physical_png.py"))
        .arg(env!("CARGO_BIN_EXE_bondline-cli"))
        .output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}
