//! Original analytic geometry; explicit percentages use the preferred bond length.
use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Document, DoublePosition, DrawingStyle, Element,
    Fragment, FragmentId, Page, PageSize, Point2,
};
use bondline_render::{Cancellation, GraphScene, PageView, Stroke};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn page(length: f64, order: BondOrder, side: DoublePosition) -> TestResult<Page> {
    let atoms = [32.0, 32.0 + length]
        .into_iter()
        .enumerate()
        .map(|(i, x)| {
            Ok(Atom::new(
                AtomId::new(u32::try_from(i + 1)?).ok_or("atom")?,
                Element::try_from(6)?,
                Some(Point2::new(x, 64.125)?),
            ))
        })
        .collect::<TestResult<Vec<_>>>()?;
    let mut document = Document::default();
    let page_id = document.pages()[0].id();
    document.set_page_size(page_id, PageSize::new(256.0, 256.0)?)?;
    document.append_fragments(
        page_id,
        vec![Fragment::new(
            FragmentId::new(1).ok_or("fragment")?,
            atoms,
            vec![Bond {
                id: BondId::new(1).ok_or("bond")?,
                begin: AtomId::new(1).ok_or("begin")?,
                end: AtomId::new(2).ok_or("end")?,
                order,
                double_position: side,
            }],
        )?],
    )?;
    Ok(document.pages()[0].clone())
}

fn strokes(graph: &GraphScene) -> TestResult<Vec<Stroke>> {
    Ok(graph
        .bond_strokes(
            FragmentId::new(1).ok_or("fragment")?,
            BondId::new(1).ok_or("bond")?,
        )
        .cloned()
        .collect())
}

#[test]
fn explicit_root_percentage_changes_centerlines_using_original_48pt_fallback() -> TestResult {
    let page = page(40.0, BondOrder::Double, DoublePosition::Center)?;
    let before = page.clone();
    // Existing public style/view API; no future renderer method can mask runtime behavior.
    let style: DrawingStyle = serde_json::from_str(r#"{"bond_spacing_percent":15.0}"#)?;
    let graph = PageView::fit(576, 576, page.size())?
        .with_drawing_style(style)
        .graph_scene(&page, &Cancellation::default())?;
    let actual = strokes(&graph)?;
    assert_eq!(actual.len(), 2);
    for (stroke, y) in actual.iter().zip([153.05, 167.45]) {
        assert!((f64::from(stroke.begin[1]) - y).abs() < 1.0 / 4096.0);
        assert!((f64::from(stroke.end[1]) - y).abs() < 1.0 / 4096.0);
        assert_eq!([stroke.begin[0], stroke.end[0]], [96.0, 176.0]);
        assert_eq!(stroke.width, 2.0);
    }
    assert_eq!(page, before);
    Ok(())
}
