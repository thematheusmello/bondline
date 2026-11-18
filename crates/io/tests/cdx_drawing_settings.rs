//! Independent original CDX scalar records; no vendor files or product writer.
use bondline_io::StoredDocument;
use bondline_model::PageSize;
use serde_json::json;
use std::sync::Arc;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

// Original chosen values:48.5pt and137.5deg, fixed16.16 little-endian.
const ROOT_VALUES: &[u8] = b"\x05\x08\x04\x00\x00\x80\x30\x00\x03\x08\x04\x00\x00\x80\x89\x00";
fn original_cdx(explicit: bool, root_values: &[u8]) -> Vec<u8> {
    let mut bytes = b"VjCD0100\x04\x03\x02\x01".to_vec();
    bytes.extend_from_slice(&[0; 16]);
    if explicit {
        bytes.extend_from_slice(b"\x00\x80\x00\x00\x00\x00");
    }
    bytes.extend_from_slice(root_values);
    // Original empty page1, its end, then the explicit/implicit Document end.
    bytes.extend_from_slice(b"\x01\x80\x01\x00\x00\x00\x00\x00\x00\x00");
    bytes
}

#[test]
fn original_explicit_cdx_projects_literal_root_settings_without_edit_authority() -> TestResult {
    assert_eq!(i32::from_le_bytes([0x00, 0x80, 0x30, 0x00]), 3_178_496);
    assert_eq!(i32::from_le_bytes([0x00, 0x80, 0x89, 0x00]), 9_011_200);
    let bytes: Arc<[u8]> = original_cdx(true, ROOT_VALUES).into();
    let pointer = bytes.as_ptr();
    let stored = StoredDocument::from_cdx(Arc::clone(&bytes), PageSize::A4)?;
    assert_eq!(
        serde_json::to_value(stored.model())?["drawing_settings"],
        json!({"bond_length_pt":48.5,"chain_angle_degrees":137.5})
    );
    assert!(!stored.is_fully_mapped());
    assert_eq!(stored.export_cdx()?, bytes.as_ref());
    assert_eq!(stored.export_cdx()?.as_ptr(), pointer);
    assert_eq!(stored.to_cdx()?.as_ref(), bytes.as_ref());
    Ok(())
}

// Additive families after the immutable1,665-byte existing-API declaration.
const LENGTH: &[u8] = b"\x05\x08\x04\x00\x00\x80\x30\x00";
const ANGLE: &[u8] = b"\x03\x08\x04\x00\x00\x80\x89\x00";
fn check_original(bytes: Vec<u8>, expected: serde_json::Value) -> TestResult {
    let bytes: Arc<[u8]> = bytes.into();
    let pointer = bytes.as_ptr();
    let stored = StoredDocument::from_cdx(Arc::clone(&bytes), PageSize::A4)?;
    let model = serde_json::to_value(stored.model())?;
    if expected.is_null() {
        assert!(
            model.get("drawing_settings").is_none(),
            "empty data stays omitted, never a fallback"
        );
    } else {
        assert_eq!(model["drawing_settings"], expected);
    }
    assert!(!stored.is_fully_mapped());
    assert_eq!(stored.export_cdx()?, bytes.as_ref());
    assert_eq!(stored.export_cdx()?.as_ptr(), pointer);
    assert_eq!(stored.to_cdx()?.as_ref(), bytes.as_ref());
    assert_eq!(
        serde_json::to_value(stored.source_info().ok_or("source")?)?,
        json!({"format":"cdx","byte_length":bytes.len(),"modified":false})
    );
    Ok(())
}

#[test]
fn implicit_scalars_are_independent_and_original_opaque_graph_bytes_remain_owned() -> TestResult {
    for (properties, expected) in [
        (&b""[..], serde_json::Value::Null),
        (LENGTH, json!({"bond_length_pt":48.5})),
        (ANGLE, json!({"chain_angle_degrees":137.5})),
        (
            ROOT_VALUES,
            json!({"bond_length_pt":48.5,"chain_angle_degrees":137.5}),
        ),
    ] {
        check_original(original_cdx(false, properties), expected)?;
    }
    // This existing MIT/Apache fixture is original Bondline data with65,535-byte
    // opaque content. Insert only our literal records after its encoded root0.
    const SOURCE: &[u8] = include_bytes!("../../cdx/tests/fixtures/original-ethane-explicit.cdx");
    assert_eq!(&SOURCE[28..34], b"\x00\x80\x00\x00\x00\x00");
    let mut bytes = SOURCE[..34].to_vec();
    bytes.extend_from_slice(ROOT_VALUES);
    bytes.extend_from_slice(&SOURCE[34..]);
    assert_eq!(&bytes[..34], &SOURCE[..34]);
    assert_eq!(&bytes[50..], &SOURCE[34..]);
    let old = StoredDocument::from_cdx(Arc::from(SOURCE), PageSize::A4)?;
    let current = StoredDocument::from_cdx(Arc::from(bytes.as_slice()), PageSize::A4)?;
    let model = serde_json::to_value(current.model())?;
    assert_eq!(
        model["drawing_settings"],
        json!({"bond_length_pt":48.5,"chain_angle_degrees":137.5})
    );
    assert_eq!(model["pages"], serde_json::to_value(old.model())?["pages"]);
    let atoms = current.model().pages()[0].fragments()[0].atoms();
    assert_eq!(
        atoms[0].position,
        Some(bondline_model::Point2::new(10.25, -3.5)?)
    );
    assert_eq!(
        atoms[1].position,
        Some(bondline_model::Point2::new(26.25, 8.5)?)
    );
    assert!(!current.is_fully_mapped());
    assert_eq!(current.export_cdx()?, bytes);
    assert_eq!(old.export_cdx()?, SOURCE);
    Ok(())
}

#[test]
fn ambiguous_width_and_unsupported_root_values_stay_opaque_per_field() -> TestResult {
    // Each tuple is a malformed/unsupported *projection*, still a bounded raw
    // property sequence. A different valid field must remain projected.
    for (bad, is_length) in [
        (
            &b"\x05\x08\x04\x00\x00\x80\x30\x00\x05\x08\x04\x00\x00\x80\x30\x00"[..],
            true,
        ),
        (&b"\x05\x08\x02\x00\x80\x30"[..], true),
        (&b"\x05\x08\x04\x00\x00\x00\x00\x00"[..], true),
        // Signed -65536 means-1pt, not unsigned65535pt (which would fit the model cap).
        (&b"\x05\x08\x04\x00\x00\x00\xff\xff"[..], true),
        (
            &b"\x03\x08\x04\x00\x00\x80\x89\x00\x03\x08\x04\x00\x00\x40\x5a\x00"[..],
            false,
        ),
        (
            &b"\x03\x08\x08\x00\x00\x80\x89\x00\x00\x00\x00\x00"[..],
            false,
        ),
        (&b"\x03\x08\x04\x00\x00\x00\x00\x00"[..], false),
        (&b"\x03\x08\x04\x00\x00\x00\xb4\x00"[..], false),
    ] {
        let mut fields = bad.to_vec();
        fields.extend_from_slice(if is_length { ANGLE } else { LENGTH });
        let expected = if is_length {
            json!({"chain_angle_degrees":137.5})
        } else {
            json!({"bond_length_pt":48.5})
        };
        for explicit in [false, true] {
            check_original(original_cdx(explicit, &fields), expected.clone())?;
        }
    }
    let mut invalid_both = b"\x05\x08\x04\x00\x00\x00\x00\x00".to_vec();
    invalid_both.extend_from_slice(b"\x03\x08\x04\x00\x00\x00\xb4\x00");
    check_original(original_cdx(true, &invalid_both), serde_json::Value::Null)?;
    Ok(())
}

#[test]
fn nonroot_values_neither_supply_defaults_nor_override_a_document_scalar() -> TestResult {
    for explicit in [false, true] {
        for (root_fields, child_fields, expected) in [
            (&b""[..], ROOT_VALUES, serde_json::Value::Null),
            (LENGTH, ANGLE, json!({"bond_length_pt":48.5})),
            // A valid page length cannot rescue an unsupported root length.
            (
                &b"\x05\x08\x04\x00\x00\x00\x00\x00"[..],
                LENGTH,
                serde_json::Value::Null,
            ),
        ] {
            let mut bytes = original_cdx(explicit, root_fields);
            let before_page_end = bytes.len() - 4;
            bytes.splice(
                before_page_end..before_page_end,
                child_fields.iter().copied(),
            );
            check_original(bytes, expected)?;
        }
    }
    Ok(())
}
