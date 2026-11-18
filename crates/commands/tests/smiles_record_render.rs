//! Receiving coverage for current complete-source SMILES Drawing captures, not a new RED.

use bondline_commands::Engine;
use serde_json::json;
use std::{
    fs,
    io::Write as _,
    process::{Command as Process, Stdio},
    time::{Duration, UNIX_EPOCH},
};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse the original isolated test directory")]
mod support;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const SOURCE: &[u8] = b"CC first\r\nC1.N.C1 \t\0\xffPRIVATE-META\r\nCC duplicate\tname\n";

fn capture_pair(engine: &mut Engine) -> TestResult<String> {
    let before = serde_json::to_value(engine.state())?;
    let owner = engine.content().clone();
    let stamp = engine.capture_stamp();
    let source = engine.content().smiles_file()?.clone();
    let mut captures = Vec::new();
    for backend in ["cpu", "gpu"] {
        let result = engine.dispatch(
            "render.capture",
            json!({"page_id":1,"width_px":1024,"height_px":1024,"backend":backend}),
        )?;
        assert!(!result.changed);
        assert_eq!(
            result.revision,
            before["revision"].as_u64().ok_or("revision")?
        );
        let capture = result.capture.ok_or("registered capture result")?;
        assert_eq!((capture.width_px, capture.height_px), (1024, 1024));
        assert_eq!(capture.mime_type, "image/png");
        if backend == "gpu" {
            let adapter = capture.adapter.as_ref().ok_or("actual GPU adapter")?;
            assert!(!adapter.name.is_empty());
            assert!(!adapter.backend.is_empty());
            assert!(!adapter.device_type.is_empty());
        } else {
            assert!(capture.adapter.is_none());
        }
        captures.push(capture.png_base64);
        assert_eq!(serde_json::to_value(engine.state())?, before);
        assert_eq!(engine.content(), &owner);
        assert!(engine.capture_stamp_matches(&stamp));
        assert_eq!(engine.content().smiles_file()?, &source);
        assert_eq!(
            engine.content().smiles_file()?.as_bytes().as_ptr(),
            source.as_bytes().as_ptr()
        );
    }
    let cpu = captures.first().ok_or("CPU PNG")?;
    let gpu = captures.get(1).ok_or("GPU PNG")?;
    assert_eq!(gpu, cpu, "actual CPU/GPU PNG bytes must agree exactly");
    Ok(cpu.clone())
}

#[test]
fn complete_source_record_captures_agree_before_and_after_the_middle_record_edit() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("render-records-λ.smi");
    fs::write(&input, SOURCE)?;
    let old_time = UNIX_EPOCH + Duration::from_secs(946_684_800);
    fs::OpenOptions::new()
        .write(true)
        .open(&input)?
        .set_times(fs::FileTimes::new().set_modified(old_time))?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_smiles", json!({"path":input}))?;
    engine.dispatch(
        "document.import_smiles_records",
        json!({"expected_revision":engine.state().revision,"selection":{"mode":"all"},
            "bond_length_pt":64,"origin_x_pt":96,"origin_y_pt":96,
            "page_width_pt":612,"page_height_pt":792}),
    )?;
    assert!(engine.stored_document()?.has_smiles_projection());
    assert_eq!(engine.document()?.pages()[0].fragments().len(), 3);
    assert!(!engine.state().file.dirty);
    assert_eq!(engine.content().smiles_file()?.as_bytes(), SOURCE);
    let before = capture_pair(&mut engine)?;
    engine.dispatch(
        "bond.set_order",
        json!({"expected_revision":engine.state().revision,"page_id":1,
            "fragment_id":2,"bond_id":2,"order":"double",
            "hydrogen_policy":"rectify_neutral_carbon"}),
    )?;
    assert_eq!(engine.state().undo_depth, 1);
    assert!(engine.state().file.dirty);
    let after = capture_pair(&mut engine)?;
    assert_ne!(
        before, after,
        "the middle record's changed bond/H must change ink"
    );
    assert_eq!(fs::read(&input)?, SOURCE);
    assert_eq!(fs::metadata(&input)?.modified()?, old_time);

    let mut oracle = Process::new(std::env::var_os("BONDLINE_PYTHON").ok_or("pinned Python")?)
        .arg("-B")
        .arg("-c")
        .arg(
            r#"
import base64, io, json, sys, PIL
from PIL import Image
assert PIL.__version__ == '12.3.0', 'Install pinned requirements-dev.txt'
payload=json.load(sys.stdin)
frames=[]
for name in ('before','after'):
    data=base64.b64decode(payload[name],validate=True)
    with Image.open(io.BytesIO(data)) as image:
        image.verify()
    with Image.open(io.BytesIO(data)) as image:
        assert image.format=='PNG' and image.mode=='RGBA' and image.size==(1024,1024)
        pixels=image.tobytes()
        assert any(pixels[i:i+4]==bytes((0,0,0,255)) for i in range(0,len(pixels),4)), 'no molecular ink'
        frames.append(pixels)
assert frames[0]!=frames[1], 'actual decoded molecular ink must change'
"#,
        )
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    oracle
        .stdin
        .take()
        .ok_or("Pillow stdin")?
        .write_all(&serde_json::to_vec(
            &json!({"before":before,"after":after}),
        )?)?;
    let output = oracle.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}
