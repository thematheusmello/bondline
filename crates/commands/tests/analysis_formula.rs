//! Public guide scope: selected atoms, or the whole drawing when nothing is selected.
use bondline_commands::Engine;
use bondline_io::StoredDocument;
use bondline_model::PageSize;
use serde_json::{Value, json};
use std::sync::Arc;

type Test<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn read(engine: &mut Engine) -> Test<Value> {
    let before = serde_json::to_value(engine.state())?;
    let snapshot = engine.capture_stamp();
    let params = json!({"expected_revision":engine.state().revision,
                        "expected_selection_epoch":engine.selection().epoch});
    let mut result = serde_json::to_value(engine.dispatch("analysis.formula", params)?)?;
    let analysis = result
        .as_object_mut()
        .ok_or("result")?
        .remove("analysis")
        .ok_or("analysis")?;
    assert_eq!(result, before);
    assert_eq!(engine.capture_stamp(), snapshot);
    Ok(analysis)
}

#[test]
fn methanol_and_ethane_report_whole_drawing_selected_fragment_and_partial_atom() -> Test {
    let mut e = Engine::default();
    assert_eq!(read(&mut e)?, json!({"scope":"document","status":"empty"}));
    for x in [72, 220] {
        e.dispatch(
            "bond.create",
            json!({"page_id":1,"begin_x_pt":x,"begin_y_pt":72,"end_x_pt":x+48,"end_y_pt":72}),
        )?;
    }
    e.dispatch("atom.set_label", json!({"expected_revision":2,"page_id":1,"fragment_id":1,"atom_id":2,"element":8,"explicit_hydrogens":1}))?;
    let expected = |scope, atoms, fragments, partial, formula| json!({"scope":scope,"status":"available","atom_count":atoms,"fragment_count":fragments,"partial_fragments":partial,"formula":formula});
    assert_eq!(read(&mut e)?, expected("document", 4, 2, false, "C3H10O"));
    for (ids, answer) in [
        (vec![2, 1], expected("selection", 2, 1, false, "CH4O")),
        (vec![2], expected("selection", 1, 1, true, "HO")),
        (vec![2, 3, 4], expected("selection", 3, 2, true, "C2H7O")),
    ] {
        e.dispatch(
            "ui.selection",
            json!({"action":"set","expected_revision":3,"page_id":1,"atom_ids":ids}),
        )?;
        assert_eq!(read(&mut e)?, answer);
    }
    let stale_selection = e
        .dispatch(
            "analysis.formula",
            json!({"expected_revision":3,"expected_selection_epoch":1}),
        )
        .err()
        .ok_or("stale selection accepted")?;
    assert!(matches!(
        stale_selection,
        bondline_commands::Error::StaleSelection
    ));
    e.dispatch("history.undo", json!({}))?;
    assert_eq!(read(&mut e)?, expected("document", 4, 2, false, "C4H12"));
    let stale_revision = e
        .dispatch(
            "analysis.formula",
            json!({"expected_revision":3,"expected_selection_epoch":e.selection().epoch}),
        )
        .err()
        .ok_or("stale revision accepted")?;
    assert!(matches!(
        stale_revision,
        bondline_commands::Error::StaleRevision
    ));
    Ok(())
}

#[test]
fn opaque_source_is_unavailable_even_when_its_model_is_empty() -> Test {
    let stored = StoredDocument::from_cdxml(
        Arc::from(
            b"<CDXML><page id='1'><graphic id='2' GraphicType='Oval'/></page></CDXML>".as_slice(),
        ),
        PageSize::A4,
    )?;
    assert!(!stored.has_display_projection());
    assert_eq!(
        read(&mut Engine::from_document(stored))?,
        json!({"scope":"document","status":"unsupported_source"})
    );
    Ok(())
}

#[test]
fn isotope_charge_and_unresolved_hydrogens_are_reported_without_guessing() -> Test {
    use bondline_model::{Atom, AtomId, Element, Fragment, FragmentId, PageId};
    let mut carbon = Atom::new(AtomId::new(5).ok_or("id")?, Element::try_from(6)?, None);
    carbon.isotope = std::num::NonZeroU16::new(13);
    carbon.charge = -1;
    for (h, answer) in [
        (
            Some(3),
            json!({"scope":"document","status":"available","atom_count":1,"fragment_count":1,"partial_fragments":false,"formula":"[13C]H3-"}),
        ),
        (
            None,
            json!({"scope":"document","status":"unresolved_hydrogens","atom_id":5}),
        ),
    ] {
        carbon.explicit_hydrogens = h;
        let mut stored = StoredDocument::default();
        stored.model_mut().append_fragments(
            PageId::new(1).ok_or("page")?,
            vec![Fragment::new(
                FragmentId::new(1).ok_or("fragment")?,
                vec![carbon.clone()],
                vec![],
            )?],
        )?;
        assert_eq!(read(&mut Engine::from_document(stored))?, answer);
    }
    Ok(())
}

#[test]
fn analysis_window_visibility_is_independent_of_history_and_queries() -> Test {
    let mut e = Engine::default();
    let initial = serde_json::to_value(e.state())?;
    for visible in [true, true, false] {
        let control = e
            .controls()
            .into_iter()
            .find(|r| r.id == "view.analysis")
            .ok_or("View control")?;
        assert!(control.enabled);
        assert_eq!(control.menu, "View");
        let reply = e.dispatch(
            "ui.analysis",
            json!({"expected_revision":0,"visible":visible}),
        )?;
        let mut state = serde_json::to_value(reply)?;
        assert_eq!(
            state
                .as_object_mut()
                .ok_or("state")?
                .remove("analysis_window_open"),
            visible.then_some(Value::Bool(true))
        );
        assert_eq!(state, initial);
        let _ = read(&mut e)?;
    }
    Ok(())
}

#[test]
fn the_scan_accepts_4096_resolved_atoms_and_refuses_an_extra_atom_without_partial_output() -> Test {
    use bondline_model::{Atom, AtomId, Element, Fragment, FragmentId, PageId};
    for count in [4096u32, 4097] {
        let mut atoms = Vec::new();
        for id in 1..=count {
            let mut atom = Atom::new(AtomId::new(id).ok_or("atom")?, Element::try_from(2)?, None);
            atom.explicit_hydrogens = Some(0);
            atoms.push(atom);
        }
        let mut stored = StoredDocument::default();
        stored.model_mut().append_fragments(
            PageId::new(1).ok_or("page")?,
            vec![Fragment::new(
                FragmentId::new(1).ok_or("fragment")?,
                atoms,
                vec![],
            )?],
        )?;
        let mut e = Engine::from_document(stored);
        let r = e.dispatch(
            "analysis.formula",
            json!({"expected_revision":0,"expected_selection_epoch":0}),
        )?;
        let result = serde_json::to_value(r.analysis)?;
        if count == 4096 {
            assert_eq!(result["formula"], "He4096");
        } else {
            assert_eq!(
                result,
                json!({"scope":"document","status":"limit_exceeded","limit":4096})
            );
        }
    }
    Ok(())
}

#[test]
fn exact_cdx_atom_labels_are_readable_and_detached_or_opaque_cdx_stays_unavailable() -> Test {
    let source = include_bytes!("../../io/tests/fixtures/cdx-label-preview/boxed.cdx");
    let mut stored = StoredDocument::from_cdx(Arc::from(source.as_slice()), PageSize::A4)?;
    let result = read(&mut Engine::from_document(stored.clone()))?;
    // The independent generator specifies separate N/H2 and Cl/H0 fragments.
    assert_eq!(result["formula"], "ClH2N");
    stored
        .model_mut()
        .set_page_size(bondline_model::PageId::new(1).ok_or("page")?, PageSize::A4)?;
    assert_eq!(
        read(&mut Engine::from_document(stored))?,
        json!({"scope":"document","status":"unsupported_source"})
    );
    let unsupported =
        include_bytes!("../../io/tests/fixtures/cdx-label-preview/absent_label_box.cdx");
    let stored = StoredDocument::from_cdx(Arc::from(unsupported.as_slice()), PageSize::A4)?;
    assert_eq!(
        read(&mut Engine::from_document(stored))?,
        json!({"scope":"document","status":"unsupported_source"})
    );
    Ok(())
}

#[test]
fn skeletal_cdx_counts_only_written_hydrogens_and_preserves_the_current_source() -> Test {
    let explicit = include_bytes!("../../io/tests/fixtures/cdx-skeletal/propane.cdx").as_slice();
    let central_missing =
        include_bytes!("../../io/tests/fixtures/cdx-skeletal/missing_h.cdx").as_slice();
    let all_missing =
        include_bytes!("../../io/tests/fixtures/cdx-skeletal/all_h_missing.cdx").as_slice();
    for (bytes, expected) in [
        (
            explicit,
            json!({"scope":"document","status":"available","atom_count":3,"fragment_count":1,"partial_fragments":false,"formula":"C3H8"}),
        ),
        (
            central_missing,
            json!({"scope":"document","status":"unresolved_hydrogens","atom_id":2}),
        ),
        (
            all_missing,
            json!({"scope":"document","status":"unresolved_hydrogens","atom_id":1}),
        ),
    ] {
        let mut stored = StoredDocument::from_cdx(Arc::from(bytes), PageSize::A4)?;
        assert!(stored.has_cdx_skeletal_preview() && !stored.is_fully_mapped());
        let mut engine = Engine::from_document(stored.clone());
        assert_eq!(read(&mut engine)?, expected);
        engine.dispatch(
            "ui.selection",
            json!({"action":"set","expected_revision":0,"page_id":1,"atom_ids":[1]}),
        )?;
        let selected = read(&mut engine)?;
        if bytes == all_missing {
            assert_eq!(
                selected,
                json!({"scope":"selection","status":"unresolved_hydrogens","atom_id":1})
            );
        } else {
            assert_eq!(
                selected,
                json!({"scope":"selection","status":"available","atom_count":1,"fragment_count":1,"partial_fragments":true,"formula":"CH3"})
            );
        }
        assert_eq!(engine.stored_document()?.export_cdx()?, bytes);
        stored
            .model_mut()
            .set_page_size(bondline_model::PageId::new(1).ok_or("page")?, PageSize::A4)?;
        assert_eq!(
            read(&mut Engine::from_document(stored))?,
            json!({"scope":"document","status":"unsupported_source"})
        );
    }
    Ok(())
}
