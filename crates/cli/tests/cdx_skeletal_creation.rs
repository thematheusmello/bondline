//! Independently constructed binary molecules must analyze and paint before they are editing targets.
use std::{path::Path, process::Command};

#[test]
fn hidden_atom_label_witnesses_open_in_bondline_and_rdkit() -> Result<(), Box<dyn std::error::Error>>
{
    let output =
        Command::new(std::env::var_os("BONDLINE_PYTHON").ok_or("pinned RDKit interpreter")?)
            .arg("-B")
            .arg(
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../xtask/oracles/cdx_skeletal_creation.py"),
            )
            .arg("--check-witnesses")
            .arg(env!("CARGO_BIN_EXE_bondline-cli"))
            .output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}
