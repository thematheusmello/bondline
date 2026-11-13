use crate::{Cancellation, Error, Frame, Primitive, Scene};
use wgpu::util::DeviceExt;

fn draw_resources(
    device: &wgpu::Device,
    scene: &Scene,
) -> Option<(wgpu::RenderPipeline, wgpu::BindGroup, wgpu::Buffer, u32)> {
    let mut bytes = Vec::new();
    for value in [scene.width, scene.height] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    let mut count = 0;
    for primitive in &scene.primitives {
        // Clip the conservative raster bounds before converting to vertex floats.
        let Some(bounds) = primitive.bounds(scene.width, scene.height) else {
            continue;
        };
        for value in [bounds.x, bounds.y, bounds.width, bounds.height] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        let (geometry, kind) = match primitive {
            Primitive::Rectangle(_) => ([0.0; 6], 0u32),
            Primitive::Stroke(stroke) => {
                let [ax, ay] = stroke.begin.map(f64::from);
                let [bx, by] = stroke.end.map(f64::from);
                let (dx, dy) = (bx - ax, by - ay);
                let length = dx.hypot(dy);
                (
                    [
                        ax as f32,
                        ay as f32,
                        (dx / length) as f32,
                        (dy / length) as f32,
                        length as f32,
                        stroke.width,
                    ],
                    1u32,
                )
            }
        };
        for value in geometry {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        bytes.extend_from_slice(&u32::from_le_bytes(bounds.color).to_le_bytes());
        bytes.extend_from_slice(&kind.to_le_bytes());
        count += 1;
    }
    if count == 0 {
        return None;
    }
    let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Bondline primitive instances"),
        contents: &bytes,
        usage: wgpu::BufferUsages::STORAGE,
    });
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Bondline primitive storage"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: std::num::NonZeroU64::new(56),
            },
            count: None,
        }],
    });
    let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Bondline primitive bindings"),
        layout: &layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }],
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Bondline primitive pipeline layout"),
        bind_group_layouts: &[&layout],
        push_constant_ranges: &[],
    });
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Bondline primitive shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("primitives.wgsl").into()),
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Bondline primitive pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vertex",
            buffers: &[],
            compilation_options: Default::default(),
        },
        primitive: Default::default(),
        depth_stencil: None,
        multisample: Default::default(),
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fragment",
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba8Unorm,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        multiview: None,
        cache: None,
    });
    Some((pipeline, group, buffer, count))
}

/// Render and read back an empty native GPU canvas, with the actual adapter identity.
#[cfg(not(target_arch = "wasm32"))]
pub async fn gpu_solid(
    width: u32,
    height: u32,
    color: [u8; 4],
) -> Result<(Frame, wgpu::AdapterInfo), Error> {
    gpu_scene(&Scene {
        width,
        height,
        background: color,
        primitives: Vec::new(),
    })
    .await
}

/// Draw ordered instances with GPU coverage evaluation and read back straight RGBA8.
pub async fn gpu_scene(scene: &Scene) -> Result<(Frame, wgpu::AdapterInfo), Error> {
    gpu_scene_with_cancel(scene, &Cancellation::default()).await
}

pub async fn gpu_scene_with_cancel(
    scene: &Scene,
    cancel: &Cancellation,
) -> Result<(Frame, wgpu::AdapterInfo), Error> {
    let count = scene.validate()?;
    cancel.check()?;
    let (width, height, color) = (scene.width, scene.height, scene.background);
    let instance = wgpu::Instance::default();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await
        .ok_or(Error::NoAdapter)?;
    let info = adapter.get_info();
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor::default(), None)
        .await?;
    cancel.check()?;
    device.push_error_scope(wgpu::ErrorFilter::Validation);
    let resources = draw_resources(&device, scene);
    if let Some(error) = device.pop_error_scope().await {
        return Err(Error::Validation(error.to_string()));
    }
    let extent = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Bondline canvas"),
        size: extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&Default::default());
    let row = width * 4;
    let pitch = row.div_ceil(256) * 256;
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Bondline canvas readback"),
        size: u64::from(pitch) * u64::from(height),
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("canvas reference"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: f64::from(color[0]) / 255.0,
                        g: f64::from(color[1]) / 255.0,
                        b: f64::from(color[2]) / 255.0,
                        a: f64::from(color[3]) / 255.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        if let Some((pipeline, group, _buffer, count)) = &resources {
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, group, &[]);
            pass.draw(0..6, 0..*count);
        }
    }
    encoder.copy_texture_to_buffer(
        wgpu::ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::ImageCopyBuffer {
            buffer: &buffer,
            layout: wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(pitch),
                rows_per_image: Some(height),
            },
        },
        extent,
    );
    queue.submit(Some(encoder.finish()));
    let (sender, receiver) = std::sync::mpsc::channel();
    buffer
        .slice(..)
        .map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        cancel.check()?;
        device.poll(wgpu::Maintain::Poll);
        match receiver.try_recv() {
            Ok(result) => {
                result?;
                break;
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                return Err(Error::Timeout(
                    std::sync::mpsc::RecvTimeoutError::Disconnected,
                ));
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                if std::time::Instant::now() >= deadline {
                    return Err(Error::Timeout(std::sync::mpsc::RecvTimeoutError::Timeout));
                }
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        }
    }
    let data = buffer.slice(..).get_mapped_range();
    let mut rgba = Vec::with_capacity(count);
    for scanline in data.chunks_exact(pitch as usize) {
        cancel.check()?;
        rgba.extend_from_slice(scanline.get(..row as usize).ok_or(Error::InvalidReadback)?);
    }
    if rgba.len() != count {
        return Err(Error::InvalidReadback);
    }
    Ok((
        Frame {
            width,
            height,
            rgba,
        },
        info,
    ))
}
