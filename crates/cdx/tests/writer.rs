use bondline_cdx::{Error, Writer};
use std::{
    io::Write,
    process::{Command, Stdio},
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn explicit(body: &[u8]) -> Vec<u8> {
    let mut bytes = b"VjCD0100\x04\x03\x02\x01".to_vec();
    bytes.extend_from_slice(&[0; 16]);
    bytes.extend_from_slice(&[0, 0x80, 0, 0, 0, 0]);
    bytes.extend_from_slice(body);
    bytes.extend_from_slice(&[0, 0]);
    bytes
}

#[test]
fn empty_root_invalid_tags_and_unbalanced_calls_have_literal_byte_expectations() -> TestResult {
    assert_eq!(Writer::new()?.finish()?, explicit(&[]));
    let mut writer = Writer::new()?;
    assert_eq!(writer.end_object(), Err(Error::ObjectBalance));
    for tag in [0, 1, 0x7fff] {
        assert_eq!(writer.start_object(tag, 1), Err(Error::InvalidTag));
    }
    for tag in [0, 0x8000, 0xffff] {
        assert_eq!(
            writer.property(tag, b"private chemical data"),
            Err(Error::InvalidTag)
        );
    }
    writer.property(0x1234, b"x")?;
    assert!(!format!("{writer:?}").contains("private chemical data"));
    assert_eq!(writer.finish()?, explicit(b"\x34\x12\x01\0x"));
    let mut unclosed = Writer::new()?;
    unclosed.start_object(0xc123, 17)?;
    assert_eq!(unclosed.finish(), Err(Error::ObjectBalance));
    Ok(())
}

#[test]
fn short_and_extended_lengths_follow_the_published_65535_threshold() -> TestResult {
    for (length, prefix) in [
        (0, b"\x34\x12\0\0".as_slice()),
        (65534, b"\x34\x12\xfe\xff".as_slice()),
        (65535, b"\x34\x12\xff\xff\xff\xff\0\0".as_slice()),
    ] {
        let payload = vec![0xa7; length];
        let mut writer = Writer::new()?;
        writer.property(0x1234, &payload)?;
        let mut body = prefix.to_vec();
        body.extend_from_slice(&payload);
        assert_eq!(writer.finish()?, explicit(&body));
    }
    Ok(())
}

#[test]
fn depth_and_record_budgets_include_every_pending_terminator() -> TestResult {
    assert_eq!(bondline_cdx::MAX_DEPTH, 256);
    assert_eq!(bondline_cdx::MAX_RECORDS, 100000);
    let mut nested = Writer::new()?;
    for id in 1..=255 {
        nested.start_object(0xc123, id)?;
    }
    assert_eq!(nested.start_object(0xc123, 256), Err(Error::DepthLimit));
    for _ in 0..255 {
        nested.end_object()?;
    }
    assert_eq!(nested.finish()?.len(), 36 + 255 * 8);
    let mut full = Writer::new()?;
    for _ in 0..99998 {
        full.property(0x1234, &[])?;
    }
    assert_eq!(full.property(0x1234, &[]), Err(Error::RecordLimit));
    assert_eq!(full.start_object(0xc123, 1), Err(Error::RecordLimit));
    let bytes = full.finish()?;
    assert_eq!(bytes.len(), 36 + 99998 * 4);
    assert_eq!(bytes.get(..34), explicit(&[]).get(..34));
    assert_eq!(bytes.get(bytes.len() - 2..), Some([0, 0].as_slice()));
    Ok(())
}

#[test]
fn output_limit_includes_extended_header_and_root_end_without_partial_writes() -> TestResult {
    assert_eq!(bondline_cdx::MAX_INPUT_BYTES, 64 * 1024 * 1024);
    let payload = vec![0x5a; 64 * 1024 * 1024 - 44];
    let mut writer = Writer::new()?;
    writer.property(0x1234, &payload)?;
    assert_eq!(writer.property(0x1234, &[]), Err(Error::InputTooLarge));
    assert_eq!(writer.start_object(0xc123, 1), Err(Error::InputTooLarge));
    let bytes = writer.finish()?;
    assert_eq!(bytes.len(), 64 * 1024 * 1024);
    assert_eq!(
        bytes.get(34..42),
        Some(b"\x34\x12\xff\xff\xd4\xff\xff\x03".as_slice())
    );
    assert_eq!(bytes.get(42..bytes.len() - 2), Some(payload.as_slice()));
    drop(bytes);
    drop(payload);
    let excessive = vec![0; 64 * 1024 * 1024 - 43];
    let mut rejected = Writer::new()?;
    assert_eq!(
        rejected.property(0x1234, &excessive),
        Err(Error::InputTooLarge)
    );
    assert_eq!(rejected.finish()?, explicit(&[]));
    Ok(())
}

#[test]
fn new_explicit_tree_matches_the_independent_original_scene_and_rdkit() -> TestResult {
    let mut writer = Writer::new()?;
    writer.property(0x4000, b"root\0\xfb")?;
    let opaque = (0_u32..65535)
        .map(|index| u8::try_from((index * 37 + 11) % 256))
        .collect::<Result<Vec<_>, _>>()?;
    writer.property(0x4002, &opaque)?;
    writer.start_object(0x8001, 42)?;
    writer.property(0x4001, b"page\0\xfc")?;
    writer.start_object(0x8003, 100)?;
    for (id, coordinates, has_opaque) in [
        (101, [-229376_i32, 671744], true),
        (102, [557056_i32, 1720320], false),
    ] {
        writer.start_object(0x8004, id)?;
        let position = coordinates
            .into_iter()
            .flat_map(i32::to_le_bytes)
            .collect::<Vec<_>>();
        writer.property(0x0200, &position)?;
        writer.property(0x0402, &[6, 0])?;
        if has_opaque {
            writer.property(0x4003, b"atom\0\xff")?;
        }
        writer.end_object()?;
    }
    writer.start_object(0x8005, 103)?;
    writer.property(0x0604, &101_u32.to_le_bytes())?;
    writer.property(0x0605, &102_u32.to_le_bytes())?;
    writer.property(0x0600, &[1, 0])?;
    writer.property(0x4004, b"bond\0\xfe")?;
    writer.end_object()?;
    writer.end_object()?;
    writer.start_object(0xc123, 77)?;
    writer.property(0x4005, b"user object\0\xfd")?;
    writer.start_object(0xc124, 78)?;
    writer.end_object()?;
    writer.end_object()?;
    writer.end_object()?;
    let bytes = writer.finish()?;
    assert_eq!(
        bytes,
        include_bytes!("fixtures/original-ethane-explicit.cdx")
    );

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
            "/../../xtask/oracles/cdx_oracle.py"
        ))
        .args(["verify", "1", "explicit"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("oracle input")?
        .write_all(&bytes)?;
    let result = child.wait_with_output()?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(())
}
