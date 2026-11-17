use bondline_cdxml::Document;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn styled_text_keeps_ordered_unicode_and_explicit_run_selectors() -> TestResult {
    const SOURCE: &str = "<CDXML><page><t id='31'><s font='7.00' size='12.5' face='96' color='4'>O&amp;H&#x2082; μ<![CDATA[&]]></s>\r\n<s>λ</s></t></page></CDXML>";
    let document = Document::parse(SOURCE.as_bytes())?;
    let element = document
        .xml()
        .descendants()
        .find(|node| node.has_tag_name("t"))
        .ok_or("t")?;
    let text = document.styled_text(element)?;
    assert_eq!(text.element(), element);
    assert_eq!(text.runs().len(), 2);
    let first = text.runs().first().ok_or("first run")?;
    assert_eq!(first.text(), "O&H₂ μ&");
    assert_eq!(
        (first.font(), first.size_pt(), first.face(), first.color()),
        (Some(7), Some(12.5), Some(96), Some(4))
    );
    let second = text.runs().get(1).ok_or("second run")?;
    assert_eq!(second.text(), "λ");
    assert_eq!(
        (
            second.font(),
            second.size_pt(),
            second.face(),
            second.color()
        ),
        (None, None, None, None)
    );
    assert_eq!(document.as_bytes(), SOURCE.as_bytes());
    Ok(())
}

// Expanded declarations frozen before production; initial declaration retained above.
use bondline_cdxml::{Edit, ResourceError, TextError, TextObject};
use std::{
    io::Write,
    process::{Command, Output, Stdio},
};

fn object<'doc, 'input: 'doc>(
    document: &'doc Document<'input>,
) -> Result<TextObject<'doc, 'input>, TextError> {
    let element = document
        .xml()
        .descendants()
        .find(|node| {
            node.is_element()
                && node.tag_name().name() == "t"
                && node.tag_name().namespace().is_none_or(str::is_empty)
        })
        .ok_or(TextError::InvalidElement)?;
    document.styled_text(element)
}

#[test]
fn missing_selectors_empty_runs_and_resources_never_invent_defaults() -> TestResult {
    let source = b"<CDXML LabelFont='7' LabelSize='14' CaptionFont='9'><t extra='keep'><s font='42' unknown='retain'/><s> </s></t></CDXML>";
    let document = Document::parse(source)?;
    let text = object(&document)?;
    assert_eq!(text.element().attribute("extra"), Some("keep"));
    assert_eq!(text.runs().len(), 2);
    let first = text.runs().first().ok_or("run")?;
    assert_eq!(first.text(), "");
    assert_eq!(first.font(), Some(42));
    assert_eq!(
        (first.size_pt(), first.face(), first.color()),
        (None, None, None)
    );
    assert_eq!(first.element().attribute("unknown"), Some("retain"));
    let second = text.runs().get(1).ok_or("run")?;
    assert_eq!(second.text(), " ");
    assert_eq!(
        (
            second.font(),
            second.size_pt(),
            second.face(),
            second.color()
        ),
        (None, None, None, None)
    );
    assert!(matches!(
        document.resources()?.font(42),
        Err(ResourceError::MissingFont(42))
    ));
    assert_eq!(document.to_bytes(), source);
    Ok(())
}

#[test]
fn exact_uint16_selectors_and_positive_size_keep_their_separate_domains() -> TestResult {
    for (raw, expected) in [("0", 0), ("+7.00", 7), (" 65535.000&#9;", 65535)] {
        let source = format!("<CDXML><t><s font='{raw}' face='{raw}' color='{raw}'/></t></CDXML>");
        let document = Document::parse(source.as_bytes())?;
        let text = object(&document)?;
        let run = text.runs().first().ok_or("run")?;
        assert_eq!(
            (run.font(), run.face(), run.color()),
            (Some(expected), Some(expected), Some(expected))
        );
    }
    for raw in [".5", "12.50", "1e2", "5e-324", "1.7976931348623157e308"] {
        let source = format!("<CDXML><t><s size='{raw}'/></t></CDXML>");
        let document = Document::parse(source.as_bytes())?;
        assert_eq!(
            object(&document)?.runs().first().ok_or("run")?.size_pt(),
            Some(raw.parse::<f64>()?)
        );
    }
    for property in ["font", "face", "color"] {
        for bad in ["-1", "65536", "1.01", "7e0", "", "NaN", "1&#160;"] {
            let source = format!("<CDXML><t><s {property}='{bad}'/></t></CDXML>");
            let document = Document::parse(source.as_bytes())?;
            assert_eq!(
                object(&document).err(),
                Some(TextError::InvalidProperty(property))
            );
            assert_eq!(document.as_bytes(), source.as_bytes());
        }
    }
    for bad in ["0", "-0", "-1", "NaN", "inf", "1e999", "1&#160;", ""] {
        let source = format!("<CDXML><t><s size='{bad}'/></t></CDXML>");
        let document = Document::parse(source.as_bytes())?;
        assert_eq!(
            object(&document).err(),
            Some(TextError::InvalidProperty("size"))
        );
    }
    Ok(())
}

#[test]
fn ordinary_attributes_and_empty_namespace_resets_are_distinct_from_lookalikes() -> TestResult {
    let source = b"<CDXML><t xmlns='' xmlns:x='urn:other'><s xmlns='' x:font='bad' font='7' x:size='bad' size=' 12.5&#9;' x:face='bad' face='0' x:color='bad' color='4'>A</s><s x:font='9' x:size='99' x:face='5' x:color='6'>B</s></t></CDXML>";
    let document = Document::parse(source)?;
    let text = object(&document)?;
    let run = text.runs().first().ok_or("run")?;
    assert_eq!(
        (run.font(), run.size_pt(), run.face(), run.color()),
        (Some(7), Some(12.5), Some(0), Some(4))
    );
    assert!(
        run.element()
            .attributes()
            .any(|a| a.namespace() == Some("urn:other")
                && a.name() == "font"
                && a.value() == "bad")
    );
    let run = text.runs().get(1).ok_or("run")?;
    assert_eq!(
        (run.font(), run.size_pt(), run.face(), run.color()),
        (None, None, None, None)
    );
    assert_eq!(document.as_bytes(), source);
    Ok(())
}

#[test]
fn nodes_must_be_ordinary_text_elements_owned_by_this_exact_document() -> TestResult {
    let source = b"<CDXML xmlns:x='urn:other'><page/><t><s>A</s></t><x:t><s>B</s></x:t></CDXML>";
    let document = Document::parse(source)?;
    let other = Document::parse(source)?;
    let foreign = other
        .xml()
        .descendants()
        .find(|node| node.has_tag_name("t"))
        .ok_or("foreign t")?;
    assert_eq!(
        document.styled_text(foreign).err(),
        Some(TextError::InvalidElement)
    );
    for element in document
        .xml()
        .descendants()
        .filter(|node| node.is_element())
    {
        if element.tag_name().name() == "t" && element.tag_name().namespace().is_none() {
            assert_eq!(document.styled_text(element)?.runs().len(), 1);
        } else {
            assert_eq!(
                document.styled_text(element).err(),
                Some(TextError::InvalidElement)
            );
        }
    }
    assert_eq!(
        document.styled_text(document.xml().root()).err(),
        Some(TextError::InvalidElement)
    );
    assert_eq!(document.as_bytes(), source);
    Ok(())
}

#[test]
fn missing_runs_and_mixed_content_fail_without_exposing_a_partial_view() -> TestResult {
    for body in ["", " &#9;&#13;\n"] {
        let source = format!("<CDXML><t>{body}</t></CDXML>");
        let document = Document::parse(source.as_bytes())?;
        assert_eq!(object(&document).err(), Some(TextError::MissingRuns));
    }
    for body in [
        "text<s/>",
        "<s/>tail",
        "&#160;<s/>",
        "<n/>",
        "<x:s xmlns:x='urn:other'/>",
        "<!--keep--><s/>",
        "<?keep data?><s/>",
        "<s><n/></s>",
        "<s>A<!--keep-->B</s>",
        "<s>A<?keep data?>B</s>",
    ] {
        let source = format!("<CDXML><t>{body}</t></CDXML>");
        let document = Document::parse(source.as_bytes())?;
        assert!(
            matches!(object(&document), Err(TextError::UnexpectedContent(_))),
            "{body}"
        );
        assert_eq!(document.as_bytes(), source.as_bytes());
    }
    let source = b"<CDXML><t><s font='7'>valid</s><s font='65536'>invalid later</s></t></CDXML>";
    let document = Document::parse(source)?;
    assert_eq!(
        object(&document).err(),
        Some(TextError::InvalidProperty("font"))
    );
    assert_eq!(document.as_bytes(), source);
    Ok(())
}

#[test]
fn standalone_view_keeps_syntax_bounds_separate_from_future_model_budgets() -> TestResult {
    let source = format!("<CDXML><t>{}</t></CDXML>", "<s/>".repeat(4097));
    let document = Document::parse(source.as_bytes())?;
    assert_eq!(object(&document)?.runs().len(), 4097);
    let value = "A".repeat(1024 * 1024 + 1);
    let source = format!("<CDXML><t><s>{value}</s></t></CDXML>");
    let document = Document::parse(source.as_bytes())?;
    let text = object(&document)?;
    assert_eq!(text.runs().first().ok_or("run")?.text(), value);
    assert_eq!(document.as_bytes(), source.as_bytes());
    Ok(())
}

fn independent(bytes: &[u8], expected: &str) -> TestResult<Output> {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child = Command::new(python)
        .args([
            "-X",
            "utf8",
            "-B",
            "-c",
            include_str!("styled_text_oracle.py"),
            expected,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child.stdin.take().ok_or("oracle stdin")?.write_all(bytes)?;
    Ok(child.wait_with_output()?)
}
fn quoted(text: &str) -> String {
    let mut result = String::from("\"");
    for character in text.chars() {
        match character {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c if c < ' ' => result.push_str(&format!("\\u{:04x}", u32::from(c))),
            c => result.push(c),
        }
    }
    result.push('"');
    result
}
fn payload(text: &TextObject<'_, '_>) -> String {
    fn number<T: std::fmt::Display>(value: Option<T>) -> String {
        value.map_or_else(|| "null".into(), |value| value.to_string())
    }
    format!(
        "[{}]",
        text.runs()
            .iter()
            .map(|run| format!(
                "[{},{},{},{},{}]",
                quoted(run.text()),
                number(run.font()),
                number(run.size_pt()),
                number(run.face()),
                number(run.color())
            ))
            .collect::<Vec<_>>()
            .join(",")
    )
}

#[test]
fn independent_expat_decimal_qualifier_rejects_semantic_and_style_changes() -> TestResult {
    let source =
        "<CDXML><t><s font='7.00' size='12.50' face='96' color='4'>A&amp;B</s><s>μ</s></t></CDXML>";
    let positive = r#"[["A&B",7,12.5,96,4],["μ",null,null,null,null]]"#;
    let output = independent(source.as_bytes(), positive)?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    for wrong in [
        r#"[["A&amp;B",7,12.5,96,4],["μ",null,null,null,null]]"#,
        r#"[["A&B",8,12.5,96,4],["μ",null,null,null,null]]"#,
        r#"[["A&B",7,12.6,96,4],["μ",null,null,null,null]]"#,
        r#"[["A&B",7,12.5,0,4],["μ",null,null,null,null]]"#,
        r#"[["A&B",7,12.5,96,3],["μ",null,null,null,null]]"#,
        r#"[["μ",null,null,null,null],["A&B",7,12.5,96,4]]"#,
        r#"[["A&B",true,12.5,96,4],["μ",null,null,null,null]]"#,
    ] {
        assert_eq!(
            independent(source.as_bytes(), wrong)?.status.code(),
            Some(1)
        );
    }
    Ok(())
}

#[test]
fn checked_run_edits_preserve_raw_source_and_match_independent_decoded_values() -> TestResult {
    const SOURCE: &str = "<?xml version='1.0'?>\r\n<CDXML><!--keep--><?keep data?><t id='31' unknown='retained'><s font='7.00' size='12.50' face='96' color='4'>A<![CDATA[&]]>B&#x2082;</s><s>μ</s></t></CDXML>";
    const EXPECTED: &str = "<?xml version='1.0'?>\r\n<CDXML><!--keep--><?keep data?><t id='31' unknown='retained'><s font='7.00' size='12.50' face='96' color='4'>O&amp;H₂ λ</s><s>μ</s></t></CDXML>";
    // Qualify literal inputs before passing any parser outputs to the independent process.
    for (bytes, expected) in [
        (
            SOURCE.as_bytes(),
            r#"[["A&B₂",7,12.5,96,4],["μ",null,null,null,null]]"#,
        ),
        (
            EXPECTED.as_bytes(),
            r#"[["O&H₂ λ",7,12.5,96,4],["μ",null,null,null,null]]"#,
        ),
    ] {
        let output = independent(bytes, expected)?;
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let document = Document::parse(SOURCE.as_bytes())?;
    let text = object(&document)?;
    assert!(
        independent(document.as_bytes(), &payload(&text))?
            .status
            .success()
    );
    let first = text.runs().first().ok_or("run")?;
    let noop = document.apply_edits(&[Edit::SetText {
        element: first.element(),
        value: "A&B₂",
    }])?;
    assert_eq!(noop, SOURCE.as_bytes());
    let output = document.apply_edits(&[Edit::SetText {
        element: first.element(),
        value: "O&H₂ λ",
    }])?;
    assert_eq!(output, EXPECTED.as_bytes());
    let reopened = Document::parse(&output)?;
    let actual = object(&reopened)?;
    assert_eq!(actual.element().attribute("unknown"), Some("retained"));
    assert!(independent(&output, &payload(&actual))?.status.success());
    assert_eq!(document.as_bytes(), SOURCE.as_bytes());
    Ok(())
}
