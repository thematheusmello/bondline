use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Document, Element, Fragment, FragmentId, Page, PageId,
    PageSize, Point2,
};
use bondline_render::{
    Cancellation, Error, GraphScene, PageView, Primitive, cpu_scene, gpu_scene, graph_page_scene,
};
use serde_json::json;
use std::{
    collections::BTreeMap,
    io::Write,
    process::{Command, Stdio},
};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
type AtomSpec = (u32, u8, [f64; 2], Option<u16>);

fn page(size: PageSize, specs: &[AtomSpec], edges: &[[u32; 2]]) -> TestResult<Page> {
    let atoms = specs
        .iter()
        .map(|(id, z, xy, h)| {
            let mut atom = Atom::new(
                AtomId::new(*id).ok_or("atom")?,
                Element::try_from(*z)?,
                Some(Point2::new(xy[0], xy[1])?),
            );
            atom.explicit_hydrogens = *h;
            Ok(atom)
        })
        .collect::<TestResult<Vec<_>>>()?;
    let bonds = edges
        .iter()
        .enumerate()
        .map(|(index, [a, b])| {
            Ok(Bond {
                id: BondId::new(u32::try_from(index + 1)?).ok_or("bond")?,
                begin: AtomId::new(*a).ok_or("begin")?,
                end: AtomId::new(*b).ok_or("end")?,
                order: BondOrder::Single,
            })
        })
        .collect::<TestResult<Vec<_>>>()?;
    let mut document = Document::default();
    let id = PageId::new(1).ok_or("page")?;
    document.set_page_size(id, size)?;
    document.append_fragments(
        id,
        vec![Fragment::new(
            FragmentId::new(7).ok_or("fragment")?,
            atoms,
            bonds,
        )?],
    )?;
    Ok(document.pages().first().ok_or("page")?.clone())
}

fn pair() -> TestResult<Page> {
    page(
        PageSize::new(128.0, 128.0)?,
        &[(11, 6, [32.0, 64.0], None), (19, 6, [96.0, 64.0], None)],
        &[[11, 19]],
    )
}

fn stroke(graph: &GraphScene) -> TestResult<&bondline_render::Stroke> {
    graph
        .bond_strokes(
            FragmentId::new(7).ok_or("fragment")?,
            BondId::new(1).ok_or("bond")?,
        )
        .next()
        .ok_or_else(|| "actual bond stroke".into())
}

fn near(actual: [f32; 2], expected: [f64; 2]) {
    for (value, reference) in actual.into_iter().zip(expected) {
        assert!((f64::from(value) - reference).abs() <= 1.0 / 256.0);
    }
}

#[test]
fn one_to_one_pixel_output_is_exact_for_old_fit_zoom_literal_and_display_h() -> TestResult {
    let cancel = Cancellation::default();
    for authored in [None, Some(1)] {
        let source = page(
            PageSize::new(256.0, 256.0)?,
            &[(11, 8, [128.0, 128.0], authored)],
            &[],
        )?;
        let before = source.clone();
        let counts = BTreeMap::from([(AtomId::new(11).ok_or("atom")?, 1)]);
        for view in [
            PageView::fit(320, 240, source.size())?,
            PageView::zoom(320, 240, source.size(), 150.0, Point2::new(128.0, 128.0)?)?,
        ] {
            assert_eq!(view.pixel_density(), 1.0);
            let raster = view.at_pixel_size(320, 240)?;
            assert_eq!(raster.zoom_percent(), view.zoom_percent());
            assert_eq!(raster.pixel_density(), 1.0);
            let (old, new) = if authored.is_some() {
                (
                    view.graph_scene(&source, &cancel)?,
                    raster.graph_scene(&source, &cancel)?,
                )
            } else {
                (
                    view.graph_scene_with_display_hydrogens(&source, &counts, &cancel)?,
                    raster.graph_scene_with_display_hydrogens(&source, &counts, &cancel)?,
                )
            };
            assert_eq!(new.scene(), old.scene());
            assert_eq!(cpu_scene(new.scene())?, cpu_scene(old.scene())?);
            assert_eq!(source, before);
            assert!(matches!(
                raster.at_pixel_size(640, 480),
                Err(Error::InvalidView)
            ));
        }
    }
    Ok(())
}

#[test]
fn logical_fit_at_two_x_preserves_its_rounded_paper_and_is_distinct_from_legacy_pixel_fit()
-> TestResult {
    let source = pair()?;
    let view = PageView::fit(320, 240, source.size())?;
    // Independent logical paper:180x180, origin(70,30). Scale all of it by2.
    let raster = view.at_pixel_size(640, 480)?;
    assert_eq!(raster.zoom_percent(), 140.625);
    assert_eq!(raster.pixel_density(), 2.0);
    assert_eq!(raster.stroke_width(), 2.8125);
    assert_eq!(
        raster.canvas_position(Point2::new(32.0, 64.0)?)?,
        [230.0, 240.0]
    );
    assert_eq!(
        raster.canvas_position(Point2::new(96.0, 64.0)?)?,
        [410.0, 240.0]
    );
    assert_eq!(
        raster.document_position([320.0, 240.0])?,
        Point2::new(64.0, 64.0)?
    );
    let graph = raster.graph_scene(&source, &Cancellation::default())?;
    assert_eq!(stroke(&graph)?.begin, [230.0, 240.0]);
    assert_eq!(stroke(&graph)?.end, [410.0, 240.0]);
    assert_ne!(graph.scene(), graph_page_scene(640, 480, &source)?.scene());
    let cpu = cpu_scene(graph.scene())?;
    let (gpu, _) = pollster::block_on(gpu_scene(graph.scene()))?;
    assert_eq!(cpu, gpu);
    Ok(())
}

#[test]
fn fractional_density_preserves_signed_logical_camera_offsets_and_fresh_stroke_geometry()
-> TestResult {
    let source = pair()?;
    let view = PageView::zoom(320, 240, source.size(), 175.5, Point2::new(80.0, 64.0)?)?;
    let raster = view.at_pixel_size(500, 375)?;
    assert_eq!(view.zoom_percent(), 175.5);
    assert_eq!(raster.zoom_percent(), 175.5);
    assert_eq!(raster.pixel_density(), 1.5625);
    assert_eq!(
        raster.canvas_position(Point2::new(32.0, 64.0)?)?,
        [119.0, 188.0]
    );
    assert_eq!(
        raster.canvas_position(Point2::new(96.0, 64.0)?)?,
        [294.5, 188.0]
    );
    let inverse = raster.document_position([119.0, 188.0])?;
    assert!((inverse.x_pt() - 32.0).abs() < 0.000_001);
    assert!((inverse.y_pt() - 64.0).abs() < 0.000_001);
    let graph = raster.graph_scene(&source, &Cancellation::default())?;
    assert_eq!(stroke(&graph)?.begin, [119.0, 188.0]);
    assert_eq!(stroke(&graph)?.end, [294.5, 188.0]);
    assert_eq!(stroke(&graph)?.width, 2.7421875);
    let cpu = cpu_scene(graph.scene())?;
    let (gpu, _) = pollster::block_on(gpu_scene(graph.scene()))?;
    assert_eq!(cpu, gpu);
    Ok(())
}

#[test]
fn capped_unequal_output_aspect_uses_min_density_and_centered_fractional_padding() -> TestResult {
    let source = page(PageSize::new(256.0, 256.0)?, &[], &[])?;
    let view = PageView::zoom(1000, 800, source.size(), 150.0, Point2::new(128.0, 128.0)?)?;
    let raster = view.at_pixel_size(2048, 1638)?;
    assert_eq!(raster.pixel_density(), 2.0475);
    assert_eq!(raster.zoom_percent(), 150.0);
    // Logical point(140,150)->(518,433); min(2048/1000,1638/800)=2.0475,
    // with0.25 physical pixel horizontal padding, zero vertical padding.
    near(
        raster.canvas_position(Point2::new(140.0, 150.0)?)?,
        [1060.855, 886.5675],
    );
    near(
        raster.canvas_position(Point2::new(128.0, 128.0)?)?,
        [1024.0, 819.0],
    );
    assert!((f64::from(raster.stroke_width()) - 3.07125).abs() <= 1.0 / 256.0);
    let graph = raster.graph_scene(&source, &Cancellation::default())?;
    assert_eq!([graph.scene().width, graph.scene().height], [2048, 1638]);
    Ok(())
}

fn font_oracle(payload: &serde_json::Value) -> TestResult<std::process::Output> {
    let python = std::env::var_os("BONDLINE_PYTHON").ok_or("pinned BONDLINE_PYTHON required")?;
    let mut child = Command::new(python)
        .arg("-B")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/element_label_oracle.py"
        ))
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../assets/fonts/Roboto-Regular.ttf"
        ))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut stdin = child.stdin.take().ok_or("oracle stdin")?;
    stdin.write_all(&serde_json::to_vec(payload)?)?;
    drop(stdin);
    Ok(child.wait_with_output()?)
}

#[test]
fn physical_nh2_masks_match_unchanged_font_oracle_and_are_not_scaled_logical_pixels() -> TestResult
{
    let source = page(
        PageSize::new(256.0, 256.0)?,
        &[(11, 7, [128.0, 128.0], None)],
        &[],
    )?;
    let before = source.clone();
    let counts = BTreeMap::from([(AtomId::new(11).ok_or("atom")?, 2)]);
    let view = PageView::zoom(400, 200, source.size(), 400.0, Point2::new(128.0, 128.0)?)?;
    let raster = view.at_pixel_size(800, 400)?;
    let graph =
        raster.graph_scene_with_display_hydrogens(&source, &counts, &Cancellation::default())?;
    assert_eq!(graph.label_em_px(), Some(96.0));
    assert_eq!(
        f64::from(graph.label_em_px().ok_or("em")?) / raster.pixel_density(),
        48.0
    );
    let boxes = graph
        .scene()
        .primitives
        .iter()
        .filter_map(|primitive| match primitive {
            Primitive::Rectangle(rect) if rect.color == [0, 0, 0, 255] => {
                Some([rect.x, rect.y, rect.width, rect.height])
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    let mut payload = json!({"symbol":"N","hydrogen_left":false,"hydrogens":2,"boxes":boxes});
    let positive = font_oracle(&payload)?;
    assert!(
        positive.status.success(),
        "{}",
        String::from_utf8_lossy(&positive.stderr)
    );
    payload["symbol"] = json!("O");
    let negative = font_oracle(&payload)?;
    assert!(
        !negative.status.success(),
        "wrong symbol passed independent font consumer"
    );
    let cpu = cpu_scene(graph.scene())?;
    let (gpu, _) = pollster::block_on(gpu_scene(graph.scene()))?;
    assert_eq!(cpu, gpu);
    let logical = cpu_scene(
        view.graph_scene_with_display_hydrogens(&source, &counts, &Cancellation::default())?
            .scene(),
    )?;
    let actual_ink = cpu
        .rgba
        .chunks_exact(4)
        .map(|pixel| pixel == [0, 0, 0, 255])
        .collect::<Vec<_>>();
    let nearest = (0..400)
        .flat_map(|y| (0..800).map(move |x| (x, y)))
        .map(|(x, y)| {
            let index = (((y / 2) * 400 + x / 2) * 4) as usize;
            logical.rgba.get(index..index + 4) == Some(&[0, 0, 0, 255][..])
        })
        .collect::<Vec<_>>();
    assert_ne!(
        actual_ink, nearest,
        "fresh96-em glyphs must not reuse48-em bitmap pixels"
    );
    assert_eq!(source, before);
    Ok(())
}

#[test]
fn output_validation_rejects_composition_invalid_sizes_and_unbounded_virtual_geometry() -> TestResult
{
    let view = PageView::zoom(
        100,
        100,
        PageSize::new(40_000.0, 40_000.0)?,
        100.0,
        Point2::new(64.0, 64.0)?,
    )?;
    for [width, height] in [[0, 1], [1, 0], [2049, 1], [1, 2049], [u32::MAX, u32::MAX]] {
        assert!(matches!(
            view.at_pixel_size(width, height),
            Err(Error::InvalidDimensions)
        ));
    }
    assert!(matches!(
        view.at_pixel_size(300, 300),
        Err(Error::InvalidViewport)
    ));
    let small = PageView::zoom(
        100,
        100,
        PageSize::new(128.0, 128.0)?,
        999.0,
        Point2::new(64.0, 64.0)?,
    )?;
    let raster = small.at_pixel_size(200, 200)?;
    assert_eq!(raster.zoom_percent(), 999.0);
    assert_eq!(raster.pixel_density(), 2.0);
    assert!(matches!(
        raster.at_pixel_size(100, 100),
        Err(Error::InvalidView)
    ));
    Ok(())
}

#[test]
fn display_counts_chemical_ink_and_cancellation_are_checked_before_camera_cropping() -> TestResult {
    let size = PageSize::new(256.0, 256.0)?;
    let source = page(size, &[(11, 8, [128.0, 128.0], None)], &[])?;
    let view =
        PageView::zoom(40, 48, size, 100.0, Point2::new(0.0, 0.0)?)?.at_pixel_size(80, 96)?;
    let counts = BTreeMap::from([(AtomId::new(11).ok_or("atom")?, 1)]);
    assert!(
        view.graph_scene_with_display_hydrogens(&source, &counts, &Cancellation::default())
            .is_ok()
    );
    assert!(matches!(
        view.graph_scene_with_display_hydrogens(
            &source,
            &BTreeMap::new(),
            &Cancellation::default()
        ),
        Err(Error::UnsupportedChemicalContent)
    ));
    let literal = page(size, &[(11, 8, [128.0, 128.0], Some(0))], &[])?;
    assert!(matches!(
        view.graph_scene_with_display_hydrogens(&literal, &counts, &Cancellation::default()),
        Err(Error::UnsupportedChemicalContent)
    ));
    let edge = page(size, &[(11, 8, [0.0, 128.0], None)], &[])?;
    assert!(matches!(
        view.graph_scene_with_display_hydrogens(&edge, &counts, &Cancellation::default()),
        Err(Error::LabelPlacement)
    ));
    let overlap = page(
        size,
        &[
            (11, 8, [128.0, 128.0], Some(1)),
            (19, 8, [128.25, 128.0], Some(1)),
        ],
        &[],
    )?;
    assert!(matches!(
        view.graph_scene(&overlap, &Cancellation::default()),
        Err(Error::LabelOverlap)
    ));
    let unsupported = page(size, &[(11, 1, [128.0, 128.0], Some(1))], &[])?;
    assert!(matches!(
        view.graph_scene(&unsupported, &Cancellation::default()),
        Err(Error::UnsupportedChemicalContent)
    ));
    let cancelled = Cancellation::default();
    cancelled.cancel();
    assert!(matches!(
        view.graph_scene(&source, &cancelled),
        Err(Error::Cancelled)
    ));
    assert!(matches!(
        view.graph_scene_with_display_hydrogens(&source, &counts, &cancelled),
        Err(Error::Cancelled)
    ));
    let tiny = PageView::zoom(100, 100, size, 10.0, Point2::new(128.0, 128.0)?)?
        .at_pixel_size(200, 200)?;
    assert!(matches!(
        tiny.graph_scene(&literal, &Cancellation::default()),
        Err(Error::LabelScale)
    ));
    let large = PageView::zoom(100, 100, size, 999.0, Point2::new(128.0, 128.0)?)?
        .at_pixel_size(300, 300)?;
    assert!(matches!(
        large.graph_scene(&literal, &Cancellation::default()),
        Err(Error::LabelScale)
    ));
    Ok(())
}

#[test]
fn nonsquare_fit_keeps_both_rounded_axes_at_fractional_and_subunit_density() -> TestResult {
    let source = page(
        PageSize::new(123.5, 81.25)?,
        &[(11, 6, [32.0, 64.0], None), (19, 6, [96.0, 64.0], None)],
        &[[11, 19]],
    )?;
    // Independently rounded logical paper:260x171 at(30,34). The axes differ.
    let logical = PageView::fit(320, 240, source.size())?;
    for (width, height, density) in [(480, 360, 1.5), (160, 120, 0.5)] {
        let raster = logical.at_pixel_size(width, height)?;
        assert_eq!(raster.pixel_density(), density);
        assert_eq!(raster.zoom_percent(), logical.zoom_percent());
        let expected = [
            density * (30.0 + 32.0 * 260.0 / 123.5),
            density * (34.0 + 64.0 * 171.0 / 81.25),
        ];
        near(raster.canvas_position(Point2::new(32.0, 64.0)?)?, expected);
        let graph = raster.graph_scene(&source, &Cancellation::default())?;
        near(stroke(&graph)?.begin, expected);
        assert!((f64::from(stroke(&graph)?.width) - density * 171.0 / 81.25).abs() <= 1.0 / 256.0);
        let cpu = cpu_scene(graph.scene())?;
        let (gpu, _) = pollster::block_on(gpu_scene(graph.scene()))?;
        assert_eq!(cpu, gpu);
    }
    Ok(())
}
