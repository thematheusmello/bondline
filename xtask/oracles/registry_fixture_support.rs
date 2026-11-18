//! Keep the historical JSON immutable while checking the current command description.
pub fn with_cdx_caption_edit(legacy: &str) -> String {
    const BEFORE: &str = "Requires native or caption-qualified CDXML and validates";
    const AFTER: &str = "Supports native drawings or qualified CDXML or CDX captions and validates";
    assert_eq!(
        legacy.matches(BEFORE).count(),
        1,
        "frozen text.set_run description"
    );
    legacy.replacen(BEFORE, AFTER, 1)
}
