//! Independent headless acceptance of the real native frame path; no native-window claim.
use super::*;
use crate::{BondlineApp, DrawingState, FileWorkflow, configure_context};
use eframe::egui::{self, Event, PointerButton};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    fs,
    process::Command,
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver, Sender},
    },
    time::{Duration, Instant},
};
#[path = "../../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse only the existing isolated test directory")]
mod support;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const LIMIT: Duration = Duration::from_secs(10);
const SOURCE: &[u8] =
    include_bytes!("../../../io/tests/fixtures/sdf-preview/valence-absent-mixed.sdf");

struct Fixture {
    app: BondlineApp,
    context: egui::Context,
    directory: support::FileTestDirectory,
    input: std::path::PathBuf,
    textures: HashMap<egui::TextureId, egui::ColorImage>,
}
impl Fixture {
    fn new() -> TestResult<Self> {
        let directory = support::FileTestDirectory::new()?;
        let input = directory.path("preview-input-λ.sdf");
        fs::write(&input, SOURCE)?;
        let mut engine = Engine::default();
        engine.set_ui_file_host(true);
        engine.dispatch("document.open_sdf", json!({"path":input}))?;
        let engine = Arc::new(Mutex::new(engine));
        let app = BondlineApp {
            smoke_frames: None,
            engine: Arc::clone(&engine),
            executor: CommandExecutor::new(engine),
            control: None,
            error: None,
            drawing: DrawingState::default(),
            files: FileWorkflow::new(directory.path(".")),
            preview: PreviewWorkflow::default(),
            sdf_fields: crate::sdf_fields::SdfFieldsWorkflow::default(),
            smiles_records: crate::smiles_records::SmilesRecordsWorkflow::default(),
        };
        let context = egui::Context::default();
        configure_context(&context);
        Ok(Self {
            app,
            context,
            directory,
            input,
            textures: HashMap::new(),
        })
    }
    fn state(&self) -> TestResult<Value> {
        Ok(serde_json::to_value(
            self.app.engine.lock().map_err(|_| "engine lock")?.state(),
        )?)
    }
    fn command(&self, name: &str, params: Value) -> TestResult {
        self.app
            .engine
            .lock()
            .map_err(|_| "engine lock")?
            .dispatch(name, params)?;
        Ok(())
    }
    fn frame_at(
        &mut self,
        size: egui::Vec2,
        origin: egui::Pos2,
        dpi: f32,
        events: Vec<Event>,
    ) -> TestResult<egui::FullOutput> {
        let mut input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(origin, size)),
            events,
            focused: true,
            ..Default::default()
        };
        input
            .viewports
            .entry(egui::ViewportId::ROOT)
            .or_default()
            .native_pixels_per_point = Some(dpi);
        let context = self.context.clone();
        let output = context.run(input, |context| self.app.update_workspace(context));
        for (id, delta) in &output.textures_delta.set {
            if let egui::ImageData::Color(image) = &delta.image {
                if let Some([x, y]) = delta.pos {
                    let current = self.textures.get_mut(id).ok_or("patch without image")?;
                    for py in 0..image.height() {
                        for px in 0..image.width() {
                            let index = (y + py) * current.width() + x + px;
                            *current
                                .pixels
                                .get_mut(index)
                                .ok_or("texture patch bounds")? =
                                image.pixels[py * image.width() + px];
                        }
                    }
                } else {
                    self.textures.insert(*id, image.as_ref().clone());
                }
            }
        }
        for id in &output.textures_delta.free {
            self.textures.remove(id);
        }
        Ok(output)
    }
    fn frame(&mut self, events: Vec<Event>) -> TestResult<egui::FullOutput> {
        self.frame_at(egui::vec2(1600.0, 1400.0), egui::Pos2::ZERO, 1.0, events)
    }
    fn ready_at(
        &mut self,
        size: egui::Vec2,
        origin: egui::Pos2,
        dpi: f32,
    ) -> TestResult<egui::FullOutput> {
        let deadline = Instant::now() + LIMIT;
        loop {
            let output = self.frame_at(size, origin, dpi, vec![])?;
            if self.app.preview.ready.is_some() && !self.app.preview.is_busy() {
                return Ok(output);
            }
            if let Some((_, error)) = &self.app.preview.error {
                return Err(format!("preview failed: {error}").into());
            }
            if Instant::now() >= deadline {
                return Err("preview did not become ready within ten seconds".into());
            }
            std::thread::sleep(Duration::from_millis(2));
        }
    }
    fn ready(&mut self) -> TestResult<egui::FullOutput> {
        self.ready_at(egui::vec2(1600.0, 1400.0), egui::Pos2::ZERO, 1.0)
    }
    fn key(&self) -> TestResult<PreviewKey> {
        Ok(self
            .app
            .preview
            .ready
            .as_ref()
            .ok_or("ready preview")?
            .key
            .clone())
    }
    fn current_source(&self) -> TestResult<Vec<u8>> {
        Ok(self
            .app
            .engine
            .lock()
            .map_err(|_| "engine lock")?
            .content()
            .sdf()?
            .as_bytes()
            .to_vec())
    }
    fn no_ready_or_hits(&self) {
        assert!(self.app.preview.ready.is_none());
        assert!(self.app.drawing.hover_target().is_none());
        assert!(self.app.drawing.gesture.is_none());
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        if let Some(job) = self.app.preview.job.take() {
            job.cancel.cancel();
            // Test-only cleanup is outside the UI frame; finite gates are released on Drop.
            let _ = job.worker.join();
        }
    }
}
fn labels(output: &egui::FullOutput) -> String {
    output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::Shape::Text(text) => Some(text.galley.text()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}
fn has_texture(output: &egui::FullOutput, context: &egui::Context, id: egui::TextureId) -> bool {
    context.tessellate(output.shapes.clone(),output.pixels_per_point).iter().any(|item|matches!(&item.primitive,egui::epaint::Primitive::Mesh(mesh) if mesh.texture_id==id))
}
fn assert_painted(fixture: &Fixture, output: &egui::FullOutput) -> TestResult {
    let ready = fixture.app.preview.ready.as_ref().ok_or("ready preview")?;
    let mut count = 0;
    for item in fixture
        .context
        .tessellate(output.shapes.clone(), output.pixels_per_point)
    {
        if let egui::epaint::Primitive::Mesh(mesh) = item.primitive
            && mesh.texture_id == ready.texture.id()
        {
            count += 1;
            assert_eq!(mesh.vertices.len(), 4);
            assert_eq!(mesh.indices.len(), 6);
            let expected = ready.key.rect();
            for point in [
                expected.left_top(),
                expected.right_top(),
                expected.left_bottom(),
                expected.right_bottom(),
            ] {
                assert!(mesh.vertices.iter().any(|v| v.pos.distance(point) < 0.001));
            }
            for uv in [
                egui::pos2(0.0, 0.0),
                egui::pos2(1.0, 0.0),
                egui::pos2(0.0, 1.0),
                egui::pos2(1.0, 1.0),
            ] {
                assert!(
                    mesh.vertices
                        .iter()
                        .any(|v| v.uv == uv && v.color == egui::Color32::WHITE)
                );
            }
            assert!(item.clip_rect.contains_rect(expected));
        }
    }
    assert_eq!(
        count, 1,
        "uploaded preview must actually be painted exactly once"
    );
    Ok(())
}
fn registered(fixture: &Fixture, key: &PreviewKey) -> TestResult<PngCapture> {
    let result = fixture
        .app
        .executor
        .prepare_ui_capture(&key.stamp, key.request())
        .map_err(|e| e.message)?
        .complete_blocking(&Cancellation::default())
        .map_err(|e| e.message)?;
    Ok(result.capture.ok_or("registered capture")?)
}
fn independent_pixels(fixture: &Fixture) -> TestResult {
    let ready = fixture.app.preview.ready.as_ref().ok_or("ready")?;
    let image = fixture
        .textures
        .get(&ready.texture.id())
        .ok_or("actual uploaded ColorImage")?;
    assert_eq!(image.size, ready.key.pixels.map(|v| v as usize));
    let rgba = image
        .pixels
        .iter()
        .flat_map(|p| p.to_array())
        .collect::<Vec<_>>();
    assert!(
        rgba.chunks_exact(4).all(|pixel| pixel[3] == 255),
        "this exact RGBA test is for opaque registry scenes"
    );
    let actual = fixture.directory.path("uploaded-preview.rgba");
    let encoded = fixture.directory.path("registered-preview.png");
    fs::write(&actual, &rgba)?;
    fs::write(&encoded, registered(fixture, &ready.key)?.png_bytes()?)?;
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let result=Command::new(python).args(["-B","-c",r#"
import sys,PIL
from pathlib import Path
from PIL import Image
assert PIL.__version__=='12.3.0'
with Image.open(sys.argv[1]) as im:
 assert im.mode=='RGBA' and im.format=='PNG'
 assert im.tobytes()==Path(sys.argv[2]).read_bytes(),'egui upload differs from unchanged registered PNG'
 w,h=im.size
 assert max(w,h)<=2048
 m=min(32,w//8,h//8);aw,ah=w-2*m,h-2*m
 a4w,a4h=210*72/25.4,297*72/25.4
 scale=min(aw/(a4w/a4h),ah)
 pw,ph=round(a4w/a4h*scale),round(scale)
 ox,oy=(w-pw)//2,(h-ph)//2
 def p(x,y):return (round(ox+x/a4w*pw),round(oy+y/a4h*ph))
 def black(box):return sum(c==(0,0,0,255) for c in im.crop(box).get_flattened_data())
 # Literal source-derived CCO: (72,108),(108,108),(108,72); CN:(108,216),(72,180).
 for xy in [(90,108),(108,90),(81,189)]:
  x,y=p(*xy);assert black((x-1,y-1,x+2,y+2))>0,('missing literal record chord',xy)
 for bounds in [(40,58,150,125),(35,165,145,235)]:
  x0,y0=p(bounds[0],bounds[1]);x1,y1=p(bounds[2],bounds[3]);assert black((x0,y0,x1,y1))>40
"#]).arg(encoded).arg(actual).output()?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(())
}

struct Gate {
    release: Option<Sender<()>>,
    queued: Receiver<()>,
}
impl Gate {
    fn queued(&self) -> TestResult {
        self.queued.recv_timeout(LIMIT)?;
        Ok(())
    }
    fn release(mut self) {
        if let Some(sender) = self.release.take() {
            let _ = sender.send(());
        }
    }
}
impl Drop for Gate {
    fn drop(&mut self) {
        if let Some(sender) = self.release.take() {
            let _ = sender.send(());
        }
    }
}
fn queued_real_job(fixture: &mut Fixture, corrupt: bool) -> TestResult<Gate> {
    let key = fixture.key()?;
    assert!(fixture.app.preview.job.is_none());
    let work = fixture
        .app
        .executor
        .prepare_ui_capture(&key.stamp, key.request())
        .map_err(|e| e.message)?;
    fixture.app.preview.ready = None;
    fixture.app.preview.error = None;
    fixture.app.preview.pending = None;
    let cancel = Cancellation::default();
    let worker_cancel = cancel.clone();
    let worker_key = key.clone();
    let context = fixture.context.clone();
    let (sender, result) = mpsc::sync_channel(1);
    let (ready_sender, queued) = mpsc::channel();
    let (release, wait) = mpsc::channel();
    let worker = std::thread::spawn(move || {
        let output = work
            .complete_blocking(&worker_cancel)
            .map_err(|e| e.message)
            .and_then(|result| {
                result
                    .capture
                    .ok_or_else(|| "registered capture absent".to_owned())
            })
            .and_then(|mut capture| {
                if corrupt {
                    capture.png_base64.replace_range(..1, "!");
                }
                decode(capture, &worker_key, &worker_cancel)
            });
        let _ = sender.send(output);
        let _ = ready_sender.send(());
        let _ = wait.recv_timeout(LIMIT);
        context.request_repaint();
    });
    fixture.app.preview.job = Some(PreviewJob {
        key,
        cancel,
        result,
        worker,
    });
    Ok(Gate {
        release: Some(release),
        queued,
    })
}

#[test]
fn independent_real_frame_uploads_and_paints_complete_registered_sdf_without_authoring_targets()
-> TestResult {
    let mut fixture = Fixture::new()?;
    let before = fixture.state()?;
    let output = fixture.ready()?;
    assert_painted(&fixture, &output)?;
    independent_pixels(&fixture)?;
    assert_eq!(fixture.state()?, before);
    assert_eq!(fixture.current_source()?, SOURCE);
    let text = labels(&output);
    assert!(text.contains("SDF collection: 2 records"));
    assert!(text.contains("Read-only chemical preview"));
    assert!(!text.contains("Chemical preview is unavailable"));
    let rect = fixture.key()?.rect();
    let from = rect.center();
    let to = from + egui::vec2(80.0, 30.0);
    fixture.frame(vec![
        Event::PointerMoved(from),
        Event::PointerButton {
            pos: from,
            button: PointerButton::Primary,
            pressed: true,
            modifiers: egui::Modifiers::NONE,
        },
        Event::PointerMoved(to),
        Event::PointerButton {
            pos: to,
            button: PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        },
    ])?;
    assert!(fixture.app.drawing.hover_target().is_none());
    assert!(fixture.app.drawing.gesture.is_none());
    assert_eq!(fixture.state()?, before);
    assert_eq!(fixture.current_source()?, SOURCE);
    Ok(())
}

#[test]
fn independent_field_history_and_identical_reopen_invalidate_full_preview_identity() -> TestResult {
    let mut fixture = Fixture::new()?;
    fixture.ready()?;
    let mut prior = fixture.key()?;
    for step in 0..4 {
        let revision = fixture.state()?["revision"].as_u64().ok_or("revision")?;
        match step {
            0=>fixture.command("sdf.field.set_value",json!({"expected_revision":revision,"record_id":1,"field_index":0,"value_lines":["new metadata"]}))?,
            1=>fixture.command("history.undo",json!({}))?,
            2=>fixture.command("history.redo",json!({}))?,
            _=>fixture.command("document.open_sdf",json!({"path":fixture.input,"discard_unsaved":true}))?,
        }
        let old_id = fixture
            .app
            .preview
            .ready
            .as_ref()
            .ok_or("old ready")?
            .texture
            .id();
        let output = fixture.frame(vec![])?;
        fixture.no_ready_or_hits();
        assert!(!has_texture(&output, &fixture.context, old_id));
        assert!(fixture.app.preview.error.is_none());
        fixture.ready()?;
        let key = fixture.key()?;
        assert_ne!(key, prior);
        prior = key;
        if step == 1 || step == 3 {
            assert_eq!(fixture.current_source()?, SOURCE);
        }
    }
    Ok(())
}

#[test]
fn independent_resize_origin_and_capped_dpi_clear_stale_texture_and_keep_uniform_geometry()
-> TestResult {
    let mut fixture = Fixture::new()?;
    fixture.ready()?;
    let mut old = fixture.key()?;
    for (size, origin, dpi) in [
        (egui::vec2(1500.0, 1400.0), egui::Pos2::ZERO, 1.0),
        (egui::vec2(1500.0, 1400.0), egui::pos2(11.0, 17.0), 1.0),
        (egui::vec2(1500.0, 1400.0), egui::pos2(11.0, 17.0), 2.0),
        (egui::vec2(1500.0, 1400.0), egui::pos2(11.0, 17.0), 4.0),
    ] {
        let id = fixture
            .app
            .preview
            .ready
            .as_ref()
            .ok_or("ready")?
            .texture
            .id();
        let output = fixture.frame_at(size, origin, dpi, vec![])?;
        fixture.no_ready_or_hits();
        assert!(!has_texture(&output, &fixture.context, id));
        fixture.ready_at(size, origin, dpi)?;
        let key = fixture.key()?;
        assert_ne!(key, old);
        assert!(key.pixels.iter().all(|v| *v <= 2048));
        let logical = key.rect().size();
        let cross = (key.pixels[0] as f32 * logical.y - key.pixels[1] as f32 * logical.x).abs();
        assert!(
            cross <= logical.x + logical.y,
            "per-axis clamp distorted capture aspect"
        );
        if dpi == 4.0 {
            assert_eq!(
                key.pixels, old.pixels,
                "DPI2 and4 both use same uniform cap here"
            );
        }
        old = key;
    }
    Ok(())
}

#[test]
fn independent_same_device_pixels_do_not_make_tiny_logical_chemical_labels_readable() -> TestResult
{
    let fixture = Fixture::new()?;
    let engine = fixture.app.engine.lock().map_err(|_| "engine lock")?;
    let large = PreviewKey::new(
        &engine,
        egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800.0, 800.0)),
        1.0,
        2048,
    )?;
    let small = PreviewKey::new(
        &engine,
        egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400.0, 400.0)),
        2.0,
        2048,
    )?;
    drop(engine);
    assert_eq!(large.pixels, small.pixels);
    assert_ne!(large, small);
    let capture = registered(&fixture, &large)?;
    assert!(large.labels_readable(capture.label_em_px));
    assert_eq!(
        decode(capture, &large, &Cancellation::default())?.size,
        [800, 800]
    );
    let capture = registered(&fixture, &small)?;
    assert!(capture.label_em_px.is_some_and(|v| v >= 8.0));
    assert!(!small.labels_readable(capture.label_em_px));
    assert!(decode(capture, &small, &Cancellation::default()).is_err());
    assert_eq!(fixture.current_source()?, SOURCE);
    Ok(())
}

#[test]
fn independent_queued_success_and_decode_error_cannot_install_after_source_changes() -> TestResult {
    for corrupt in [false, true] {
        let mut fixture = Fixture::new()?;
        fixture.ready()?;
        let old = fixture.key()?;
        let gate = queued_real_job(&mut fixture, corrupt)?;
        gate.queued()?;
        assert!(fixture.app.preview.is_busy());
        assert!(
            fixture
                .app
                .preview
                .job
                .as_ref()
                .is_some_and(|j| !j.worker.is_finished())
        );
        fixture.command(
            "sdf.field.set_value",
            json!({"expected_revision":1,"record_id":1,"field_index":0,"value_lines":["changed"]}),
        )?;
        fixture.frame(vec![])?;
        fixture.no_ready_or_hits();
        assert!(fixture.app.preview.error.is_none());
        assert!(fixture.app.preview.is_busy());
        gate.release();
        fixture.ready()?;
        assert_ne!(fixture.key()?, old);
        assert!(fixture.app.preview.error.is_none());
    }
    // A current corrupted real payload becomes a keyed error; a new source clears it.
    let mut fixture = Fixture::new()?;
    fixture.ready()?;
    let gate = queued_real_job(&mut fixture, true)?;
    gate.queued()?;
    gate.release();
    let deadline = Instant::now() + LIMIT;
    loop {
        let output = fixture.frame(vec![])?;
        if fixture.app.preview.error.is_some() {
            assert!(labels(&output).contains("Preview unavailable:"));
            break;
        }
        if Instant::now() >= deadline {
            return Err("decode error did not retire".into());
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    fixture.no_ready_or_hits();
    assert!(!fixture.app.preview.is_busy());
    fixture.command("document.open_sdf", json!({"path":fixture.input}))?;
    fixture.frame(vec![])?;
    assert!(fixture.app.preview.error.is_none());
    fixture.ready()?;
    Ok(())
}

#[test]
fn independent_close_epoch_and_queued_decode_hold_native_finalization_until_worker_retires()
-> TestResult {
    let mut fixture = Fixture::new()?;
    fixture.ready()?;
    let old = fixture.key()?;
    let gate = queued_real_job(&mut fixture, false)?;
    gate.queued()?;
    fixture.command(
        "document.close",
        json!({"expected_revision":1,"action":"request"}),
    )?;
    fixture.frame(vec![])?;
    fixture.no_ready_or_hits();
    assert_eq!(fixture.state()?["close"], "approved");
    assert!(fixture.app.preview.is_busy());
    fixture.command(
        "document.close",
        json!({"expected_revision":1,"action":"cancel"}),
    )?;
    fixture.frame(vec![])?;
    assert_eq!(fixture.state()?["close"], "open");
    assert_ne!(
        fixture
            .app
            .preview
            .desired
            .as_ref()
            .ok_or("new desired")?
            .close_epoch,
        old.close_epoch
    );
    assert!(fixture.app.preview.is_busy());
    fixture.no_ready_or_hits();
    fixture.command(
        "document.close",
        json!({"expected_revision":1,"action":"request"}),
    )?;
    fixture.frame(vec![])?;
    assert_eq!(fixture.state()?["close"], "approved");
    assert!(fixture.app.preview.is_busy());
    gate.release();
    let deadline = Instant::now() + LIMIT;
    loop {
        fixture.frame(vec![])?;
        if fixture.state()?["close"] == "committed" {
            break;
        }
        if Instant::now() >= deadline {
            return Err("close did not drain preview worker".into());
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    assert!(!fixture.app.preview.is_busy());
    assert!(fixture.app.preview.ready.is_none());
    assert_eq!(fixture.current_source()?, SOURCE);
    Ok(())
}
