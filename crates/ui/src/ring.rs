//! Disposable ring gestures; both preview and release validate the real registry candidate.
use super::{HoverContext, readable_graph, selection::Surface};
use bondline_commands::{
    ChairKind, ChairRingCreate, Command, EffectiveTool, Engine, Error, NamedRingCreate,
    OrderHydrogenPolicy, RingContext, RingCreate, RingCreateRequest, RingPhase, RingPlacement,
    RingSide,
};
use bondline_model::{AtomId, BondId, Document, Fragment, FragmentId, Page, Point2};
use bondline_render::{GraphScene, HitTarget, PageView};
use eframe::egui;
use std::sync::Arc;

struct Candidate {
    params: RingCreateRequest,
    document: Arc<Document>,
    graph: GraphScene,
}

struct Gesture {
    proof: RingContext,
    hover: HoverContext,
    layer: egui::LayerId,
    params: RingCreateRequest,
    begin: Point2,
    bond_end: Option<Point2>,
    press: egui::Pos2,
    drag_distance: f32,
    dragged: bool,
    choose_side: bool,
}

impl Gesture {
    fn update(&mut self, screen: egui::Pos2, point: Point2) {
        self.dragged |= self.press.distance(screen) > self.drag_distance;
        if !self.dragged {
            return;
        }
        match placement_mut(&mut self.params) {
            RingPlacement::Free {
                end_x_pt, end_y_pt, ..
            }
            | RingPlacement::Atom {
                end_x_pt, end_y_pt, ..
            } => {
                *end_x_pt = point.x_pt();
                *end_y_pt = point.y_pt();
            }
            RingPlacement::Bond { side, .. } => {
                if let Some(end) = self.bond_end {
                    let cross = cross(self.begin, end, point);
                    if cross != 0.0 {
                        *side = if cross > 0.0 {
                            RingSide::Positive
                        } else {
                            RingSide::Negative
                        };
                        self.choose_side = false;
                    }
                }
            }
        }
    }

    fn candidate(
        &mut self,
        engine: &Engine,
        view: &PageView,
        density: f32,
    ) -> Result<Candidate, Error> {
        let first = candidate(engine, &self.params, view, density);
        if !self.choose_side {
            return first;
        }
        self.choose_side = false;
        match first {
            Ok(candidate) => Ok(candidate),
            Err(first_error) => {
                if let RingPlacement::Bond { side, .. } = placement_mut(&mut self.params) {
                    *side = match side {
                        RingSide::Positive => RingSide::Negative,
                        RingSide::Negative => RingSide::Positive,
                    };
                }
                candidate(engine, &self.params, view, density).map_err(|_| first_error)
            }
        }
    }
}

#[derive(Default)]
pub(super) struct RingWorkflow {
    gesture: Option<Gesture>,
}

impl RingWorkflow {
    pub fn cancel(&mut self) {
        self.gesture = None;
    }

    pub fn is_drawing(&self) -> bool {
        self.gesture.is_some()
    }

    pub fn sync(&mut self, engine: &Engine, hover: HoverContext, layer: egui::LayerId) {
        if self.gesture.as_ref().is_some_and(|gesture| {
            gesture.hover != hover || gesture.layer != layer || !gesture.proof.is_current(engine)
        }) {
            self.cancel();
        }
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
                self.cancel();
                if !surface.accepts(context, *pos) {
                    return Ok(None);
                }
                let (size, kind, chair) = match engine.effective_tool() {
                    EffectiveTool::Ring { size } => (size, None, None),
                    EffectiveTool::NamedRing { kind } => (kind.size(), Some(kind), None),
                    EffectiveTool::ChairRing { kind } => (6, None, Some(kind)),
                    _ => return Ok(None),
                };
                if !supported_modifiers(*modifiers, kind.is_some()) {
                    return Err(Error::UnsupportedRing(
                        "modified ring gestures are unavailable",
                    ));
                }
                let proof = engine.ring_context(surface.page.id().get())?;
                let point = surface.point(*pos)?;
                let hit = surface
                    .graph
                    .hit_test(surface.local(*pos), 5.0 * surface.density)?;
                let (placement, begin, bond_end) = match hit {
                    None => {
                        let end = match chair {
                            Some(kind) => chair_click_end(engine, kind, point)?,
                            None => click_end(point, size),
                        };
                        (
                            RingPlacement::Free {
                                begin_x_pt: point.x_pt(),
                                begin_y_pt: point.y_pt(),
                                end_x_pt: end.0,
                                end_y_pt: end.1,
                                side: RingSide::Positive,
                            },
                            point,
                            None,
                        )
                    }
                    Some(HitTarget::Atom {
                        fragment_id,
                        atom_id,
                    }) => {
                        let fragment = fragment(surface.page, fragment_id)?;
                        let begin = position(fragment, atom_id)?;
                        let end = match chair {
                            Some(kind) => {
                                chair_atom_click_end(engine, kind, fragment, atom_id, begin)?
                            }
                            None => atom_click_end(fragment, atom_id, begin, size)?,
                        };
                        (
                            RingPlacement::Atom {
                                fragment_id: fragment_id.get(),
                                atom_id: atom_id.get(),
                                end_x_pt: end.0,
                                end_y_pt: end.1,
                                side: RingSide::Positive,
                            },
                            begin,
                            None,
                        )
                    }
                    Some(HitTarget::Bond {
                        fragment_id,
                        bond_id,
                    }) => {
                        let (begin, end, side) = bond_anchor(surface.page, fragment_id, bond_id)?;
                        (
                            RingPlacement::Bond {
                                fragment_id: fragment_id.get(),
                                bond_id: bond_id.get(),
                                side,
                            },
                            begin,
                            Some(end),
                        )
                    }
                };
                let choose_side = matches!(placement, RingPlacement::Bond { .. });
                let expected_revision = engine.state().revision;
                let page_id = surface.page.id().get();
                let hydrogen_policy = OrderHydrogenPolicy::RectifyNeutralCarbon;
                let params = if let Some(kind) = chair {
                    ChairRingCreate {
                        expected_revision,
                        page_id,
                        kind,
                        hydrogen_policy,
                        placement,
                    }
                    .into()
                } else {
                    match kind {
                        None => RingCreate {
                            expected_revision,
                            page_id,
                            size,
                            hydrogen_policy,
                            placement,
                        }
                        .into(),
                        Some(kind) => NamedRingCreate {
                            expected_revision,
                            page_id,
                            kind,
                            hydrogen_policy,
                            placement,
                            phase: modifiers.shift.then_some(RingPhase::SingleFirst),
                        }
                        .into(),
                    }
                };
                self.gesture = Some(Gesture {
                    proof,
                    hover: surface.hover,
                    layer: surface.response.layer_id,
                    choose_side,
                    params,
                    begin,
                    bond_end,
                    press: *pos,
                    drag_distance: context.options(|options| options.input_options.max_click_dist),
                    dragged: false,
                });
                Ok(None)
            }
            egui::Event::PointerMoved(pos) => {
                if !surface.accepts(context, *pos) {
                    self.cancel();
                    return Ok(None);
                }
                let point = match surface.point(*pos) {
                    Ok(point) => point,
                    Err(error) => {
                        self.cancel();
                        return Err(error);
                    }
                };
                if let Some(gesture) = &mut self.gesture {
                    gesture.update(*pos, point);
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
                if !surface.accepts(context, *pos) {
                    return Ok(None);
                }
                if !supported_modifiers(
                    *modifiers,
                    matches!(&gesture.params, RingCreateRequest::Named(_)),
                ) {
                    return Err(Error::UnsupportedRing(
                        "modified ring gestures are unavailable",
                    ));
                }
                gesture.update(*pos, surface.point(*pos)?);
                let candidate = gesture.candidate(engine, surface.viewport, surface.density)?;
                if !gesture.proof.is_current(engine) {
                    return Ok(None);
                }
                Ok(Some(Command::RingCreate(candidate.params)))
            }
            _ => Ok(None),
        }
    }

    pub fn preview(
        &mut self,
        engine: &Engine,
        view: &PageView,
        density: f32,
    ) -> Option<Result<(Arc<Document>, GraphScene), Error>> {
        let gesture = self.gesture.as_mut()?;
        gesture.proof.is_current(engine).then(|| {
            gesture
                .candidate(engine, view, density)
                .map(|candidate| (candidate.document, candidate.graph))
        })
    }
}

fn placement_mut(params: &mut RingCreateRequest) -> &mut RingPlacement {
    match params {
        RingCreateRequest::Sized(params) => &mut params.placement,
        RingCreateRequest::Named(params) => &mut params.placement,
        RingCreateRequest::Chair(params) => &mut params.placement,
    }
}

fn supported_modifiers(modifiers: egui::Modifiers, named: bool) -> bool {
    modifiers.is_none() || (named && modifiers == egui::Modifiers::SHIFT)
}

fn candidate(
    engine: &Engine,
    params: &RingCreateRequest,
    view: &PageView,
    density: f32,
) -> Result<Candidate, Error> {
    let stored = engine.preview_ring_request(params)?;
    let page_id = match params {
        RingCreateRequest::Sized(params) => params.page_id,
        RingCreateRequest::Named(params) => params.page_id,
        RingCreateRequest::Chair(params) => params.page_id,
    };
    let page = stored
        .model()
        .pages()
        .iter()
        .find(|page| page.id().get() == page_id)
        .ok_or(bondline_model::Error::MissingPage(page_id))?;
    let graph = readable_graph(
        view.graph_scene(page, &bondline_render::Cancellation::default())?,
        density,
    )?;
    Ok(Candidate {
        params: params.clone(),
        document: stored.model_snapshot(),
        graph,
    })
}

fn fragment(page: &Page, id: FragmentId) -> Result<&Fragment, Error> {
    page.fragments()
        .iter()
        .find(|fragment| fragment.id() == id)
        .map(AsRef::as_ref)
        .ok_or_else(|| bondline_model::Error::MissingFragment(id.get()).into())
}

fn position(fragment: &Fragment, id: AtomId) -> Result<Point2, Error> {
    fragment
        .atoms()
        .iter()
        .find(|atom| atom.id == id)
        .ok_or(Error::MissingAtom {
            fragment_id: fragment.id().get(),
            atom_id: id.get(),
        })?
        .position
        .ok_or(Error::UnsupportedRing(
            "a ring target or neighbor has no authored position",
        ))
}

fn click_end(begin: Point2, size: u8) -> (f64, f64) {
    let (dx, dy) = if size == 6 { (0.0, -48.0) } else { (48.0, 0.0) };
    // A drag may replace an invalid click default; the candidate builder validates it.
    (begin.x_pt() + dx, begin.y_pt() + dy)
}

fn atom_click_end(
    fragment: &Fragment,
    id: AtomId,
    begin: Point2,
    size: u8,
) -> Result<(f64, f64), Error> {
    let Some(outward) = outward(fragment, id, begin)? else {
        return Ok(click_end(begin, size));
    };
    let interior = (f64::from(size) - 2.0) * std::f64::consts::PI / f64::from(size);
    let angle = outward.1.atan2(outward.0) - interior / 2.0;
    Ok((
        begin.x_pt() + 48.0 * angle.cos(),
        begin.y_pt() + 48.0 * angle.sin(),
    ))
}

fn outward(fragment: &Fragment, id: AtomId, begin: Point2) -> Result<Option<(f64, f64)>, Error> {
    let mut sum = (0.0, 0.0);
    let mut smallest = None;
    for bond in fragment.bonds() {
        let neighbor = if bond.begin == id {
            bond.end
        } else if bond.end == id {
            bond.begin
        } else {
            continue;
        };
        let point = position(fragment, neighbor)?;
        let dx = point.x_pt() - begin.x_pt();
        let dy = point.y_pt() - begin.y_pt();
        let length = dx.hypot(dy);
        if length == 0.0 {
            return Err(Error::UnsupportedRing(
                "a coincident neighbor has no ring orientation",
            ));
        }
        let unit = (dx / length, dy / length);
        sum.0 += unit.0;
        sum.1 += unit.1;
        if smallest.is_none_or(|(old, _)| neighbor < old) {
            smallest = Some((neighbor, unit));
        }
    }
    let Some((_, axis)) = smallest else {
        return Ok(None);
    };
    let length = sum.0.hypot(sum.1);
    let outward = if length <= 1e-12 {
        let normal = (axis.1, -axis.0);
        if normal.1 < 0.0 || (normal.1 == 0.0 && normal.0 > 0.0) {
            normal
        } else {
            (-normal.0, -normal.1)
        }
    } else {
        (-sum.0 / length, -sum.1 / length)
    };
    Ok(Some(outward))
}

fn chair_click_end(engine: &Engine, kind: ChairKind, begin: Point2) -> Result<(f64, f64), Error> {
    let length = engine
        .document()?
        .drawing_settings()
        .bond_length_pt()
        .unwrap_or(48.0);
    let sign = match kind {
        ChairKind::One => 1.0,
        ChairKind::Two => -1.0,
    };
    let y = sign * (2.0 * std::f64::consts::SQRT_2 + 3.0_f64.sqrt()) / (2.0 * 17.0_f64.sqrt());
    let scale = length / 0.5_f64.hypot(y);
    Ok((begin.x_pt() + 0.5 * scale, begin.y_pt() + y * scale))
}

fn chair_atom_click_end(
    engine: &Engine,
    kind: ChairKind,
    fragment: &Fragment,
    id: AtomId,
    begin: Point2,
) -> Result<(f64, f64), Error> {
    let Some(outward) = outward(fragment, id, begin)? else {
        return chair_click_end(engine, kind, begin);
    };
    let centroid = kind
        .reference_vertices()
        .iter()
        .fold((0.0, 0.0), |sum, [x, y]| (sum.0 + x, sum.1 + y));
    let angle = outward.1.atan2(outward.0) - centroid.1.atan2(centroid.0);
    let length = engine
        .document()?
        .drawing_settings()
        .bond_length_pt()
        .unwrap_or(48.0);
    Ok((
        begin.x_pt() + length * angle.cos(),
        begin.y_pt() + length * angle.sin(),
    ))
}

fn bond_anchor(
    page: &Page,
    fragment_id: FragmentId,
    bond_id: BondId,
) -> Result<(Point2, Point2, RingSide), Error> {
    let fragment = fragment(page, fragment_id)?;
    let bond = fragment
        .bonds()
        .iter()
        .find(|bond| bond.id == bond_id)
        .ok_or(Error::MissingBond {
            fragment_id: fragment_id.get(),
            bond_id: bond_id.get(),
        })?;
    let begin = position(fragment, bond.begin)?;
    let end = position(fragment, bond.end)?;
    let mut positive = false;
    let mut negative = false;
    for neighbor in fragment.bonds().iter().filter(|other| other.id != bond_id) {
        let other = if neighbor.begin == bond.begin || neighbor.begin == bond.end {
            Some(neighbor.end)
        } else if neighbor.end == bond.begin || neighbor.end == bond.end {
            Some(neighbor.begin)
        } else {
            None
        };
        if let Some(other) = other {
            let distance = cross(begin, end, position(fragment, other)?);
            positive |= distance > 0.0;
            negative |= distance < 0.0;
        }
    }
    let side = if positive && !negative {
        RingSide::Negative
    } else {
        RingSide::Positive
    };
    Ok((begin, end, side))
}

pub(super) fn chair_bond_command(
    engine: &Engine,
    page: &Page,
    fragment_id: FragmentId,
    bond_id: BondId,
    kind: ChairKind,
    view: &PageView,
    density: f32,
) -> Result<Command, Error> {
    let (_, _, side) = bond_anchor(page, fragment_id, bond_id)?;
    let mut params: RingCreateRequest = ChairRingCreate {
        expected_revision: engine.state().revision,
        page_id: page.id().get(),
        kind,
        hydrogen_policy: OrderHydrogenPolicy::RectifyNeutralCarbon,
        placement: RingPlacement::Bond {
            fragment_id: fragment_id.get(),
            bond_id: bond_id.get(),
            side,
        },
    }
    .into();
    match candidate(engine, &params, view, density) {
        Ok(candidate) => Ok(Command::RingCreate(candidate.params)),
        Err(first_error) => {
            if let RingPlacement::Bond { side, .. } = placement_mut(&mut params) {
                *side = match side {
                    RingSide::Positive => RingSide::Negative,
                    RingSide::Negative => RingSide::Positive,
                };
            }
            candidate(engine, &params, view, density)
                .map(|candidate| Command::RingCreate(candidate.params))
                .map_err(|_| first_error)
        }
    }
}

fn cross(begin: Point2, end: Point2, point: Point2) -> f64 {
    (end.x_pt() - begin.x_pt()) * (point.y_pt() - begin.y_pt())
        - (end.y_pt() - begin.y_pt()) * (point.x_pt() - begin.x_pt())
}
