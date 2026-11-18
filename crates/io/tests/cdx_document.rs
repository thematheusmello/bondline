use bondline_io::{
    DocumentError, StoredDocument,
    mol::{MolTransform, Version},
};
use bondline_model::{PageId, PageSize, Point2};
use serde_json::json;
use std::{borrow::Cow, mem::size_of, sync::Arc};

type TestResult = Result<(), Box<dyn std::error::Error>>;
const LEGACY: &[u8] = include_bytes!("../../cdx/tests/fixtures/original-ethane-explicit.cdx");
const HEADER: &[u8] =
    include_bytes!("../../cdx/tests/fixtures/original-ethane-header-document.cdx");

#[test]
fn both_original_framings_keep_owned_bytes_and_literal_elemental_coordinates() -> TestResult {
    for source in [LEGACY, HEADER] {
        let bytes: Arc<[u8]> = Arc::from(source);
        let pointer = bytes.as_ptr();
        let stored = StoredDocument::from_cdx(bytes, PageSize::A4)?;
        assert_eq!(stored.export_cdx()?, source);
        assert_eq!(stored.export_cdx()?.as_ptr(), pointer);
        assert!(matches!(stored.to_cdx()?, Cow::Borrowed(_)));
        assert_eq!(
            serde_json::to_value(stored.source_info().ok_or("source")?)?,
            json!({"format":"cdx", "byte_length":source.len(), "modified":false})
        );
        let page = stored.model().pages().first().ok_or("page")?;
        assert_eq!(page.size(), PageSize::A4);
        let fragment = page.fragments().first().ok_or("fragment")?;
        assert_eq!(
            (
                fragment.id().get(),
                fragment.atoms().len(),
                fragment.bonds().len()
            ),
            (1, 2, 1)
        );
        let first = fragment.atoms().first().ok_or("first atom")?;
        let second = fragment.atoms().get(1).ok_or("second atom")?;
        assert_eq!((first.id.get(), second.id.get()), (1, 2));
        assert_eq!(first.position, Some(Point2::new(10.25, -3.5)?));
        assert_eq!(second.position, Some(Point2::new(26.25, 8.5)?));
        assert_eq!(
            (first.explicit_hydrogens, second.explicit_hydrogens),
            (None, None)
        );
        assert!(
            !stored.is_fully_mapped(),
            "the elemental view omits opaque source content"
        );
        assert_eq!(stored.mol_transform(), None);
        assert_eq!(stored.mol_version(), None);
        assert!(!format!("{stored:?}").contains("root\0"));
    }
    Ok(())
}

#[test]
fn authored_change_refuses_stale_source_and_restoring_a_snapshot_restores_all_bytes() -> TestResult
{
    let mut stored = StoredDocument::from_cdx(Arc::from(HEADER), PageSize::A4)?;
    let before = stored.clone();
    let page_id = PageId::new(1).ok_or("page id")?;
    stored
        .model_mut()
        .set_page_size(page_id, PageSize::new(420.0, 594.0)?)?;
    assert!(stored.source_info().ok_or("source")?.modified);
    assert!(matches!(
        stored.export_cdx(),
        Err(DocumentError::UnsupportedCdxEdit)
    ));
    assert!(matches!(
        stored.to_cdx(),
        Err(DocumentError::UnsupportedCdxEdit)
    ));
    assert_eq!(before.export_cdx()?, HEADER);
    let after = stored.clone();
    stored = before.clone();
    assert_eq!(stored, before);
    assert_eq!(stored.to_cdx()?.as_ref(), HEADER);
    stored = after.clone();
    assert_eq!(stored, after);
    assert_eq!(
        stored.to_cdx().err().ok_or("stale source error")?.code(),
        "unsupported_content"
    );
    Ok(())
}

#[test]
fn detached_equal_model_keeps_the_original_clean_baseline() -> TestResult {
    let mut stored = StoredDocument::from_cdx(Arc::from(LEGACY), PageSize::A4)?;
    let same = stored.model().clone();
    *stored.model_mut() = same;
    assert!(!stored.source_info().ok_or("source")?.modified);
    assert_eq!(stored.export_cdx()?, LEGACY);
    Ok(())
}

#[test]
fn opaque_only_changes_are_not_erased_by_model_equality() -> TestResult {
    let mut changed = LEGACY.to_vec();
    let offset = changed
        .windows(6)
        .position(|value| value == b"root\0\xfb")
        .ok_or("opaque property")?;
    *changed.get_mut(offset).ok_or("opaque byte")? = b'R';
    let a = StoredDocument::from_cdx(Arc::from(LEGACY), PageSize::A4)?;
    let b = StoredDocument::from_cdx(Arc::from(changed.as_slice()), PageSize::A4)?;
    assert_eq!(a.model(), b.model());
    assert_ne!(a, b);
    assert_eq!(b.export_cdx()?, changed);
    assert!(!b.source_info().ok_or("source")?.modified);
    Ok(())
}

#[test]
fn retained_storage_deduplicates_shared_source_and_keeps_original_model_baseline() -> TestResult {
    let stored = StoredDocument::from_cdx(Arc::from(HEADER), PageSize::A4)?;
    let clone = stored.clone();
    let one = StoredDocument::retained_bytes([&stored]);
    assert!(one >= HEADER.len() + size_of::<StoredDocument>());
    assert_eq!(StoredDocument::retained_bytes([&stored, &stored]), one);
    assert_eq!(
        StoredDocument::retained_bytes([&stored, &clone]) - one,
        size_of::<StoredDocument>()
    );
    let mut edited = clone.clone();
    edited.model_mut().set_page_size(
        PageId::new(1).ok_or("page id")?,
        PageSize::new(420.0, 594.0)?,
    )?;
    assert!(StoredDocument::retained_bytes([&edited]) > one);
    let mut restored = edited.clone();
    *restored.model_mut() = stored.model().clone();
    assert_eq!(restored.export_cdx()?, HEADER);
    Ok(())
}

#[test]
fn no_source_or_cross_format_export_cannot_drop_unknown_binary_content() -> TestResult {
    let native = StoredDocument::default();
    assert!(matches!(
        native.export_cdx(),
        Err(DocumentError::NoCdxSource)
    ));
    assert!(matches!(native.to_cdx(), Err(DocumentError::NoCdxSource)));
    let stored = StoredDocument::from_cdx(Arc::from(LEGACY), PageSize::A4)?;
    assert!(matches!(
        stored.export_cdxml(),
        Err(DocumentError::NoCdxmlSource)
    ));
    assert!(matches!(
        stored.export_mol(),
        Err(DocumentError::NoMolSource)
    ));
    assert!(matches!(
        stored.to_cdxml(),
        Err(DocumentError::CrossFormatSource {
            from: "cdx",
            to: "cdxml"
        })
    ));
    let frame = MolTransform::new(24.0, Point2::new(72.0, 72.0)?)?;
    for version in [Version::V2000, Version::V3000] {
        assert!(matches!(
            stored.to_mol_as(version, frame),
            Err(DocumentError::CrossFormatSource {
                from: "cdx",
                to: "mol"
            })
        ));
    }
    assert_eq!(stored.export_cdx()?, LEGACY);
    Ok(())
}

#[test]
fn syntax_unsupported_metadata_and_input_budget_keep_distinct_error_codes() -> TestResult {
    assert_eq!(
        StoredDocument::from_cdx(Arc::from(&b"not a CDX document"[..]), PageSize::A4)
            .err()
            .ok_or("syntax error")?
            .code(),
        "invalid_document"
    );
    let mut source = bondline_cdx::Writer::new()?;
    source.start_object(0x8001, 42)?;
    source.start_object(0x8003, 100)?;
    source.start_object(0x8004, 101)?;
    // Published UINT16 NumHydrogens now projects literal data without source authority.
    source.property(0x042b, &4_u16.to_le_bytes())?;
    source.end_object()?;
    source.end_object()?;
    source.end_object()?;
    let source = source.finish()?;
    let stored = StoredDocument::from_cdx(source.clone().into(), PageSize::A4)?;
    assert_eq!(
        stored.model().pages()[0].fragments()[0].atoms()[0].explicit_hydrogens,
        Some(4)
    );
    assert!(!stored.is_fully_mapped());
    assert!(!stored.has_display_projection());
    assert_eq!(stored.to_cdx()?.as_ref(), source);
    let mut unsupported = bondline_cdx::Writer::new()?;
    unsupported.start_object(0x8001, 42)?;
    unsupported.start_object(0x8003, 100)?;
    unsupported.start_object(0x8004, 101)?;
    unsupported.property(0x0430, &[1])?;
    unsupported.end_object()?;
    unsupported.end_object()?;
    unsupported.end_object()?;
    assert_eq!(
        StoredDocument::from_cdx(unsupported.finish()?.into(), PageSize::A4)
            .err()
            .ok_or("unsupported metadata")?
            .code(),
        "unsupported_content"
    );
    assert_eq!(
        StoredDocument::from_cdx(
            Arc::from(vec![0; bondline_cdx::MAX_INPUT_BYTES + 1]),
            PageSize::A4,
        )
        .err()
        .ok_or("input budget")?
        .code(),
        "resource_limit"
    );
    Ok(())
}
