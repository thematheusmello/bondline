use crate::{Cancellation, Error, Frame, Primitive, Scene};
use wgpu::util::DeviceExt;

// At most 4096 validated primitives, each clipped to the 2048-row capture bound.
const MAX_STROKE_ROW_BYTES: usize = 4096 * crate::MAX_CAPTURE_DIMENSION_PX as usize * 8;

struct DrawResources {
    pipeline: wgpu::RenderPipeline,
    group: wgpu::BindGroup,
    _buffers: [wgpu::Buffer; 2],
    count: u32,
}

fn draw_resources(
    device: &wgpu::Device,
    scene: &Scene,
    cancel: &Cancellation,
) -> Result<Option<DrawResources>, Error> {
    let limits = device.limits();
    let storage_limit =
        u64::from(limits.max_storage_buffer_binding_size).min(limits.max_buffer_size);
    let mut row_bytes = 0_usize;
    for primitive in &scene.primitives {
        cancel.check()?;
        if let Primitive::Stroke(stroke) = primitive
            && let Some(bounds) = stroke.bounds(scene.width, scene.height)
        {
            row_bytes = usize::try_from(bounds.height)
                .ok()
                .and_then(|height| height.checked_mul(8))
                .and_then(|additional| row_bytes.checked_add(additional))
                .ok_or(Error::SceneTooComplex)?;
        }
    }
    let primitive_bytes = scene
        .primitives
        .len()
        .checked_mul(28)
        .and_then(|size| size.checked_add(8))
        .ok_or(Error::SceneTooComplex)?;
    if row_bytes > MAX_STROKE_ROW_BYTES
        || u64::try_from(row_bytes.max(8)).map_err(|_| Error::SceneTooComplex)? > storage_limit
        || u64::try_from(primitive_bytes).map_err(|_| Error::SceneTooComplex)? > storage_limit
    {
        return Err(Error::SceneTooComplex);
    }
    // Reserve once: per-primitive geometric growth could exceed the logical row cap.
    let mut bytes = Vec::new();
    let mut rows = Vec::new();
    bytes
        .try_reserve_exact(primitive_bytes)
        .map_err(|_| Error::SceneTooComplex)?;
    rows.try_reserve_exact(row_bytes.max(8))
        .map_err(|_| Error::SceneTooComplex)?;
    for value in [scene.width, scene.height] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    let mut count = 0_u32;
    for primitive in &scene.primitives {
        cancel.check()?;
        // Integer pixel-cell bounds enclose the same centres as the CPU predicate.
        let Some(bounds) = primitive.bounds(scene.width, scene.height) else {
            continue;
        };
        let (row_offset, kind) = if let Primitive::Stroke(stroke) = primitive {
            let row_offset = u32::try_from(rows.len() / 8).map_err(|_| Error::SceneTooComplex)?;
            let additional = usize::try_from(bounds.height)
                .ok()
                .and_then(|height| height.checked_mul(8))
                .ok_or(Error::SceneTooComplex)?;
            let end = rows
                .len()
                .checked_add(additional)
                .ok_or(Error::SceneTooComplex)?;
            if end > MAX_STROKE_ROW_BYTES
                || u64::try_from(end).map_err(|_| Error::SceneTooComplex)? > storage_limit
            {
                return Err(Error::SceneTooComplex);
            }
            let right = bounds
                .x
                .checked_add(bounds.width)
                .ok_or(Error::InvalidGeometry)?;
            let bottom = bounds
                .y
                .checked_add(bounds.height)
                .ok_or(Error::InvalidGeometry)?;
            for y in bounds.y..bottom {
                cancel.check()?;
                let mut range = bounds.x..right;
                if !stroke.pixel_range(false, y, &mut range) {
                    range = 0..0;
                }
                for value in [range.start, range.end] {
                    rows.extend_from_slice(&value.to_le_bytes());
                }
            }
            (row_offset, 1_u32)
        } else {
            (0, 0_u32)
        };
        for value in [
            bounds.x,
            bounds.y,
            bounds.width,
            bounds.height,
            u32::from_le_bytes(bounds.color),
            kind,
            row_offset,
        ] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        count = count.checked_add(1).ok_or(Error::SceneTooComplex)?;
    }
    if count == 0 {
        return Ok(None);
    }
    cancel.check()?;
    if rows.is_empty() {
        rows.extend_from_slice(&[0; 8]);
    }
    if u64::try_from(bytes.len()).map_err(|_| Error::SceneTooComplex)? > storage_limit {
        return Err(Error::SceneTooComplex);
    }
    let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Bondline primitive instances"),
        contents: &bytes,
        usage: wgpu::BufferUsages::STORAGE,
    });
    let row_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Bondline exact stroke row geometry"),
        contents: &rows,
        usage: wgpu::BufferUsages::STORAGE,
    });
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Bondline primitive storage"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: std::num::NonZeroU64::new(36),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: std::num::NonZeroU64::new(8),
                },
                count: None,
            },
        ],
    });
    let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Bondline primitive bindings"),
        layout: &layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: row_buffer.as_entire_binding(),
            },
        ],
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
    Ok(Some(DrawResources {
        pipeline,
        group,
        _buffers: [buffer, row_buffer],
        count,
    }))
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

/// Draw ordered GPU instances from bounded host-prepared stroke row geometry.
/// Colour replacement and readback retain straight RGBA8 without blending.
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
    let resources = draw_resources(&device, scene, cancel)?;
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
        if let Some(resources) = &resources {
            pass.set_pipeline(&resources.pipeline);
            pass.set_bind_group(0, &resources.group, &[]);
            pass.draw(0..6, 0..resources.count);
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
