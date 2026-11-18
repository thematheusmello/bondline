//! Independent SetText declaration. Root owns API RED and all execution.
use bondline_cdxml::{Document, Edit};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn plain_text_changes_only_the_selected_leaf_body() -> TestResult {
    let source = br#"<CDXML><!--keep--><?keep data?><t p='12 34'><s face="96">OH</s><s>stay</s></t></CDXML>"#;
    let expected = br#"<CDXML><!--keep--><?keep data?><t p='12 34'><s face="96">NH2</s><s>stay</s></t></CDXML>"#;
    let document = Document::parse(source)?;
    let s = document
        .xml()
        .descendants()
        .find(|n| n.has_tag_name("s"))
        .ok_or("s")?;
    let output = document.apply_edits(&[Edit::SetText {
        element: s,
        value: "NH2",
    }])?;
    assert_eq!(output, expected);
    assert_eq!(document.as_bytes(), source);
    Ok(())
}

// Bounded expansion, declared after the first API RED and before SetText production.
use bondline_cdxml::{Error, MAX_INPUT_BYTES, MAX_NODES};
use std::{
    io::Write,
    process::{Command, Output, Stdio},
};

// Independent XML 1.0 reader: concatenate character-data callbacks only within
// each direct parent. CDATA/reference segmentation is not a text-value oracle.
fn expat_check(bytes: &[u8], expected: &str) -> TestResult<Output> {
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
            r#"
import json, sys, xml.parsers.expat as expat
rows, stack = [], []
p = expat.ParserCreate(namespace_separator='|')
p.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
def start(name, attrs):
    rows.append([name, attrs, ''])
    stack.append(len(rows)-1)
def chars(value):
    if stack:
        rows[stack[-1]][2] += value
def end(name):
    stack.pop()
p.StartElementHandler = start
p.CharacterDataHandler = chars
p.EndElementHandler = end
p.Parse(sys.stdin.buffer.read(), True)
assert rows == json.loads(sys.argv[1]), repr(rows)
"#,
            expected,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child.stdin.take().ok_or("Expat stdin")?.write_all(bytes)?;
    Ok(child.wait_with_output()?)
}

fn expat_rows(bytes: &[u8], expected: &str) -> TestResult {
    let result = expat_check(bytes, expected)?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(())
}

#[test]
fn literal_expat_controls_and_changed_text_preserve_decoded_characters_and_trivia() -> TestResult {
    // Qualify this external checker against original XML before giving it output.
    let control = b"<CDXML><s a='x'>A&amp;B</s></CDXML>";
    expat_rows(control, r#"[["CDXML",{},""],["s",{"a":"x"},"A&B"]]"#)?;
    assert!(
        !expat_check(control, r#"[["CDXML",{},""],["s",{"a":"x"},"A&amp;B"]]"#)?
            .status
            .success()
    );
    assert!(
        !expat_check(control, r#"[["CDXML",{},""],["s",{"a":"wrong"},"A&B"]]"#)?
            .status
            .success()
    );

    let source = br#"<?xml version='1.0'?><CDXML><!--keep--><?keep data?><t><s face='96'>A<![CDATA[B&</s>]]>C&#x44;</s><s>stay</s></t></CDXML>"#;
    let value = "μ 😀 & < > \" ' \t\r\n literal &amp; ]]>";
    let expected = "<?xml version='1.0'?><CDXML><!--keep--><?keep data?><t><s face='96'>μ 😀 &amp; &lt; &gt; &quot; &apos; &#x9;&#xD;&#xA; literal &amp;amp; ]]&gt;</s><s>stay</s></t></CDXML>";
    let before = r#"[["CDXML",{},""],["t",{},""],["s",{"face":"96"},"AB&</s>CD"],["s",{},"stay"]]"#;
    let after = r#"[["CDXML",{},""],["t",{},""],["s",{"face":"96"},"μ 😀 & < > \" ' \t\r\n literal &amp; ]]>"],["s",{},"stay"]]"#;
    expat_rows(source, before)?;
    expat_rows(expected.as_bytes(), after)?;
    let document = Document::parse(source)?;
    let s = document
        .xml()
        .descendants()
        .find(|n| n.has_tag_name("s"))
        .ok_or("s")?;
    let output = document.apply_edits(&[Edit::SetText { element: s, value }])?;
    assert_eq!(output, expected.as_bytes());
    assert_eq!(document.as_bytes(), source);
    expat_rows(&output, after)
}

#[test]
fn semantic_noops_keep_entities_cdata_line_endings_empty_spelling_and_unicode_exact() -> TestResult
{
    let cases = [
        (
            "<CDXML><s>A<![CDATA[B&]]>C&#x44;</s></CDXML>",
            "AB&CD",
            r#"[["CDXML",{},""],["s",{},"AB&CD"]]"#,
        ),
        (
            "<CDXML><s>x\r\ny</s></CDXML>",
            "x\ny",
            r#"[["CDXML",{},""],["s",{},"x\ny"]]"#,
        ),
        (
            "<CDXML><s>x&#xD;y</s></CDXML>",
            "x\ry",
            r#"[["CDXML",{},""],["s",{},"x\ry"]]"#,
        ),
        (
            "<CDXML><s>&amp;amp;</s></CDXML>",
            "&amp;",
            r#"[["CDXML",{},""],["s",{},"&amp;"]]"#,
        ),
        (
            "<CDXML><s>&#xE9;</s></CDXML>",
            "é",
            r#"[["CDXML",{},""],["s",{},"é"]]"#,
        ),
        (
            "<CDXML><s>e&#x301;</s></CDXML>",
            "e\u{301}",
            r#"[["CDXML",{},""],["s",{},"e\u0301"]]"#,
        ),
        (
            "<CDXML><s/></CDXML>",
            "",
            r#"[["CDXML",{},""],["s",{},""]]"#,
        ),
        (
            "<CDXML><s ></s></CDXML>",
            "",
            r#"[["CDXML",{},""],["s",{},""]]"#,
        ),
        (
            "<CDXML><s><![CDATA[]]></s></CDXML>",
            "",
            r#"[["CDXML",{},""],["s",{},""]]"#,
        ),
    ];
    for (source, value, rows) in cases {
        expat_rows(source.as_bytes(), rows)?;
        let document = Document::parse(source.as_bytes())?;
        let s = document
            .xml()
            .root_element()
            .first_element_child()
            .ok_or("s")?;
        assert_eq!(
            document.apply_edits(&[Edit::SetText { element: s, value }])?,
            source.as_bytes()
        );
        assert_eq!(document.as_bytes(), source.as_bytes());
    }
    // XML equality is not Unicode normalization, or literal CRLF normalization on write.
    for (source, value, expected, rows) in [
        (
            "<CDXML><s>e&#x301;</s></CDXML>",
            "é",
            "<CDXML><s>é</s></CDXML>",
            r#"[["CDXML",{},""],["s",{},"é"]]"#,
        ),
        (
            "<CDXML><s>x\r\ny</s></CDXML>",
            "x\r\ny",
            "<CDXML><s>x&#xD;&#xA;y</s></CDXML>",
            r#"[["CDXML",{},""],["s",{},"x\r\ny"]]"#,
        ),
        (
            "<CDXML><s a='v' >OH</s></CDXML>",
            "",
            "<CDXML><s a='v' ></s></CDXML>",
            r#"[["CDXML",{},""],["s",{"a":"v"},""]]"#,
        ),
    ] {
        expat_rows(expected.as_bytes(), rows)?;
        let document = Document::parse(source.as_bytes())?;
        let s = document
            .xml()
            .root_element()
            .first_element_child()
            .ok_or("s")?;
        let output = document.apply_edits(&[Edit::SetText { element: s, value }])?;
        assert_eq!(output, expected.as_bytes());
        assert_eq!(document.as_bytes(), source.as_bytes());
        expat_rows(&output, rows)?;
    }
    Ok(())
}

#[test]
fn qualified_selfclosing_text_and_attributes_compose_without_rebuilding_the_tag() -> TestResult {
    let source = "<CDXML xmlns:f='urn:f'><f:tēxt id='7' h='&#51;' note=\"a > b\" /></CDXML>";
    let expected = "<CDXML xmlns:f='urn:f'><f:tēxt id='8' h='&#51;' note=\"a > b\"  fresh=\"μ &amp;\">&lt;OH&gt;</f:tēxt></CDXML>";
    let before = r#"[["CDXML",{},""],["urn:f|tēxt",{"id":"7","h":"3","note":"a > b"},""]]"#;
    let after = r#"[["CDXML",{},""],["urn:f|tēxt",{"id":"8","h":"3","note":"a > b","fresh":"μ &"},"<OH>"]]"#;
    expat_rows(source.as_bytes(), before)?;
    expat_rows(expected.as_bytes(), after)?;
    let document = Document::parse(source.as_bytes())?;
    let s = document
        .xml()
        .root_element()
        .first_element_child()
        .ok_or("qualified text")?;
    let text = Edit::SetText {
        element: s,
        value: "<OH>",
    };
    let id = Edit::SetAttribute {
        element: s,
        name: "id",
        value: "8",
    };
    let h = Edit::SetAttribute {
        element: s,
        name: "h",
        value: "3",
    };
    let fresh = Edit::SetAttribute {
        element: s,
        name: "fresh",
        value: "μ &",
    };
    for edits in [[text, id, h, fresh], [id, h, fresh, text]] {
        let output = document.apply_edits(&edits)?;
        assert_eq!(output, expected.as_bytes());
        expat_rows(&output, after)?;
        assert_eq!(document.as_bytes(), source.as_bytes());
    }
    Ok(())
}

#[test]
fn descendant_text_expansion_precedes_ancestor_append_at_the_same_source_offset() -> TestResult {
    let source = b"<CDXML><t><s/></t></CDXML>";
    let expected = b"<CDXML><t><s>OH</s><s>2</s></t></CDXML>";
    let after = r#"[["CDXML",{},""],["t",{},""],["s",{},"OH"],["s",{},"2"]]"#;
    expat_rows(source, r#"[["CDXML",{},""],["t",{},""],["s",{},""]]"#)?;
    expat_rows(expected, after)?;
    let document = Document::parse(source)?;
    let t = document
        .xml()
        .root_element()
        .first_element_child()
        .ok_or("t")?;
    let s = t.first_element_child().ok_or("s")?;
    let text = Edit::SetText {
        element: s,
        value: "OH",
    };
    let append = Edit::AppendChildren {
        element: t,
        xml: "<s>2</s>",
    };
    for edits in [[text, append], [append, text]] {
        let output = document.apply_edits(&edits)?;
        assert_eq!(output, expected);
        expat_rows(&output, after)?;
        assert_eq!(document.as_bytes(), source);
    }
    Ok(())
}

#[test]
fn any_nontext_direct_child_refuses_before_an_equal_or_empty_noop() -> TestResult {
    for (body, value) in [
        ("<!--keep-->", ""),
        ("<?keep data?>", ""),
        ("<b/>", ""),
        ("OH<!--keep-->", "OH"),
        ("<?keep data?>OH", "OH"),
        ("OH<b/>", "OH"),
        ("OH<b>NH2</b>H", "OHNH2H"),
    ] {
        let source = format!("<CDXML><s>{body}</s></CDXML>");
        let document = Document::parse(source.as_bytes())?;
        let s = document
            .xml()
            .root_element()
            .first_element_child()
            .ok_or("s")?;
        assert!(
            matches!(
                document.apply_edits(&[Edit::SetText { element: s, value }]),
                Err(Error::InvalidTextContent)
            ),
            "{body}"
        );
        assert_eq!(document.as_bytes(), source.as_bytes());
    }
    Ok(())
}

#[test]
fn text_owns_the_same_container_conflict_key_as_append_even_for_noops() -> TestResult {
    let source = b"<CDXML><s id='1'>OH</s><s id='2'>H</s></CDXML>";
    let document = Document::parse(source)?;
    let root = document.xml().root_element();
    let first = root.first_element_child().ok_or("first s")?;
    let second = first.next_sibling_element().ok_or("second s")?;
    let noop = Edit::SetText {
        element: first,
        value: "OH",
    };
    let changed = Edit::SetText {
        element: first,
        value: "NH2",
    };
    for edits in [
        [noop, noop],
        [noop, changed],
        [changed, noop],
        [
            noop,
            Edit::AppendChildren {
                element: first,
                xml: "",
            },
        ],
        [
            Edit::AppendChildren {
                element: first,
                xml: "",
            },
            noop,
        ],
        [
            changed,
            Edit::AppendChildren {
                element: first,
                xml: "2",
            },
        ],
        [
            Edit::AppendChildren {
                element: first,
                xml: "2",
            },
            changed,
        ],
    ] {
        assert!(matches!(
            document.apply_edits(&edits),
            Err(Error::ConflictingEdits)
        ));
        assert_eq!(document.as_bytes(), source);
    }
    // Distinct leaves and attribute removals are independent targets.
    let output = document.apply_edits(&[
        changed,
        Edit::SetText {
            element: second,
            value: "2",
        },
        Edit::RemoveAttribute {
            element: second,
            name: "id",
        },
    ])?;
    let expected = b"<CDXML><s id='1'>NH2</s><s >2</s></CDXML>";
    let rows = r#"[["CDXML",{},""],["s",{"id":"1"},"NH2"],["s",{},"2"]]"#;
    expat_rows(expected, rows)?;
    assert_eq!(output, expected);
    assert_eq!(document.as_bytes(), source);
    expat_rows(&output, rows)
}

#[test]
fn ownership_illegal_xml_characters_and_late_errors_never_publish_partial_edits() -> TestResult {
    let source = b"<CDXML p='old'><!--keep--><s>OH</s></CDXML>";
    let document = Document::parse(source)?;
    let foreign = Document::parse(source)?;
    let root = document.xml().root_element();
    let s = root.first_element_child().ok_or("s")?;
    let comment = root.children().find(|n| n.is_comment()).ok_or("comment")?;
    let text = s.first_child().ok_or("text")?;
    for target in [
        document.xml().root(),
        comment,
        text,
        foreign.xml().root_element(),
    ] {
        assert!(matches!(
            document.apply_edits(&[Edit::SetText {
                element: target,
                value: "NH2"
            }]),
            Err(Error::InvalidElement)
        ));
        assert_eq!(document.as_bytes(), source);
        assert_eq!(foreign.as_bytes(), source);
    }
    for value in ["\0", "a\u{1f}b", "\u{fffe}", "\u{ffff}"] {
        assert!(matches!(
            document.apply_edits(&[
                Edit::SetAttribute {
                    element: root,
                    name: "p",
                    value: "changed"
                },
                Edit::SetText { element: s, value },
            ]),
            Err(Error::InvalidTextValue)
        ));
        // Only the new text operation changes error classification.
        assert!(matches!(
            document.apply_edits(&[Edit::SetAttribute {
                element: root,
                name: "p",
                value
            }]),
            Err(Error::InvalidAttributeValue)
        ));
        assert_eq!(document.as_bytes(), source);
    }
    // The same receiver remains usable after every failed batch.
    let output = document.apply_edits(&[Edit::SetText {
        element: s,
        value: "NH2",
    }])?;
    assert_eq!(output, b"<CDXML p='old'><!--keep--><s>NH2</s></CDXML>");
    assert_eq!(document.as_bytes(), source);
    Ok(())
}

#[test]
fn full_inner_range_survives_long_quoted_tags_and_cdata_containing_closing_text() -> TestResult {
    let padding = "x".repeat(128 * 1024);
    let source =
        format!("<CDXML><s pad='{padding}' note='a > b'>A<![CDATA[B</s>C]]>&#x44;</s></CDXML>");
    let expected = format!("<CDXML><s pad='{padding}' note='a > b'>NH2</s></CDXML>");
    let document = Document::parse(source.as_bytes())?;
    let s = document
        .xml()
        .root_element()
        .first_element_child()
        .ok_or("s")?;
    assert_eq!(
        document.apply_edits(&[Edit::SetText {
            element: s,
            value: "AB</s>CD"
        }])?,
        source.as_bytes()
    );
    let output = document.apply_edits(&[Edit::SetText {
        element: s,
        value: "NH2",
    }])?;
    assert_eq!(output, expected.as_bytes());
    assert_eq!(document.as_bytes(), source.as_bytes());
    Ok(())
}

#[test]
fn escaped_growth_and_new_text_nodes_keep_the_existing_resource_failures() -> TestResult {
    let source = b"<CDXML><s/></CDXML>";
    let document = Document::parse(source)?;
    let s = document
        .xml()
        .root_element()
        .first_element_child()
        .ok_or("s")?;
    // Five escaped bytes per '&': exceed the insertion bound before allocation.
    let value = "&".repeat(MAX_INPUT_BYTES / 5 + 1);
    assert!(value.len() < MAX_INPUT_BYTES);
    assert!(value.len() * 5 > MAX_INPUT_BYTES);
    assert!(matches!(
        document.apply_edits(&[Edit::SetText {
            element: s,
            value: &value
        }]),
        Err(Error::InputTooLarge)
    ));
    assert_eq!(document.as_bytes(), source);
    drop(value);

    // Document + CDXML + s + these n elements fills the pinned syntax-node cap.
    // The independently authored final XML adds exactly one text node.
    let siblings = "<n/>".repeat(MAX_NODES as usize - 3);
    let source = format!("<CDXML><s/>{siblings}</CDXML>");
    let final_xml = format!("<CDXML><s>x</s>{siblings}</CDXML>");
    let document = Document::parse(source.as_bytes())?;
    let s = document
        .xml()
        .root_element()
        .first_element_child()
        .ok_or("s")?;
    let expected = Document::parse(final_xml.as_bytes())
        .err()
        .ok_or("literal final XML must exceed the existing node cap")?;
    assert!(
        matches!(expected, Error::Xml(_)),
        "existing parser classification"
    );
    let actual = document
        .apply_edits(&[Edit::SetText {
            element: s,
            value: "x",
        }])
        .err()
        .ok_or("new text must obey the same node cap")?;
    assert_eq!(
        std::mem::discriminant(&actual),
        std::mem::discriminant(&expected)
    );
    assert_eq!(actual.to_string(), expected.to_string());
    assert_eq!(document.as_bytes(), source.as_bytes());
    Ok(())
}
