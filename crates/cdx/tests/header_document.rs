use bondline_cdx::{Document, Error, Framing, LengthEncoding, MAX_DEPTH, MAX_RECORDS, RecordKind};
use std::{
    io::Write,
    process::{Command, Stdio},
};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const SOURCE: &[u8] = include_bytes!("fixtures/original-ethane-header-document.cdx");

fn oracle(arguments: &[&str], input: &[u8]) -> TestResult<Vec<u8>> {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child = Command::new(python)
        .arg("-B")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../xtask/oracles/cdx_header_oracle.py"
        ))
        .args(arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child.stdin.take().ok_or("oracle stdin")?.write_all(input)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(output.stdout)
}

fn table(document: &Document<'_>) -> String {
    fn index(value: Option<usize>) -> String {
        value.map_or_else(|| "-".to_owned(), |value| value.to_string())
    }
    let mut lines = Vec::new();
    for record in document.records() {
        let range = record.range();
        let suffix = format!("{} {} {}", index(record.parent()), range.start, range.end);
        lines.push(match record.kind() {
            RecordKind::ObjectStart { tag, id } => format!("start {tag} {id} {suffix}"),
            RecordKind::Property {
                tag,
                length,
                payload,
            } => {
                let length = match length {
                    LengthEncoding::Short => "short",
                    LengthEncoding::Extended => "extended",
                };
                format!(
                    "prop {tag} {length} {} {} {} {suffix}",
                    payload.len(),
                    payload.start,
                    payload.end
                )
            }
            RecordKind::ObjectEnd { start } => format!("end {} {suffix}", index(*start)),
            RecordKind::FileEnd => format!("file_end {suffix}"),
        });
    }
    lines.join("\n") + "\n"
}

#[test]
fn original_header_document_requires_both_object_and_file_ends() -> TestResult {
    oracle(&["self-test"], &[])?;
    assert_eq!(SOURCE.len(), 65_746);
    assert_eq!(SOURCE, oracle(&["fixture"], &[])?.as_slice());
    oracle(&["verify", "1"], SOURCE)?;
    let document = Document::parse(SOURCE)?;
    assert_eq!(document.framing(), Framing::HeaderDocument);
    assert_eq!(document.as_bytes(), SOURCE);
    assert_eq!(document.to_bytes(), SOURCE);
    assert_eq!(document.header(), SOURCE.get(..22).ok_or("header")?);
    assert_eq!(document.records().len(), 30);
    assert_eq!(table(&document).as_bytes(), oracle(&["describe"], SOURCE)?);
    let root = document.records().first().ok_or("root")?;
    assert_eq!(root.kind(), &RecordKind::ObjectStart { tag: 0x8000, id: 0 });
    assert_eq!(root.range(), 22..28);
    let file_end = document.records().last().ok_or("file end")?;
    assert_eq!(file_end.kind(), &RecordKind::FileEnd);
    assert_eq!(file_end.parent(), None);
    assert_eq!(file_end.range(), 65_744..65_746);
    assert_eq!(document.payload(file_end), Err(Error::InvalidProperty));
    Ok(())
}

#[test]
fn header_document_property_edits_preserve_both_ends_and_independent_chemistry() -> TestResult {
    let document = Document::parse(SOURCE)?;
    for record in document
        .records()
        .iter()
        .filter(|record| matches!(record.kind(), RecordKind::Property { .. }))
    {
        assert_eq!(
            document.replace_property(record, document.payload(record)?)?,
            SOURCE
        );
    }
    let order = document
        .records()
        .iter()
        .find(|record| matches!(record.kind(), RecordKind::Property { tag: 0x0600, .. }))
        .ok_or("bond order")?;
    let edited = document.replace_property(order, &2_u16.to_le_bytes())?;
    oracle(&["verify", "2"], &edited)?;
    let parsed = Document::parse(&edited)?;
    assert_eq!(parsed.framing(), Framing::HeaderDocument);
    assert_eq!(table(&parsed).as_bytes(), oracle(&["describe"], &edited)?);
    assert_eq!(
        parsed.replace_property(order, &[0]),
        Err(Error::InvalidProperty)
    );
    assert_eq!(document.as_bytes(), SOURCE);
    Ok(())
}

#[test]
fn opaque_length_edits_keep_reserved_header_and_unrelated_records_exact() -> TestResult {
    let mut source = SOURCE.to_vec();
    source
        .get_mut(12..22)
        .ok_or("reserved")?
        .copy_from_slice(&[0x79, 0x81, 0x12, 0xff, 0x80, 0, 0x7f, 0x55, 0xaa, 3]);
    let document = Document::parse(&source)?;
    assert_eq!(document.framing(), Framing::HeaderDocument);
    assert_eq!(document.header(), source.get(..22).ok_or("header")?);
    let property = document
        .records()
        .get(2)
        .ok_or("extended opaque property")?;
    assert_eq!(property.parent(), Some(0));
    let original = property.range();
    for length in [0, 65_534, 65_535, 65_536] {
        let value = vec![0xa7; length];
        let output = document.replace_property(property, &value)?;
        let parsed = Document::parse(&output)?;
        let changed = parsed.records().get(2).ok_or("changed property")?;
        assert_eq!(parsed.framing(), Framing::HeaderDocument);
        assert_eq!(parsed.records().len(), document.records().len());
        assert_eq!(parsed.payload(changed)?, value);
        assert_eq!(changed.parent(), Some(0));
        assert_eq!(
            source.get(..original.start),
            output.get(..changed.range().start)
        );
        assert_eq!(
            source.get(original.end..),
            output.get(changed.range().end..)
        );
        assert_eq!(table(&parsed).as_bytes(), oracle(&["describe"], &output)?);
    }
    assert_eq!(document.to_bytes(), source);
    Ok(())
}

#[test]
fn successful_legacy_reserved_header_interpretations_win_the_ambiguity() -> TestResult {
    // The native root's six bytes are opaque legacy header bytes if the legacy tree is complete.
    let legacy = SOURCE.get(..SOURCE.len() - 2).ok_or("without file end")?;
    let document = Document::parse(legacy)?;
    assert_eq!(document.framing(), Framing::ImplicitDocument);
    assert_eq!(document.header(), legacy.get(..28).ok_or("legacy header")?);
    assert_eq!(document.to_bytes(), legacy);

    let old = include_bytes!("fixtures/original-ethane-explicit.cdx");
    for id in [0_u32, 0x12fea73b] {
        let mut coincident = old.to_vec();
        coincident
            .get_mut(22..24)
            .ok_or("legacy reserved tag")?
            .copy_from_slice(&0x8000_u16.to_le_bytes());
        coincident
            .get_mut(24..28)
            .ok_or("legacy reserved ID")?
            .copy_from_slice(&id.to_le_bytes());
        let document = Document::parse(&coincident)?;
        assert_eq!(document.framing(), Framing::ExplicitDocument);
        assert_eq!(document.header(), coincident.get(..28).ok_or("header")?);
        assert_eq!(document.to_bytes(), coincident);
    }
    Ok(())
}

#[test]
fn exact_eof_remains_required_and_truncations_never_become_partial_native_documents() -> TestResult
{
    for length in 0..SOURCE.len() {
        let source = SOURCE.get(..length).ok_or("prefix")?;
        // One genuinely complete legacy tree is inherently indistinguishable from a missing native file end.
        if length == SOURCE.len() - 2 {
            assert_eq!(
                Document::parse(source)?.framing(),
                Framing::ImplicitDocument
            );
        } else {
            assert!(Document::parse(source).is_err(), "accepted prefix{length}");
        }
    }
    for tail in [
        &[0][..],
        &[0, 0][..],
        &[0, 0, 0, 0][..],
        &[1, 0][..],
        &[0x80][..],
    ] {
        let mut trailing = SOURCE.to_vec();
        trailing.extend_from_slice(tail);
        assert!(matches!(
            Document::parse(&trailing),
            Err(Error::TrailingData)
        ));
    }
    let mut nonzero_file_tag = SOURCE.to_vec();
    *nonzero_file_tag
        .get_mut(SOURCE.len() - 2)
        .ok_or("file end")? = 1;
    assert!(matches!(
        Document::parse(&nonzero_file_tag),
        Err(Error::TrailingData)
    ));

    // A root marker anywhere else, or an unsupported magic, cannot enable the new profile.
    let mut absent_marker = SOURCE.to_vec();
    *absent_marker.get_mut(23).ok_or("root tag")? = 0x81;
    assert!(matches!(
        Document::parse(&absent_marker),
        Err(Error::TrailingData)
    ));
    let mut invalid_magic = SOURCE.to_vec();
    *invalid_magic.get_mut(0).ok_or("magic")? = b'X';
    assert!(matches!(
        Document::parse(&invalid_magic),
        Err(Error::InvalidHeader)
    ));
    Ok(())
}

fn header_root() -> TestResult<Vec<u8>> {
    Ok(SOURCE.get(..28).ok_or("prefix and root")?.to_vec())
}

#[test]
fn native_record_budget_includes_root_object_and_separate_file_end() -> TestResult {
    for count in [MAX_RECORDS - 3, MAX_RECORDS - 2] {
        let mut source = header_root()?;
        for _ in 0..count {
            source.extend_from_slice(&[1, 0x40, 0, 0]);
        }
        source.extend_from_slice(&[0, 0, 0, 0]);
        if count == MAX_RECORDS - 3 {
            let document = Document::parse(&source)?;
            assert_eq!(document.framing(), Framing::HeaderDocument);
            assert_eq!(document.records().len(), MAX_RECORDS);
            assert_eq!(
                document.records().last().ok_or("end")?.kind(),
                &RecordKind::FileEnd
            );
        } else {
            assert!(matches!(Document::parse(&source), Err(Error::RecordLimit)));
        }
    }
    Ok(())
}

#[test]
fn native_depth_budget_counts_the_encoded_header_root() -> TestResult {
    for depth in [MAX_DEPTH, MAX_DEPTH + 1] {
        let mut source = header_root()?;
        for _ in 1..depth {
            source.extend_from_slice(&[1, 0xc0, 0, 0, 0, 0]);
        }
        source.extend_from_slice(&vec![0; (depth + 1) * 2]);
        if depth == MAX_DEPTH {
            let document = Document::parse(&source)?;
            assert_eq!(document.framing(), Framing::HeaderDocument);
            assert_eq!(document.records().len(), depth * 2 + 1);
        } else {
            assert!(matches!(Document::parse(&source), Err(Error::DepthLimit)));
        }
    }
    Ok(())
}
