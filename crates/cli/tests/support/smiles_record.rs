use serde_json::{Value, json};
use std::{
    io::Write,
    process::{Command, Stdio},
};

pub const SOURCE: &[u8] = b"\r\nCC PRIVATE_FIRST\r\n[13CH2]=[CH1][OH1]\t PRIVATE_SECOND\xff\x00\r\n\tPRIVATE_IGNORED\r\n[NH4+]";

pub fn params(revision: u64, record_id: u32) -> Value {
    json!({"expected_revision":revision,"record_id":record_id})
}

pub fn assert_private(value: &Value) {
    for marker in ["PRIVATE_FIRST", "PRIVATE_SECOND", "PRIVATE_IGNORED"] {
        assert!(!value.to_string().contains(marker));
    }
}

pub fn assert_summary(value: &Value, revision: u64) {
    assert_eq!(value["revision"], revision);
    assert_eq!(value["document"], Value::Null);
    assert_eq!(
        value["smiles_file"],
        json!({"format":"smiles","byte_length":82,"record_count":3})
    );
    assert_eq!(
        value["source"],
        json!({"format":"smiles","byte_length":82,"modified":false})
    );
    assert_eq!(value["file"]["format"], "smiles");
    assert_eq!(value["file"]["dirty"], false);
    assert_eq!(value["undo_depth"], 0);
    assert_eq!(value["redo_depth"], 0);
    assert_private(value);
}

pub fn assert_query(
    value: &Value,
    revision: u64,
    generation: u64,
    record_id: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let record = match record_id {
        2 => json!({
            "input_revision":revision,"session_generation":generation,"record_id":2,"line_number":3,
            "full_range":{"start":20,"end":58},"notation_range":{"start":20,"end":38},
            "separator_range":{"start":38,"end":40},"application_data_range":{"start":40,"end":56},
            "line_ending_range":{"start":56,"end":58},"atom_count":3,"bond_count":2,
            "notation":"[13CH2]=[CH1][OH1]"
        }),
        3 => json!({
            "input_revision":revision,"session_generation":generation,"record_id":3,"line_number":5,
            "full_range":{"start":76,"end":82},"notation_range":{"start":76,"end":82},
            "separator_range":null,"application_data_range":null,
            "line_ending_range":{"start":82,"end":82},"atom_count":1,"bond_count":0,
            "notation":"[NH4+]"
        }),
        _ => return Err("unsupported independent test record".into()),
    };
    assert_eq!(value["smiles_record"], record);
    assert_eq!(
        value["smiles_record"]
            .as_object()
            .ok_or("record object")?
            .len(),
        12
    );
    assert!(serde_json::to_vec(&value["smiles_record"])?.len() <= 128 * 1024);
    assert_summary(value, revision);
    assert_eq!(value["changed"], false);
    Ok(())
}

pub fn consume_notation(value: &Value) -> Result<(), Box<dyn std::error::Error>> {
    let notation = value["smiles_record"]["notation"]
        .as_str()
        .ok_or("exact notation")?;
    assert_eq!(notation, "[13CH2]=[CH1][OH1]");
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut process = Command::new(python)
        .arg("-B")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../io/tests/smiles_oracle.py"
        ))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    process
        .stdin
        .take()
        .ok_or("oracle stdin")?
        .write_all(notation.as_bytes())?;
    let output = process.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let actual: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(actual["rdkit"], "2026.09.2");
    assert_eq!(actual["accepted"], true);
    assert_eq!(actual["source_ascii"], notation);
    assert_eq!(actual["source_bytes"], 18);
    let atoms = actual["atoms"].as_array().ok_or("external atoms")?;
    assert_eq!(atoms.len(), 3);
    for (atom, (element, isotope, h)) in atoms.iter().zip([(6, 13, 2), (6, 0, 1), (8, 0, 1)]) {
        assert_eq!(atom["element"], element);
        assert_eq!(atom["isotope"], isotope);
        assert_eq!(atom["explicit_h"], h);
        assert_eq!(atom["charge"], 0);
    }
    assert_eq!(actual["bonds"], json!([[0, 1, "DOUBLE"], [1, 2, "SINGLE"]]));
    Ok(())
}
