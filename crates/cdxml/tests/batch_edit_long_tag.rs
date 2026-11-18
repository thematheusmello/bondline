//! Exact-byte control for repeated edits of one long opening tag; timings are observations.
use bondline_cdxml::{Document, Edit};
use std::time::Instant;

#[test]
fn long_opening_tag_accepts_255_new_attributes_in_caller_order()
-> Result<(), Box<dyn std::error::Error>> {
    let padding = "x".repeat(128 * 1024);
    let source = format!("<CDXML padding='{padding}' />");
    let document = Document::parse(source.as_bytes())?;
    let names: Vec<_> = (0..255).map(|i| format!("added{i}")).collect();
    let edits: Vec<_> = names
        .iter()
        .map(|name| Edit::SetAttribute {
            element: document.xml().root_element(),
            name,
            value: "v",
        })
        .collect();
    let mut expected = format!("<CDXML padding='{padding}' ");
    for i in 0..255 {
        expected.push_str(&format!(" added{i}=\"v\""));
    }
    expected.push_str("/>");
    let start = Instant::now();
    let output = document.apply_edits(&edits)?;
    eprintln!("long opening-tag batch: {:?}", start.elapsed());
    assert_eq!(output, expected.as_bytes());
    assert_eq!(document.as_bytes(), source.as_bytes());
    Ok(())
}

#[test]
fn batch_source_spans_survive_compacted_parser_offset_limits()
-> Result<(), Box<dyn std::error::Error>> {
    // roxmltree documents truncated QName/u8-whitespace offsets; its full range stays exact.
    let name = "a".repeat(70_000);
    let spaces = " \t\r\n".repeat(80);
    let source = format!("<CDXML {name}{spaces}={spaces}'&#65;' drop='old' tail=\"keep\"/>");
    let doc = Document::parse(source.as_bytes())?;
    let root = doc.xml().root_element();
    assert_eq!(
        doc.apply_edits(&[Edit::SetAttribute {
            element: root,
            name: &name,
            value: "A"
        }])?,
        source.as_bytes()
    );
    let output = doc.apply_edits(&[
        Edit::SetAttribute {
            element: root,
            name: &name,
            value: "after & '<",
        },
        Edit::RemoveAttribute {
            element: root,
            name: "drop",
        },
    ])?;
    let expected =
        format!("<CDXML {name}{spaces}={spaces}'after &amp; &apos;&lt;'  tail=\"keep\"/>");
    assert_eq!(output, expected.as_bytes());
    assert_eq!(doc.as_bytes(), source.as_bytes());
    Ok(())
}
