//! Independent resource supplement. Root owns API/runtime receiving; no parser policy changes.
use bondline_cdxml::{
    Document, Edit, Error, MAX_ATTRIBUTES, MAX_ATTRIBUTES_PER_ELEMENT, MAX_DEPTH, MAX_EDITS,
    MAX_INPUT_BYTES, MAX_NODES,
};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

// Expected final XML is independently assembled, never produced by the batch editor.
// This checks exact propagation of existing parser errors, including Xml node-limit errors.
fn same_final_resource_error(actual: Result<Vec<u8>, Error>, final_xml: &str) -> TestResult {
    let expected = Document::parse(final_xml.as_bytes())
        .err()
        .ok_or("literal final XML exceeds an existing resource bound")?;
    let actual = actual
        .err()
        .ok_or("batch must refuse the same final resource excess")?;
    assert_eq!(
        std::mem::discriminant(&actual),
        std::mem::discriminant(&expected)
    );
    assert_eq!(actual.to_string(), expected.to_string());
    Ok(())
}

#[test]
fn exact_edit_cap_accepts_distinct_noops_and_excess_precedes_conflicts() -> TestResult {
    assert_eq!(MAX_EDITS, MAX_ATTRIBUTES + MAX_NODES as usize);
    assert_eq!(MAX_EDITS, 200_000);
    let source = b"<CDXML/>";
    let doc = Document::parse(source)?;
    let root = doc.xml().root_element();
    let names: Vec<_> = (0..MAX_EDITS).map(|i| format!("absent{i}")).collect();
    let at_limit: Vec<_> = names
        .iter()
        .map(|name| Edit::RemoveAttribute {
            element: root,
            name: name.as_str(),
        })
        .collect();
    assert_eq!(doc.apply_edits(&at_limit)?, source);
    let excess: Vec<_> = (0..=MAX_EDITS)
        .map(|_| Edit::RemoveAttribute {
            element: root,
            name: "same_absent_name",
        })
        .collect();
    assert!(
        matches!(doc.apply_edits(&excess), Err(Error::EditLimit)),
        "count wins over the intentional duplicate conflicts"
    );
    assert_eq!(doc.as_bytes(), source);
    Ok(())
}

#[test]
fn attribute_limits_apply_to_the_final_document_not_an_intermediate_plan() -> TestResult {
    let attributes = (0..MAX_ATTRIBUTES_PER_ELEMENT)
        .map(|i| format!(" a{i}='x'"))
        .collect::<String>();
    let source = format!("<CDXML{attributes}/>");
    let doc = Document::parse(source.as_bytes())?;
    let root = doc.xml().root_element();
    let too_many = format!("<CDXML{attributes} extra=\"v\"/>");
    assert!(matches!(
        Document::parse(too_many.as_bytes()),
        Err(Error::AttributeLimit)
    ));
    same_final_resource_error(
        doc.apply_edits(&[Edit::SetAttribute {
            element: root,
            name: "extra",
            value: "v",
        }]),
        &too_many,
    )?;
    let replacement = doc.apply_edits(&[
        Edit::SetAttribute {
            element: root,
            name: "extra",
            value: "v",
        },
        Edit::RemoveAttribute {
            element: root,
            name: "a0",
        },
    ])?;
    let expected = format!(
        "<CDXML{} extra=\"v\"/>",
        attributes.replacen("a0='x'", "", 1)
    );
    assert_eq!(replacement, expected.as_bytes());
    assert_eq!(doc.as_bytes(), source.as_bytes());

    // 500 distinct elements times 200 attributes, not an overfull single element.
    let per_child = 200;
    assert_eq!(MAX_ATTRIBUTES % per_child, 0);
    let child_attributes = (0..per_child)
        .map(|i| format!(" a{i}='x'"))
        .collect::<String>();
    let child = format!("<n{child_attributes}/>");
    let source = format!(
        "<CDXML>{}</CDXML>",
        child.repeat(MAX_ATTRIBUTES / per_child)
    );
    let doc = Document::parse(source.as_bytes())?;
    let final_xml = source.replacen("<CDXML>", "<CDXML extra=\"v\">", 1);
    assert!(matches!(
        Document::parse(final_xml.as_bytes()),
        Err(Error::AttributeLimit)
    ));
    same_final_resource_error(
        doc.apply_edits(&[Edit::SetAttribute {
            element: doc.xml().root_element(),
            name: "extra",
            value: "v",
        }]),
        &final_xml,
    )?;
    assert_eq!(doc.as_bytes(), source.as_bytes());
    Ok(())
}

#[test]
fn appended_depth_and_node_limits_keep_the_existing_direct_parse_error() -> TestResult {
    let source = format!(
        "<CDXML>{}{}</CDXML>",
        "<a>".repeat(MAX_DEPTH - 1),
        "</a>".repeat(MAX_DEPTH - 1)
    );
    let doc = Document::parse(source.as_bytes())?;
    let mut deepest = doc.xml().root_element();
    for _ in 1..MAX_DEPTH {
        deepest = deepest
            .first_element_child()
            .ok_or("literal nested element")?;
    }
    let final_xml = source.replacen("</a>", "<z/></a>", 1);
    assert!(matches!(
        Document::parse(final_xml.as_bytes()),
        Err(Error::DepthLimit)
    ));
    same_final_resource_error(
        doc.apply_edits(&[Edit::AppendChildren {
            element: deepest,
            xml: "<z/>",
        }]),
        &final_xml,
    )?;
    assert_eq!(doc.as_bytes(), source.as_bytes());

    let source = b"<CDXML/>";
    let doc = Document::parse(source)?;
    let children = "<n/>".repeat(MAX_NODES as usize);
    let final_xml = format!("<CDXML>{children}</CDXML>");
    assert!(
        matches!(Document::parse(final_xml.as_bytes()), Err(Error::Xml(_))),
        "the installed parser's node bound is not Writer::NodeLimit"
    );
    same_final_resource_error(
        doc.apply_edits(&[Edit::AppendChildren {
            element: doc.xml().root_element(),
            xml: &children,
        }]),
        &final_xml,
    )?;
    assert_eq!(doc.as_bytes(), source);
    Ok(())
}

#[test]
fn raw_and_escaped_attribute_growth_cannot_publish_overbound_output() -> TestResult {
    let source = b"<CDXML/>";
    let doc = Document::parse(source)?;
    let root = doc.xml().root_element();
    {
        let value = "x".repeat(MAX_INPUT_BYTES);
        let final_xml = format!("<CDXML new=\"{value}\"/>");
        assert!(matches!(
            Document::parse(final_xml.as_bytes()),
            Err(Error::InputTooLarge)
        ));
        same_final_resource_error(
            doc.apply_edits(&[Edit::SetAttribute {
                element: root,
                name: "new",
                value: &value,
            }]),
            &final_xml,
        )?;
    }
    {
        let count = MAX_INPUT_BYTES / 5;
        let value = "&".repeat(count);
        let final_xml = format!("<CDXML new=\"{}\"/>", "&amp;".repeat(count));
        assert!(value.len() < MAX_INPUT_BYTES && final_xml.len() > MAX_INPUT_BYTES);
        assert!(matches!(
            Document::parse(final_xml.as_bytes()),
            Err(Error::InputTooLarge)
        ));
        same_final_resource_error(
            doc.apply_edits(&[Edit::SetAttribute {
                element: root,
                name: "new",
                value: &value,
            }]),
            &final_xml,
        )?;
    }
    assert_eq!(doc.as_bytes(), source);
    Ok(())
}
