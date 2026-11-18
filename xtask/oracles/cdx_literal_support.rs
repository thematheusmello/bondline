//! Original CDX bytes and independent struct facts; test-only external reader.
use serde_json::{Value, json};
use std::{
    io::Write,
    process::{Command, Stdio},
};
pub type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
pub const METADATA49: &str =
    include_str!("../../crates/io/tests/fixtures/cdx-literal-metadata/old49.json");
pub fn source(name: &str) -> &'static [u8] {
    let raw: &[u8] = match name {
        "absent_implied" => {
            include_bytes!("../../crates/io/tests/fixtures/cdx-literal-metadata/absent_implied.cdx")
        }
        "scalar_zero_implied" => include_bytes!(
            "../../crates/io/tests/fixtures/cdx-literal-metadata/scalar_zero_implied.cdx"
        ),
        "zero_present_explicit" => include_bytes!(
            "../../crates/io/tests/fixtures/cdx-literal-metadata/zero_present_explicit.cdx"
        ),
        "methylammonium_explicit" => include_bytes!(
            "../../crates/io/tests/fixtures/cdx-literal-metadata/methylammonium_explicit.cdx"
        ),
        "methylammonium_extended" => include_bytes!(
            "../../crates/io/tests/fixtures/cdx-literal-metadata/methylammonium_extended.cdx"
        ),
        "methylammonium_embedded" => include_bytes!(
            "../../crates/io/tests/fixtures/cdx-literal-metadata/methylammonium_embedded.cdx"
        ),
        "methyl_doublet" => {
            include_bytes!("../../crates/io/tests/fixtures/cdx-literal-metadata/methyl_doublet.cdx")
        }
        "singlet" => {
            include_bytes!("../../crates/io/tests/fixtures/cdx-literal-metadata/singlet.cdx")
        }
        "triplet" => {
            include_bytes!("../../crates/io/tests/fixtures/cdx-literal-metadata/triplet.cdx")
        }
        "extrema" => {
            include_bytes!("../../crates/io/tests/fixtures/cdx-literal-metadata/extrema.cdx")
        }
        "reordered" => {
            include_bytes!("../../crates/io/tests/fixtures/cdx-literal-metadata/reordered.cdx")
        }
        "default_carbon_fields" => include_bytes!(
            "../../crates/io/tests/fixtures/cdx-literal-metadata/default_carbon_fields.cdx"
        ),
        "deuteron" => {
            include_bytes!("../../crates/io/tests/fixtures/cdx-literal-metadata/deuteron.cdx")
        }
        "charge4_low_one" => include_bytes!(
            "../../crates/io/tests/fixtures/cdx-literal-metadata/charge4_low_one.cdx"
        ),
        "charge4_high_one" => include_bytes!(
            "../../crates/io/tests/fixtures/cdx-literal-metadata/charge4_high_one.cdx"
        ),
        "charge4_high_minus_one" => include_bytes!(
            "../../crates/io/tests/fixtures/cdx-literal-metadata/charge4_high_minus_one.cdx"
        ),
        "charge4_low_noise_max" => include_bytes!(
            "../../crates/io/tests/fixtures/cdx-literal-metadata/charge4_low_noise_max.cdx"
        ),
        "charge4_min" => {
            include_bytes!("../../crates/io/tests/fixtures/cdx-literal-metadata/charge4_min.cdx")
        }
        _ => &[],
    };
    assert!(!raw.is_empty(), "declared original literal CDX source");
    raw
}
pub fn facts(name: &str) -> Value {
    let all: Value = serde_json::from_str(include_str!(
        "../../crates/io/tests/fixtures/cdx-literal-metadata/literal-facts.json"
    ))
    .unwrap_or(Value::Null);
    assert!(all.is_object(), "frozen literal CDX facts object");
    let row = all.get(name).cloned().unwrap_or(Value::Null);
    assert!(row.is_object(), "declared original literal CDX facts");
    row
}
/// Complete expected A4 document, projected only from independently frozen facts.
pub fn model(name: &str) -> Result<Value> {
    let facts = facts(name);
    let all_atoms = facts["atoms"].as_array().ok_or("frozen CDX atoms")?;
    let all_bonds = facts["bonds"].as_array().ok_or("frozen CDX bonds")?;
    let mut fragments = Vec::new();
    for fragment in facts["fragments"]
        .as_array()
        .ok_or("frozen CDX fragments")?
    {
        let mut atoms = Vec::new();
        for id in fragment["atom_ids"].as_array().ok_or("frozen atom ids")? {
            let fact = all_atoms
                .iter()
                .find(|fact| fact["model_id"] == *id)
                .ok_or("frozen atom identity")?;
            let mut atom = serde_json::Map::new();
            atom.insert("id".into(), id.clone());
            for key in [
                "element",
                "position",
                "isotope",
                "charge",
                "radical",
                "explicit_hydrogens",
            ] {
                atom.insert(
                    key.into(),
                    fact.get(key).ok_or("frozen atom field")?.clone(),
                );
            }
            atoms.push(Value::Object(atom));
        }
        let mut bonds = Vec::new();
        for id in fragment["bond_ids"].as_array().ok_or("frozen bond ids")? {
            let fact = all_bonds
                .iter()
                .find(|fact| fact["model_id"] == *id)
                .ok_or("frozen bond identity")?;
            let mut bond = serde_json::Map::new();
            bond.insert("id".into(), id.clone());
            for key in ["begin", "end", "order"] {
                bond.insert(
                    key.into(),
                    fact.get(key).ok_or("frozen bond field")?.clone(),
                );
            }
            bonds.push(Value::Object(bond));
        }
        fragments.push(json!({"id":fragment.get("model_id").ok_or("frozen fragment identity")?,"atoms":atoms,"bonds":bonds}));
    }
    Ok(
        json!({"pages":[{"id":1,"size":{"width_pt":210.0*72.0/25.4,"height_pt":297.0*72.0/25.4},"fragments":fragments}]}),
    )
}
pub fn read(raw: &[u8]) -> Result<Value> {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child = Command::new(python)
        .args([
            "-X",
            "utf8",
            "-B",
            "-c",
            include_str!("../../crates/io/tests/fixtures/cdx-literal-metadata/literal_oracle.py"),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("literal CDX reader stdin")?
        .write_all(raw)?;
    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(format!(
            "strict independent literal CDX reader: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(serde_json::from_slice(&output.stdout)?)
}
