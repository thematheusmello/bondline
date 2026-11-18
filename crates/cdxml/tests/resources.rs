use bondline_cdxml::{Document, Edit, ResourceError};
use std::{
    io::Write,
    process::{Command, Stdio},
};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const SOURCE: &str = "<?xml version='1.0'?>\r\n<CDXML Vendor='keep'><colortable id='5' extra='retained'><!--palette--><color r='1.000' g='1' b='1'/><color r='0' g='0' b='0'/><color r='.125' g='0.500' b='.875' note='keep'/></colortable><fonttable id='6'><font id='65535' name='A &amp; B' charset='opaque-charset'/><font id='0' name='μ' vendor='keep'/><font id='7.00' name=''/></fonttable><page id='1'><fragment id='2'><n id='7' p='72 72'/></fragment></page><other keep='&#x41;'/></CDXML>";
const EDITED: &str = "<?xml version='1.0'?>\r\n<CDXML Vendor='keep'><colortable id='5' extra='retained'><!--palette--><color r='1.000' g='1' b='1'/><color r='0' g='0' b='0'/><color r='.125' g='0.25' b='.875' note='keep'/></colortable><fonttable id='6'><font id='65535' name='New &amp; &lt;Face&gt;' charset='opaque-charset'/><font id='0' name='μ' vendor='keep'/><font id='7.00' name=''/></fonttable><page id='1'><fragment id='2'><n id='7' p='72 72'/></fragment></page><other keep='&#x41;'/></CDXML>";

#[test]
fn explicit_resources_follow_uint16_ids_and_ordered_palette_indices() -> TestResult {
    let document = Document::parse(SOURCE.as_bytes())?;
    let resources = document.resources()?;
    assert_eq!(
        resources.font_table().and_then(|n| n.attribute("id")),
        Some("6")
    );
    assert_eq!(
        resources.color_table().and_then(|n| n.attribute("id")),
        Some("5")
    );
    assert_eq!(
        resources.fonts().map(|f| f.id()).collect::<Vec<_>>(),
        [0, 7, 65535]
    );
    assert_eq!(resources.font(65535)?.name(), "A & B");
    assert_eq!(resources.font(65535)?.charset(), Some("opaque-charset"));
    assert_eq!(resources.font(0)?.name(), "μ");
    assert_eq!(resources.font(0)?.charset(), None);
    assert_eq!(resources.font(7)?.name(), "");
    assert_eq!(resources.font(7)?.element().attribute("id"), Some("7.00"));
    assert_eq!(
        resources
            .colors()
            .iter()
            .map(|c| c.index())
            .collect::<Vec<_>>(),
        [2, 3, 4]
    );
    assert_eq!(resources.color(0)?, [0.0, 0.0, 0.0]);
    assert_eq!(resources.color(1)?, [1.0, 1.0, 1.0]);
    assert_eq!(resources.color(2)?, [1.0, 1.0, 1.0]);
    assert_eq!(resources.color(3)?, [0.0, 0.0, 0.0]);
    assert_eq!(resources.color(4)?, [0.125, 0.5, 0.875]);
    assert_eq!(
        resources.colors().get(2).ok_or("colour")?.rgb(),
        [0.125, 0.5, 0.875]
    );
    assert_eq!(resources.color(5), Err(ResourceError::MissingColor(5)));
    assert!(matches!(
        resources.font(6),
        Err(ResourceError::MissingFont(6))
    ));
    assert_eq!(document.to_bytes(), SOURCE.as_bytes());
    Ok(())
}

#[test]
fn absent_tables_have_only_the_published_four_color_defaults_and_no_font_default() -> TestResult {
    let document = Document::parse(b"<CDXML><page/></CDXML>")?;
    let resources = document.resources()?;
    assert!(resources.font_table().is_none() && resources.color_table().is_none());
    assert_eq!(resources.fonts().len(), 0);
    assert!(resources.colors().is_empty());
    for (index, expected) in [(0, [0.0; 3]), (1, [1.0; 3]), (2, [1.0; 3]), (3, [0.0; 3])] {
        assert_eq!(resources.color(index)?, expected);
    }
    assert_eq!(resources.color(4), Err(ResourceError::MissingColor(4)));
    assert!(matches!(
        resources.font(0),
        Err(ResourceError::MissingFont(0))
    ));
    Ok(())
}

#[test]
fn a_present_one_color_table_does_not_invent_a_foreground_entry() -> TestResult {
    let document = Document::parse(
        b"<CDXML><colortable><color r='.25' g='.5' b='.75'/></colortable></CDXML>",
    )?;
    let resources = document.resources()?;
    assert_eq!(resources.color(2)?, [0.25, 0.5, 0.75]);
    assert_eq!(resources.color(3), Err(ResourceError::MissingColor(3)));
    assert_eq!(resources.color(0)?, [0.0; 3]);
    assert_eq!(resources.color(1)?, [1.0; 3]);
    Ok(())
}

#[test]
fn malformed_resource_data_is_a_typed_error_without_changing_syntax_admission_or_source()
-> TestResult {
    let cases = [
        ("<fonttable/>", ResourceError::EmptyTable("fonttable")),
        ("<colortable/>", ResourceError::EmptyTable("colortable")),
        (
            "<fonttable><font name='A'/></fonttable>",
            ResourceError::InvalidFontId,
        ),
        (
            "<fonttable><font id='1'/></fonttable>",
            ResourceError::MissingFontName,
        ),
        (
            "<fonttable><font id='1' name='A'/><font id='1.00' name='B'/></fonttable>",
            ResourceError::DuplicateFontId(1),
        ),
        (
            "<colortable><color r='0' g='0'/></colortable>",
            ResourceError::InvalidColorComponent("b"),
        ),
        (
            "<fonttable><color r='0' g='0' b='0'/></fonttable>",
            ResourceError::UnexpectedContent("fonttable"),
        ),
        (
            "<colortable><other/><color r='0' g='0' b='0'/></colortable>",
            ResourceError::UnexpectedContent("colortable"),
        ),
        (
            "<fonttable>text<font id='1' name='A'/></fonttable>",
            ResourceError::UnexpectedContent("fonttable"),
        ),
        (
            "<colortable>\u{a0}<color r='0' g='0' b='0'/></colortable>",
            ResourceError::UnexpectedContent("colortable"),
        ),
        (
            "<fonttable><font id='1' name='A'><other/></font></fonttable>",
            ResourceError::UnexpectedContent("font"),
        ),
        (
            "<colortable><color r='0' g='0' b='0'>text</color></colortable>",
            ResourceError::UnexpectedContent("color"),
        ),
    ];
    for (body, expected) in cases {
        let source = format!("<CDXML>{body}</CDXML>");
        let document = Document::parse(source.as_bytes())?;
        assert_eq!(document.resources().err(), Some(expected), "{body}");
        assert_eq!(document.as_bytes(), source.as_bytes());
        assert_eq!(document.to_bytes(), source.as_bytes());
    }
    Ok(())
}

#[test]
fn font_ids_use_the_existing_exact_integer_decoder_and_the_unsigned_16_bit_domain() -> TestResult {
    for value in [
        "-1",
        "65536",
        "1.5",
        "1e0",
        "",
        "NaN",
        "9999999999999999999999999999",
    ] {
        let source = format!("<CDXML><fonttable><font id='{value}' name='A'/></fonttable></CDXML>");
        assert_eq!(
            Document::parse(source.as_bytes())?.resources().err(),
            Some(ResourceError::InvalidFontId),
            "{value}"
        );
    }
    for (value, expected) in [("0", 0), ("+7.000", 7), (" 65535 ", 65535)] {
        let source = format!("<CDXML><fonttable><font id='{value}' name='A'/></fonttable></CDXML>");
        assert_eq!(
            Document::parse(source.as_bytes())?
                .resources()?
                .font(expected)?
                .id(),
            expected
        );
    }
    Ok(())
}

#[test]
fn rgb_channels_are_finite_unit_interval_values_and_are_not_quantized() -> TestResult {
    for value in ["-0.001", "1.001", "NaN", "inf", "-inf", "1e999", "", "half"] {
        for channel in ["r", "g", "b"] {
            let attributes = ["r", "g", "b"]
                .map(|name| format!("{name}='{}'", if name == channel { value } else { "0" }))
                .join(" ");
            let source = format!("<CDXML><colortable><color {attributes}/></colortable></CDXML>");
            assert_eq!(
                Document::parse(source.as_bytes())?.resources().err(),
                Some(ResourceError::InvalidColorComponent(channel)),
                "{source}"
            );
        }
    }
    let document = Document::parse(
        b"<CDXML><colortable><color r='-0' g='1e-1' b='1.0'/></colortable></CDXML>",
    )?;
    assert_eq!(document.resources()?.color(2)?, [0.0, 0.1, 1.0]);
    Ok(())
}

#[test]
fn numeric_xml_whitespace_and_qualified_lookalike_attributes_keep_their_distinct_meanings()
-> TestResult {
    let source = b"<CDXML xmlns:x='urn:other'><colortable>\n<?note keep?><color x:r='NaN' r=' &#9;.125&#13; ' g=' 0.500 ' b=' 1 '/></colortable><fonttable><!--keep--><font x:id='invalid' id=' 7.00 ' x:name='wrong' name='right' x:charset='wrong'/></fonttable></CDXML>";
    let document = Document::parse(source)?;
    let resources = document.resources()?;
    assert_eq!(resources.color(2)?, [0.125, 0.5, 1.0]);
    assert_eq!(resources.font(7)?.name(), "right");
    assert_eq!(resources.font(7)?.charset(), None);
    assert_eq!(document.as_bytes(), source);
    let qualified_only = Document::parse(
        b"<CDXML xmlns:x='urn:other'><fonttable><font x:id='7' name='A'/></fonttable></CDXML>",
    )?;
    assert_eq!(
        qualified_only.resources().err(),
        Some(ResourceError::InvalidFontId)
    );
    Ok(())
}

#[test]
fn duplicate_misplaced_and_namespaced_table_lookalikes_are_not_silently_resolved() -> TestResult {
    for (body, expected) in [
        (
            "<fonttable/><fonttable/>",
            ResourceError::DuplicateTable("fonttable"),
        ),
        (
            "<colortable/><colortable/>",
            ResourceError::DuplicateTable("colortable"),
        ),
        (
            "<page><fonttable/></page>",
            ResourceError::UnsupportedTable("fonttable"),
        ),
        (
            "<page><colortable/></page>",
            ResourceError::UnsupportedTable("colortable"),
        ),
        (
            "<x:fonttable xmlns:x='urn:other'/>",
            ResourceError::UnsupportedTable("fonttable"),
        ),
        (
            "<colortable xmlns='urn:other'/>",
            ResourceError::UnsupportedTable("colortable"),
        ),
        (
            "<fonttable xmlns:x='urn:other'><x:font id='1' name='A'/></fonttable>",
            ResourceError::UnexpectedContent("fonttable"),
        ),
        (
            "<colortable><color xmlns='urn:other' r='0' g='0' b='0'/></colortable>",
            ResourceError::UnexpectedContent("colortable"),
        ),
    ] {
        let source = format!("<CDXML>{body}</CDXML>");
        assert_eq!(
            Document::parse(source.as_bytes())?.resources().err(),
            Some(expected),
            "{source}"
        );
    }
    Ok(())
}

#[test]
fn explicit_empty_namespace_resets_keep_resource_elements_unnamespaced() -> TestResult {
    let source = b"<CDXML><colortable xmlns=''><color xmlns='' r='.25' g='.5' b='.75'/></colortable><fonttable xmlns=''><font xmlns='' id='0' name='A'/></fonttable></CDXML>";
    let document = Document::parse(source)?;
    let resources = document.resources()?;
    assert_eq!(resources.font(0)?.name(), "A");
    assert_eq!(resources.color(2)?, [0.25, 0.5, 0.75]);
    assert_eq!(document.as_bytes(), source);
    // Existing root admission is deliberately unchanged by the resource view.
    assert!(matches!(
        Document::parse(b"<CDXML xmlns=''/>"),
        Err(bondline_cdxml::Error::InvalidRoot)
    ));
    Ok(())
}

fn expat_resources(bytes: &[u8], edited: bool) -> TestResult {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child = Command::new(python).args(["-X", "utf8", "-B", "-c", r#"
import sys, xml.parsers.expat as expat
from decimal import Decimal
fonts=[]; colors=[]; stack=[]; others=[]; comments=[]
def start(name, attrs):
    stack.append(name)
    if stack==['CDXML','fonttable','font']:
        value=Decimal(attrs['id']); assert value==int(value) and 0<=value<=65535
        fonts.append((int(value),attrs['name'],attrs.get('charset'),attrs.get('vendor')))
    if stack==['CDXML','colortable','color']:
        colors.append(tuple(Decimal(attrs[c]) for c in ('r','g','b')))
    if name=='other': others.append(attrs)
p=expat.ParserCreate(namespace_separator='|')
p.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
p.StartElementHandler=start; p.EndElementHandler=lambda name:stack.pop(); p.CommentHandler=comments.append
p.Parse(sys.stdin.buffer.read(), True)
edited=sys.argv[1]=='yes'
assert fonts==[(65535,'New & <Face>' if edited else 'A & B','opaque-charset',None),(0,'μ',None,'keep'),(7,'',None,None)],fonts
assert colors==[(Decimal(1),)*3,(Decimal(0),)*3,(Decimal('.125'),Decimal('.25') if edited else Decimal('.5'),Decimal('.875'))],colors
assert others==[{'keep':'A'}] and comments==['palette'],(others,comments)
"#, if edited { "yes" } else { "no" }]).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
    child.stdin.take().ok_or("oracle stdin")?.write_all(bytes)?;
    let result = child.wait_with_output()?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(())
}

#[test]
fn resource_nodes_drive_existing_source_edits_with_literal_bytes_and_independent_expat_decimal()
-> TestResult {
    // Qualify the independent consumer on both manually declared inputs before product output.
    expat_resources(SOURCE.as_bytes(), false)?;
    expat_resources(EDITED.as_bytes(), true)?;
    let document = Document::parse(SOURCE.as_bytes())?;
    let resources = document.resources()?;
    let output = document.apply_edits(&[
        Edit::SetAttribute {
            element: resources.font(65535)?.element(),
            name: "name",
            value: "New & <Face>",
        },
        Edit::SetAttribute {
            element: resources.colors().get(2).ok_or("colour")?.element(),
            name: "g",
            value: "0.25",
        },
    ])?;
    assert_eq!(output, EDITED.as_bytes());
    expat_resources(&output, true)?;
    let reopened = Document::parse(&output)?;
    assert_eq!(reopened.resources()?.font(65535)?.name(), "New & <Face>");
    assert_eq!(reopened.resources()?.color(4)?, [0.125, 0.25, 0.875]);
    assert_eq!(document.as_bytes(), SOURCE.as_bytes());
    let other = Document::parse(SOURCE.as_bytes())?;
    assert!(matches!(
        other.apply_edits(&[Edit::SetAttribute {
            element: resources.font(65535)?.element(),
            name: "name",
            value: "foreign"
        }]),
        Err(bondline_cdxml::Error::InvalidElement)
    ));
    Ok(())
}
