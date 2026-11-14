use serde_json::{Value, json};
use std::{fs, path::Path};

pub const SOURCE:&[u8]=br#"<?xml version="1.0" encoding="UTF-8"?><CDXML CreationProgram="Bondline 0.1.0"><page id="1" DrawingSpace="Poster" Width="612" Height="792"><fragment id="2"><n id="3" NodeType="Element" Element="6" Charge="0" Radical="None" NumHydrogens="4"/></fragment><fragment id="4"><n id="5" NodeType="Element" Element="6" Charge="0" Radical="None" Isotope="13" NumHydrogens="3"/><n id="6" NodeType="Element" Element="8" Charge="0" Radical="None" NumHydrogens="1"/><b id="7" B="5" E="6" Order="1"/></fragment></page></CDXML>"#;
pub fn params(revision: u64, output: &Path) -> Value {
    json!({"expected_revision":revision,"page_id":1,"fragment_id":2,"output_path":output})
}
pub fn assert_export(result: &Value, output: &Path) -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(result["changed"], false);
    assert_eq!(
        result["smiles_export"],
        json!({"profile":"neutral_explicit_h_fragment_v1","scope":"selected_fragment_chemical_graph","not_encoded":["geometry","document_context"],"input_revision":1,"session_generation":1,"page_id":1,"fragment_id":2,"atom_count":2,"bond_count":1,"output":{"path":fs::canonicalize(output)?,"byte_length":13,"directory_synced":result["smiles_export"]["output"]["directory_synced"]}})
    );
    assert!(result["smiles_export"]["output"]["directory_synced"].is_boolean());
    assert_eq!(fs::read(output)?, b"[13CH3][OH1]\n");
    assert_eq!(result["revision"], 1);
    assert_eq!(result["file"]["dirty"], false);
    assert_eq!(result["source"]["modified"], false);
    assert_eq!(result["undo_depth"], 0);
    assert_eq!(result["redo_depth"], 0);
    assert_eq!(
        result["document"]["pages"][0]["fragments"]
            .as_array()
            .ok_or("fragments")?
            .len(),
        2
    );
    assert!(result.get("file_operation").is_none());
    assert!(result.get("conversion").is_none());
    Ok(())
}
pub fn consume(output: &Path) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write as _;
    use std::process::{Command, Stdio};
    let file = fs::read(output)?;
    assert_eq!(file, b"[13CH3][OH1]\n");
    let notation = file.strip_suffix(b"\n").ok_or("single LF")?;
    assert!(notation.is_ascii() && !notation.iter().any(u8::is_ascii_whitespace));
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child = Command::new(python)
        .arg("-B")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../io/tests/smiles_write_oracle.py"
        ))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child.stdin.take().ok_or("stdin")?.write_all(notation)?;
    let result = child.wait_with_output()?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let value: Value = serde_json::from_slice(&result.stdout)?;
    assert_eq!(value["notation"]["source_ascii"], "[13CH3][OH1]");
    assert_eq!(value["notation"]["rdkit"], "2026.09.2");
    assert_eq!(value["notation"]["atoms"][0]["isotope"], 13);
    assert_eq!(value["notation"]["atoms"][0]["explicit_h"], 3);
    assert_eq!(value["notation"]["atoms"][1]["element"], 8);
    assert_eq!(value["notation"]["atoms"][1]["explicit_h"], 1);
    assert_eq!(value["notation"]["bonds"], json!([[0, 1, "SINGLE"]]));
    assert_eq!(value["meaning"]["smiles"], "[13CH3]O");
    assert_eq!(value["meaning"]["radicals"], json!([0, 0]));
    assert_eq!(value["meaning"]["conformers"], 0);
    Ok(())
}
