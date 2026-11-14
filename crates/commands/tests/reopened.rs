use bondline_commands::Engine;
use bondline_io::StoredDocument;
use bondline_model::PageSize;
use serde_json::json;

const ORIGINAL: &[u8] = include_bytes!("../../io/tests/fixtures/bondline-v0.cdxml");

#[test]
fn reopened_native_drawing_can_render_edit_encode_and_undo_without_losing_its_source()
-> Result<(), Box<dyn std::error::Error>> {
    let stored = StoredDocument::from_cdxml(ORIGINAL.into(), PageSize::A4)?;
    let mut reopened = Engine::from_document(stored);
    let original = reopened.stored_document()?.clone();
    let first = reopened.dispatch(
        "render.capture",
        json!({"page_id":1,"width_px":1024,"height_px":1024,"backend":"cpu"}),
    )?;
    let png = first.capture.ok_or("capture")?.png_base64;
    let result=reopened.dispatch("bond.create",json!({"page_id":1,"begin_x_pt":100.0,"begin_y_pt":300.0,"end_x_pt":140.0,"end_y_pt":300.0}))?;
    assert_eq!((result.revision, result.undo_depth), (1, 1));
    assert!(result.source.as_ref().ok_or("source")?.modified);
    let edited = reopened.stored_document()?.to_cdxml()?.into_owned();
    let parsed = StoredDocument::from_cdxml(edited.into(), PageSize::A4)?;
    assert_eq!(parsed.model(), reopened.document()?);
    assert_eq!(
        parsed
            .model()
            .pages()
            .first()
            .ok_or("page")?
            .fragments()
            .len(),
        2
    );
    reopened.dispatch("history.undo", json!({}))?;
    assert_eq!(reopened.stored_document()?, &original);
    assert_eq!(reopened.stored_document()?.to_cdxml()?.as_ref(), ORIGINAL);
    let after = reopened.dispatch(
        "render.capture",
        json!({"page_id":1,"width_px":1024,"height_px":1024,"backend":"gpu"}),
    )?;
    assert_eq!(after.capture.ok_or("capture")?.png_base64, png);
    reopened.dispatch("history.redo", json!({}))?;
    assert_eq!(reopened.document()?, parsed.model());
    Ok(())
}
