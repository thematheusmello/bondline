//! Original caption declaration: model API qualification, then actual renderer RED.
use bondline_model::{Document, PageSize, Point2, StyledText, TextId, TextRun, TextStyle};
use bondline_render::{Primitive, graph_page_scene};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn explicit_regular_roboto_caption_paints_its_actual_red_glyph() -> TestResult {
    let mut document = Document::default();
    let page_id = document.pages()[0].id();
    document.set_page_size(page_id, PageSize::new(256.0, 256.0)?)?;
    document.append_texts(
        page_id,
        vec![StyledText::new(
            TextId::new(7).ok_or("text")?,
            Point2::new(32.0, 96.0)?,
            vec![TextRun::new(
                "H".to_owned(),
                TextStyle::new("Roboto".to_owned(), 12.0, 0, [1.0, 0.0, 0.0])?,
            )?],
        )?],
    )?;
    let before = document.clone();
    let graph = graph_page_scene(576, 576, &document.pages()[0])?;
    assert!(
        graph.scene().primitives.iter().any(|primitive| {
            matches!(primitive, Primitive::Rectangle(rect) if rect.color == [255, 0, 0, 255])
        }),
        "the actual page caption must paint, not an empty chemical projection"
    );
    assert_eq!(
        graph.label_em_px(),
        None,
        "chemical label metrics stay separate"
    );
    assert_eq!(document, before);
    Ok(())
}
