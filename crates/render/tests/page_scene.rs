use bondline_model::PageSize;
use bondline_render::{Error, Rectangle, Scene, cpu_scene, gpu_scene, page_scene};

#[test]
fn page_pixels_match_an_independently_drawn_reference_grid()
-> Result<(), Box<dyn std::error::Error>> {
    let scene = page_scene(8, 8, PageSize::new(72.0, 144.0)?)?;
    let expected: Vec<u8> = [
        "........", "..bbb...", "..bwb...", "..bwb...", "..bwb...", "..bwbsss", "..bbbsss",
        ".....sss",
    ]
    .into_iter()
    .flat_map(|row| row.bytes())
    .flat_map(|pixel| {
        let shade = match pixel {
            b'b' => 196,
            b'w' => 255,
            b's' => 210,
            _ => 232,
        };
        [shade, shade, shade, 255]
    })
    .collect();
    assert_eq!(cpu_scene(&scene)?.rgba, expected);
    let (gpu, _) = pollster::block_on(gpu_scene(&scene))?;
    assert_eq!(gpu.rgba, expected);
    Ok(())
}

#[test]
fn layered_clipped_rectangles_match_on_cpu_and_gpu_without_alpha_blending()
-> Result<(), Box<dyn std::error::Error>> {
    let scene = Scene {
        width: 4,
        height: 2,
        background: [255; 4],
        rectangles: vec![
            Rectangle {
                x: 0,
                y: 0,
                width: 2,
                height: 2,
                color: [255, 0, 0, 255],
            },
            Rectangle {
                x: 1,
                y: 1,
                width: 10,
                height: 10,
                color: [0, 255, 0, 128],
            },
        ],
    };
    let cpu = cpu_scene(&scene)?;
    assert_eq!(
        cpu.rgba,
        [
            255, 0, 0, 255, 255, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 255,
            0, 255, 0, 128, 0, 255, 0, 128, 0, 255, 0, 128
        ]
    );
    assert_eq!(pollster::block_on(gpu_scene(&scene))?.0, cpu);
    Ok(())
}

#[test]
fn renderer_rejects_hostile_geometry_before_gpu_initialization() {
    let scene = Scene {
        width: 1,
        height: 1,
        background: [0; 4],
        rectangles: vec![Rectangle {
            x: u32::MAX,
            y: 0,
            width: 1,
            height: 1,
            color: [255; 4],
        }],
    };
    assert!(matches!(cpu_scene(&scene), Err(Error::InvalidGeometry)));
    assert!(matches!(
        pollster::block_on(gpu_scene(&scene)),
        Err(Error::InvalidGeometry)
    ));
    assert!(page_scene(0, 8, PageSize::A4).is_err());
    assert!(page_scene(u32::MAX, 8, PageSize::A4).is_err());
    let excessive = Scene {
        width: 1,
        height: 1,
        background: [0; 4],
        rectangles: vec![
            Rectangle {
                x: 0,
                y: 0,
                width: 1,
                height: 1,
                color: [255; 4]
            };
            4097
        ],
    };
    assert!(matches!(cpu_scene(&excessive), Err(Error::SceneTooComplex)));
    assert!(matches!(
        pollster::block_on(gpu_scene(&excessive)),
        Err(Error::SceneTooComplex)
    ));
}

#[test]
fn odd_readback_pitches_and_one_pixel_pages_match() -> Result<(), Box<dyn std::error::Error>> {
    for (width, height) in [(1, 1), (2, 3), (9, 17), (63, 33), (2048, 1)] {
        let scene = page_scene(width, height, PageSize::A4)?;
        let cpu = cpu_scene(&scene)?;
        if (width, height) == (1, 1) {
            assert_eq!(cpu.rgba, [255; 4]);
        }
        assert_eq!(pollster::block_on(gpu_scene(&scene))?.0, cpu);
    }
    Ok(())
}
