use crate::{Error, Frame, Rectangle, Scene};
use wgpu::util::DeviceExt;

fn draw_resources(
    device: &wgpu::Device,
    scene: &Scene,
) -> Option<(wgpu::RenderPipeline, wgpu::BindGroup, wgpu::Buffer, u32)> {
    // Clip on the host so distant integer coordinates cannot lose precision in the vertex shader.
    let rectangles: Vec<_> = scene
        .rectangles
        .iter()
        .filter_map(|rectangle| {
            let right = (rectangle.x + rectangle.width).min(scene.width);
            let bottom = (rectangle.y + rectangle.height).min(scene.height);
            (rectangle.x < right && rectangle.y < bottom).then(|| Rectangle {
                width: right - rectangle.x,
                height: bottom - rectangle.y,
                ..rectangle.clone()
            })
        })
        .collect();
    if rectangles.is_empty() {
        return None;
    }
    let mut bytes = Vec::new();
    for value in [scene.width, scene.height] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    for Rectangle {
        x,
        y,
        width,
        height,
        color,
    } in &rectangles
    {
        for value in [*x, *y, *width, *height, u32::from_le_bytes(*color)] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
    }
    let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Bondline rectangle instances"),
        contents: &bytes,
        usage: wgpu::BufferUsages::STORAGE,
    });
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Bondline rectangle storage"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: std::num::NonZeroU64::new(28),
            },
            count: None,
        }],
    });
    let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Bondline rectangle bindings"),
        layout: &layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }],
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Bondline rectangle pipeline layout"),
        bind_group_layouts: &[&layout],
        push_constant_ranges: &[],
    });
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Bondline rectangle shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("rectangles.wgsl").into()),
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Bondline rectangle pipeline"),
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
    Some((pipeline, group, buffer, rectangles.len() as u32))
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
        rectangles: Vec::new(),
    })
    .await
}

/// Draw ordered rectangle instances with the GPU rasterizer and read back straight RGBA8.
pub async fn gpu_scene(scene: &Scene) -> Result<(Frame, wgpu::AdapterInfo), Error> {
    let count = scene.validate()?;
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
        label: Some("Bondline empty canvas"),
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
            label: Some("empty canvas reference"),
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
    device.poll(wgpu::Maintain::Wait);
    receiver.recv_timeout(std::time::Duration::from_secs(10))??;
    let data = buffer.slice(..).get_mapped_range();
    let mut rgba = Vec::with_capacity(count);
    for scanline in data.chunks_exact(pitch as usize) {
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
