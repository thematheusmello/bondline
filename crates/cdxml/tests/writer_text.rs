use bondline_cdxml::{Error, MAX_INPUT_BYTES, MAX_NODES, Writer};
type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn writer_emits_owned_unicode_text_with_xml_line_endings_preserved() -> TestResult {
    let mut writer = Writer::new(&[])?;
    writer.start_element("t", &[])?;
    writer.start_element("s", &[])?;
    writer.text("β & < > \" ' \r\n\t")?;
    writer.text("tail")?;
    writer.end_element()?;
    writer.end_element()?;
    let bytes = writer.finish()?;
    assert_eq!(
        std::str::from_utf8(&bytes)?,
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><CDXML><t><s>β &amp; &lt; &gt; &quot; &apos; &#xD;&#xA;&#x9;tail</s></t></CDXML>"
    );
    Ok(())
}

#[test]
fn invalid_or_expanded_oversize_text_leaves_the_writer_usable() -> TestResult {
    let mut writer = Writer::new(&[])?;
    writer.start_element("s", &[])?;
    assert!(matches!(
        writer.text("before\0after"),
        Err(Error::InvalidTextValue)
    ));
    let too_large = "&".repeat(MAX_INPUT_BYTES / 5);
    assert!(matches!(writer.text(&too_large), Err(Error::InputTooLarge)));
    writer.text("ok")?;
    writer.end_element()?;
    assert_eq!(
        writer.finish()?,
        b"<?xml version=\"1.0\" encoding=\"UTF-8\"?><CDXML><s>ok</s></CDXML>"
    );
    Ok(())
}

#[test]
fn adjacent_text_chunks_share_one_syntax_node() -> TestResult {
    let mut writer = Writer::new(&[])?;
    for _ in 0..MAX_NODES {
        writer.text("a")?;
    }
    let bytes = writer.finish()?;
    assert_eq!(
        bytes.iter().filter(|&&b| b == b'a').count(),
        MAX_NODES as usize
    );
    Ok(())
}
