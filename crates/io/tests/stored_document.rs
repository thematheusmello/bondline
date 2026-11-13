use bondline_io::{DocumentError, StoredDocument};
use bondline_model::{PageId, PageSize};
use std::sync::Arc;

const SOURCE: &[u8] = include_bytes!("../../cdxml/tests/fixtures/unknown.cdxml");

#[test]
fn owned_source_survives_its_input_and_opaque_bytes_are_not_exposed_by_debug()
-> Result<(), Box<dyn std::error::Error>> {
    let bytes: Arc<[u8]> = Arc::from(SOURCE);
    let pointer = bytes.as_ptr();
    let stored = StoredDocument::from_cdxml(Arc::clone(&bytes), PageSize::new(612.0, 792.0)?)?;
    drop(bytes);
    assert_eq!(stored.export_cdxml()?, SOURCE);
    assert_eq!(stored.export_cdxml()?.as_ptr(), pointer);
    assert_eq!(
        stored
            .model()
            .pages()
            .first()
            .ok_or("missing page")?
            .fragments()
            .len(),
        1
    );
    let info = stored.source_info().ok_or("missing source info")?;
    assert_eq!(
        serde_json::to_value(info)?,
        serde_json::json!({"format":"cdxml","byte_length":SOURCE.len(),"modified":false})
    );
    assert!(!format!("{stored:?}").contains("keep-this-processing-instruction"));
    Ok(())
}

#[test]
fn unsynchronized_edits_cannot_export_stale_xml_and_restoring_the_model_restores_passthrough()
-> Result<(), Box<dyn std::error::Error>> {
    let mut stored = StoredDocument::from_cdxml(Arc::from(SOURCE), PageSize::new(612.0, 792.0)?)?;
    let original = stored.clone();
    let id = PageId::new(1).ok_or("invalid page")?;
    stored
        .model_mut()
        .set_page_size(id, PageSize::new(72.0, 144.0)?)?;
    assert!(stored.source_info().ok_or("missing info")?.modified);
    assert!(matches!(
        stored.export_cdxml(),
        Err(DocumentError::UnsupportedEdit)
    ));
    assert_eq!(original.export_cdxml()?, SOURCE);
    stored
        .model_mut()
        .set_page_size(id, PageSize::new(612.0, 792.0)?)?;
    assert_eq!(stored, original);
    assert_eq!(stored.export_cdxml()?, SOURCE);
    assert!(matches!(
        StoredDocument::default().export_cdxml(),
        Err(DocumentError::NoCdxmlSource)
    ));
    Ok(())
}

#[test]
fn incomplete_page_mapping_is_explicit_instead_of_silently_flattening_pages()
-> Result<(), Box<dyn std::error::Error>> {
    for bytes in [
        b"<CDXML><page/><page/></CDXML>".as_slice(),
        b"<CDXML><table><page/></table></CDXML>",
    ] {
        assert!(StoredDocument::from_cdxml(Arc::from(bytes), PageSize::A4).is_err());
    }
    let empty = StoredDocument::from_cdxml(Arc::from(b"<CDXML/>".as_slice()), PageSize::A4)?;
    assert_eq!(empty.export_cdxml()?, b"<CDXML/>");
    Ok(())
}
