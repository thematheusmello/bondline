use bondline_cdxml::{Document, Error, MAX_INPUT_BYTES};
use std::{
    io::Write,
    process::{Command, Stdio},
};

fn expat_attribute(
    bytes: &[u8],
    tag: &str,
    id: &str,
    name: &str,
    value: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut process = Command::new(python)
        .args([
            "-c",
            r#"
import sys,xml.parsers.expat as expat
stream=sys.stdin.buffer
tag,identifier,name,value=[stream.read(int(stream.readline())).decode('utf8') for _ in range(4)]
found=[]
parser=expat.ParserCreate(namespace_separator='|')
def start(element,attributes):
    if element==tag and attributes.get('id','')==identifier:
        found.append(attributes.get(name))
parser.StartElementHandler=start
parser.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
parser.Parse(stream.read(),True)
assert found==[value], 'attribute differs from independent expectation'
"#,
        ])
        .env("PYTHONIOENCODING", "ascii")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut input = process.stdin.take().ok_or("missing Expat stdin")?;
    for field in [tag, id, name, value] {
        writeln!(input, "{}", field.len())?;
        input.write_all(field.as_bytes())?;
    }
    input.write_all(bytes)?;
    drop(input);
    let output = process.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

#[test]
fn changing_an_atom_coordinate_preserves_every_unrelated_source_byte()
-> Result<(), Box<dyn std::error::Error>> {
    let source = include_bytes!("fixtures/unknown.cdxml");
    let document = Document::parse(source)?;
    let atom = document
        .xml()
        .descendants()
        .find(|node| node.attribute("id") == Some("3"))
        .ok_or("missing atom")?;
    let output = document.replace_attribute(atom, "p", "13.25 25.00")?;
    let expected = std::str::from_utf8(source)?.replacen("p='12.50 25.00'", "p='13.25 25.00'", 1);
    assert_eq!(output, expected.as_bytes());
    assert_eq!(document.as_bytes(), source);
    expat_attribute(&output, "n", "3", "p", "13.25 25.00")?;
    Ok(())
}

#[test]
fn namespaced_values_escape_xml_and_keep_the_original_quote_delimiters()
-> Result<(), Box<dyn std::error::Error>> {
    let value = "μ & < > \" ' \t\r\n";
    let escaped = "μ &amp; &lt; &gt; &quot; &apos; &#x9;&#xD;&#xA;";
    for quote in ['\'', '"'] {
        let source = format!(
            "<?xml version='1.0'?><CDXML xmlns:f='urn:test' untouched='00255.00' f:value = {quote}old{quote}><f:opaque><![CDATA[x < y]]></f:opaque></CDXML>"
        );
        let document = Document::parse(source.as_bytes())?;
        assert!(matches!(
            document.replace_attribute(document.xml().root_element(), "xmlns:f", "urn:changed"),
            Err(Error::MissingAttribute)
        ));
        let output = document.replace_attribute(document.xml().root_element(), "f:value", value)?;
        let expected = source.replacen(
            &format!("{quote}old{quote}"),
            &format!("{quote}{escaped}{quote}"),
            1,
        );
        assert_eq!(output, expected.as_bytes());
        expat_attribute(&output, "CDXML", "", "urn:test|value", value)?;
    }
    Ok(())
}

#[test]
fn no_ops_and_large_attribute_positions_do_not_normalize_or_overwrite_other_content()
-> Result<(), Box<dyn std::error::Error>> {
    let name = format!("attribute_{}", "a".repeat(70_000));
    let spaces = " \t\r\n".repeat(80);
    let source = format!(
        "\u{feff}<CDXML entity='&#x41;&#65;' empty = '' {name}{spaces}={spaces}'before' tail=\"untouched\"/>\r\n"
    );
    let document = Document::parse(source.as_bytes())?;
    let root = document.xml().root_element();
    assert_eq!(
        document.replace_attribute(root, "entity", "AA")?,
        source.as_bytes()
    );
    let output = document.replace_attribute(root, &name, "after")?;
    assert_eq!(output, source.replacen("'before'", "'after'", 1).as_bytes());
    expat_attribute(&output, "CDXML", "", &name, "after")?;
    let empty = document.replace_attribute(root, "empty", "nonempty")?;
    assert_eq!(
        empty,
        source
            .replacen("empty = ''", "empty = 'nonempty'", 1)
            .as_bytes()
    );
    let cleared = document.replace_attribute(root, "entity", "")?;
    expat_attribute(&cleared, "CDXML", "", "entity", "")?;
    Ok(())
}

#[test]
fn invalid_edits_never_mutate_the_input_or_return_partial_xml()
-> Result<(), Box<dyn std::error::Error>> {
    let source = b"<CDXML value='original'/>";
    let document = Document::parse(source)?;
    let other = Document::parse(source)?;
    let root = document.xml().root_element();
    assert!(matches!(
        document.replace_attribute(other.xml().root_element(), "value", "changed"),
        Err(Error::InvalidElement)
    ));
    assert!(matches!(
        document.replace_attribute(document.xml().root(), "value", "changed"),
        Err(Error::InvalidElement)
    ));
    assert!(matches!(
        document.replace_attribute(root, "missing", "changed"),
        Err(Error::MissingAttribute)
    ));
    for value in ["\0", "\u{1}", "\u{b}", "\u{fffe}", "\u{ffff}"] {
        assert!(matches!(
            document.replace_attribute(root, "value", value),
            Err(Error::InvalidAttributeValue)
        ));
    }
    let expansion = "\"".repeat(MAX_INPUT_BYTES / 6 + 1);
    assert!(matches!(
        document.replace_attribute(root, "value", &expansion),
        Err(Error::InputTooLarge)
    ));
    assert_eq!(document.as_bytes(), source);
    assert_eq!(other.as_bytes(), source);
    Ok(())
}

#[test]
fn xml_character_range_boundaries_are_confirmed_by_expat() -> Result<(), Box<dyn std::error::Error>>
{
    let source = b"<CDXML value='before'/>";
    let document = Document::parse(source)?;
    let value = "\t\n\r \u{d7ff}\u{e000}\u{fffd}\u{10000}\u{10ffff}";
    let output = document.replace_attribute(document.xml().root_element(), "value", value)?;
    expat_attribute(&output, "CDXML", "", "value", value)?;
    Ok(())
}
