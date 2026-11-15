//! Existing-renderer control, not aromatic perception or a new depiction style.

use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Document, Element, Fragment, FragmentId, PageId, Point2,
};
use bondline_render::{Frame, HitTarget, Primitive, cpu_scene, gpu_scene, graph_page_scene};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const SIZE: u32 = 1024;
const GEOMETRY_EPSILON: f64 = 1.0 / 1024.0;

fn pixel(frame: &Frame, x: u32, y: u32) -> TestResult<&[u8]> {
    let offset = usize::try_from((u64::from(y) * u64::from(frame.width) + u64::from(x)) * 4)?;
    Ok(frame.rgba.get(offset..offset + 4).ok_or("RGBA pixel")?)
}

fn close(actual: [f32; 2], expected: [f64; 2]) {
    for (actual, expected) in actual.into_iter().zip(expected) {
        assert!(
            (f64::from(actual) - expected).abs() <= GEOMETRY_EPSILON,
            "projected coordinate {actual} differs from analytic {expected}"
        );
    }
}

/// Check every pixel center in the middle half of one independently defined strip.
/// These corridors are distant from the vertex labels and adjacent ring edges.
fn strip_pixels(
    frame: &Frame,
    begin: [f64; 2],
    end: [f64; 2],
    width: f64,
    expected: &[u8],
) -> TestResult<usize> {
    let vector = [end[0] - begin[0], end[1] - begin[1]];
    let length = vector[0].hypot(vector[1]);
    let mut checked = 0;
    for y in 0..SIZE {
        for x in 0..SIZE {
            let delta = [f64::from(x) + 0.5 - begin[0], f64::from(y) + 0.5 - begin[1]];
            let along = (delta[0] * vector[0] + delta[1] * vector[1]) / length.powi(2);
            let across = (delta[0] * vector[1] - delta[1] * vector[0]).abs() / length;
            if (0.25..=0.75).contains(&along) && across <= width / 2.0 {
                assert_eq!(
                    pixel(frame, x, y)?,
                    expected,
                    "analytic strip pixel {x},{y}"
                );
                checked += 1;
            }
        }
    }
    assert!(
        checked > 0,
        "no represented pixel centers in the analytic strip"
    );
    Ok(checked)
}

#[test]
fn fixed_a4_kekule_hexagon_keeps_all_literal_ch_bonds_and_cpu_gpu_ink() -> TestResult {
    // A regular hexagon has circumradius equal to its side. The left vertical edge
    // and minimum atom-center coordinates (96,96) are fixed before invoking render.
    let half_width = 32.0 * 3.0_f64.sqrt();
    let points = [
        [96.0, 128.0],
        [96.0, 192.0],
        [96.0 + half_width, 224.0],
        [96.0 + 2.0 * half_width, 192.0],
        [96.0 + 2.0 * half_width, 128.0],
        [96.0 + half_width, 96.0],
    ];
    let edges = [
        (1, 2, BondOrder::Single),
        (2, 3, BondOrder::Double),
        (3, 4, BondOrder::Single),
        (4, 5, BondOrder::Double),
        (5, 6, BondOrder::Single),
        (6, 1, BondOrder::Double),
    ];
    let atoms = points
        .into_iter()
        .enumerate()
        .map(|(index, point)| {
            let mut atom = Atom::new(
                AtomId::new(u32::try_from(index + 1)?).ok_or("atom identity")?,
                Element::try_from(6)?,
                Some(Point2::new(point[0], point[1])?),
            );
            atom.explicit_hydrogens = Some(1);
            Ok(atom)
        })
        .collect::<TestResult<Vec<_>>>()?;
    let bonds = edges
        .into_iter()
        .enumerate()
        .map(|(index, (begin, end, order))| {
            Ok(Bond {
                id: BondId::new(u32::try_from(index + 1)?).ok_or("bond identity")?,
                begin: AtomId::new(begin).ok_or("begin identity")?,
                end: AtomId::new(end).ok_or("end identity")?,
                order,
            })
        })
        .collect::<TestResult<Vec<_>>>()?;
    let fragment_id = FragmentId::new(1).ok_or("fragment identity")?;
    let mut document = Document::default();
    document.append_fragments(
        PageId::new(1).ok_or("page identity")?,
        vec![Fragment::new(fragment_id, atoms, bonds)?],
    )?;
    let before = document.clone();
    let page = document.pages().first().ok_or("A4 page")?;
    // A4 points and the rounded 1024-square paper are independent literal expectations:
    // 210x297mm at72pt/in, margin32, paper679x960 at(172,32).
    let a4 = [210.0 * 72.0 / 25.4, 297.0 * 72.0 / 25.4];
    assert!((page.size().width_pt() - a4[0]).abs() < 1e-10);
    assert!((page.size().height_pt() - a4[1]).abs() < 1e-10);
    let fragment = page.fragments().first().ok_or("hexagon")?;
    assert_eq!((fragment.atoms().len(), fragment.bonds().len()), (6, 6));
    assert_eq!(
        fragment
            .bonds()
            .iter()
            .map(|bond| bond.order)
            .collect::<Vec<_>>(),
        vec![
            BondOrder::Single,
            BondOrder::Double,
            BondOrder::Single,
            BondOrder::Double,
            BondOrder::Single,
            BondOrder::Double,
        ]
    );
    for (atom, point) in fragment.atoms().iter().zip(points) {
        let position = atom.position.ok_or("authored position")?;
        assert_eq!([position.x_pt(), position.y_pt()], point);
        assert_eq!(atom.element.get(), 6);
        assert_eq!(atom.explicit_hydrogens, Some(1));
    }
    for (begin, end, _) in edges {
        let a = points
            .get(usize::try_from(begin - 1)?)
            .ok_or("begin point")?;
        let b = points.get(usize::try_from(end - 1)?).ok_or("end point")?;
        assert!(((a[0] - b[0]).hypot(a[1] - b[1]) - 64.0).abs() < 1e-10);
    }

    let graph = graph_page_scene(SIZE, SIZE, page)
        .map_err(|error| format!("fixed A4 alternating hexagon refused: {error:?} ({error})"))?;
    let frame = cpu_scene(graph.scene())?;
    assert_eq!(
        graph
            .scene()
            .primitives
            .iter()
            .filter(|p| matches!(p, Primitive::Stroke(_)))
            .count(),
        9
    );
    let project = |point: [f64; 2]| {
        [
            172.0 + point[0] * 679.0 / a4[0],
            32.0 + point[1] * 960.0 / a4[1],
        ]
    };
    let width = (679.0 / a4[0]).min(960.0 / a4[1]);
    let mut black = Vec::new();
    let mut gaps = Vec::new();
    for (index, (begin, end, order)) in edges.into_iter().enumerate() {
        let begin = project(
            *points
                .get(usize::try_from(begin - 1)?)
                .ok_or("begin point")?,
        );
        let end = project(*points.get(usize::try_from(end - 1)?).ok_or("end point")?);
        let vector = [end[0] - begin[0], end[1] - begin[1]];
        let length = vector[0].hypot(vector[1]);
        let normal = [-vector[1] / length, vector[0] / length];
        let bond_id = BondId::new(u32::try_from(index + 1)?).ok_or("bond identity")?;
        let strokes = graph.bond_strokes(fragment_id, bond_id).collect::<Vec<_>>();
        let offsets: &[f64] = if order == BondOrder::Single {
            &[0.0]
        } else {
            &[-1.5, 1.5]
        };
        assert_eq!(strokes.len(), offsets.len());
        for (stroke, offset) in strokes.into_iter().zip(offsets) {
            let shifted = |point: [f64; 2]| {
                [
                    point[0] + normal[0] * offset * width,
                    point[1] + normal[1] * offset * width,
                ]
            };
            let (a, b) = (shifted(begin), shifted(end));
            close(stroke.begin, a);
            close(stroke.end, b);
            assert!((f64::from(stroke.width) - width).abs() <= GEOMETRY_EPSILON);
            assert_eq!(stroke.color, [0, 0, 0, 255]);
            black.push(strip_pixels(&frame, a, b, width, &[0, 0, 0, 255])?);
            let midpoint = [((a[0] + b[0]) / 2.0) as f32, ((a[1] + b[1]) / 2.0) as f32];
            assert_eq!(
                graph.hit_test(midpoint, 0.0)?,
                Some(HitTarget::Bond {
                    fragment_id,
                    bond_id
                })
            );
        }
        if order == BondOrder::Double {
            gaps.push(strip_pixels(
                &frame,
                begin,
                end,
                width,
                &[255, 255, 255, 255],
            )?);
        }
    }
    let (gpu, adapter) = pollster::block_on(gpu_scene(graph.scene()))?;
    assert_eq!(gpu, frame);
    assert_eq!(document, before);
    eprintln!(
        "fixed A4 Kekule control: black strip pixels={black:?}, white gap pixels={gaps:?}, backend={:?}, adapter={}",
        adapter.backend, adapter.name
    );
    Ok(())
}
