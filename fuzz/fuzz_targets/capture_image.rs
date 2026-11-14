#![no_main]

use base64::{Engine as _, engine::general_purpose::STANDARD};
use bondline_commands::{PngCapture, RenderBackend};
use libfuzzer_sys::fuzz_target;

fn word(data: &[u8], offset: usize) -> u32 {
    data.get(offset..offset + 4)
        .and_then(|bytes| bytes.try_into().ok())
        .map(u32::from_le_bytes)
        .unwrap_or(0)
}

fn assert_envelope(bytes: &[u8], width: u32, height: u32) {
    assert!((1..=2048).contains(&width) && (1..=2048).contains(&height));
    let raw = u64::from(height) * (4 * u64::from(width) + 1);
    let limit = raw + 5 * raw.div_ceil(65_535) + 63;
    assert!(bytes.len() as u64 <= limit);
    assert_eq!(bytes.get(..8), Some(b"\x89PNG\r\n\x1a\n".as_slice()));
    assert_eq!(
        u32::from_be_bytes(bytes.get(16..20).unwrap().try_into().unwrap()),
        width
    );
    assert_eq!(
        u32::from_be_bytes(bytes.get(20..24).unwrap().try_into().unwrap()),
        height
    );
    assert_eq!(bytes.get(24..29), Some([8, 6, 0, 0, 0].as_slice()));

    // Walk all framing independently; do not duplicate the reader's fixed-offset tail check.
    let mut cursor = 8_usize;
    let mut chunks = Vec::new();
    while cursor < bytes.len() {
        let header = bytes.get(cursor..cursor.checked_add(8).unwrap()).unwrap();
        let size = u32::from_be_bytes(header.get(..4).unwrap().try_into().unwrap());
        let name: [u8; 4] = header.get(4..8).unwrap().try_into().unwrap();
        let end = cursor
            .checked_add(12)
            .unwrap()
            .checked_add(size as usize)
            .unwrap();
        assert!(end <= bytes.len());
        chunks.push((name, size));
        assert!(chunks.len() <= 3);
        cursor = end;
    }
    assert_eq!(cursor, bytes.len());
    assert_eq!(chunks.len(), 3);
    assert_eq!(chunks.first(), Some(&(*b"IHDR", 13)));
    assert!(matches!(chunks.get(1), Some((name, size)) if name == b"IDAT" && *size > 0));
    assert_eq!(chunks.get(2), Some(&(*b"IEND", 0)));
}

fn inspect(capture: PngCapture, canonical_payload: Option<&[u8]>) {
    let before = capture.clone();
    let first = capture.png_bytes();
    assert_eq!(
        first,
        capture.png_bytes(),
        "repeated decode must be deterministic"
    );
    match first {
        Ok(bytes) => {
            assert_eq!(capture.mime_type, "image/png");
            assert_envelope(&bytes, capture.width_px, capture.height_px);
            assert_eq!(STANDARD.encode(&bytes), capture.png_base64);
            if let Some(payload) = canonical_payload {
                assert_eq!(
                    bytes, payload,
                    "canonical byte path must return the original payload"
                );
            }
        }
        Err(error) => assert!(matches!(
            error.code(),
            "invalid_capture_image" | "resource_limit"
        )),
    }
    assert_eq!(capture.mime_type, before.mime_type);
    assert_eq!(capture.width_px, before.width_px);
    assert_eq!(capture.height_px, before.height_px);
    assert_eq!(capture.png_base64, before.png_base64);
    assert!(matches!(capture.backend, RenderBackend::Cpu));
    assert!(capture.adapter.is_none() && before.adapter.is_none());
    assert_eq!(
        capture.label_em_px.map(f32::to_bits),
        before.label_em_px.map(f32::to_bits)
    );
}

fuzz_target!(|data: &[u8]| {
    // Original input framing: MIME selector, widthLEu32, heightLEu32, then body bytes.
    let mime_type = if data.first().copied().unwrap_or(0) & 1 == 0 {
        "image/png"
    } else {
        "image/jpeg"
    };
    let width_px = word(data, 1);
    let height_px = word(data, 5);
    let payload = data.get(9..).unwrap_or_default();
    let capture = |png_base64| PngCapture {
        mime_type,
        width_px,
        height_px,
        backend: RenderBackend::Cpu,
        adapter: None,
        png_base64,
        label_em_px: None,
    };
    inspect(capture(STANDARD.encode(payload)), Some(payload));
    // Invalid UTF-8 cannot inhabit the public String field; the binary path still ran.
    if let Ok(text) = std::str::from_utf8(payload) {
        inspect(capture(text.to_owned()), None);
    }
});
