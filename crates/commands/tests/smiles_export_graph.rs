use bondline_commands::Engine;
use bondline_io::StoredDocument;
use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Element, Fragment, FragmentId, PageId,
};
use serde_json::json;
use std::{fs, num::NonZeroU16, process::Command};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse the existing isolated test directory")]
mod support;

fn atom_id(isotope: u16) -> Result<AtomId, Box<dyn std::error::Error>> {
    AtomId::new(u32::from(isotope) * 10 + 7).ok_or("atom ID".into())
}

fn drawing() -> Result<StoredDocument, Box<dyn std::error::Error>> {
    use BondOrder::{Double as D, Single as S, Triple as T};
    // Isotopes distinguish authored vertices across independent parser traversal orders.
    // Positive isotope labels do not assert that these nuclei exist physically.
    let mut atoms = Vec::new();
    for (isotope, element, hydrogens) in [
        (11, 6, 1),
        (12, 6, 1),
        (13, 6, 1),
        (14, 6, 1),
        (15, 6, 1),
        (16, 6, 1),
        (17, 6, 1),
        (18, 6, 1),
        (23, 6, 2),
        (0, 1, 0),
        (2, 1, 0),
        (41, 6, 1),
        (42, 6, 1),
        (51, 6, 1),
        (52, 6, 3),
        (53, 8, 1),
        (54, 9, 0),
    ] {
        let mut atom = Atom::new(atom_id(isotope)?, Element::try_from(element)?, None);
        atom.isotope = NonZeroU16::new(isotope);
        atom.explicit_hydrogens = Some(hydrogens);
        atoms.push(atom);
    }
    atoms.reverse();
    let mut bonds = Vec::new();
    for (index, (left, right, order)) in [
        (11, 12, D),
        (12, 13, S),
        (13, 14, D),
        (14, 15, S),
        (15, 16, D),
        (16, 17, S),
        (17, 18, D),
        (18, 11, S),
        (23, 0, S),
        (23, 2, S),
        (41, 42, T),
        (51, 52, S),
        (51, 53, S),
        (51, 54, S),
    ]
    .into_iter()
    .enumerate()
    {
        let (left, right) = if index % 2 == 0 {
            (right, left)
        } else {
            (left, right)
        };
        bonds.push(Bond {
            double_position: bondline_model::DoublePosition::Unspecified,
            id: BondId::new(u32::try_from(index * 7 + 3)?).ok_or("bond ID")?,
            begin: atom_id(left)?,
            end: atom_id(right)?,
            order,
        });
    }
    bonds.reverse();
    let selected = Fragment::new(FragmentId::new(42).ok_or("selected ID")?, atoms, bonds)?;
    let mut decoy = Atom::new(
        AtomId::new(1).ok_or("decoy atom")?,
        Element::try_from(6)?,
        None,
    );
    decoy.isotope = NonZeroU16::new(99);
    decoy.explicit_hydrogens = Some(4);
    let first = Fragment::new(FragmentId::new(1).ok_or("first ID")?, vec![decoy], vec![])?;
    let mut document = StoredDocument::default();
    document
        .model_mut()
        .append_fragments(PageId::new(1).ok_or("page")?, vec![first, selected])?;
    Ok(document)
}

#[test]
fn selected_export_preserves_full_keyed_graph_without_requiring_drawable_geometry()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = support::FileTestDirectory::new()?;
    let output = directory.path("selected-graph-λ.smi");
    let document = drawing()?;
    assert!(document.is_fully_mapped());
    let mut engine = Engine::from_document(document);
    let owner = engine.content().clone();
    let before = serde_json::to_value(engine.state())?;
    let result = serde_json::to_value(engine.dispatch(
        "file.export_smiles_fragment",
        json!({
            "expected_revision":0,"page_id":1,"fragment_id":42,"output_path":output,
        }),
    )?)?;
    assert_eq!(engine.content(), &owner);
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert_eq!(result["changed"], false);
    let receipt = &result["smiles_export"];
    assert_eq!(receipt["page_id"], 1);
    assert_eq!(receipt["fragment_id"], 42);
    assert_eq!(receipt["atom_count"], 17);
    assert_eq!(receipt["bond_count"], 14);
    assert_eq!(
        receipt["not_encoded"],
        json!(["geometry", "document_context"])
    );
    assert_eq!(
        receipt["output"]["byte_length"],
        fs::metadata(&output)?.len()
    );
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let external = Command::new(python)
        .arg("-B")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/smiles_export_graph_oracle.py"
        ))
        .arg(&output)
        .output()?;
    assert!(
        external.status.success(),
        "{}",
        String::from_utf8_lossy(&external.stderr)
    );
    println!("{}", String::from_utf8_lossy(&external.stdout));
    assert_eq!(engine.content(), &owner);
    assert_eq!(serde_json::to_value(engine.state())?, before);
    Ok(())
}
