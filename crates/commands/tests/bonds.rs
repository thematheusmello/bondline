use bondline_commands::{Command, Engine, NoParams};
use serde_json::json;

fn params() -> serde_json::Value {
    json!({"page_id":1,"begin_x_pt":10.0,"begin_y_pt":20.5,"end_x_pt":50.0,"end_y_pt":20.5})
}

#[test]
fn a_single_carbon_bond_is_one_exact_history_step_with_fresh_stable_ids()
-> Result<(), Box<dyn std::error::Error>> {
    let mut engine = Engine::default();
    let before = engine.stored_document().clone();
    let result = engine.dispatch("bond.create", params())?;
    assert!(result.changed);
    assert_eq!(result.revision, 1);
    assert_eq!(result.undo_depth, 1);
    assert_eq!(
        serde_json::to_value(&result.document)?["pages"][0]["fragments"],
        json!([
            {"id":1,"atoms":[
                {"id":1,"element":6,"position":{"x_pt":10.0,"y_pt":20.5},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":3},
                {"id":2,"element":6,"position":{"x_pt":50.0,"y_pt":20.5},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":3}
            ],"bonds":[{"id":1,"begin":1,"end":2,"order":"single"}]}
        ])
    );
    let after = engine.stored_document().clone();
    engine.execute(Command::Undo(NoParams {}))?;
    assert_eq!(engine.stored_document(), &before);
    engine.execute(Command::Redo(NoParams {}))?;
    assert_eq!(engine.stored_document(), &after);
    let second = engine.dispatch("bond.create", params())?;
    let fragments = &serde_json::to_value(&second.document)?["pages"][0]["fragments"];
    assert_eq!(fragments[1]["id"], 2);
    assert_eq!(fragments[1]["atoms"][0]["id"], 3);
    assert_eq!(fragments[1]["atoms"][1]["id"], 4);
    assert_eq!(fragments[1]["bonds"][0]["id"], 2);
    assert_eq!(
        serde_json::to_value(&result.document)?["pages"][0]["fragments"]
            .as_array()
            .ok_or("fragments")?
            .len(),
        1
    );
    Ok(())
}

#[test]
fn invalid_bond_requests_leave_document_revision_and_redo_untouched()
-> Result<(), Box<dyn std::error::Error>> {
    let mut engine = Engine::default();
    engine.dispatch("bond.create", params())?;
    engine.execute(Command::Undo(NoParams {}))?;
    let original = engine.stored_document().clone();
    for (field, value) in [
        ("page_id", json!(0)),
        ("page_id", json!(2)),
        ("begin_x_pt", json!(100001)),
        ("end_x_pt", json!(10)),
        ("extra", json!(true)),
    ] {
        let mut invalid = params();
        invalid[field] = value;
        assert!(engine.dispatch("bond.create", invalid).is_err());
        assert_eq!(engine.stored_document(), &original);
        assert_eq!(engine.state().revision, 2);
        assert_eq!(engine.state().redo_depth, 1);
    }
    Ok(())
}

#[test]
fn captured_bonds_use_an_immutable_input_snapshot_and_cannot_turn_blank()
-> Result<(), Box<dyn std::error::Error>> {
    let mut engine = Engine::default();
    engine.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":60,"height_pt":60}),
    )?;
    let blank = engine
        .dispatch(
            "render.capture",
            json!({"page_id":1,"width_px":80,"height_px":80,"backend":"cpu"}),
        )?
        .capture
        .ok_or("capture")?
        .png_base64;
    engine.dispatch("bond.create", params())?;
    let plan=engine.prepare_json(br#"{"command":"render.capture","params":{"page_id":1,"width_px":80,"height_px":80,"backend":"cpu"}}"#)?;
    engine.execute(Command::Undo(NoParams {}))?;
    let captured = plan.complete()?;
    assert_eq!(captured.revision, 2);
    assert_eq!(engine.state().revision, 3);
    let image = captured.capture.ok_or("capture")?.png_base64;
    assert_ne!(image, blank);
    engine.execute(Command::Redo(NoParams {}))?;
    let gpu = engine
        .dispatch(
            "render.capture",
            json!({"page_id":1,"width_px":80,"height_px":80,"backend":"gpu"}),
        )?
        .capture
        .ok_or("gpu")?;
    assert_eq!(gpu.png_base64, image);
    assert!(gpu.adapter.is_some());
    Ok(())
}

#[test]
fn branch_edits_never_reuse_deleted_object_ids() -> Result<(), Box<dyn std::error::Error>> {
    let mut engine = Engine::default();
    engine.dispatch("bond.create", params())?;
    engine.execute(Command::Undo(NoParams {}))?;
    let created = engine.dispatch("bond.create", params())?;
    let fragments = &serde_json::to_value(&created.document)?["pages"][0]["fragments"];
    assert_eq!(fragments[0]["id"], 2);
    assert_eq!(fragments[0]["atoms"][0]["id"], 3);
    assert_eq!(fragments[0]["atoms"][1]["id"], 4);
    assert_eq!(fragments[0]["bonds"][0]["id"], 2);
    assert_eq!(created.redo_depth, 0);
    Ok(())
}

#[test]
fn exhausted_id_domains_and_source_backing_reject_creation_atomically()
-> Result<(), Box<dyn std::error::Error>> {
    use bondline_io::StoredDocument;
    use bondline_model::{
        Atom, AtomId, Bond, BondId, BondOrder, Element, Fragment, FragmentId, PageId, PageSize,
        Point2,
    };
    for domain in 0..3 {
        let mut document = StoredDocument::default();
        let first = AtomId::new(1).ok_or("atom")?;
        let second = AtomId::new(if domain == 0 { u32::MAX } else { 2 }).ok_or("atom")?;
        let fragment = Fragment::new(
            FragmentId::new(if domain == 2 { u32::MAX } else { 1 }).ok_or("fragment")?,
            vec![
                Atom::new(first, Element::try_from(6)?, Some(Point2::new(1.0, 1.0)?)),
                Atom::new(second, Element::try_from(6)?, Some(Point2::new(2.0, 2.0)?)),
            ],
            vec![Bond {
                id: BondId::new(if domain == 1 { u32::MAX } else { 1 }).ok_or("bond")?,
                begin: first,
                end: second,
                order: BondOrder::Single,
            }],
        )?;
        document
            .model_mut()
            .append_fragments(PageId::new(1).ok_or("page")?, vec![fragment])?;
        let before = document.clone();
        let mut engine = Engine::from_document(document);
        assert_eq!(
            engine
                .dispatch("bond.create", params())
                .err()
                .ok_or("success")?
                .info()
                .code,
            "identity_exhausted"
        );
        assert_eq!(engine.stored_document(), &before);
        assert_eq!(engine.state().revision, 0);
        assert_eq!(engine.state().undo_depth, 0);
    }
    let source = include_bytes!("../../cdxml/tests/fixtures/unknown.cdxml");
    let document =
        StoredDocument::from_cdxml(std::sync::Arc::from(source.as_slice()), PageSize::A4)?;
    let before = document.clone();
    let mut engine = Engine::from_document(document);
    assert_eq!(
        engine
            .dispatch("bond.create", params())
            .err()
            .ok_or("success")?
            .info()
            .code,
        "unsupported_content"
    );
    assert_eq!(engine.stored_document(), &before);
    assert_eq!(engine.stored_document().export_cdxml()?, source);
    Ok(())
}
