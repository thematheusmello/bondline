use bondline_model::{Document, PageId, Point2, StyledText, TextId, TextRun, TextStyle};
use serde_json::json;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn literal_styled_caption_retains_ordered_unicode_runs_and_empty_page_json() -> TestResult {
    let mut document = Document::default();
    let page = PageId::new(1).ok_or("page")?;
    let empty = serde_json::to_value(&document)?;
    assert_eq!(empty["pages"][0]["fragments"], json!([]));
    assert!(empty["pages"][0].get("texts").is_none());
    let style = TextStyle::new("Roboto".into(), 18.0, 0, [0.125, 0.25, 1.0])?;
    let first = TextRun::new(" Aλ & ".into(), style.clone())?;
    let second = TextRun::new(
        "CO₂".into(),
        TextStyle::new("Roboto".into(), 12.0, 96, [0.0; 3])?,
    )?;
    let caption = StyledText::new(
        TextId::new(7).ok_or("text id")?,
        Point2::new(72.5, 96.25)?,
        vec![first, second],
    )?;
    assert_eq!(caption.runs()[0].style(), &style);
    document.append_texts(page, vec![caption])?;
    let actual = &document.pages()[0].texts()[0];
    assert_eq!(actual.id().get(), 7);
    assert_eq!(actual.position(), Point2::new(72.5, 96.25)?);
    assert_eq!(
        serde_json::to_value(actual)?,
        json!({
            "id":7,"position":{"x_pt":72.5,"y_pt":96.25},
            "runs":[
                {"text":" Aλ & ","style":{"font_family":"Roboto","size_pt":18.0,"face":0,"rgb":[0.125,0.25,1.0]}},
                {"text":"CO₂","style":{"font_family":"Roboto","size_pt":12.0,"face":96,"rgb":[0.0,0.0,0.0]}}
            ]
        })
    );
    Ok(())
}
