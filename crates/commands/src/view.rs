//! Window view state is separate from authored content, saved baselines and history.

use crate::{CloseState, CommandResult, Engine, Error, StoredContent, UiFilePhase};
use bondline_model::{PageSize, Point2};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
#[schemars(extend("type" = "object", "unevaluatedProperties" = false))]
pub enum UiView {
    Inspect {},
    Fit {},
    Actual {},
    SetZoom {
        #[schemars(range(min = 1, max = 999))]
        percent: f64,
    },
    Magnify {
        #[schemars(range(min = 1, max = 100_000))]
        width_px: u32,
        #[schemars(range(min = 1, max = 100_000))]
        height_px: u32,
    },
    Reduce {
        #[schemars(range(min = 1, max = 100_000))]
        width_px: u32,
        #[schemars(range(min = 1, max = 100_000))]
        height_px: u32,
    },
    Pan {
        center_x_pt: f64,
        center_y_pt: f64,
    },
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ViewState {
    /// None selects the legacy fitted page; explicit values are percentages, not DPI.
    pub zoom_percent: Option<f64>,
    /// None centers the page; a retained center is clamped only when projecting a new page.
    pub center: Option<Point2>,
    pub epoch: u64,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct ViewOperation {
    pub zoom_percent: Option<f64>,
    pub center_x_pt: Option<f64>,
    pub center_y_pt: Option<f64>,
    pub epoch: u64,
    pub changed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prepared_sdf_extent: Option<PageSize>,
}

impl Engine {
    pub fn view(&self) -> ViewState {
        self.view
    }

    /// Display bounds only; a prepared collection never becomes an authored Drawing.
    pub fn view_page_size(&self) -> Result<PageSize, Error> {
        match self.content() {
            StoredContent::Sdf { .. } => self
                .sdf_view_extent
                .as_ref()
                .filter(|(stamp, _)| self.capture_stamp_matches(stamp))
                .map(|(_, size)| *size)
                .ok_or(Error::ViewTargetUnavailable),
            _ => self
                .document()?
                .pages()
                .first()
                .map(|page| page.size())
                .ok_or(Error::InvalidPageId),
        }
    }

    pub(super) fn install_sdf_view_extent(
        &mut self,
        context: &crate::capture::ViewCaptureContext,
        size: PageSize,
    ) -> bool {
        if !matches!(self.content(), StoredContent::Sdf { .. })
            || self.close_state() != CloseState::Open
            || self.close_policy.epoch != context.close_epoch
            || self.view().epoch != context.state.epoch
            || !self.capture_stamp_matches(&context.stamp)
            || self
                .sdf_view_extent
                .as_ref()
                .is_some_and(|(stamp, prior)| stamp == &context.stamp && *prior == size)
        {
            return false;
        }
        self.sdf_view_extent = Some((context.stamp.clone(), size));
        true
    }

    /// Current displayed percentage when native layout or an explicit zoom is known.
    pub fn view_percent(&self) -> Option<f64> {
        self.view.zoom_percent.or_else(|| {
            let [width, height] = self.view_canvas_size?;
            fit_percent(width, height, self.view_page_size().ok()?).ok()
        })
    }

    /// Resolve the same camera used for rendering, authored-point input and hits.
    pub fn page_view(
        &self,
        width: u32,
        height: u32,
        page_id: bondline_model::PageId,
    ) -> Result<bondline_render::PageView, Error> {
        if matches!(self.content(), StoredContent::Sdf { .. }) {
            if page_id.get() != 1 {
                return Err(bondline_model::Error::MissingPage(page_id.get()).into());
            }
            return page_view_for(self.view, width, height, self.view_page_size()?);
        }
        let page = self
            .document()?
            .pages()
            .iter()
            .find(|page| page.id() == page_id)
            .ok_or(bondline_model::Error::MissingPage(page_id.get()))?;
        page_view_for(self.view, width, height, page.size())
    }

    pub fn drawing_view_scene(
        &self,
        width: u32,
        height: u32,
        page_id: bondline_model::PageId,
        cancel: &bondline_render::Cancellation,
    ) -> Result<bondline_render::GraphScene, Error> {
        let view = self.page_view(width, height, page_id)?;
        Ok(bondline_io::display::drawing_page_scene_with_view(
            &view,
            self.stored_document()?,
            page_id,
            cancel,
        )?)
    }

    /// Native layout context for discoverable relative bindings, not an authored edit.
    pub fn set_view_canvas_size(&mut self, width: u32, height: u32) -> Result<(), Error> {
        validate_canvas(width, height)?;
        self.view_canvas_size = Some([width, height]);
        Ok(())
    }

    pub(super) fn navigate_view(&mut self, params: UiView) -> Result<CommandResult, Error> {
        let mut next = self.view;
        if !matches!(params, UiView::Inspect {}) {
            if self.close_state() != CloseState::Open
                || self
                    .file_ui_state()
                    .request
                    .is_some_and(|request| request.phase != UiFilePhase::Running)
            {
                return Err(Error::UiBusy);
            }
            let drawing_page = if matches!(self.content(), StoredContent::Sdf { .. }) {
                None
            } else {
                let stored = self.stored_document()?;
                if !stored.is_fully_mapped() {
                    return Err(Error::UnsupportedDocumentPreview);
                }
                Some(stored.model().pages().first().ok_or(Error::InvalidPageId)?)
            };
            match params {
                UiView::Inspect {} => {}
                UiView::Fit {} => {
                    next.zoom_percent = None;
                    next.center = None;
                }
                UiView::Actual {} => next.zoom_percent = Some(100.0),
                UiView::SetZoom { percent } => {
                    if !percent.is_finite() || !(1.0..=999.0).contains(&percent) {
                        return Err(Error::InvalidView(
                            "percentage must be finite within 1..999",
                        ));
                    }
                    next.zoom_percent = Some(percent);
                }
                UiView::Magnify {
                    width_px,
                    height_px,
                }
                | UiView::Reduce {
                    width_px,
                    height_px,
                } => {
                    validate_canvas(width_px, height_px)?;
                    let current = match self.view.zoom_percent {
                        Some(percent) => percent,
                        None => fit_percent(width_px, height_px, self.view_page_size()?)?,
                    };
                    let magnify = matches!(params, UiView::Magnify { .. });
                    if (magnify && current < 999.0) || (!magnify && current > 1.0) {
                        next.zoom_percent = Some(
                            if magnify {
                                current * 1.25
                            } else {
                                current / 1.25
                            }
                            .clamp(1.0, 999.0),
                        );
                        if magnify
                            && let Some(page) = drawing_page
                            && let Some((generation, page_id, focus)) = self.view_focus
                            && generation == self.generation
                            && page_id == page.id()
                        {
                            next.center = Some(Point2::new(
                                focus.x_pt().clamp(0.0, page.size().width_pt()),
                                focus.y_pt().clamp(0.0, page.size().height_pt()),
                            )?);
                        }
                    }
                }
                UiView::Pan {
                    center_x_pt,
                    center_y_pt,
                } => {
                    if self.view.zoom_percent.is_none() {
                        return Err(Error::UnsupportedView("select a zoom before panning"));
                    }
                    let size = self.view_page_size()?;
                    if !center_x_pt.is_finite()
                        || !center_y_pt.is_finite()
                        || !(0.0..=size.width_pt()).contains(&center_x_pt)
                        || !(0.0..=size.height_pt()).contains(&center_y_pt)
                    {
                        return Err(Error::InvalidView(
                            "center must be finite and inside the page",
                        ));
                    }
                    next.center = Some(Point2::new(center_x_pt, center_y_pt)?);
                }
            }
        }
        let changed = next != self.view;
        if changed {
            next.epoch = self
                .view
                .epoch
                .checked_add(1)
                .ok_or(Error::UiViewEpochExhausted)?;
            self.view = next;
        }
        Ok(CommandResult {
            ui_view_operation: Some(ViewOperation {
                zoom_percent: self.view.zoom_percent,
                center_x_pt: self.view.center.map(Point2::x_pt),
                center_y_pt: self.view.center.map(Point2::y_pt),
                epoch: self.view.epoch,
                changed,
                prepared_sdf_extent: matches!(self.content(), StoredContent::Sdf { .. })
                    .then(|| self.view_page_size().ok())
                    .flatten(),
            }),
            ..self.state()
        })
    }
}

pub(super) fn page_view_for(
    state: ViewState,
    width: u32,
    height: u32,
    size: PageSize,
) -> Result<bondline_render::PageView, Error> {
    Ok(match state.zoom_percent {
        None => bondline_render::PageView::fit(width, height, size)?,
        Some(percent) => {
            let center = state
                .center
                .unwrap_or(Point2::new(size.width_pt() / 2.0, size.height_pt() / 2.0)?);
            let center = Point2::new(
                center.x_pt().clamp(0.0, size.width_pt()),
                center.y_pt().clamp(0.0, size.height_pt()),
            )?;
            bondline_render::PageView::zoom(width, height, size, percent, center)?
        }
    })
}

fn validate_canvas(width: u32, height: u32) -> Result<(), Error> {
    if !(1..=100_000).contains(&width) || !(1..=100_000).contains(&height) {
        return Err(Error::InvalidView(
            "canvas dimensions must be within 1..100000 logical units",
        ));
    }
    Ok(())
}

pub(super) fn fit_percent(
    width: u32,
    height: u32,
    page: bondline_model::PageSize,
) -> Result<f64, Error> {
    let viewport = bondline_render::PageViewport::new(width, height, page)?;
    Ok((f64::from(viewport.paper().width) / page.width_pt())
        .min(f64::from(viewport.paper().height) / page.height_pt())
        * 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Command;

    #[test]
    fn exhausted_view_epoch_allows_inspection_and_noops_but_refuses_changes_atomically()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut engine = Engine::default();
        engine.view.epoch = u64::MAX;
        let state = serde_json::to_value(engine.state())?;
        let view = engine.view();
        let stamp = engine.capture_stamp();
        for action in [UiView::Inspect {}, UiView::Fit {}] {
            let result = engine.execute(Command::UiView(action))?;
            assert!(!result.ui_view_operation.ok_or("view operation")?.changed);
        }
        assert!(matches!(
            engine.execute(Command::UiView(UiView::Actual {})),
            Err(Error::UiViewEpochExhausted)
        ));
        assert_eq!(engine.view(), view);
        assert_eq!(engine.capture_stamp(), stamp);
        assert_eq!(serde_json::to_value(engine.state())?, state);
        Ok(())
    }

    #[test]
    fn typed_nonfinite_view_input_is_rejected_without_a_json_validation_dependency()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut engine = Engine::default();
        engine.execute(Command::UiView(UiView::Actual {}))?;
        let state = serde_json::to_value(engine.state())?;
        let view = engine.view();
        let stamp = engine.capture_stamp();
        for value in [f64::NAN, f64::NEG_INFINITY, f64::INFINITY] {
            for action in [
                UiView::SetZoom { percent: value },
                UiView::Pan {
                    center_x_pt: value,
                    center_y_pt: 100.0,
                },
                UiView::Pan {
                    center_x_pt: 100.0,
                    center_y_pt: value,
                },
            ] {
                assert!(matches!(
                    engine.execute(Command::UiView(action)),
                    Err(Error::InvalidView(_))
                ));
                assert_eq!(engine.view(), view);
                assert_eq!(engine.capture_stamp(), stamp);
                assert_eq!(serde_json::to_value(engine.state())?, state);
            }
        }
        Ok(())
    }
}
