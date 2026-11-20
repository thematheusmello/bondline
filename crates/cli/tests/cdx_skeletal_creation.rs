//! Create, save and reopen labels against independently constructed binary molecules and RDKit.
use std::{path::Path, process::Command};

#[test]
fn created_cdx_atom_labels_match_independent_files_through_history()
-> Result<(), Box<dyn std::error::Error>> {
    let output =
        Command::new(std::env::var_os("BONDLINE_PYTHON").ok_or("pinned RDKit interpreter")?)
            .arg("-B")
            .arg(
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../xtask/oracles/cdx_skeletal_creation.py"),
            )
            .arg("--verify")
            .arg(env!("CARGO_BIN_EXE_bondline-cli"))
            .output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}
