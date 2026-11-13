use std::{
    io::Write,
    process::{Command, Stdio},
};

use bondline_cdx::{Document, LengthEncoding, RecordKind};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn oracle(arguments: &[&str], input: &[u8]) -> TestResult<Vec<u8>> {
    oracle_with_text_newlines(arguments, input, false)
}

fn oracle_with_text_newlines(
    arguments: &[&str],
    input: &[u8],
    translate_to_crlf: bool,
) -> TestResult<Vec<u8>> {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let script = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../xtask/oracles/cdx_oracle.py"
    );
    let mut command = Command::new(python);
    command.arg("-B");
    if translate_to_crlf {
        command
            .args([
                "-c",
                r#"import io,runpy,sys
sys.stdout=io.TextIOWrapper(sys.stdout.buffer,encoding="utf-8",newline="\r\n")
runpy.run_path(sys.argv.pop(1),run_name="__main__")
"#,
            ])
            .arg(script);
    } else {
        command.arg(script);
    }
    let mut child = command
        .args(arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("independent oracle stdin")?
        .write_all(input)?;
    let result = child.wait_with_output()?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(result.stdout)
}

#[test]
fn record_protocol_stays_lf_when_python_text_stdout_translates_to_crlf() -> TestResult {
    let source = oracle(&["fixture"], &[])?;
    assert_eq!(
        oracle_with_text_newlines(&["fixture"], &[], true)?,
        source,
        "binary fixture output must remain byte-exact"
    );
    let document = Document::parse(&source)?;
    assert_eq!(
        oracle_with_text_newlines(&["describe"], &source, true)?,
        record_table(&document).as_bytes(),
        "record table is an exact LF protocol, independent of text newline translation"
    );
    Ok(())
}

fn index_text(index: Option<usize>) -> String {
    index.map_or_else(|| "-".to_owned(), |value| value.to_string())
}

fn record_table(document: &Document<'_>) -> String {
    let mut lines = Vec::new();
    for record in document.records() {
        let parent = index_text(record.parent());
        let range = record.range();
        let suffix = format!("{parent} {} {}", range.start, range.end);
        lines.push(match record.kind() {
            RecordKind::ObjectStart { tag, id } => format!("start {tag} {id} {suffix}"),
            RecordKind::Property {
                tag,
                length,
                payload,
            } => {
                let encoding = match length {
                    LengthEncoding::Short => "short",
                    LengthEncoding::Extended => "extended",
                };
                format!(
                    "prop {tag} {encoding} {} {} {} {suffix}",
                    payload.len(),
                    payload.start,
                    payload.end
                )
            }
            RecordKind::ObjectEnd { start } => format!("end {} {suffix}", index_text(*start)),
        });
    }
    lines.join("\n") + "\n"
}

#[test]
fn original_fixture_and_bond_edit_are_read_by_external_rdkit() -> TestResult {
    oracle(&["self-test"], &[])?;
    for framing in ["implicit", "explicit"] {
        check_fixture_and_edit(framing)?;
    }
    Ok(())
}

fn check_fixture_and_edit(framing: &str) -> TestResult {
    let original = oracle(&["fixture", framing], &[])?;
    let document = Document::parse(&original)?;
    assert_eq!(document.as_bytes(), original);
    assert_eq!(document.to_bytes(), original);
    assert_eq!(
        record_table(&document).as_bytes(),
        oracle(&["describe"], &original)?
    );
    oracle(&["verify", "1", framing], &document.to_bytes())?;

    let order = document
        .records()
        .iter()
        .find(|record| matches!(record.kind(), RecordKind::Property { tag: 0x0600, .. }))
        .ok_or("bond order property")?;
    let ethene = document.replace_property(order, &2_u16.to_le_bytes())?;
    oracle(&["verify", "2", framing], &ethene)?;
    assert_eq!(
        document.as_bytes(),
        original,
        "property edits must not mutate the borrowed source"
    );
    let edited = Document::parse(&ethene)?;
    assert_eq!(
        record_table(&edited).as_bytes(),
        oracle(&["describe"], &ethene)?
    );
    Ok(())
}

#[test]
fn opaque_property_length_changes_preserve_every_other_byte_and_owner() -> TestResult {
    let original = oracle(&["fixture"], &[])?;
    let document = Document::parse(&original)?;
    let target = document
        .records()
        .iter()
        .find(|record| matches!(record.kind(), RecordKind::Property { tag: 0x4002, .. }))
        .ok_or("opaque extended property")?;
    assert_eq!(
        document.replace_property(target, document.payload(target)?)?,
        original
    );
    for length in [0, 1, 32767, 32768, 65534, 65535, 65536] {
        let edited = document.replace_property(target, &vec![0xa7; length])?;
        oracle(&["verify-opaque-edit", &length.to_string()], &edited)?;
        let parsed = Document::parse(&edited)?;
        assert_eq!(
            record_table(&parsed).as_bytes(),
            oracle(&["describe"], &edited)?
        );
    }

    let foreign_document = Document::parse(&original)?;
    let foreign_property = foreign_document
        .records()
        .iter()
        .find(|record| matches!(record.kind(), RecordKind::Property { tag: 0x4002, .. }))
        .ok_or("foreign opaque property")?;
    assert!(document.payload(foreign_property).is_err());
    assert!(
        document
            .replace_property(foreign_property, b"wrong owner")
            .is_err()
    );
    assert_eq!(document.as_bytes(), original);
    Ok(())
}
