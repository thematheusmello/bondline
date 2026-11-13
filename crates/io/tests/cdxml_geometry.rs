use bondline_io::cdxml::{Error, Geometry};
use bondline_model::Point2;
use std::{
    io::Write,
    process::{Command, Stdio},
};

const ORIGINAL: &[u8] = include_bytes!("../../cdxml/tests/fixtures/unknown.cdxml");

#[test]
fn node_geometry_uses_xy_points_and_does_not_invent_missing_coordinates()
-> Result<(), Box<dyn std::error::Error>> {
    let source =
        br#"<CDXML><page><fragment><n id="1" p="72 144"/><n id="2"/></fragment></page></CDXML>"#;
    let geometry = Geometry::parse(source)?;
    let nodes = geometry.nodes();
    assert_eq!(nodes.len(), 2);
    let first = nodes.first().ok_or("missing first node")?;
    assert_eq!(first.id(), "1");
    assert_eq!(first.position(), Some(Point2::new(72.0, 144.0)?));
    assert_eq!(nodes.get(1).ok_or("missing second node")?.position(), None);
    assert_eq!(geometry.as_bytes(), source);
    Ok(())
}

#[test]
fn moving_a_node_preserves_unmodelled_properties_and_expat_confirms_the_result()
-> Result<(), Box<dyn std::error::Error>> {
    let geometry = Geometry::parse(ORIGINAL)?;
    let output = geometry.with_node_position("3", Point2::new(72.0, 144.0)?)?;
    let expected = std::str::from_utf8(ORIGINAL)?.replacen("p='12.50 25.00'", "p='72 144'", 1);
    assert_eq!(output, expected.as_bytes());
    assert_eq!(geometry.as_bytes(), ORIGINAL);
    assert_eq!(
        geometry.with_node_position("3", Point2::new(12.5, 25.0)?)?,
        ORIGINAL
    );
    let imported = Geometry::parse(&output)?;
    assert_eq!(
        imported.nodes().first().ok_or("missing node")?.position(),
        Some(Point2::new(72.0, 144.0)?)
    );

    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut oracle = Command::new(python)
        .args([
            "-c",
            r#"
import sys,xml.parsers.expat as expat
nodes={}
parser=expat.ParserCreate(namespace_separator='|')
def start(name,attrs):
    if 'id' in attrs:
        nodes[attrs['id']]=(name,attrs)
parser.StartElementHandler=start
parser.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
parser.Parse(sys.stdin.buffer.read(),True)
assert nodes['3'][1]['p']=='72 144'
assert nodes['3'][1]['Isotope']=='13'
assert nodes['4'][1]['p']=='42.50 25'
assert nodes['5'][1]['Order']=='2.00'
assert nodes['5'][1]['urn:bondline:fixture|bond']=='untouched'
"#,
        ])
        .env("PYTHONIOENCODING", "ascii")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    oracle
        .stdin
        .take()
        .ok_or("missing Expat stdin")?
        .write_all(&output)?;
    let result = oracle.wait_with_output()?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(())
}

#[test]
fn malformed_coordinates_and_unsupported_node_layout_fail_explicitly()
-> Result<(), Box<dyn std::error::Error>> {
    for value in [
        "",
        " ",
        "72",
        "72 144 216",
        "NaN 0",
        "0 inf",
        "-100001 0",
        "0 100001",
        "72,144",
        "72\u{a0}144",
    ] {
        let source =
            format!("<CDXML><page><fragment><n id='1' p='{value}'/></fragment></page></CDXML>");
        assert!(matches!(
            Geometry::parse(source.as_bytes()),
            Err(Error::InvalidPosition)
        ));
    }
    for source in [
        "<CDXML><n id='1' p='1 2'/></CDXML>",
        "<CDXML><page><group><fragment><n id='1' p='1 2'/></fragment></group></page></CDXML>",
        "<CDXML><wrapper><CDXML><page><fragment><n id='1' p='1 2'/></fragment></page></CDXML></wrapper></CDXML>",
        "<CDXML xmlns:f='urn:test'><page><f:fragment><n id='1' p='1 2'/></f:fragment></page></CDXML>",
    ] {
        assert!(matches!(
            Geometry::parse(source.as_bytes()),
            Err(Error::UnsupportedNodeLayout)
        ));
    }
    let duplicate = b"<CDXML><page><fragment><n id='1'/><n id='1'/></fragment></page></CDXML>";
    assert!(matches!(
        Geometry::parse(duplicate),
        Err(Error::DuplicateNodeId)
    ));
    let escaped_duplicate =
        b"<CDXML><page><fragment><n id='1'/><n id='&#49;'/></fragment></page></CDXML>";
    assert!(matches!(
        Geometry::parse(escaped_duplicate),
        Err(Error::DuplicateNodeId)
    ));
    for id in ["", " id=''", " id=' '"] {
        let source = format!("<CDXML><page><fragment><n{id}/></fragment></page></CDXML>");
        assert!(matches!(
            Geometry::parse(source.as_bytes()),
            Err(Error::InvalidNodeId)
        ));
    }
    let extension = b"<CDXML xmlns:f='urn:test'><f:n id='1' p='not a position'/></CDXML>";
    let geometry = Geometry::parse(extension)?;
    assert!(geometry.nodes().is_empty());
    assert_eq!(geometry.as_bytes(), extension);
    Ok(())
}

#[test]
fn missing_positions_and_unknown_nodes_reject_edits_without_source_changes()
-> Result<(), Box<dyn std::error::Error>> {
    let source =
        b"<CDXML><page><fragment><n id='1'/><n id='2' p='&#55;2 144'/></fragment></page></CDXML>";
    let geometry = Geometry::parse(source)?;
    let target = Point2::new(72.0, 144.0)?;
    assert!(matches!(
        geometry.with_node_position("1", target),
        Err(Error::MissingPosition)
    ));
    assert!(matches!(
        geometry.with_node_position("missing", target),
        Err(Error::MissingNode)
    ));
    assert_eq!(geometry.with_node_position("2", target)?, source);
    assert_eq!(geometry.as_bytes(), source);
    Ok(())
}
