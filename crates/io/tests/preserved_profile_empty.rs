//! First independent current-API preserved-profile receiving declaration.
use bondline_io::StoredDocument;
use bondline_model::{PageId, PageSize};
use std::borrow::Cow;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn noncanonical_empty_page_resize_preserves_original_source_and_other_lexemes() -> TestResult {
    let source = b"<?xml version='1.0' encoding='UTF-8'?>\r\n<CDXML CreationProgram='Bondline 0.1.0'><!--root--><page Height='792.000' id='91' DrawingSpace=\"Poster\" Width='612.00' /></CDXML>";
    let mut stored = StoredDocument::from_cdxml(source.as_slice().into(), PageSize::A4)?;
    assert!(
        stored.is_fully_mapped(),
        "closed lexical equivalent must have complete edit authority"
    );
    assert!(matches!(stored.to_cdxml()?, Cow::Borrowed(_)));
    let before = stored.clone();
    stored
        .model_mut()
        .set_page_size(PageId::new(1).ok_or("page")?, PageSize::new(600.0, 792.0)?)?;
    let output = stored.to_cdxml()?;
    assert!(matches!(output, Cow::Owned(_)));
    assert_eq!(output.as_ref(), b"<?xml version='1.0' encoding='UTF-8'?>\r\n<CDXML CreationProgram='Bondline 0.1.0'><!--root--><page Height='792.000' id='91' DrawingSpace=\"Poster\" Width='600' /></CDXML>");
    assert_eq!(before.to_cdxml()?.as_ref(), source);
    assert!(
        stored.export_cdxml().is_err(),
        "unchanged-only export cannot return stale bytes"
    );
    Ok(())
}
