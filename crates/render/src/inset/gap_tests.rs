//! Independent pixel-lattice expectations; no product Gap/Stroke predicate is shared.
use super::*;
use crate::{Cancellation, Error, Rectangle, Stroke};
use bondline_model::DoublePosition;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[derive(Clone)]
struct Case {
    name: &'static str,
    axis: Stroke,
    secondary: Stroke,
    side: DoublePosition,
    size: [u32; 2],
}

fn line(begin: [f32; 2], end: [f32; 2], width: f32) -> Stroke {
    Stroke {
        begin,
        end,
        width,
        color: [0, 0, 0, 255],
    }
}
fn mask(x: u32, y: u32, width: u32, height: u32) -> Rectangle {
    Rectangle {
        x,
        y,
        width,
        height,
        color: [255; 4],
    }
}
fn literal() -> Case {
    Case {
        name: "short inset after a long axis-only span",
        axis: line([10.0, 20.0], [50.0, 20.0], 1.0),
        secondary: line([40.0, 23.0], [45.0, 23.0], 1.0),
        side: DoublePosition::Right,
        size: [64, 48],
    }
}
fn fixtures() -> Vec<Case> {
    let mut cases = vec![
        literal(),
        Case {
            name: "fractional horizontal Left",
            axis: line([6.125, 28.375], [56.125, 28.375], 0.75),
            secondary: line([15.125, 22.875], [43.125, 22.875], 0.75),
            side: DoublePosition::Left,
            size: [64, 48],
        },
        Case {
            name: "vertical Right",
            axis: line([30.0, 8.0], [30.0, 56.0], 1.0),
            secondary: line([24.0, 16.0], [24.0, 40.0], 1.0),
            side: DoublePosition::Right,
            size: [64, 64],
        },
        Case {
            name: "fractional vertical Left",
            axis: line([24.25, 8.75], [24.25, 56.75], 0.75),
            secondary: line([29.75, 16.25], [29.75, 43.25], 0.75),
            side: DoublePosition::Left,
            size: [64, 64],
        },
        Case {
            name: "diagonal45 fractional Right",
            axis: line([8.25, 8.25], [48.25, 48.25], 1.25),
            secondary: line([15.25, 21.25], [35.25, 41.25], 1.25),
            side: DoublePosition::Right,
            size: [64, 64],
        },
        Case {
            name: "diagonal3-4 fractional Right",
            axis: line([12.25, 6.75], [36.25, 38.75], 1.0),
            secondary: line([14.25, 17.75], [26.25, 33.75], 1.0),
            side: DoublePosition::Right,
            size: [48, 48],
        },
        Case {
            name: "diagonal4-3 fractional Left",
            axis: line([6.75, 12.25], [38.75, 36.25], 1.5),
            secondary: line([17.75, 14.25], [33.75, 26.25], 1.5),
            side: DoublePosition::Left,
            size: [48, 48],
        },
        Case {
            name: "one-column common span",
            axis: line([4.25, 9.5], [60.25, 9.5], 0.5),
            secondary: line([33.125, 11.5], [33.875, 11.5], 0.5),
            side: DoublePosition::Right,
            size: [64, 32],
        },
        Case {
            name: "positive common span without a pixel center",
            axis: line([4.25, 9.5], [60.25, 9.5], 0.5),
            secondary: line([33.625, 11.5], [34.125, 11.5], 0.5),
            side: DoublePosition::Right,
            size: [64, 32],
        },
        Case {
            name: "cropped finite caps",
            axis: line([-4.25, 12.25], [40.25, 12.25], 1.0),
            secondary: line([-1.25, 15.75], [19.25, 15.75], 1.0),
            side: DoublePosition::Right,
            size: [32, 32],
        },
        Case {
            name: "wide fractional gap with asymmetric caps",
            axis: line([7.125, 12.375], [55.125, 12.375], 0.75),
            secondary: line([17.125, 23.625], [41.125, 23.625], 0.75),
            side: DoublePosition::Right,
            size: [64, 48],
        },
    ];
    // Reverse both authored finite rays and swap side: exactly the same physical gap.
    let reversed = cases
        .iter()
        .map(|case| {
            let mut value = case.clone();
            std::mem::swap(&mut value.axis.begin, &mut value.axis.end);
            std::mem::swap(&mut value.secondary.begin, &mut value.secondary.end);
            value.side = match value.side {
                DoublePosition::Right => DoublePosition::Left,
                DoublePosition::Left => DoublePosition::Right,
                _ => return value,
            };
            value
        })
        .collect::<Vec<_>>();
    cases.extend(reversed);
    cases
}

// Independent unnormalized analytic test: projection is against EACH actual cap.
// Cross is positive to the right of a directed ray on the y-down canvas. No
// normalization, epsilon, nominal middle stroke or product covers/bounds is used.
fn projected(stroke: &Stroke, pixel: [f64; 2]) -> (f64, f64, f64) {
    let dx = f64::from(stroke.end[0]) - f64::from(stroke.begin[0]);
    let dy = f64::from(stroke.end[1]) - f64::from(stroke.begin[1]);
    let x = pixel[0] - f64::from(stroke.begin[0]);
    let y = pixel[1] - f64::from(stroke.begin[1]);
    (x * dx + y * dy, dx * dx + dy * dy, dx * y - dy * x)
}
fn closed_ink(stroke: &Stroke, pixel: [f64; 2]) -> bool {
    let (along, length2, cross) = projected(stroke, pixel);
    let radius = f64::from(stroke.width) / 2.0;
    along >= 0.0 && along <= length2 && cross * cross <= radius * radius * length2
}
fn true_gap(case: &Case, pixel: [f64; 2]) -> bool {
    let (a, alen2, across_a) = projected(&case.axis, pixel);
    let (b, blen2, across_b) = projected(&case.secondary, pixel);
    if a < 0.0 || a > alen2 || b < 0.0 || b > blen2 {
        return false;
    }
    let sign = match case.side {
        DoublePosition::Right => 1.0,
        DoublePosition::Left => -1.0,
        _ => return false,
    };
    let ar = f64::from(case.axis.width) / 2.0;
    let br = f64::from(case.secondary.width) / 2.0;
    sign * across_a > 0.0
        && sign * across_b < 0.0
        && across_a * across_a > ar * ar * alen2
        && across_b * across_b > br * br * blen2
}
fn masked(x: u32, y: u32, masks: &[&Rectangle]) -> bool {
    masks.iter().any(|r| {
        u64::from(x) >= u64::from(r.x)
            && u64::from(x) < u64::from(r.x) + u64::from(r.width)
            && u64::from(y) >= u64::from(r.y)
            && u64::from(y) < u64::from(r.y) + u64::from(r.height)
    })
}
fn exhaustive(
    case: &Case,
    masks: &[&Rectangle],
    other: Option<&Stroke>,
    excluded: &[&Stroke],
) -> usize {
    let mut pixels = 0;
    for y in 0..case.size[1] {
        for x in 0..case.size[0] {
            let point = [f64::from(x) + 0.5, f64::from(y) + 0.5];
            if true_gap(case, point)
                && !masked(x, y, masks)
                && other.is_none_or(|stroke| closed_ink(stroke, point))
                && excluded.iter().all(|stroke| !closed_ink(stroke, point))
            {
                pixels += 1;
            }
        }
    }
    pixels
}

#[test]
fn exact_short_common_span_rejects_axis_only_white_pixels_and_ink_edge_equality() -> TestResult {
    let case = literal();
    let gap = Gap::new(&case.axis, &case.secondary, case.side)?;
    let cancel = Cancellation::default();
    assert_eq!(exhaustive(&case, &[], None, &[]), 5);
    assert!(gap.has_visible_pixel(case.size, &[], None, &[], &cancel)?);
    let common_span = mask(40, 0, 5, case.size[1]);
    assert_eq!(exhaustive(&case, &[&common_span], None, &[]), 0);
    assert!(!gap.has_visible_pixel(case.size, &[&common_span], None, &[], &cancel)?);
    // This nominal averaged band has white pixels, but they lie outside the true gap.
    let old_midpoint_proxy = line([25.0, 21.5], [47.5, 21.5], 2.0);
    let witness = [25.5, 21.5];
    assert!(closed_ink(&old_midpoint_proxy, witness));
    assert!(!true_gap(&case, witness));
    assert!(!masked(25, 21, &[&common_span]));
    // The closed actual ink edges at20.5/22.5 must not qualify even with no exclusions.
    assert!(!true_gap(&case, [42.5, 20.5]));
    assert!(!true_gap(&case, [42.5, 22.5]));
    let only_open_gap_row = mask(0, 21, case.size[0], 1);
    assert!(!gap.has_visible_pixel(case.size, &[&only_open_gap_row], None, &[], &cancel)?);
    Ok(())
}

#[test]
fn actual_caps_and_edges_match_independent_exhaustive_pixels_with_masks_other_and_excluded_ink()
-> TestResult {
    let cancel = Cancellation::default();
    for (index, case) in fixtures().into_iter().enumerate() {
        let gap = Gap::new(&case.axis, &case.secondary, case.side)?;
        let full = mask(0, 0, case.size[0], case.size[1]);
        let left = mask(0, 0, case.size[0] / 2, case.size[1]);
        let stripe = mask(0, case.size[1] / 3, case.size[0], 2);
        let masks = [vec![], vec![&full], vec![&left, &stripe]];
        let needle = line([25.5, -2.0], [25.5, 66.0], 0.25);
        let diagonal = line([-1.25, 0.75], [63.75, 49.5], 1.5);
        let blocker = line([-1.0, 21.5], [66.0, 21.5], 2.0);
        let others = [None, Some(&needle), Some(&diagonal)];
        let exclusions = [vec![], vec![&case.axis, &case.secondary], vec![&blocker]];
        for mask_set in &masks {
            for other in others {
                for excluded in &exclusions {
                    let expected = exhaustive(&case, mask_set, other, excluded) != 0;
                    let actual =
                        gap.has_visible_pixel(case.size, mask_set, other, excluded, &cancel)?;
                    assert_eq!(
                        actual,
                        expected,
                        "case{index}: {}, masks={}, other={other:?}, exclusions={}",
                        case.name,
                        mask_set.len(),
                        excluded.len()
                    );
                }
            }
        }
    }
    Ok(())
}

#[test]
fn explicit_other_and_exclusion_controls_require_real_gap_pixels_after_mask_jumps() -> TestResult {
    let case = literal();
    let gap = Gap::new(&case.axis, &case.secondary, case.side)?;
    let cancel = Cancellation::default();
    let crossing = line([42.5, 0.0], [42.5, 47.0], 0.25);
    let axis_only = line([25.5, 0.0], [25.5, 47.0], 0.25);
    assert_eq!(exhaustive(&case, &[], Some(&crossing), &[]), 1);
    assert!(gap.has_visible_pixel(case.size, &[], Some(&crossing), &[], &cancel)?);
    assert_eq!(exhaustive(&case, &[], Some(&axis_only), &[]), 0);
    assert!(!gap.has_visible_pixel(case.size, &[], Some(&axis_only), &[], &cancel)?);
    let all = line([40.0, 21.5], [45.0, 21.5], 1.0);
    assert_eq!(exhaustive(&case, &[], None, &[&all]), 0);
    assert!(!gap.has_visible_pixel(case.size, &[], None, &[&all], &cancel)?);
    let first = mask(40, 21, 3, 1);
    let next = line([43.0, 21.5], [44.0, 21.5], 1.0);
    assert_eq!(exhaustive(&case, &[&first], None, &[&next]), 1);
    assert!(gap.has_visible_pixel(case.size, &[&first], None, &[&next], &cancel)?);
    // The only surviving44.5 pixel is excluded by this finite optional other cap.
    let earlier = line([40.0, 21.5], [44.0, 21.5], 1.0);
    assert_eq!(exhaustive(&case, &[&first], Some(&earlier), &[&next]), 0);
    assert!(!gap.has_visible_pixel(case.size, &[&first], Some(&earlier), &[&next], &cancel)?);
    Ok(())
}

#[test]
fn cancellation_precedes_gap_search_even_for_fully_masked_or_nonintersecting_queries() -> TestResult
{
    let case = literal();
    let gap = Gap::new(&case.axis, &case.secondary, case.side)?;
    let full = mask(0, 0, case.size[0], case.size[1]);
    let outside = line([100.0, 100.0], [110.0, 100.0], 1.0);
    let cancel = Cancellation::default();
    cancel.cancel();
    for (masks, other) in [
        (vec![], None),
        (vec![&full], None),
        (vec![], Some(&outside)),
    ] {
        assert!(matches!(
            gap.has_visible_pixel(case.size, &masks, other, &[], &cancel),
            Err(Error::Cancelled)
        ));
    }
    Ok(())
}

#[test]
fn rounded_oblique_actual_edges_match_each_individually_isolated_lattice_point() -> TestResult {
    // Original actual-f32 fixtures, not layout output. A single endpoint differs
    // from a parallel3:4/4:3 construction by exactly one f32 ULP. The finite caps
    // and slightly different real edge normals must both be honored.
    let first = Case {
        name: "3-4 Right with one-ULP secondary end x",
        axis: line([6.0, 4.0], [36.0, 44.0], 1.0),
        secondary: line([9.0, 13.0], [30.000002, 41.0], 1.0),
        side: DoublePosition::Right,
        size: [56, 56],
    };
    let second = Case {
        name: "4-3 Left with one-ULP secondary end y",
        axis: line([4.25, 6.75], [44.25, 36.75], 1.0),
        secondary: line([11.25, 8.25], [43.25, 32.250004], 1.0),
        side: DoublePosition::Left,
        size: [56, 56],
    };
    assert_eq!(first.secondary.end[0].to_bits(), 30.0_f32.to_bits() + 1);
    assert_eq!(second.secondary.end[1].to_bits(), 32.25_f32.to_bits() + 1);
    let original = [first, second];
    let cancel = Cancellation::default();
    for source in &original {
        let mut reversed = source.clone();
        std::mem::swap(&mut reversed.axis.begin, &mut reversed.axis.end);
        std::mem::swap(&mut reversed.secondary.begin, &mut reversed.secondary.end);
        reversed.side = match source.side {
            DoublePosition::Right => DoublePosition::Left,
            DoublePosition::Left => DoublePosition::Right,
            _ => return Err("declared side".into()),
        };
        for case in [source, &reversed] {
            let delta = |stroke: &Stroke| {
                [
                    f64::from(stroke.end[0]) - f64::from(stroke.begin[0]),
                    f64::from(stroke.end[1]) - f64::from(stroke.begin[1]),
                ]
            };
            let a = delta(&case.axis);
            let b = delta(&case.secondary);
            let cross = a[0] * b[1] - a[1] * b[0];
            let dot = a[0] * b[0] + a[1] * b[1];
            assert_ne!(
                cross, 0.0,
                "fixture must actually be nonparallel: {}",
                case.name
            );
            assert!(dot > 0.0, "fixture rays must be cooriented");
            assert!(
                cross.abs() < dot.abs() * 1e-6,
                "only a small rounding skew is declared"
            );
            assert!(
                exhaustive(case, &[], None, &[]) > 0,
                "non-vacuous positive gap"
            );
            let gap = Gap::new(&case.axis, &case.secondary, case.side)?;
            let cap_mask = mask(23, 0, 4, case.size[1]);
            let lower_mask = mask(0, 31, case.size[0], 3);
            let mask_sets = [vec![], vec![&cap_mask, &lower_mask]];
            for masks in &mask_sets {
                let mut expected_visible = 0;
                let mut expected_absent = 0;
                for y in 0..case.size[1] {
                    for x in 0..case.size[0] {
                        let center = [x as f32 + 0.5, y as f32 + 0.5];
                        let point = [f64::from(x) + 0.5, f64::from(y) + 0.5];
                        // Nominal0.2-length,0.2-width butt rectangle contains this
                        // pixel center alone. Its true f32 endpoints remain well
                        // inside the neighboring half-pixel boundaries.
                        let probe = line(
                            [center[0] - 0.1, center[1]],
                            [center[0] + 0.1, center[1]],
                            0.2,
                        );
                        assert!(f64::from(probe.begin[0]) > point[0] - 0.5);
                        assert!(f64::from(probe.end[0]) < point[0] + 0.5);
                        assert!(f64::from(probe.width) / 2.0 < 0.5);
                        assert!(closed_ink(&probe, point));
                        for neighbor in [
                            [point[0] - 1.0, point[1]],
                            [point[0] + 1.0, point[1]],
                            [point[0], point[1] - 1.0],
                            [point[0], point[1] + 1.0],
                        ] {
                            assert!(!closed_ink(&probe, neighbor));
                        }
                        let expected = true_gap(case, point) && !masked(x, y, masks);
                        if expected {
                            expected_visible += 1;
                        } else {
                            expected_absent += 1;
                        }
                        let actual =
                            gap.has_visible_pixel(case.size, masks, Some(&probe), &[], &cancel)?;
                        assert_eq!(
                            actual,
                            expected,
                            "{}; cross={cross}; pixel({x},{y}), masks={}",
                            case.name,
                            masks.len()
                        );
                    }
                }
                assert!(
                    expected_visible > 0 && expected_absent > 0,
                    "both outcomes must be checked"
                );
            }
        }
    }
    Ok(())
}

#[test]
fn finite_cooriented_cap_wedge_outside_endpoint_box_has_a_real_gap_pixel() -> TestResult {
    // Original accepted Gap inputs, intentionally independent of a3w generator.
    // The directions differ by about1.15degrees, with entirely finite exact endpoints.
    let case = Case {
        name: "finite cap wedge extends beyond the centerline endpoint box",
        axis: line([0.0, 100.0], [10000.0, 0.0], 0.001),
        secondary: line([0.0, 301.0], [10000.0, 401.0], 0.001),
        side: DoublePosition::Right,
        size: [10004, 512],
    };
    let point = [10001.5, 200.5];
    assert_eq!(
        projected(&case.axis, point),
        (100004950.0, 100010000.0, 2005150.0)
    );
    assert_eq!(
        projected(&case.secondary, point),
        (100004950.0, 100010000.0, -2005150.0)
    );
    assert!(true_gap(&case, point));
    assert!(!closed_ink(&case.axis, point));
    assert!(!closed_ink(&case.secondary, point));
    let probe = line([10001.25, 200.5], [10001.75, 200.5], 0.25);
    assert!(closed_ink(&probe, point));
    for neighbor in [
        [10000.5, 200.5],
        [10002.5, 200.5],
        [10001.5, 199.5],
        [10001.5, 201.5],
    ] {
        assert!(!closed_ink(&probe, neighbor));
    }
    let gap = Gap::new(&case.axis, &case.secondary, case.side)?;
    assert!(
        gap.covers(point[0], point[1]),
        "actual strict finite-cap predicates include the literal pixel"
    );
    let cancel = Cancellation::default();
    assert!(
        gap.has_visible_pixel(case.size, &[], Some(&probe), &[], &cancel)?,
        "coarse bounds must not remove a real gap/probe intersection outside endpoint extrema"
    );
    assert!(
        gap.has_visible_pixel(
            case.size,
            &[],
            Some(&probe),
            &[&case.axis, &case.secondary],
            &cancel
        )?,
        "excluding the actual ink edges must retain this independently established gap pixel"
    );
    let cover = mask(10001, 200, 1, 1);
    assert!(!gap.has_visible_pixel(case.size, &[&cover], Some(&probe), &[], &cancel)?);
    assert!(!gap.has_visible_pixel(case.size, &[], Some(&probe), &[&probe], &cancel)?);
    Ok(())
}
