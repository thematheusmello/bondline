//! Bounded native-input evidence around the actual app; no document or text capture.
use eframe::egui::{self, Event};
use serde_json::{Value, json};
use std::{
    fs::File,
    io::{self, Write},
    sync::{Arc, Mutex},
};

#[derive(Default)]
struct Trace {
    rows: Vec<Value>,
    events: usize,
    truncated: bool,
    last_modifiers: Option<egui::Modifiers>,
}

struct InputReview {
    app: bondline_ui::BondlineApp,
    trace: Arc<Mutex<Trace>>,
    frame: u64,
}

fn observation(event: &Event) -> Option<String> {
    match event {
        Event::PointerMoved(_)
        | Event::PointerButton { .. }
        | Event::WindowFocused(_)
        | Event::ModifiersChanged(_) => Some(format!("{event:?}")),
        _ => None,
    }
}

impl eframe::App for InputReview {
    fn raw_input_hook(&mut self, context: &egui::Context, input: &mut egui::RawInput) {
        self.frame = self.frame.saturating_add(1);
        if let Ok(mut trace) = self.trace.lock() {
            let changed = trace.last_modifiers != Some(input.modifiers);
            trace.last_modifiers = Some(input.modifiers);
            let mut events = Vec::new();
            for event in &input.events {
                if let Some(value) = observation(event) {
                    if trace.events == 4096 {
                        trace.truncated = true;
                        break;
                    }
                    trace.events += 1;
                    events.push(value);
                }
            }
            if !events.is_empty() || changed {
                if trace.rows.len() == 4096 {
                    trace.truncated = true;
                } else {
                    trace.rows.push(json!({
                        "frame": self.frame, "viewport": format!("{:?}", input.viewport_id),
                        "focused": input.focused, "ctrl": input.modifiers.ctrl,
                        "modifiers": format!("{:?}", input.modifiers), "events": events,
                    }));
                }
            }
        }
        self.app.raw_input_hook(context, input);
    }

    fn update(&mut self, context: &egui::Context, frame: &mut eframe::Frame) {
        self.app.update(context, frame);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let executable = std::env::current_exe()?;
    let directory = executable
        .parent()
        .ok_or_else(|| io::Error::other("executable directory"))?;
    let mut file = File::create_new(directory.join("bondline-input-trace.jsonl"))?;
    let trace = Arc::new(Mutex::new(Trace::default()));
    let app_trace = Arc::clone(&trace);
    eframe::run_native(
        "Bondline Input Review",
        bondline_ui::native_options()?,
        Box::new(move |cc| {
            Ok(Box::new(InputReview {
                app: bondline_ui::BondlineApp::new(cc, false, None)?,
                trace: app_trace,
                frame: 0,
            }))
        }),
    )?;
    let trace = trace
        .lock()
        .map_err(|_| io::Error::other("input trace poisoned"))?;
    for row in &trace.rows {
        serde_json::to_writer(&mut file, row)?;
        file.write_all(b"\n")?;
    }
    file.flush()?;
    if trace.truncated {
        return Err(io::Error::other("input trace exceeded4096 events").into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn trace_excludes_text_clipboard_ime_screenshots_and_keys() {
        use super::*;
        for event in [
            Event::Text("private".into()),
            Event::Paste("private".into()),
            Event::Ime(egui::ImeEvent::Preedit("private".into())),
            Event::Screenshot {
                viewport_id: egui::ViewportId::ROOT,
                image: Arc::new(egui::ColorImage::new([1, 1], egui::Color32::WHITE)),
            },
            Event::Key {
                key: egui::Key::A,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            },
        ] {
            assert_eq!(observation(&event), None);
        }
        assert_eq!(
            observation(&Event::WindowFocused(false)),
            Some("WindowFocused(false)".into())
        );
        assert_eq!(
            observation(&Event::ModifiersChanged(egui::Modifiers::CTRL)),
            Some("ModifiersChanged(Modifiers { ctrl: true })".into())
        );
    }
}
