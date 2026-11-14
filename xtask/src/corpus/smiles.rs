//! Three original pinned notation vectors, not general SMILES or chemical perception.

use bondline_io::smiles::{BondOrder, Hydrogens, SmilesDocument};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{error::Error, sync::Arc};

fn atom(element: u8, aromatic: bool, isotope: Option<u16>, charge: i8, hydrogens: Value) -> Value {
    json!({"element":element,"aromatic":aromatic,"isotope":isotope,"charge":charge,"hydrogens":hydrogens})
}

fn literal(bytes: &[u8]) -> Result<Value, Box<dyn Error>> {
    Ok(match bytes {
        b"[13CH3][CH2][OH]\n" => json!({"notation_end":16,"chain_starts":1,
            "atoms":[atom(6,false,Some(13),0,json!(3)),atom(6,false,None,0,json!(2)),atom(8,false,None,0,json!(1))],
            "bonds":[[0,1,"single",null],[1,2,"single",null]],"rings":[]}),
        b"C%73CC%73.C%74CCC%74\n" => json!({"notation_end":20,"chain_starts":2,
            "atoms":vec![atom(6,false,None,0,json!("implicit"));7],
            "bonds":[[0,1,"single",null],[0,2,"single",73],[1,2,"single",null],[3,4,"single",null],[3,6,"single",74],[4,5,"single",null],[5,6,"single",null]],
            "rings":[{"label":73,"open_range":[1,4],"close_range":[6,9]},{"label":74,"open_range":[11,14],"close_range":[17,20]}]}),
        b"[Na+].[O-][CH2]C(C)C\n" => json!({"notation_end":20,"chain_starts":2,
            "atoms":[atom(11,false,None,1,json!(0)),atom(8,false,None,-1,json!(0)),atom(6,false,None,0,json!(2)),atom(6,false,None,0,json!("implicit")),atom(6,false,None,0,json!("implicit")),atom(6,false,None,0,json!("implicit"))],
            "bonds":[[1,2,"single",null],[2,3,"single",null],[3,4,"single",null],[3,5,"single",null]],"rings":[]}),
        _ => return Err("SMILES corpus vector has no reviewed literal expectations".into()),
    })
}

fn notation(document: &SmilesDocument) -> Value {
    let atoms = document
        .atoms()
        .iter()
        .map(|a| {
            atom(
                a.element.get(),
                a.aromatic,
                a.isotope,
                a.charge,
                match a.hydrogens {
                    Hydrogens::Implicit => json!("implicit"),
                    Hydrogens::Bracket(n) => json!(n),
                },
            )
        })
        .collect::<Vec<_>>();
    let mut edges = document
        .bonds()
        .iter()
        .map(|b| {
            (
                b.atoms[0].min(b.atoms[1]),
                b.atoms[0].max(b.atoms[1]),
                match b.order {
                    BondOrder::Single => "single",
                    BondOrder::Double => "double",
                    BondOrder::Triple => "triple",
                    BondOrder::Aromatic => "aromatic",
                },
                b.ring.as_ref().map(|r| r.label),
            )
        })
        .collect::<Vec<_>>();
    edges.sort_unstable();
    let mut rings = document
        .bonds()
        .iter()
        .filter_map(|b| b.ring.as_ref())
        .collect::<Vec<_>>();
    rings.sort_by_key(|r| r.label);
    let rings=rings.into_iter().map(|r|json!({"label":r.label,"open_range":[r.open_range.start,r.open_range.end],"close_range":[r.close_range.start,r.close_range.end]})).collect::<Vec<_>>();
    json!({"notation_end":document.notation_range().end,"chain_starts":document.chain_starts(),"atoms":atoms,"bonds":edges,"rings":rings})
}

fn compare(bytes: &[u8], actual: &Value) -> Result<(), Box<dyn Error>> {
    let expected = literal(bytes)?;
    if actual != &expected {
        return Err("SMILES notation differs from independent literal graph/H/isotope/charge/ring expectations".into());
    }
    let external = super::python_oracle(
        include_str!("../../../crates/io/tests/smiles_oracle.py"),
        bytes,
    )?;
    if external["rdkit"] != "2026.09.2"
        || external["accepted"] != true
        || external["source_ascii"] != std::str::from_utf8(bytes)?
        || external["source_bytes"] != bytes.len()
        || external["source_sha256"] != format!("{:x}", Sha256::digest(bytes))
        || external["parameters"]
            != json!({"sanitize":false,"removeHs":false,"parseName":false,"allowCXSMILES":false})
        || external["skip_cleanup_exposed"] != false
    {
        return Err("external SMILES source/parser contract differs".into());
    }
    let external_atoms = external["atoms"]
        .as_array()
        .ok_or("external SMILES atoms")?;
    let atoms = expected["atoms"].as_array().ok_or("literal SMILES atoms")?;
    if external_atoms.len() != atoms.len() {
        return Err("external SMILES atom count differs".into());
    }
    for (index, expected) in atoms.iter().enumerate() {
        let actual = external_atoms.get(index).ok_or("external SMILES atom")?;
        let implicit = expected["hydrogens"] == "implicit";
        if actual["element"] != expected["element"]
            || actual["isotope"] != expected["isotope"].as_u64().unwrap_or(0)
            || actual["charge"] != expected["charge"]
            || actual["explicit_h"]
                != if implicit {
                    json!(0)
                } else {
                    expected["hydrogens"].clone()
                }
            || actual["no_implicit"] != !implicit
            || actual["needs_update_cache"] != true
        {
            return Err("external SMILES element/isotope/charge/bracket-H differs".into());
        }
    }
    let bonds = expected["bonds"]
        .as_array()
        .ok_or("literal SMILES bonds")?
        .iter()
        .map(|b| json!([b[0], b[1], b[2].as_str().map(str::to_ascii_uppercase)]))
        .collect::<Vec<_>>();
    if external["bonds"] != json!(bonds) {
        return Err("external SMILES topology differs".into());
    }
    Ok(())
}

pub(super) fn check(bytes: &[u8]) -> Result<(), Box<dyn Error>> {
    if bytes.len() > bondline_io::smiles::MAX_SMILES_BYTES {
        return Err("SMILES corpus input exceeds the 64 KiB notation bound".into());
    }
    let document = SmilesDocument::parse(Arc::from(bytes))?;
    if document.as_bytes() != bytes || document.notation_range().start != 0 {
        return Err("SMILES source byte loss".into());
    }
    compare(document.as_bytes(), &notation(&document))
}

#[cfg(test)]
mod tests {
    use super::*;
    const CHAIN: &[u8] =
        include_bytes!("../../../crates/io/tests/fixtures/smiles/isotopic-chain.smi");
    const RINGS: &[u8] =
        include_bytes!("../../../crates/io/tests/fixtures/smiles/two-ring-labels.smi");
    const SALT: &[u8] = include_bytes!("../../../crates/io/tests/fixtures/smiles/salt-branch.smi");

    #[test]
    fn original_exact_sources_and_literal_notation_match_the_independent_consumer()
    -> Result<(), Box<dyn Error>> {
        for bytes in [CHAIN, RINGS, SALT] {
            check(bytes)?;
            let document = SmilesDocument::parse(Arc::from(bytes))?;
            assert_eq!(document.as_bytes(), bytes);
            assert_eq!(notation(&document), literal(bytes)?);
        }
        Ok(())
    }

    #[test]
    fn corpus_observes_literal_aromatic_markers_without_promoting_element_case()
    -> Result<(), Box<dyn Error>> {
        for (source, expected) in [
            (CHAIN, false),
            (b"[13cH3][CH2][OH]\n".as_slice(), true),
            (b"C:C\n".as_slice(), false),
        ] {
            let document = SmilesDocument::parse(Arc::from(source))?;
            assert_eq!(notation(&document)["atoms"][0]["aromatic"], expected);
        }
        let document = SmilesDocument::parse(Arc::from(CHAIN))?;
        let mut wrong = notation(&document);
        wrong["atoms"][0]["aromatic"] = json!(true);
        assert!(compare(CHAIN, &wrong).is_err());
        Ok(())
    }

    #[test]
    fn corpus_rejects_changed_raw_source_graph_h_charge_isotope_and_ring_identity()
    -> Result<(), Box<dyn Error>> {
        for (bytes, changes) in [
            (
                CHAIN,
                vec![
                    ("/notation_end", json!(17)),
                    ("/chain_starts", json!(2)),
                    ("/atoms/0/element", json!(7)),
                    ("/atoms/0/aromatic", json!(true)),
                    ("/atoms/0/isotope", json!(12)),
                    ("/atoms/0/charge", json!(1)),
                    ("/atoms/1/hydrogens", json!(0)),
                    ("/bonds/0/1", json!(2)),
                    ("/bonds/0/2", json!("double")),
                ],
            ),
            (
                RINGS,
                vec![
                    ("/atoms/0/hydrogens", json!(0)),
                    ("/bonds/1/3", json!(74)),
                    ("/rings/0/label", json!(74)),
                    ("/rings/1/close_range/0", json!(16)),
                ],
            ),
            (
                SALT,
                vec![
                    ("/atoms/0/charge", json!(0)),
                    ("/atoms/1/charge", json!(0)),
                    ("/atoms/0/hydrogens", json!("implicit")),
                ],
            ),
        ] {
            let document = SmilesDocument::parse(Arc::from(bytes))?;
            for (pointer, value) in changes {
                let mut changed = notation(&document);
                *changed.pointer_mut(pointer).ok_or("literal field")? = value;
                assert!(compare(bytes, &changed).is_err(), "{pointer}");
            }
        }
        for bytes in [
            b"[13CH3][CH2][SH]\n".as_slice(),
            b"[13CH3][CH2][OH]".as_slice(),
            b"[13CH3][CH2][OH]\r\n".as_slice(),
        ] {
            assert!(SmilesDocument::parse(Arc::from(bytes)).is_ok());
            assert!(check(bytes).is_err());
        }
        assert!(check(b"c1ccccc1\n").is_err());
        assert!(check(&vec![b'C'; bondline_io::smiles::MAX_SMILES_BYTES + 1]).is_err());
        Ok(())
    }
}
