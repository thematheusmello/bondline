//! GUI-free CPU reference and native GPU readback foundation.

/// An owned, tightly packed RGBA8 frame.
#[derive(Debug, PartialEq, Eq)]
pub struct Frame {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

/// Failures are explicit; a missing GPU never silently becomes a CPU result.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("capture dimensions must be between 1 and 2048 pixels")]
    InvalidDimensions,
    #[error("no native GPU adapter is available")]
    NoAdapter,
    #[error(transparent)]
    Device(#[from] wgpu::RequestDeviceError),
    #[error(transparent)]
    Readback(#[from] wgpu::BufferAsyncError),
    #[error(transparent)]
    Timeout(#[from] std::sync::mpsc::RecvTimeoutError),
    #[error("GPU readback layout did not match its descriptor")]
    InvalidReadback,
}

fn byte_count(width: u32, height: u32) -> Result<usize, Error> {
    // shortcut: captures are bounded to 2048 per axis; use tiled rendering for print-resolution export.
    if !(1..=2048).contains(&width) || !(1..=2048).contains(&height) {
        return Err(Error::InvalidDimensions);
    }
    usize::try_from(u64::from(width) * u64::from(height) * 4).map_err(|_| Error::InvalidDimensions)
}

/// CPU reference for an empty RGBA8 canvas.
pub fn cpu_solid(width: u32, height: u32, color: [u8; 4]) -> Result<Frame, Error> {
    let count = byte_count(width, height)?;
    Ok(Frame {
        width,
        height,
        rgba: color.repeat(count / 4),
    })
}

/// Render and read back an empty native GPU canvas, with the actual adapter identity.
#[cfg(not(target_arch = "wasm32"))]
pub async fn gpu_solid(
    width: u32,
    height: u32,
    color: [u8; 4],
) -> Result<(Frame, wgpu::AdapterInfo), Error> {
    let count = byte_count(width, height)?;
    let instance = wgpu::Instance::default();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await
        .ok_or(Error::NoAdapter)?;
    let info = adapter.get_info();
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor::default(), None)
        .await?;
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
        let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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

#[cfg(test)]
mod tests {
    #[test]
    fn cpu_clear_matches_independent_pixel_reference() -> Result<(), Box<dyn std::error::Error>> {
        let frame = super::cpu_solid(2, 1, [12, 34, 56, 255])?;
        assert_eq!(frame.rgba, [12, 34, 56, 255, 12, 34, 56, 255]);
        assert!(super::cpu_solid(0, 1, [0; 4]).is_err());
        assert!(super::cpu_solid(u32::MAX, u32::MAX, [0; 4]).is_err());
        Ok(())
    }
}
