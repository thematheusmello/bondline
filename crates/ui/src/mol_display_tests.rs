//! Real headless app frames; literal source facts are independent of MOL projection.
use super::*;
use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Element, Fragment, FragmentId, Page, PageSize,
};
use bondline_render::{GraphScene, HitTarget, Primitive};
use egui::{Event, Modifiers, PointerButton, Pos2};
use serde_json::{Value, json};
use std::{
    fs,
    io::Write as _,
    process::{Command as Process, Stdio},
};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse only the existing isolated test directory")]
#[allow(
    clippy::duplicate_mod,
    reason = "the unchanged helper is private to sibling preview tests; keep old tests intact"
)]
mod support;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const SOURCE: &[u8] =
    include_bytes!("../../io/tests/fixtures/sdf-preview/valence-absent-mixed.sdf");
const CASES: [(&str, usize, usize); 2] = [("ethanol", 0, 371), ("methylamine", 410, 713)];

type LiteralFacts = (
    &'static [u8],
    &'static [[f64; 2]],
    &'static [u16],
    &'static [[u32; 2]],
);

fn facts(case: usize) -> LiteralFacts {
    if case == 0 {
        (
            &[6, 6, 8],
            &[[42.0, 60.0], [78.0, 60.0], [78.0, 24.0]],
            &[3, 2, 1],
            &[[1, 2], [2, 3]],
        )
    } else {
        (&[6, 7], &[[108.0, 108.0], [72.0, 72.0]], &[3, 2], &[[1, 2]])
    }
}

fn literal_engine(case: usize, hydrogens: &[Option<u16>]) -> TestResult<Engine> {
    let (elements, positions, _, edges) = facts(case);
    assert_eq!(elements.len(), hydrogens.len());
    let atoms = elements
        .iter()
        .zip(positions)
        .zip(hydrogens)
        .enumerate()
        .map(|(i, ((z, p), h))| {
            let mut atom = Atom::new(
                AtomId::new(u32::try_from(i + 1)?).ok_or("atom")?,
                Element::try_from(*z)?,
                Some(Point2::new(p[0], p[1])?),
            );
            atom.explicit_hydrogens = *h;
            Ok(atom)
        })
        .collect::<TestResult<Vec<_>>>()?;
    let bonds = edges
        .iter()
        .enumerate()
        .map(|(i, [a, b])| {
            Ok(Bond {
                id: BondId::new(u32::try_from(i + 1)?).ok_or("bond")?,
                begin: AtomId::new(*a).ok_or("begin")?,
                end: AtomId::new(*b).ok_or("end")?,
                order: BondOrder::Single,
            })
        })
        .collect::<TestResult<Vec<_>>>()?;
    let mut stored = Engine::default().stored_document()?.clone();
    stored.model_mut().append_fragments(
        PageId::new(1).ok_or("page")?,
        vec![Fragment::new(
            FragmentId::new(1).ok_or("fragment")?,
            atoms,
            bonds,
        )?],
    )?;
    Ok(Engine::from_document(stored))
}
fn page(engine: &Engine) -> TestResult<&Page> {
    Ok(engine.document()?.pages().first().ok_or("page")?)
}
fn assert_model(engine: &Engine, case: usize, hydrogens: &[Option<u16>]) -> TestResult {
    let (elements, positions, _, edges) = facts(case);
    assert_eq!(page(engine)?.size(), PageSize::A4);
    let fragments = page(engine)?.fragments();
    assert_eq!(fragments.len(), 1);
    let fragment = fragments.first().ok_or("fragment")?;
    assert_eq!(fragment.id().get(), 1);
    let actual = fragment
        .atoms()
        .iter()
        .map(|a| {
            Ok((
                a.id.get(),
                a.element.get(),
                [
                    a.position.ok_or("position")?.x_pt(),
                    a.position.ok_or("position")?.y_pt(),
                ],
                a.explicit_hydrogens,
            ))
        })
        .collect::<TestResult<Vec<_>>>()?;
    let expected = elements
        .iter()
        .zip(positions)
        .zip(hydrogens)
        .enumerate()
        .map(|(i, ((z, p), h))| Ok((u32::try_from(i + 1)?, *z, *p, *h)))
        .collect::<TestResult<Vec<_>>>()?;
    assert_eq!(actual, expected);
    assert_eq!(
        fragment
            .bonds()
            .iter()
            .map(|b| [b.begin.get(), b.end.get()])
            .collect::<Vec<_>>(),
        edges
    );
    Ok(())
}

struct Fixture {
    app: BondlineApp,
    context: egui::Context,
    directory: support::FileTestDirectory,
}
impl Fixture {
    fn new(mut engine: Engine, directory: support::FileTestDirectory) -> Self {
        engine.set_ui_file_host(true);
        let engine = Arc::new(Mutex::new(engine));
        let app = BondlineApp {
            smoke_frames: None,
            engine: Arc::clone(&engine),
            executor: CommandExecutor::new(engine),
            control: None,
            error: None,
            drawing: DrawingState::default(),
            files: FileWorkflow::new(directory.path(".")),
            preview: PreviewWorkflow::default(),
            sdf_fields: crate::sdf_fields::SdfFieldsWorkflow::default(),
        };
        let context = egui::Context::default();
        configure_context(&context);
        Self {
            app,
            context,
            directory,
        }
    }
    fn mol(case: usize) -> TestResult<(Self, std::path::PathBuf)> {
        let directory = support::FileTestDirectory::new()?;
        let (name, start, end) = CASES[case];
        let input = directory.path(&format!("{name}-λ.mol"));
        fs::write(&input, SOURCE.get(start..end).ok_or("literal MOL span")?)?;
        let mut engine = Engine::default();
        engine.dispatch("document.open_mol", json!({"path":input}))?;
        Ok((Self::new(engine, directory), input))
    }
    fn frame(&mut self, events: Vec<Event>) -> egui::FullOutput {
        let mut input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                Pos2::ZERO,
                egui::vec2(1600.0, 1400.0),
            )),
            events,
            focused: true,
            ..Default::default()
        };
        input
            .viewports
            .entry(egui::ViewportId::ROOT)
            .or_default()
            .native_pixels_per_point = Some(1.0);
        let context = self.context.clone();
        context.run(input, |ctx| self.app.update_workspace(ctx))
    }
    fn state(&self) -> TestResult<Value> {
        Ok(serde_json::to_value(
            self.app.engine.lock().map_err(|_| "engine")?.state(),
        )?)
    }
}

#[derive(Debug, PartialEq)]
enum Painted {
    Rect(egui::Rect, egui::Color32),
    Stroke(Vec<Pos2>, egui::Color32),
}
fn canvas(output: &egui::FullOutput) -> TestResult<egui::Rect> {
    output
        .shapes
        .iter()
        .find_map(|s| match &s.shape {
            egui::Shape::Rect(r) if r.fill == egui::Color32::from_gray(196) => Some(s.clip_rect),
            _ => None,
        })
        .ok_or_else(|| "actual painted paper/clip missing".into())
}
fn actual(output: &egui::FullOutput, canvas: egui::Rect) -> Vec<Painted> {
    output
        .shapes
        .iter()
        .filter(|s| s.clip_rect == canvas)
        .filter_map(|s| match &s.shape {
            egui::Shape::Rect(r) if r.fill != egui::Color32::from_gray(232) => {
                Some(Painted::Rect(r.rect, r.fill))
            }
            egui::Shape::Path(p) if p.fill == egui::Color32::BLACK => {
                assert!(p.closed);
                Some(Painted::Stroke(p.points.clone(), p.fill))
            }
            _ => None,
        })
        .collect()
}
fn expected_shapes(graph: &GraphScene, canvas: egui::Rect) -> TestResult<Vec<Painted>> {
    graph
        .scene()
        .primitives
        .iter()
        .map(|p| {
            Ok(match p {
                Primitive::Rectangle(r) => Painted::Rect(
                    egui::Rect::from_min_size(
                        canvas.min + egui::vec2(r.x as f32, r.y as f32),
                        egui::vec2(r.width as f32, r.height as f32),
                    ),
                    egui::Color32::from_rgba_unmultiplied(
                        r.color[0], r.color[1], r.color[2], r.color[3],
                    ),
                ),
                Primitive::Stroke(s) => Painted::Stroke(
                    s.corners()?
                        .into_iter()
                        .map(|[x, y]| canvas.min + egui::vec2(x, y))
                        .collect(),
                    egui::Color32::BLACK,
                ),
            })
        })
        .collect()
}
fn graph(engine: &Engine, canvas: egui::Rect) -> TestResult<GraphScene> {
    // This protected literal renderer is intentionally independent of the new IO route.
    Ok(bondline_render::graph_page_scene(
        canvas.width().floor() as u32,
        canvas.height().floor() as u32,
        page(engine)?,
    )?)
}
fn assert_vectors(
    output: &egui::FullOutput,
    literal: &Engine,
) -> TestResult<(egui::Rect, GraphScene)> {
    let canvas = canvas(output)?;
    let graph = graph(literal, canvas)?;
    let a = actual(output, canvas);
    let b = expected_shapes(&graph, canvas)?;
    assert!(
        a == b,
        "actual app canvas differs from literal fixed-source expectation: {} vs{} primitives; first difference {:?}",
        a.len(),
        b.len(),
        a.iter().zip(&b).position(|(a, b)| a != b)
    );
    Ok((canvas, graph))
}
fn button(p: Pos2, pressed: bool) -> Event {
    Event::PointerButton {
        pos: p,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    }
}
fn h_point(case: usize, canvas: egui::Rect, drawn: &GraphScene) -> TestResult<(Pos2, HitTarget)> {
    let target = HitTarget::Atom {
        fragment_id: FragmentId::new(1).ok_or("fragment")?,
        atom_id: AtomId::new(if case == 0 { 3 } else { 2 }).ok_or("atom")?,
    };
    let (_, positions, _, _) = facts(case);
    let p = positions.last().ok_or("heteroatom")?;
    let viewport = bondline_render::PageViewport::new(
        canvas.width().floor() as u32,
        canvas.height().floor() as u32,
        PageSize::A4,
    )?;
    let anchor = viewport.canvas_position(Point2::new(p[0], p[1])?)?;
    let unknown = literal_engine(case, &vec![None; positions.len()])?;
    let bare = graph(&unknown, canvas)?;
    let em = 12.0 * viewport.stroke_width();
    for primitive in &drawn.scene().primitives {
        if let Primitive::Rectangle(r) = primitive
            && r.color == [0, 0, 0, 255]
        {
            let point = [r.x as f32 + r.width as f32 / 2.0, r.y as f32 + 0.5];
            if point[0] < anchor[0] - 6.0
                && point[0] > anchor[0] - 3.0 * em
                && (point[1] - anchor[1]).abs() < em
                && drawn.hit_test(point, 0.0)? == Some(target)
                && bare.hit_test(point, 5.0)? != Some(target)
            {
                return Ok((canvas.min + egui::vec2(point[0], point[1]), target));
            }
        }
    }
    Err("independent H ink away from legacy bare atom target".into())
}
fn molecule_oracle(raw: &[u8], case: usize) -> TestResult {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child = Process::new(python)
        .arg("-B")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../io/tests/mol_oracle.py"
        ))
        .arg("describe")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child.stdin.take().ok_or("oracle stdin")?.write_all(raw)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(value["smiles"], if case == 0 { "CCO" } else { "CN" });
    assert_eq!(value["formula"], if case == 0 { "C2H6O" } else { "CH5N" });
    let atoms = value["atoms"].as_array().ok_or("external atoms")?;
    assert_eq!(
        atoms
            .iter()
            .map(|a| a["hydrogens"].clone())
            .collect::<Vec<_>>(),
        facts(case).2.iter().map(|h| json!(*h)).collect::<Vec<_>>()
    );
    let expected = if case == 0 {
        json!([[-1.25, 0.5, 0.0], [0.25, 0.5, 0.0], [0.25, 2.0, 0.0]])
    } else {
        json!([[1.5, -1.5, 0.0], [0.0, 0.0, 0.0]])
    };
    assert_eq!(
        json!(
            atoms
                .iter()
                .map(|a| a["position"].clone())
                .collect::<Vec<_>>()
        ),
        expected
    );
    Ok(())
}

#[test]
fn ordinary_source_mol_paints_real_oh_nh2_and_hits_without_authoring_h() -> TestResult {
    for (case, &(_, start, end)) in CASES.iter().enumerate() {
        let (mut fixture, input) = Fixture::mol(case)?;
        let raw = SOURCE.get(start..end).ok_or("span")?;
        let (_, positions, h, _) = facts(case);
        let none = vec![None; positions.len()];
        {
            let engine = fixture.app.engine.lock().map_err(|_| "engine")?;
            assert!(engine.stored_document()?.is_fully_mapped());
            assert_model(&engine, case, &none)?;
        }
        let before = fixture.state()?;
        fixture.frame(vec![]);
        let output = fixture.frame(vec![]);
        let literal = literal_engine(case, &h.iter().copied().map(Some).collect::<Vec<_>>())?;
        let (canvas, expected) = assert_vectors(&output, &literal)?;
        let bare = literal_engine(case, &none)?;
        assert!(
            actual(&output, canvas) != expected_shapes(&graph(&bare, canvas)?, canvas)?,
            "missing H counterfactual accepted"
        );
        let (point, target) = h_point(case, canvas, &expected)?;
        fixture.frame(vec![Event::PointerMoved(point)]);
        assert_eq!(fixture.app.drawing.hover_target(), Some(target));
        assert_eq!(fixture.state()?, before);
        assert!(fixture.app.error.is_none());
        let saved = fixture.directory.path("unchanged.mol");
        {
            let mut engine = fixture.app.engine.lock().map_err(|_| "engine")?;
            engine.dispatch("document.save_mol", json!({"path":saved}))?;
            assert_model(&engine, case, &none)?;
            assert_eq!(engine.state().undo_depth, 0);
        }
        assert_eq!(fs::read(&input)?, raw);
        assert_eq!(fs::read(&saved)?, raw);
        molecule_oracle(&fs::read(&saved)?, case)?;
        {
            let mut engine = fixture.app.engine.lock().map_err(|_| "engine")?;
            engine.dispatch("document.open_mol", json!({"path":saved}))?;
            assert_model(&engine, case, &none)?;
            assert_eq!(engine.state().undo_depth, 0);
        }
        let reopened = fixture.state()?;
        let output = fixture.frame(vec![]);
        assert_vectors(&output, &literal)?;
        assert_eq!(fixture.state()?, reopened);
    }
    Ok(())
}

#[test]
fn same_frame_rejected_mol_edit_reacquires_source_aware_geometry_and_h_hits() -> TestResult {
    for case in 0..2 {
        let (mut fixture, _) = Fixture::mol(case)?;
        fixture.frame(vec![]);
        let initial = fixture.frame(vec![]);
        let (_, _, h, _) = facts(case);
        let literal = literal_engine(case, &h.iter().copied().map(Some).collect::<Vec<_>>())?;
        let canvas = canvas(&initial)?;
        let expected = graph(&literal, canvas)?;
        let (hydrogen, target) = h_point(case, canvas, &expected)?;
        let viewport = bondline_render::PageViewport::new(
            canvas.width().floor() as u32,
            canvas.height().floor() as u32,
            PageSize::A4,
        )?;
        let to_canvas = |x, y| -> TestResult<Pos2> {
            let [x, y] = viewport.canvas_position(Point2::new(x, y)?)?;
            Ok(canvas.min + egui::vec2(x, y))
        };
        let carbon = facts(case).1.first().ok_or("terminal carbon")?;
        let begin = to_canvas(carbon[0], carbon[1])?;
        let end = to_canvas(300.0, 500.0)?;
        let before = fixture.state()?;
        let output = fixture.frame(vec![
            Event::PointerMoved(begin),
            button(begin, true),
            Event::PointerMoved(end),
            button(end, false),
            Event::PointerMoved(hydrogen),
            button(hydrogen, true),
        ]);
        assert_eq!(
            fixture.app.error.as_deref(),
            Some(
                bondline_commands::Error::UnsupportedExtensionSource
                    .to_string()
                    .as_str()
            )
        );
        assert_eq!(
            fixture.state()?,
            before,
            "failed MOL edit changed source/history/association"
        );
        assert_vectors(&output, &literal)?;
        assert_eq!(
            fixture
                .app
                .drawing
                .gesture
                .as_ref()
                .ok_or("fresh same-frame H press")?
                .target,
            Some(target),
            "post-command graph lost the H label hit region"
        );
        fixture.frame(vec![button(hydrogen, false)]);
    }
    Ok(())
}

#[test]
fn source_free_and_native_cdxml_keep_unknown_zero_and_unusual_literal_h_policy() -> TestResult {
    for hydrogens in [vec![None, None], vec![Some(0), Some(12)]] {
        let directory = support::FileTestDirectory::new()?;
        let cdxml = directory.path("literal.cdxml");
        let literal = literal_engine(1, &hydrogens)?;
        let mut fixture = Fixture::new(literal_engine(1, &hydrogens)?, directory);
        let before = fixture.state()?;
        fixture.frame(vec![]);
        let output = fixture.frame(vec![]);
        assert_vectors(&output, &literal)?;
        assert_eq!(fixture.state()?, before);
        {
            let mut engine = fixture.app.engine.lock().map_err(|_| "engine")?;
            engine.dispatch("document.save_cdxml", json!({"path":cdxml}))?;
            engine.dispatch("document.open_cdxml", json!({"path":cdxml}))?;
            assert!(engine.stored_document()?.is_fully_mapped());
            assert_model(&engine, 1, &hydrogens)?;
        }
        let before = fixture.state()?;
        let output = fixture.frame(vec![]);
        assert_vectors(&output, &literal)?;
        assert_eq!(fixture.state()?, before);
    }
    Ok(())
}
