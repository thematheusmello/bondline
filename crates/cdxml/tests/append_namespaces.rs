//! Independently declared namespace self-containment controls; Root owns all execution.
use bondline_cdxml::{Document, Edit, Error};
use std::{
    io::Write,
    process::{Command, Stdio},
};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

struct Parent {
    source: &'static str,
    prefix: &'static str,
    suffix: &'static str,
    ancestors: &'static [&'static str],
}

const PARENTS: [Parent; 2] = [
    Parent {
        source: "<CDXML xmlns:f='urn:outer'><f:box xmlns='urn:default'/></CDXML>",
        prefix: "<CDXML xmlns:f='urn:outer'><f:box xmlns='urn:default'>",
        suffix: "</f:box></CDXML>",
        ancestors: &["CDXML", "urn:outer|box"],
    },
    Parent {
        source: "<CDXML xmlns:f='urn:outer'><scope xmlns='urn:default'><f:box><!--kept--><?keep data?></f:box></scope></CDXML>",
        prefix: "<CDXML xmlns:f='urn:outer'><scope xmlns='urn:default'><f:box><!--kept--><?keep data?>",
        suffix: "</f:box></scope></CDXML>",
        ancestors: &["CDXML", "urn:default|scope", "urn:outer|box"],
    },
];

// This external reader has no access to the Rust namespace decoder or edit plan.
// Expected expanded names are literal declarations, not an own-reader round trip.
fn expat_expanded_names(bytes: &[u8], expected: &[&str]) -> TestResult {
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
import sys, xml.parsers.expat as expat
names = []
p = expat.ParserCreate(namespace_separator='|')
p.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
p.StartElementHandler = lambda name, attrs: names.append(name)
p.Parse(sys.stdin.buffer.read(), True)
assert names == sys.argv[1:], (names, sys.argv[1:])
"#,
        ])
        .args(expected)
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
fn destination_default_namespace_cannot_capture_unqualified_fragment_elements() -> TestResult {
    for parent in &PARENTS {
        let doc = Document::parse(parent.source.as_bytes())?;
        let element = doc
            .xml()
            .descendants()
            .find(|n| {
                n.is_element()
                    && n.tag_name().name() == "box"
                    && n.tag_name().namespace() == Some("urn:outer")
            })
            .ok_or("original box")?;
        for xml in [
            "<n/>",
            "<n/><m xmlns='urn:local'/>",
            "<n xmlns='urn:local'/><m/>",
            "<g:item xmlns:g='urn:inner'><child/></g:item>",
            "<g:item xmlns:g='urn:inner'><g:leaf/><child/></g:item>",
        ] {
            // A preceding otherwise-valid edit must not escape from the failed batch.
            let result = doc.apply_edits(&[
                Edit::SetAttribute {
                    element: doc.xml().root_element(),
                    name: "probe",
                    value: "valid",
                },
                Edit::AppendChildren { element, xml },
            ]);
            assert!(
                matches!(result, Err(Error::InvalidFragment)),
                "source={} fragment={xml}",
                parent.source
            );
            assert_eq!(doc.as_bytes(), parent.source.as_bytes());
        }
    }
    Ok(())
}

#[test]
fn explicit_namespace_scopes_and_inner_undeclarations_remain_valid() -> TestResult {
    let accepted: [(&str, &[&str]); 6] = [
        ("<n xmlns=''><child/></n>", &["n", "child"]),
        (
            "<n xmlns='urn:local'><child xmlns=''><leaf/></child><again/></n>",
            &["urn:local|n", "child", "leaf", "urn:local|again"],
        ),
        (
            "<g:item xmlns:g='urn:inner'><g:leaf/></g:item>",
            &["urn:inner|item", "urn:inner|leaf"],
        ),
        (
            "<g:item xmlns:g='urn:inner' xmlns=''><child/></g:item>",
            &["urn:inner|item", "child"],
        ),
        (
            "<g:item xmlns:g='urn:inner'><child xmlns=''><leaf/></child></g:item>",
            &["urn:inner|item", "child", "leaf"],
        ),
        (
            "<n xmlns=''/><!--new--><?local p?><g:item xmlns:g='urn:inner'><g:leaf/></g:item>text&amp;more<n xmlns='urn:local'/>",
            &["n", "urn:inner|item", "urn:inner|leaf", "urn:local|n"],
        ),
    ];
    for parent in &PARENTS {
        let doc = Document::parse(parent.source.as_bytes())?;
        let element = doc
            .xml()
            .descendants()
            .find(|n| {
                n.is_element()
                    && n.tag_name().name() == "box"
                    && n.tag_name().namespace() == Some("urn:outer")
            })
            .ok_or("original box")?;
        for &(xml, child_names) in &accepted {
            let output = doc.apply_edits(&[Edit::AppendChildren { element, xml }])?;
            let expected = format!("{}{}{}", parent.prefix, xml, parent.suffix);
            assert_eq!(output, expected.as_bytes());
            assert_eq!(doc.as_bytes(), parent.source.as_bytes());
            let mut names = parent.ancestors.to_vec();
            names.extend_from_slice(child_names);
            expat_expanded_names(&output, &names)?;
        }
    }
    Ok(())
}

#[test]
fn empty_append_is_exact_and_non_element_content_does_not_require_a_namespace_reset() -> TestResult
{
    for parent in &PARENTS {
        let doc = Document::parse(parent.source.as_bytes())?;
        let element = doc
            .xml()
            .descendants()
            .find(|n| {
                n.is_element()
                    && n.tag_name().name() == "box"
                    && n.tag_name().namespace() == Some("urn:outer")
            })
            .ok_or("original box")?;
        assert_eq!(
            doc.apply_edits(&[Edit::AppendChildren { element, xml: "" }])?,
            parent.source.as_bytes()
        );
        let xml = "text&amp;more<!--new--><?local p?>";
        let output = doc.apply_edits(&[Edit::AppendChildren { element, xml }])?;
        let expected = format!("{}{}{}", parent.prefix, xml, parent.suffix);
        assert_eq!(output, expected.as_bytes());
        assert_eq!(doc.as_bytes(), parent.source.as_bytes());
        expat_expanded_names(&output, parent.ancestors)?;
    }
    Ok(())
}

#[test]
fn destination_without_default_namespace_allows_unnamespaced_descendants() -> TestResult {
    let source = b"<CDXML xmlns:f='urn:outer'><f:box/></CDXML>";
    let doc = Document::parse(source)?;
    let element = doc
        .xml()
        .root_element()
        .first_element_child()
        .ok_or("original box")?;
    let xml = "<n/><g:item xmlns:g='urn:inner'><child/></g:item>";
    let output = doc.apply_edits(&[Edit::AppendChildren { element, xml }])?;
    assert_eq!(output, b"<CDXML xmlns:f='urn:outer'><f:box><n/><g:item xmlns:g='urn:inner'><child/></g:item></f:box></CDXML>");
    assert_eq!(doc.as_bytes(), source);
    expat_expanded_names(
        &output,
        &["CDXML", "urn:outer|box", "n", "urn:inner|item", "child"],
    )
}
