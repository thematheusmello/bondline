//! Disposable selection gestures; persistent state and authoring stay in the registry.
use super::HoverContext;
use bondline_commands::{
    CaptureStamp, Command, EffectiveTool, Engine, Error, SelectionContext, SelectionTranslate,
    UiAtomLabel, UiSelection,
};
use bondline_model::{AtomId, Page, Point2};
use bondline_render::{GraphScene, HitTarget, PageView};
use eframe::egui;
use std::collections::{BTreeMap, BTreeSet};

pub(super) struct Surface<'a> {
    pub page: &'a Page,
    pub graph: &'a GraphScene,
    pub viewport: &'a PageView,
    pub response: &'a egui::Response,
    pub origin: egui::Pos2,
    pub density: f32,
    pub hover: HoverContext,
}
impl Surface<'_> {
    pub(super) fn local(&self, pos: egui::Pos2) -> [f32; 2] {
        let pos = (pos - self.origin) * self.density;
        [pos.x, pos.y]
    }
    pub(super) fn accepts(&self, context: &egui::Context, pos: egui::Pos2) -> bool {
        self.response.interact_rect.contains(pos)
            && self.viewport.contains(self.local(pos))
            && context.layer_id_at(pos) == Some(self.response.layer_id)
    }
    pub(super) fn point(&self, pos: egui::Pos2) -> Result<Point2, Error> {
        Ok(self.viewport.document_position(self.local(pos))?)
    }
}
struct Gesture {
    proof: SelectionContext,
    hover: HoverContext,
    target: Option<HitTarget>,
    translate: bool,
    shift: bool,
    start: Point2,
    end: Point2,
    screen_start: egui::Pos2,
    screen_end: egui::Pos2,
    drag_distance: f32,
    dragged: bool,
}
impl Gesture {
    fn dragging(&self) -> bool {
        self.dragged
    }
    fn update_end(&mut self, screen: egui::Pos2, point: Point2) {
        self.dragged |= self.screen_start.distance(screen) > self.drag_distance;
        self.screen_end = screen;
        self.end = point;
    }
    fn params(&self, engine: &Engine) -> SelectionTranslate {
        SelectionTranslate {
            expected_revision: engine.state().revision,
            expected_selection_epoch: engine.selection().epoch,
            delta_x_pt: self.end.x_pt() - self.start.x_pt(),
            delta_y_pt: self.end.y_pt() - self.start.y_pt(),
        }
    }
}
struct Click {
    source: CaptureStamp,
    hover: HoverContext,
    target: HitTarget,
    position: egui::Pos2,
    time: f64,
}
#[derive(Default)]
pub(super) struct SelectionWorkflow {
    gesture: Option<Gesture>,
    click: Option<Click>,
}
impl SelectionWorkflow {
    pub fn cancel(&mut self) -> bool {
        self.gesture.take().is_some()
    }
    pub fn reset(&mut self) {
        self.gesture = None;
        self.click = None;
    }
    pub fn is_dragging(&self) -> bool {
        self.gesture.is_some()
    }
    pub fn sync(&mut self, engine: &Engine, hover: HoverContext) {
        if self
            .gesture
            .as_ref()
            .is_some_and(|gesture| gesture.hover != hover || !gesture.proof.is_current(engine))
        {
            self.gesture = None;
        }
        if self.click.as_ref().is_some_and(|click| {
            click.hover != hover || !engine.capture_stamp_matches(&click.source)
        }) {
            self.click = None;
        }
    }
    pub fn escape(&mut self, engine: &Engine) -> Option<Command> {
        if self.cancel() {
            return None;
        }
        self.click = None;
        Some(Command::UiSelection(UiSelection::Clear {
            expected_revision: engine.state().revision,
        }))
    }
    pub fn event(
        &mut self,
        engine: &Engine,
        context: &egui::Context,
        event: &egui::Event,
        surface: &Surface<'_>,
    ) -> Result<Option<Command>, Error> {
        self.sync(engine, surface.hover);
        match event {
            egui::Event::Key {
                key,
                pressed: true,
                repeat: false,
                modifiers,
                ..
            } if matches!(
                key,
                egui::Key::ArrowLeft
                    | egui::Key::ArrowRight
                    | egui::Key::ArrowUp
                    | egui::Key::ArrowDown
            ) && (*modifiers == egui::Modifiers::NONE
                || *modifiers == egui::Modifiers::SHIFT) =>
            {
                self.cancel();
                if engine.selection().atom_ids.is_empty()
                    || !engine.selection_translation_available()
                {
                    return Ok(None);
                }
                let step = if modifiers.shift { 10.0 } else { 1.0 };
                let (x, y) = match key {
                    egui::Key::ArrowLeft => (-step, 0.0),
                    egui::Key::ArrowRight => (step, 0.0),
                    egui::Key::ArrowUp => (0.0, -step),
                    _ => (0.0, step),
                };
                Ok(Some(Command::SelectionTranslate(SelectionTranslate {
                    expected_revision: engine.state().revision,
                    expected_selection_epoch: engine.selection().epoch,
                    delta_x_pt: x,
                    delta_y_pt: y,
                })))
            }
            egui::Event::PointerMoved(pos) => {
                if !surface.accepts(context, *pos) {
                    self.reset();
                    return Ok(None);
                }
                if let Some(gesture) = &mut self.gesture {
                    gesture.update_end(*pos, surface.point(*pos)?);
                }
                Ok(None)
            }
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers,
            } => {
                self.cancel();
                if !surface.accepts(context, *pos) {
                    return Ok(None);
                }
                let target = surface
                    .graph
                    .hit_test(surface.local(*pos), 5.0 * surface.density)?;
                if modifiers.alt && !modifiers.ctrl && !modifiers.command && !modifiers.shift {
                    return Ok(match target {
                        Some(HitTarget::Atom {
                            fragment_id,
                            atom_id,
                        }) => Some(Command::UiAtomLabel(UiAtomLabel::Open {
                            expected_revision: engine.state().revision,
                            page_id: surface.page.id().get(),
                            fragment_id: fragment_id.get(),
                            atom_id: atom_id.get(),
                        })),
                        _ => None,
                    });
                }
                if *modifiers != egui::Modifiers::NONE && *modifiers != egui::Modifiers::SHIFT {
                    return Ok(None);
                }
                let translate = if let Some(target) = target {
                    !modifiers.shift
                        && engine.selection_translation_available()
                        && target_atoms(surface.page, target, false)?
                            .iter()
                            .all(|id| engine.selection().atom_ids.binary_search(id).is_ok())
                } else {
                    false
                };
                let point = surface.point(*pos)?;
                self.gesture = Some(Gesture {
                    proof: engine.selection_context()?,
                    hover: surface.hover,
                    target,
                    translate,
                    shift: modifiers.shift,
                    start: point,
                    end: point,
                    screen_start: *pos,
                    screen_end: *pos,
                    drag_distance: context.options(|options| options.input_options.max_click_dist),
                    dragged: false,
                });
                Ok(None)
            }
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                ..
            } => {
                let Some(mut gesture) = self.gesture.take() else {
                    return Ok(None);
                };
                if !gesture.proof.is_current(engine)
                    || gesture.hover != surface.hover
                    || !surface.accepts(context, *pos)
                {
                    return Ok(None);
                }
                gesture.update_end(*pos, surface.point(*pos)?);
                if gesture.dragging() && gesture.translate {
                    self.click = None;
                    return Ok(Some(Command::SelectionTranslate(gesture.params(engine))));
                }
                let ids = if let Some(target) = gesture.target {
                    if gesture.dragging() {
                        self.click = None;
                        return Ok(None);
                    }
                    let now = context.input(|input| input.time);
                    let double = !gesture.shift
                        && self.click.as_ref().is_some_and(|click| {
                            click.target == target
                                && click.hover == surface.hover
                                && engine.capture_stamp_matches(&click.source)
                                && now - click.time
                                    < context.options(|options| {
                                        options.input_options.max_double_click_delay
                                    })
                                && click.position.distance(*pos) <= gesture.drag_distance
                        });
                    self.click = if double {
                        None
                    } else {
                        Some(Click {
                            source: engine.capture_stamp(),
                            hover: surface.hover,
                            target,
                            position: *pos,
                            time: now,
                        })
                    };
                    let targets = target_atoms(surface.page, target, double)?;
                    if gesture.shift {
                        let mut ids: BTreeSet<_> =
                            engine.selection().atom_ids.iter().copied().collect();
                        for id in targets {
                            if !ids.remove(&id) {
                                ids.insert(id);
                            }
                        }
                        ids.into_iter().collect()
                    } else {
                        targets
                    }
                } else {
                    self.click = None;
                    let a = gesture.start;
                    let b = gesture.end;
                    let mut ids: BTreeSet<_> = if gesture.shift {
                        engine.selection().atom_ids.iter().copied().collect()
                    } else {
                        BTreeSet::new()
                    };
                    if gesture.dragging() {
                        for atom in surface
                            .page
                            .fragments()
                            .iter()
                            .flat_map(|fragment| fragment.atoms())
                        {
                            if atom.position.is_some_and(|p| {
                                p.x_pt() >= a.x_pt().min(b.x_pt())
                                    && p.x_pt() <= a.x_pt().max(b.x_pt())
                                    && p.y_pt() >= a.y_pt().min(b.y_pt())
                                    && p.y_pt() <= a.y_pt().max(b.y_pt())
                            }) {
                                ids.insert(atom.id.get());
                            }
                        }
                    }
                    ids.into_iter().collect()
                };
                Ok(Some(Command::UiSelection(UiSelection::Set {
                    expected_revision: engine.state().revision,
                    page_id: surface.page.id().get(),
                    atom_ids: ids,
                })))
            }
            _ => Ok(None),
        }
    }
    pub fn preview_translation(&self, engine: &Engine) -> Option<SelectionTranslate> {
        let gesture = self.gesture.as_ref()?;
        (gesture.translate && gesture.dragging() && gesture.proof.is_current(engine))
            .then(|| gesture.params(engine))
    }
    pub fn paint(
        &self,
        painter: &egui::Painter,
        origin: egui::Pos2,
        density: f32,
        page: &Page,
        viewport: &PageView,
        engine: &Engine,
    ) {
        let color = egui::Color32::from_rgb(40, 120, 180);
        if engine.effective_tool() == EffectiveTool::Selection
            && engine.selection().page_id == Some(page.id().get())
        {
            let mut bounds = egui::Rect::NOTHING;
            for atom in page
                .fragments()
                .iter()
                .flat_map(|fragment| fragment.atoms())
            {
                if engine
                    .selection()
                    .atom_ids
                    .binary_search(&atom.id.get())
                    .is_ok()
                    && let Some(point) = atom.position
                    && let Ok([x, y]) = viewport.canvas_position(point)
                {
                    let pos = origin + egui::vec2(x, y) / density;
                    bounds.extend_with(pos);
                    painter.circle_stroke(pos, 4.0, egui::Stroke::new(1.0, color));
                }
            }
            if bounds.is_finite() {
                painter.rect_stroke(bounds.expand(6.0), 0.0, egui::Stroke::new(1.0, color));
            }
        }
        if let Some(gesture) = &self.gesture
            && gesture.target.is_none()
            && gesture.dragging()
            && gesture.proof.is_current(engine)
        {
            painter.rect_stroke(
                egui::Rect::from_two_pos(gesture.screen_start, gesture.screen_end),
                0.0,
                egui::Stroke::new(1.0, color),
            );
        }
    }
}

fn target_atoms(page: &Page, target: HitTarget, connected: bool) -> Result<Vec<u32>, Error> {
    let fragment_id = match target {
        HitTarget::Atom { fragment_id, .. } | HitTarget::Bond { fragment_id, .. } => fragment_id,
    };
    let fragment = page
        .fragments()
        .iter()
        .find(|fragment| fragment.id() == fragment_id)
        .ok_or(bondline_model::Error::MissingFragment(fragment_id.get()))?;
    let seeds = match target {
        HitTarget::Atom { atom_id, .. } => vec![atom_id],
        HitTarget::Bond { bond_id, .. } => {
            let bond = fragment
                .bonds()
                .iter()
                .find(|bond| bond.id == bond_id)
                .ok_or(Error::MissingBond {
                    fragment_id: fragment_id.get(),
                    bond_id: bond_id.get(),
                })?;
            vec![bond.begin, bond.end]
        }
    };
    let mut selected: BTreeSet<AtomId> = seeds.iter().copied().collect();
    if connected {
        let mut adjacency: BTreeMap<AtomId, Vec<AtomId>> = BTreeMap::new();
        for bond in fragment.bonds() {
            adjacency.entry(bond.begin).or_default().push(bond.end);
            adjacency.entry(bond.end).or_default().push(bond.begin);
        }
        let mut pending = seeds;
        while let Some(id) = pending.pop() {
            if let Some(neighbors) = adjacency.get(&id) {
                for neighbor in neighbors {
                    if selected.insert(*neighbor) {
                        pending.push(*neighbor);
                    }
                }
            }
        }
    }
    Ok(selected.into_iter().map(AtomId::get).collect())
}
