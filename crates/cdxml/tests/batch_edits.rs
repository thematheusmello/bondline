//! Independent first source-preserving batch declaration; Root owns API RED execution.
use bondline_cdxml::{Document, Edit};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn one_batch_set_keeps_the_original_quote_and_every_unrelated_byte() -> TestResult {
    let source = br#"<CDXML p='1' untouched="keep"/>"#;
    let document = Document::parse(source)?;
    let output = document.apply_edits(&[Edit::SetAttribute {
        element: document.xml().root_element(),
        name: "p",
        value: "2",
    }])?;
    assert_eq!(output, br#"<CDXML p='2' untouched="keep"/>"#);
    assert_eq!(document.as_bytes(), source);
    Ok(())
}

// First bounded expansion, independently declared before removal/append production.
use bondline_cdxml::Error;
use std::{
    io::Write,
    process::{Command, Stdio},
};
fn expat_events(bytes: &[u8], expected: &str) -> TestResult {
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
import json,sys,xml.parsers.expat as expat
found=[]
p=expat.ParserCreate(namespace_separator='|')
p.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
p.StartElementHandler=lambda name,attrs:found.append(['start',name,attrs])
p.EndElementHandler=lambda name:found.append(['end',name])
p.CommentHandler=lambda text:found.append(['comment',text])
p.ProcessingInstructionHandler=lambda name,data:found.append(['pi',name,data])
p.Parse(sys.stdin.buffer.read(),True)
assert found==json.loads(sys.argv[1]),repr(found)
"#,
            expected,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child.stdin.take().ok_or("Expat stdin")?.write_all(bytes)?;
    let result = child.wait_with_output()?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(())
}

#[test]
fn mixed_quote_batch_retains_entities_numeric_lexemes_opaque_content_and_qualified_attributes()
-> TestResult {
    let source = "\u{feff}<?xml version='1.0' encoding='UTF-8'?>\r\n<CDXML xmlns:f='urn:f' f:note=\"old\"><!--keep--><?probe keep?><n id='7' p='12.50 25.00' h='&#51;' f:drop = \"x\" /><f:opaque a='unchanged'/></CDXML>";
    let doc = Document::parse(source.as_bytes())?;
    let root = doc.xml().root_element();
    let n = doc
        .xml()
        .descendants()
        .find(|n| n.has_tag_name("n"))
        .ok_or("n")?;
    let noops = [
        Edit::SetAttribute {
            element: n,
            name: "h",
            value: "3",
        },
        Edit::SetAttribute {
            element: n,
            name: "p",
            value: "12.50 25.00",
        },
        Edit::RemoveAttribute {
            element: n,
            name: "absent",
        },
    ];
    assert_eq!(doc.apply_edits(&noops)?, source.as_bytes());
    let value = "μ & < > \" ' \t\r\n";
    let output = doc.apply_edits(&[
        Edit::SetAttribute {
            element: root,
            name: "f:note",
            value,
        },
        Edit::SetAttribute {
            element: n,
            name: "h",
            value: "3",
        },
        Edit::SetAttribute {
            element: n,
            name: "p",
            value: "12.50 25.00",
        },
        Edit::RemoveAttribute {
            element: n,
            name: "f:drop",
        },
        Edit::RemoveAttribute {
            element: n,
            name: "absent",
        },
    ])?;
    let expected = source
        .replacen(
            "f:note=\"old\"",
            "f:note=\"μ &amp; &lt; &gt; &quot; &apos; &#x9;&#xD;&#xA;\"",
            1,
        )
        .replacen("f:drop = \"x\"", "", 1);
    assert_eq!(output, expected.as_bytes());
    assert_eq!(doc.as_bytes(), source.as_bytes());
    expat_events(
        &output,
        r#"[["start","CDXML",{"urn:f|note":"μ & < > \" ' \t\r\n"}],["comment","keep"],["pi","probe","keep"],["start","n",{"id":"7","p":"12.50 25.00","h":"3"}],["end","n"],["start","urn:f|opaque",{"a":"unchanged"}],["end","urn:f|opaque"],["end","CDXML"]]"#,
    )
}

#[test]
fn new_attributes_keep_caller_order_and_compose_with_same_point_selfclosing_expansion() -> TestResult
{
    let source = b"<CDXML><page id='p' /></CDXML>";
    let doc = Document::parse(source)?;
    let root = doc.xml().root_element();
    let page = root
        .children()
        .find(|n| n.has_tag_name("page"))
        .ok_or("page")?;
    let output = doc.apply_edits(&[
        Edit::SetAttribute {
            element: page,
            name: "beta",
            value: "& \"",
        },
        Edit::SetAttribute {
            element: root,
            name: "top",
            value: "x",
        },
        Edit::AppendChildren {
            element: page,
            xml: "<n id='n'/><!--child--><?pi d?>",
        },
        Edit::SetAttribute {
            element: page,
            name: "alpha",
            value: "two",
        },
    ])?;
    assert_eq!(output,br#"<CDXML top="x"><page id='p'  beta="&amp; &quot;" alpha="two"><n id='n'/><!--child--><?pi d?></page></CDXML>"#);
    assert_eq!(doc.as_bytes(), source);
    expat_events(
        &output,
        r#"[["start","CDXML",{"top":"x"}],["start","page",{"id":"p","beta":"& \"","alpha":"two"}],["start","n",{"id":"n"}],["end","n"],["comment","child"],["pi","pi","d"],["end","page"],["end","CDXML"]]"#,
    )
}

#[test]
fn original_parent_and_descendant_edits_do_not_conflict_or_reuse_shifted_offsets() -> TestResult {
    let source = br#"<CDXML><page role='old'><n p="1 2"/><!--x--></page><other/></CDXML>"#;
    let doc = Document::parse(source)?;
    let root = doc.xml().root_element();
    let page = doc
        .xml()
        .descendants()
        .find(|n| n.has_tag_name("page"))
        .ok_or("page")?;
    let n = doc
        .xml()
        .descendants()
        .find(|n| n.has_tag_name("n"))
        .ok_or("n")?;
    let output = doc.apply_edits(&[
        Edit::AppendChildren {
            element: root,
            xml: "<tail/>",
        },
        Edit::AppendChildren {
            element: page,
            xml: "<n id='2'/>",
        },
        Edit::SetAttribute {
            element: n,
            name: "flag",
            value: "v",
        },
        Edit::SetAttribute {
            element: n,
            name: "p",
            value: "123.5 456.00",
        },
        Edit::AppendChildren {
            element: n,
            xml: "<inside/>",
        },
        Edit::SetAttribute {
            element: page,
            name: "role",
            value: "new",
        },
    ])?;
    assert_eq!(output,br#"<CDXML><page role='new'><n p="123.5 456.00" flag="v"><inside/></n><!--x--><n id='2'/></page><other/><tail/></CDXML>"#);
    assert_eq!(doc.as_bytes(), source);
    expat_events(
        &output,
        r#"[["start","CDXML",{}],["start","page",{"role":"new"}],["start","n",{"p":"123.5 456.00","flag":"v"}],["start","inside",{}],["end","inside"],["end","n"],["comment","x"],["start","n",{"id":"2"}],["end","n"],["end","page"],["start","other",{}],["end","other"],["start","tail",{}],["end","tail"],["end","CDXML"]]"#,
    )
}

#[test]
fn duplicate_qnames_and_appends_conflict_even_when_entries_are_equal_or_noops() -> TestResult {
    let source = b"<CDXML x='same'/>";
    let doc = Document::parse(source)?;
    let root = doc.xml().root_element();
    for edits in [
        vec![
            Edit::SetAttribute {
                element: root,
                name: "x",
                value: "same",
            },
            Edit::SetAttribute {
                element: root,
                name: "x",
                value: "same",
            },
        ],
        vec![
            Edit::RemoveAttribute {
                element: root,
                name: "absent",
            },
            Edit::RemoveAttribute {
                element: root,
                name: "absent",
            },
        ],
        vec![
            Edit::SetAttribute {
                element: root,
                name: "new",
                value: "v",
            },
            Edit::RemoveAttribute {
                element: root,
                name: "new",
            },
        ],
        vec![
            Edit::SetAttribute {
                element: root,
                name: "x",
                value: "different",
            },
            Edit::RemoveAttribute {
                element: root,
                name: "x",
            },
        ],
        vec![
            Edit::AppendChildren {
                element: root,
                xml: "",
            },
            Edit::AppendChildren {
                element: root,
                xml: "",
            },
        ],
        vec![
            Edit::AppendChildren {
                element: root,
                xml: "<n/>",
            },
            Edit::AppendChildren {
                element: root,
                xml: "",
            },
        ],
    ] {
        assert!(matches!(
            doc.apply_edits(&edits),
            Err(Error::ConflictingEdits)
        ));
        assert_eq!(doc.as_bytes(), source);
    }
    Ok(())
}

#[test]
fn ownership_names_and_bad_values_fail_atomically_after_other_valid_edits() -> TestResult {
    let source = b"<CDXML xmlns:f='urn:f' x='old'/>";
    let doc = Document::parse(source)?;
    let other = Document::parse(source)?;
    let root = doc.xml().root_element();
    let foreign = other.xml().root_element();
    for bad in [
        Edit::SetAttribute {
            element: foreign,
            name: "x",
            value: "v",
        },
        Edit::RemoveAttribute {
            element: foreign,
            name: "absent",
        },
        Edit::AppendChildren {
            element: foreign,
            xml: "",
        },
        Edit::SetAttribute {
            element: doc.xml().root(),
            name: "x",
            value: "v",
        },
        Edit::RemoveAttribute {
            element: doc.xml().root(),
            name: "absent",
        },
        Edit::AppendChildren {
            element: doc.xml().root(),
            xml: "",
        },
    ] {
        assert!(matches!(
            doc.apply_edits(&[
                Edit::SetAttribute {
                    element: root,
                    name: "x",
                    value: "valid"
                },
                bad
            ]),
            Err(Error::InvalidElement)
        ));
        assert_eq!(doc.as_bytes(), source);
    }
    for name in ["", "9bad", "space name", "f:new", "µ", "xmlns", "xmlns:f"] {
        assert!(
            matches!(
                doc.apply_edits(&[Edit::SetAttribute {
                    element: root,
                    name,
                    value: "v"
                }]),
                Err(Error::InvalidName)
            ),
            "{name}"
        );
    }
    for name in ["xmlns", "xmlns:f"] {
        assert!(matches!(
            doc.apply_edits(&[Edit::RemoveAttribute {
                element: root,
                name
            }]),
            Err(Error::InvalidName)
        ));
    }
    for value in ["\0", "\u{1f}", "\u{fffe}"] {
        assert!(matches!(
            doc.apply_edits(&[
                Edit::SetAttribute {
                    element: root,
                    name: "x",
                    value: "valid"
                },
                Edit::SetAttribute {
                    element: root,
                    name: "different",
                    value
                }
            ]),
            Err(Error::InvalidAttributeValue)
        ));
    }
    assert_eq!(doc.as_bytes(), source);
    Ok(())
}

#[test]
fn append_fragments_are_selfcontained_qualified_parents_expand_and_empty_stays_exact() -> TestResult
{
    let source = b"<CDXML xmlns:f='urn:outer'><f:box a='old'/></CDXML>";
    let doc = Document::parse(source)?;
    let root = doc.xml().root_element();
    let parent = root
        .children()
        .find(|n| n.is_element())
        .ok_or("qualified parent")?;
    assert_eq!(
        doc.apply_edits(&[Edit::AppendChildren {
            element: parent,
            xml: ""
        }])?,
        source
    );
    assert_eq!(doc.apply_edits(&[])?, source);
    let output = doc.apply_edits(&[
        Edit::SetAttribute {
            element: parent,
            name: "a",
            value: "new",
        },
        Edit::AppendChildren {
            element: parent,
            xml: "<g:item xmlns:g='urn:inner'/><![CDATA[a<b]]>",
        },
    ])?;
    assert_eq!(output,b"<CDXML xmlns:f='urn:outer'><f:box a='new'><g:item xmlns:g='urn:inner'/><![CDATA[a<b]]></f:box></CDXML>");
    expat_events(
        &output,
        r#"[["start","CDXML",{}],["start","urn:outer|box",{"a":"new"}],["start","urn:inner|item",{}],["end","urn:inner|item"],["end","urn:outer|box"],["end","CDXML"]]"#,
    )?;
    for xml in [
        "<n>",
        "</f:box><outside/>",
        "<?xml version='1.0'?><n/>",
        "<!DOCTYPE n><n/>",
        "<!DOCTYPE n [<!ENTITY x 'y'>]><n/>",
        "<f:n/>",
        "<n>&unknown;</n>",
    ] {
        assert!(
            matches!(
                doc.apply_edits(&[
                    Edit::SetAttribute {
                        element: parent,
                        name: "a",
                        value: "valid"
                    },
                    Edit::AppendChildren {
                        element: parent,
                        xml
                    }
                ]),
                Err(Error::InvalidFragment)
            ),
            "{xml}"
        );
        assert_eq!(doc.as_bytes(), source);
    }
    Ok(())
}

#[test]
fn ancestor_append_cannot_enter_an_adjacent_selfclosing_descendant() -> TestResult {
    let source = b"<CDXML><page/></CDXML>";
    let document = Document::parse(source)?;
    let root = document.xml().root_element();
    let page = root.first_element_child().ok_or("page")?;
    for reverse in [false, true] {
        let mut edits = [
            Edit::AppendChildren {
                element: root,
                xml: "<outer/>",
            },
            Edit::AppendChildren {
                element: page,
                xml: "<inner/>",
            },
        ];
        if reverse {
            edits.reverse();
        }
        let output = document.apply_edits(&edits)?;
        assert_eq!(output, b"<CDXML><page><inner/></page><outer/></CDXML>");
        expat_events(
            &output,
            r#"[["start","CDXML",{}],["start","page",{}],["start","inner",{}],["end","inner"],["end","page"],["start","outer",{}],["end","outer"],["end","CDXML"]]"#,
        )?;
        assert_eq!(document.as_bytes(), source);
    }
    Ok(())
}
