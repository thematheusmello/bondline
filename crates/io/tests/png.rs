use bondline_render::Frame;
use std::{
    io::Write,
    process::{Command, Stdio},
};

#[test]
fn png_is_decoded_by_an_independent_pillow_process() -> Result<(), Box<dyn std::error::Error>> {
    let frame = Frame {
        width: 3,
        height: 2,
        rgba: vec![
            255, 0, 0, 255, 0, 255, 0, 128, 0, 0, 255, 0, 12, 34, 56, 255, 0, 0, 0, 0, 255, 255,
            255, 255,
        ],
    };
    let png = bondline_io::encode_png(&frame)?;
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child = Command::new(python).args(["-c", r#"
import io,json,sys,PIL
from PIL import Image
assert PIL.__version__ == '12.3.0', 'Install requirements-dev.txt for the pinned independent PNG oracle'
data=sys.stdin.buffer.read()
with Image.open(io.BytesIO(data)) as image:
    image.verify()
with Image.open(io.BytesIO(data)) as image:
    image.load()
    print(json.dumps({'format':image.format,'size':image.size,'mode':image.mode,'rgba':list(image.tobytes())}))
"#]).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
    child
        .stdin
        .take()
        .ok_or("missing oracle stdin")?
        .write_all(&png)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "Pillow PNG verification failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let decoded: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(decoded["format"], "PNG");
    assert_eq!(decoded["mode"], "RGBA");
    assert_eq!(decoded["size"], serde_json::json!([3, 2]));
    assert_eq!(decoded["rgba"], serde_json::json!(frame.rgba));
    Ok(())
}

#[test]
fn png_encoder_rejects_bad_frames_before_writing() {
    for (width, height, rgba) in [
        (1, 1, vec![0; 3]),
        (0, 1, vec![]),
        (u32::MAX, u32::MAX, vec![]),
        (2049, 1, vec![]),
    ] {
        assert!(
            bondline_io::encode_png(&Frame {
                width,
                height,
                rgba
            })
            .is_err()
        );
    }
}
