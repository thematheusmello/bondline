//! Independent position and box splices and whole-paper XML controls.
use bondline_io::{StoredDocument, display::drawing_page_scene_with_view};
use bondline_model::{PageSize, Point2};
use bondline_render::{Cancellation, PageView, cpu_scene};
use std::{path::Path, sync::Arc};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn read(name: &str) -> Result<Arc<[u8]>> {
    Ok(std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name),
    )?
    .into())
}

#[test]
fn moving_hidden_printed_and_scoped_atoms_matches_original_binary_and_xml_witnesses() -> Result {
    for (case, src, ids, dx, dy) in [
        (
            "hidden",
            "cdx-mixed-skeleton/ethanol.cdx",
            &[2][..],
            8.25,
            -5.125,
        ),
        (
            "printed",
            "cdx-mixed-skeleton/methanol.cdx",
            &[2],
            -5.125,
            3.25,
        ),
        (
            "combined",
            "cdx-mixed-skeleton/amino_alcohol.cdx",
            &[1, 3, 5],
            8.25,
            -5.125,
        ),
        (
            "scoped",
            "cdx-mixed-skeletal-move/twins.cdx",
            &[2, 3],
            -5.125,
            3.25,
        ),
    ] {
        let bytes = read(src)?;
        let source = StoredDocument::from_cdx(bytes.clone(), PageSize::A4)?;
        let page = &source.model().pages()[0];
        assert!(source.has_cdx_skeletal_label_preview());
        let after = source.with_cdx_skeletal_translation(page.id(), ids, dx, dy)?;
        let expected = read(&format!("cdx-mixed-skeletal-move/{case}.cdx"))?;
        assert_eq!(after.to_cdx()?.as_ref(), expected.as_ref());
        assert_eq!(source.to_cdx()?.as_ref(), bytes.as_ref());
        let xml = read(&format!("cdx-mixed-skeletal-move/{case}.cdxml"))?;
        let control = StoredDocument::from_cdxml(xml, PageSize::A4)?;
        for scale in [1, 2] {
            let view = PageView::zoom(
                512 * scale,
                512 * scale,
                page.size(),
                100. * f64::from(scale),
                Point2::new(256., 256.)?,
            )?;
            let render = |doc: &StoredDocument| -> Result<_> {
                Ok(cpu_scene(
                    drawing_page_scene_with_view(&view, doc, page.id(), &Cancellation::default())?
                        .scene(),
                )?)
            };
            assert_eq!(render(&after)?, render(&control)?, "{case} {scale}x");
        }
        let same =
            after.with_cdx_skeletal_translation(page.id(), ids, 0.5 / 65536., -0.5 / 65536.)?;
        assert!(Arc::ptr_eq(&same.model_snapshot(), &after.model_snapshot()));
        let back = after.with_cdx_skeletal_translation(page.id(), ids, -dx, -dy)?;
        assert_eq!(back.to_cdx()?.as_ref(), bytes.as_ref());
        assert!(Arc::ptr_eq(
            &back.model_snapshot(),
            &source.model_snapshot()
        ));
    }
    Ok(())
}

#[test]
fn reject_collisions_clipped_ink_unsafe_coordinates_and_detached_geometry() -> Result {
    let bytes = read("cdx-mixed-skeleton/methanol.cdx")?;
    let source = StoredDocument::from_cdx(bytes.clone(), PageSize::A4)?;
    let page = source.model().pages()[0].id();
    let stamp = source.model_snapshot();
    for (ids, dx, dy) in [
        (&[2][..], -80., 48.), // printed O exactly on hidden C
        (&[2], 301., 0.),      // O position is on paper, OH glyph is outside
        (&[1, 2], 40000., 0.),
        (&[2], 32555., 0.), // signed coordinate can contain O, not the right edge of H
        (&[1, 9], 5., 5.),
        (&[1, 1], 5., 5.),
    ] {
        assert!(
            source
                .with_cdx_skeletal_translation(page, ids, dx, dy)
                .is_err(),
            "{ids:?}, {dx} {dy}"
        );
        assert!(Arc::ptr_eq(&source.model_snapshot(), &stamp));
        assert_eq!(source.to_cdx()?.as_ref(), bytes.as_ref());
    }
    let mut detached = source.clone();
    detached.model_mut().set_page_size(page, PageSize::A4)?;
    assert!(
        detached
            .with_cdx_skeletal_translation(page, &[2], 5., 5.)
            .is_err()
    );
    Ok(())
}
