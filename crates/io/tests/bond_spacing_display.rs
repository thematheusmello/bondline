//! Literal preferred-length geometry; complete owners override stale camera data.
use bondline_io::{StoredDocument, display};
use bondline_model::{BondId, DrawingSettings, DrawingStyle, FragmentId, PageId, PageSize};
use bondline_render::{Cancellation, PageView};
use std::sync::Arc;

type TestResult = Result<(), Box<dyn std::error::Error>>;
const SOURCE: &[u8] = br#"<CDXML CreationProgram="Bondline 0.1.0" BondLength="40" BondSpacing="18" LineWidth=".6"><page id="1" DrawingSpace="Poster" Width="256" Height="256"><fragment id="2"><n id="3" NodeType="Element" Element="6" Charge="0" Radical="None" p="32 64.125" NumHydrogens="2"/><n id="4" NodeType="Element" Element="6" Charge="0" Radical="None" p="72 64.125" NumHydrogens="2"/><b id="5" B="3" E="4" Order="2"/></fragment></page></CDXML>"#;

#[test]
fn complete_owner_preferences_and_style_override_stale_cameras_without_mutation() -> TestResult {
    let mut stored = StoredDocument::from_cdxml(SOURCE.into(), PageSize::A4)?;
    assert!(stored.is_fully_mapped());
    let page = PageId::new(1).ok_or("page")?;
    let fragment = FragmentId::new(1).ok_or("fragment")?;
    let bond = BondId::new(1).ok_or("bond")?;
    let cancel = Cancellation::default();
    for (preferred, rows) in [
        (20.0, [156.65, 163.85]),
        (40.0, [153.05, 167.45]),
        (80.0, [145.85, 174.65]),
    ] {
        stored
            .model_mut()
            .set_drawing_settings(DrawingSettings::new(Some(preferred), None)?);
        let snapshot = stored.model_snapshot();
        let bytes = stored.to_cdxml()?.into_owned();
        let stale = PageView::fit(576, 576, stored.model().pages()[0].size())?
            .with_drawing_settings(DrawingSettings::new(Some(12.0), None)?)
            .with_drawing_style(
                DrawingStyle::new(Some(2.0))?.with_bond_spacing_percent(Some(30.0))?,
            );
        for graph in [
            display::drawing_page_scene(576, 576, &stored, page, &cancel)?,
            display::drawing_page_scene_with_view(&stale, &stored, page, &cancel)?,
        ] {
            let strokes = graph.bond_strokes(fragment, bond).collect::<Vec<_>>();
            assert_eq!(strokes.len(), 2);
            for (stroke, expected_y) in strokes.iter().zip(rows) {
                assert_eq!((stroke.begin[0], stroke.end[0]), (96.0, 176.0));
                assert!((f64::from(stroke.begin[1]) - expected_y).abs() < 0.0001);
                assert_eq!(stroke.begin[1], stroke.end[1]);
                assert!((f64::from(stroke.width) - 1.2).abs() < 0.0001);
            }
            assert_eq!(graph.label_em_px(), Some(24.0));
        }
        assert!(Arc::ptr_eq(&snapshot, &stored.model_snapshot()));
        assert_eq!(stored.to_cdxml()?.as_ref(), bytes);
    }
    Ok(())
}
