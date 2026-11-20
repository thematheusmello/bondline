//! Browser hosting for the same egui workspace and typed command engine.
use crate::{DrawingState, configure_context, show_workspace, web_recovery};
use bondline_commands::Engine;
use bondline_model::Document;
use eframe::egui;
use std::sync::Arc;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WebHandle {
    runner: eframe::WebRunner,
}

impl Default for WebHandle {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl WebHandle {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            runner: eframe::WebRunner::new(),
        }
    }

    pub async fn start(
        &self,
        canvas: web_sys::HtmlCanvasElement,
        recovery_writer: bool,
    ) -> Result<(), JsValue> {
        let mut options = eframe::WebOptions::default();
        let descriptor = Arc::clone(&options.wgpu_options.device_descriptor);
        options.wgpu_options.device_descriptor = Arc::new(move |adapter| {
            let mut result = descriptor(adapter);
            result.required_limits.max_inter_stage_shader_components = result
                .required_limits
                .max_inter_stage_shader_components
                .min(adapter.limits().max_inter_stage_shader_components);
            result
        });
        self.runner
            .start(
                canvas,
                options,
                Box::new(move |cc| Ok(Box::new(BrowserApp::new(cc, recovery_writer)?))),
            )
            .await
    }

    /// Host automation uses the identical registry; unavailable native operations refuse.
    pub fn execute(&self, request: JsValue) -> Result<String, JsValue> {
        let request = request
            .dyn_ref::<js_sys::JsString>()
            .ok_or_else(|| JsValue::from_str("A JSON request string is required."))?;
        if request.length() as usize > bondline_commands::MAX_REQUEST_BYTES {
            return Err(JsValue::from_str(
                "The JSON request exceeds the 1 MiB limit.",
            ));
        }
        let request = request
            .as_string()
            .ok_or_else(|| JsValue::from_str("A JSON request string is required."))?;
        let mut app = self
            .runner
            .app_mut::<BrowserApp>()
            .ok_or_else(|| JsValue::from_str("The workspace is not running."))?;
        let result = match app.engine.execute_json(request.as_bytes()) {
            Ok(result) => serde_json::to_value(result),
            Err(error) => Ok(serde_json::json!({"error":error.info()})),
        }
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
        app.persist_changes();
        app.context.request_repaint();
        serde_json::to_string(&result).map_err(|error| JsValue::from_str(&error.to_string()))
    }

    pub fn registry(&self) -> Result<String, JsValue> {
        serde_json::to_string(&bondline_commands::registry())
            .map_err(|error| JsValue::from_str(&error.to_string()))
    }

    pub fn recovery_key(&self) -> String {
        web_recovery::KEY.to_owned()
    }

    pub fn host_status(&self) -> Result<String, JsValue> {
        let app = self
            .runner
            .app_mut::<BrowserApp>()
            .ok_or_else(|| JsValue::from_str("The workspace is not running."))?;
        serde_json::to_string(&serde_json::json!({"error":app.error,"recovery_allowed":app.recovery_allowed,"recovery":app.recovery_message,
            "unsaved_in_tab":(app.engine.state().file.dirty && !app.recovery_current) || app.drawing.authoring_modal(),
            "zoom_percent":app.engine.view().zoom_percent}))
            .map_err(|error| JsValue::from_str(&error.to_string()))
    }

    /// Logical canvas geometry for browser accessibility/automation, never a hit proof.
    pub fn canvas_geometry(&self) -> Result<String, JsValue> {
        let app = self
            .runner
            .app_mut::<BrowserApp>()
            .ok_or_else(|| JsValue::from_str("The workspace is not running."))?;
        let key = app
            .drawing
            .hover_context
            .ok_or_else(|| JsValue::from_str("The canvas has not painted."))?;
        serde_json::to_string(
            &serde_json::json!({"x":key.canvas_origin.x,"y":key.canvas_origin.y,
            "width":key.width,"height":key.height,"pixels_per_point":key.pixels_per_point}),
        )
        .map_err(|error| JsValue::from_str(&error.to_string()))
    }

    pub fn destroy(&self) {
        self.runner.destroy();
    }
}

struct BrowserApp {
    context: egui::Context,
    engine: Engine,
    drawing: DrawingState,
    error: Option<String>,
    storage: Option<web_sys::Storage>,
    recovery_allowed: bool,
    recovery_current: bool,
    observed_source: Option<String>,
    observed: Arc<Document>,
    recovery_message: String,
}

impl BrowserApp {
    fn new(
        cc: &eframe::CreationContext<'_>,
        recovery_writer: bool,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        if cc.wgpu_render_state.is_none() {
            return Err(std::io::Error::other("The browser could not initialize WebGPU.").into());
        }
        configure_context(&cc.egui_ctx);
        let mut engine = Engine::default();
        let mut recovery_allowed = recovery_writer;
        let mut recovery_current = false;
        let mut observed_source = None;
        let mut message = if recovery_writer {
            "Changes are kept in this browser. Local file controls are unavailable."
        } else {
            "Browser recovery is unavailable in this tab. Close other Bondline tabs and reload to continue."
        }.to_owned();
        let storage = match web_sys::window()
            .ok_or_else(|| JsValue::from_str("Browser window unavailable"))
            .and_then(|window| window.local_storage())
        {
            Ok(Some(storage)) => {
                match storage.get_item(web_recovery::KEY) {
                    Ok(Some(source)) => match web_recovery::decode(&source) {
                        Ok(stored) => {
                            engine = Engine::from_document(stored);
                            observed_source = Some(source);
                            recovery_current = true;
                            if recovery_writer {
                                message = "Restored the drawing kept in this browser. Local file controls are unavailable.".into();
                            }
                        }
                        Err(error) => {
                            recovery_allowed = false;
                            message = format!(
                                "Earlier browser data is preserved: {error} Changes in this tab are not being saved."
                            );
                        }
                    },
                    Ok(None) => {}
                    Err(_) => {
                        recovery_allowed = false;
                        message = "Browser recovery could not be read. Changes in this tab are not being saved.".into();
                    }
                }
                Some(storage)
            }
            _ => {
                recovery_allowed = false;
                message =
                    "Browser recovery is unavailable. Changes in this tab are not being saved."
                        .into();
                None
            }
        };
        // A fresh browser session starts readable even in a short browser viewport.
        engine.dispatch("ui.view", serde_json::json!({"action":"actual"}))?;
        let observed = engine.stored_document()?.model_snapshot();
        Ok(Self {
            context: cc.egui_ctx.clone(),
            engine,
            drawing: DrawingState::default(),
            error: None,
            storage,
            recovery_allowed,
            recovery_current,
            observed_source,
            observed,
            recovery_message: message,
        })
    }

    fn persist_changes(&mut self) {
        let Ok(document) = self.engine.stored_document() else {
            return;
        };
        let current = document.model_snapshot();
        if Arc::ptr_eq(&current, &self.observed) {
            return;
        }
        self.observed = current;
        self.recovery_current = false;
        if !self.recovery_allowed {
            return;
        }
        let result = web_recovery::encode(document).and_then(|source| {
            let storage = self.storage.as_ref().ok_or_else(|| "Browser recovery is unavailable.".to_owned())?;
            let actual = storage.get_item(web_recovery::KEY).map_err(|_| "Browser recovery could not be read; the earlier data is preserved.".to_owned())?;
            if actual != self.observed_source {
                self.recovery_allowed = false;
                return Err("The browser draft changed elsewhere. That data is preserved; changes in this tab are not being saved.".to_owned());
            }
            storage.set_item(web_recovery::KEY, &source)
                .map_err(|_| "Browser recovery could not be saved. The earlier stored drawing is unchanged.".to_owned())?;
            self.observed_source = Some(source);
            self.recovery_current = true;
            Ok(())
        });
        self.recovery_message = match result {
            Ok(()) => "Drawing kept in this browser. Local file controls are unavailable.".into(),
            Err(error) => error,
        };
    }
}

impl eframe::App for BrowserApp {
    fn update(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::bottom("browser-recovery").show(context, |ui| {
            ui.label(&self.recovery_message);
        });
        self.drawing
            .sync_authoring_forms(&self.engine, context, &mut self.error, false);
        show_workspace(
            context,
            &mut self.engine,
            &mut self.error,
            &mut self.drawing,
        );
        self.persist_changes();
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }
}
