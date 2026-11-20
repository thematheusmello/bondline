//! Compare actual Open, Analysis, selection and Save As with an independent MOL reader.
use std::{path::Path, process::Command};

#[test]
fn mol_formulas_match_rdkit_and_survive_exact_save_and_reopen()
-> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        root.join(".cache/oracles/format-python/bin/python")
            .into_os_string()
    });
    let output = Command::new(python)
        .arg("-B")
        .arg(root.join("xtask/oracles/mol_formula.py"))
        .arg(env!("CARGO_BIN_EXE_bondline-cli"))
        .output()?;
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}
