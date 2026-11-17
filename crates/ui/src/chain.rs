//! One disposable chain gesture or numeric draft; the registry owns every edit.
use super::{
    HoverContext,
    chain_math::{Acyclic, Snaking},
    readable_graph,
    selection::Surface,
};
use bondline_commands::{
    ChainContext, ChainCreate, ChainCreatePath, ChainKind, ChainPathPlacement, ChainPathPoint,
    ChainPreferences, ChainTarget, Command, Engine, Error, OrderHydrogenPolicy, UiChainTool,
};
use bondline_model::{Document, Point2};
use bondline_render::{GraphScene, HitTarget, PageView};
use eframe::egui;
use std::sync::Arc;

mod form;

enum Walk {
    Acyclic(Acyclic),
    Snaking(Snaking),
}
struct Gesture {
    proof: ChainContext,
    hover: HoverContext,
    layer: egui::LayerId,
    target: ChainTarget,
    kind: ChainKind,
    preferences: ChainPreferences,
    walk: Walk,
    points: Vec<Point2>,
    invalid: bool,
    press: egui::Pos2,
    drag_distance: f32,
    dragged: bool,
    previous_pointer: Point2,
    modifiers: egui::Modifiers,
    temporary: bool,
}

impl Gesture {
    fn modifiers(&mut self, value: egui::Modifiers) -> Result<(), Error> {
        if unsupported(value) {
            return Err(unavailable(
                "Shift and Command chain gestures are unavailable",
            ));
        }
        if self.temporary && !value.ctrl {
            return Err(unavailable("Release the mouse before releasing Control"));
        }
        let chord = value.ctrl && value.alt;
        let rising = chord && !(self.modifiers.ctrl && self.modifiers.alt);
        if rising {
            if self.kind != ChainKind::Acyclic
                || self.temporary
                || self.preferences.fixed_lengths
                || !self.preferences.fixed_angles
            {
                return Err(unavailable(
                    "Control+Alt reversal requires Acyclic with Fixed Lengths off and Fixed Angles on",
                ));
            }
            if let Walk::Acyclic(walk) = &mut self.walk {
                walk.reverse();
            }
            if self.dragged {
                if self.invalid {
                    return Err(unavailable("Cancel this invalid chain before reversing it"));
                }
                if let Walk::Acyclic(walk) = &mut self.walk {
                    self.points = walk.sample(self.previous_pointer)?;
                }
            }
        } else if value.alt && !chord && self.dragged {
            return Err(unavailable("Alt repeat starts with a new click"));
        }
        self.modifiers = value;
        Ok(())
    }

    fn sample(&mut self, screen: egui::Pos2, point: Point2) -> Result<(), Error> {
        self.dragged |= self.press.distance(screen) > self.drag_distance;
        if !self.dragged {
            self.previous_pointer = point;
            return Ok(());
        }
        if self.modifiers.alt && !self.modifiers.ctrl {
            return Err(unavailable("Alt repeat requires a click, not a drag"));
        }
        if self.kind == ChainKind::Acyclic
            && self.modifiers.ctrl
            && !self.modifiers.alt
            && !self.temporary
        {
            if self.invalid {
                return Err(unavailable(
                    "Cancel this invalid chain before entering Snaking",
                ));
            }
            if let Walk::Acyclic(walk) = &self.walk {
                self.walk = Walk::Snaking(Snaking::from_prefix(
                    walk.points().to_vec(),
                    self.preferences,
                    self.previous_pointer,
                )?);
                self.temporary = true;
            }
        }
        let result = match &mut self.walk {
            Walk::Acyclic(walk) => walk.sample(point),
            Walk::Snaking(walk) => walk.sample(point),
        };
        match result {
            Ok(points) => {
                self.points = points;
                self.invalid = false;
                self.previous_pointer = point;
                Ok(())
            }
            Err(error) => {
                self.invalid = true;
                Err(error)
            }
        }
    }

    fn command(&self, engine: &Engine) -> Result<Command, Error> {
        if self.invalid || self.points.len() < 2 {
            return Err(unavailable("a chain needs at least two valid vertices"));
        }
        let points: Vec<_> = self
            .points
            .iter()
            .map(|p| ChainPathPoint {
                x_pt: p.x_pt(),
                y_pt: p.y_pt(),
            })
            .collect();
        let placement = match self.target {
            ChainTarget::Free { .. } => ChainPathPlacement::Free { points },
            ChainTarget::Atom {
                fragment_id,
                atom_id,
            } => ChainPathPlacement::Atom {
                fragment_id,
                atom_id,
                new_points: points.into_iter().skip(1).collect(),
            },
        };
        Ok(Command::ChainCreatePath(ChainCreatePath {
            expected_revision: engine.state().revision,
            page_id: self.hover.page.get(),
            hydrogen_policy: OrderHydrogenPolicy::RectifyNeutralCarbon,
            placement,
        }))
    }
}

struct Canvas {
    view: PageView,
    hover: HoverContext,
    layer: egui::LayerId,
}
struct Repeat {
    proof: ChainContext,
    hover: HoverContext,
    layer: egui::LayerId,
    count: u8,
}
struct PendingRepeat {
    hover: HoverContext,
    layer: egui::LayerId,
    count: u8,
}
struct Candidate {
    document: Arc<Document>,
    graph: GraphScene,
}

#[derive(Default)]
pub(super) struct ChainWorkflow {
    gesture: Option<Gesture>,
    editor: form::AddEditor,
    repeat: Option<Repeat>,
    pending_repeat: Option<PendingRepeat>,
    canvas: Option<Canvas>,
}

impl ChainWorkflow {
    pub fn present(&self) -> bool {
        self.editor.present()
    }
    pub fn sync_form(&mut self, engine: &Engine, error: &mut Option<String>) {
        self.editor.sync(engine, error);
    }
    pub fn begin_frame(&mut self) {
        self.canvas = None;
    }
    pub fn lose_surface(&mut self) {
        self.cancel_gesture();
        self.repeat = None;
        self.canvas = None;
        self.editor.lose_canvas();
    }
    pub fn finish_frame(&mut self) {
        if self.canvas.is_none() {
            self.lose_surface();
        }
    }
    pub fn cancel_gesture(&mut self) {
        self.gesture = None;
        self.pending_repeat = None;
    }
    pub fn is_drawing(&self) -> bool {
        self.gesture.is_some()
    }
    pub fn count(&self) -> Option<usize> {
        self.gesture
            .as_ref()
            .filter(|g| g.dragged && !g.invalid)
            .map(|g| g.points.len())
    }
    pub fn set_canvas(&mut self, view: PageView, hover: HoverContext, layer: egui::LayerId) {
        self.canvas = Some(Canvas { view, hover, layer });
    }
    pub fn sync(&mut self, engine: &Engine, hover: HoverContext, layer: egui::LayerId) {
        self.editor.sync_canvas(hover, layer);
        if self
            .gesture
            .as_ref()
            .is_some_and(|g| g.hover != hover || g.layer != layer || !g.proof.is_current(engine))
        {
            self.cancel_gesture();
        }
        if self
            .repeat
            .as_ref()
            .is_some_and(|r| r.hover != hover || r.layer != layer || !r.proof.is_current(engine))
        {
            self.repeat = None;
        }
    }
    pub fn observe(&mut self, event: &egui::Event) -> Result<(), Error> {
        if let egui::Event::ModifiersChanged(modifiers) = event
            && let Some(gesture) = &mut self.gesture
            && let Err(error) = gesture.modifiers(*modifiers)
        {
            self.cancel_gesture();
            return Err(error);
        }
        Ok(())
    }
    pub fn event(
        &mut self,
        engine: &Engine,
        context: &egui::Context,
        event: &egui::Event,
        surface: &Surface<'_>,
    ) -> Result<Option<Command>, Error> {
        self.sync(engine, surface.hover, surface.response.layer_id);
        match event {
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers,
            } => {
                self.cancel_gesture();
                if !surface.accepts(context, *pos) {
                    return Ok(None);
                }
                let bondline_commands::EffectiveTool::Chain { kind } = engine.effective_tool()
                else {
                    return Ok(None);
                };
                if unsupported(*modifiers) {
                    return Err(unavailable(
                        "Shift and Command chain gestures are unavailable",
                    ));
                }
                let (target, begin) = match surface
                    .graph
                    .hit_test(surface.local(*pos), 5.0 * surface.density)?
                {
                    None => {
                        let p = surface.point(*pos)?;
                        (
                            ChainTarget::Free {
                                x_pt: p.x_pt(),
                                y_pt: p.y_pt(),
                            },
                            p,
                        )
                    }
                    Some(HitTarget::Atom {
                        fragment_id,
                        atom_id,
                    }) => {
                        let p = surface
                            .page
                            .fragments()
                            .iter()
                            .find(|f| f.id() == fragment_id)
                            .and_then(|f| f.atoms().iter().find(|a| a.id == atom_id))
                            .and_then(|a| a.position)
                            .ok_or(unavailable("a chain anchor needs an authored position"))?;
                        (
                            ChainTarget::Atom {
                                fragment_id: fragment_id.get(),
                                atom_id: atom_id.get(),
                            },
                            p,
                        )
                    }
                    Some(HitTarget::Bond { .. }) => {
                        return Err(unavailable(
                            "start a chain on paper or an eligible carbon atom",
                        ));
                    }
                };
                let preferences = engine.chain_preferences()?;
                let walk = if kind == ChainKind::Snaking {
                    Walk::Snaking(Snaking::new(begin, preferences))
                } else {
                    Walk::Acyclic(Acyclic::new(begin, preferences))
                };
                let mut gesture = Gesture {
                    proof: engine.chain_context(surface.page.id().get())?,
                    hover: surface.hover,
                    layer: surface.response.layer_id,
                    target,
                    kind,
                    preferences,
                    walk,
                    points: vec![begin],
                    invalid: false,
                    press: *pos,
                    drag_distance: context.options(|o| o.input_options.max_click_dist),
                    dragged: false,
                    previous_pointer: begin,
                    modifiers: egui::Modifiers::NONE,
                    temporary: false,
                };
                gesture.modifiers(*modifiers)?;
                self.gesture = Some(gesture);
                Ok(None)
            }
            egui::Event::PointerMoved(pos) => {
                if !surface.accepts(context, *pos) {
                    self.cancel_gesture();
                    return Ok(None);
                }
                let point = surface.point(*pos)?;
                if let Some(gesture) = &mut self.gesture
                    && let Err(error) = gesture.sample(*pos, point)
                {
                    if matches!(error, Error::UnsupportedChain(_)) {
                        self.cancel_gesture();
                    }
                    return Err(error);
                }
                Ok(None)
            }
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers,
            } => {
                let Some(mut gesture) = self.gesture.take() else {
                    return Ok(None);
                };
                if !gesture.proof.is_current(engine) || !surface.accepts(context, *pos) {
                    return Ok(None);
                }
                gesture.modifiers(*modifiers)?;
                gesture.sample(*pos, surface.point(*pos)?)?;
                if gesture.dragged {
                    if gesture.points.len() < 2 {
                        return Ok(None);
                    }
                    let command = gesture.command(engine)?;
                    candidate(engine, &command, surface.viewport, surface.density)?;
                    return Ok(Some(command));
                }
                if modifiers.alt
                    && !modifiers.ctrl
                    && let Some(repeat) = &self.repeat
                    && repeat.proof.is_current(engine)
                    && repeat.hover == surface.hover
                    && repeat.layer == surface.response.layer_id
                {
                    let params = engine.chain_repeat_candidates(
                        &repeat.proof,
                        surface.page.id().get(),
                        &gesture.target,
                        repeat.count,
                    )?;
                    let (index, _) =
                        numeric_candidate(engine, &params, surface.viewport, surface.density)?;
                    let params = params
                        .get(index)
                        .cloned()
                        .ok_or(Error::InvalidChain("numeric pose index is invalid"))?;
                    self.pending_repeat = Some(PendingRepeat {
                        count: repeat.count,
                        hover: surface.hover,
                        layer: surface.response.layer_id,
                    });
                    return Ok(Some(Command::ChainCreate(params)));
                }
                Ok(Some(Command::UiChainTool(UiChainTool::OpenAdd {
                    expected_revision: engine.state().revision,
                    page_id: surface.page.id().get(),
                    target: gesture.target,
                })))
            }
            _ => Ok(None),
        }
    }

    /// Called immediately after execution while the same Engine guard is held.
    pub fn executed(&mut self, engine: &Engine, changed: bool) {
        if let Some(pending) = self.pending_repeat.take()
            && changed
        {
            self.repeat = make_repeat(engine, pending.count, pending.hover, pending.layer);
        }
    }
    pub fn preview(
        &self,
        engine: &Engine,
        view: &PageView,
        density: f32,
    ) -> Option<Result<(Arc<Document>, GraphScene), Error>> {
        if self.editor.present() {
            return self.editor.preview(engine, view, density);
        }
        let g = self.gesture.as_ref()?;
        (g.dragged && !g.invalid && g.points.len() >= 2 && g.proof.is_current(engine)).then(|| {
            candidate(engine, &g.command(engine)?, view, density).map(|c| (c.document, c.graph))
        })
    }
    pub fn show_form(
        &mut self,
        context: &egui::Context,
        engine: Option<&mut Engine>,
        events: &[egui::Event],
        editable: bool,
    ) {
        if let Some(repeat) =
            self.editor
                .show(context, engine, events, editable, self.canvas.as_ref())
        {
            self.repeat = Some(repeat);
        }
    }
}

fn make_repeat(
    engine: &Engine,
    count: u8,
    mut hover: HoverContext,
    layer: egui::LayerId,
) -> Option<Repeat> {
    let proof = engine.chain_context(hover.page.get()).ok()?;
    hover.revision = engine.state().revision;
    hover.tool = engine.bond_tool();
    hover.view_epoch = engine.view().epoch;
    Some(Repeat {
        proof,
        count,
        hover,
        layer,
    })
}

fn candidate(
    engine: &Engine,
    command: &Command,
    view: &PageView,
    density: f32,
) -> Result<Candidate, Error> {
    let (stored, page_id) = match command {
        Command::ChainCreate(params) => (engine.preview_chain_create(params)?, params.page_id),
        Command::ChainCreatePath(params) => (engine.preview_chain_path(params)?, params.page_id),
        _ => {
            return Err(Error::InvalidChain(
                "chain preview needs an explicit chain command",
            ));
        }
    };
    let page = stored
        .model()
        .pages()
        .iter()
        .find(|p| p.id().get() == page_id)
        .ok_or(bondline_model::Error::MissingPage(page_id))?;
    let graph = readable_graph(
        view.graph_scene(page, &bondline_render::Cancellation::default())?,
        density,
    )?;
    Ok(Candidate {
        document: stored.model_snapshot(),
        graph,
    })
}

fn numeric_candidate(
    engine: &Engine,
    params: &[ChainCreate],
    view: &PageView,
    density: f32,
) -> Result<(usize, Candidate), Error> {
    let mut first = None;
    for (index, params) in params.iter().enumerate() {
        match candidate(engine, &Command::ChainCreate(params.clone()), view, density) {
            Ok(candidate) => return Ok((index, candidate)),
            Err(error) => {
                if first.is_none() {
                    first = Some(error);
                }
            }
        }
    }
    Err(first.unwrap_or(Error::InvalidChain("no numeric chain poses are available")))
}

fn unsupported(m: egui::Modifiers) -> bool {
    m.shift || m.mac_cmd || (m.command && !m.ctrl)
}
fn unavailable(message: &'static str) -> Error {
    Error::UnsupportedChain(message)
}
