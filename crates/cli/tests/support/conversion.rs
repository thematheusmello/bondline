use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};

pub fn params(revision: u64, input: &Path, output: &Path) -> Value {
    json!({"expected_revision":revision,"input_path":input,"output_path":output,
        "bond_length_pt":48.0,"origin_x_pt":72.0,"origin_y_pt":72.0,
        "page_width_pt":210.0*72.0/25.4,"page_height_pt":297.0*72.0/25.4})
}

pub fn extend(revision: u64) -> Value {
    json!({"expected_revision":revision,"page_id":1,"fragment_id":1,"begin_atom_id":2,
        "end_x_pt":160.0,"end_y_pt":112.0,"hydrogen_policy":"rectify_terminal_carbon"})
}

pub fn assert_conversion(
    result: &Value,
    input: &Path,
    output: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(result["changed"], false);
    assert_eq!(result["revision"], 0);
    assert_eq!(
        result["conversion"],
        json!({"profile":"neutral_carbon_path_v1",
        "input_revision":0,"session_generation":0,"input_path":fs::canonicalize(input)?,
        "input_byte_length":3,"atom_count":2,"bond_count":1,
        "output":{"path":fs::canonicalize(output)?,"byte_length":fs::metadata(output)?.len(),
            "directory_synced":result["conversion"]["output"]["directory_synced"]}})
    );
    assert!(result["conversion"]["output"]["directory_synced"].is_boolean());
    assert_eq!(
        result["file"],
        json!({"format":"cdxml","path":null,"dirty":false})
    );
    assert_eq!(result["undo_depth"], 0);
    assert_eq!(result["redo_depth"], 0);
    assert!(result.get("source").is_none());
    assert!(result.get("file_operation").is_none());
    assert_eq!(fs::read(input)?, b"CC\n");
    Ok(())
}

pub fn assert_chain(result: &Value, count: usize) -> Result<(), Box<dyn std::error::Error>> {
    let fragments = result["document"]["pages"][0]["fragments"]
        .as_array()
        .ok_or("fragments")?;
    assert_eq!(fragments.len(), 1);
    let atoms = fragments[0]["atoms"].as_array().ok_or("atoms")?;
    assert_eq!(atoms.len(), count);
    assert_eq!(fragments[0]["id"], 1);
    for (index, atom) in atoms.iter().enumerate() {
        assert_eq!(atom["id"], index + 1);
        assert_eq!(atom["element"], 6);
        assert_eq!(atom["charge"], 0);
        assert_eq!(atom["isotope"], Value::Null);
        assert_eq!(atom["radical"], "none");
        assert_eq!(
            atom["explicit_hydrogens"],
            if index == 0 || index + 1 == count {
                3
            } else {
                2
            }
        );
        let [x, y] = [[72, 72], [120, 72], [160, 112]][index];
        assert_eq!(atom["position"], json!({"x_pt":x as f64,"y_pt":y as f64}));
    }
    assert_eq!(
        fragments[0]["bonds"].as_array().ok_or("bonds")?.len(),
        count - 1
    );
    for (index, bond) in fragments[0]["bonds"]
        .as_array()
        .ok_or("bonds")?
        .iter()
        .enumerate()
    {
        assert_eq!(
            bond,
            &json!({"id":index+1,"begin":index+1,"end":index+2,"order":"single"})
        );
    }
    Ok(())
}

pub fn independent(path: &Path, count: usize) -> Result<(), Box<dyn std::error::Error>> {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let output = Command::new(python)
        .arg("-B")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../commands/tests/smiles_conversion_oracle.py"
        ))
        .arg(count.to_string())
        .stdin(fs::File::open(path)?)
        .output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}
