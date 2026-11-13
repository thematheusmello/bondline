use bondline_commands::{Cancellation, Command, Engine};
use serde_json::{Value, json};
use std::process::{Command as Process, Stdio};

fn request(backend: &str) -> Value {
    json!({"page_id":1,"width_px":8,"height_px":8,"backend":backend})
}

#[test]
fn deferred_capture_keeps_its_request_revision_and_independent_page_pixels()
-> Result<(), Box<dyn std::error::Error>> {
    let mut engine = Engine::default();
    engine.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":72,"height_pt":144}),
    )?;
    let capture = engine.prepare(Command::parse("render.capture", request("cpu"))?)?;
    assert!(capture.is_deferred());
    engine.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":144,"height_pt":72}),
    )?;
    let before = serde_json::to_vec(&engine.state())?;
    let result = engine.finish(capture.complete()?)?;
    assert_eq!(result.revision, 1);
    assert!(!result.changed);
    assert_eq!(result.undo_depth, 1);
    assert_eq!(result.redo_depth, 0);
    assert_eq!(serde_json::to_vec(&engine.state())?, before);
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child = Process::new(python)
        .args([
            "-c",
            r#"
import base64,io,json,sys,PIL
from PIL import Image
assert PIL.__version__ == '12.3.0', 'Install requirements-dev.txt'
result=json.load(sys.stdin)
assert result['document']['pages'][0]['size'] == {'width_pt':72.0,'height_pt':144.0}
capture=result['capture']
assert capture['backend']=='cpu' and capture['adapter'] is None
assert capture['width_px']==8 and capture['height_px']==8 and capture['mime_type']=='image/png'
data=base64.b64decode(capture['png_base64'],validate=True)
with Image.open(io.BytesIO(data)) as image:
    image.verify()
with Image.open(io.BytesIO(data)) as image:
    assert image.format=='PNG' and image.mode=='RGBA' and image.size==(8,8)
    grid=['........','..bbb...','..bwb...','..bwb...','..bwb...','..bwbsss','..bbbsss','.....sss']
    colors={'.':232,'b':196,'w':255,'s':210}
    expected=bytes(channel for row in grid for pixel in row for channel in [colors[pixel]]*3+[255])
    assert image.tobytes()==expected, 'capture differs from independent 8x8 page grid'
"#,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    serde_json::to_writer(child.stdin.take().ok_or("missing oracle stdin")?, &result)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

#[test]
fn explicit_gpu_capture_matches_cpu_and_reports_the_real_adapter()
-> Result<(), Box<dyn std::error::Error>> {
    let mut engine = Engine::default();
    let cpu = engine
        .dispatch("render.capture", request("cpu"))?
        .capture
        .ok_or("missing CPU capture")?;
    let gpu = engine
        .dispatch("render.capture", request("gpu"))?
        .capture
        .ok_or("missing GPU capture")?;
    assert_eq!(gpu.png_base64, cpu.png_base64);
    let adapter = gpu.adapter.ok_or("GPU did not report its adapter")?;
    assert!(!adapter.name.is_empty());
    assert!(!adapter.backend.is_empty());
    assert_eq!(engine.state().revision, 0);
    assert_eq!(engine.state().undo_depth, 0);
    Ok(())
}

#[test]
fn capture_validation_and_cancellation_cannot_mutate_authoring_or_history()
-> Result<(), Box<dyn std::error::Error>> {
    let mut engine = Engine::default();
    let before = serde_json::to_vec(&engine.state())?;
    for (field, value, code) in [
        ("width_px", json!(0), "invalid_params"),
        ("width_px", json!(2049), "invalid_params"),
        ("height_px", json!(u32::MAX), "invalid_params"),
        ("page_id", json!(0), "invalid_params"),
        ("page_id", json!(2), "not_found"),
        ("backend", json!("automatic"), "invalid_params"),
        ("typo", json!(true), "invalid_params"),
    ] {
        let mut params = request("cpu");
        params[field] = value;
        assert_eq!(
            engine
                .dispatch("render.capture", params)
                .err()
                .ok_or("accepted invalid capture")?
                .info()
                .code,
            code
        );
        assert_eq!(serde_json::to_vec(&engine.state())?, before);
    }
    let cancel = Cancellation::default();
    cancel.cancel();
    let prepared = engine.prepare(Command::parse("render.capture", request("gpu"))?)?;
    assert_eq!(
        prepared
            .complete_with_cancel(&cancel)
            .err()
            .ok_or("ignored cancellation")?
            .info()
            .code,
        "cancelled"
    );
    let edit = engine.prepare(Command::parse(
        "page.set_size",
        json!({"page_id":1,"width_pt":72,"height_pt":144}),
    )?)?;
    assert!(!edit.is_deferred());
    // An already committed authoring edit must still report success if its caller cancels.
    assert!(engine.finish(edit.complete_with_cancel(&cancel)?)?.changed);
    assert_eq!(engine.state().revision, 1);
    Ok(())
}
