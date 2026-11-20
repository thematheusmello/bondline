//! Decode the published page with Pillow and a separate PNG chunk reader.
use bondline_io::{StoredDocument, encode_page_png};
use bondline_model::{PageId, PageSize};
use bondline_render::Cancellation;
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
    sync::Arc,
};

type Result = std::result::Result<(), Box<dyn std::error::Error>>;

#[test]
fn physical_page_matches_independent_size_density_pixel_positions_and_alpha() -> Result {
    let source = include_bytes!("fixtures/cdx-skeletal/propane.cdx");
    let doc = StoredDocument::from_cdx(Arc::from(source.as_slice()), PageSize::A4)?;
    let page = PageId::new(1).ok_or("page 1")?;
    for (dpi, transparent) in [(144, false), (300, true)] {
        let bytes = encode_page_png(&doc, page, dpi, transparent, &Cancellation::default())?;
        let mut child = Command::new(std::env::var_os("BONDLINE_PYTHON").ok_or("pinned Pillow")?)
            .arg("-B")
            .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/published_png_oracle.py"))
            .args([dpi.to_string(), transparent.to_string()])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()?;
        child
            .stdin
            .take()
            .ok_or("oracle stdin")?
            .write_all(&bytes)?;
        let result = child.wait_with_output()?;
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
    assert_eq!(doc.export_cdx()?, source);
    let cancelled = Cancellation::default();
    cancelled.cancel();
    assert!(encode_page_png(&doc, page, 300, false, &cancelled).is_err());
    assert!(encode_page_png(&doc, page, 0, false, &Cancellation::default()).is_err());
    Ok(())
}

#[test]
fn letter_print_bounds_and_opaque_document_guards_are_independent_of_capture() -> Result {
    let mut doc = StoredDocument::default();
    let page = PageId::new(1).ok_or("page")?;
    doc.model_mut()
        .set_page_size(page, PageSize::new(612., 792.)?)?;
    let cancel = Cancellation::default();
    let png = encode_page_png(&doc, page, 300, false, &cancel)?;
    assert_eq!(png.get(16..20), Some(2550u32.to_be_bytes().as_slice()));
    assert_eq!(png.get(20..24), Some(3300u32.to_be_bytes().as_slice()));
    assert!(encode_page_png(&doc, page, 1200, false, &cancel).is_err());
    assert!(encode_page_png(&doc, PageId::new(2).ok_or("page 2")?, 72, false, &cancel).is_err());
    let opaque = include_bytes!("fixtures/cdx-skeletal/custom_root.cdx");
    let stored = StoredDocument::from_cdx(Arc::from(opaque.as_slice()), PageSize::A4)?;
    assert!(encode_page_png(&stored, page, 72, false, &cancel).is_err());
    assert_eq!(stored.export_cdx()?, opaque);
    Ok(())
}
