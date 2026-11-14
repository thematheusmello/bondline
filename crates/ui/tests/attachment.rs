use bondline_commands::Engine;
use bondline_model::Point2;
use bondline_ui::{DrawingState, configure_context, show_workspace};
use eframe::egui::{self, Event, Key, Modifiers, PointerButton, Pos2};
use serde_json::json;
use std::{fs, process::Command as Process};
#[path = "../../../xtask/oracles/files_test_support.rs"]
mod file_support;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

struct Workspace {
    context: egui::Context,
    engine: Engine,
    drawing: DrawingState,
    error: Option<String>,
    screen: egui::Rect,
    dpi: f32,
    host_origin: Pos2,
    focused: bool,
}

impl Workspace {
    fn new(seed: bool) -> TestResult<Self> {
        let context = egui::Context::default();
        configure_context(&context);
        let mut engine = Engine::default();
        if seed {
            engine.dispatch("bond.create",json!({"page_id":1,"begin_x_pt":100.0,"begin_y_pt":200.0,"end_x_pt":140.0,"end_y_pt":200.0}))?;
        }
        Ok(Self {
            context,
            engine,
            drawing: DrawingState::default(),
            error: None,
            screen: egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(1000.0, 900.0)),
            dpi: 1.0,
            host_origin: Pos2::new(100.0, 100.0),
            focused: true,
        })
    }

    fn frame(&mut self, events: Vec<Event>, cover: Option<egui::Rect>) -> egui::FullOutput {
        let mut input = egui::RawInput {
            screen_rect: Some(self.screen),
            events,
            focused: self.focused,
            ..Default::default()
        };
        let viewport = input.viewports.entry(egui::ViewportId::ROOT).or_default();
        viewport.native_pixels_per_point = Some(self.dpi);
        viewport.inner_rect = Some(egui::Rect::from_min_size(
            self.host_origin,
            self.screen.size(),
        ));
        self.context.begin_pass(input);
        if let Some(rect) = cover {
            egui::Area::new(egui::Id::new("attachment-foreground"))
                .order(egui::Order::Foreground)
                .fixed_pos(rect.min)
                .show(&self.context, |ui| {
                    ui.allocate_exact_size(rect.size(), egui::Sense::hover());
                });
        }
        show_workspace(
            &self.context,
            &mut self.engine,
            &mut self.error,
            &mut self.drawing,
        );
        self.context.end_pass()
    }

    fn points(&mut self) -> TestResult<[Pos2; 4]> {
        let output = self.frame(vec![], None);
        assert!(self.error.is_none(), "{:?}", self.error);
        let paper = paper(&output)?;
        let size = self.engine.document().pages().first().ok_or("page")?.size();
        Ok([
            (100.0, 200.0),
            (140.0, 200.0),
            (180.0, 240.0),
            (220.0, 280.0),
        ]
        .map(|(x, y)| {
            paper.min
                + egui::vec2(
                    paper.width() * x / size.width_pt() as f32,
                    paper.height() * y / size.height_pt() as f32,
                )
        }))
    }
}

fn paper(output: &egui::FullOutput) -> TestResult<egui::Rect> {
    output
        .shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            egui::Shape::Rect(rect) if rect.fill == egui::Color32::from_gray(196) => {
                Some(rect.rect)
            }
            _ => None,
        })
        .ok_or_else(|| "paper border".into())
}

fn button(pos: Pos2, pressed: bool) -> Event {
    Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    }
}

fn drag(begin: Pos2, end: Pos2) -> Vec<Event> {
    vec![
        Event::PointerMoved(begin),
        button(begin, true),
        Event::PointerMoved(end),
        button(end, false),
    ]
}

fn escape(pressed: bool) -> Event {
    Event::Key {
        key: Key::Escape,
        physical_key: Some(Key::Escape),
        pressed,
        repeat: false,
        modifiers: Modifiers::NONE,
    }
}

fn label_point(output: &egui::FullOutput, anchor: Pos2) -> TestResult<Pos2> {
    output
        .shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            egui::Shape::Rect(rect)
                if rect.fill == egui::Color32::WHITE
                    && rect.rect.area() < 1000.0
                    && rect.rect.contains(anchor) =>
            {
                let inside = rect.rect.shrink(1.0);
                [
                    inside.left_top(),
                    inside.left_bottom(),
                    inside.right_top(),
                    inside.right_bottom(),
                ]
                .into_iter()
                .find(|point| point.distance(anchor) > 6.0)
            }
            _ => None,
        })
        .ok_or_else(|| "visible label point away from atom anchor".into())
}

#[test]
fn atom_drag_extends_same_fragment_with_exact_history_and_literal_hydrogens() -> TestResult {
    let mut workspace = Workspace::new(true)?;
    let [_, begin, end, _] = workspace.points()?;
    let before = workspace.engine.stored_document().clone();
    workspace.frame(drag(begin, end), None);
    assert!(workspace.error.is_none(), "{:?}", workspace.error);
    let fragments = workspace
        .engine
        .document()
        .pages()
        .first()
        .ok_or("page")?
        .fragments();
    assert_eq!(
        fragments.len(),
        1,
        "an attachment cannot create a second independent C-C fragment"
    );
    let fragment = fragments.first().ok_or("fragment")?;
    assert_eq!(
        (
            fragment.id().get(),
            fragment.atoms().len(),
            fragment.bonds().len()
        ),
        (1, 3, 2)
    );
    assert_eq!(
        fragment
            .atoms()
            .iter()
            .map(|atom| (atom.id.get(), atom.explicit_hydrogens))
            .collect::<Vec<_>>(),
        [(1, Some(3)), (2, Some(2)), (3, Some(3))]
    );
    assert_eq!(
        fragment.atoms().get(1).ok_or("anchor")?.position,
        Some(Point2::new(140.0, 200.0)?)
    );
    let added = fragment
        .atoms()
        .get(2)
        .and_then(|atom| atom.position)
        .ok_or("added point")?;
    assert!((added.x_pt() - 180.0).abs() < 0.0001 && (added.y_pt() - 240.0).abs() < 0.0001);
    assert_eq!(
        fragment
            .bonds()
            .iter()
            .map(|bond| (bond.id.get(), bond.begin.get(), bond.end.get()))
            .collect::<Vec<_>>(),
        [(1, 1, 2), (2, 2, 3)]
    );
    assert_eq!(
        (
            workspace.engine.state().revision,
            workspace.engine.state().undo_depth
        ),
        (2, 2)
    );
    let after = workspace.engine.stored_document().clone();
    workspace.engine.dispatch("history.undo", json!({}))?;
    assert_eq!(workspace.engine.stored_document(), &before);
    workspace.engine.dispatch("history.redo", json!({}))?;
    assert_eq!(workspace.engine.stored_document(), &after);
    Ok(())
}

#[test]
fn two_batched_attachments_pick_the_atom_created_by_the_prior_release() -> TestResult {
    let mut workspace = Workspace::new(true)?;
    let [_, begin, middle, end] = workspace.points()?;
    let before = workspace.engine.stored_document().clone();
    let events = drag(begin, middle)
        .into_iter()
        .chain(drag(middle, end))
        .collect();
    workspace.frame(events, None);
    assert!(workspace.error.is_none(), "{:?}", workspace.error);
    let fragments = workspace
        .engine
        .document()
        .pages()
        .first()
        .ok_or("page")?
        .fragments();
    assert_eq!(fragments.len(), 1);
    let fragment = fragments.first().ok_or("fragment")?;
    assert_eq!(
        fragment
            .atoms()
            .iter()
            .map(|atom| (atom.id.get(), atom.explicit_hydrogens))
            .collect::<Vec<_>>(),
        [(1, Some(3)), (2, Some(2)), (3, Some(2)), (4, Some(3))]
    );
    assert_eq!(
        fragment
            .bonds()
            .iter()
            .map(|bond| (bond.begin.get(), bond.end.get()))
            .collect::<Vec<_>>(),
        [(1, 2), (2, 3), (3, 4)]
    );
    assert_eq!(
        (
            workspace.engine.state().revision,
            workspace.engine.state().undo_depth
        ),
        (3, 3)
    );
    let after = workspace.engine.stored_document().clone();
    workspace.engine.dispatch("history.undo", json!({}))?;
    workspace.engine.dispatch("history.undo", json!({}))?;
    assert_eq!(workspace.engine.stored_document(), &before);
    workspace.engine.dispatch("history.redo", json!({}))?;
    workspace.engine.dispatch("history.redo", json!({}))?;
    assert_eq!(workspace.engine.stored_document(), &after);
    Ok(())
}

#[test]
fn label_drag_preview_uses_the_atom_anchor_but_click_jitter_uses_raw_press_position() -> TestResult
{
    let mut workspace = Workspace::new(true)?;
    let [_, anchor, end, _] = workspace.points()?;
    let initial = workspace.frame(vec![], None);
    let press = label_point(&initial, anchor)?;
    let before = workspace.engine.stored_document().clone();
    workspace.frame(vec![button(press, true), button(press, false)], None);
    assert!(
        workspace.error.is_none(),
        "a plain label click is not an attachment: {:?}",
        workspace.error
    );
    assert_eq!(workspace.engine.stored_document(), &before);
    let jitter = press + egui::vec2(1.0, 1.0);
    workspace.frame(
        vec![
            button(press, true),
            Event::PointerMoved(jitter),
            button(jitter, false),
        ],
        None,
    );
    assert!(
        workspace.error.is_none(),
        "label jitter is not an occupied-endpoint error: {:?}",
        workspace.error
    );
    assert_eq!(workspace.engine.stored_document(), &before);
    let output = workspace.frame(vec![button(press, true), Event::PointerMoved(end)], None);
    let preview = output
        .shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            egui::Shape::Path(path) if path.fill == egui::Color32::from_rgb(70, 105, 155) => {
                Some(path)
            }
            _ => None,
        })
        .ok_or("preview")?;
    let start = preview.points.first().ok_or("preview start")?;
    let last = preview.points.last().ok_or("preview last")?;
    let snapped = Pos2::new((start.x + last.x) * 0.5, (start.y + last.y) * 0.5);
    assert!(
        snapped.distance(anchor) < 0.01,
        "preview must start at model anchor, not the raw label press"
    );
    assert_eq!(
        workspace.engine.stored_document(),
        &before,
        "preview is transient"
    );
    workspace.frame(vec![button(end, false)], None);
    assert!(workspace.error.is_none(), "{:?}", workspace.error);
    assert_eq!(workspace.engine.document().pages()[0].fragments().len(), 1);
    assert_eq!(
        workspace.engine.document().pages()[0].fragments()[0]
            .atoms()
            .len(),
        3
    );
    Ok(())
}

#[test]
fn a_new_ineligible_primary_press_cancels_the_old_gesture() -> TestResult {
    let mut workspace = Workspace::new(true)?;
    let [_, begin, end, _] = workspace.points()?;
    let before = workspace.engine.stored_document().clone();
    let state = serde_json::to_value(workspace.engine.state())?;
    workspace.frame(vec![button(begin, true)], None);
    workspace.frame(
        vec![
            button(Pos2::new(5.0, 100.0), true),
            Event::PointerMoved(end),
            button(end, false),
        ],
        None,
    );
    assert_eq!(workspace.engine.stored_document(), &before);
    assert_eq!(serde_json::to_value(workspace.engine.state())?, state);
    Ok(())
}

#[test]
fn a_foreground_covered_release_cannot_commit_an_attachment() -> TestResult {
    let mut workspace = Workspace::new(true)?;
    let [_, begin, end, _] = workspace.points()?;
    let cover = egui::Rect::from_min_size(end - egui::vec2(5.0, 5.0), egui::vec2(30.0, 30.0));
    workspace.frame(vec![], Some(cover));
    workspace.frame(vec![], Some(cover));
    let before = workspace.engine.stored_document().clone();
    let state = serde_json::to_value(workspace.engine.state())?;
    workspace.frame(drag(begin, end), Some(cover));
    assert_eq!(workspace.engine.stored_document(), &before);
    assert_eq!(serde_json::to_value(workspace.engine.state())?, state);
    Ok(())
}

#[test]
fn chemistry_guidance_is_visible_before_a_gesture_even_with_an_existing_error() -> TestResult {
    let mut workspace = Workspace::new(true)?;
    for prior in [None, Some("Original test diagnostic".to_owned())] {
        workspace.error = prior;
        let output = workspace.frame(vec![], None);
        assert!(output.shapes.iter().any(|shape|matches!(&shape.shape,egui::Shape::Text(text)
            if text.galley.text().contains("terminal CH3 to CH2") && shape.clip_rect.contains_rect(shape.shape.visual_bounding_rect()))),"the deliberate H adjustment must be visible before commitment");
    }
    assert_eq!(workspace.engine.state().revision, 1);
    Ok(())
}

#[test]
fn a_fresh_button_position_overrides_a_different_cached_hover_target() -> TestResult {
    let mut workspace = Workspace::new(true)?;
    let [other, begin, end, _] = workspace.points()?;
    workspace.frame(vec![Event::PointerMoved(other)], None);
    assert!(
        matches!(workspace.drawing.hover_target(),Some(bondline_render::HitTarget::Atom {atom_id,..}) if atom_id.get()==1)
    );
    workspace.frame(
        vec![
            button(begin, true),
            Event::PointerMoved(end),
            button(end, false),
        ],
        None,
    );
    assert!(workspace.error.is_none(), "{:?}", workspace.error);
    let atoms = workspace.engine.document().pages()[0].fragments()[0].atoms();
    assert_eq!(
        atoms
            .iter()
            .map(|atom| atom.explicit_hydrogens)
            .collect::<Vec<_>>(),
        [Some(3), Some(2), Some(3)]
    );
    Ok(())
}

#[test]
fn a_same_frame_new_bond_can_be_extended_and_a_later_press_can_finish_next_frame() -> TestResult {
    let mut workspace = Workspace::new(false)?;
    let [first, second, third, fourth] = workspace.points()?;
    workspace.frame(
        drag(first, second)
            .into_iter()
            .chain(drag(second, third))
            .chain([button(third, true), Event::PointerMoved(fourth)])
            .collect(),
        None,
    );
    assert!(workspace.error.is_none(), "{:?}", workspace.error);
    assert_eq!(
        (
            workspace.engine.state().revision,
            workspace.engine.state().undo_depth
        ),
        (2, 2)
    );
    assert_eq!(workspace.engine.document().pages()[0].fragments().len(), 1);
    assert_eq!(
        workspace.engine.document().pages()[0].fragments()[0]
            .atoms()
            .len(),
        3
    );
    workspace.frame(vec![button(fourth, false)], None);
    assert!(workspace.error.is_none(), "{:?}", workspace.error);
    assert_eq!(
        (
            workspace.engine.state().revision,
            workspace.engine.state().undo_depth
        ),
        (3, 3)
    );
    assert_eq!(workspace.engine.document().pages()[0].fragments().len(), 1);
    assert_eq!(
        workspace.engine.document().pages()[0].fragments()[0]
            .atoms()
            .len(),
        4
    );
    Ok(())
}

#[test]
fn unsupported_starts_and_existing_release_targets_never_fall_back_to_independent_bonds()
-> TestResult {
    for mode in 0..5 {
        let mut workspace = Workspace::new(true)?;
        if mode == 0 {
            let stored = workspace.engine.stored_document();
            let source = stored.model().pages()[0].fragments()[0].as_ref();
            let mut atoms = source.atoms().to_vec();
            atoms.get_mut(1).ok_or("atom")?.explicit_hydrogens = None;
            let replacement =
                bondline_model::Fragment::new(source.id(), atoms, source.bonds().to_vec())?;
            let mut stored = stored.clone();
            stored
                .model_mut()
                .replace_fragment(bondline_model::PageId::new(1).ok_or("page")?, replacement)?;
            workspace.engine = Engine::from_document(stored);
        }
        let [first, begin, end, _] = workspace.points()?;
        if mode == 4 {
            workspace.frame(drag(begin, end), None);
            assert!(workspace.error.is_none());
        }
        let before = workspace.engine.stored_document().clone();
        let state = serde_json::to_value(workspace.engine.state())?;
        let middle = Pos2::new((first.x + begin.x) * 0.5, (first.y + begin.y) * 0.5);
        let (press, release) = match mode {
            1 => (middle, end),
            2 => (begin, first),
            3 => (begin, middle),
            _ => (begin, end + egui::vec2(30.0, 0.0)),
        };
        workspace.frame(drag(press, release), None);
        assert!(
            workspace.error.is_some(),
            "an intentional unsupported drag needs a diagnostic, mode {mode}"
        );
        assert_eq!(workspace.engine.stored_document(), &before, "mode {mode}");
        assert_eq!(
            serde_json::to_value(workspace.engine.state())?,
            state,
            "mode {mode}"
        );
    }
    Ok(())
}

#[test]
fn source_backed_mol_remains_guarded_after_a_real_attachment_drag() -> TestResult {
    let mut workspace = Workspace::new(false)?;
    workspace.engine.dispatch("document.open_mol",json!({"path":concat!(env!("CARGO_MANIFEST_DIR"),"/../io/tests/fixtures/mol/ethane-v2000.mol")}))?;
    let output = workspace.frame(vec![], None);
    let paper = paper(&output)?;
    let page = workspace.engine.document().pages().first().ok_or("page")?;
    let position = page.fragments()[0].atoms()[1]
        .position
        .ok_or("atom position")?;
    let point = |x: f64, y: f64| {
        paper.min
            + egui::vec2(
                paper.width() * x as f32 / page.size().width_pt() as f32,
                paper.height() * y as f32 / page.size().height_pt() as f32,
            )
    };
    let (begin, end) = (
        point(position.x_pt(), position.y_pt()),
        point(position.x_pt() + 40.0, position.y_pt() + 40.0),
    );
    let before = workspace.engine.stored_document().clone();
    let state = serde_json::to_value(workspace.engine.state())?;
    workspace.frame(drag(begin, end), None);
    assert!(
        workspace
            .error
            .as_deref()
            .is_some_and(|error| error.contains("native CDXML"))
    );
    assert_eq!(workspace.engine.stored_document(), &before);
    assert_eq!(serde_json::to_value(workspace.engine.state())?, state);
    Ok(())
}

#[test]
fn revision_dpi_viewport_and_origin_changes_cancel_a_held_attachment_through_idle_frames()
-> TestResult {
    for mode in 0..5 {
        let mut workspace = Workspace::new(true)?;
        let [_, begin, end, _] = workspace.points()?;
        workspace.frame(vec![button(begin, true)], None);
        match mode {
            0 => {
                workspace.engine.dispatch(
                    "page.set_size",
                    json!({"page_id":1,"width_pt":612,"height_pt":792}),
                )?;
            }
            1 => workspace.dpi = 2.0,
            2 => workspace.host_origin += egui::vec2(0.25, 0.25),
            3 => workspace.screen = workspace.screen.translate(egui::vec2(0.25, 0.25)),
            _ => workspace.screen.max.x += 100.0,
        }
        let before = workspace.engine.stored_document().clone();
        let state = serde_json::to_value(workspace.engine.state())?;
        for _ in 0..3 {
            workspace.frame(vec![], None);
        }
        workspace.frame(vec![Event::PointerMoved(end), button(end, false)], None);
        assert_eq!(
            workspace.engine.stored_document(),
            &before,
            "context mode {mode}"
        );
        assert_eq!(
            serde_json::to_value(workspace.engine.state())?,
            state,
            "context mode {mode}"
        );
    }
    Ok(())
}

fn assert_invalid_viewport_cancels_attachment(events: Vec<Event>) -> TestResult {
    let mut workspace = Workspace::new(true)?;
    let [_, begin, end, _] = workspace.points()?;
    workspace.frame(vec![button(begin, true)], None);
    let before = workspace.engine.stored_document().clone();
    let state = serde_json::to_value(workspace.engine.state())?;
    let original_screen = workspace.screen;
    workspace.screen.max.x = 100_001.0;
    let invalid = workspace.frame(events, None);
    assert!(invalid.shapes.iter().any(|shape| {
        matches!(&shape.shape, egui::Shape::Text(text) if text.galley.text().contains("viewport dimensions must be between 1 and 100000 pixels"))
    }));
    assert_eq!(workspace.engine.stored_document(), &before);
    assert_eq!(serde_json::to_value(workspace.engine.state())?, state);
    workspace.screen = original_screen;
    for _ in 0..3 {
        workspace.frame(vec![], None);
    }
    workspace.frame(vec![Event::PointerMoved(end), button(end, false)], None);
    assert_eq!(workspace.engine.stored_document(), &before);
    assert_eq!(serde_json::to_value(workspace.engine.state())?, state);
    Ok(())
}

#[test]
fn an_invalid_viewport_cancels_attachment_even_after_original_geometry_returns() -> TestResult {
    assert_invalid_viewport_cancels_attachment(vec![])
}

#[test]
fn escape_down_and_up_during_an_invalid_viewport_cannot_leave_an_attachment_armed() -> TestResult {
    assert_invalid_viewport_cancels_attachment(vec![escape(true), escape(false)])
}

#[test]
fn escape_focus_pointer_loss_and_close_cancel_attachment_without_rearming() -> TestResult {
    for mode in 0..4 {
        let mut workspace = Workspace::new(true)?;
        let [_, begin, end, _] = workspace.points()?;
        workspace.frame(vec![button(begin, true)], None);
        let events = match mode {
            0 => vec![
                escape(true),
                Event::PointerMoved(end),
                escape(false),
                button(end, false),
            ],
            1 => vec![
                Event::WindowFocused(false),
                Event::PointerMoved(end),
                Event::WindowFocused(true),
                button(end, false),
            ],
            2 => vec![
                Event::PointerGone,
                Event::PointerMoved(end),
                button(end, false),
            ],
            _ => {
                workspace.engine.dispatch(
                    "document.close",
                    json!({"expected_revision":1,"action":"request"}),
                )?;
                vec![button(end, false)]
            }
        };
        let before = workspace.engine.stored_document().clone();
        let state = serde_json::to_value(workspace.engine.state())?;
        workspace.frame(events, None);
        for _ in 0..3 {
            workspace.frame(vec![], None);
        }
        assert_eq!(
            workspace.engine.stored_document(),
            &before,
            "cancel mode {mode}"
        );
        assert_eq!(
            serde_json::to_value(workspace.engine.state())?,
            state,
            "cancel mode {mode}"
        );
    }
    Ok(())
}

#[test]
fn two_device_pixels_per_logical_point_still_attach_at_document_coordinates() -> TestResult {
    let mut workspace = Workspace::new(true)?;
    workspace.dpi = 2.0;
    let [_, begin, end, _] = workspace.points()?;
    workspace.frame(drag(begin, end), None);
    assert!(workspace.error.is_none(), "{:?}", workspace.error);
    let fragment = &workspace.engine.document().pages()[0].fragments()[0];
    assert_eq!(fragment.atoms().len(), 3);
    assert_eq!(
        fragment.atoms()[1].position,
        Some(Point2::new(140.0, 200.0)?)
    );
    let end = fragment.atoms()[2].position.ok_or("new point")?;
    assert!((end.x_pt() - 180.0).abs() < 0.0001 && (end.y_pt() - 240.0).abs() < 0.0001);
    Ok(())
}

#[test]
fn a_new_foreground_covered_press_cannot_reuse_an_earlier_attachment() -> TestResult {
    let mut workspace = Workspace::new(true)?;
    let [covered, begin, end, _] = workspace.points()?;
    workspace.frame(vec![button(begin, true)], None);
    let cover = egui::Rect::from_min_size(covered - egui::vec2(5.0, 5.0), egui::vec2(30.0, 30.0));
    workspace.frame(vec![], Some(cover));
    workspace.frame(vec![], Some(cover));
    let before = workspace.engine.stored_document().clone();
    let state = serde_json::to_value(workspace.engine.state())?;
    workspace.frame(
        vec![
            button(covered, true),
            Event::PointerMoved(end),
            button(end, false),
        ],
        Some(cover),
    );
    assert_eq!(workspace.engine.stored_document(), &before);
    assert_eq!(serde_json::to_value(workspace.engine.state())?, state);
    Ok(())
}

#[test]
fn a_saved_ui_attachment_survives_undo_redo_and_reopen_with_an_independent_file_consumer()
-> TestResult {
    let directory = file_support::FileTestDirectory::new()?;
    let path = directory.path("ui-attachment.cdxml");
    let mut workspace = Workspace::new(true)?;
    workspace
        .engine
        .dispatch("document.save_cdxml", json!({"path":path}))?;
    file_support::independently_read_file(&path, 1)?;
    let [_, begin, end, _] = workspace.points()?;
    let before = workspace.engine.stored_document().clone();
    workspace.frame(drag(begin, end), None);
    assert!(workspace.error.is_none(), "{:?}", workspace.error);
    let after = workspace.engine.stored_document().clone();
    workspace
        .engine
        .dispatch("document.save_cdxml", json!({"path":path,"overwrite":true}))?;
    let saved = fs::read(&path)?;
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let output=Process::new(python).args(["-B","-c",r#"
import sys
from decimal import Decimal
from xml.parsers import expat
from rdkit import Chem,rdBase
from rdkit.Chem import rdMolDescriptors
assert rdBase.rdkitVersion=='2026.09.2'
raw=sys.stdin.buffer.read(64*1024*1024+1)
assert len(raw)<=64*1024*1024
nodes=[];bonds=[];pages=[];fragments=[]
def start(name,attrs):
    if name=='n': nodes.append(attrs)
    elif name=='b': bonds.append(attrs)
    elif name=='page': pages.append(attrs)
    elif name=='fragment': fragments.append(attrs)
p=expat.ParserCreate();p.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
p.StartElementHandler=start;p.Parse(raw,True)
assert len(nodes)==3 and len(bonds)==2 and len(pages)==len(fragments)==1
assert [n['NumHydrogens'] for n in nodes]==['3','2','3']
assert all(n['Element']=='6' and n['Charge']=='0' and n['Radical']=='None' for n in nodes)
positions=[tuple(Decimal(v) for v in n['p'].split()) for n in nodes]
assert positions[0]==(100,200) and positions[1]==(140,200)
# f32 logical pointer coordinates round before conversion back to document points.
# This predeclared 0.0001pt bound is below one ten-thousandth of a display pixel here.
assert abs(positions[2][0]-180)<Decimal('0.0001') and abs(positions[2][1]-240)<Decimal('0.0001')
ids=[n['id'] for n in nodes]
assert [(b['B'],b['E'],b['Order']) for b in bonds]==[(ids[0],ids[1],'1'),(ids[1],ids[2],'1')]
all_ids=[item['id'] for item in pages+fragments+nodes+bonds]
assert len(all_ids)==len(set(all_ids))
molecules=Chem.MolsFromCDXML(raw.decode('utf-8'));assert len(molecules)==1
m=molecules[0]
assert [a.GetAtomicNum() for a in m.GetAtoms()]==[6,6,6]
assert [a.GetTotalNumHs() for a in m.GetAtoms()]==[3,2,3]
assert [(b.GetBeginAtomIdx(),b.GetEndAtomIdx(),b.GetBondTypeAsDouble()) for b in m.GetBonds()]==[(0,1,1.0),(1,2,1.0)]
assert Chem.MolToSmiles(m)=='CCC' and rdMolDescriptors.CalcMolFormula(m)=='C3H8'
"#]).stdin(fs::File::open(&path)?).output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    workspace.engine.dispatch("history.undo", json!({}))?;
    assert_eq!(workspace.engine.stored_document(), &before);
    assert!(workspace.engine.state().file.dirty);
    workspace.engine.dispatch("history.redo", json!({}))?;
    assert_eq!(workspace.engine.stored_document(), &after);
    assert!(!workspace.engine.state().file.dirty);
    workspace
        .engine
        .dispatch("document.open_cdxml", json!({"path":path}))?;
    assert_eq!(workspace.engine.document(), after.model());
    assert_eq!(
        workspace.engine.stored_document().to_cdxml()?.as_ref(),
        saved
    );
    let output = workspace.frame(vec![], None);
    assert!(workspace.error.is_none(), "{:?}", workspace.error);
    assert!(paper(&output)?.is_positive());
    Ok(())
}
