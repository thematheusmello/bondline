//! Original predeclared second-duplicate splices; no Bondline parser/writer expectations.

use super::sdf_support;
use serde_json::json;
use std::{fs, path::Path};

pub fn source(mixed: bool) -> &'static [u8] {
    if mixed {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf"
        ))
    } else {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../io/tests/fixtures/sdf/ordered-fields-v2000.sdf"
        ))
    }
}

pub fn field_index(mixed: bool) -> usize {
    if mixed { 2 } else { 5 }
}

pub fn edited_bytes(mixed: bool) -> Vec<u8> {
    let source = source(mixed);
    let (start, end) = if mixed { (378, 385) } else { (383, 387) };
    [&source[..start], b"revised\nmultiline\n", &source[end..]].concat()
}

pub fn assert_edited_file(path: &Path, mixed: bool) -> Result<(), Box<dyn std::error::Error>> {
    let bytes = fs::read(path)?;
    assert_eq!(bytes, edited_bytes(mixed));
    if mixed {
        assert_eq!(bytes.len(), 788);
        assert_eq!(&bytes[473..], &source(true)[462..]);
        let v3 = sdf_support::describe_source("v3000_oracle.py", &bytes[..344])?;
        let v2 = sdf_support::describe_source("mol_oracle.py", &bytes[473..755])?;
        assert_eq!(v3["source_bytes"], 344);
        assert_eq!(v3["atoms"].as_array().ok_or("V3000 atoms")?.len(), 2);
        assert_eq!(v3["bonds"].as_array().ok_or("V3000 bonds")?.len(), 1);
        assert_eq!(v2["smiles"], "CC");
        assert_eq!(v2["formula"], "C2H6");
    } else {
        assert_eq!(bytes.len(), 704);
        let raw = sdf_support::describe_source("sdf_oracle.py", &bytes)?;
        assert_eq!(
            raw["source_sha256"],
            "a38749176c76066c2326853f55974f4d450c34bb03fa4f25c9adbf1b2477358b"
        );
        assert_eq!(raw["records"][0]["range"], json!([0, 438]));
        assert_eq!(raw["records"][1]["range"], json!([438, 704]));
        assert_eq!(
            raw["records"][0]["fields"][4]["value_lines"],
            json!(["one"])
        );
        assert_eq!(
            raw["records"][0]["fields"][5]["value_lines"],
            json!(["revised", "multiline"])
        );
        assert_eq!(raw["records"][0]["fields"][3]["dt_number"], "0009");
        assert_eq!(raw["records"][0]["rdkit"]["smiles"], "CC");
        assert_eq!(raw["records"][1]["rdkit"]["smiles"], "CC");
        assert_eq!(
            raw["rdkit_records"]
                .as_array()
                .ok_or("supplier diagnostics")?
                .len(),
            1
        );
    }
    Ok(())
}
