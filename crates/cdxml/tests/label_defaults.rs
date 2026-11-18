use bondline_cdxml::Document;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn label_defaults_resolve_each_property_from_its_actual_source_owner() -> TestResult {
    const SOURCE: &[u8] = br#"<CDXML LabelFont='3' LabelSize='10' LabelFace='96'><page><fragment><n id='11' LabelSize='14.5'><t LabelFace='0'><s font='7'>OH</s></t></n></fragment></page></CDXML>"#;
    let document = Document::parse(SOURCE)?;
    let text = document
        .xml()
        .descendants()
        .find(|n| n.has_tag_name("t"))
        .ok_or("t")?;
    let defaults = document.label_defaults(text)?;
    let font = defaults.font().ok_or("font")?;
    assert_eq!(font.value(), 3);
    assert_eq!(font.owner(), document.xml().root_element());
    assert_eq!(font.attribute_value(), "3");
    let size = defaults.size_pt().ok_or("size")?;
    assert_eq!(size.value(), 14.5);
    assert_eq!(size.owner().attribute("id"), Some("11"));
    assert_eq!(defaults.face().ok_or("face")?.value(), 0);
    assert_eq!(defaults.face().ok_or("face")?.owner(), text);
    assert_eq!(document.styled_text(text)?.runs()[0].size_pt(), None);
    assert_eq!(document.to_bytes(), SOURCE);
    Ok(())
}

// Independent expanded declarations; append after the frozen initial test unchanged.
use bondline_cdxml::{Edit, LabelDefaultsError, ResourceError};
use std::{
    io::Write,
    process::{Command, Output, Stdio},
};

type LabelValueResult<T> = Result<T, Box<dyn std::error::Error>>;

fn label_node<'doc, 'input: 'doc>(
    document: &'doc Document<'input>,
    id: &str,
) -> LabelValueResult<roxmltree::Node<'doc, 'input>> {
    document
        .xml()
        .descendants()
        .find(|node| {
            node.is_element()
                && node.attributes().any(|attribute| {
                    attribute.namespace().is_none()
                        && attribute.name() == "id"
                        && attribute.value() == id
                })
        })
        .ok_or_else(|| format!("missing test node {id}").into())
}

fn label_error(error: LabelDefaultsError) -> (&'static str, Option<&'static str>) {
    match error {
        LabelDefaultsError::InvalidElement => ("InvalidElement", None),
        LabelDefaultsError::InvalidProperty(property) => ("InvalidProperty", Some(property)),
        LabelDefaultsError::UnsupportedOwner(property) => ("UnsupportedOwner", Some(property)),
    }
}

fn expect_label_error(
    source: &str,
    id: &str,
    expected: (&'static str, Option<&'static str>),
) -> TestResult {
    let document = Document::parse(source.as_bytes())?;
    let error = document
        .label_defaults(label_node(&document, id)?)
        .err()
        .ok_or("lookup unexpectedly accepted")?;
    assert_eq!(label_error(error), expected, "{source}");
    assert_eq!(document.as_bytes(), source.as_bytes());
    assert_eq!(document.to_bytes(), source.as_bytes());
    Ok(())
}

#[test]
fn every_documented_lookup_role_is_accepted_without_claiming_object_layout() -> TestResult {
    let source=b"<CDXML id='1' LabelFont='7' LabelSize='12.5' LabelFace='96'><page><n id='2'/><b id='3'/><t id='4'><s/></t><table id='5'/><spectrum id='6'/><constraint id='7'/><tlcplate id='8'/></page></CDXML>";
    let document = Document::parse(source)?;
    for (id, role) in [
        ("1", "CDXML"),
        ("2", "n"),
        ("3", "b"),
        ("4", "t"),
        ("5", "table"),
        ("6", "spectrum"),
        ("7", "constraint"),
        ("8", "tlcplate"),
    ] {
        let target = label_node(&document, id)?;
        assert_eq!(target.tag_name().name(), role);
        let defaults = document.label_defaults(target)?;
        let font = defaults.font().ok_or("font")?;
        let copied_font = font;
        assert_eq!((font.value(), copied_font.value()), (7_i16, 7_i16));
        assert_eq!(font.owner(), document.xml().root_element());
        assert_eq!(font.attribute_value(), "7");
        assert_eq!(defaults.size_pt().ok_or("size")?.value(), 12.5);
        assert_eq!(defaults.face().ok_or("face")?.value(), 96_i16);
    }
    assert_eq!(document.as_bytes(), source);
    Ok(())
}

#[test]
fn missing_and_partial_defaults_never_become_resources_or_style_run_values() -> TestResult {
    for attributes in ["", "LabelSize='12.5'"] {
        let source = format!(
            "<CDXML id='1' {attributes}><fonttable><font id='65535' name='Unresolved family' charset='unknown'/></fonttable><page><n id='11'><t id='21'><s font='65535' size='24' face='65535'>OH</s></t></n></page></CDXML>"
        );
        let document = Document::parse(source.as_bytes())?;
        for id in ["1", "11", "21"] {
            let defaults = document.label_defaults(label_node(&document, id)?)?;
            assert!(defaults.font().is_none());
            assert!(defaults.face().is_none());
            assert_eq!(
                defaults.size_pt().map(|value| value.value()),
                if attributes.is_empty() {
                    None
                } else {
                    Some(12.5)
                }
            );
        }
        assert_eq!(
            document.resources()?.font(65535)?.name(),
            "Unresolved family"
        );
        assert!(matches!(
            document.resources()?.font(0),
            Err(ResourceError::MissingFont(0))
        ));
        let styled = document.styled_text(label_node(&document, "21")?)?;
        let run = styled.runs().first().ok_or("run")?;
        assert_eq!(
            (run.font(), run.size_pt(), run.face()),
            (Some(65535), Some(24.0), Some(65535))
        );
        assert_eq!(document.as_bytes(), source.as_bytes());
    }
    let document = Document::parse(
        b"<CDXML LabelFont='7' LabelSize='12' LabelFace='96'><t id='21'><s>OH</s></t></CDXML>",
    )?;
    assert_eq!(
        document
            .label_defaults(label_node(&document, "21")?)?
            .font()
            .ok_or("font")?
            .value(),
        7
    );
    let text = document.styled_text(label_node(&document, "21")?)?;
    let run = text.runs().first().ok_or("run")?;
    assert_eq!((run.font(), run.size_pt(), run.face()), (None, None, None));
    assert!(
        matches!(
            document.resources()?.font(7),
            Err(ResourceError::MissingFont(7))
        ),
        "default scalar lookup must not require a present font resource"
    );
    Ok(())
}

#[test]
fn label_integer_defaults_are_signed_data_distinct_from_unsigned_runs_and_fonts() -> TestResult {
    for (raw, expected) in [
        ("-32768", i16::MIN),
        ("32767", i16::MAX),
        (" +7.000 ", 7),
        ("-0.00", 0),
    ] {
        let source = format!("<CDXML LabelFont='{raw}' LabelFace='{raw}'><n id='11'/></CDXML>");
        let document = Document::parse(source.as_bytes())?;
        let defaults = document.label_defaults(label_node(&document, "11")?)?;
        assert_eq!(defaults.font().ok_or("font")?.value(), expected);
        assert_eq!(defaults.face().ok_or("face")?.value(), expected);
        assert_eq!(defaults.font().ok_or("font")?.attribute_value(), raw);
        assert_eq!(document.as_bytes(), source.as_bytes());
    }
    for property in ["LabelFont", "LabelFace"] {
        for raw in [
            "-32769",
            "32768",
            "65535",
            "1.5",
            "1e0",
            "",
            "NaN",
            "9999999999999999999999999999",
        ] {
            let source = format!("<CDXML {property}='{raw}'><n id='11'/></CDXML>");
            expect_label_error(&source, "11", ("InvalidProperty", Some(property)))?;
        }
    }
    let source=b"<CDXML LabelFont='-1' LabelFace='-32768'><fonttable><font id='65535' name='Unsigned resource'/></fonttable><t id='21'><s font='65535' face='65535'>A</s></t></CDXML>";
    let document = Document::parse(source)?;
    let defaults = document.label_defaults(label_node(&document, "21")?)?;
    assert_eq!(defaults.font().ok_or("font")?.value(), -1_i16);
    assert_eq!(defaults.face().ok_or("face")?.value(), i16::MIN);
    assert_eq!(
        document.resources()?.font(65535)?.name(),
        "Unsigned resource"
    );
    let text = document.styled_text(label_node(&document, "21")?)?;
    let run = text.runs().first().ok_or("run")?;
    assert_eq!((run.font(), run.face()), (Some(u16::MAX), Some(u16::MAX)));
    assert_eq!(document.as_bytes(), source);
    Ok(())
}

#[test]
fn xml_label_size_policy_is_positive_finite_without_an_extra_font_maximum() -> TestResult {
    for (raw, expected) in [
        (".125", 0.125),
        (" 12.50 ", 12.5),
        ("+3.25e2", 325.0),
        ("1e-9", 1e-9),
        ("100001", 100001.0),
        ("1e308", 1e308),
        ("5e-324", 5e-324),
    ] {
        let source = format!("<CDXML LabelSize='{raw}'><n id='11'/></CDXML>");
        let document = Document::parse(source.as_bytes())?;
        let size = document
            .label_defaults(label_node(&document, "11")?)?
            .size_pt()
            .ok_or("size")?;
        assert_eq!(size.value(), expected, "{raw}");
        assert_eq!(size.attribute_value(), raw);
        assert_eq!(document.as_bytes(), source.as_bytes());
    }
    for raw in [
        "0", "-0", "-0.01", "-1e-9999", "1e-9999", "NaN", "inf", "-inf", "1e309", "", "\u{a0}12",
        "12\u{a0}",
    ] {
        let source = format!("<CDXML LabelSize='{raw}'><n id='11'/></CDXML>");
        expect_label_error(&source, "11", ("InvalidProperty", Some("LabelSize")))?;
    }
    Ok(())
}

#[test]
fn ordinary_defaults_on_unsupported_containing_owners_are_never_skipped() -> TestResult {
    for owner in ["page", "group", "fragment", "geometry", "x:n"] {
        for property in ["LabelFont", "LabelSize", "LabelFace"] {
            let source = format!(
                "<CDXML xmlns:x='urn:other' LabelFont='3' LabelSize='10' LabelFace='96'><{owner} {property}='bad'><t id='21' {property}='7'><s/></t></{owner}></CDXML>"
            );
            expect_label_error(&source, "21", ("UnsupportedOwner", Some(property)))?;
        }
    }
    let source=b"<CDXML LabelFont='3'><page><group><fragment><n id='11'/></fragment></group></page></CDXML>";
    let document = Document::parse(source)?;
    assert_eq!(
        document
            .label_defaults(label_node(&document, "11")?)?
            .font()
            .ok_or("font")?
            .value(),
        3
    );
    assert_eq!(document.as_bytes(), source);
    Ok(())
}

#[test]
fn shadowed_invalid_values_and_diagnostics_follow_nearest_owner_then_property_order() -> TestResult
{
    let cases = [
        (
            "<CDXML LabelFont='65535'><n id='11' LabelFont='7'/></CDXML>",
            "LabelFont",
        ),
        (
            "<CDXML LabelSize='0'><n id='11' LabelSize='12'/></CDXML>",
            "LabelSize",
        ),
        (
            "<CDXML LabelFace='32768'><n id='11' LabelFace='0'/></CDXML>",
            "LabelFace",
        ),
        (
            "<CDXML LabelFont='bad'><n id='11' LabelFace='bad'/></CDXML>",
            "LabelFace",
        ),
        (
            "<CDXML LabelFont='bad'><n id='11' LabelSize='0'/></CDXML>",
            "LabelSize",
        ),
        (
            "<CDXML><n id='11' LabelFace='bad' LabelSize='0' LabelFont='bad'/></CDXML>",
            "LabelFont",
        ),
    ];
    for (source, property) in cases {
        expect_label_error(source, "11", ("InvalidProperty", Some(property)))?;
    }
    let source = b"<CDXML id='1' LabelFont='3'><page LabelFont='bad'><n id='11'/></page></CDXML>";
    let document = Document::parse(source)?;
    assert_eq!(
        document
            .label_defaults(label_node(&document, "1")?)?
            .font()
            .ok_or("font")?
            .value(),
        3,
        "only the requested element and its ancestors participate"
    );
    assert_eq!(document.as_bytes(), source);
    Ok(())
}

#[test]
fn target_identity_namespace_resets_and_qualified_lookalikes_are_distinct() -> TestResult {
    let source=b"<CDXML xmlns:x='urn:other' id='1' LabelFont='3' x:LabelFont='bad'><page id='2'><x:n xmlns='urn:other' id='9' x:LabelSize='bad'><n xmlns='' id='11' LabelSize='14.5' x:LabelFont='bad'><t id='21' LabelFace='0'><s id='22'>OH</s></t></n></x:n><geometry id='23'/><CDXML id='24'/><!--keep--></page></CDXML>";
    let document = Document::parse(source)?;
    let target = label_node(&document, "21")?;
    let defaults = document.label_defaults(target)?;
    assert_eq!(defaults.font().ok_or("font")?.value(), 3);
    assert_eq!(defaults.size_pt().ok_or("size")?.value(), 14.5);
    assert_eq!(
        defaults.size_pt().ok_or("size")?.owner(),
        label_node(&document, "11")?
    );
    assert_eq!(defaults.face().ok_or("face")?.value(), 0);
    for id in ["2", "9", "22", "23", "24"] {
        let error = document
            .label_defaults(label_node(&document, id)?)
            .err()
            .ok_or("unsupported target accepted")?;
        assert_eq!(label_error(error), ("InvalidElement", None), "{id}");
    }
    for node in [
        document.xml().root(),
        document
            .xml()
            .descendants()
            .find(|node| node.is_text())
            .ok_or("text")?,
        document
            .xml()
            .descendants()
            .find(|node| node.is_comment())
            .ok_or("comment")?,
    ] {
        let error = document
            .label_defaults(node)
            .err()
            .ok_or("non-element accepted")?;
        assert_eq!(label_error(error), ("InvalidElement", None));
    }
    let identical = Document::parse(source)?;
    let error = document
        .label_defaults(label_node(&identical, "21")?)
        .err()
        .ok_or("foreign target accepted")?;
    assert_eq!(label_error(error), ("InvalidElement", None));
    let expected = r#"[{"id":"21","font":[3,"CDXML","1","3"],"size_pt":[14.5,"n","11","14.5"],"face":[0,"t","21","0"]}]"#;
    let independent = independent_label_defaults(source, expected)?;
    assert!(
        independent.status.success(),
        "{}",
        String::from_utf8_lossy(&independent.stderr)
    );
    let reset = label_node(&document, "11")?;
    assert!(reset.tag_name().namespace().is_none_or(str::is_empty));
    assert!(
        reset
            .attributes()
            .any(|a| a.namespace() == Some("urn:other")
                && a.name() == "LabelFont"
                && a.value() == "bad")
    );
    assert_eq!(document.as_bytes(), source);
    Ok(())
}

fn label_quoted(value: &str) -> String {
    let mut output = String::from("\"");
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            c if c < ' ' => output.push_str(&format!("\\u{:04x}", u32::from(c))),
            c => output.push(c),
        }
    }
    output.push('"');
    output
}

fn label_payload(document: &Document<'_>, id: &str) -> LabelValueResult<String> {
    let defaults = document.label_defaults(label_node(document, id)?)?;
    macro_rules! item {
        ($value:expr) => {
            $value.map_or_else(
                || "null".to_owned(),
                |value| {
                    format!(
                        "[{},{},{},{}]",
                        value.value(),
                        label_quoted(value.owner().tag_name().name()),
                        label_quoted(value.owner().attribute("id").unwrap_or("")),
                        label_quoted(value.attribute_value())
                    )
                },
            )
        };
    }
    Ok(format!(
        "{{\"id\":{},\"font\":{},\"size_pt\":{},\"face\":{}}}",
        label_quoted(id),
        item!(defaults.font()),
        item!(defaults.size_pt()),
        item!(defaults.face())
    ))
}

fn independent_label_defaults(bytes: &[u8], expected: &str) -> LabelValueResult<Output> {
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
            include_str!("label_defaults_oracle.py"),
            expected,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child.stdin.take().ok_or("oracle stdin")?.write_all(bytes)?;
    Ok(child.wait_with_output()?)
}

#[test]
fn provenance_nodes_drive_exact_owned_edits_with_an_independent_expat_decimal_reader() -> TestResult
{
    const SOURCE: &str = "<?xml version='1.0'?>\r\n<CDXML xmlns:x='urn:other' id='1' LabelFont='+3.000' LabelSize='10.50' LabelFace='96' vendor='keep'><fonttable><font id='65535' name='No resolution implied' charset='unknown'/></fonttable><page id='2'><fragment id='3'><n id='11' LabelSize='&#x9;12.50&#xD;&#xA;' x:LabelFace='not a default'><t id='21' LabelFace='0'><s>O&amp;H</s></t></n></fragment></page><x:note keep='&#x41;'/></CDXML>";
    const EDITED: &str = "<?xml version='1.0'?>\r\n<CDXML xmlns:x='urn:other' id='1' LabelFont='-2' LabelSize='10.50' LabelFace='96' vendor='keep'><fonttable><font id='65535' name='No resolution implied' charset='unknown'/></fonttable><page id='2'><fragment id='3'><n id='11' LabelSize='18.25' x:LabelFace='not a default'><t id='21' LabelFace='0'><s>O&amp;H</s></t></n></fragment></page><x:note keep='&#x41;'/></CDXML>";
    const BEFORE: &str = r#"[{"id":"21","font":[3,"CDXML","1","+3.000"],"size_pt":[12.5,"n","11","\t12.50\r\n"],"face":[0,"t","21","0"]}]"#;
    const AFTER: &str = r#"[{"id":"21","font":[-2,"CDXML","1","-2"],"size_pt":[18.25,"n","11","18.25"],"face":[0,"t","21","0"]}]"#;
    // Qualify the independent checker on both original declarations before product output.
    for (source, expected) in [(SOURCE, BEFORE), (EDITED, AFTER)] {
        let checked = independent_label_defaults(source.as_bytes(), expected)?;
        assert!(
            checked.status.success(),
            "{}",
            String::from_utf8_lossy(&checked.stderr)
        );
    }
    for wrong in [
        BEFORE.replacen("[3,\"CDXML\",\"1\"", "[3,\"n\",\"11\"", 1),
        BEFORE.replacen("[12.5,\"n\",\"11\"", "[10.5,\"CDXML\",\"1\"", 1),
        BEFORE.replacen("[0,\"t\",\"21\"", "[96,\"CDXML\",\"1\"", 1),
    ] {
        assert_ne!(wrong, BEFORE);
        assert!(
            !independent_label_defaults(SOURCE.as_bytes(), &wrong)?
                .status
                .success()
        );
    }
    let invalid = SOURCE.replacen("<page id='2'>", "<page id='2' LabelFont='7'>", 1);
    let expected_error = r#"[{"id":"21","error":["UnsupportedOwner","LabelFont"]}]"#;
    assert!(
        independent_label_defaults(invalid.as_bytes(), expected_error)?
            .status
            .success()
    );
    assert!(
        !independent_label_defaults(invalid.as_bytes(), BEFORE)?
            .status
            .success()
    );

    let document = Document::parse(SOURCE.as_bytes())?;
    assert_eq!(format!("[{}]", label_payload(&document, "21")?), BEFORE);
    let defaults = document.label_defaults(label_node(&document, "21")?)?;
    let font = defaults.font().ok_or("font")?;
    let size = defaults.size_pt().ok_or("size")?;
    assert_eq!(size.attribute_value(), "\t12.50\r\n");
    assert_eq!(
        document.apply_edits(&[Edit::SetAttribute {
            element: size.owner(),
            name: "LabelSize",
            value: size.attribute_value()
        }])?,
        SOURCE.as_bytes(),
        "decoded no-op retains quotes, references and all bytes"
    );
    let output = document.apply_edits(&[
        Edit::SetAttribute {
            element: font.owner(),
            name: "LabelFont",
            value: "-2",
        },
        Edit::SetAttribute {
            element: size.owner(),
            name: "LabelSize",
            value: "18.25",
        },
    ])?;
    assert_eq!(output, EDITED.as_bytes());
    let checked = independent_label_defaults(&output, AFTER)?;
    assert!(
        checked.status.success(),
        "{}",
        String::from_utf8_lossy(&checked.stderr)
    );
    let reopened = Document::parse(&output)?;
    assert_eq!(format!("[{}]", label_payload(&reopened, "21")?), AFTER);
    let view = document.styled_text(label_node(&document, "21")?)?;
    let run = view.runs().first().ok_or("run")?;
    assert_eq!((run.font(), run.size_pt(), run.face()), (None, None, None));
    assert!(matches!(
        document.resources()?.font(3),
        Err(ResourceError::MissingFont(3))
    ));
    let other = Document::parse(SOURCE.as_bytes())?;
    assert!(matches!(
        other.apply_edits(&[Edit::SetAttribute {
            element: size.owner(),
            name: "LabelSize",
            value: "18.25"
        }]),
        Err(bondline_cdxml::Error::InvalidElement)
    ));
    assert_eq!(document.as_bytes(), SOURCE.as_bytes());
    Ok(())
}
