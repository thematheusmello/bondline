use bondline_cdxml::{Document, Error, MAX_DEPTH, MAX_INPUT_BYTES, MAX_NODES};
use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn unknown_syntax_round_trips_byte_for_byte_and_expat_checks_independent_values()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = include_bytes!("fixtures/unknown.cdxml");
    let document = Document::parse(fixture)?;
    assert_eq!(document.to_bytes(), fixture);
    assert_eq!(document.xml().root_element().tag_name().name(), "CDXML");
    let bond = document
        .xml()
        .descendants()
        .find(|node| node.has_tag_name("b"))
        .ok_or("missing bond")?;
    assert_eq!(bond.attribute("Order"), Some("2.00"));
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut oracle = Command::new(python).args(["-c", r#"
import sys,xml.parsers.expat as expat
parser=expat.ParserCreate(namespace_separator='|')
elements=[]
text=[]
comments=[]
instructions=[]
parser.StartElementHandler=lambda name,attrs: elements.append((name,attrs))
parser.CharacterDataHandler=text.append
parser.CommentHandler=comments.append
parser.ProcessingInstructionHandler=lambda target,data: instructions.append((target,data))
parser.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
parser.Parse(sys.stdin.buffer.read(),True)
assert [name for name,_ in elements]==['CDXML','page','fragment','n','n','b','urn:bondline:fixture|data','urn:bondline:fixture|record','t','s']
assert elements[0][1]['urn:bondline:fixture|setting']=='opaque & quoted'
assert elements[3][1]['Isotope']=='13' and elements[5][1]['Order']=='2.00'
assert 'x < y & z' in ''.join(text) and 'μ²' in ''.join(text) and 'A & B Δ' in ''.join(text)
assert ' keep ' in comments
assert instructions==[('bondline','keep-this-processing-instruction')]
"#]).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
    oracle
        .stdin
        .take()
        .ok_or("missing oracle stdin")?
        .write_all(&document.to_bytes())?;
    let output = oracle.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let windows = std::str::from_utf8(fixture)?.replace('\n', "\r\n");
    let mut bom = vec![0xef, 0xbb, 0xbf];
    bom.extend_from_slice(windows.as_bytes());
    assert_eq!(Document::parse(&bom)?.to_bytes(), bom);
    Ok(())
}

#[test]
fn malformed_unsupported_or_hostile_documents_fail_without_partial_output() {
    for input in [
        b"".as_slice(),
        b"<Other/>",
        b"<CDXML><page></CDXML>",
        b"<CDXML/><CDXML/>",
        b"<CDXML bad='1' bad='2'/>",
        b"<CDXML>&missing;</CDXML>",
        b"<CDXML>&#0;</CDXML>",
        b"<CDXML>\0</CDXML>",
        b"<?xml version='1.1'?><CDXML/>",
        b"<?xml version='1.0' encoding='UTF-16'?><CDXML/>",
        b"<!DOCTYPE CDXML [<!ENTITY custom 'expanded'>]><CDXML>&custom;</CDXML>",
        b"<CDXML>\xff</CDXML>",
    ] {
        assert!(Document::parse(input).is_err(), "accepted {input:?}");
    }
    assert!(matches!(
        Document::parse(&vec![b' '; MAX_INPUT_BYTES + 1]),
        Err(Error::InputTooLarge)
    ));
    let deep = format!(
        "<CDXML>{}{}</CDXML>",
        "<page>".repeat(MAX_DEPTH),
        "</page>".repeat(MAX_DEPTH)
    );
    assert!(matches!(
        Document::parse(deep.as_bytes()),
        Err(Error::DepthLimit)
    ));
    let wide = format!("<CDXML>{}</CDXML>", "<n/>".repeat(MAX_NODES as usize));
    assert!(Document::parse(wide.as_bytes()).is_err());
}

#[test]
fn attribute_budgets_are_enforced_before_building_the_tree()
-> Result<(), Box<dyn std::error::Error>> {
    let attributes: String = (0..256).map(|index| format!(" a{index}='x'")).collect();
    assert!(Document::parse(format!("<CDXML{attributes}/>").as_bytes()).is_ok());
    assert!(matches!(
        Document::parse(format!("<CDXML{attributes} excess='x'/>").as_bytes()),
        Err(Error::AttributeLimit)
    ));
    let elements = format!("<n{attributes}/>").repeat(391);
    assert!(matches!(
        Document::parse(format!("<CDXML>{elements}</CDXML>").as_bytes()),
        Err(Error::AttributeLimit)
    ));
    // A bracket inside a quoted external identifier is not an internal subset.
    let external = b"<!DOCTYPE CDXML SYSTEM 'https://example.invalid/[offline].dtd'><CDXML/>";
    assert_eq!(Document::parse(external)?.as_bytes(), external);
    Ok(())
}
