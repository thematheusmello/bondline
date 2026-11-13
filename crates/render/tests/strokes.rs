use bondline_render::{Error, Primitive, Rectangle, Scene, Stroke, cpu_scene, gpu_scene};

fn scene(stroke: Stroke) -> Scene {
    Scene {
        width: 8,
        height: 8,
        background: [255; 4],
        primitives: vec![Primitive::Stroke(stroke)],
    }
}

#[test]
fn native_butt_outline_has_clockwise_corners_without_extending_endpoints()
-> Result<(), Box<dyn std::error::Error>> {
    let stroke = Stroke {
        begin: [1.0, 3.0],
        end: [6.0, 3.0],
        width: 2.0,
        color: [0; 4],
    };
    assert_eq!(
        stroke.corners()?,
        [[1.0, 2.0], [6.0, 2.0], [6.0, 4.0], [1.0, 4.0]]
    );
    Ok(())
}

#[test]
fn butt_strokes_match_hand_drawn_pixel_grids_and_reversed_endpoints()
-> Result<(), Box<dyn std::error::Error>> {
    let cases = [
        (
            [1.0, 3.0],
            [6.0, 3.0],
            2.0,
            [
                "........", "........", ".#####..", ".#####..", "........", "........", "........",
                "........",
            ],
        ),
        (
            [3.0, 1.0],
            [3.0, 6.0],
            2.0,
            [
                "........", "..##....", "..##....", "..##....", "..##....", "..##....", "........",
                "........",
            ],
        ),
        (
            [1.0, 1.0],
            [6.0, 6.0],
            1.0,
            [
                "........", ".#......", "..#.....", "...#....", "....#...", ".....#..", "........",
                "........",
            ],
        ),
    ];
    for (begin, end, width, rows) in cases {
        let expected: Vec<u8> = rows
            .into_iter()
            .flat_map(str::bytes)
            .flat_map(|pixel| {
                if pixel == b'#' {
                    [0, 0, 0, 255]
                } else {
                    [255; 4]
                }
            })
            .collect();
        for (begin, end) in [(begin, end), (end, begin)] {
            let input = scene(Stroke {
                begin,
                end,
                width,
                color: [0, 0, 0, 255],
            });
            assert_eq!(cpu_scene(&input)?.rgba, expected);
            assert_eq!(pollster::block_on(gpu_scene(&input))?.0.rgba, expected);
        }
    }
    Ok(())
}

// Coordinates/widths are integers in quarter-pixel units. This oracle uses only exact
// integer dot/cross products; it calls no renderer geometry or coverage helper.
fn oracle(begin: [i128; 2], end: [i128; 2], width: i128, x: u32, y: u32) -> (bool, bool) {
    let dx = end[0] - begin[0];
    let dy = end[1] - begin[1];
    let px = i128::from(x) * 4 + 2 - begin[0];
    let py = i128::from(y) * 4 + 2 - begin[1];
    let length2 = dx * dx + dy * dy;
    let along = px * dx + py * dy;
    let cross = px * dy - py * dx;
    let inside = (0..=length2).contains(&along) && 4 * cross * cross <= width * width * length2;
    // Declared before implementation: f32 GPU coverage may differ only within 1/32 pixel
    // of a true stroke boundary. CPU reference must match the integer oracle exactly.
    let length = (length2 as f64).sqrt();
    let t = along as f64 / length / 4.0;
    let side = cross.abs() as f64 / length / 4.0;
    let extent = length / 4.0;
    let radius = width as f64 / 8.0;
    let distance = if inside {
        t.min(extent - t).min(radius - side)
    } else {
        (-t).max(t - extent)
            .max(0.0)
            .hypot((side - radius).max(0.0))
    };
    (inside, distance <= 1.0 / 32.0)
}

#[test]
fn clipped_oblique_and_fractional_strokes_agree_with_an_independent_integer_oracle()
-> Result<(), Box<dyn std::error::Error>> {
    let cases = [
        ([5, 7], [105, 63], 5),
        ([7, 71], [111, 3], 11),
        ([-40, 12], [160, 80], 5),
        ([24, -80], [80, 180], 13),
        ([-400000, -400000], [400000, 400000], 4),
        ([-400000, 400000], [400000, -399872], 7),
        ([0, 0], [1, 1], 1),
        ([400000, 0], [399999, 1], 400000),
        ([5, 7], [105, 63], 1),
        ([11, 13], [23, 29], 10),
        ([128, 96], [0, 0], 17),
    ];
    for (begin, end, width) in cases {
        for (begin, end) in [(begin, end), (end, begin)] {
            let input = Scene {
                width: 32,
                height: 24,
                background: [255; 4],
                primitives: vec![Primitive::Stroke(Stroke {
                    begin: begin.map(|value| value as f32 / 4.0),
                    end: end.map(|value| value as f32 / 4.0),
                    width: width as f32 / 4.0,
                    color: [12, 34, 56, 78],
                })],
            };
            let cpu = cpu_scene(&input)?;
            let (gpu, info) = pollster::block_on(gpu_scene(&input))?;
            assert!(!info.name.is_empty());
            for y in 0..input.height {
                for x in 0..input.width {
                    let (inside, boundary) = oracle(begin, end, width, x, y);
                    let expected = if inside { [12, 34, 56, 78] } else { [255; 4] };
                    let offset = ((y * input.width + x) * 4) as usize;
                    assert_eq!(
                        &cpu.rgba[offset..offset + 4],
                        &expected,
                        "CPU {begin:?}->{end:?} at {x},{y}"
                    );
                    let actual = &gpu.rgba[offset..offset + 4];
                    assert!(
                        actual == expected
                            || (boundary && (actual == [12, 34, 56, 78] || actual == [255; 4])),
                        "GPU {begin:?}->{end:?} at {x},{y}: {actual:?}, expected {expected:?}"
                    );
                }
            }
        }
    }
    Ok(())
}

#[test]
fn interleaved_strokes_and_rectangles_preserve_source_replacement_order()
-> Result<(), Box<dyn std::error::Error>> {
    let mut input = scene(Stroke {
        begin: [0.0, 2.0],
        end: [8.0, 2.0],
        width: 2.0,
        color: [0, 0, 0, 128],
    });
    input.primitives.insert(
        0,
        Primitive::Rectangle(Rectangle {
            x: 0,
            y: 0,
            width: 8,
            height: 8,
            color: [1, 2, 3, 4],
        }),
    );
    input.primitives.push(Primitive::Rectangle(Rectangle {
        x: 3,
        y: 1,
        width: 2,
        height: 2,
        color: [5, 6, 7, 8],
    }));
    let cpu = cpu_scene(&input)?;
    for y in 0..8 {
        for x in 0..8 {
            let expected = if (1..3).contains(&y) {
                if (3..5).contains(&x) {
                    [5, 6, 7, 8]
                } else {
                    [0, 0, 0, 128]
                }
            } else {
                [1, 2, 3, 4]
            };
            let offset = ((y * 8 + x) * 4) as usize;
            assert_eq!(&cpu.rgba[offset..offset + 4], &expected);
        }
    }
    assert_eq!(pollster::block_on(gpu_scene(&input))?.0, cpu);
    Ok(())
}

#[test]
fn invalid_strokes_and_mixed_primitive_budgets_fail_before_gpu_initialization() {
    let valid = Stroke {
        begin: [0.0, 0.0],
        end: [1.0, 1.0],
        width: 1.0,
        color: [0; 4],
    };
    for invalid in [
        Stroke {
            begin: [f32::NAN, 0.0],
            ..valid.clone()
        },
        Stroke {
            end: [0.0, f32::INFINITY],
            ..valid.clone()
        },
        Stroke {
            begin: [-100001.0, 0.0],
            ..valid.clone()
        },
        Stroke {
            end: valid.begin,
            ..valid.clone()
        },
        Stroke {
            width: 0.0,
            ..valid.clone()
        },
        Stroke {
            width: -1.0,
            ..valid.clone()
        },
        Stroke {
            width: f32::NAN,
            ..valid.clone()
        },
        Stroke {
            width: 100001.0,
            ..valid.clone()
        },
        Stroke {
            begin: [0.0, 0.0],
            end: [f32::MIN_POSITIVE, 0.0],
            ..valid.clone()
        },
    ] {
        let input = scene(invalid);
        assert!(matches!(cpu_scene(&input), Err(Error::InvalidStroke)));
        assert!(matches!(
            pollster::block_on(gpu_scene(&input)),
            Err(Error::InvalidStroke)
        ));
    }
    let mut excessive = scene(valid.clone());
    excessive.primitives = vec![Primitive::Stroke(valid); 4096];
    excessive.primitives.push(Primitive::Rectangle(Rectangle {
        x: 0,
        y: 0,
        width: 1,
        height: 1,
        color: [0; 4],
    }));
    assert!(matches!(cpu_scene(&excessive), Err(Error::SceneTooComplex)));
    assert!(matches!(
        pollster::block_on(gpu_scene(&excessive)),
        Err(Error::SceneTooComplex)
    ));
}

#[test]
fn boundary_allowance_does_not_extend_past_the_finite_stroke() {
    let (inside, boundary) = oracle([4, 4], [20, 4], 4, 5, 0);
    assert!(!inside);
    assert!(
        !boundary,
        "a point half a pixel beyond the cap is not within 1/32 of the finite boundary"
    );
}
