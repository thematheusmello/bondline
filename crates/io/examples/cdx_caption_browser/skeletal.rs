//! CPU raster in WebAssembly with independent paper, source-byte and hydrogen expectations.
use super::Result;
use bondline_io::{
    StoredDocument,
    display::{DisplayError, drawing_page_scene_with_view},
};
use bondline_model::{PageId, PageSize, Point2};
use bondline_render::{Cancellation, PageView, cpu_scene};
use std::sync::Arc;

pub(super) fn verify() -> Result {
    let view = PageView::zoom(
        512,
        512,
        PageSize::new(512.0, 512.0)?,
        100.0,
        Point2::new(256.0, 256.0)?,
    )?;
    let id = PageId::new(1).ok_or("page")?;
    let cancel = Cancellation::default();
    let scene = |doc: &StoredDocument| drawing_page_scene_with_view(&view, doc, id, &cancel);
    macro_rules! fixture {
        ($name:literal) => {
            include_bytes!(concat!("../../tests/fixtures/cdx-skeletal/", $name)).as_slice()
        };
    }
    let control_bytes = fixture!("optical-control.cdxml");
    let control = StoredDocument::from_cdxml(Arc::from(control_bytes), PageSize::A4)?;
    let expected = cpu_scene(scene(&control)?.scene())?;
    // The 4 pt bond from (160,240) to (224,192) covers this entire pixel.
    let pixel = (512 * 216 + 191) * 4;
    if expected.rgba.get(pixel..pixel + 4) != Some(&[0, 0, 0, 255])
        || control.to_cdxml()?.as_ref() != control_bytes
    {
        return Err("optical CDXML control failed its ink or source-byte check".into());
    }
    let literal = [Some(3), Some(2), Some(3)];
    for (bytes, hydrogens) in [
        (fixture!("propane.cdx"), literal),
        (fixture!("dormant_true.cdx"), literal),
        (fixture!("dormant_nonterminal.cdx"), literal),
        (fixture!("missing_h.cdx"), [Some(3), None, Some(3)]),
        (fixture!("all_h_missing.cdx"), [None; 3]),
    ] {
        let doc = StoredDocument::from_cdx(Arc::from(bytes), PageSize::A4)?;
        let graph = scene(&doc)?;
        let authored = doc
            .model()
            .pages()
            .iter()
            .flat_map(|p| p.fragments())
            .flat_map(|f| f.atoms())
            .map(|a| a.explicit_hydrogens);
        if !doc.has_cdx_skeletal_preview()
            || cpu_scene(graph.scene())? != expected
            || graph.label_em_px().is_some()
            || !authored.eq(hydrogens)
            || doc.export_cdx()? != bytes
        {
            return Err("CDX skeletal paper, original H fields or exact Save diverged".into());
        }
    }
    // Propane's straightened middle carbon requires ink and is outside this profile.
    let bytes = fixture!("collinear.cdx");
    let rejected = StoredDocument::from_cdx(Arc::from(bytes), PageSize::A4)?;
    if !matches!(scene(&rejected), Err(DisplayError::IncompleteSource))
        || rejected.export_cdx()? != bytes
    {
        return Err("mandatory carbon label lost its data-only source boundary".into());
    }
    for (binary, xml) in [
        (
            include_bytes!("../../tests/fixtures/cdx-mixed-skeleton/methanol.cdx").as_slice(),
            include_bytes!("../../tests/fixtures/mixed-labels/methanol.cdxml").as_slice(),
        ),
        (
            include_bytes!("../../tests/fixtures/cdx-mixed-skeleton/amino_alcohol.cdx").as_slice(),
            include_bytes!("../../tests/fixtures/mixed-labels/amino_alcohol.cdxml").as_slice(),
        ),
    ] {
        let stored = StoredDocument::from_cdx(Arc::from(binary), PageSize::A4)?;
        let reference = StoredDocument::from_cdxml(Arc::from(xml), PageSize::A4)?;
        let model = stored.model();
        let atoms = model.pages()[0].fragments()[0].atoms();
        if !stored.has_cdx_skeletal_label_preview()
            || stored.has_cdx_skeletal_preview()
            || stored.has_cdx_label_edit_profile()
            || !atoms
                .iter()
                .any(|atom| atom.explicit_hydrogens.is_none() && atom.authored_label.is_none())
            || !atoms.iter().any(|atom| atom.authored_label.is_some())
            || cpu_scene(scene(&stored)?.scene())? != cpu_scene(scene(&reference)?.scene())?
            || stored.export_cdx()? != binary
        {
            return Err("printed skeletal CDX lost its source, chemistry or paper".into());
        }
    }
    Ok(())
}
