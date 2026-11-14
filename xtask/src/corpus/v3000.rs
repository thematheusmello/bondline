//! The original pinned V3000 field vector, not general chemistry or application parity.

use bondline_io::mol::v3000::{MolFile, Value as RawValue};
use serde_json::{Value, json};
use std::error::Error;

fn raw_view(document: &MolFile<'_>) -> Value {
    let atoms: Vec<_> = document
        .atoms()
        .iter()
        .map(|atom| {
            let attributes: Vec<_> = atom
                .attributes
                .iter()
                .map(|attribute| {
                    let value = match &attribute.value {
                        RawValue::Text(text) => json!({"text":text}),
                        RawValue::List(values) => json!({"list":values}),
                    };
                    json!({"key":attribute.key,"value":value})
                })
                .collect();
            json!({"source_index":atom.source_index,"symbol":atom.symbol,
            "element":atom.element.map(bondline_model::Element::get),
            "position":[atom.x,atom.y,atom.z],"mapping":atom.mapping,
            "charge":atom.charge,"radical":atom.radical,
            "isotope":atom.isotope.map(std::num::NonZeroU16::get),
            "declared_valence":atom.declared_valence,"attributes":attributes})
        })
        .collect();
    let bonds: Vec<_> = document
        .bonds()
        .iter()
        .map(|bond| {
            json!({"source_index":bond.source_index,"begin":bond.begin,"end":bond.end,
            "order":bond.order,"attribute_count":bond.attributes.len()})
        })
        .collect();
    json!({"header":document.header(),"atoms":atoms,"bonds":bonds})
}

fn compare(bytes: &[u8], raw: &Value, fragment: &Value) -> Result<(), Box<dyn Error>> {
    // Independently read the original, fixed subset. Decimal supplies the declared
    // 24pt/unit, origin(0,0), inverted-Y transform; this does not infer chemical units.
    let reference = super::python_oracle(
        r#"
import json,sys
from decimal import Decimal
source=sys.stdin.buffer.read(345)
assert len(source)==344, 'original V3000 vector byte bound'
lines=source.decode('ascii').splitlines()
assert len(lines)==15 and lines[3][33:39]==' V3000', 'explicit V3000 signature'
assert lines[4:7]==['M  V30 BEGIN CTAB','M  V30 COUNTS 2 1 0 0 0','M  V30 BEGIN ATOM']
assert lines[9:11]==['M  V30 END ATOM','M  V30 BEGIN BOND']
assert lines[12:]==['M  V30 END BOND','M  V30 END CTAB','M  END']
raw_atoms=[]
atoms=[]
for index,(line,source_id,x) in enumerate(zip(lines[7:9],[10,40],['-1.25','0.25']),1):
    assert line.startswith('M  V30 ')
    fields=line[7:].split()
    assert fields==[str(source_id),'C',x,'0.5','0','0','VAL=4'], 'original atom fields'
    point=[Decimal(value) for value in fields[2:5]]
    raw_atoms.append({'source_index':source_id,'symbol':'C','element':6,
        'position':[float(value) for value in point],'mapping':0,'charge':0,
        'radical':'none','isotope':None,'declared_valence':4,
        'attributes':[{'key':'VAL','value':{'text':'4'}}]})
    atoms.append({'id':index,'element':6,
        'position':{'x_pt':float(point[0]*Decimal(24)), 'y_pt':float(-point[1]*Decimal(24))},
        'isotope':None,'charge':0,'radical':'none','explicit_hydrogens':4-1})
assert lines[11]=='M  V30 7 1 10 40', 'original source bond ID/order/endpoints'
result={'raw':{'header':lines[:3],'atoms':raw_atoms,
    'bonds':[{'source_index':7,'begin':10,'end':40,'order':1,'attribute_count':0}]},
    'fragment':{'id':1,'atoms':atoms,'bonds':[{'id':1,'begin':1,'end':2,'order':'single'}]}}
sys.stdout.buffer.write((json.dumps(result,separators=(',',':'),allow_nan=False)+'\n').encode('ascii'))
"#,
        bytes,
    )?;
    if reference != json!({"raw":raw,"fragment":fragment}) {
        return Err(
            "MOL V3000 raw/model projection differs from independent literal/Decimal fields".into(),
        );
    }
    let external: Value = serde_json::from_slice(&super::python_output(
        include_str!("../../../crates/io/tests/v3000_oracle.py"),
        &["describe"],
        bytes,
    )?)?;
    let atoms: Vec<_> = reference["raw"]["atoms"]
        .as_array()
        .ok_or("V3000 reference atoms")?
        .iter()
        .map(|atom| {
            json!({"element":6,"isotope":0,"charge":0,"explicit_h":3,
            "query":null,"radical_electrons":0,"position":atom["position"],
            "no_implicit":true,"map":0})
        })
        .collect();
    let expected = json!({"rdkit":"2026.09.2",
        "source_sha256":"1867a5904e742030431b523a31167951ff998007a89b9c3d94ec88cf7d4c4380",
        "source_bytes":344,"atoms":atoms,
        "bonds":[{"begin":0,"end":1,"order":1.0,"direction":"NONE","stereo":"STEREONONE"}]});
    if external != expected {
        return Err("MOL V3000 pinned original graph differs from external RDKit".into());
    }
    Ok(())
}

pub(super) fn check(bytes: &[u8]) -> Result<(), Box<dyn Error>> {
    let document = MolFile::parse(bytes)?;
    if document.as_bytes() != bytes {
        return Err("MOL V3000 source byte loss".into());
    }
    let frame = bondline_io::mol::MolTransform::new(24.0, bondline_model::Point2::new(0.0, 0.0)?)?;
    let fragment = serde_json::to_value(document.to_fragment(frame)?)?;
    compare(bytes, &raw_view(&document), &fragment)
}

#[cfg(test)]
mod tests {
    use super::*;
    const SOURCE: &[u8] =
        include_bytes!("../../../crates/io/tests/fixtures/v3000/noncontiguous-cc.mol");

    #[test]
    fn original_source_sparse_ids_and_dense_projection_match_independent_readers()
    -> Result<(), Box<dyn Error>> {
        check(SOURCE)?;
        let raw = MolFile::parse(SOURCE)?;
        assert_eq!(raw.as_bytes(), SOURCE);
        assert_eq!(
            raw.atoms()
                .iter()
                .map(|atom| atom.source_index)
                .collect::<Vec<_>>(),
            [10, 40]
        );
        assert_eq!(raw.bonds().first().ok_or("bond")?.source_index, 7);
        let frame =
            bondline_io::mol::MolTransform::new(24.0, bondline_model::Point2::new(0.0, 0.0)?)?;
        let projected = serde_json::to_value(raw.to_fragment(frame)?)?;
        assert_eq!(
            projected["atoms"][0]["position"],
            json!({"x_pt":-30.0,"y_pt":-12.0})
        );
        assert_eq!(
            projected["atoms"][1]["position"],
            json!({"x_pt":6.0,"y_pt":-12.0})
        );
        Ok(())
    }

    #[test]
    fn independent_v3000_comparison_rejects_changed_raw_model_ids_metadata_and_coordinates()
    -> Result<(), Box<dyn Error>> {
        let document = MolFile::parse(SOURCE)?;
        let raw = raw_view(&document);
        let frame =
            bondline_io::mol::MolTransform::new(24.0, bondline_model::Point2::new(0.0, 0.0)?)?;
        let fragment = serde_json::to_value(document.to_fragment(frame)?)?;
        for (path, value) in [
            ("/atoms/0/source_index", json!(1)),
            ("/bonds/0/source_index", json!(1)),
            ("/bonds/0/end", json!(10)),
            ("/atoms/0/position/0", json!(-1.2)),
            ("/atoms/0/attributes/0/value/text", json!("3")),
        ] {
            let mut changed = raw.clone();
            *changed.pointer_mut(path).ok_or("raw mutation")? = value;
            assert!(compare(SOURCE, &changed, &fragment).is_err(), "raw {path}");
        }
        for (path, value) in [
            ("/atoms/0/id", json!(10)),
            ("/atoms/0/element", json!(7)),
            ("/atoms/0/explicit_hydrogens", json!(2)),
            ("/atoms/0/radical", json!("doublet")),
            ("/atoms/0/position/y_pt", json!(12.0)),
            ("/bonds/0/order", json!("double")),
        ] {
            let mut changed = fragment.clone();
            *changed.pointer_mut(path).ok_or("model mutation")? = value;
            assert!(compare(SOURCE, &raw, &changed).is_err(), "model {path}");
        }
        Ok(())
    }

    #[test]
    fn checker_refuses_truncated_extended_wrong_version_and_different_valid_vectors()
    -> Result<(), Box<dyn Error>> {
        let text = std::str::from_utf8(SOURCE)?;
        for changed in [
            text.replace("M  V30 END CTAB\n", ""),
            text.replace("V3000", "V2000"),
            text.replace("VAL=4", "VAL=4 CFG=1"),
            text.replace("7 1 10 40", "7 2 10 40"),
        ] {
            assert!(check(changed.as_bytes()).is_err());
        }
        assert!(
            check(include_bytes!(
                "../../../crates/io/tests/fixtures/mol/rdkit-CH3.mol"
            ))
            .is_err()
        );
        assert!(super::super::check_mol(SOURCE).is_err());
        Ok(())
    }
}
