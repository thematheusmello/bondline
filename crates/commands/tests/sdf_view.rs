use bondline_commands::{Engine, PngCapture};
use serde_json::json;
use std::{
    fs,
    io::Write as _,
    process::{Command as Process, Stdio},
};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(
    dead_code,
    reason = "reuse the original bounded temporary-directory helper"
)]
mod support;

type TestResult = Result<(), Box<dyn std::error::Error>>;
const SOURCE: &[u8] =
    include_bytes!("../../io/tests/fixtures/sdf-preview/valence-absent-mixed.sdf");

fn independent_one_x(capture: &PngCapture) -> TestResult {
    assert_eq!([capture.width_px, capture.height_px], [848, 651]);
    assert_eq!(capture.label_em_px, Some(12.0));
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child = Process::new(python)
        .args([
            "-B",
            "-c",
            r#"
import io,sys,PIL
from PIL import Image
assert PIL.__version__=='12.3.0'
raw=sys.stdin.buffer.read()
with Image.open(io.BytesIO(raw)) as im: im.verify()
with Image.open(io.BytesIO(raw)) as im:
 assert im.format=='PNG' and im.mode=='RGBA' and im.size==(848,651)
 black=lambda box:sum(pixel==(0,0,0,255) for pixel in im.crop(box).get_flattened_data())
 # Literal authored-unit transforms at100% and integer camera translation:
 # C,C,O,C,N anchors=(396,290),(432,290),(432,254),(432,398),(396,362).
 for x,y in [(414,290),(432,272),(414,380)]:
  assert black((x-1,y-1,x+2,y+2))>0,('missing source-direction chord',x,y)
 for box in [(428,249,437,260),(420,249,428,260),(392,357,401,368),
             (379,357,387,368),(387,363,392,371)]:
  assert black(box)>0,('missing O/OH/N/H/subscript ink',box)
 assert black((428,237,437,247))==0,'horizontal-left OH has no stacked H above'
 assert black((380,315,450,337))==0,'independent rows collapsed'
 assert black((411,341,418,348))==0,'source y direction reversed'
"#,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("PNG consumer stdin")?
        .write_all(&capture.png_bytes()?)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

#[test]
fn sdf_actual_bootstraps_one_x_current_capture_then_pan_without_authoring() -> TestResult {
    assert_eq!(SOURCE.len(), 740);
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("whole-source-化学.sdf");
    let output = directory.path("retained-source.sdf");
    fs::write(&input, SOURCE)?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_sdf", json!({"path":input}))?;
    let content = engine.content().clone();
    let before = serde_json::to_value(engine.state())?;
    let stamp = engine.capture_stamp();
    // The baseline must fail here at the actual registry guard, not at a future API.
    let actual = engine.dispatch("ui.view", json!({"action":"actual"}))?;
    let actual = serde_json::to_value(actual)?
        .get("ui_view_operation")
        .ok_or("Actual operation")?
        .clone();
    assert_eq!(actual["zoom_percent"], 100.0);
    assert_eq!(actual["changed"], true);
    engine.dispatch("render.capture",json!({
        "page_id":1,"width_px":848,"height_px":651,"backend":"cpu",
        "view":{"logical_width":848,"logical_height":651,"expected_view_epoch":actual["epoch"]}
    }))?.capture.ok_or("successful current-view bootstrap")?;
    let panned = engine.dispatch(
        "ui.view",
        json!({"action":"pan","center_x_pt":100,"center_y_pt":144}),
    )?;
    let panned = serde_json::to_value(panned)?
        .get("ui_view_operation")
        .ok_or("Pan operation")?
        .clone();
    let capture=engine.dispatch("render.capture",json!({
        "page_id":1,"width_px":848,"height_px":651,"backend":"cpu",
        "view":{"logical_width":848,"logical_height":651,"expected_view_epoch":panned["epoch"]}
    }))?.capture.ok_or("one-x PNG")?;
    independent_one_x(&capture)?;
    assert_eq!(engine.content(), &content);
    assert_eq!(engine.capture_stamp(), stamp);
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert!(
        engine.document().is_err(),
        "a display camera never becomes a Drawing"
    );
    engine.dispatch("document.save_sdf", json!({"path":output}))?;
    assert_eq!(fs::read(input)?, SOURCE);
    assert_eq!(fs::read(output)?, SOURCE);
    Ok(())
}

// Additive acceptance after the frozen first runtime RED; no future test hooks.
use bondline_commands::{Cancellation, Command};
use serde_json::Value;

type AnyResult<T> = Result<T, Box<dyn std::error::Error>>;

fn open_source(
    directory: &support::FileTestDirectory,
    name: &str,
    bytes: &[u8],
) -> AnyResult<Engine> {
    let input = directory.path(name);
    fs::write(&input, bytes)?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_sdf", json!({"path":input}))?;
    Ok(engine)
}

fn operation(engine: &mut Engine, params: Value) -> AnyResult<Value> {
    serde_json::to_value(engine.dispatch("ui.view", params)?)?
        .get("ui_view_operation")
        .cloned()
        .ok_or_else(|| "view operation missing".into())
}

fn prepared_extent(engine: &mut Engine) -> AnyResult<Option<Value>> {
    Ok(operation(engine, json!({"action":"inspect"}))?
        .get("prepared_sdf_extent")
        .cloned())
}

fn current_request(engine: &Engine) -> Value {
    json!({"page_id":1,"width_px":848,"height_px":651,"backend":"cpu",
      "view":{"logical_width":848,"logical_height":651,"expected_view_epoch":engine.view().epoch}})
}

fn current_capture(engine: &mut Engine) -> AnyResult<PngCapture> {
    engine
        .dispatch("render.capture", current_request(engine))?
        .capture
        .ok_or_else(|| "current-view PNG missing".into())
}

fn prepared_capture(engine: &mut Engine) -> AnyResult<bondline_commands::Prepared> {
    Ok(engine.prepare(Command::parse("render.capture", current_request(engine))?)?)
}

fn run_python(args: &[&str], input: &[u8]) -> AnyResult<Value> {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child = Process::new(python)
        .arg("-B")
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("independent consumer stdin")?
        .write_all(input)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(serde_json::from_slice(&output.stdout)?)
}

#[test]
fn sdf_camera_actions_before_extent_keep_authored_state_and_require_real_bounds_for_pan()
-> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let mut engine = open_source(&directory, "unprepared.sdf", SOURCE)?;
    engine.set_view_canvas_size(848, 651)?;
    let before = serde_json::to_value(engine.state())?;
    let stamp = engine.capture_stamp();
    let content = engine.content().clone();
    assert!(prepared_extent(&mut engine)?.is_none());
    for id in ["view.actual", "view.fit"] {
        assert!(
            engine
                .controls()
                .iter()
                .any(|control| control.id == id && control.enabled),
            "missing enabled {id}"
        );
    }
    operation(&mut engine, json!({"action":"actual"}))?;
    let epoch = engine.view().epoch;
    let same = operation(&mut engine, json!({"action":"actual"}))?;
    assert_eq!(same["changed"], false);
    assert_eq!(engine.view().epoch, epoch);
    assert_eq!(
        operation(&mut engine, json!({"action":"set_zoom","percent":125}))?["zoom_percent"],
        125.0
    );
    assert_eq!(
        operation(
            &mut engine,
            json!({"action":"magnify","width_px":848,"height_px":651})
        )?["zoom_percent"],
        156.25
    );
    assert_eq!(
        operation(
            &mut engine,
            json!({"action":"reduce","width_px":848,"height_px":651})
        )?["zoom_percent"],
        125.0
    );
    assert_eq!(
        engine
            .dispatch(
                "ui.view",
                json!({"action":"pan","center_x_pt":100,"center_y_pt":144})
            )
            .err()
            .ok_or("pan fabricated bounds")?
            .info()
            .code,
        "viewport_unavailable"
    );
    operation(&mut engine, json!({"action":"fit"}))?;
    for action in ["magnify", "reduce"] {
        assert_eq!(
            engine
                .dispatch(
                    "ui.view",
                    json!({"action":action,"width_px":848,"height_px":651})
                )
                .err()
                .ok_or("relative Fit fabricated extent")?
                .info()
                .code,
            "viewport_unavailable"
        );
    }
    assert_eq!(
        engine
            .dispatch(
                "ui.view",
                json!({"action":"pan","center_x_pt":0,"center_y_pt":0})
            )
            .err()
            .ok_or("Fit pan accepted")?
            .info()
            .code,
        "unsupported_content"
    );
    assert!(prepared_extent(&mut engine)?.is_none());
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert_eq!(engine.content(), &content);
    assert_eq!(engine.capture_stamp(), stamp);
    engine.dispatch(
        "document.close",
        json!({"expected_revision":1,"action":"request"}),
    )?;
    let closed = serde_json::to_value(engine.state())?;
    let view = engine.view();
    assert_eq!(
        engine
            .dispatch("ui.view", json!({"action":"actual"}))
            .err()
            .ok_or("close guard bypass")?
            .info()
            .code,
        "ui_busy"
    );
    assert_eq!(serde_json::to_value(engine.state())?, closed);
    assert_eq!(engine.view(), view);
    engine.dispatch(
        "document.close",
        json!({"expected_revision":1,"action":"cancel"}),
    )?;
    engine.set_ui_file_host(true);
    engine.dispatch("ui.file", json!({"expected_revision":1,"action":"open"}))?;
    let pending = serde_json::to_value(engine.state())?;
    assert_eq!(
        engine
            .dispatch("ui.view", json!({"action":"actual"}))
            .err()
            .ok_or("picker guard bypass")?
            .info()
            .code,
        "ui_busy"
    );
    assert_eq!(serde_json::to_value(engine.state())?, pending);
    Ok(())
}

#[test]
fn failed_and_cancelled_current_capture_never_prepares_extent_or_changes_state() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let mut engine = open_source(&directory, "failure.sdf", SOURCE)?;
    let before = serde_json::to_value(engine.state())?;
    let fit = engine.view();
    let error = engine
        .dispatch(
            "render.capture",
            json!({"page_id":1,"width_px":100,"height_px":100,"backend":"cpu",
        "view":{"logical_width":100,"logical_height":100,"expected_view_epoch":0}}),
        )
        .err()
        .ok_or("unreadable Fit reported success")?;
    assert_eq!(error.info().code, "unsupported_scale");
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert_eq!(engine.view(), fit);
    assert!(
        prepared_extent(&mut engine)?.is_none(),
        "a failed image installs nothing"
    );
    operation(&mut engine, json!({"action":"actual"}))?;
    let view = operation(&mut engine, json!({"action":"inspect"}))?;
    for late in [false, true] {
        let prepared = prepared_capture(&mut engine)?;
        let cancel = Cancellation::default();
        let error = if late {
            let completed = prepared.complete_with_cancel(&cancel)?;
            cancel.cancel();
            engine
                .finish(completed)
                .err()
                .ok_or("late cancellation accepted")?
        } else {
            cancel.cancel();
            prepared
                .complete_with_cancel(&cancel)
                .err()
                .ok_or("early cancellation accepted")?
        };
        assert_eq!(error.info().code, "cancelled");
        assert_eq!(operation(&mut engine, json!({"action":"inspect"}))?, view);
        assert_eq!(serde_json::to_value(engine.state())?, before);
        assert!(prepared_extent(&mut engine)?.is_none());
    }
    current_capture(&mut engine)?;
    let prepared = prepared_extent(&mut engine)?.ok_or("successful extent")?;
    let mut too_small = current_request(&engine);
    too_small["width_px"] = json!(8);
    too_small["height_px"] = json!(8);
    assert_eq!(
        engine
            .dispatch("render.capture", too_small)
            .err()
            .ok_or("tiny physical labels accepted")?
            .info()
            .code,
        "unsupported_scale"
    );
    assert_eq!(
        prepared_extent(&mut engine)?,
        Some(prepared),
        "failed work cannot replace existing prepared extent"
    );
    assert_eq!(serde_json::to_value(engine.state())?, before);
    Ok(())
}

#[test]
fn stale_source_view_close_and_foreign_finish_return_historical_image_without_cache_install()
-> TestResult {
    let directory = support::FileTestDirectory::new()?;
    for scenario in [
        "view_aba",
        "source_undo",
        "identical_open",
        "close_aba",
        "close_pending",
        "foreign",
    ] {
        let mut engine = open_source(&directory, &format!("{scenario}.sdf"), SOURCE)?;
        operation(&mut engine, json!({"action":"actual"}))?;
        let completed = prepared_capture(&mut engine)?.complete()?;
        assert!(prepared_extent(&mut engine)?.is_none());
        let mut foreign = None;
        match scenario {
            "view_aba" => {
                operation(&mut engine, json!({"action":"set_zoom","percent":125}))?;
                operation(&mut engine, json!({"action":"actual"}))?;
            }
            "source_undo" => {
                engine.dispatch("sdf.field.set_value",json!({"expected_revision":1,"record_id":1,"field_index":0,"value_lines":["metadata-only change"]}))?;
                engine.dispatch("history.undo", json!({}))?;
                assert_eq!(engine.content().sdf()?.as_bytes(), SOURCE);
            }
            "identical_open" => {
                engine.dispatch(
                    "document.open_sdf",
                    json!({"path":directory.path(&format!("{scenario}.sdf"))}),
                )?;
            }
            "close_aba" | "close_pending" => {
                engine.dispatch(
                    "document.close",
                    json!({"expected_revision":1,"action":"request"}),
                )?;
                if scenario == "close_aba" {
                    engine.dispatch(
                        "document.close",
                        json!({"expected_revision":1,"action":"cancel"}),
                    )?;
                }
            }
            "foreign" => {
                let mut other = open_source(&directory, "same-bytes-foreign.sdf", SOURCE)?;
                operation(&mut other, json!({"action":"actual"}))?;
                assert_eq!(other.content(), engine.content());
                assert_eq!(other.state().revision, engine.state().revision);
                assert_eq!(other.view(), engine.view());
                assert_ne!(other.capture_stamp(), engine.capture_stamp());
                foreign = Some(other);
            }
            _ => return Err("unknown fixed lifecycle scenario".into()),
        }
        let original = serde_json::to_value(engine.state())?;
        let target = foreign.as_mut().unwrap_or(&mut engine);
        let before = serde_json::to_value(target.state())?;
        let camera = operation(target, json!({"action":"inspect"}))?;
        let reply = target.finish(completed)?;
        assert_eq!(reply.revision, 1);
        assert!(!reply.changed && reply.document.is_none());
        assert!(reply.capture.is_some());
        assert_eq!(reply.close, target.state().close);
        assert_eq!(reply.close_epoch, target.state().close_epoch);
        assert!(
            prepared_extent(target)?.is_none(),
            "stale {scenario} installed an extent"
        );
        assert_eq!(operation(target, json!({"action":"inspect"}))?, camera);
        assert_eq!(serde_json::to_value(target.state())?, before);
        assert_eq!(serde_json::to_value(engine.state())?, original);
    }
    Ok(())
}

#[test]
fn tall_current_capture_uses_exact_extent_and_validates_every_off_camera_record() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let bytes = SOURCE.repeat(6);
    assert_eq!(bytes.len(), 4440);
    assert_eq!(
        bytes.windows(5).filter(|line| *line == b"$$$$\n").count(),
        12
    );
    let mut engine = open_source(&directory, "tall.sdf", &bytes)?;
    let actual = fs::read(directory.path("tall.sdf"))?;
    let oracle = format!("{}/../io/tests/mol_oracle.py", env!("CARGO_MANIFEST_DIR"));
    let first = run_python(&[&oracle], actual.get(..371).ok_or("first original MOL")?)?;
    let last = run_python(
        &[&oracle],
        actual.get(4110..4413).ok_or("last original MOL")?,
    )?;
    assert_eq!(first["formula"], "C2H6O");
    assert_eq!(last["formula"], "CH5N");
    assert_eq!(first["atoms"][0]["position"], json!([-1.25, 0.5, 0.0]));
    assert_eq!(last["atoms"][0]["position"], json!([1.5, -1.5, 0.0]));
    assert_eq!(last["atoms"][1]["hydrogens"], 2);
    let content = engine.content().clone();
    operation(&mut engine, json!({"action":"actual"}))?;
    current_capture(&mut engine)?;
    let size = prepared_extent(&mut engine)?.ok_or("tall prepared size")?;
    assert_eq!(
        size,
        json!({"width_pt":210.0*72.0/25.4,"height_pt":1368.0}),
        "12 source-coordinate rows:72+12*(36+72)"
    );
    let mut images = Vec::new();
    for record in [1, 7, 12] {
        let top = 72 + (record - 1) * 108;
        operation(
            &mut engine,
            json!({"action":"pan","center_x_pt":100,"center_y_pt":top+18}),
        )?;
        let capture = current_capture(&mut engine)?;
        assert_eq!(capture.label_em_px, Some(12.0));
        images.push(json!({"record":record,"capture":capture}));
    }
    run_python(
        &[
            "-c",
            r#"
import base64,io,json,sys,PIL
from PIL import Image
assert PIL.__version__=='12.3.0'
for item in json.load(sys.stdin):
 with Image.open(io.BytesIO(base64.b64decode(item['capture']['png_base64'],validate=True))) as im:
  assert im.format=='PNG' and im.mode=='RGBA' and im.size==(848,651)
  points=[(414,344),(432,326)] if item['record']%2 else [(414,326)]
  for x,y in points:
   assert any(v==(0,0,0,255) for v in im.crop((x-1,y-1,x+2,y+2)).get_flattened_data()),('missing row',item['record'],x,y)
print(json.dumps({'source_rows':[1,7,12],'physical_size':[848,651]}))
"#,
        ],
        &serde_json::to_vec(&images)?,
    )?;
    assert_eq!(
        engine.state().collection.ok_or("collection")?.record_count,
        12
    );
    assert_eq!(engine.content(), &content);
    assert_eq!(fs::read(directory.path("tall.sdf"))?, bytes);
    // Keep a real first-row camera while replacing only the last source record.
    operation(
        &mut engine,
        json!({"action":"pan","center_x_pt":100,"center_y_pt":90}),
    )?;
    let mut bad = bytes.clone();
    let needle = b"    0.0000    0.0000    0.0000 N   0";
    let at = bad
        .windows(needle.len())
        .rposition(|line| line == needle)
        .ok_or("last original N")?;
    let symbol = bad.get_mut(at + 31).ok_or("literal N byte")?;
    assert_eq!(*symbol, b'N');
    *symbol = b'P';
    assert_eq!(bad.iter().zip(&bytes).filter(|(a, b)| a != b).count(), 1);
    let bad_path = directory.path("last-unresolved.sdf");
    fs::write(&bad_path, &bad)?;
    engine.dispatch("document.open_sdf", json!({"path":bad_path}))?;
    assert_eq!(
        engine
            .view()
            .center
            .ok_or("retained first-row camera")?
            .y_pt(),
        90.0
    );
    let before = serde_json::to_value(engine.state())?;
    let camera = operation(&mut engine, json!({"action":"inspect"}))?;
    let error = engine
        .dispatch("render.capture", current_request(&engine))
        .err()
        .ok_or("off-camera unsupported chemistry skipped")?;
    assert_eq!(error.info().code, "unsupported_content");
    assert!(error.info().message.contains("record 12"));
    assert!(prepared_extent(&mut engine)?.is_none());
    assert_eq!(operation(&mut engine, json!({"action":"inspect"}))?, camera);
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert_eq!(fs::read(directory.path("last-unresolved.sdf"))?, bad);
    let last = run_python(&[&oracle], bad.get(4110..4413).ok_or("last P MOL")?)?;
    assert_eq!(
        last["atoms"][1]["element"], 15,
        "unsupported preview input is a real P record"
    );
    Ok(())
}

#[test]
fn closed_optional_capture_view_keeps_legacy_bytes_and_rejects_bad_or_stale_contexts() -> TestResult
{
    let directory = support::FileTestDirectory::new()?;
    let mut engine = open_source(&directory, "legacy.sdf", SOURCE)?;
    let legacy = json!({"page_id":1,"width_px":1024,"height_px":1024,"backend":"cpu"});
    let first = engine
        .dispatch("render.capture", legacy.clone())?
        .capture
        .ok_or("legacy PNG")?
        .png_bytes()?;
    let mut null = legacy.clone();
    null["view"] = Value::Null;
    assert_eq!(
        serde_json::to_value(Command::parse("render.capture", null.clone())?)?["params"],
        legacy
    );
    assert_eq!(
        engine
            .dispatch("render.capture", null)?
            .capture
            .ok_or("null legacy PNG")?
            .png_bytes()?,
        first
    );
    assert!(prepared_extent(&mut engine)?.is_none());
    operation(&mut engine, json!({"action":"set_zoom","percent":200}))?;
    assert_eq!(
        engine
            .dispatch("render.capture", legacy)?
            .capture
            .ok_or("unchanged legacy PNG")?
            .png_bytes()?,
        first
    );
    assert!(
        prepared_extent(&mut engine)?.is_none(),
        "legacy success does not prepare a current-view extent"
    );
    let before = serde_json::to_value(engine.state())?;
    let camera = operation(&mut engine, json!({"action":"inspect"}))?;
    let epoch = engine.view().epoch;
    for view in [
        json!(false),
        json!([]),
        json!({}),
        json!({"logical_width":848,"logical_height":651}),
        json!({"logical_width":0,"logical_height":651,"expected_view_epoch":epoch}),
        json!({"logical_width":848,"logical_height":100001,"expected_view_epoch":epoch}),
        json!({"logical_width":848.5,"logical_height":651,"expected_view_epoch":epoch}),
        json!({"logical_width":848,"logical_height":651,"expected_view_epoch":-1}),
        json!({"logical_width":848,"logical_height":651,"expected_view_epoch":0.5}),
        json!({"logical_width":848,"logical_height":651,"expected_view_epoch":epoch,"center_x_pt":0}),
    ] {
        let mut params = current_request(&engine);
        params["view"] = view;
        assert_eq!(
            engine
                .dispatch("render.capture", params)
                .err()
                .ok_or("invalid nested view accepted")?
                .info()
                .code,
            "invalid_params"
        );
        assert_eq!(serde_json::to_value(engine.state())?, before);
        assert_eq!(operation(&mut engine, json!({"action":"inspect"}))?, camera);
    }
    for stale in [epoch - 1, u64::MAX] {
        let mut params = current_request(&engine);
        params["view"]["expected_view_epoch"] = json!(stale);
        assert_eq!(
            engine
                .prepare(Command::parse("render.capture", params)?)
                .err()
                .ok_or("stale view prepared")?
                .info()
                .code,
            "stale_view"
        );
    }
    assert!(prepared_extent(&mut engine)?.is_none());
    let metadata = bondline_commands::registry()
        .into_iter()
        .find(|command| command.name == "render.capture")
        .ok_or("capture registry")?;
    assert!(metadata.read_only);
    let schema = serde_json::to_value(metadata.input_schema)?;
    assert_eq!(schema["additionalProperties"], false);
    let required = schema["required"]
        .as_array()
        .ok_or("legacy required keys")?;
    assert_eq!(required.len(), 4);
    for name in ["page_id", "width_px", "height_px", "backend"] {
        assert!(required.contains(&json!(name)));
    }
    let nested = &schema["$defs"]["CaptureView"];
    assert_eq!(nested["additionalProperties"], false);
    let required = nested["required"]
        .as_array()
        .ok_or("closed view required keys")?;
    assert_eq!(required.len(), 3);
    for name in ["logical_width", "logical_height", "expected_view_epoch"] {
        assert!(required.contains(&json!(name)));
    }
    for name in ["logical_width", "logical_height"] {
        assert_eq!(nested["properties"][name]["minimum"], 1);
        assert_eq!(nested["properties"][name]["maximum"], 100000);
    }
    Ok(())
}

#[test]
fn prepared_sdf_extent_never_grants_drawing_save_or_native_receipt_authority() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let mut engine = open_source(&directory, "authority.sdf", SOURCE)?;
    operation(&mut engine, json!({"action":"actual"}))?;
    current_capture(&mut engine)?;
    let extent = prepared_extent(&mut engine)?.ok_or("prepared SDF")?;
    assert_eq!(extent, serde_json::to_value(bondline_model::PageSize::A4)?);
    let before = serde_json::to_value(engine.state())?;
    for (command, params) in [
        (
            "bond.create",
            json!({"page_id":1,"begin_x_pt":72,"begin_y_pt":72,"end_x_pt":108,"end_y_pt":72}),
        ),
        (
            "page.set_size",
            json!({"page_id":1,"width_pt":612,"height_pt":792}),
        ),
        (
            "document.save_cdxml",
            json!({"path":directory.path("lossy.cdxml")}),
        ),
        (
            "document.save_mol",
            json!({"path":directory.path("lossy.mol")}),
        ),
    ] {
        assert_eq!(
            engine
                .dispatch(command, params)
                .err()
                .ok_or("preview granted source authority")?
                .info()
                .code,
            "unsupported_content"
        );
        assert_eq!(serde_json::to_value(engine.state())?, before);
    }
    assert!(!directory.path("lossy.cdxml").exists() && !directory.path("lossy.mol").exists());
    let output = directory.path("whole-source-save.sdf");
    let save = engine.prepare(Command::parse("document.save_sdf", json!({"path":output}))?)?;
    current_capture(&mut engine)?;
    assert_eq!(
        engine
            .prepare(Command::parse("document.save_sdf", json!({"path":output}))?)
            .err()
            .ok_or("capture released file reservation")?
            .info()
            .code,
        "busy"
    );
    engine.finish(save.complete()?)?;
    assert_eq!(fs::read(output)?, SOURCE);
    let epoch = engine.view().epoch;
    engine.dispatch("sdf.field.set_value",json!({"expected_revision":1,"record_id":1,"field_index":0,"value_lines":["source-only metadata changed"]}))?;
    assert_eq!(engine.view().epoch, epoch);
    assert!(
        prepared_extent(&mut engine)?.is_none(),
        "geometry equality never validates another source's cache"
    );
    assert_eq!(engine.state().undo_depth, 1);
    assert!(engine.state().file.dirty);
    engine.set_ui_file_host(true);
    let requested =
        engine.dispatch("ui.file", json!({"expected_revision":2,"action":"save_as"}))?;
    let id = requested.ui_file.request.ok_or("file intent")?.id;
    engine.claim_ui_file(id)?;
    let before = serde_json::to_value(engine.state())?;
    assert_eq!(
        engine
            .prepare_ui_file(
                id,
                Command::parse("render.capture", current_request(&engine))?
            )
            .err()
            .ok_or("capture acknowledged a native save")?
            .info()
            .code,
        "invalid_params"
    );
    assert_eq!(serde_json::to_value(engine.state())?, before);
    engine.dispatch(
        "ui.file",
        json!({"expected_revision":2,"action":"cancel","request_id":id}),
    )?;
    let smiles = directory.path("foreign-kind.smi");
    fs::write(&smiles, b"CC\n")?;
    engine.dispatch(
        "document.open_smiles",
        json!({"path":smiles,"discard_unsaved":true}),
    )?;
    let before = serde_json::to_value(engine.state())?;
    assert_eq!(
        engine
            .dispatch("ui.view", json!({"action":"actual"}))
            .err()
            .ok_or("SMILES gained a camera")?
            .info()
            .code,
        "unsupported_content"
    );
    assert_eq!(
        engine
            .dispatch("render.capture", current_request(&engine))
            .err()
            .ok_or("SMILES gained projection")?
            .info()
            .code,
        "unsupported_content"
    );
    assert!(prepared_extent(&mut engine)?.is_none());
    assert_eq!(serde_json::to_value(engine.state())?, before);
    let native = directory.path("native-empty.cdxml");
    Engine::default().dispatch("document.save_cdxml", json!({"path":native}))?;
    let opaque = String::from_utf8(fs::read(native)?)?.replacen(
        "<CDXML",
        "<CDXML VendorUnknown=\"must survive\"",
        1,
    );
    let opaque_path = directory.path("opaque.cdxml");
    fs::write(&opaque_path, opaque.as_bytes())?;
    engine.dispatch("document.open_cdxml", json!({"path":opaque_path}))?;
    let before = serde_json::to_value(engine.state())?;
    assert_eq!(
        engine
            .dispatch("ui.view", json!({"action":"actual"}))
            .err()
            .ok_or("opaque Drawing bypassed mapped guard")?
            .info()
            .code,
        "unsupported_content"
    );
    assert_eq!(
        engine
            .dispatch("render.capture", current_request(&engine))
            .err()
            .ok_or("current-view bypassed mapped source guard")?
            .info()
            .code,
        "unsupported_content"
    );
    assert_eq!(serde_json::to_value(engine.state())?, before);
    let opaque_copy = directory.path("opaque-copy.cdxml");
    engine.dispatch("document.save_cdxml", json!({"path":opaque_copy}))?;
    assert_eq!(fs::read(opaque_copy)?, opaque.as_bytes());
    Ok(())
}
