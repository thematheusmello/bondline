use bondline_io::cdx::{ElementalGraph, MappingError};
use bondline_io::{
    DocumentError, StoredDocument,
    mol::{MolTransform, Version},
};
use bondline_model::{BondOrder, PageId, PageSize, Point2};
use std::{
    io::Write,
    mem::size_of,
    process::{Command, Stdio},
    sync::Arc,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;
const EXPLICIT: &[u8] = include_bytes!("fixtures/cdx/metadata-explicit.cdx");
const HEADER: &[u8] = include_bytes!("fixtures/cdx/metadata-header.cdx");

#[test]
fn original_metadata_is_read_only_and_keeps_every_byte_in_both_framings() -> TestResult {
    for input in [EXPLICIT, HEADER] {
        // Syntax must already be valid; the intended RED is unsupported metadata.
        let syntax = bondline_cdx::Document::parse(input)?;
        assert_eq!(syntax.to_bytes(), input);
        let graph = ElementalGraph::parse(input)?;
        let fragment = graph.fragments().first().ok_or("fragment")?;
        assert_eq!((fragment.atoms().len(), fragment.bonds().len()), (2, 1));
        let first = fragment.atoms().first().ok_or("first")?;
        let second = fragment.atoms().get(1).ok_or("second")?;
        assert_eq!((first.id.get(), second.id.get()), (1, 2));
        assert_eq!(first.position, Some(Point2::new(10.25, -3.5)?));
        assert_eq!(second.position, Some(Point2::new(26.25, 8.5)?));
        assert!(
            fragment
                .atoms()
                .iter()
                .all(|a| a.element.get() == 6 && a.explicit_hydrogens.is_none())
        );
        let bond = fragment.bonds().first().ok_or("bond")?;
        assert_eq!(
            (bond.begin.get(), bond.end.get(), bond.order),
            (1, 2, BondOrder::Single)
        );
        assert_eq!(graph.as_bytes(), input);
        assert_eq!(graph.to_bytes(), input);
        assert!(!format!("{graph:?}").contains("Original metadata advisory"));
        let preview = PageSize::new(72.0, 144.0)?;
        let stored = StoredDocument::from_cdx(Arc::from(input), preview)?;
        assert_eq!(
            stored.model().pages().first().ok_or("page")?.size(),
            preview
        );
        assert!(!stored.is_fully_mapped());
        assert_eq!(stored.to_cdx()?.as_ref(), input);
        assert!(!format!("{stored:?}").contains("Original metadata advisory"));
    }
    Ok(())
}

fn field_range(
    source: &[u8],
    owner: u32,
    tag: u16,
) -> Result<std::ops::Range<usize>, Box<dyn std::error::Error>> {
    let document = bondline_cdx::Document::parse(source)?;
    document.records().iter().find(|record| {
        matches!(record.kind(), bondline_cdx::RecordKind::Property { tag: actual, .. } if *actual == tag)
        && record.parent().and_then(|parent| document.records().get(parent)).is_some_and(|parent| {
            matches!(parent.kind(), bondline_cdx::RecordKind::ObjectStart { id, .. } if *id == owner)
        })
    }).map(|record| record.range()).ok_or_else(|| "original field".into())
}

// Literal test assembly, independent of Bondline's CDX writer/replacement APIs.
fn changed(
    source: &[u8],
    owner: u32,
    old: u16,
    tag: u16,
    payload: &[u8],
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let range = field_range(source, owner, old)?;
    let mut field = tag.to_le_bytes().to_vec();
    field.extend_from_slice(&u16::try_from(payload.len())?.to_le_bytes());
    field.extend_from_slice(payload);
    let mut bytes = source.to_vec();
    bytes.splice(range, field);
    Ok(bytes)
}

fn without(source: &[u8], owner: u32, tag: u16) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let range = field_range(source, owner, tag)?;
    let mut bytes = source.to_vec();
    bytes.drain(range);
    Ok(bytes)
}

#[test]
fn independent_struct_and_rdkit_consume_both_original_metadata_vectors() -> TestResult {
    for (input, framing, hash) in [
        (
            EXPLICIT,
            "explicit",
            "148133b6c2bf5a4bde92c9a8adff636e3f2ef9bf19238b9a1f99003e9e5d7c05",
        ),
        (
            HEADER,
            "header",
            "9af210eb978d085b143fad97074d55aa6512c44e02370d4a41e0376d06176a39",
        ),
    ] {
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
                "/tests/cdx_metadata_oracle.py"
            ))
            .args(["verify", framing])
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
        let expected: serde_json::Value = serde_json::from_slice(&output.stdout)?;
        assert_eq!(expected["sha256"], hash);
        assert_eq!(expected["bytes"], input.len());
        assert_eq!(expected["external_smiles"], "CC");
        assert_eq!(
            (expected["atoms"].as_u64(), expected["bonds"].as_u64()),
            (Some(2), Some(1))
        );
        assert_eq!(ElementalGraph::parse(input)?.to_bytes(), input);
    }
    Ok(())
}

#[test]
fn unknown_and_symmetric_cip_states_remain_distinct_source_without_stereo_perception() -> TestResult
{
    let expected = ElementalGraph::parse(EXPLICIT)?;
    for (owner, tag) in [(101, 0x0437), (102, 0x0437), (103, 0x060a)] {
        let missing = without(EXPLICIT, owner, tag)?;
        for value in [0, 1] {
            let input = changed(EXPLICIT, owner, tag, tag, &[value])?;
            let graph = ElementalGraph::parse(&input)?;
            assert_eq!(graph.fragments(), expected.fragments());
            assert_eq!(graph.to_bytes(), input);
            assert_ne!(input, missing);
        }
        assert_eq!(
            ElementalGraph::parse(&missing)?.fragments(),
            expected.fragments()
        );
        for value in [2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 127, 128, 255] {
            let input = changed(EXPLICIT, owner, tag, tag, &[value])?;
            assert!(
                matches!(ElementalGraph::parse(&input), Err(MappingError::UnsupportedProperty { tag: actual, .. }) if actual == tag)
            );
        }
    }
    Ok(())
}

#[test]
fn metadata_fixed_widths_and_duplicate_fields_are_checked() -> TestResult {
    for (owner, tag, width) in [
        (100, 0x000a, 2),
        (101, 0x000a, 2),
        (103, 0x000a, 2),
        (100, 0x0204, 16),
        (101, 0x044d, 4),
        (101, 0x0437, 1),
        (103, 0x060a, 1),
    ] {
        for length in [0, width - 1, width + 1] {
            let input = changed(EXPLICIT, owner, tag, tag, &vec![0; length])?;
            let error = ElementalGraph::parse(&input)
                .err()
                .ok_or("invalid width accepted")?;
            assert!(
                matches!(error, MappingError::InvalidProperty { tag: actual, .. } if actual == tag)
            );
            assert_eq!(error.code(), "invalid_document");
        }
    }
    for (owner, tag) in [
        (100, 0x000a),
        (101, 0x000a),
        (103, 0x000a),
        (100, 0x0204),
        (101, 0x044d),
        (101, 0x0437),
        (103, 0x060a),
        (103, 0x0010),
    ] {
        let range = field_range(EXPLICIT, owner, tag)?;
        let mut input = EXPLICIT.to_vec();
        input.splice(
            range.end..range.end,
            EXPLICIT
                .get(range)
                .ok_or("original field bytes")?
                .iter()
                .copied(),
        );
        assert!(
            matches!(ElementalGraph::parse(&input), Err(MappingError::DuplicateProperty { tag: actual, .. }) if actual == tag)
        );
    }
    Ok(())
}

#[test]
fn semantic_metadata_cannot_hide_on_incorrect_owners() -> TestResult {
    for (tag, payload, correct_owner) in [
        (0x0437, vec![1], 0x8004),
        (0x044d, 7_u32.to_le_bytes().to_vec(), 0x8004),
        (0x060a, vec![1], 0x8005),
        (0x0010, vec![0, 0], 0x8005),
    ] {
        for (owner, old, kind) in [
            (0, 0x4000, 0x8000),
            (42, 0x0204, 0x8001),
            (100, 0x000a, 0x8003),
            (101, 0x4003, 0x8004),
            (103, 0x4004, 0x8005),
            (77, 0x4005, 0xc123),
        ] {
            if kind == correct_owner {
                continue;
            }
            let input = changed(EXPLICIT, owner, old, tag, &payload)?;
            assert!(
                matches!(ElementalGraph::parse(&input), Err(MappingError::UnsupportedProperty { tag: actual, .. }) if actual == tag)
            );
            if owner == 77 {
                let parsed = bondline_cdx::Document::parse(&input)?;
                let offset = parsed
                    .records()
                    .iter()
                    .find_map(|r| match r.kind() {
                        bondline_cdx::RecordKind::ObjectStart { id: 77, .. } => {
                            Some(r.range().start)
                        }
                        _ => None,
                    })
                    .ok_or("custom object")?;
                let mut group = input;
                group
                    .get_mut(offset..offset + 2)
                    .ok_or("object tag")?
                    .copy_from_slice(&0x8002_u16.to_le_bytes());
                assert!(
                    matches!(ElementalGraph::parse(&group), Err(MappingError::UnsupportedProperty { tag: actual, .. }) if actual == tag)
                );
            }
        }
    }
    for (owner, old) in [(101, 0x4003), (103, 0x4004)] {
        let input = changed(EXPLICIT, owner, old, 0x0204, &[0; 16])?;
        assert!(matches!(
            ElementalGraph::parse(&input),
            Err(MappingError::UnsupportedProperty { tag: 0x0204, .. })
        ));
    }
    Ok(())
}

#[test]
fn unformatted_warning_is_opaque_but_style_run_framing_is_bounded() -> TestResult {
    for payload in [&b"\0\0"[..], &b"\0\0\0\xff\x80\xfe"[..]] {
        let input = changed(EXPLICIT, 103, 0x0010, 0x0010, payload)?;
        assert_eq!(ElementalGraph::parse(&input)?.to_bytes(), input);
    }
    for payload in [
        vec![],
        vec![0],
        vec![1, 0],
        vec![1, 0, 0],
        vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        vec![255, 255, 0],
    ] {
        let input = changed(EXPLICIT, 103, 0x0010, 0x0010, &payload)?;
        assert!(matches!(
            ElementalGraph::parse(&input),
            Err(MappingError::InvalidProperty { tag: 0x0010, .. })
        ));
    }
    let mut styled = vec![1, 0];
    styled.extend_from_slice(&[0; 10]);
    styled.extend_from_slice(b"Original styled advisory");
    let input = changed(EXPLICIT, 103, 0x0010, 0x0010, &styled)?;
    let error = ElementalGraph::parse(&input)
        .err()
        .ok_or("styled advisory accepted")?;
    assert!(matches!(
        error,
        MappingError::UnsupportedProperty { tag: 0x0010, .. }
    ));
    assert_eq!(error.code(), "unsupported_content");
    Ok(())
}

#[test]
fn atom_id_is_retained_metadata_and_never_an_endpoint_or_application_identity() -> TestResult {
    for value in [0_u32, 101, 70001, u32::MAX] {
        let input = changed(EXPLICIT, 101, 0x044d, 0x044d, &value.to_le_bytes())?;
        let input = changed(&input, 102, 0x044d, 0x044d, &value.to_le_bytes())?;
        let graph = ElementalGraph::parse(&input)?;
        assert_eq!(
            graph.fragments(),
            ElementalGraph::parse(EXPLICIT)?.fragments()
        );
        assert_eq!(graph.to_bytes(), input);
    }
    for (tag, value) in [(0x0604, 70001_u32), (0x0605, 90005)] {
        let input = changed(EXPLICIT, 103, tag, tag, &value.to_le_bytes())?;
        assert!(
            matches!(ElementalGraph::parse(&input), Err(MappingError::UnresolvedReference { id, .. }) if id == value)
        );
    }
    Ok(())
}

#[test]
fn nonstereo_cip_does_not_authorize_other_unmapped_chemical_properties() -> TestResult {
    let input = changed(EXPLICIT, 101, 0x4003, 0x042b, &[0, 0])?;
    let graph = ElementalGraph::parse(&input)?;
    let mut expected = serde_json::to_value(ElementalGraph::parse(EXPLICIT)?.fragments())?;
    expected[0]["atoms"][0]["explicit_hydrogens"] = serde_json::json!(0);
    assert_eq!(serde_json::to_value(graph.fragments())?, expected);
    assert_eq!(graph.as_bytes(), input);
    assert_eq!(graph.to_bytes(), input);
    let stored = StoredDocument::from_cdx(input.clone().into(), PageSize::A4)?;
    assert!(!stored.is_fully_mapped());
    assert!(!stored.has_display_projection());
    assert_eq!(stored.to_cdx()?.as_ref(), input);
    for (owner, old, tag, payload) in [
        (101, 0x4003, 0x0430, vec![1]),
        (101, 0x4003, 0x0431, vec![1, 0]),
        (101, 0x4003, 0x0446, vec![1]),
        (101, 0x4003, 0x0447, vec![1, 0]),
        (103, 0x4004, 0x0601, vec![6, 0]),
        (103, 0x4004, 0x060b, vec![1, 0, 0, 0]),
    ] {
        let input = changed(EXPLICIT, owner, old, tag, &payload)?;
        assert!(
            matches!(ElementalGraph::parse(&input), Err(MappingError::UnsupportedProperty { tag: actual, .. }) if actual == tag)
        );
    }
    Ok(())
}

#[test]
fn metadata_snapshot_identity_baseline_and_export_guards_keep_complete_source() -> TestResult {
    let original: Arc<[u8]> = Arc::from(HEADER);
    let pointer = original.as_ptr();
    let clean = StoredDocument::from_cdx(original, PageSize::A4)?;
    assert_eq!(clean.export_cdx()?.as_ptr(), pointer);
    assert!(!clean.is_fully_mapped());
    let altered = changed(HEADER, 101, 0x044d, 0x044d, &u32::MAX.to_le_bytes())?;
    let other = StoredDocument::from_cdx(Arc::from(altered.as_slice()), PageSize::A4)?;
    assert_eq!(other.model(), clean.model());
    assert_ne!(other, clean);
    assert_eq!(other.to_cdx()?.as_ref(), altered);
    assert_eq!(
        StoredDocument::retained_bytes([&clean, &clean]),
        StoredDocument::retained_bytes([&clean])
    );
    let mut edited = clean.clone();
    assert_eq!(
        StoredDocument::retained_bytes([&clean, &edited])
            - StoredDocument::retained_bytes([&clean]),
        size_of::<StoredDocument>()
    );
    edited.model_mut().set_page_size(
        PageId::new(1).ok_or("page id")?,
        PageSize::new(144.0, 288.0)?,
    )?;
    assert!(edited.source_info().ok_or("source")?.modified);
    assert!(matches!(
        edited.to_cdx(),
        Err(DocumentError::UnsupportedCdxEdit)
    ));
    assert!(matches!(
        edited.export_cdx(),
        Err(DocumentError::UnsupportedCdxEdit)
    ));
    assert_eq!(clean.to_cdx()?.as_ref(), HEADER);
    assert!(matches!(
        clean.to_cdxml(),
        Err(DocumentError::CrossFormatSource {
            from: "cdx",
            to: "cdxml"
        })
    ));
    let frame = MolTransform::new(24.0, Point2::new(72.0, 72.0)?)?;
    for version in [Version::V2000, Version::V3000] {
        assert!(matches!(
            clean.to_mol_as(version, frame),
            Err(DocumentError::CrossFormatSource {
                from: "cdx",
                to: "mol"
            })
        ));
    }
    *edited.model_mut() = clean.model().clone();
    assert_eq!(edited.export_cdx()?, HEADER);
    Ok(())
}
