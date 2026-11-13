use bondline_cdxml::{
    Document, Error, MAX_ATTRIBUTES, MAX_DEPTH, MAX_INPUT_BYTES, MAX_NODES, Writer,
};
use std::{
    io::Write,
    process::{Command, Stdio},
};

#[test]
fn new_documents_preserve_attribute_values_under_independent_expat_decoding()
-> Result<(), Box<dyn std::error::Error>> {
    let mut writer = Writer::new(&[
        ("CreationProgram", "Bondline"),
        ("Comment", "A & B < C > D \"'\t\r\nμ²🧪"),
    ])?;
    writer.start_element(
        "page",
        &[
            ("id", "1"),
            ("DrawingSpace", "Poster"),
            ("Width", "612"),
            ("Height", "792"),
        ],
    )?;
    writer.start_element("fragment", &[("id", "2")])?;
    writer.empty_element(
        "n",
        &[
            ("id", "3"),
            ("Element", "6"),
            ("p", "-12.5 20"),
            ("NumHydrogens", "3"),
        ],
    )?;
    writer.empty_element("n", &[("id", "4"), ("Element", "8"), ("Charge", "-1")])?;
    writer.empty_element("b", &[("id", "5"), ("B", "3"), ("E", "4"), ("Order", "1")])?;
    writer.end_element()?;
    writer.end_element()?;
    let output = writer.finish()?;
    assert_eq!(Document::parse(&output)?.to_bytes(), output);
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child = Command::new(python)
        .args([
            "-c",
            r#"
import sys,xml.parsers.expat as expat
nodes=[]
p=expat.ParserCreate(namespace_separator='|')
p.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
p.StartElementHandler=lambda name,attrs:nodes.append((name,attrs))
p.Parse(sys.stdin.buffer.read(),True)
assert [name for name,_ in nodes]==['CDXML','page','fragment','n','n','b']
assert nodes[0][1]=={'CreationProgram':'Bondline','Comment':'A & B < C > D "\'\t\r\nμ²🧪'}
assert nodes[1][1]=={'id':'1','DrawingSpace':'Poster','Width':'612','Height':'792'}
assert nodes[3][1]=={'id':'3','Element':'6','p':'-12.5 20','NumHydrogens':'3'}
assert nodes[4][1]=={'id':'4','Element':'8','Charge':'-1'}
assert nodes[5][1]=={'id':'5','B':'3','E':'4','Order':'1'}
"#,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("oracle stdin")?
        .write_all(&output)?;
    let result = child.wait_with_output()?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(())
}

#[test]
fn invalid_operations_are_atomic_and_unbalanced_output_is_rejected()
-> Result<(), Box<dyn std::error::Error>> {
    let mut writer = Writer::new(&[])?;
    for name in ["", "1n", "n x", "n:foreign", "n<", "μ"] {
        assert!(writer.empty_element(name, &[]).is_err());
        assert!(writer.start_element("page", &[(name, "x")]).is_err());
    }
    assert!(
        writer
            .empty_element("n", &[("id", "1"), ("id", "2")])
            .is_err()
    );
    assert!(
        writer
            .empty_element("n", &[("xmlns", "urn:changed")])
            .is_err()
    );
    for value in ["\0", "\u{1}", "\u{b}", "\u{fffe}", "\u{ffff}"] {
        assert!(writer.start_element("page", &[("Comment", value)]).is_err());
    }
    assert!(writer.end_element().is_err(), "root is owned by finish");
    writer.empty_element("page", &[("id", "1")])?;
    let output = writer.finish()?;
    let parsed = Document::parse(&output)?;
    assert_eq!(
        parsed
            .xml()
            .root_element()
            .children()
            .filter(|n| n.is_element())
            .count(),
        1
    );
    let mut unfinished = Writer::new(&[])?;
    unfinished.start_element("page", &[])?;
    assert!(unfinished.finish().is_err());
    Ok(())
}

#[test]
fn writer_reserves_closing_tags_and_bounds_escaped_output_before_writing()
-> Result<(), Box<dyn std::error::Error>> {
    let mut writer = Writer::new(&[])?;
    let oversized = "<".repeat(MAX_INPUT_BYTES / 4);
    assert!(matches!(
        writer.start_element("page", &[("Comment", &oversized)]),
        Err(Error::InputTooLarge)
    ));
    assert!(matches!(
        writer.start_element(&"a".repeat(MAX_INPUT_BYTES / 2), &[]),
        Err(Error::InputTooLarge)
    ));
    writer.empty_element("page", &[])?;
    assert!(writer.finish()?.len() < 128);
    Ok(())
}

#[test]
fn writer_enforces_depth_node_and_attribute_budgets_without_losing_prior_state()
-> Result<(), Box<dyn std::error::Error>> {
    let mut deep = Writer::new(&[])?;
    for _ in 1..MAX_DEPTH {
        deep.start_element("page", &[])?;
    }
    assert!(matches!(
        deep.empty_element("n", &[]),
        Err(Error::DepthLimit)
    ));
    for _ in 1..MAX_DEPTH {
        deep.end_element()?;
    }
    deep.finish()?;
    let mut wide = Writer::new(&[])?;
    for _ in 2..MAX_NODES {
        wide.empty_element("page", &[])?;
    }
    assert!(wide.empty_element("page", &[]).is_err());
    wide.finish()?;
    let names: Vec<_> = (0..257).map(|i| format!("a{i}")).collect();
    let attributes: Vec<_> = names.iter().map(|name| (name.as_str(), "x")).collect();
    let mut attributes_writer = Writer::new(&[])?;
    assert!(matches!(
        attributes_writer.empty_element("page", &attributes),
        Err(Error::AttributeLimit)
    ));
    let chunk = attributes.get(..256).ok_or("attributes")?;
    for _ in 0..MAX_ATTRIBUTES / 256 {
        attributes_writer.empty_element("page", chunk)?;
    }
    attributes_writer.empty_element(
        "page",
        chunk.get(..MAX_ATTRIBUTES % 256).ok_or("last attributes")?,
    )?;
    assert!(matches!(
        attributes_writer.empty_element("page", &[("extra", "x")]),
        Err(Error::AttributeLimit)
    ));
    attributes_writer.finish()?;
    Ok(())
}
