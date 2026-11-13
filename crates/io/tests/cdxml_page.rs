use bondline_io::StoredDocument;
use bondline_model::PageSize;

#[test]
fn complete_poster_extents_override_only_the_explicit_preview_fallback()
-> Result<(), Box<dyn std::error::Error>> {
    let fallback = PageSize::new(72.0, 144.0)?;
    for (properties, expected) in [
        (
            "DrawingSpace='Poster' Width='312.5' Height='456.25'",
            PageSize::new(312.5, 456.25)?,
        ),
        (
            "DrawingSpace='Pages' Width='not-an-extent' Height='-1'",
            fallback,
        ),
        ("Width='312.5' Height='456.25'", fallback),
        ("DrawingSpace='Poster'", fallback),
    ] {
        let source = format!("<CDXML><page id='5' {properties}/></CDXML>");
        let stored = StoredDocument::from_cdxml(source.as_bytes().into(), fallback)?;
        assert_eq!(
            stored.model().pages().first().ok_or("page")?.size(),
            expected,
            "{properties}"
        );
        assert_eq!(stored.export_cdxml()?, source.as_bytes());
    }
    Ok(())
}

#[test]
fn incomplete_or_invalid_explicit_poster_dimensions_are_rejected()
-> Result<(), Box<dyn std::error::Error>> {
    for properties in [
        "Width='72'",
        "Height='144'",
        "Width='0' Height='144'",
        "Width='72' Height='-1'",
        "Width='NaN' Height='144'",
        "Width='72' Height='inf'",
        "Width='broken' Height='144'",
        "Width='100001' Height='144'",
    ] {
        let source = format!("<CDXML><page DrawingSpace='Poster' {properties}/></CDXML>");
        assert!(
            StoredDocument::from_cdxml(source.as_bytes().into(), PageSize::A4).is_err(),
            "accepted {properties}"
        );
        assert_eq!(
            bondline_cdxml::Document::parse(source.as_bytes())?.to_bytes(),
            source.as_bytes(),
            "syntax retains unsupported properties"
        );
    }
    Ok(())
}
