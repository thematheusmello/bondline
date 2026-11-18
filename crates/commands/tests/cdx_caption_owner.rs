//! The caption display grant keeps exact source owners and transient views.
use bondline_commands::{Command, Engine};
use serde_json::{Value, json};
use std::{fs, process::Command as Process, process::Stdio, sync::Arc};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse the existing isolated file directory")]
mod files;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const A: &[u8] = include_bytes!("../../io/tests/fixtures/cdx-caption/caption_ascii.cdx");
const B: &[u8] = include_bytes!("../../io/tests/fixtures/cdx-caption/caption_ascii_explicit.cdx");
const FRACTIONAL: &[u8] =
    include_bytes!("../../io/tests/fixtures/cdx-caption/caption_fractional.cdx");
const VECTORS: &str = include_str!("../../io/tests/fixtures/cdx-caption/caption-vectors.json");

fn model(name: &str) -> Result<Value> {
    let manifest: Value = serde_json::from_str(VECTORS)?;
    Ok(manifest["cases"]
        .as_array()
        .ok_or("missing cases")?
        .iter()
        .find(|case| case["name"] == name)
        .ok_or("missing named model")?["model"]
        .clone())
}

#[test]
fn equal_model_caption_sources_retire_the_previous_save_and_capture_contexts() -> Result {
    let expected = model("caption_ascii")?;
    assert_eq!(model("caption_ascii_explicit")?, expected);
    assert_ne!(A, B);
    let dir = files::FileTestDirectory::new()?;
    let original = dir.path("caption-a-原.cdx");
    let replacement = dir.path("caption-b-λ.cdx");
    let output = dir.path("held-caption-保存.cdx");
    fs::write(&original, A)?;
    fs::write(&replacement, B)?;
    let mut session = Engine::default();
    session.dispatch("document.open_cdx", json!({"path":original}))?;
    assert_eq!(serde_json::to_value(session.document()?)?, expected);
    assert!(session.stored_document()?.has_display_projection());
    let capture = session.capture_stamp();
    let file = session.ui_file_context_stamp();
    let save = session
        .prepare(Command::parse("document.save_cdx", json!({"path":output}))?)?
        .complete()?;

    session.dispatch("document.open_cdx", json!({"path":replacement}))?;
    assert_eq!(serde_json::to_value(session.document()?)?, expected);
    assert!(session.stored_document()?.has_display_projection());
    assert!(!session.capture_stamp_matches(&capture));
    assert!(!session.ui_file_context_stamp_matches(&file));
    let before = serde_json::to_value(session.state())?;
    session.finish(save)?;
    assert_eq!(serde_json::to_value(session.state())?, before);
    assert_eq!(session.stored_document()?.export_cdx()?, B);
    assert_eq!(fs::read(&output)?, A);
    assert_eq!(fs::read(&replacement)?, B);

    session.dispatch("document.open_cdx", json!({"path":original}))?;
    assert_eq!(serde_json::to_value(session.document()?)?, expected);
    assert!(!session.capture_stamp_matches(&capture));
    assert!(!session.ui_file_context_stamp_matches(&file));
    assert_eq!(session.stored_document()?.export_cdx()?, A);
    Ok(())
}

#[test]
fn caption_fit_and_actual_capture_preserve_source_at_fractional_density() -> Result {
    let dir = files::FileTestDirectory::new()?;
    let path = dir.path("caption-12.05-λ.cdx");
    fs::write(&path, FRACTIONAL)?;
    let mut session = Engine::default();
    session.dispatch("document.open_cdx", json!({"path":path}))?;
    assert_eq!(
        serde_json::to_value(session.document()?)?,
        model("caption_fractional")?
    );
    let original = session.stored_document()?.model_snapshot();
    let state = serde_json::to_value(session.state())?;
    let stamp = session.capture_stamp();
    let mut images = Vec::new();
    for action in ["fit", "actual"] {
        assert!(
            !session
                .dispatch("ui.view", json!({"action":action}))?
                .changed
        );
        let view = session.view();
        let request = |backend| {
            json!({
                "page_id":1,"width_px":768,"height_px":768,"backend":backend,
                "view":{"logical_width":512,"logical_height":512,"expected_view_epoch":view.epoch}
            })
        };
        let cpu = session
            .dispatch("render.capture", request("cpu"))?
            .capture
            .ok_or("missing CPU caption")?;
        let gpu = session
            .dispatch("render.capture", request("gpu"))?
            .capture
            .ok_or("missing GPU caption")?;
        assert_eq!(cpu.png_base64, gpu.png_base64, "{action}");
        assert!(gpu.adapter.is_some());
        assert_eq!(serde_json::to_value(session.state())?, state);
        assert!(session.capture_stamp_matches(&stamp));
        assert!(Arc::ptr_eq(
            &original,
            &session.stored_document()?.model_snapshot()
        ));
        images.push(cpu.png_base64);
    }
    let python = std::env::var_os("BONDLINE_PYTHON").ok_or("pinned BONDLINE_PYTHON")?;
    let mut consumer = Process::new(python)
        .args([
            "-B",
            "-c",
            r#"
import base64, io, json, sys
import PIL
from PIL import Image, ImageFont
assert PIL.__version__ == '12.3.0'
images = json.load(sys.stdin)
assert len(images) == 2 and images[0] != images[1]
source_x, source_y = 80.25, 100.75
for encoded, scale, margin in zip(images, (0.875 * 1.5, 1.5), (48, 0)):
    with Image.open(io.BytesIO(base64.b64decode(encoded, validate=True))) as im:
        assert im.format == 'PNG' and im.mode == 'RGBA' and im.size == (768, 768)
        ink = [(i % 768, i // 768) for i, color in enumerate(im.getdata()) if color == (0, 0, 0, 255)]
        assert ink
        actual = [min(p[0] for p in ink), min(p[1] for p in ink),
                  max(p[0] for p in ink), max(p[1] for p in ink)]
    font = ImageFont.truetype(sys.argv[1], round(12.05 * scale))
    left, top, right, bottom = font.getbbox('AV2', anchor='ls')
    x, y = margin + source_x * scale, margin + source_y * scale
    independent = [x + left, y + top, x + right, y + bottom]
    assert all(abs(a - b) <= 4 for a, b in zip(actual, independent)), (actual, independent)
"#,
        ])
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../assets/fonts/Roboto-Regular.ttf"
        ))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    serde_json::to_writer(
        consumer
            .stdin
            .take()
            .ok_or("missing image consumer input")?,
        &images,
    )?;
    let verified = consumer.wait_with_output()?;
    assert!(
        verified.status.success(),
        "{}",
        String::from_utf8_lossy(&verified.stderr)
    );
    assert_eq!(fs::read(path)?, FRACTIONAL);
    Ok(())
}
