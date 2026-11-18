#[path = "support/boxed_atom.rs"]
mod support;
use bondline_render::{position_atom_label, validate_atom_label};
use support::*;
#[test]
fn positioned_boxes_match_independent_fonttools_outline_and_gpos_vectors() -> TestResult {
    let cases = cases()?;
    assert_eq!(cases.len(), 156);
    for v in &cases {
        let a = atom(v)?;
        validate_atom_label(&a)?;
        let label = position_atom_label(&a)?;
        let expected: [f64; 4] = serde_json::from_value(v["bounding_box"].clone())?;
        for (a, b) in label.bounding_box().ok_or("box")?.iter().zip(expected) {
            assert!((a - b).abs() < 1e-10, "{} {a} != {b}", v["name"]);
        }
        assert_eq!(label.runs()[0].text(), v["text"].as_str().ok_or("body")?);
        assert_eq!(label.position(), None);
    }
    Ok(())
}

#[test]
fn raw_input_agreement_is_required_and_generation_only_changes_body_and_box() -> TestResult {
    use std::sync::Arc;
    let v = cases()?
        .into_iter()
        .find(|v| v["name"] == "OH-12pt-80-100")
        .ok_or("vector")?;
    let original = atom(&v)?;
    let before = original.clone();
    let mut changed = original.clone();
    changed.element = bondline_model::Element::try_from(7)?;
    changed.explicit_hydrogens = Some(2);
    assert!(
        validate_atom_label(&changed).is_err(),
        "old OH must not prove new N/H2"
    );
    let label = position_atom_label(&changed)?;
    assert_eq!(label.runs()[0].text(), "NH2");
    let old = original.authored_label.as_ref().ok_or("label")?;
    assert_eq!(label.runs()[0].style(), old.runs()[0].style());
    assert_eq!(label.position(), old.position());
    assert_eq!(label.justification(), old.justification());
    changed.authored_label = Some(Arc::new(label));
    validate_atom_label(&changed)?;
    assert_eq!(changed.position, original.position);
    assert_eq!(original, before);
    let mut loose = original.clone();
    loose.authored_label = Some(Arc::new(old.with_run_text(0, "HO".into())?));
    assert!(validate_atom_label(&loose).is_err());
    assert_eq!(position_atom_label(&loose)?.runs()[0].text(), "OH");
    Ok(())
}

#[test]
fn profile_errors_and_fixed_point_tolerance_do_not_certify_unknown_styles_or_geometry() -> TestResult
{
    use bondline_model::{AtomLabel, Point2, Radical, TextJustification, TextRun, TextStyle};
    use std::{num::NonZeroU16, sync::Arc};
    let v = cases()?
        .into_iter()
        .find(|v| v["name"] == "Cl-12pt-80-100")
        .ok_or("vector")?;
    let original = atom(&v)?;
    let label = original.authored_label.as_ref().ok_or("label")?;
    for (p, j, lj, a, font, face) in [
        (
            Some(Point2::new(80.0, 100.0)?),
            Some(TextJustification::Left),
            Some(TextJustification::Left),
            None,
            "Roboto",
            0,
        ),
        (None, None, Some(TextJustification::Left), None, "Roboto", 0),
        (
            None,
            Some(TextJustification::Center),
            Some(TextJustification::Left),
            None,
            "Roboto",
            0,
        ),
        (None, Some(TextJustification::Left), None, None, "Roboto", 0),
        (
            None,
            Some(TextJustification::Left),
            Some(TextJustification::Left),
            Some(bondline_model::LabelAlignment::Left),
            "Roboto",
            0,
        ),
        (
            None,
            Some(TextJustification::Left),
            Some(TextJustification::Left),
            None,
            "Other",
            0,
        ),
        (
            None,
            Some(TextJustification::Left),
            Some(TextJustification::Left),
            None,
            "Roboto",
            96,
        ),
    ] {
        let mut bad = original.clone();
        bad.authored_label = Some(Arc::new(
            AtomLabel::new(
                p,
                j,
                lj,
                a,
                vec![TextRun::new(
                    "Cl".into(),
                    TextStyle::new(font.into(), 12.0, face, [0.2, 0.4, 0.8])?,
                )?],
            )?
            .with_bounding_box(label.bounding_box())?,
        ));
        assert!(validate_atom_label(&bad).is_err());
        assert!(position_atom_label(&bad).is_err());
    }
    let mut variants = Vec::new();
    let mut a = original.clone();
    a.charge = 1;
    variants.push(a);
    let mut a = original.clone();
    a.isotope = NonZeroU16::new(35);
    variants.push(a);
    let mut a = original.clone();
    a.radical = Radical::Doublet;
    variants.push(a);
    let mut a = original.clone();
    a.explicit_hydrogens = None;
    variants.push(a);
    let mut a = original.clone();
    a.position = None;
    variants.push(a);
    let mut a = original.clone();
    a.show_terminal_carbon_labels = Some(true);
    variants.push(a);
    let mut a = original.clone();
    a.show_non_terminal_carbon_labels = Some(false);
    variants.push(a);
    let mut a = original.clone();
    a.element = bondline_model::Element::try_from(1)?;
    a.explicit_hydrogens = Some(1);
    variants.push(a);
    for bad in variants {
        assert!(validate_atom_label(&bad).is_err());
        assert!(position_atom_label(&bad).is_err());
    }
    let raw = label.bounding_box().ok_or("box")?;
    for amount in [0.5 / 65536.0, -0.5 / 65536.0] {
        let mut a = original.clone();
        a.authored_label = Some(Arc::new(
            label.with_bounding_box(Some(raw.map(|x| x + amount)))?,
        ));
        validate_atom_label(&a)?;
    }
    for index in 0..4 {
        let mut bounds = raw;
        bounds[index] += 2.0 / 65536.0;
        let mut a = original.clone();
        a.authored_label = Some(Arc::new(label.with_bounding_box(Some(bounds))?));
        assert!(validate_atom_label(&a).is_err());
    }
    let mut a = original.clone();
    a.authored_label = Some(Arc::new(label.with_bounding_box(None)?));
    assert!(matches!(
        validate_atom_label(&a),
        Err(bondline_render::Error::UnsupportedChemicalContent)
    ));
    let mut a = original.clone();
    a.authored_label = Some(Arc::new(label.with_bounding_box(Some([0.0; 4]))?));
    assert!(validate_atom_label(&a).is_err());
    let mut a = original.clone();
    a.position = Some(Point2::new(100000.0, 100000.0)?);
    assert!(position_atom_label(&a).is_err());
    assert_eq!(original, atom(&v)?);
    Ok(())
}
