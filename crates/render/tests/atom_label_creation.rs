//! Proposed missing-new-API compiler declaration; name/preconditions await Root lock.
#[path = "support/boxed_atom.rs"]
mod support;
use bondline_model::{TextJustification, TextStyle};
use bondline_render::create_atom_label;
use support::*;

#[test]
fn deliberate_new_style_uses_existing_156_independent_box_vectors_without_mutation() -> TestResult {
    let cases = cases()?;
    assert_eq!(cases.len(), 156);
    for v in cases {
        let mut atom = atom(&v)?;
        atom.authored_label = None;
        for terminal in [None, Some(false), Some(true)] {
            for nonterminal in [None, Some(false), Some(true)] {
                atom.show_terminal_carbon_labels = terminal;
                atom.show_non_terminal_carbon_labels = nonterminal;
                let before = atom.clone();
                let style = TextStyle::new(
                    "Roboto".into(),
                    v["size_pt"].as_f64().ok_or("size")?,
                    0,
                    [0.4, 0.6, 0.2],
                )?;
                let label: bondline_model::AtomLabel = create_atom_label(&atom, style.clone())?;
                assert_eq!(label.position(), None);
                assert_eq!(label.justification(), Some(TextJustification::Left));
                assert_eq!(label.label_justification(), Some(TextJustification::Left));
                assert_eq!(label.label_alignment(), None);
                assert_eq!(label.runs().len(), 1);
                assert_eq!(label.runs()[0].text(), v["text"].as_str().ok_or("body")?);
                assert_eq!(label.runs()[0].style(), &style);
                let expected: [f64; 4] = serde_json::from_value(v["bounding_box"].clone())?;
                for (actual, expected) in
                    label.bounding_box().ok_or("box")?.into_iter().zip(expected)
                {
                    assert!(
                        (actual - expected).abs() < 1e-10,
                        "{} {actual} !={expected}",
                        v["name"]
                    );
                }
                assert_eq!(atom, before);
            }
        }
    }
    Ok(())
}

#[test]
fn creation_refuses_existing_labels_unknown_styles_and_unsupported_atom_data() -> TestResult {
    use bondline_model::{Element, Radical};
    use std::num::NonZeroU16;
    let v = cases()?
        .into_iter()
        .find(|v| v["name"] == "C-12pt-80-100")
        .ok_or("vector")?;
    let original = atom(&v)?;
    let style = TextStyle::new("Roboto".into(), 12.0, 0, [0.4, 0.6, 0.2])?;
    assert!(
        create_atom_label(&original, style.clone()).is_err(),
        "creation cannot silently replace an owned label"
    );
    let mut unlabelled = original;
    unlabelled.authored_label = None;
    for style in [
        TextStyle::new("Other".into(), 12.0, 0, [0.4, 0.6, 0.2])?,
        TextStyle::new("Roboto".into(), 12.0, 1, [0.4, 0.6, 0.2])?,
    ] {
        assert!(create_atom_label(&unlabelled, style).is_err());
    }
    let mut invalid = Vec::new();
    let mut a = unlabelled.clone();
    a.position = None;
    invalid.push(a);
    let mut a = unlabelled.clone();
    a.explicit_hydrogens = None;
    invalid.push(a);
    let mut a = unlabelled.clone();
    a.charge = 1;
    invalid.push(a);
    let mut a = unlabelled.clone();
    a.isotope = NonZeroU16::new(13);
    invalid.push(a);
    let mut a = unlabelled.clone();
    a.radical = Radical::Doublet;
    invalid.push(a);
    let mut a = unlabelled;
    a.element = Element::try_from(1)?;
    a.explicit_hydrogens = Some(1);
    invalid.push(a);
    for atom in invalid {
        let before = atom.clone();
        assert!(create_atom_label(&atom, style.clone()).is_err());
        assert_eq!(atom, before);
    }
    Ok(())
}
