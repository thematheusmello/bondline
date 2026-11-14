use base64::{Engine as _, engine::general_purpose::STANDARD};
use bondline_commands::{CaptureImageError, Engine, PngCapture, RenderBackend};
use bondline_render::Frame;
use serde_json::json;
use std::{
    io::Write as _,
    process::{Command as Process, Stdio},
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn real_registry_png_bytes_match_independent_base64_and_keep_wire_shape() -> TestResult {
    let mut engine = Engine::default();
    let before = serde_json::to_vec(&engine.state())?;
    let capture = engine
        .dispatch(
            "render.capture",
            json!({"page_id":1,"width_px":8,"height_px":8,"backend":"cpu"}),
        )?
        .capture
        .ok_or("capture")?;
    let bytes = capture.png_bytes()?;
    assert_eq!(bytes, STANDARD.decode(&capture.png_base64)?);
    assert_eq!(bytes.get(..8), Some(b"\x89PNG\r\n\x1a\n".as_slice()));
    assert_eq!(
        bytes.get(8..29),
        Some(
            [
                0, 0, 0, 13, b'I', b'H', b'D', b'R', 0, 0, 0, 8, 0, 0, 0, 8, 8, 6, 0, 0, 0
            ]
            .as_slice()
        )
    );
    assert_eq!(capture.label_em_px, None);
    let wire = serde_json::to_value(&capture)?;
    assert_eq!(wire.as_object().ok_or("capture object")?.len(), 6);
    assert!(wire.get("label_em_px").is_none());
    assert_eq!(serde_json::to_vec(&engine.state())?, before);
    Ok(())
}

fn packet(width: u32, height: u32, bytes: &[u8]) -> PngCapture {
    PngCapture {
        mime_type: "image/png",
        width_px: width,
        height_px: height,
        backend: RenderBackend::Cpu,
        adapter: None,
        png_base64: STANDARD.encode(bytes),
        label_em_px: None,
    }
}

fn page() -> Result<PngCapture, Box<dyn std::error::Error>> {
    Ok(Engine::default()
        .dispatch(
            "render.capture",
            json!({"page_id":1,"width_px":8,"height_px":8,"backend":"cpu"}),
        )?
        .capture
        .ok_or("capture")?)
}

fn python(script: &str, bytes: &[u8]) -> TestResult {
    let interpreter = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child = Process::new(interpreter)
        .args(["-B", "-c", script])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child.stdin.take().ok_or("oracle stdin")?.write_all(bytes)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

#[test]
fn actual_png_faces_pillow_and_a_literal_independent_eight_pixel_page() -> TestResult {
    let mut engine = Engine::default();
    engine.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":72,"height_pt":144}),
    )?;
    let capture = engine
        .dispatch(
            "render.capture",
            json!({"page_id":1,"width_px":8,"height_px":8,"backend":"cpu"}),
        )?
        .capture
        .ok_or("capture")?;
    python(
        r#"
import io,sys,PIL
from PIL import Image
assert PIL.__version__=='12.3.0'
source=sys.stdin.buffer.read()
with Image.open(io.BytesIO(source)) as image: image.verify()
with Image.open(io.BytesIO(source)) as image:
 assert image.format=='PNG' and image.mode=='RGBA' and image.size==(8,8)
 grid=['........','..bbb...','..bwb...','..bwb...','..bwb...','..bwbsss','..bbbsss','.....sss']
 colors={'.':232,'b':196,'w':255,'s':210}
 expected=bytes(channel for row in grid for pixel in row for channel in [colors[pixel]]*3+[255])
 assert image.tobytes()==expected
"#,
        &capture.png_bytes()?,
    )?;
    Ok(())
}

#[test]
fn actual_label_metric_is_retained_in_rust_and_omitted_from_wire() -> TestResult {
    let mut engine = Engine::default();
    engine.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":200,"height_pt":200}),
    )?;
    engine.dispatch(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":50,"begin_y_pt":100,"end_x_pt":150,"end_y_pt":100}),
    )?;
    let before = serde_json::to_vec(&engine.state())?;
    for (size, expected_em) in [(464, 24.0), (864, 48.0)] {
        let capture = engine
            .dispatch(
                "render.capture",
                json!({"page_id":1,"width_px":size,"height_px":size,"backend":"cpu"}),
            )?
            .capture
            .ok_or("capture")?;
        assert_eq!(capture.label_em_px, Some(expected_em));
        assert!(serde_json::to_value(&capture)?.get("label_em_px").is_none());
        assert!(!capture.png_bytes()?.is_empty());
    }
    assert_eq!(serde_json::to_vec(&engine.state())?, before);
    Ok(())
}

#[test]
fn metadata_and_per_dimension_limits_refuse_before_base64_decode() -> TestResult {
    let valid = page()?;
    let mut wrong_mime = valid.clone();
    wrong_mime.mime_type = "image/jpeg";
    assert_eq!(wrong_mime.png_bytes(), Err(CaptureImageError::InvalidMime));
    for (width, height) in [(0, 8), (8, 0), (2049, 1), (1, 2049), (u32::MAX, u32::MAX)] {
        let mut capture = valid.clone();
        capture.width_px = width;
        capture.height_px = height;
        assert_eq!(
            capture.png_bytes(),
            Err(CaptureImageError::InvalidDimensions)
        );
    }
    // A1x1 own PNG budget is73 bytes:5 filtered +5 stored header +6zlib +57PNG.
    let mut capture = packet(1, 1, b"");
    for source in ["!".repeat(101), "!".repeat(100)] {
        capture.png_base64 = source;
        assert_eq!(capture.png_bytes(), Err(CaptureImageError::SizeLimit));
    }
    capture.png_base64 = "!".repeat(96);
    assert_eq!(capture.png_bytes(), Err(CaptureImageError::InvalidBase64));
    for source in ["A", "AA=", "A===", "AB==", "AA A", "-_=="] {
        capture.png_base64 = source.to_owned();
        assert_eq!(
            capture.png_bytes(),
            Err(CaptureImageError::InvalidBase64),
            "{source}"
        );
    }
    Ok(())
}

#[test]
fn truncated_and_forged_ihdr_profile_are_refused_without_decoding_pixels() -> TestResult {
    let valid = page()?.png_bytes()?;
    for length in [0, 7, 8, 15, 16, 28, 32, 33, 40, valid.len() - 1] {
        assert!(
            packet(8, 8, valid.get(..length).ok_or("prefix")?)
                .png_bytes()
                .is_err()
        );
    }
    for (range, replacement, error) in [
        (0..1, vec![0], CaptureImageError::InvalidHeader),
        (
            8..12,
            14_u32.to_be_bytes().to_vec(),
            CaptureImageError::InvalidHeader,
        ),
        (12..16, b"JUNK".to_vec(), CaptureImageError::InvalidHeader),
        (
            16..20,
            9_u32.to_be_bytes().to_vec(),
            CaptureImageError::DimensionsMismatch,
        ),
        (
            20..24,
            u32::MAX.to_be_bytes().to_vec(),
            CaptureImageError::DimensionsMismatch,
        ),
        (24..25, vec![16], CaptureImageError::UnsupportedEncoding),
        (25..26, vec![2], CaptureImageError::UnsupportedEncoding),
        (26..27, vec![1], CaptureImageError::UnsupportedEncoding),
        (27..28, vec![1], CaptureImageError::UnsupportedEncoding),
        (28..29, vec![1], CaptureImageError::UnsupportedEncoding),
    ] {
        let mut bytes = valid.clone();
        bytes
            .get_mut(range)
            .ok_or("mutated field")?
            .copy_from_slice(&replacement);
        assert_eq!(packet(8, 8, &bytes).png_bytes(), Err(error));
    }
    Ok(())
}

#[test]
fn own_output_chunk_framing_rejects_auxiliary_duplicate_and_trailing_data() -> TestResult {
    let valid = page()?.png_bytes()?;
    let end = valid.len() - 12;
    for (range, replacement) in [
        (33..37, 0_u32.to_be_bytes().to_vec()),
        (33..37, u32::MAX.to_be_bytes().to_vec()),
        (37..41, b"zTXt".to_vec()),
        (end..end + 4, 1_u32.to_be_bytes().to_vec()),
        (end + 4..end + 8, b"IDAT".to_vec()),
    ] {
        let mut bytes = valid.clone();
        bytes
            .get_mut(range)
            .ok_or("chunk field")?
            .copy_from_slice(&replacement);
        assert_eq!(
            packet(8, 8, &bytes).png_bytes(),
            Err(CaptureImageError::InvalidFraming)
        );
    }
    let mut duplicate = valid.get(..end).ok_or("prefix")?.to_vec();
    duplicate.extend_from_slice(&[0, 0, 0, 0, b'I', b'D', b'A', b'T', 0, 0, 0, 0]);
    duplicate.extend_from_slice(valid.get(end..).ok_or("IEND")?);
    assert_eq!(
        packet(8, 8, &duplicate).png_bytes(),
        Err(CaptureImageError::InvalidFraming)
    );
    let mut trailing = valid;
    trailing.push(0);
    assert_eq!(
        packet(8, 8, &trailing).png_bytes(),
        Err(CaptureImageError::InvalidFraming)
    );
    Ok(())
}

#[test]
fn uncompressible_actual_writer_output_fits_stored_block_and_maximum_dimension_budgets()
-> TestResult {
    let mut random = 0x6543_210f_u32;
    for (width, height) in [
        (1, 1),
        (1, 2048),
        (2048, 1),
        (64, 255),
        (65, 255),
        (2048, 2048),
    ] {
        let length = usize::try_from(width)? * usize::try_from(height)? * 4;
        let rgba = (0..length)
            .map(|_| {
                random ^= random << 13;
                random ^= random >> 17;
                random ^= random << 5;
                random as u8
            })
            .collect();
        let frame = Frame {
            width,
            height,
            rgba,
        };
        let bytes = bondline_io::encode_png(&frame)?;
        let filtered = usize::try_from(height)? * (usize::try_from(width)? * 4 + 1);
        let budget = filtered + 5 * filtered.div_ceil(65_535) + 63;
        assert!(bytes.len() <= budget, "{} > {budget}", bytes.len());
        if (width, height) == (64, 255) {
            assert_eq!(filtered, 65_535);
            assert_eq!(bytes.len(), 65_603);
        }
        if (width, height) == (2048, 2048) {
            assert_eq!(budget, 16_780_612);
        }
        assert_eq!(packet(width, height, &bytes).png_bytes()?, bytes);
    }
    Ok(())
}

#[test]
fn crc_and_compressed_payload_validation_stays_with_the_real_decoder() -> TestResult {
    let mut bytes = page()?.png_bytes()?;
    *bytes.get_mut(41).ok_or("first IDAT payload byte")? ^= 1;
    assert_eq!(
        packet(8, 8, &bytes).png_bytes()?,
        bytes,
        "preflight is not a deflate/CRC decoder"
    );
    python(
        r#"
import io,sys,PIL
from PIL import Image
assert PIL.__version__=='12.3.0'
try:
 with Image.open(io.BytesIO(sys.stdin.buffer.read())) as image: image.verify()
except (OSError,SyntaxError): pass
else: raise AssertionError('independent decoder accepted corrupted payload/CRC')
"#,
        &bytes,
    )?;
    Ok(())
}
