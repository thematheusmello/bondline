use super::*;
type TestResult = Result<(), Box<dyn std::error::Error>>;

fn p(x: f64, y: f64) -> Result<Point2, bondline_model::Error> {
    Point2::new(x, y)
}
fn preferences(fixed_lengths: bool, fixed_angles: bool) -> ChainPreferences {
    ChainPreferences {
        bond_length_pt: 48.0,
        angle_degrees: 90.0,
        fixed_lengths,
        fixed_angles,
    }
}
fn literal(points: &[Point2], expected: &[[f64; 2]]) {
    assert_eq!(points.len(), expected.len());
    for (point, [x, y]) in points.iter().zip(expected) {
        assert!((point.x_pt() - x).abs() <= 1e-9, "x: {point:?} versus {x}");
        assert!((point.y_pt() - y).abs() <= 1e-9, "y: {point:?} versus {y}");
    }
}

#[test]
fn independent_integer_paths_distinguish_center_and_endpoint_axes_and_reject_a_count_overflow()
-> TestResult {
    let begin = p(144.0, 144.0)?;
    let mut fixed = Acyclic::new(begin, preferences(true, true));
    literal(
        &fixed.sample(p(216.0, 216.0)?)?,
        &[
            [144.0, 144.0],
            [192.0, 144.0],
            [192.0, 192.0],
            [240.0, 192.0],
        ],
    );
    let before = fixed.points().to_vec();
    assert!(fixed.sample(p(100000.0, 100000.0)?).is_err());
    assert_eq!(fixed.points(), before);
    let mut stretched = Acyclic::new(begin, preferences(false, false));
    literal(
        &stretched.sample(p(244.0, 144.0)?)?,
        &[
            [144.0, 144.0],
            [184.0, 124.0],
            [204.0, 164.0],
            [244.0, 144.0],
        ],
    );
    Ok(())
}

#[test]
fn unsnapped_reversal_follows_the_current_axis_and_explicit_off_reversal_keeps_the_endpoint()
-> TestResult {
    let mut fixed = Acyclic::new(p(144.0, 144.0)?, preferences(true, false));
    fixed.sample(p(216.0, 216.0)?)?;
    literal(
        &fixed.sample(p(72.0, 216.0)?)?,
        &[[144.0, 144.0], [96.0, 144.0], [96.0, 192.0], [48.0, 192.0]],
    );
    let mut stretched = Acyclic::new(p(144.0, 144.0)?, preferences(false, true));
    stretched.sample(p(244.0, 144.0)?)?;
    stretched.reverse();
    literal(
        &stretched.sample(p(244.0, 144.0)?)?,
        &[
            [144.0, 144.0],
            [184.0, 164.0],
            [204.0, 124.0],
            [244.0, 144.0],
        ],
    );
    Ok(())
}

#[test]
fn literal_snaking_walk_backtracks_and_duplicate_samples_do_not_pop_a_short_converted_edge()
-> TestResult {
    let mut walk = Snaking::new(p(144.0, 144.0)?, preferences(true, true));
    for [x, y] in [[192.0, 144.0], [192.0, 192.0], [240.0, 192.0]] {
        walk.sample(p(x, y)?)?;
    }
    literal(
        &walk.sample(p(240.0, 144.0)?)?,
        &[
            [144.0, 144.0],
            [192.0, 144.0],
            [192.0, 192.0],
            [240.0, 192.0],
            [240.0, 144.0],
        ],
    );
    walk.sample(p(240.0, 192.0)?)?;
    literal(
        &walk.sample(p(192.0, 192.0)?)?,
        &[[144.0, 144.0], [192.0, 144.0], [192.0, 192.0]],
    );
    assert!(walk.sample(p(100000.0, 100000.0)?).is_err());
    literal(
        &walk.sample(p(192.0, 192.0)?)?,
        &[[144.0, 144.0], [192.0, 144.0], [192.0, 192.0]],
    );
    let initial = vec![p(100.0, 100.0)?, p(110.0, 100.0)?];
    let mut short = Snaking::from_prefix(initial, preferences(false, true), p(110.0, 100.0)?)?;
    literal(
        &short.sample(p(110.0, 100.0)?)?,
        &[[100.0, 100.0], [110.0, 100.0]],
    );
    literal(
        &short.sample(p(109.0, 100.0)?)?,
        &[[100.0, 100.0], [109.0, 100.0]],
    );
    Ok(())
}
