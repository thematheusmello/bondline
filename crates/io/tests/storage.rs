use bondline_io::StoredDocument;
use bondline_model::{PageId, PageSize};
use std::{mem::size_of, sync::Arc};

const SOURCE: &[u8] = include_bytes!("../../cdxml/tests/fixtures/unknown.cdxml");

#[test]
fn source_accounting_uses_allocation_identity_instead_of_byte_equality()
-> Result<(), Box<dyn std::error::Error>> {
    let bytes: Arc<[u8]> = SOURCE.into();
    let first = StoredDocument::from_cdxml(Arc::clone(&bytes), PageSize::A4)?;
    let shared = StoredDocument::from_cdxml(bytes, PageSize::A4)?;
    let distinct = StoredDocument::from_cdxml(SOURCE.into(), PageSize::A4)?;
    assert_eq!(first, shared);
    assert_eq!(first, distinct);
    assert_eq!(
        StoredDocument::retained_bytes([&first, &distinct])
            - StoredDocument::retained_bytes([&first, &shared]),
        SOURCE.len(),
        "separate equal XML allocations cost another complete source payload",
    );
    let clone = first.clone();
    let once = StoredDocument::retained_bytes([&first]);
    assert_eq!(StoredDocument::retained_bytes([&first, &first]), once);
    assert_eq!(
        StoredDocument::retained_bytes([&first, &clone]) - once,
        size_of::<StoredDocument>()
    );
    Ok(())
}

#[test]
fn modified_documents_still_account_for_the_opaque_source_and_baseline()
-> Result<(), Box<dyn std::error::Error>> {
    let original = StoredDocument::from_cdxml(SOURCE.into(), PageSize::A4)?;
    let mut edited = original.clone();
    edited
        .model_mut()
        .set_page_size(PageId::new(1).ok_or("page")?, PageSize::new(72.0, 144.0)?)?;
    assert!(
        StoredDocument::retained_bytes([&edited]) > StoredDocument::retained_bytes([&original])
    );
    assert_eq!(
        StoredDocument::retained_bytes([&original, &edited]),
        StoredDocument::retained_bytes([&edited]) + size_of::<StoredDocument>(),
        "the original model/source already remain in the edited document's backing",
    );
    assert!(edited.export_cdxml().is_err());
    assert_eq!(original.export_cdxml()?, SOURCE);
    Ok(())
}
