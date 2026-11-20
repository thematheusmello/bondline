//! Browser CPU and file-boundary acceptance for signed, source-preserving atom moves.
use bondline_io::{StoredDocument, display::drawing_page_scene_with_view};
use bondline_model::{PageId, PageSize, Point2};
use bondline_render::{Cancellation, PageView, cpu_scene};
use std::{error::Error, sync::Arc};

pub(super) fn verify() -> Result<(), Box<dyn Error>> {
    let page = PageId::new(1).ok_or("page")?;
    let source = include_bytes!("../../tests/fixtures/cdx-skeletal/all_h_missing.cdx");
    let expected =
        include_bytes!("../../tests/fixtures/cdx-skeletal-move/all_h_missing_center.cdx");
    let original = StoredDocument::from_cdx(Arc::from(source.as_slice()), PageSize::A4)?;
    let moved = original.with_cdx_skeletal_translation(page, &[2], 8.25, -5.125)?;
    let restored = moved.with_cdx_skeletal_translation(page, &[2], -8.25, 5.125)?;
    let bytes = moved.export_cdx()?;
    let reopened = StoredDocument::from_cdx(Arc::from(bytes), PageSize::A4)?;
    if bytes != expected
        || reopened.model() != moved.model()
        || !Arc::ptr_eq(&original.model_snapshot(), &restored.model_snapshot())
        || restored.export_cdx()? != source
        || reopened
            .model()
            .pages()
            .iter()
            .flat_map(|p| p.fragments())
            .flat_map(|f| f.atoms())
            .any(|a| a.explicit_hydrogens.is_some())
    {
        return Err("skeletal move changed raw H fields or saved coordinate bytes".into());
    }
    let xml = include_str!("../../tests/fixtures/cdx-skeletal/optical-control.cdxml")
        .replace(r#"p="224 192""#, r#"p="232.25 186.875""#);
    let control = StoredDocument::from_cdxml(Arc::from(xml.as_bytes()), PageSize::A4)?;
    for (zoom, size) in [(100.0, 512), (200.0, 1024)] {
        let view = PageView::zoom(
            size,
            size,
            PageSize::new(512., 512.)?,
            zoom,
            Point2::new(256., 256.)?,
        )?;
        let scene = |doc| drawing_page_scene_with_view(&view, doc, page, &Cancellation::default());
        let drawn = scene(&reopened)?;
        if drawn.label_em_px().is_some()
            || cpu_scene(drawn.scene())? != cpu_scene(scene(&control)?.scene())?
        {
            return Err("moved skeletal ink differs from the independent CDXML control".into());
        }
    }
    let raw = include_bytes!("../../tests/fixtures/cdx-skeletal-move/aliased_extended.cdx");
    let target = include_bytes!("../../tests/fixtures/cdx-skeletal-move/aliased_moved.cdx");
    let alias = StoredDocument::from_cdx(Arc::from(raw.as_slice()), PageSize::A4)?;
    if alias
        .with_cdx_skeletal_translation(page, &[2, 4], -5.125, 3.25)?
        .export_cdx()?
        != target
    {
        return Err("scoped IDs or extended property lengths changed during movement".into());
    }
    Ok(())
}
