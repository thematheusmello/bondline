use bondline_chem::{CompositionError, FormulaIsotopes};
use bondline_io::smiles::SmilesDocument;
use bondline_render::Cancellation;
use std::sync::Arc;

type Test = Result<(), Box<dyn std::error::Error>>;
const ISOTOPIC: &[u8] = include_bytes!("fixtures/smiles/isotopic-chain.smi");
const RINGS: &[u8] = include_bytes!("fixtures/smiles/two-ring-labels.smi");
const SALT: &[u8] = include_bytes!("fixtures/smiles/salt-branch.smi");

#[test]
fn one_record_uses_the_real_edges_bracket_h_charge_and_positive_isotopes() -> Test {
    for (source, formula, atoms, components, defaulted) in [
        (ISOTOPIC, "C[13C]H6O", 3, 1, 0),
        (RINGS, "C7H14", 7, 2, 7),
        (SALT, "C4H9NaO", 6, 2, 3),
        (b"[2H][NH3+]".as_slice(), "H3[2H]N+", 2, 1, 0),
        (b"[H+]", "H+", 1, 1, 0),
        (b"[H][H]", "H2", 2, 1, 0),
        (b"C1.C1", "C2H6", 2, 1, 2),
        (b"C.C", "C2H8", 2, 2, 2),
        (b"B", "BH3", 1, 1, 1),
        (b"Cl", "ClH", 1, 1, 1),
        (b"P(F)(F)(F)F", "F4HP", 5, 1, 5),
        (b"S(F)(F)F", "F3HS", 4, 1, 4),
        // OpenSMILES count on sources refused by RDKit's default chemical sanitization.
        (b"N(F)(F)(F)F", "F4HN", 5, 1, 5),
        (b"C(F)(F)(F)(F)F", "CF5", 6, 1, 6),
    ] {
        let doc = SmilesDocument::parse(Arc::from(source))?;
        let result = doc
            .record_composition(&Cancellation::default())?
            .ok_or("missing composition")?;
        assert_eq!(
            result.composition.hill_formula(FormulaIsotopes::Separate)?,
            formula
        );
        assert_eq!(
            (
                result.atom_count,
                result.component_count,
                result.defaulted_hydrogen_atoms
            ),
            (atoms, components, defaulted)
        );
        assert_eq!(doc.as_bytes(), source);
    }
    Ok(())
}

#[test]
fn keeps_ambiguous_markers_unavailable_and_observes_cancellation() -> Test {
    for source in ["[0C]", "c1ccccc1", "C:C", "[nH]"] {
        let doc = SmilesDocument::parse(Arc::from(source.as_bytes()))?;
        assert!(
            doc.record_composition(&Cancellation::default())?.is_none(),
            "{source}"
        );
        assert_eq!(doc.as_bytes(), source.as_bytes());
    }
    let cancelled = Cancellation::default();
    cancelled.cancel();
    let doc = SmilesDocument::parse(Arc::from(ISOTOPIC))?;
    assert!(matches!(
        doc.record_composition(&cancelled),
        Err(CompositionError::Cancelled)
    ));
    Ok(())
}
