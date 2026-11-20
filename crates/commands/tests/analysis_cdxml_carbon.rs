use bondline_commands::Engine;
use bondline_io::StoredDocument;
use bondline_model::{PageId, PageSize};
use serde_json::{Value, json};
use std::sync::Arc;

const BUTANE: &str = include_str!("../../io/tests/fixtures/cdxml-carbon-analysis/butane.cdxml");
type Test<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn source(xml: &str) -> Test<StoredDocument> {
    Ok(StoredDocument::from_cdxml(
        Arc::from(xml.as_bytes()),
        PageSize::A4,
    )?)
}

fn analysis(engine: &mut Engine, command: &str) -> Test<Value> {
    let params = serde_json::to_value(engine.analysis_request())?;
    Ok(serde_json::to_value(
        engine
            .dispatch(command, params)?
            .analysis
            .ok_or("missing report")?,
    )?)
}

#[test]
fn reads_complete_carbon_connectivity_without_granting_paper_or_edits() -> Test {
    for (xml, formula, derived) in [
        (BUTANE.to_string(), "C4H10", 3),
        (BUTANE.replace("NumHydrogens=\"3\"", ""), "C4H10", 4),
        (
            BUTANE.replace("E=\"75\" Order=\"1\"", "E=\"75\" Order=\"2\""),
            "C4H8",
            3,
        ),
        (
            BUTANE.replace("E=\"75\" Order=\"1\"", "E=\"75\" Order=\"3\""),
            "C4H6",
            3,
        ),
    ] {
        let stored = source(&xml)?;
        assert!(!stored.has_display_projection() && !stored.is_fully_mapped());
        assert_eq!(stored.export_cdxml()?, xml.as_bytes());
        let mut engine = Engine::from_document(stored);
        assert!(!engine.selection_available());
        let answer = analysis(&mut engine, "analysis.formula")?;
        assert_eq!(
            answer,
            json!({"scope":"document","status":"available","atom_count":4,
            "fragment_count":1,"partial_fragments":false,"formula":formula,"defaulted_hydrogen_atoms":derived})
        );
        let result = analysis(&mut engine, "analysis.properties")?;
        assert_eq!(result["formula"], formula);
        assert!(result["masses"]["exact_mass_da"].is_number());
        assert_eq!(engine.stored_document()?.export_cdxml()?, xml.as_bytes());
    }
    let mut detached = source(BUTANE)?;
    detached
        .model_mut()
        .set_page_size(PageId::new(1).ok_or("page")?, PageSize::new(600., 530.)?)?;
    assert!(!detached.has_analysis_coverage());
    assert!(detached.export_cdxml().is_err());
    Ok(())
}

#[test]
fn hidden_chemistry_extensions_external_defaults_and_bad_valence_stay_unresolved() -> Test {
    for (old, new) in [
        ("<!DOCTYPE CDXML>", "<!DOCTYPE CDXML SYSTEM 'other.dtd'>"),
        ("<page id=", "<page xmlns:x='urn:test' id="),
        ("<fragment id=", "<fragment NumRepeats='4' id="),
        ("<n id=\"24\"", "<n EnhancedStereoType='Absolute' id=\"24\""),
        ("<n id=\"24\"", "<n NodeType='ElementList' id=\"24\""),
        ("Element=\"6\"/>", "Element=\"6\"><t><s>Cy</s></t></n>"),
        ("E=\"75\" Order=\"1\"", "E=\"75\" Order=\"1.5\""),
        (
            "E=\"75\" Order=\"1\"",
            "E=\"75\" Order=\"1\" Display='WedgeBegin'",
        ),
        ("</fragment>", "<opaque_chemistry/></fragment>"),
        ("<font name=", "<font QueryElementList='C,N' name="),
        ("NumHydrogens=\"3\"", "NumHydrogens=\"5\""),
    ] {
        let xml = BUTANE.replace(old, new);
        assert_ne!(xml, BUTANE);
        if let Ok(doc) = source(&xml) {
            assert_eq!(doc.export_cdxml()?, xml.as_bytes());
            assert!(!doc.has_analysis_coverage(), "{new}");
        }
    }
    let grouped = BUTANE
        .replace("<fragment id=", "<group><fragment id=")
        .replace("</fragment>", "</fragment></group>");
    assert!(!source(&grouped)?.has_analysis_coverage());
    Ok(())
}
