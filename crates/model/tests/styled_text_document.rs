use bondline_model::{Document, PageId, Point2, StyledText, TextId, TextRun, TextStyle};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn untouched_document_json_is_byte_exact_and_text_count_is_bounded_atomically() -> TestResult {
    let mut document = Document::default();
    assert_eq!(
        serde_json::to_string(&document)?,
        r#"{"pages":[{"id":1,"size":{"width_pt":595.2755905511812,"height_pt":841.8897637795276},"fragments":[]}]}"#
    );
    let before = document.clone();
    let run = TextRun::new(
        String::new(),
        TextStyle::new("Roboto".into(), 12.0, 0, [0.0; 3])?,
    )?;
    let texts = (1..=100_001)
        .map(|id| {
            StyledText::new(
                TextId::new(id).ok_or("text id")?,
                Point2::new(0.0, 0.0)?,
                vec![run.clone()],
            )
            .map_err(Into::into)
        })
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
    assert!(
        document
            .append_texts(PageId::new(1).ok_or("page")?, texts)
            .is_err()
    );
    assert_eq!(document, before);
    Ok(())
}
