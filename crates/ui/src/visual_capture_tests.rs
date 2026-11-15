//! Explicit dev capture of the actual application frame; never opens a native window.
use super::*;
use eframe::{egui_wgpu, wgpu};
use serde_json::json;
use std::{
    future::Future,
    path::PathBuf,
    process::Command as Process,
    task::{Context, Poll, Waker},
    time::{Duration, Instant},
};

type CaptureResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const WIDTH: u32 = 1600;
const HEIGHT: u32 = 1000;

fn resolve<T>(future: impl Future<Output = T>) -> CaptureResult<T> {
    let mut future = std::pin::pin!(future);
    let mut context = Context::from_waker(Waker::noop());
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Poll::Ready(value) = future.as_mut().poll(&mut context) {
            return Ok(value);
        }
        if Instant::now() >= deadline {
            return Err("offscreen GPU initialization exceeded ten seconds".into());
        }
        std::thread::sleep(Duration::from_millis(1));
    }
}

fn gpu_frame(
    context: &egui::Context,
    outputs: &[egui::FullOutput],
) -> CaptureResult<(Vec<u8>, wgpu::AdapterInfo)> {
    let instance = wgpu::Instance::default();
    let adapter = resolve(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))?
        .ok_or("no offscreen GPU adapter")?;
    let info = adapter.get_info();
    let (device, queue) =
        resolve(adapter.request_device(&wgpu::DeviceDescriptor::default(), None))??;
    device.push_error_scope(wgpu::ErrorFilter::Validation);
    // These match the current native renderer's gamma-space output and dithering default.
    let mut renderer =
        egui_wgpu::Renderer::new(&device, wgpu::TextureFormat::Rgba8Unorm, None, 1, true);
    for output in outputs {
        // Warm frames may introduce the atlas before the final frame's incremental update.
        for (id, delta) in &output.textures_delta.set {
            renderer.update_texture(&device, &queue, *id, delta);
        }
        assert!(output.textures_delta.free.is_empty());
    }
    let output = outputs.last().ok_or("missing actual application frame")?;
    let jobs = context.tessellate(output.shapes.clone(), output.pixels_per_point);
    assert!(!jobs.is_empty());
    let screen = egui_wgpu::ScreenDescriptor {
        size_in_pixels: [WIDTH, HEIGHT],
        pixels_per_point: output.pixels_per_point,
    };
    let extent = wgpu::Extent3d {
        width: WIDTH,
        height: HEIGHT,
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("actual Bondline application frame"),
        size: extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&Default::default());
    let row = WIDTH * 4;
    let pitch =
        row.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("actual application frame readback"),
        size: u64::from(pitch) * u64::from(HEIGHT),
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    let callbacks = renderer.update_buffers(&device, &queue, &mut encoder, &jobs, &screen);
    {
        let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("actual App egui frame"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        // The pinned egui-wgpu API requires this safe lifetime conversion; the pass
        // is dropped before the encoder is used for readback.
        renderer.render(&mut pass.forget_lifetime(), &jobs, &screen);
    }
    encoder.copy_texture_to_buffer(
        wgpu::ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::ImageCopyBuffer {
            buffer: &readback,
            layout: wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(pitch),
                rows_per_image: Some(HEIGHT),
            },
        },
        extent,
    );
    queue.submit(callbacks.into_iter().chain([encoder.finish()]));
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    readback
        .slice(..)
        .map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        device.poll(wgpu::Maintain::Poll);
        match receiver.try_recv() {
            Ok(result) => {
                result?;
                break;
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                return Err("GPU readback callback disconnected".into());
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                if Instant::now() >= deadline {
                    return Err("GPU frame readback exceeded ten seconds".into());
                }
                std::thread::sleep(Duration::from_millis(1));
            }
        }
    }
    let data = readback.slice(..).get_mapped_range();
    let mut rgba = Vec::with_capacity((WIDTH * HEIGHT * 4) as usize);
    for scanline in data.chunks_exact(pitch as usize) {
        rgba.extend_from_slice(scanline.get(..row as usize).ok_or("short GPU scanline")?);
    }
    drop(data);
    readback.unmap();
    if let Some(error) = resolve(device.pop_error_scope())? {
        return Err(error.into());
    }
    assert_eq!(rgba.len(), (WIDTH * HEIGHT * 4) as usize);
    Ok((rgba, info))
}

#[test]
#[ignore = "explicit full application-frame artifact, uses offscreen GPU without a native window"]
fn actual_application_frame_to_png() -> CaptureResult {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/visual-capture");
    std::fs::create_dir_all(&directory)?;
    let directory = directory.canonicalize()?;
    let name =
        std::env::var("BONDLINE_VISUAL_CAPTURE_NAME").unwrap_or_else(|_| "current-app".to_owned());
    if name.is_empty()
        || name.len() > 64
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    {
        return Err("capture name must be one to64 ASCII letters, digits or hyphens".into());
    }
    let raw_path = directory.join(format!("{name}.rgba"));
    let mut engine = Engine::default();
    engine.set_ui_file_host(true);
    let commands = [
        json!({"command":"page.set_size","params":{"page_id":1,"width_pt":420,"height_pt":280}}),
        json!({"command":"bond.create","params":{"page_id":1,"order":"single","begin_x_pt":95,"begin_y_pt":65,"end_x_pt":225,"end_y_pt":65}}),
        json!({"command":"bond.create","params":{"page_id":1,"order":"double","begin_x_pt":95,"begin_y_pt":140,"end_x_pt":225,"end_y_pt":140}}),
        json!({"command":"bond.create","params":{"page_id":1,"order":"triple","begin_x_pt":95,"begin_y_pt":215,"end_x_pt":225,"end_y_pt":215}}),
        json!({"command":"ui.bond_tool","params":{"order":"double"}}),
    ];
    for command in &commands {
        engine.dispatch(
            command["command"].as_str().ok_or("command name")?,
            command["params"].clone(),
        )?;
    }
    let before = serde_json::to_value(engine.state())?;
    assert_eq!(
        engine
            .controls()
            .iter()
            .find(|control| control.id == "tool.bond.double")
            .and_then(|control| control.selected),
        Some(true)
    );
    let engine = Arc::new(Mutex::new(engine));
    let mut app = BondlineApp {
        smoke_frames: None,
        engine: Arc::clone(&engine),
        executor: CommandExecutor::new(Arc::clone(&engine)),
        control: None,
        error: None,
        drawing: DrawingState::default(),
        files: FileWorkflow::new(directory.clone()),
        preview: PreviewWorkflow::default(),
        sdf_fields: crate::sdf_fields::SdfFieldsWorkflow::default(),
    };
    let context = egui::Context::default();
    configure_context(&context);
    let mut outputs = Vec::new();
    for frame in 0..3 {
        let mut input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(WIDTH as f32, HEIGHT as f32),
            )),
            time: Some(f64::from(frame) / 60.0),
            focused: true,
            ..Default::default()
        };
        input
            .viewports
            .entry(egui::ViewportId::ROOT)
            .or_default()
            .native_pixels_per_point = Some(1.0);
        outputs.push(context.run(input, |context| app.update_workspace(context)));
    }
    assert!(app.error.is_none(), "{:?}", app.error);
    assert_eq!(
        serde_json::to_value(engine.lock().map_err(|_| "engine")?.state())?,
        before,
        "capturing actual frames must not author edits"
    );
    let output = outputs.last().ok_or("application frame")?;
    for name in ["bondline", "File", "Edit", "Page"] {
        assert!(output.shapes.iter().any(|shape| {
            matches!(&shape.shape, egui::Shape::Text(text) if text.galley.text() == name)
        }));
    }
    let components = output.shapes.iter().filter(|shape| {
        matches!(&shape.shape, egui::Shape::Path(path) if path.closed && path.points.len() == 4 && path.fill == egui::Color32::BLACK)
    }).count();
    assert_eq!(
        components, 6,
        "actual Single/Double/Triple canvas components"
    );
    let border = output
        .shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            egui::Shape::Rect(rect) if rect.fill == egui::Color32::from_gray(196) => {
                Some(rect.rect)
            }
            _ => None,
        })
        .ok_or("actual paper border")?;
    assert!(border.width() > 900.0 && border.height() > 500.0);
    assert!(output.shapes.iter().any(|shape| {
        matches!(&shape.shape, egui::Shape::Rect(rect) if rect.fill == egui::Color32::WHITE && rect.rect == border.shrink(1.0))
    }), "actual white paper must be inset exactly one pixel inside its border");
    let (rgba, adapter) = gpu_frame(&context, &outputs)?;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&raw_path)?;
    file.write_all(&rgba)?;
    let python =
        std::env::var_os("BONDLINE_PYTHON").ok_or("BONDLINE_PYTHON must select pinned Pillow")?;
    let encoded = Process::new(python)
        .arg("-c")
        .arg(r#"from pathlib import Path
import hashlib,json,sys,PIL
from PIL import Image
assert PIL.__version__ == '12.3.0'
p=Path(sys.argv[1]); w,h=map(int,sys.argv[2:])
data=p.read_bytes(); assert len(data)==w*h*4
image=Image.frombytes('RGBA',(w,h),data)
assert image.getextrema()[3]==(255,255)
png=p.with_suffix('.png'); image.save(png)
with Image.open(png) as reopened:
 assert reopened.size==(w,h) and reopened.tobytes()==data
print(json.dumps({'png':str(png),'png_bytes':png.stat().st_size,'png_sha256':hashlib.sha256(png.read_bytes()).hexdigest(),'rgba_bytes':len(data),'rgba_sha256':hashlib.sha256(data).hexdigest(),'pillow':PIL.__version__}))
"#)
        .arg(&raw_path)
        .arg(WIDTH.to_string())
        .arg(HEIGHT.to_string())
        .output()?;
    assert!(
        encoded.status.success(),
        "{}",
        String::from_utf8_lossy(&encoded.stderr)
    );
    let encoding: serde_json::Value = serde_json::from_slice(&encoded.stdout)?;
    let receipt = json!({
        "kind":"actual_app_offscreen_frame", "width":WIDTH,"height":HEIGHT,"pixels_per_point":1.0,
        "actual_app_method":"BondlineApp::update_workspace", "warm_frames":3,
        "output_format":"Rgba8Unorm", "samples":1,"dithering":true,
        "adapter":{"name":adapter.name,"backend":format!("{:?}",adapter.backend),"device_type":format!("{:?}",adapter.device_type),"vendor":adapter.vendor,"device":adapter.device,"driver":adapter.driver,"driver_info":adapter.driver_info},
        "seed_commands":commands,"state":before,"encoding":encoding,
        "native_window":false,"native_input":false,"golden_approved":false
    });
    std::fs::write(
        directory.join(format!("{name}.json")),
        serde_json::to_vec_pretty(&receipt)?,
    )?;
    Ok(())
}
