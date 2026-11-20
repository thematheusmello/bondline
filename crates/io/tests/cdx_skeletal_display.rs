//! Full-page optics and hit regions against the same independent 3-4-5 bond construction.
use bondline_io::{
    StoredDocument,
    display::{drawing_page_scene, drawing_page_scene_with_view},
};
use bondline_model::{AtomId, BondId, FragmentId, PageSize, Point2};
use bondline_render::{Cancellation, Frame, HitTarget, PageView, cpu_scene};
use std::sync::Arc;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const POINTS: [[f32; 2]; 3] = [[160.0, 240.0], [224.0, 192.0], [288.0, 240.0]];

fn check_paper(frame: &Frame) {
    let mut ink = 0;
    for y in 1..511 {
        for x in 1..511 {
            let start = (512 * y + x) * 4;
            let rgba = &frame.rgba[start..start + 4];
            if rgba == [0, 0, 0, 255] {
                let distance = |a: [f32; 2], b: [f32; 2]| {
                    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
                    let (x, y) = (x as f32 + 0.5 - a[0], y as f32 + 0.5 - a[1]);
                    let t = ((x * dx + y * dy) / (80.0 * 80.0)).clamp(0.0, 1.0);
                    (x - t * dx).hypot(y - t * dy)
                };
                assert!(distance(POINTS[0], POINTS[1]).min(distance(POINTS[1], POINTS[2])) <= 2.72);
                ink += 1;
            } else {
                assert_eq!(rgba, [255, 255, 255, 255]);
            }
        }
    }
    assert_eq!(ink, 638);
    for (x, y) in [(191, 216), (255, 215), (175, 228), (272, 228)] {
        let start = (512 * y + x) * 4;
        assert_eq!(&frame.rgba[start..start + 4], [0, 0, 0, 255]);
    }
}

#[test]
fn four_skeletal_cdx_encodings_share_whole_paper_pixels_and_all_five_hits() -> Result {
    let paper = PageSize::new(512.0, 512.0)?;
    let center = Point2::new(256.0, 256.0)?;
    let control = StoredDocument::from_cdxml(
        Arc::from(include_bytes!("fixtures/cdx-skeletal/optical-control.cdxml").as_slice()),
        PageSize::A4,
    )?;
    let cancel = Cancellation::default();
    for zoom in [100.0, 125.0] {
        let view = PageView::zoom(512, 512, paper, zoom, center)?;
        let expected = cpu_scene(
            drawing_page_scene_with_view(
                &view,
                &control,
                control.model().pages()[0].id(),
                &cancel,
            )?
            .scene(),
        )?;
        if zoom == 100.0 {
            check_paper(&expected);
        }
        for bytes in [
            include_bytes!("fixtures/cdx-skeletal/propane.cdx").as_slice(),
            include_bytes!("fixtures/cdx-skeletal/missing_h.cdx").as_slice(),
            include_bytes!("fixtures/cdx-skeletal/all_h_missing.cdx").as_slice(),
            include_bytes!("fixtures/cdx-skeletal/dormant_true.cdx").as_slice(),
        ] {
            let stored = StoredDocument::from_cdx(Arc::from(bytes), PageSize::A4)?;
            let page = &stored.model().pages()[0];
            let graph = drawing_page_scene_with_view(&view, &stored, page.id(), &cancel)?;
            assert_eq!(cpu_scene(graph.scene())?, expected);
            assert!(graph.label_em_px().is_none() && graph.text_em_range_px().is_none());
            let fragment_id = FragmentId::new(1).ok_or("fragment")?;
            let projected =
                |point: [f32; 2]| point.map(|p| 256.0 + (p - 256.0) * (zoom as f32 / 100.0));
            for (i, p) in POINTS.into_iter().enumerate() {
                let atom_id = AtomId::new(u32::try_from(i + 1)?).ok_or("atom")?;
                assert_eq!(
                    graph.hit_test(projected(p), 0.0)?,
                    Some(HitTarget::Atom {
                        fragment_id,
                        atom_id
                    })
                );
            }
            for (i, p) in [[192.0, 216.0], [256.0, 216.0]].into_iter().enumerate() {
                let bond_id = BondId::new(u32::try_from(i + 1)?).ok_or("bond")?;
                assert_eq!(
                    graph.hit_test(projected(p), 0.0)?,
                    Some(HitTarget::Bond {
                        fragment_id,
                        bond_id
                    })
                );
            }
            assert_eq!(graph.hit_test(projected([216.0, 240.0]), 0.0)?, None);
            if zoom == 100.0 {
                let fit = drawing_page_scene(512, 512, &stored, page.id(), &cancel)?;
                let other = drawing_page_scene(
                    512,
                    512,
                    &control,
                    control.model().pages()[0].id(),
                    &cancel,
                )?;
                assert_eq!(cpu_scene(fit.scene())?, cpu_scene(other.scene())?);
            }
            assert_eq!(stored.export_cdx()?, bytes);
        }
    }
    Ok(())
}
