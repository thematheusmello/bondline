use std::{
    io::Write,
    process::{Command, Stdio},
};

use bondline_cdx::Document;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const EXTERNAL_ETHANE: &[u8] = include_bytes!("fixtures/external-ethane-indigo-1.47.0.cdx");

fn independent_ethane(input: &[u8]) -> TestResult<Vec<u8>> {
    let python = std::env::var_os("BONDLINE_PYTHON").ok_or("pinned BONDLINE_PYTHON is required")?;
    let mut child = Command::new(python)
        .arg("-B")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/external_ethane_oracle.py"
        ))
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

#[test]
fn exact_external_positioned_ethane_keeps_both_final_words_and_round_trips() -> TestResult {
    assert_eq!(EXTERNAL_ETHANE.len(), 258);
    assert_eq!(EXTERNAL_ETHANE.get(254..), Some([0, 0, 0, 0].as_slice()));
    let document = Document::parse(EXTERNAL_ETHANE)?;
    assert_eq!(document.as_bytes(), EXTERNAL_ETHANE);
    assert_eq!(document.to_bytes(), EXTERNAL_ETHANE);
    assert_eq!(
        independent_ethane(&document.to_bytes())?,
        b"{\"atoms\":2,\"bonds\":1,\"formula\":\"C2H6\",\"smiles\":\"CC\",\"hydrogens\":[3,3],\"byte_length\":258,\"record_count\":24,\"header_bytes\":28,\"document_end\":[254,256],\"file_end\":[256,258],\"raw_y_x16_16\":[[0,0],[0,2949120]]}\n"
    );
    Ok(())
}

fn described_records(input: &[u8]) -> TestResult<Vec<u8>> {
    let python = std::env::var_os("BONDLINE_PYTHON").ok_or("pinned BONDLINE_PYTHON")?;
    let mut child = Command::new(python)
        .arg("-B")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/external_ethane_oracle.py"
        ))
        .arg("describe")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child.stdin.take().ok_or("oracle stdin")?.write_all(input)?;
    let result = child.wait_with_output()?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(result.stdout)
}

fn table(document: &Document<'_>) -> TestResult<Vec<u8>> {
    use bondline_cdx::{LengthEncoding, RecordKind};
    let index = |n: Option<usize>| n.map_or_else(|| "-".to_owned(), |v| v.to_string());
    let mut lines = Vec::new();
    for record in document.records() {
        let range = record.range();
        let suffix = format!("{} {} {}", index(record.parent()), range.start, range.end);
        lines.push(match record.kind() {
            RecordKind::ObjectStart { tag, id } => format!("start {tag} {id} {suffix}"),
            RecordKind::ObjectEnd { start } => format!("end {} {suffix}", index(*start)),
            RecordKind::FileEnd => format!("file_end {suffix}"),
            RecordKind::Property {
                tag,
                length,
                payload,
            } => {
                let encoding = match length {
                    LengthEncoding::Short => "short",
                    LengthEncoding::Extended => "extended",
                };
                let hex = document
                    .payload(record)?
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>();
                format!(
                    "prop {tag} {encoding} {} {} {} {suffix} {hex}",
                    payload.len(),
                    payload.start,
                    payload.end
                )
            }
        });
    }
    Ok((lines.join("\n") + "\n").into_bytes())
}

fn explicit(source: &[u8]) -> TestResult<Vec<u8>> {
    let mut result = source.get(..28).ok_or("header")?.to_vec();
    result.extend_from_slice(&[0, 0x80, 0, 0, 0, 0]);
    result.extend_from_slice(source.get(28..).ok_or("body")?);
    Ok(result)
}

#[test]
fn implicit_external_table_retains_document_properties_and_distinct_file_end() -> TestResult {
    use bondline_cdx::{Error, Framing, LengthEncoding, RecordKind};
    let document = Document::parse(EXTERNAL_ETHANE)?;
    assert_eq!(document.framing(), Framing::ImplicitDocumentFileEnd);
    assert_eq!(
        document.header(),
        EXTERNAL_ETHANE.get(..28).ok_or("header")?
    );
    assert_eq!(document.records().len(), 24);
    let first = document.records().first().ok_or("document property")?;
    assert_eq!(first.parent(), None);
    assert_eq!(first.range(), 28..36);
    assert_eq!(
        first.kind(),
        &RecordKind::Property {
            tag: 0x0805,
            length: LengthEncoding::Short,
            payload: 32..36
        }
    );
    assert_eq!(document.payload(first)?, [0, 0, 30, 0]);
    let root_end = document.records().get(22).ok_or("document end")?;
    assert_eq!(root_end.kind(), &RecordKind::ObjectEnd { start: None });
    assert_eq!(root_end.range(), 254..256);
    let file_end = document.records().get(23).ok_or("file end")?;
    assert_eq!(file_end.kind(), &RecordKind::FileEnd);
    assert_eq!(file_end.parent(), None);
    assert_eq!(file_end.range(), 256..258);
    assert_eq!(document.payload(file_end), Err(Error::InvalidProperty));
    assert_eq!(table(&document)?, described_records(EXTERNAL_ETHANE)?);
    Ok(())
}

#[test]
fn encoded_root_has_its_own_parent_and_end_before_file_end() -> TestResult {
    use bondline_cdx::{Framing, RecordKind};
    let source = explicit(EXTERNAL_ETHANE)?;
    assert_eq!(source.len(), 264);
    let document = Document::parse(&source)?;
    assert_eq!(document.framing(), Framing::ExplicitDocumentFileEnd);
    assert_eq!(document.header().len(), 28);
    assert_eq!(document.records().len(), 25);
    let root = document.records().first().ok_or("root")?;
    assert_eq!(root.kind(), &RecordKind::ObjectStart { tag: 0x8000, id: 0 });
    assert_eq!(root.range(), 28..34);
    assert_eq!(root.parent(), None);
    let first = document.records().get(1).ok_or("property")?;
    assert_eq!(first.parent(), Some(0));
    assert_eq!(first.range(), 34..42);
    let root_end = document.records().get(23).ok_or("root end")?;
    assert_eq!(root_end.kind(), &RecordKind::ObjectEnd { start: Some(0) });
    assert_eq!(root_end.range(), 260..262);
    assert_eq!(root_end.parent(), None);
    let file_end = document.records().last().ok_or("file end")?;
    assert_eq!(file_end.kind(), &RecordKind::FileEnd);
    assert_eq!(file_end.range(), 262..264);
    assert_eq!(file_end.parent(), None);
    assert_eq!(document.to_bytes(), source);
    assert_eq!(table(&document)?, described_records(&source)?);
    Ok(())
}

#[test]
fn canonical_unknown_body_is_supported_without_changing_legacy_successes_or_negatives() -> TestResult
{
    use bondline_cdx::{Error, Framing};
    let old = include_bytes!("fixtures/unknown-implicit.cdx");
    assert_eq!(old.len(), 70);
    assert_eq!(
        old.get(12..28),
        Some((0_u8..16).collect::<Vec<_>>().as_slice())
    );
    let mut canonical = old.to_vec();
    canonical.get_mut(12..28).ok_or("reserved")?.fill(0);
    for (source, framing) in [
        (old.to_vec(), Framing::ImplicitDocument),
        (canonical.clone(), Framing::ImplicitDocument),
        (explicit(old)?, Framing::ExplicitDocument),
        (explicit(&canonical)?, Framing::ExplicitDocument),
    ] {
        let doc = Document::parse(&source)?;
        assert_eq!(doc.framing(), framing);
        assert_eq!(doc.to_bytes(), source);
    }
    for mut source in [old.to_vec(), explicit(old)?] {
        source.extend_from_slice(&[0, 0]);
        assert!(matches!(Document::parse(&source), Err(Error::TrailingData)));
    }
    for (mut source, framing) in [
        (canonical.clone(), Framing::ImplicitDocumentFileEnd),
        (explicit(&canonical)?, Framing::ExplicitDocumentFileEnd),
    ] {
        source.extend_from_slice(&[0, 0]);
        let doc = Document::parse(&source)?;
        assert_eq!(doc.framing(), framing);
        assert_eq!(doc.to_bytes(), source);
        assert_eq!(table(&doc)?, described_records(&source)?);
    }
    let embedded = include_bytes!("fixtures/original-ethane-header-document.cdx");
    assert_eq!(
        Document::parse(embedded)?.framing(),
        Framing::HeaderDocument
    );
    Ok(())
}

#[test]
fn exact_eof_and_zero_header_gate_keep_truncation_and_padding_refusals() -> TestResult {
    use bondline_cdx::{Error, Framing};
    let legacy = EXTERNAL_ETHANE.get(..256).ok_or("legacy prefix")?;
    let doc = Document::parse(legacy)?;
    assert_eq!(doc.framing(), Framing::ImplicitDocument);
    assert_eq!(doc.records().len(), 23);
    assert_eq!(doc.to_bytes(), legacy);
    let encoded = explicit(EXTERNAL_ETHANE)?;
    assert_eq!(
        Document::parse(encoded.get(..262).ok_or("encoded legacy")?)?.framing(),
        Framing::ExplicitDocument
    );
    for length in [28, 254, 255, 257] {
        assert!(
            matches!(
                Document::parse(EXTERNAL_ETHANE.get(..length).ok_or("prefix")?),
                Err(Error::Truncated { .. })
            ),
            "length{length}"
        );
    }
    for tail in [&[0_u8][..], &[0, 0], &[1, 0], &[0, 0, 0, 0]] {
        let mut source = EXTERNAL_ETHANE.to_vec();
        source.extend_from_slice(tail);
        assert!(matches!(Document::parse(&source), Err(Error::TrailingData)));
    }
    for index in 12..28 {
        let mut source = EXTERNAL_ETHANE.to_vec();
        *source.get_mut(index).ok_or("reserved")? = 1;
        assert!(
            matches!(Document::parse(&source), Err(Error::TrailingData)),
            "nonzero reserved byte{index}"
        );
    }
    let mut bad_end = EXTERNAL_ETHANE.to_vec();
    bad_end
        .get_mut(256..258)
        .ok_or("file end")?
        .copy_from_slice(&[1, 0]);
    assert!(matches!(
        Document::parse(&bad_end),
        Err(Error::TrailingData)
    ));
    let mut embedded = EXTERNAL_ETHANE.to_vec();
    embedded
        .get_mut(22..28)
        .ok_or("embedded root")?
        .copy_from_slice(&[0, 0x80, 0, 0, 0, 0]);
    assert_eq!(
        Document::parse(&embedded)?.framing(),
        Framing::HeaderDocument
    );
    assert_eq!(Document::parse(&embedded)?.header().len(), 22);
    Ok(())
}

#[test]
fn checked_opaque_edits_reindex_both_final_words_and_retain_source_ownership() -> TestResult {
    use bondline_cdx::{Error, Framing, RecordKind};
    for source in [EXTERNAL_ETHANE.to_vec(), explicit(EXTERNAL_ETHANE)?] {
        let document = Document::parse(&source)?;
        let property = document
            .records()
            .iter()
            .find(|r| matches!(r.kind(), RecordKind::Property { tag: 0x0805, .. }))
            .ok_or("document property")?;
        assert_eq!(
            document.replace_property(property, document.payload(property)?)?,
            source
        );
        let old_range = property.range();
        for length in [0, 3, 65_534, 65_535] {
            let value = vec![0xa7; length];
            let mut expected = source.get(..old_range.start + 2).ok_or("prefix")?.to_vec();
            if length < 65_535 {
                expected.extend_from_slice(&u16::try_from(length)?.to_le_bytes());
            } else {
                expected.extend_from_slice(&[0xff, 0xff]);
                expected.extend_from_slice(&u32::try_from(length)?.to_le_bytes());
            }
            expected.extend_from_slice(&value);
            expected.extend_from_slice(source.get(old_range.end..).ok_or("suffix")?);
            let edited = document.replace_property(property, &value)?;
            assert_eq!(edited, expected);
            assert_eq!(document.as_bytes(), source);
            let parsed = Document::parse(&edited)?;
            assert!(matches!(
                parsed.framing(),
                Framing::ImplicitDocumentFileEnd | Framing::ExplicitDocumentFileEnd
            ));
            assert_eq!(
                parsed.records().last().ok_or("file end")?.kind(),
                &RecordKind::FileEnd
            );
            assert_eq!(
                parsed.records().last().ok_or("file end")?.range(),
                edited.len() - 2..edited.len()
            );
            assert_eq!(
                edited.get(edited.len() - 4..),
                Some([0, 0, 0, 0].as_slice())
            );
            assert_eq!(table(&parsed)?, described_records(&edited)?);
            assert_eq!(
                parsed.replace_property(property, &[]),
                Err(Error::InvalidProperty)
            );
        }
    }
    Ok(())
}

#[test]
fn distinct_file_end_consumes_the_record_budget_for_both_root_profiles() -> TestResult {
    use bondline_cdx::{Error, MAX_RECORDS};
    for encoded in [false, true] {
        let mut source = EXTERNAL_ETHANE.get(..28).ok_or("header")?.to_vec();
        if encoded {
            source.extend_from_slice(&[0, 0x80, 0, 0, 0, 0]);
        }
        let properties = MAX_RECORDS - if encoded { 3 } else { 2 };
        for _ in 0..properties {
            source.extend_from_slice(&[1, 0x40, 0, 0]);
        }
        source.extend_from_slice(&[0, 0, 0, 0]);
        assert_eq!(Document::parse(&source)?.records().len(), MAX_RECORDS);
        let at = if encoded { 34 } else { 28 };
        source.splice(at..at, [1, 0x40, 0, 0]);
        assert!(matches!(Document::parse(&source), Err(Error::RecordLimit)));
    }
    Ok(())
}

#[test]
fn encoded_depth_and_whole_input_bound_include_the_new_envelope_exactly() -> TestResult {
    use bondline_cdx::{Error, MAX_DEPTH, MAX_INPUT_BYTES};
    for encoded in [false, true] {
        for depth in [MAX_DEPTH, MAX_DEPTH + 1] {
            let mut source = EXTERNAL_ETHANE.get(..28).ok_or("header")?.to_vec();
            if encoded {
                source.extend_from_slice(&[0, 0x80, 0, 0, 0, 0]);
            }
            for _ in usize::from(encoded)..depth {
                source.extend_from_slice(&[1, 0xc0, 0, 0, 0, 0]);
            }
            source.extend_from_slice(&vec![0; (depth + usize::from(!encoded) + 1) * 2]);
            if depth == MAX_DEPTH {
                assert!(Document::parse(&source).is_ok());
            } else {
                assert!(matches!(Document::parse(&source), Err(Error::DepthLimit)));
            }
        }
    }
    let length = MAX_INPUT_BYTES - 40;
    let mut source = Vec::with_capacity(MAX_INPUT_BYTES + 1);
    source.extend_from_slice(EXTERNAL_ETHANE.get(..28).ok_or("header")?);
    source.extend_from_slice(&[1, 0x40, 0xff, 0xff]);
    source.extend_from_slice(&u32::try_from(length)?.to_le_bytes());
    source.resize(MAX_INPUT_BYTES - 4, 0xa7);
    source.extend_from_slice(&[0, 0, 0, 0]);
    let document = Document::parse(&source)?;
    assert_eq!(document.as_bytes().len(), MAX_INPUT_BYTES);
    assert_eq!(
        document
            .payload(document.records().first().ok_or("property")?)?
            .len(),
        length
    );
    drop(document);
    source.push(0);
    assert!(matches!(
        Document::parse(&source),
        Err(Error::InputTooLarge)
    ));
    Ok(())
}
