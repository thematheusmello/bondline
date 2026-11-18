//! Separate new-type qualification; do not let it mask the existing-API runtime RED.
use bondline_io::{DocumentError, StoredDocument};
use bondline_model::{DrawingSettings, PageSize};
use serde_json::json;
use std::{borrow::Cow, sync::Arc};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn style_only_cdx_model_drift_refuses_source_export_and_exact_restoration_reuses_it() -> TestResult
{
    // Independent original explicit Document0/empty page1;48.5pt,137.5deg.
    let mut raw = b"VjCD0100\x04\x03\x02\x01".to_vec();
    raw.extend_from_slice(&[0; 16]);
    raw.extend_from_slice(b"\x00\x80\x00\x00\x00\x00\x05\x08\x04\x00\x00\x80\x30\x00\x03\x08\x04\x00\x00\x80\x89\x00\x01\x80\x01\x00\x00\x00\x00\x00\x00\x00");
    let bytes: Arc<[u8]> = raw.into();
    let clean = StoredDocument::from_cdx(Arc::clone(&bytes), PageSize::A4)?;
    let owner = clean.model_snapshot();
    let original = clean.model().drawing_settings();
    assert_eq!(
        (original.bond_length_pt(), original.chain_angle_degrees()),
        (Some(48.5), Some(137.5))
    );
    assert!(matches!(clean.to_cdx()?, Cow::Borrowed(_)));
    let mut changed = clean.clone();
    changed
        .model_mut()
        .set_drawing_settings(DrawingSettings::new(Some(64.25), None)?);
    assert_eq!(
        serde_json::to_value(changed.model())?["drawing_settings"],
        json!({"bond_length_pt":64.25})
    );
    assert_eq!(
        serde_json::to_value(changed.model())?["pages"],
        serde_json::to_value(clean.model())?["pages"]
    );
    assert!(!Arc::ptr_eq(&owner, &changed.model_snapshot()));
    assert!(changed.source_info().ok_or("source")?.modified);
    assert!(matches!(
        changed.export_cdx(),
        Err(DocumentError::UnsupportedCdxEdit)
    ));
    assert!(matches!(
        changed.to_cdx(),
        Err(DocumentError::UnsupportedCdxEdit)
    ));
    assert!(!changed.is_fully_mapped());
    assert_eq!(clean.export_cdx()?, bytes.as_ref());
    assert!(Arc::ptr_eq(&owner, &clean.model_snapshot()));
    changed.model_mut().set_drawing_settings(original);
    assert!(!changed.source_info().ok_or("source")?.modified);
    assert!(matches!(changed.to_cdx()?, Cow::Borrowed(_)));
    assert_eq!(changed.export_cdx()?.as_ptr(), bytes.as_ptr());
    assert_eq!(changed.export_cdx()?, bytes.as_ref());
    assert!(!changed.is_fully_mapped());
    Ok(())
}
