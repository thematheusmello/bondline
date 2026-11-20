//! External RDKit graphs and real files through the shipped command-line application.
use std::process::Command;

#[test]
fn v2000_v3000_percentage_tables_survive_selection_and_exact_reopen()
-> Result<(), Box<dyn std::error::Error>> {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let output = Command::new(python)
        .args([
            "-B",
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../xtask/oracles/elemental_mol.py"
            ),
            env!("CARGO_BIN_EXE_bondline-cli"),
        ])
        .output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}
