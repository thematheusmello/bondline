//! Existing public API runtime declaration, frozen before PageSetSize guard changes.
use bondline_commands::Engine;
use bondline_io::StoredDocument;
use bondline_model::PageSize;
use serde_json::json;
use std::sync::Arc;
type TestResult = Result<(), Box<dyn std::error::Error>>;
const SOURCE: &[u8] = br#"<CDXML CreationProgram='Original independent test'><fonttable><font id='42' name='Roboto'/></fonttable><colortable><color r='1' g='1' b='1'/><color r='0' g='0' b='0'/><color r='0.2' g='0.4' b='0.8'/></colortable><page id='90' DrawingSpace='Poster' Width='256' Height='256'><fragment id='51'><n id='73' Element='8' NumHydrogens='1' p='72 96'><t id='21' p='71.5 96.25' Justification='Left' LabelJustification='Auto' LabelAlignment='Right'><s font='42' size='12.5' face='96' color='4'>OH</s></t></n></fragment></page></CDXML>"#;

#[test]
fn authored_node_child_text_blocks_page_changes_and_even_numeric_noops_without_mutating_any_owner()
-> TestResult {
    for size in [256.0, 512.0] {
        let mut e = Engine::from_document(StoredDocument::from_cdxml(SOURCE.into(), PageSize::A4)?);
        let state = serde_json::to_value(e.state())?;
        let content = e.content().clone();
        let model = e.stored_document()?.model_snapshot();
        let stamp = e.capture_stamp();
        let file = e.ui_file_context_stamp();
        let error = e
            .dispatch(
                "page.set_size",
                json!({"page_id":1,"width_pt":size,"height_pt":size}),
            )
            .err()
            .ok_or("authored-label PageSetSize must refuse before no-op or mutation")?;
        assert_eq!(error.info().code, "unsupported_content");
        assert_eq!(serde_json::to_value(e.state())?, state);
        assert_eq!(e.content(), &content);
        assert_eq!(e.capture_stamp(), stamp);
        assert!(e.ui_file_context_stamp_matches(&file));
        assert!(Arc::ptr_eq(&model, &e.stored_document()?.model_snapshot()));
        assert_eq!(e.stored_document()?.to_cdxml()?.as_ref(), SOURCE);
    }
    Ok(())
}
// Append to command_atom_label_first_red.rs after its frozen existing-API runtime RED.
use bondline_model::{
    Atom, Fragment, FragmentId, PageId, Point2, StyledText, TextId, TextRun, TextStyle,
};

fn source_free_label() -> Result<StoredDocument, Box<dyn std::error::Error>> {
    let mut stored = StoredDocument::default();
    let atom: Atom = serde_json::from_value(
        json!({"id":1,"element":8,"position":{"x_pt":72.0,"y_pt":96.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":1,
        "authored_label":{"position":{"x_pt":71.5,"y_pt":96.25},"justification":"left","label_justification":"auto","label_alignment":"right",
        "runs":[{"text":"OH","style":{"font_family":"Roboto","size_pt":12.5,"face":96,"rgb":[0.2,0.4,0.8]}}]}}),
    )?;
    let page = PageId::new(1).ok_or("page")?;
    stored.model_mut().append_fragments(
        page,
        vec![Fragment::new(
            FragmentId::new(1).ok_or("fragment")?,
            vec![atom],
            vec![],
        )?],
    )?;
    stored.model_mut().append_texts(
        page,
        vec![StyledText::new(
            TextId::new(1).ok_or("caption")?,
            Point2::new(72.0, 144.0)?,
            vec![TextRun::new(
                "caption".into(),
                TextStyle::new("Roboto".into(), 12.0, 0, [0.0; 3])?,
            )?],
        )?],
    )?;
    Ok(stored)
}

#[test]
fn source_free_and_exact_native_labels_refuse_dependent_commands_without_escaping_opaque_guards()
-> TestResult {
    let own = source_free_label()?;
    let canonical = StoredDocument::from_cdxml(own.to_cdxml()?.into_owned().into(), PageSize::A4)?;
    let registry = serde_json::to_string(&bondline_commands::registry())?;
    assert_eq!(bondline_commands::registry().len(), 50);
    for stored in [own, canonical] {
        assert!(!stored.is_fully_mapped());
        let mut e = Engine::from_document(stored);
        let state = serde_json::to_value(e.state())?;
        let content = e.content().clone();
        let stamp = e.capture_stamp();
        let file = e.ui_file_context_stamp();
        let model = e.stored_document()?.model_snapshot();
        for (name, params) in [
            (
                "page.set_size",
                json!({"page_id":1,"width_pt":612.0,"height_pt":792.0}),
            ),
            (
                "atom.set_label",
                json!({"expected_revision":0,"page_id":1,"fragment_id":1,"atom_id":1,"element":8,"explicit_hydrogens":1}),
            ),
            (
                "atom.set_label",
                json!({"expected_revision":0,"page_id":1,"fragment_id":1,"atom_id":1,"element":7,"explicit_hydrogens":2}),
            ),
            (
                "text.set_run",
                json!({"expected_revision":0,"page_id":1,"text_id":1,"run_index":0,"text":"caption"}),
            ),
            (
                "text.set_run",
                json!({"expected_revision":0,"page_id":1,"text_id":1,"run_index":0,"text":"changed"}),
            ),
            (
                "document.set_drawing_settings",
                json!({"expected_revision":0,"bond_length_pt":null,"chain_angle_degrees":null}),
            ),
            (
                "document.set_document_settings",
                json!({"expected_revision":0,"bond_length_pt":null,"chain_angle_degrees":null,"line_width_pt":null}),
            ),
            (
                "bond.create",
                json!({"page_id":1,"begin_x_pt":144.0,"begin_y_pt":192.0,"end_x_pt":192.0,"end_y_pt":192.0}),
            ),
            (
                "bond.extend",
                json!({"expected_revision":0,"page_id":1,"fragment_id":1,"begin_atom_id":1,"end_x_pt":120.0,"end_y_pt":96.0,"hydrogen_policy":"rectify_terminal_carbon"}),
            ),
            (
                "bond.set_order",
                json!({"expected_revision":0,"page_id":1,"fragment_id":1,"bond_id":1,"order":"single","hydrogen_policy":"rectify_neutral_carbon"}),
            ),
            (
                "ui.selection",
                json!({"action":"all","expected_revision":0,"page_id":1}),
            ),
            (
                "chain.create",
                json!({"expected_revision":0,"page_id":1,"count":3,"angle_degrees":120.0,"hydrogen_policy":"rectify_neutral_carbon",
                "placement":{"mode":"free","begin_x_pt":144.0,"begin_y_pt":192.0,"end_x_pt":192.0,"end_y_pt":192.0,"side":"positive"}}),
            ),
            (
                "ring.create",
                json!({"expected_revision":0,"page_id":1,"size":6,"hydrogen_policy":"rectify_neutral_carbon",
                "placement":{"mode":"free","begin_x_pt":144.0,"begin_y_pt":192.0,"end_x_pt":192.0,"end_y_pt":192.0,"side":"positive"}}),
            ),
            (
                "render.capture",
                json!({"page_id":1,"width_px":1024,"height_px":1024,"backend":"cpu"}),
            ),
        ] {
            let error = e
                .dispatch(name, params)
                .err()
                .ok_or_else(|| format!("{name} acquired unsupported child-label authority"))?;
            assert_eq!(error.info().code, "unsupported_content", "{name}: {error}");
            assert_eq!(serde_json::to_value(e.state())?, state, "{name}");
            assert_eq!(e.content(), &content, "{name}");
            assert_eq!(e.capture_stamp(), stamp);
            assert!(e.ui_file_context_stamp_matches(&file));
            assert!(Arc::ptr_eq(&model, &e.stored_document()?.model_snapshot()));
        }
        for id in [
            "page.a4",
            "page.letter",
            "page.rotate",
            "tool.bond.single",
            "tool.bond.double",
            "tool.bond.triple",
        ] {
            let control = e
                .controls()
                .into_iter()
                .find(|v| v.id == id)
                .ok_or("existing control")?;
            assert!(!control.enabled, "{id} advertises unsupported authoring");
            assert_eq!(control.disabled_reason, Some("unsupported_content"), "{id}");
        }
    }
    assert_eq!(
        serde_json::to_string(&bondline_commands::registry())?,
        registry
    );
    Ok(())
}
