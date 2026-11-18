//! Independent literal ring geometry/render controls, not a new renderer RED or golden.
use base64::{Engine as _, engine::general_purpose::STANDARD};
use bondline_commands::Engine;
use bondline_io::StoredDocument;
use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Element, Fragment, FragmentId, Page, PageId, PageSize,
    Point2,
};
use bondline_render::{
    Cancellation, Frame, GraphScene, HitTarget, cpu_scene, graph_page_scene,
    graph_page_scene_with_display_hydrogens,
};
use serde_json::json;
use std::{
    collections::BTreeMap,
    process::{Command as Process, Stdio},
    sync::Arc,
};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const DIM: u32 = 640;
const SIDE: f64 = 48.0;
const COORD_TOL: f64 = 1e-9;
const ANGLE_TOL: f64 = 1e-10;

#[derive(Clone, Copy, Debug)]
enum Case {
    Free,
    Fused,
    Spiro,
}
impl Case {
    fn label(self) -> &'static str {
        match self {
            Self::Free => "free",
            Self::Fused => "fused",
            Self::Spiro => "spiro",
        }
    }
    fn counts(self) -> (usize, usize, u32) {
        match self {
            Self::Free => (6, 6, 12),
            Self::Fused => (10, 11, 18),
            Self::Spiro => (11, 12, 20),
        }
    }
}
fn literal(case: Case) -> TestResult<StoredDocument> {
    // Original analytic vertices, independent of ring.create or the SMILES layout code.
    // A=(120,168), B=(120,120); positive algebraic cross-product interior lies east.
    let q = 24.0 * 3.0_f64.sqrt();
    let mut points = vec![
        [120.0, 168.0],
        [120.0, 120.0],
        [120.0 + q, 96.0],
        [120.0 + 2.0 * q, 120.0],
        [120.0 + 2.0 * q, 168.0],
        [120.0 + q, 192.0],
    ];
    let mut h = vec![2_u16; 6];
    let mut edges = vec![(1, 2), (2, 3), (3, 4), (4, 5), (5, 6), (6, 1)];
    match case {
        Case::Free => {}
        Case::Fused => {
            // Second polygon is west of preserved directed edge1:1->2, side negative.
            h[0] = 1;
            h[1] = 1;
            points.extend([
                [120.0 - q, 96.0],
                [120.0 - 2.0 * q, 120.0],
                [120.0 - 2.0 * q, 168.0],
                [120.0 - q, 192.0],
            ]);
            h.extend([2; 4]);
            edges.extend([(2, 7), (7, 8), (8, 9), (9, 10), (10, 1)]);
        }
        Case::Spiro => {
            // Old/new centers are A+(q,-24) and A+(-q,24): opposing bisectors.
            // New first edge A->(120,216), positive side; the shared atom is H0.
            h[0] = 0;
            points.extend([
                [120.0, 216.0],
                [120.0 - q, 240.0],
                [120.0 - 2.0 * q, 216.0],
                [120.0 - 2.0 * q, 168.0],
                [120.0 - q, 144.0],
            ]);
            h.extend([2; 5]);
            edges.extend([(1, 7), (7, 8), (8, 9), (9, 10), (10, 11), (11, 1)]);
        }
    }
    let atoms = points
        .into_iter()
        .zip(h)
        .enumerate()
        .map(|(i, (p, h))| {
            let mut atom = Atom::new(
                AtomId::new(u32::try_from(i + 1)?).ok_or("atom")?,
                Element::try_from(6)?,
                Some(Point2::new(p[0], p[1])?),
            );
            atom.explicit_hydrogens = Some(h);
            Ok(atom)
        })
        .collect::<TestResult<Vec<_>>>()?;
    let bonds = edges
        .into_iter()
        .enumerate()
        .map(|(i, (a, b))| {
            Ok(Bond {
                double_position: bondline_model::DoublePosition::Unspecified,
                id: BondId::new(u32::try_from(i + 1)?).ok_or("bond")?,
                begin: AtomId::new(a).ok_or("begin")?,
                end: AtomId::new(b).ok_or("end")?,
                order: BondOrder::Single,
            })
        })
        .collect::<TestResult<Vec<_>>>()?;
    let mut stored = StoredDocument::default();
    let page = PageId::new(1).ok_or("page")?;
    stored
        .model_mut()
        .set_page_size(page, PageSize::new(288.0, 288.0)?)?;
    stored.model_mut().append_fragments(
        page,
        vec![Fragment::new(
            FragmentId::new(1).ok_or("fragment")?,
            atoms,
            bonds,
        )?],
    )?;
    Ok(stored)
}
fn point(page: &Page, id: u32) -> TestResult<[f64; 2]> {
    let p = page.fragments()[0]
        .atoms()
        .iter()
        .find(|a| a.id.get() == id)
        .and_then(|a| a.position)
        .ok_or("literal position")?;
    Ok([p.x_pt(), p.y_pt()])
}
fn check_polygon(page: &Page, ids: &[u32]) -> TestResult {
    assert_eq!(ids.len(), 6);
    for i in 0..ids.len() {
        let a = point(page, ids[(i + ids.len() - 1) % ids.len()])?;
        let b = point(page, ids[i])?;
        let c = point(page, ids[(i + 1) % ids.len()])?;
        let u = [a[0] - b[0], a[1] - b[1]];
        let v = [c[0] - b[0], c[1] - b[1]];
        assert!((u[0].hypot(u[1]) - SIDE).abs() <= COORD_TOL);
        assert!((v[0].hypot(v[1]) - SIDE).abs() <= COORD_TOL);
        let angle = (u[0] * v[1] - u[1] * v[0])
            .abs()
            .atan2(u[0] * v[0] + u[1] * v[1]);
        assert!((angle - 2.0 * std::f64::consts::PI / 3.0).abs() <= ANGLE_TOL);
    }
    Ok(())
}
fn geometry(page: &Page, case: Case) -> TestResult {
    assert_eq!(page.id().get(), 1);
    assert_eq!(page.size(), PageSize::new(288.0, 288.0)?);
    assert_eq!(page.fragments().len(), 1);
    let fragment = &page.fragments()[0];
    let (atoms, bonds, h) = case.counts();
    assert_eq!(fragment.atoms().len(), atoms);
    assert_eq!(fragment.bonds().len(), bonds);
    assert_eq!(
        fragment
            .atoms()
            .iter()
            .map(|a| u32::from(a.explicit_hydrogens.unwrap_or(0)))
            .sum::<u32>(),
        h
    );
    assert!(fragment.atoms().iter().all(|a| a.element.get() == 6
        && a.isotope.is_none()
        && a.charge == 0
        && a.radical == bondline_model::Radical::None
        && a.explicit_hydrogens.is_some()));
    assert!(
        fragment
            .bonds()
            .iter()
            .all(|b| b.order == BondOrder::Single)
    );
    check_polygon(page, &[1, 2, 3, 4, 5, 6])?;
    match case {
        Case::Free => {}
        Case::Fused => check_polygon(page, &[1, 2, 7, 8, 9, 10])?,
        Case::Spiro => check_polygon(page, &[1, 7, 8, 9, 10, 11])?,
    }
    Ok(())
}
fn pixel(frame: &Frame, x: u32, y: u32) -> TestResult<&[u8]> {
    let at = usize::try_from((y * frame.width + x) * 4)?;
    frame
        .rgba
        .get(at..at + 4)
        .ok_or_else(|| "pixel bounds".into())
}
fn analytic_ink(page: &Page, graph: &GraphScene, frame: &Frame, case: Case) -> TestResult {
    assert_eq!([frame.width, frame.height], [DIM, DIM]);
    //288pt paper in640px with32px margins is exactly2px/pt. A2px-wide strip
    //contains the nearest pixel center to its midpoint (distance<=sqrt(0.5)<1px).
    for bond in page.fragments()[0].bonds() {
        let a = point(page, bond.begin.get())?;
        let b = point(page, bond.end.get())?;
        let center = [32.0 + a[0] + b[0], 32.0 + a[1] + b[1]];
        let [x, y] = center.map(|v| v.floor() as u32);
        let probe = [x as f32 + 0.5, y as f32 + 0.5];
        assert_eq!(
            pixel(frame, x, y)?,
            [0, 0, 0, 255],
            "{}: bond{} midpoint ink",
            case.label(),
            bond.id.get()
        );
        assert_eq!(
            graph.hit_test(probe, 0.0)?,
            Some(HitTarget::Bond {
                fragment_id: FragmentId::new(1).ok_or("fragment")?,
                bond_id: bond.id
            })
        );
        assert_eq!(
            graph
                .bond_strokes(FragmentId::new(1).ok_or("fragment")?, bond.id)
                .count(),
            1
        );
    }
    // Fixed shared/left edge strip, well away from either endpoint's C/H glyph masks.
    for y in 308..332 {
        for x in 270..274 {
            assert_eq!(
                pixel(frame, x, y)?,
                if [271, 272].contains(&x) {
                    &[0, 0, 0, 255]
                } else {
                    &[255, 255, 255, 255]
                },
                "vertical corridor{x},{y}"
            );
        }
    }
    for [x, y] in match case {
        Case::Free => vec![[355, 319]],
        Case::Fused => vec![[355, 319], [188, 319]],
        Case::Spiro => vec![[355, 319], [188, 415]],
    } {
        assert_eq!(
            pixel(frame, x, y)?,
            [255, 255, 255, 255],
            "{}: empty ring interior",
            case.label()
        );
        assert_eq!(graph.hit_test([x as f32 + 0.5, y as f32 + 0.5], 0.0)?, None);
    }
    Ok(())
}
fn captures(engine: &mut Engine) -> TestResult<Vec<String>> {
    let before = serde_json::to_value(engine.state())?;
    let content = engine.content().clone();
    let stamp = engine.capture_stamp();
    let file = engine.ui_file_context_stamp();
    let view = engine.view();
    let model = engine.stored_document()?.model_snapshot();
    let generation = engine.session_generation();
    let mut images = Vec::new();
    for backend in ["cpu", "gpu"] {
        let result = engine.dispatch(
            "render.capture",
            json!({"page_id":1,"width_px":DIM,"height_px":DIM,"backend":backend}),
        )?;
        assert!(!result.changed);
        let image = result.capture.ok_or("actual registered image")?;
        assert_eq!((image.width_px, image.height_px), (DIM, DIM));
        assert_eq!(image.mime_type, "image/png");
        if backend == "gpu" {
            let adapter = image.adapter.as_ref().ok_or("actual GPU adapter")?;
            assert!(
                !adapter.name.is_empty()
                    && !adapter.backend.is_empty()
                    && !adapter.device_type.is_empty()
            );
            println!(
                "literal ring control GPU backend={} name={}",
                adapter.backend, adapter.name
            );
        } else {
            assert!(image.adapter.is_none());
        }
        images.push(image.png_base64);
        assert_eq!(serde_json::to_value(engine.state())?, before);
        assert_eq!(engine.content(), &content);
        assert_eq!(engine.capture_stamp(), stamp);
        assert!(engine.ui_file_context_stamp_matches(&file));
        assert!(Arc::ptr_eq(
            &model,
            &engine.stored_document()?.model_snapshot()
        ));
        assert_eq!(engine.view(), view);
        assert_eq!(engine.session_generation(), generation);
    }
    assert_eq!(images[0], images[1]);
    Ok(images)
}
fn pillow(images: &[String], frame: &Frame) -> TestResult {
    let mut child=Process::new(std::env::var_os("BONDLINE_PYTHON").ok_or("pinned external Python")?)
        .args(["-B","-c",r#"
import base64,io,json,sys,PIL
from PIL import Image
assert PIL.__version__=='12.3.0'
p=json.load(sys.stdin)
expected=base64.b64decode(p['rgba'],validate=True)
assert len(expected)==640*640*4
assert len(p['pngs'])>=2
for encoded in p['pngs']:
 raw=base64.b64decode(encoded,validate=True)
 with Image.open(io.BytesIO(raw)) as im: im.verify()
 with Image.open(io.BytesIO(raw)) as im:
  assert im.format=='PNG' and im.mode=='RGBA' and im.size==(640,640)
  assert im.tobytes()==expected,'registered PNG differs from the analytically probed literal model frame'
"#]).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
    serde_json::to_writer(
        child.stdin.take().ok_or("Pillow stdin")?,
        &json!({"pngs":images,"rgba":STANDARD.encode(&frame.rgba)}),
    )?;
    let result = child.wait_with_output()?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(())
}
fn compare_page(actual: &Page, expected: &Page) -> TestResult {
    assert_eq!(actual.id(), expected.id());
    assert_eq!(actual.size(), expected.size());
    assert_eq!(actual.fragments().len(), expected.fragments().len());
    for (a, e) in actual.fragments().iter().zip(expected.fragments()) {
        assert_eq!(a.id(), e.id());
        assert_eq!(a.bonds(), e.bonds());
        assert_eq!(a.atoms().len(), e.atoms().len());
        for (a, e) in a.atoms().iter().zip(e.atoms()) {
            let p = a.position.ok_or("actual position")?;
            let q = e.position.ok_or("literal position")?;
            assert!((p.x_pt() - q.x_pt()).abs() <= COORD_TOL);
            assert!((p.y_pt() - q.y_pt()).abs() <= COORD_TOL);
            let mut a = a.clone();
            let mut e = e.clone();
            a.position = None;
            e.position = None;
            assert_eq!(a, e);
        }
    }
    Ok(())
}

#[test]
fn literal_regular_fused_and_spiro_single_ring_pages_are_readable_cpu_gpu_controls() -> TestResult {
    // Passing-first existing-renderer consistency control; no ring command is invoked.
    for case in [Case::Free, Case::Fused, Case::Spiro] {
        let stored = literal(case)?;
        let before = stored.clone();
        let page = &stored.model().pages()[0];
        geometry(page, case)?;
        let graph = graph_page_scene(DIM, DIM, page)?;
        let counts = page.fragments()[0]
            .atoms()
            .iter()
            .map(|a| Ok((a.id, a.explicit_hydrogens.ok_or("literal H")?)))
            .collect::<TestResult<BTreeMap<_, _>>>()?;
        let safe = graph_page_scene_with_display_hydrogens(
            DIM,
            DIM,
            page,
            &counts,
            &Cancellation::default(),
        )?;
        assert_eq!(
            graph.scene(),
            safe.scene(),
            "{} literal/same-count attachment scenes",
            case.label()
        );
        let frame = cpu_scene(graph.scene())?;
        analytic_ink(page, &graph, &frame, case)?;
        assert_eq!(stored, before);
        let mut engine = Engine::from_document(stored);
        let images = captures(&mut engine)?;
        pillow(&images, &frame)?;
    }
    Ok(())
}

#[test]
fn ring_create_captures_equal_independent_literal_regular_fused_and_spiro_pages() -> TestResult {
    // Command receiving comparison: any pre-implementation UnknownCommand is NOT a renderer RED.
    for case in [Case::Free, Case::Fused, Case::Spiro] {
        let reference = literal(case)?;
        let page = &reference.model().pages()[0];
        let graph = graph_page_scene(DIM, DIM, page)?;
        let frame = cpu_scene(graph.scene())?;
        analytic_ink(page, &graph, &frame, case)?;
        let mut actual = Engine::default();
        actual.dispatch(
            "page.set_size",
            json!({"page_id":1,"width_pt":288,"height_pt":288}),
        )?;
        let before = actual.state();
        actual.dispatch("ring.create",json!({"expected_revision":before.revision,"page_id":1,"size":6,
            "hydrogen_policy":"rectify_neutral_carbon","placement":{"mode":"free","begin_x_pt":120,"begin_y_pt":168,"end_x_pt":120,"end_y_pt":120,"side":"positive"}}))?;
        match case {
            Case::Free => {}
            Case::Fused => {
                actual.dispatch("ring.create",json!({"expected_revision":actual.state().revision,"page_id":1,"size":6,
                "hydrogen_policy":"rectify_neutral_carbon","placement":{"mode":"bond","fragment_id":1,"bond_id":1,"side":"negative"}}))?;
            }
            Case::Spiro => {
                actual.dispatch("ring.create",json!({"expected_revision":actual.state().revision,"page_id":1,"size":6,
                "hydrogen_policy":"rectify_neutral_carbon","placement":{"mode":"atom","fragment_id":1,"atom_id":1,"end_x_pt":120,"end_y_pt":216,"side":"positive"}}))?;
            }
        }
        let expected_edits = if matches!(case, Case::Free) { 1 } else { 2 };
        assert_eq!(actual.state().revision, before.revision + expected_edits);
        assert_eq!(
            actual.state().undo_depth,
            before.undo_depth + usize::try_from(expected_edits)?
        );
        geometry(&actual.document()?.pages()[0], case)?;
        compare_page(&actual.document()?.pages()[0], page)?;
        let mut reference = Engine::from_document(reference);
        let reference_images = captures(&mut reference)?;
        let actual_images = captures(&mut actual)?;
        assert_eq!(actual_images, reference_images);
        let images = [reference_images, actual_images].concat();
        pillow(&images, &frame)?;
    }
    Ok(())
}
