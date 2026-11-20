//! Independent XML lexical facts needed before a projection may grant edit authority.
use bondline_cdxml::{Document, Edit, Error, SyntaxFeatures};
use std::{
    io::Write,
    process::{Command, Stdio},
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn check(source: &[u8], expected: [bool; 4]) -> TestResult {
    let document = Document::parse(source)?;
    assert_eq!(
        document.features(),
        SyntaxFeatures {
            doctype: expected[0],
            processing_instructions: expected[1],
            cdata: expected[2],
            namespace_declarations: expected[3],
        }
    );
    assert_eq!(document.as_bytes(), source);
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let expected: String = expected
        .into_iter()
        .map(|value| if value { '1' } else { '0' })
        .collect();
    let mut child = Command::new(python)
        .args([
            "-X",
            "utf8",
            "-B",
            "-c",
            r#"
import sys,xml.parsers.expat as expat
flags=[False]*4
def flag(index):
    def observed(*args): flags[index]=True
    return observed
p=expat.ParserCreate(namespace_separator='|')
p.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
p.StartDoctypeDeclHandler=flag(0)
p.ProcessingInstructionHandler=flag(1)
p.StartCdataSectionHandler=flag(2)
p.StartNamespaceDeclHandler=flag(3)
p.Parse(sys.stdin.buffer.read(),True)
assert ''.join('1' if value else '0' for value in flags)==sys.argv[1],flags
"#,
        ])
        .arg(expected)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child.stdin.take().ok_or("Expat stdin")?.write_all(source)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

#[test]
fn declarations_comments_and_quoted_syntax_do_not_fabricate_extended_features() -> TestResult {
    for source in [
        "\u{feff}<?xml version='1.0' encoding='UTF-8' standalone='yes'?>\r\n<CDXML/>",
        "<!-- <!DOCTYPE CDXML [x]> <?probe x?> <![CDATA[x]]> xmlns:p='urn:p' --><CDXML note=\"xmlns:p='urn:p' &lt;?probe?&gt;\"/>",
    ] {
        check(source.as_bytes(), [false; 4])?;
    }
    Ok(())
}

#[test]
fn actual_doctype_pi_cdata_and_all_namespace_declarations_have_independent_expat_evidence()
-> TestResult {
    for (source, expected) in [
        (
            "<!DOCTYPE CDXML SYSTEM 'https://example.invalid/never-fetch.dtd'><CDXML/>",
            [true, false, false, false],
        ),
        ("<?probe keep?><CDXML/>", [false, true, false, false]),
        ("<CDXML><![CDATA[ ]]></CDXML>", [false, false, true, false]),
        ("<CDXML xmlns:p='urn:p'/>", [false, false, false, true]),
        ("<CDXML><x xmlns=''/></CDXML>", [false, false, false, true]),
        (
            "<CDXML xmlns:xml='http://www.w3.org/XML/1998/namespace'/>",
            [false, false, false, true],
        ),
        (
            "<CDXML><x xmlns:p='urn:p'/><x xmlns=''/></CDXML>",
            [false, false, false, true],
        ),
        (
            "<!DOCTYPE CDXML SYSTEM 'https://example.invalid/never-fetch.dtd'><CDXML xmlns:p='urn:p'><?probe keep?><![CDATA[x]]><p:x/></CDXML>",
            [true; 4],
        ),
    ] {
        check(source.as_bytes(), expected)?;
    }
    Ok(())
}

#[test]
fn features_describe_their_own_immutable_bytes_after_a_source_edit() -> TestResult {
    let source = b"<CDXML/>";
    let document = Document::parse(source)?;
    let changed = document.apply_edits(&[Edit::AppendChildren {
        element: document.xml().root_element(),
        xml: "<![CDATA[x]]><?probe keep?><p:x xmlns:p='urn:p'/>",
    }])?;
    check(&changed, [false, true, true, true])?;
    assert_eq!(document.features(), SyntaxFeatures::default());
    assert_eq!(document.as_bytes(), source);
    Ok(())
}

#[test]
fn collecting_facts_does_not_admit_unsupported_xml() {
    assert!(matches!(
        Document::parse(b"<CDXML xmlns=''/>"),
        Err(Error::InvalidRoot)
    ));
    assert!(matches!(
        Document::parse(b"<!DOCTYPE CDXML [<!ENTITY x 'x'>]><CDXML/>"),
        Err(Error::UnsupportedDtd)
    ));
    assert!(matches!(
        Document::parse(b"<?xml version='1.1'?><CDXML/>"),
        Err(Error::UnsupportedDeclaration)
    ));
    assert!(matches!(
        Document::parse(b"<CDXML xmlns='urn:foreign'/>"),
        Err(Error::InvalidRoot)
    ));
    assert!(matches!(Document::parse(b"\xff"), Err(Error::Utf8(_))));
}

#[test]
fn bare_cdxml_doctype_is_distinct_from_a_system_or_different_declaration() -> TestResult {
    for (source, bare) in [
        ("<!DOCTYPE CDXML><CDXML/>", true),
        ("<!DOCTYPE CDXML \n><CDXML/>", true),
        ("<CDXML/>", false),
        ("<!DOCTYPE CDXML SYSTEM 'unused.dtd'><CDXML/>", false),
        (
            "<!-- <!DOCTYPE CDXML> --><!DOCTYPE CDXML PUBLIC '-//Example' 'unused.dtd'><CDXML/>",
            false,
        ),
        ("<!DOCTYPE Foreign><CDXML/>", false),
    ] {
        let doc = Document::parse(source.as_bytes())?;
        assert_eq!(doc.has_bare_cdxml_doctype(), bare, "{source}");
        assert_eq!(doc.to_bytes(), source.as_bytes());
    }
    Ok(())
}
