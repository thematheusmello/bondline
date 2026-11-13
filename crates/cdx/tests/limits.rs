use bondline_cdx::{Document, Error, MAX_DEPTH, MAX_INPUT_BYTES, MAX_RECORDS};

const HEADER: &[u8] = b"VjCD0100\x04\x03\x02\x01\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";

#[test]
fn input_size_is_bounded_before_header_or_record_work() -> Result<(), Box<dyn std::error::Error>> {
    let length = MAX_INPUT_BYTES - 38; //28-byte header +8-byte extended property prefix +2-byte end.
    let mut source = HEADER.to_vec();
    source.extend_from_slice(&[1, 0x40, 0xff, 0xff]);
    source.extend_from_slice(&u32::try_from(length)?.to_le_bytes());
    source.resize(MAX_INPUT_BYTES - 2, 0x8f);
    source.extend_from_slice(&[0, 0]);
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

#[test]
fn simultaneous_encoded_depth_is_bounded_and_flat_drop_does_not_recurse()
-> Result<(), Box<dyn std::error::Error>> {
    for depth in [MAX_DEPTH, MAX_DEPTH + 1] {
        let mut source = HEADER.to_vec();
        for _ in 0..depth {
            source.extend_from_slice(&[1, 0xc0, 0, 0, 0, 0]);
        }
        source.extend_from_slice(&vec![0; (depth + 1) * 2]);
        if depth == MAX_DEPTH {
            let document = Document::parse(&source)?;
            assert_eq!(document.records().len(), depth * 2 + 1);
            drop(document);
        } else {
            assert!(matches!(Document::parse(&source), Err(Error::DepthLimit)));
        }
    }
    Ok(())
}

#[test]
fn total_record_budget_counts_properties_and_root_terminator()
-> Result<(), Box<dyn std::error::Error>> {
    let mut source = HEADER.to_vec();
    for _ in 0..MAX_RECORDS - 1 {
        source.extend_from_slice(&[1, 0x40, 0, 0]);
    }
    source.extend_from_slice(&[0, 0]);
    assert_eq!(Document::parse(&source)?.records().len(), MAX_RECORDS);
    source.splice(28..28, [1, 0x40, 0, 0]);
    assert!(matches!(Document::parse(&source), Err(Error::RecordLimit)));
    Ok(())
}

#[test]
fn maximum_lengths_and_truncated_fixed_fields_are_checked_without_allocation() {
    for tail in [
        &[1][..],
        &[1, 0x80, 0, 0, 0][..],
        &[1, 0x40, 0][..],
        &[1, 0x40, 0xff, 0xff, 0xff, 0xff, 0xff][..],
        &[1, 0x40, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff][..],
        &[1, 0x40, 1, 0][..],
    ] {
        let mut source = HEADER.to_vec();
        source.extend_from_slice(tail);
        assert!(matches!(
            Document::parse(&source),
            Err(Error::Truncated { .. })
        ));
    }
}
