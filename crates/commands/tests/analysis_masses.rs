//! Command path: source coverage, uncapped selections, histories and honest unknowns.
use bondline_commands::Engine;
use bondline_io::StoredDocument;
use bondline_model::{Atom, AtomId, Element, Fragment, FragmentId, PageId};
use serde_json::{Value, json};

type Test<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn properties(engine: &mut Engine) -> Test<Value> {
    let before = serde_json::to_value(engine.state())?;
    let stamp = engine.capture_stamp();
    let request = serde_json::to_value(engine.analysis_request())?;
    let mut result =
        serde_json::to_value(engine.dispatch("analysis.properties", request.clone())?)?;
    let mut report = result
        .as_object_mut()
        .ok_or("result")?
        .remove("analysis")
        .ok_or("analysis")?;
    assert_eq!(result, before);
    assert_eq!(engine.capture_stamp(), stamp);
    // Old formula-only response stays exact for this same revision and selection.
    let masses = report.as_object_mut().ok_or("report")?.remove("masses");
    let old = serde_json::to_value(engine.dispatch("analysis.formula", request)?)?;
    assert_eq!(old["analysis"], report);
    if let Some(masses) = masses {
        report["masses"] = masses;
    }
    Ok(report)
}

fn check(report: &Value, formula: &str, avg: f64, exact: f64) -> Test {
    assert_eq!(report["formula"], formula);
    assert!(
        (report["masses"]["molecular_weight"]
            .as_f64()
            .ok_or("weight")?
            - avg)
            .abs()
            < 1e-8
    );
    assert!(
        (report["masses"]["exact_mass_da"]
            .as_f64()
            .ok_or("exact mass")?
            - exact)
            .abs()
            < 1e-8
    );
    Ok(())
}

#[test]
fn skeletal_source_scope_defaults_and_partial_contribution_use_one_composition() -> Test {
    use std::sync::Arc;
    let bytes = include_bytes!("../../io/tests/fixtures/cdx-mixed-skeleton/ethanol.cdx");
    let stored =
        StoredDocument::from_cdx(Arc::from(bytes.as_slice()), bondline_model::PageSize::A4)?;
    let mut e = Engine::from_document(stored);
    let all = properties(&mut e)?;
    assert_eq!(all["defaulted_hydrogen_atoms"], 2);
    check(&all, "C2H6O", 46.069, 46.04186481295)?;
    e.dispatch(
        "ui.selection",
        json!({"action":"set","expected_revision":0,"page_id":1,"atom_ids":[2,3]}),
    )?;
    let alcohol_end = properties(&mut e)?;
    assert_eq!(alcohol_end["defaulted_hydrogen_atoms"], 1);
    assert_eq!(alcohol_end["partial_fragments"], true);
    check(&alcohol_end, "CH3O", 31.034, 31.01838971626)?;
    // Pure skeletal pages use the same selection and source-aware hydrogen path.
    let bytes = include_bytes!("../../io/tests/fixtures/cdx-skeletal/all_h_missing.cdx");
    let stored =
        StoredDocument::from_cdx(Arc::from(bytes.as_slice()), bondline_model::PageSize::A4)?;
    let mut e = Engine::from_document(stored);
    let stale = e.analysis_request();
    e.dispatch(
        "ui.selection",
        json!({"action":"set","expected_revision":0,"page_id":1,"atom_ids":[1]}),
    )?;
    let selected = properties(&mut e)?;
    assert_eq!(selected["partial_fragments"], true);
    check(&selected, "CH3", 15.035, 15.02347509669)?;
    assert!(matches!(
        e.dispatch("analysis.properties", serde_json::to_value(stale)?),
        Err(bondline_commands::Error::StaleSelection)
    ));
    Ok(())
}

#[test]
fn formula_survives_unavailable_mass_and_missing_hides_all_calculated_fields() -> Test {
    for (z, isotope, hydrogen, expected) in [
        (43, 0, Some(0), "Tc"),
        (118, 293, Some(0), "[293Og]"),
        (6, 13, None, ""),
    ] {
        let mut atom = Atom::new(AtomId::new(5).ok_or("atom")?, Element::try_from(z)?, None);
        atom.isotope = std::num::NonZeroU16::new(isotope);
        atom.explicit_hydrogens = hydrogen;
        let mut stored = StoredDocument::default();
        stored.model_mut().append_fragments(
            PageId::new(1).ok_or("page")?,
            vec![Fragment::new(
                FragmentId::new(1).ok_or("fragment")?,
                vec![atom],
                vec![],
            )?],
        )?;
        let report = properties(&mut Engine::from_document(stored))?;
        if hydrogen.is_none() {
            assert_eq!(
                report,
                json!({"scope":"document","status":"unresolved_hydrogens","atom_id":5})
            );
        } else {
            assert_eq!(report["formula"], expected);
            let unknown =
                json!({"reason":"unknown_isotope","atomic_number":z,"mass_number":isotope});
            assert_eq!(
                report["masses"]["molecular_weight"],
                if isotope == 0 {
                    json!({"reason":"no_standard_atomic_weight","atomic_number":z})
                } else {
                    unknown.clone()
                }
            );
            assert_eq!(
                report["masses"]["exact_mass_da"],
                if isotope == 0 {
                    json!({"reason":"no_natural_isotope","atomic_number":z})
                } else {
                    unknown
                }
            );
        }
    }
    Ok(())
}
