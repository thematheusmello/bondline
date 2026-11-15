//! Independent ordered-frame history regressions; no native input or reference pixels.
use super::*;
use bondline_model::{BondOrder, PageSize};
use egui::{Event, Key, Modifiers, PointerButton, Pos2};
use serde_json::{Value, json};
use std::fs;

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(
    dead_code,
    clippy::duplicate_mod,
    reason = "reuse the existing private test directory helper without changing other tests"
)]
mod support;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

struct Workspace {
    engine: Engine,
    context: egui::Context,
    drawing: DrawingState,
    error: Option<String>,
    directory: support::FileTestDirectory,
}

impl Workspace {
    fn with_engine(engine: Engine, directory: support::FileTestDirectory) -> Self {
        let context = egui::Context::default();
        configure_context(&context);
        Self {
            engine,
            context,
            drawing: DrawingState::default(),
            error: None,
            directory,
        }
    }

    fn pair(native_source: bool) -> TestResult<Self> {
        let directory = support::FileTestDirectory::new()?;
        let mut engine = Engine::default();
        engine.dispatch(
            "page.set_size",
            json!({"page_id":1,"width_pt":128,"height_pt":128}),
        )?;
        engine.dispatch(
            "bond.create",
            json!({"page_id":1,"begin_x_pt":40,"begin_y_pt":64,
                   "end_x_pt":88,"end_y_pt":64}),
        )?;
        if native_source {
            let path = directory.path("native-history.cdxml");
            engine.dispatch("document.save_cdxml", json!({"path":path}))?;
            engine.dispatch("document.open_cdxml", json!({"path":path}))?;
            assert_eq!(engine.state().undo_depth, 0);
            assert!(!engine.state().file.dirty);
        }
        Ok(Self::with_engine(engine, directory))
    }

    fn frame(&mut self, events: Vec<Event>) -> egui::FullOutput {
        let screen = egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(1600.0, 1400.0));
        let mut input = egui::RawInput {
            screen_rect: Some(screen),
            focused: true,
            events,
            ..Default::default()
        };
        let viewport = input.viewports.entry(egui::ViewportId::ROOT).or_default();
        viewport.native_pixels_per_point = Some(1.0);
        viewport.inner_rect = Some(egui::Rect::from_min_size(
            egui::pos2(100.0, 100.0),
            screen.size(),
        ));
        let context = self.context.clone();
        context.run(input, |ctx| {
            show_workspace(ctx, &mut self.engine, &mut self.error, &mut self.drawing);
        })
    }

    fn warm(&mut self) -> egui::FullOutput {
        self.frame(vec![]);
        self.frame(vec![])
    }

    fn state(&self) -> TestResult<Value> {
        Ok(serde_json::to_value(self.engine.state())?)
    }

    fn point(&self, output: &egui::FullOutput, page: PageSize, p: [f64; 2]) -> TestResult<Pos2> {
        let rect = canvas(output)?;
        let viewport = bondline_render::PageViewport::new(
            rect.width().floor() as u32,
            rect.height().floor() as u32,
            page,
        )?;
        let [x, y] = viewport.canvas_position(Point2::new(p[0], p[1])?)?;
        Ok(rect.min + egui::vec2(x, y))
    }

    fn assert_pair(&self, order: BondOrder, h: u16) -> TestResult {
        let page = self.engine.document()?.pages().first().ok_or("page")?;
        assert_eq!(page.size(), PageSize::new(128.0, 128.0)?);
        assert_eq!(page.fragments().len(), 1);
        let fragment = page.fragments().first().ok_or("fragment")?;
        assert_eq!(fragment.id().get(), 1);
        assert_eq!(fragment.atoms().len(), 2);
        assert_eq!(fragment.bonds().len(), 1);
        for (atom, (id, x)) in fragment.atoms().iter().zip([(1, 40.0), (2, 88.0)]) {
            assert_eq!(atom.id.get(), id);
            assert_eq!(atom.position, Some(Point2::new(x, 64.0)?));
            assert_eq!(atom.explicit_hydrogens, Some(h));
        }
        let bond = fragment.bonds().first().ok_or("bond")?;
        assert_eq!((bond.id.get(), bond.begin.get(), bond.end.get()), (1, 1, 2));
        assert_eq!(bond.order, order);
        Ok(())
    }
}

fn canvas(output: &egui::FullOutput) -> TestResult<egui::Rect> {
    output
        .shapes
        .iter()
        .find_map(|paint| match &paint.shape {
            egui::Shape::Rect(rect) if rect.fill == egui::Color32::from_gray(196) => {
                Some(paint.clip_rect)
            }
            _ => None,
        })
        .ok_or_else(|| "real paper/canvas missing".into())
}

fn tap(key: Key, modifiers: Modifiers) -> [Event; 2] {
    [true, false].map(|pressed| Event::Key {
        key,
        physical_key: Some(key),
        pressed,
        repeat: false,
        modifiers,
    })
}

fn undo() -> [Event; 2] {
    tap(Key::Z, Modifiers::COMMAND)
}

fn redo() -> [Event; 2] {
    tap(Key::Z, Modifiers::COMMAND | Modifiers::SHIFT)
}

fn pointer(pos: Pos2, pressed: bool) -> Event {
    Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    }
}

#[test]
fn numeric_order_then_undo_preserves_the_prior_complete_source_and_retires_the_hotspot()
-> TestResult {
    for native in [false, true] {
        let mut workspace = Workspace::pair(native)?;
        let output = workspace.warm();
        let point = workspace.point(&output, PageSize::new(128.0, 128.0)?, [64.0, 64.0])?;
        let before = workspace.engine.content().clone();
        let state = workspace.state()?;
        let mode = workspace.engine.bond_tool();
        let revision = workspace.engine.state().revision;
        let depth = workspace.engine.state().undo_depth;
        workspace.frame(
            [Event::PointerMoved(point)]
                .into_iter()
                .chain(tap(Key::Num2, Modifiers::NONE))
                .chain(undo())
                .collect(),
        );
        assert!(workspace.error.is_none(), "{:?}", workspace.error);
        assert_eq!(workspace.engine.content(), &before);
        workspace.assert_pair(BondOrder::Single, 3)?;
        assert_eq!(workspace.engine.state().revision, revision + 2);
        assert_eq!(workspace.engine.state().undo_depth, depth);
        assert_eq!(workspace.engine.state().redo_depth, 1);
        assert_eq!(workspace.state()?["source"], state["source"]);
        assert_eq!(workspace.state()?["file"], state["file"]);
        assert_eq!(workspace.engine.bond_tool(), mode);
        if native {
            assert_eq!(
                workspace.engine.stored_document()?.to_cdxml()?.as_ref(),
                fs::read(workspace.directory.path("native-history.cdxml"))?
            );
        }
        let after_history = workspace.state()?;
        workspace.frame(tap(Key::Num3, Modifiers::NONE).to_vec());
        assert_eq!(
            workspace.state()?,
            after_history,
            "history invalidates the old hotspot"
        );
        workspace.frame(
            [Event::PointerMoved(point)]
                .into_iter()
                .chain(tap(Key::Num3, Modifiers::NONE))
                .collect(),
        );
        workspace.assert_pair(BondOrder::Triple, 1)?;
    }
    Ok(())
}

#[test]
fn click_release_then_undo_reverts_the_click_instead_of_the_seed() -> TestResult {
    let mut workspace = Workspace::pair(false)?;
    let output = workspace.warm();
    let point = workspace.point(&output, PageSize::new(128.0, 128.0)?, [64.0, 64.0])?;
    let before = workspace.engine.content().clone();
    let revision = workspace.engine.state().revision;
    let depth = workspace.engine.state().undo_depth;
    workspace.frame(
        [
            Event::PointerMoved(point),
            pointer(point, true),
            pointer(point, false),
        ]
        .into_iter()
        .chain(undo())
        .collect(),
    );
    assert!(workspace.error.is_none(), "{:?}", workspace.error);
    assert_eq!(workspace.engine.content(), &before);
    workspace.assert_pair(BondOrder::Single, 3)?;
    assert_eq!(workspace.engine.state().revision, revision + 2);
    assert_eq!(workspace.engine.state().undo_depth, depth);
    assert_eq!(workspace.engine.state().redo_depth, 1);
    Ok(())
}

#[test]
fn order_undo_redo_in_one_packet_uses_history_availability_at_each_event() -> TestResult {
    let mut workspace = Workspace::pair(true)?;
    let output = workspace.warm();
    let point = workspace.point(&output, PageSize::new(128.0, 128.0)?, [64.0, 64.0])?;
    let revision = workspace.engine.state().revision;
    let source = workspace.state()?["source"].clone();
    workspace.frame(
        [Event::PointerMoved(point)]
            .into_iter()
            .chain(tap(Key::Num2, Modifiers::NONE))
            .chain(undo())
            .chain(redo())
            .collect(),
    );
    assert!(workspace.error.is_none(), "{:?}", workspace.error);
    workspace.assert_pair(BondOrder::Double, 2)?;
    assert_eq!(workspace.engine.state().revision, revision + 3);
    assert_eq!(
        (
            workspace.engine.state().undo_depth,
            workspace.engine.state().redo_depth
        ),
        (1, 0)
    );
    assert!(workspace.engine.state().file.dirty);
    let after_source = workspace.state()?["source"].clone();
    assert_eq!(after_source["format"], source["format"]);
    assert_eq!(after_source["byte_length"], source["byte_length"]);
    assert_eq!(after_source["modified"], true);
    Ok(())
}

#[test]
fn history_restored_page_size_is_used_by_a_later_same_packet_pointer_gesture() -> TestResult {
    let mut workspace = Workspace::pair(false)?;
    workspace.engine.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":256,"height_pt":128}),
    )?;
    let output = workspace.warm();
    // Pointer coordinates describe the square page restored by Undo, not the current wide page.
    let square = PageSize::new(128.0, 128.0)?;
    let begin = workspace.point(&output, square, [80.0, 100.0])?;
    let end = workspace.point(&output, square, [104.0, 100.0])?;
    let revision = workspace.engine.state().revision;
    let depth = workspace.engine.state().undo_depth;
    workspace.frame(
        undo()
            .into_iter()
            .chain([
                Event::PointerMoved(begin),
                pointer(begin, true),
                Event::PointerMoved(end),
                pointer(end, false),
            ])
            .collect(),
    );
    assert!(workspace.error.is_none(), "{:?}", workspace.error);
    let page = workspace.engine.document()?.pages().first().ok_or("page")?;
    assert_eq!(page.size(), square);
    assert_eq!(page.fragments().len(), 2);
    let fragment = page.fragments().get(1).ok_or("new fragment")?;
    assert_eq!(fragment.id().get(), 2);
    assert_eq!(fragment.bonds().len(), 1);
    for (atom, (id, x)) in fragment.atoms().iter().zip([(3, 80.0), (4, 104.0)]) {
        assert_eq!(atom.id.get(), id);
        let p = atom.position.ok_or("point")?;
        assert!((p.x_pt() - x).abs() < 0.0001 && (p.y_pt() - 100.0).abs() < 0.0001);
        assert_eq!(atom.explicit_hydrogens, Some(3));
    }
    assert_eq!(workspace.engine.state().revision, revision + 2);
    assert_eq!(workspace.engine.state().undo_depth, depth);
    assert_eq!(workspace.engine.state().redo_depth, 0);
    Ok(())
}

#[test]
fn sdf_undo_redo_fallback_retains_the_whole_collection_without_a_drawing() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let path = directory.path("whole-history.sdf");
    let raw = include_bytes!("../../io/tests/fixtures/sdf/ordered-fields-v2000.sdf");
    fs::write(&path, raw)?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_sdf", json!({"path":path}))?;
    let revision = engine.state().revision;
    engine.dispatch("sdf.field.set_value", json!({"expected_revision":revision,"record_id":1,"field_index":5,"value_lines":["History test"]}))?;
    let mut workspace = Workspace::with_engine(engine, directory);
    workspace.warm();
    let before = workspace.engine.content().clone();
    let state = workspace.state()?;
    let revision = workspace.engine.state().revision;
    assert_eq!(state["document"], Value::Null);
    assert_eq!(state["collection"]["record_count"], 2);
    workspace.frame(undo().into_iter().chain(redo()).collect());
    assert!(workspace.error.is_none(), "{:?}", workspace.error);
    assert_eq!(workspace.engine.content(), &before);
    assert_eq!(workspace.engine.state().revision, revision + 2);
    assert_eq!(
        (
            workspace.engine.state().undo_depth,
            workspace.engine.state().redo_depth
        ),
        (1, 0)
    );
    let after = workspace.state()?;
    for field in ["document", "collection", "source", "file"] {
        assert_eq!(after[field], state[field]);
    }
    assert_eq!(fs::read(&path)?, raw);
    Ok(())
}

#[test]
fn numeric_order_before_an_undo_menu_click_preserves_event_order() -> TestResult {
    let mut workspace = Workspace::pair(false)?;
    let output = workspace.warm();
    let bond = workspace.point(&output, PageSize::new(128.0, 128.0)?, [64.0, 64.0])?;
    let text_center = |output: &egui::FullOutput, label: &str| -> TestResult<Pos2> {
        output
            .shapes
            .iter()
            .find_map(|paint| match &paint.shape {
                egui::Shape::Text(text) if text.galley.text() == label => {
                    Some(text.galley.rect.translate(text.pos.to_vec2()).center())
                }
                _ => None,
            })
            .ok_or_else(|| format!("visible {label} text missing").into())
    };
    let edit = text_center(&output, "Edit")?;
    workspace.frame(vec![
        Event::PointerMoved(edit),
        pointer(edit, true),
        pointer(edit, false),
    ]);
    let menu = workspace.frame(vec![]);
    let undo_item = text_center(&menu, "Undo")?;
    let before = workspace.engine.content().clone();
    let revision = workspace.engine.state().revision;
    let depth = workspace.engine.state().undo_depth;
    workspace.frame(
        [Event::PointerMoved(bond)]
            .into_iter()
            .chain(tap(Key::Num2, Modifiers::NONE))
            .chain([
                Event::PointerMoved(undo_item),
                pointer(undo_item, true),
                pointer(undo_item, false),
            ])
            .collect(),
    );
    assert!(workspace.error.is_none(), "{:?}", workspace.error);
    assert_eq!(workspace.engine.content(), &before);
    workspace.assert_pair(BondOrder::Single, 3)?;
    assert_eq!(workspace.engine.state().revision, revision + 2);
    assert_eq!(workspace.engine.state().undo_depth, depth);
    assert_eq!(workspace.engine.state().redo_depth, 1);
    Ok(())
}
