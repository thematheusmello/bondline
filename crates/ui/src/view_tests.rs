//! Independent original-A4 view and numeric field acceptance through actual App frames.
use super::*;
use egui::{Event, Key, Modifiers, PointerButton, Pos2};
use serde_json::{Value, json};
use std::{
    collections::{HashMap, HashSet},
    fs,
    process::{Command as Process, Stdio},
};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(
    dead_code,
    clippy::duplicate_mod,
    reason = "reuse the original atomic test-directory helper without changing old tests"
)]
mod support;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

struct Fixture {
    app: BondlineApp,
    context: egui::Context,
    directory: support::FileTestDirectory,
    nodes: HashMap<egui::accesskit::NodeId, egui::accesskit::Node>,
    width: f32,
    height: f32,
    dpi: f32,
}
impl Fixture {
    fn new(width: f32, height: f32, dpi: f32) -> TestResult<Self> {
        let directory = support::FileTestDirectory::new()?;
        let mut engine = Engine::default();
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
            smiles_records: crate::smiles_records::SmilesRecordsWorkflow::default(),
        };
        let context = egui::Context::default();
        configure_context(&context);
        context.enable_accesskit();
        Ok(Self {
            app,
            context,
            directory,
            nodes: HashMap::new(),
            width,
            height,
            dpi,
        })
    }
    fn frame(&mut self, events: Vec<Event>) -> egui::FullOutput {
        let mut input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                Pos2::ZERO,
                egui::vec2(self.width, self.height),
            )),
            events,
            focused: true,
            ..Default::default()
        };
        input
            .viewports
            .entry(egui::ViewportId::ROOT)
            .or_default()
            .native_pixels_per_point = Some(self.dpi);
        let context = self.context.clone();
        let output = context.run(input, |ctx| self.app.update_workspace(ctx));
        if let Some(update) = &output.platform_output.accesskit_update {
            self.nodes.extend(update.nodes.iter().cloned());
        }
        output
    }
    fn warm(&mut self) -> egui::FullOutput {
        self.frame(vec![]);
        self.frame(vec![])
    }
    fn command(&self, name: &str, params: Value) -> TestResult<Value> {
        Ok(serde_json::to_value(
            self.app
                .engine
                .lock()
                .map_err(|_| "engine")?
                .dispatch(name, params)?,
        )?)
    }
    fn state(&self) -> TestResult<Value> {
        Ok(serde_json::to_value(
            self.app.engine.lock().map_err(|_| "engine")?.state(),
        )?)
    }
    fn view(&self) -> TestResult<Value> {
        self.command("ui.view", json!({"action":"inspect"}))?
            .get("ui_view_operation")
            .cloned()
            .ok_or_else(|| "view operation".into())
    }
    fn node(
        &self,
        name: &str,
        roles: &[egui::accesskit::Role],
    ) -> TestResult<(egui::accesskit::NodeId, egui::Rect)> {
        let root = egui::accesskit::NodeId(egui::accesskit_root_id().value());
        let mut seen = HashSet::new();
        let mut stack = vec![root];
        while let Some(id) = stack.pop() {
            if seen.insert(id)
                && let Some(node) = self.nodes.get(&id)
            {
                stack.extend_from_slice(node.children());
            }
        }
        self.nodes
            .iter()
            .filter(|(id, node)| {
                seen.contains(id)
                    && roles.contains(&node.role())
                    && node.supports_action(egui::accesskit::Action::Focus)
            })
            .find_map(|(id, node)| {
                let labelled = node.name() == Some(name)
                    || node.labelled_by().iter().any(|id| {
                        seen.contains(id)
                            && self
                                .nodes
                                .get(id)
                                .is_some_and(|label| label.name() == Some(name))
                    });
                if !labelled {
                    return None;
                }
                let r = node.bounds()?;
                Some((
                    *id,
                    egui::Rect::from_min_max(
                        egui::pos2(r.x0 as f32, r.y0 as f32),
                        egui::pos2(r.x1 as f32, r.y1 as f32),
                    ),
                ))
            })
            .ok_or_else(|| format!("current actionable {name} field/control missing").into())
    }
    fn choose(&mut self, label: &str) -> TestResult {
        self.warm();
        let (_, r) = self.node(label, &[egui::accesskit::Role::Button])?;
        self.frame(vec![
            Event::PointerMoved(r.center()),
            button(r.center(), true),
        ]);
        self.frame(vec![button(r.center(), false)]);
        Ok(())
    }
}
fn button(pos: Pos2, pressed: bool) -> Event {
    Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    }
}
fn key(code: Key, pressed: bool, modifiers: Modifiers) -> Event {
    Event::Key {
        key: code,
        physical_key: Some(code),
        pressed,
        repeat: false,
        modifiers,
    }
}
fn tap(code: Key) -> Vec<Event> {
    vec![
        key(code, true, Modifiers::NONE),
        key(code, false, Modifiers::NONE),
    ]
}
fn canvas(output: &egui::FullOutput) -> TestResult<egui::Rect> {
    output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::Shape::Rect(r)
                if r.fill == egui::Color32::from_gray(232)
                    && r.rect.width() > 400.0
                    && r.rect.height() > 400.0 =>
            {
                Some(r.rect.intersect(shape.clip_rect))
            }
            _ => None,
        })
        .max_by(|a, b| a.area().total_cmp(&b.area()))
        .ok_or_else(|| "actual canvas background".into())
}
fn projected(
    canvas: egui::Rect,
    page: [f64; 2],
    percent: f64,
    center: Option<[f64; 2]>,
    point: [f64; 2],
) -> Pos2 {
    let scale = percent / 100.0;
    let center = center.unwrap_or([page[0] / 2.0, page[1] / 2.0]);
    // Declared independent camera policy: exact uniform scale, nearest integer translation.
    let offset = [
        (f64::from(canvas.width().floor()) / 2.0 - center[0] * scale).round(),
        (f64::from(canvas.height().floor()) / 2.0 - center[1] * scale).round(),
    ];
    canvas.min
        + egui::vec2(
            (point[0] * scale + offset[0]) as f32,
            (point[1] * scale + offset[1]) as f32,
        )
}
fn pair(index: u32, order: &str, h: u16, y: f64) -> Value {
    let a = index * 2 - 1;
    let b = index * 2;
    json!({"id":index,"atoms":[{"id":a,"element":6,"position":{"x_pt":220.0,"y_pt":y},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":h},{"id":b,"element":6,"position":{"x_pt":300.0,"y_pt":y},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":h}],"bonds":[{"id":index,"begin":a,"end":b,"order":order}]})
}
fn independent_file(path: &std::path::Path, expected: &Value) -> TestResult {
    let python = std::env::var_os("BONDLINE_PYTHON").ok_or("pinned external interpreter")?;
    let mut child = Process::new(&python)
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../xtask/oracles/cdxml_graph.py"
        ))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("oracle stdin")?
        .write_all(&fs::read(path)?)?;
    let result = child.wait_with_output()?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&result.stdout)?,
        expected["pages"][0]["fragments"]
    );
    let result=Process::new(python).args(["-B","-c",r#"
from pathlib import Path
from rdkit import Chem,rdBase
import sys
assert rdBase.rdkitVersion=='2026.09.2'
ms=Chem.MolsFromCDXML(Path(sys.argv[1]).read_text(encoding='utf-8'));assert len(ms)==2
for m,w,h in zip(ms,[2,3],[2,1]):
 assert m.GetNumAtoms()==2 and m.GetNumBonds()==1
 assert m.GetBondWithIdx(0).GetBondTypeAsDouble()==w
 assert all(a.GetAtomicNum()==6 and a.GetFormalCharge()==0 and a.GetIsotope()==0 and a.GetTotalNumHs()==h for a in m.GetAtoms())
"#]).arg(path).output()?;
    assert!(
        result.status.success(),
        "actual saved-file chemistry: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(())
}

#[test]
fn native_sized_original_a4_actual_and_zoom_field_enable_multiple_bonds_without_page_edits()
-> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let controls = f.command("ui.controls", json!({}))?;
    let rows = controls["controls"].as_array().ok_or("controls")?;
    for (action, key) in [
        ("actual", "f5"),
        ("fit", "f6"),
        ("magnify", "f7"),
        ("reduce", "f8"),
    ] {
        let row = rows
            .iter()
            .find(|row| row["menu"] == "View" && row["command"]["params"]["action"] == action)
            .ok_or_else(|| format!("central View binding {action} missing"))?;
        assert_eq!(row["command"]["command"], "ui.view");
        assert_eq!(
            row["shortcut"],
            json!({"key":key,"primary":false,"shift":false})
        );
        if matches!(action, "magnify" | "reduce") {
            assert_eq!(row["enabled"], false);
            assert_eq!(row["disabled_reason"], "viewport_unavailable");
        }
    }
    f.warm();
    let blank = f.state()?;
    let page = [210.0 * 72.0 / 25.4, 297.0 * 72.0 / 25.4];
    assert_eq!(
        blank["document"]["pages"][0]["size"],
        json!({"width_pt":page[0],"height_pt":page[1]})
    );
    let output = f.frame(tap(Key::F5));
    assert_eq!(f.view()?["zoom_percent"], 100.0);
    assert_eq!(
        f.state()?,
        blank,
        "Actual changes appearance, never the physical page/history/source"
    );
    let area = canvas(&output)?;
    let white = output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::Shape::Rect(r) if r.fill == egui::Color32::WHITE => {
                Some(r.rect.intersect(shape.clip_rect))
            }
            _ => None,
        })
        .max_by(|a, b| a.area().total_cmp(&b.area()))
        .ok_or("white paper")?;
    assert!(
        white.top() <= area.top() + 1.0 && white.bottom() >= area.bottom() - 1.0,
        "Actual A4 must clip vertically in this native-sized canvas rather than silently remain Fit"
    );
    f.choose("Double bond")?;
    let output = f.warm();
    let area = canvas(&output)?;
    let a = projected(area, page, 100.0, None, [220.0, 380.0]);
    let b = projected(area, page, 100.0, None, [300.0, 380.0]);
    f.frame(vec![
        Event::PointerMoved(a),
        button(a, true),
        Event::PointerMoved(b),
        button(b, false),
    ]);
    let mut expected = blank["document"].clone();
    expected["pages"][0]["fragments"] = json!([pair(1, "double", 2, 380.0)]);
    assert_eq!(
        f.state()?["document"],
        expected,
        "actual-size inverse maps native-sized original A4 input exactly"
    );
    assert!(f.app.error.is_none(), "{:?}", f.app.error);
    let before_zoom = f.state()?;
    f.frame(tap(Key::F7));
    let zoom = f.view()?;
    assert_eq!(
        (
            zoom["zoom_percent"].clone(),
            zoom["center_x_pt"].clone(),
            zoom["center_y_pt"].clone()
        ),
        (json!(125.0), json!(260.0), json!(380.0))
    );
    assert_eq!(f.state()?, before_zoom);
    let (id, _) = f.node(
        "Zoom",
        &[
            egui::accesskit::Role::SpinButton,
            egui::accesskit::Role::TextInput,
        ],
    )?;
    f.frame(vec![Event::AccessKitActionRequest(
        egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::Focus,
            target: id,
            data: None,
        },
    )]);
    let focused = f
        .context
        .memory(|m| m.focused())
        .ok_or("Zoom keyboard focus")?;
    assert!(
        egui::TextEdit::load_state(&f.context, focused).is_some(),
        "Zoom must be actual text editing"
    );
    f.frame(vec![
        key(Key::A, true, Modifiers::COMMAND),
        key(Key::A, false, Modifiers::COMMAND),
        Event::Text("150".to_owned()),
    ]);
    assert_eq!(
        f.view()?,
        zoom,
        "editing Zoom does not apply an uncommitted number"
    );
    f.frame(tap(Key::F5));
    assert_eq!(
        f.view()?,
        zoom,
        "F5 must not escape numeric-field text focus"
    );
    f.frame(tap(Key::Enter));
    assert_eq!(f.view()?["zoom_percent"], 150.0);
    assert_eq!(
        (
            f.view()?["center_x_pt"].clone(),
            f.view()?["center_y_pt"].clone()
        ),
        (json!(260.0), json!(380.0))
    );
    assert_eq!(f.state()?, before_zoom);
    f.choose("Triple bond")?;
    let output = f.warm();
    let area = canvas(&output)?;
    let a = projected(area, page, 150.0, Some([260.0, 380.0]), [220.0, 460.0]);
    let b = projected(area, page, 150.0, Some([260.0, 380.0]), [300.0, 460.0]);
    let output = f.frame(vec![
        Event::PointerMoved(a),
        button(a, true),
        Event::PointerMoved(b),
        button(b, false),
    ]);
    expected["pages"][0]["fragments"] =
        json!([pair(1, "double", 2, 380.0), pair(2, "triple", 1, 460.0)]);
    assert_eq!(f.state()?["document"], expected);
    assert_eq!(f.state()?["revision"], 2);
    assert_eq!(f.state()?["undo_depth"], 2);
    assert!(f.app.error.is_none(), "{:?}", f.app.error);
    let count=output.shapes.iter().filter(|shape|matches!(&shape.shape,egui::Shape::Path(path)if path.closed&&path.points.len()==4&&path.fill==egui::Color32::BLACK)).count();
    assert_eq!(
        count, 5,
        "all actual Double/Triple vector components remain visible"
    );
    let path = f.directory.path("a4-view-multiple-化学.cdxml");
    f.command("document.save_cdxml", json!({"path":path}))?;
    independent_file(&path, &expected)?;
    let view = f.view()?;
    f.command("history.undo", json!({}))?;
    let mut one = expected.clone();
    one["pages"][0]["fragments"] = json!([pair(1, "double", 2, 380.0)]);
    assert_eq!(f.state()?["document"], one);
    assert_eq!(f.view()?, view);
    f.command("history.redo", json!({}))?;
    assert_eq!(f.state()?["document"], expected);
    assert_eq!(f.view()?, view);
    f.command("document.open_cdxml", json!({"path":path}))?;
    assert_eq!(
        f.state()?["document"],
        expected,
        "Open resets authoring history without changing window view"
    );
    assert_eq!(f.state()?["undo_depth"], 0);
    assert_eq!(f.view()?, view);
    Ok(())
}

#[test]
fn actual_native_sized_frame_has_a_current_focusable_zoom_field_without_authoring_changes()
-> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let before = f.state()?;
    let output = f.warm();
    let (_, bounds) = f.node(
        "Zoom",
        &[
            egui::accesskit::Role::SpinButton,
            egui::accesskit::Role::TextInput,
        ],
    )?;
    assert!(bounds.width() > 0.0 && bounds.height() > 0.0);
    assert!(egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(1160.0, 800.0)).contains_rect(bounds));
    assert!(canvas(&output)?.height() > 0.0);
    assert_eq!(
        f.state()?,
        before,
        "actual frame/metrics infrastructure must not edit or dirty original A4"
    );
    Ok(())
}
#[test]
fn ordered_view_a_b_a_cancels_old_gesture_and_allows_later_fresh_input_at_dpi_two() -> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 2.0)?;
    f.warm();
    f.frame(tap(Key::F5));
    assert_eq!(f.view()?["zoom_percent"], 100.0);
    f.choose("Double bond")?;
    let output = f.warm();
    let area = canvas(&output)?;
    let page = [210.0 * 72.0 / 25.4, 297.0 * 72.0 / 25.4];
    let a = projected(area, page, 100.0, None, [220.0, 380.0]);
    let b = projected(area, page, 100.0, None, [300.0, 380.0]);
    let c = projected(area, page, 100.0, None, [220.0, 460.0]);
    let d = projected(area, page, 100.0, None, [300.0, 460.0]);
    let before = f.state()?;
    let epoch = f.view()?["epoch"].as_u64().ok_or("view epoch")?;
    let output = f.frame(vec![
        Event::PointerMoved(a),
        button(a, true),
        Event::PointerMoved(b),
    ]);
    assert_eq!(
        f.state()?,
        before,
        "an actual held preview is not an authored edit"
    );
    let preview_components = output
        .shapes
        .iter()
        .filter(|shape| {
            matches!(&shape.shape,
                egui::Shape::Path(path) if path.closed && path.points.len() == 4
                    && path.fill != egui::Color32::TRANSPARENT
            )
        })
        .count();
    assert_eq!(
        preview_components, 2,
        "the armed Double preview has two real vector components"
    );
    let mut events = tap(Key::F6);
    events.extend(tap(Key::F5));
    events.extend([
        Event::PointerMoved(b),
        button(b, false),
        Event::PointerMoved(c),
        button(c, true),
        Event::PointerMoved(d),
        button(d, false),
    ]);
    f.frame(events);
    let mut expected = before["document"].clone();
    expected["pages"][0]["fragments"] = json!([pair(1, "double", 2, 460.0)]);
    assert_eq!(
        f.state()?["document"],
        expected,
        "Fit→Actual must retire the earlier gesture before a later same-packet press"
    );
    assert_eq!(f.state()?["revision"], 1);
    assert_eq!(f.state()?["undo_depth"], 1);
    assert_eq!(f.view()?["epoch"], epoch + 2);
    assert_eq!(f.view()?["zoom_percent"], 100.0);
    assert!(f.app.error.is_none(), "{:?}", f.app.error);

    // A true no-op Actual action must preserve an armed gesture's current context.
    let a = projected(area, page, 100.0, None, [220.0, 540.0]);
    let b = projected(area, page, 100.0, None, [300.0, 540.0]);
    f.frame(vec![Event::PointerMoved(a), button(a, true)]);
    let mut events = tap(Key::F5);
    events.extend([Event::PointerMoved(b), button(b, false)]);
    f.frame(events);
    expected["pages"][0]["fragments"] =
        json!([pair(1, "double", 2, 460.0), pair(2, "double", 2, 540.0)]);
    assert_eq!(f.state()?["document"], expected);
    assert_eq!(f.state()?["revision"], 2);
    assert_eq!(f.state()?["undo_depth"], 2);
    assert_eq!(f.view()?["epoch"], epoch + 2);
    assert!(f.app.error.is_none(), "{:?}", f.app.error);
    Ok(())
}
fn wheel(delta: [f32; 2], modifiers: Modifiers) -> Event {
    Event::MouseWheel {
        unit: egui::MouseWheelUnit::Point,
        delta: egui::vec2(delta[0], delta[1]),
        modifiers,
    }
}

#[test]
fn actual_pan_sliders_and_plain_wheel_preserve_authoring_and_require_fresh_uncovered_input()
-> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 2.0)?;
    f.warm();
    f.choose("Double bond")?;
    f.command("ui.view", json!({"action":"set_zoom","percent":200}))?;
    f.command(
        "ui.view",
        json!({"action":"pan","center_x_pt":200,"center_y_pt":300}),
    )?;
    let before = f.state()?;
    let output = f.warm();
    let area = canvas(&output)?;
    let p = area.center();
    let (horizontal, horizontal_bounds) =
        f.node("Horizontal position", &[egui::accesskit::Role::Slider])?;
    let (_, vertical_bounds) = f.node("Vertical position", &[egui::accesskit::Role::Slider])?;
    let screen = egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(1160.0, 800.0));
    assert!(screen.contains_rect(horizontal_bounds) && screen.contains_rect(vertical_bounds));
    f.frame(vec![Event::AccessKitActionRequest(
        egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::SetValue,
            target: horizontal,
            data: Some(egui::accesskit::ActionData::NumericValue(240.0)),
        },
    )]);
    assert_eq!(
        (
            f.view()?["center_x_pt"].clone(),
            f.view()?["center_y_pt"].clone()
        ),
        (json!(240.0), json!(300.0))
    );
    let epoch = f.view()?["epoch"].as_u64().ok_or("view epoch")?;
    f.frame(vec![
        Event::PointerMoved(p),
        wheel([20.0, -40.0], Modifiers::NONE),
    ]);
    assert_eq!(
        (
            f.view()?["center_x_pt"].clone(),
            f.view()?["center_y_pt"].clone()
        ),
        (json!(230.0), json!(320.0)),
        "Point-wheel delta moves content; camera center changes by minus delta / 2 at 200%"
    );
    assert_eq!(f.view()?["epoch"], epoch + 1);
    assert_eq!(f.state()?, before);

    let fixed = f.view()?;
    f.frame(vec![
        Event::PointerMoved(egui::pos2(1.0, 1.0)),
        wheel([20.0, -40.0], Modifiers::NONE),
    ]);
    assert_eq!(
        f.view()?,
        fixed,
        "wheel outside the chemical canvas cannot pan"
    );
    for modifiers in [
        Modifiers::SHIFT,
        Modifiers::CTRL,
        Modifiers::ALT,
        Modifiers::COMMAND,
    ] {
        f.frame(vec![
            Event::PointerMoved(p),
            wheel([20.0, -40.0], modifiers),
        ]);
        assert_eq!(f.view()?, fixed, "modified wheel cannot act as plain pan");
    }
    f.frame(vec![
        Event::PointerMoved(p),
        Event::WindowFocused(false),
        Event::WindowFocused(true),
    ]);
    f.frame(vec![wheel([20.0, -40.0], Modifiers::NONE)]);
    assert_eq!(
        f.view()?,
        fixed,
        "focus restoration alone cannot reuse a stale positional context"
    );
    f.frame(vec![
        Event::PointerMoved(p),
        wheel([20.0, -40.0], Modifiers::NONE),
    ]);
    assert_eq!(
        (
            f.view()?["center_x_pt"].clone(),
            f.view()?["center_y_pt"].clone()
        ),
        (json!(220.0), json!(340.0)),
        "a fresh foreground-valid point rearms plain wheel"
    );
    let fixed = f.view()?;
    f.warm();
    let (zoom, _) = f.node(
        "Zoom",
        &[
            egui::accesskit::Role::SpinButton,
            egui::accesskit::Role::TextInput,
        ],
    )?;
    f.frame(vec![Event::AccessKitActionRequest(
        egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::Focus,
            target: zoom,
            data: None,
        },
    )]);
    let focused = f.context.memory(|m| m.focused()).ok_or("Zoom text focus")?;
    assert!(egui::TextEdit::load_state(&f.context, focused).is_some());
    f.frame(vec![
        Event::PointerMoved(p),
        wheel([20.0, -40.0], Modifiers::NONE),
    ]);
    assert_eq!(
        f.view()?,
        fixed,
        "text editing suppresses canvas pan even with a fresh position"
    );
    f.choose("Double bond")?;
    let output = f.warm();
    let area = canvas(&output)?;
    let page = [210.0 * 72.0 / 25.4, 297.0 * 72.0 / 25.4];
    let a = projected(area, page, 200.0, Some([220.0, 340.0]), [220.0, 380.0]);
    let b = projected(area, page, 200.0, Some([220.0, 340.0]), [300.0, 380.0]);
    f.frame(vec![Event::PointerMoved(a), button(a, true)]);
    f.frame(vec![
        Event::PointerMoved(b),
        wheel([20.0, 0.0], Modifiers::NONE),
        button(b, false),
    ]);
    assert_eq!(
        f.state()?,
        before,
        "real wheel pan retires the armed gesture before its old release"
    );
    assert_eq!(
        (
            f.view()?["center_x_pt"].clone(),
            f.view()?["center_y_pt"].clone()
        ),
        (json!(210.0), json!(340.0))
    );
    let epoch = f.view()?["epoch"].as_u64().ok_or("view epoch")?;
    let a = projected(area, page, 200.0, Some([210.0, 340.0]), [220.0, 380.0]);
    let b = projected(area, page, 200.0, Some([210.0, 340.0]), [300.0, 380.0]);
    f.frame(vec![Event::PointerMoved(a), button(a, true)]);
    f.frame(vec![
        Event::PointerMoved(b),
        wheel([0.0, 0.0], Modifiers::NONE),
        button(b, false),
    ]);
    let mut expected = before["document"].clone();
    expected["pages"][0]["fragments"] = json!([pair(1, "double", 2, 380.0)]);
    assert_eq!(
        f.state()?["document"],
        expected,
        "zero wheel is a true view no-op and must preserve current gesture context"
    );
    assert_eq!(f.state()?["revision"], 1);
    assert_eq!(f.state()?["undo_depth"], 1);
    assert_eq!(f.view()?["epoch"], epoch);
    assert!(f.app.error.is_none(), "{:?}", f.app.error);
    Ok(())
}
fn set_slider(target: egui::accesskit::NodeId, value: f64) -> Event {
    Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
        action: egui::accesskit::Action::SetValue,
        target,
        data: Some(egui::accesskit::ActionData::NumericValue(value)),
    })
}

fn exact_camera_center(f: &Fixture, expected: [f64; 2]) -> TestResult {
    let view = f.view()?;
    for (field, expected) in ["center_x_pt", "center_y_pt"].into_iter().zip(expected) {
        let actual = view[field]
            .as_f64()
            .ok_or("explicit finite camera center")?;
        assert!(
            (actual - expected).abs() < 1e-10,
            "{field}: actual {actual:.15} versus declared f64 camera {expected:.15}"
        );
    }
    Ok(())
}

#[test]
fn axis_actions_resolve_at_their_raw_event_and_wheel_units_use_the_exact_fractional_scale()
-> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 2.0)?;
    f.warm();
    f.command("ui.view", json!({"action":"set_zoom","percent":175.5}))?;
    f.command(
        "ui.view",
        json!({"action":"pan","center_x_pt":200,"center_y_pt":300}),
    )?;
    let before = f.state()?;
    f.warm();
    let (horizontal, _) = f.node("Horizontal position", &[egui::accesskit::Role::Slider])?;
    let (vertical, _) = f.node("Vertical position", &[egui::accesskit::Role::Slider])?;
    let epoch = f.view()?["epoch"].as_u64().ok_or("view epoch")?;
    f.frame(vec![
        set_slider(vertical, 400.0),
        set_slider(horizontal, 240.0),
    ]);
    exact_camera_center(&f, [240.0, 400.0])?;
    assert_eq!(
        f.view()?["epoch"],
        epoch + 2,
        "two distinct axis actions retain raw event order"
    );
    f.warm();
    let (horizontal, _) = f.node("Horizontal position", &[egui::accesskit::Role::Slider])?;
    let (vertical, _) = f.node("Vertical position", &[egui::accesskit::Role::Slider])?;
    let mut events = vec![set_slider(vertical, 410.0)];
    events.extend(tap(Key::F5));
    events.push(set_slider(horizontal, 250.0));
    f.frame(events);
    exact_camera_center(&f, [250.0, 410.0])?;
    assert_eq!(f.view()?["zoom_percent"], 100.0);
    assert_eq!(
        f.view()?["epoch"],
        epoch + 5,
        "a view action between axis changes cannot replay an old other coordinate"
    );
    f.warm();
    let (horizontal, _) = f.node("Horizontal position", &[egui::accesskit::Role::Slider])?;
    f.frame(vec![
        set_slider(horizontal, 270.0),
        set_slider(horizontal, 250.0),
    ]);
    exact_camera_center(&f, [250.0, 410.0])?;
    assert_eq!(
        f.view()?["epoch"],
        epoch + 7,
        "same-axis A→B→A actions remain real ordered changes despite equal final widget value"
    );

    f.command("ui.view", json!({"action":"set_zoom","percent":175.5}))?;
    let output = f.warm();
    let area = canvas(&output)?;
    let p = area.center();
    let mut expected = [250.0, 410.0];
    let scale = 1.755_f64;
    let epoch = f.view()?["epoch"].as_u64().ok_or("view epoch")?;
    for (index, unit, delta, factor) in [
        (
            1,
            egui::MouseWheelUnit::Point,
            [20.0_f32, -40.0_f32],
            [1.0, 1.0],
        ),
        (
            2,
            egui::MouseWheelUnit::Line,
            [0.1_f32, -0.3_f32],
            [24.0, 24.0],
        ),
        (
            3,
            egui::MouseWheelUnit::Page,
            [0.03125_f32, -0.0625_f32],
            [
                f64::from(area.width().floor()) * 0.8,
                f64::from(area.height().floor()) * 0.8,
            ],
        ),
    ] {
        expected[0] -= f64::from(delta[0]) * factor[0] / scale;
        expected[1] -= f64::from(delta[1]) * factor[1] / scale;
        f.frame(vec![
            Event::PointerMoved(p),
            Event::MouseWheel {
                unit,
                delta: egui::vec2(delta[0], delta[1]),
                modifiers: Modifiers::NONE,
            },
        ]);
        exact_camera_center(&f, expected)?;
        assert_eq!(f.view()?["epoch"], epoch + index);
        assert_eq!(
            f.state()?,
            before,
            "view-only raw events do not edit source, page or history"
        );
    }
    assert!(f.app.error.is_none(), "{:?}", f.app.error);
    Ok(())
}
#[test]
fn open_preserves_stored_camera_but_new_pan_input_uses_the_displayed_clamped_center() -> TestResult
{
    for action in ["wheel", "horizontal", "vertical"] {
        let mut f = Fixture::new(1160.0, 800.0, 2.0)?;
        let input = f.directory.path("small-camera-source-化学.cdxml");
        let output = f.directory.path("small-camera-copy.cdxml");
        let mut writer = Engine::default();
        writer.dispatch(
            "page.set_size",
            json!({"page_id":1,"width_pt":100,"height_pt":100}),
        )?;
        writer.dispatch("document.save_cdxml", json!({"path":input}))?;
        let raw = fs::read(&input)?;
        f.warm();
        f.command("ui.view", json!({"action":"set_zoom","percent":200}))?;
        f.command(
            "ui.view",
            json!({"action":"pan","center_x_pt":400,"center_y_pt":450}),
        )?;
        let camera = f.view()?;
        f.command("document.open_cdxml", json!({"path":input}))?;
        assert_eq!(
            f.view()?,
            camera,
            "Open must preserve raw stored center and epoch without silently resetting window state"
        );
        let output_frame = f.warm();
        assert_eq!(
            f.view()?,
            camera,
            "layout may clamp only projection, not rewrite stored camera"
        );
        let before = f.state()?;
        assert_eq!(
            before["document"]["pages"][0]["size"],
            json!({"width_pt":100.0,"height_pt":100.0})
        );
        assert_eq!(before["undo_depth"], 0);
        assert_eq!(before["source"]["modified"], false);
        let stamp = f.app.engine.lock().map_err(|_| "engine")?.capture_stamp();
        let area = canvas(&output_frame)?;
        let p = projected(
            area,
            [100.0, 100.0],
            200.0,
            Some([100.0, 100.0]),
            [80.0, 80.0],
        );
        f.frame(vec![Event::PointerMoved(p)]);
        let expected = match action {
            "wheel" => {
                f.frame(vec![wheel([20.0, 0.0], Modifiers::NONE)]);
                [90.0, 100.0]
            }
            "horizontal" => {
                let (id, _) = f.node("Horizontal position", &[egui::accesskit::Role::Slider])?;
                f.frame(vec![set_slider(id, 90.0)]);
                [90.0, 100.0]
            }
            "vertical" => {
                let (id, _) = f.node("Vertical position", &[egui::accesskit::Role::Slider])?;
                f.frame(vec![set_slider(id, 80.0)]);
                [100.0, 80.0]
            }
            _ => return Err("unknown declared test action".into()),
        };
        exact_camera_center(&f, expected)?;
        assert_eq!(
            f.view()?["epoch"],
            camera["epoch"].as_u64().ok_or("camera epoch")? + 1
        );
        assert_eq!(
            f.state()?,
            before,
            "{action} changes only the camera, retaining the complete opened source/history"
        );
        assert!(
            f.app
                .engine
                .lock()
                .map_err(|_| "engine")?
                .capture_stamp_matches(&stamp)
        );
        assert!(f.app.error.is_none(), "{action}: {:?}", f.app.error);
        f.command("document.save_cdxml", json!({"path":output}))?;
        assert_eq!(fs::read(&input)?, raw);
        assert_eq!(
            fs::read(&output)?,
            raw,
            "view changes cannot regenerate the opened source"
        );
    }
    Ok(())
}
#[test]
fn pan_controls_follow_authored_y_direction_and_horizontal_keyboard_actions_reach_the_engine()
-> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    f.warm();
    f.command("ui.view", json!({"action":"actual"}))?;
    f.command(
        "ui.view",
        json!({"action":"pan","center_x_pt":250,"center_y_pt":400}),
    )?;
    let before = f.state()?;
    f.warm();
    let height = 297.0 * 72.0 / 25.4;
    let (vertical, bounds) = f.node("Vertical position", &[egui::accesskit::Role::Slider])?;
    let node = f
        .nodes
        .get(&vertical)
        .ok_or("current vertical slider node")?;
    assert_eq!(node.min_numeric_value(), Some(0.0));
    assert_eq!(
        node.max_numeric_value(),
        Some(height),
        "y-down orientation cannot be implemented by advertising reversed AX bounds"
    );
    let top = egui::pos2(bounds.center().x, bounds.top() + bounds.height() * 0.2);
    let bottom = egui::pos2(bounds.center().x, bounds.top() + bounds.height() * 0.8);
    f.frame(vec![
        Event::PointerMoved(bounds.center()),
        button(bounds.center(), true),
    ]);
    f.frame(vec![Event::PointerMoved(top)]);
    let top_y = f.view()?["center_y_pt"].as_f64().ok_or("top camera y")?;
    f.frame(vec![Event::PointerMoved(bottom)]);
    f.frame(vec![button(bottom, false)]);
    let bottom_y = f.view()?["center_y_pt"].as_f64().ok_or("bottom camera y")?;
    assert!(
        top_y < bottom_y,
        "moving down the actual vertical control must increase authored y: top {top_y}, bottom {bottom_y}"
    );
    assert!((0.0..=height).contains(&top_y) && (0.0..=height).contains(&bottom_y));
    assert_eq!(f.view()?["center_x_pt"], 250.0);
    f.warm();
    let node = f
        .nodes
        .get(&vertical)
        .ok_or("updated vertical slider node")?;
    assert_eq!(node.min_numeric_value(), Some(0.0));
    assert_eq!(node.max_numeric_value(), Some(height));
    assert_eq!(
        node.numeric_value(),
        Some(bottom_y),
        "the actual AX value is authored y, not a mirrored visual proxy"
    );

    let (horizontal, _) = f.node("Horizontal position", &[egui::accesskit::Role::Slider])?;
    f.frame(vec![Event::AccessKitActionRequest(
        egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::Focus,
            target: horizontal,
            data: None,
        },
    )]);
    assert_eq!(
        f.context.memory(|m| m.focused()).map(|id| id.value()),
        Some(horizontal.0)
    );
    let epoch = f.view()?["epoch"].as_u64().ok_or("keyboard view epoch")?;
    let start = f.view()?["center_x_pt"].as_f64().ok_or("start camera x")?;
    f.frame(tap(Key::ArrowRight));
    let right = f.view()?["center_x_pt"].as_f64().ok_or("right camera x")?;
    assert!(
        right > start,
        "focused ArrowRight must commit a horizontal pan through the registry"
    );
    assert_eq!(f.view()?["epoch"], epoch + 1);
    f.frame(tap(Key::ArrowLeft));
    let left = f.view()?["center_x_pt"].as_f64().ok_or("left camera x")?;
    assert!(
        left < right,
        "focused ArrowLeft must commit the opposite pan"
    );
    assert_eq!(f.view()?["epoch"], epoch + 2);
    assert_eq!(f.view()?["center_y_pt"], bottom_y);
    assert_eq!(
        f.state()?,
        before,
        "foreground pan controls cannot author canvas bonds or alter history/source"
    );
    assert!(f.app.error.is_none(), "{:?}", f.app.error);
    Ok(())
}

#[test]
fn numeric_zoom_accessibility_a_b_a_keeps_ordered_epochs_and_cancels_an_earlier_gesture()
-> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    f.warm();
    f.command("ui.view", json!({"action":"set_zoom","percent":125}))?;
    f.choose("Double bond")?;
    let output = f.warm();
    let before = f.state()?;
    let epoch = f.view()?["epoch"].as_u64().ok_or("Zoom epoch")?;
    let (zoom, _) = f.node(
        "Zoom",
        &[
            egui::accesskit::Role::SpinButton,
            egui::accesskit::Role::TextInput,
        ],
    )?;
    let area = canvas(&output)?;
    let page = [210.0 * 72.0 / 25.4, 297.0 * 72.0 / 25.4];
    let a = projected(area, page, 125.0, None, [220.0, 380.0]);
    let b = projected(area, page, 125.0, None, [300.0, 380.0]);
    f.frame(vec![
        Event::PointerMoved(a),
        button(a, true),
        Event::PointerMoved(b),
    ]);
    f.frame(vec![
        set_slider(zoom, 200.0),
        set_slider(zoom, 125.0),
        Event::PointerMoved(b),
        button(b, false),
    ]);
    assert_eq!(f.view()?["zoom_percent"], 125.0);
    assert_eq!(
        f.view()?["epoch"],
        epoch + 2,
        "two raw numeric Zoom requests cannot collapse into an equal final widget value"
    );
    assert_eq!(
        f.state()?,
        before,
        "Zoom A→B→A retires an armed gesture even when its final transform matches"
    );
    f.frame(vec![set_slider(zoom, 200.0)]);
    assert_eq!(f.view()?["zoom_percent"], 200.0);
    assert_eq!(f.view()?["epoch"], epoch + 3);
    f.frame(vec![set_slider(zoom, 200.0)]);
    assert_eq!(
        f.view()?["epoch"],
        epoch + 3,
        "a genuinely equal numeric request remains a no-op"
    );
    assert_eq!(f.state()?, before);
    assert!(f.app.error.is_none(), "{:?}", f.app.error);
    Ok(())
}
#[test]
fn numeric_zoom_recovers_from_an_invalid_large_page_camera_without_authoring_changes() -> TestResult
{
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    f.command(
        "page.set_size",
        json!({"page_id":1,"width_pt":100000,"height_pt":100000}),
    )?;
    f.command("ui.view", json!({"action":"set_zoom","percent":999}))?;
    let before = f.state()?;
    let page = PageId::new(1).ok_or("page id")?;
    assert!(
        f.app
            .engine
            .lock()
            .map_err(|_| "engine")?
            .page_view(800, 600, page)
            .is_err(),
        "the original large-page camera must actually exceed the bounded renderer profile"
    );
    f.warm();
    let stamp = f.app.engine.lock().map_err(|_| "engine")?.capture_stamp();
    let (zoom, _) = f.node(
        "Zoom",
        &[
            egui::accesskit::Role::SpinButton,
            egui::accesskit::Role::TextInput,
        ],
    )?;
    f.frame(vec![set_slider(zoom, 1.0)]);
    assert_eq!(
        f.view()?["zoom_percent"],
        1.0,
        "numeric recovery must still dispatch when the prior camera could not be constructed"
    );
    let output = f.warm();
    assert!(
        f.app
            .engine
            .lock()
            .map_err(|_| "engine")?
            .page_view(800, 600, page)
            .is_ok()
    );
    assert!(
        canvas(&output)?.area() > 0.0,
        "the recovered actual workspace must draw its canvas again"
    );
    assert_eq!(f.state()?, before);
    assert!(
        f.app
            .engine
            .lock()
            .map_err(|_| "engine")?
            .capture_stamp_matches(&stamp)
    );
    f.command("ui.view", json!({"action":"set_zoom","percent":999}))?;
    f.warm();
    f.frame(tap(Key::F6));
    assert_eq!(
        f.view()?["zoom_percent"],
        Value::Null,
        "the existing Fit recovery shortcut remains available too"
    );
    assert_eq!(f.state()?, before);
    Ok(())
}
#[test]
fn vertical_accessibility_steps_are_advertised_truthfully_and_change_only_the_camera() -> TestResult
{
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let path = f.directory.path("vertical-step-source-化学.cdxml");
    f.command("document.save_cdxml", json!({"path":path}))?;
    f.command("document.open_cdxml", json!({"path":path}))?;
    let raw = fs::read(&path)?;
    f.command("ui.view", json!({"action":"set_zoom","percent":150}))?;
    f.command(
        "ui.view",
        json!({"action":"pan","center_x_pt":250,"center_y_pt":400}),
    )?;
    f.warm();
    let before = f.state()?;
    let stamp = f.app.engine.lock().map_err(|_| "engine")?.capture_stamp();
    let epoch = f.view()?["epoch"].as_u64().ok_or("step camera epoch")?;
    let height = 297.0 * 72.0 / 25.4;
    let (vertical, _) = f.node("Vertical position", &[egui::accesskit::Role::Slider])?;
    let node = f.nodes.get(&vertical).ok_or("current vertical slider")?;
    assert!(
        node.supports_action(egui::accesskit::Action::Increment),
        "an interior vertical position must advertise Increment"
    );
    assert!(
        node.supports_action(egui::accesskit::Action::Decrement),
        "an interior vertical position must advertise Decrement"
    );
    assert_eq!(node.min_numeric_value(), Some(0.0));
    assert_eq!(node.max_numeric_value(), Some(height));
    assert_eq!(node.numeric_value(), Some(400.0));
    let mut previous = 400.0;
    for (index, action) in [
        (1, egui::accesskit::Action::Increment),
        (2, egui::accesskit::Action::Decrement),
    ] {
        f.frame(vec![Event::AccessKitActionRequest(
            egui::accesskit::ActionRequest {
                action,
                target: vertical,
                data: None,
            },
        )]);
        let y = f.view()?["center_y_pt"]
            .as_f64()
            .ok_or("stepped camera y")?;
        match action {
            egui::accesskit::Action::Increment => {
                assert!(y > previous, "Increment must increase authored camera y")
            }
            egui::accesskit::Action::Decrement => {
                assert!(y < previous, "Decrement must decrease authored camera y")
            }
            _ => return Err("unexpected declared step action".into()),
        }
        assert!((0.0..=height).contains(&y));
        assert_eq!(f.view()?["center_x_pt"], 250.0);
        assert_eq!(f.view()?["epoch"], epoch + index);
        f.warm();
        let node = f.nodes.get(&vertical).ok_or("updated vertical slider")?;
        assert_eq!(node.min_numeric_value(), Some(0.0));
        assert_eq!(node.max_numeric_value(), Some(height));
        assert_eq!(node.numeric_value(), Some(y));
        assert!(node.supports_action(egui::accesskit::Action::Increment));
        assert!(node.supports_action(egui::accesskit::Action::Decrement));
        assert_eq!(
            f.state()?,
            before,
            "AX pan steps preserve the whole opened source, file and history"
        );
        assert!(
            f.app
                .engine
                .lock()
                .map_err(|_| "engine")?
                .capture_stamp_matches(&stamp)
        );
        assert_eq!(fs::read(&path)?, raw);
        previous = y;
    }
    assert!(f.app.error.is_none(), "{:?}", f.app.error);
    Ok(())
}
fn independently_read_actual_saved_graph(path: &std::path::Path) -> TestResult<Value> {
    let python = std::env::var_os("BONDLINE_PYTHON").ok_or("pinned external interpreter")?;
    let mut child = Process::new(python)
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../xtask/oracles/cdxml_graph.py"
        ))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("actual saved-file oracle input")?
        .write_all(&fs::read(path)?)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(serde_json::from_slice(&output.stdout)?)
}

fn finish_actual_file_workflow(f: &mut Fixture) -> TestResult {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        f.frame(vec![]);
        if !f.app.files.is_busy()
            && !f.app.executor.has_active_work()
            && f.state()?["ui_file"]["request"].is_null()
        {
            return Ok(());
        }
        if std::time::Instant::now() >= deadline {
            return Err("actual FileWorkflow job did not finish within ten seconds".into());
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

#[test]
fn actual_frame_undo_then_save_in_one_packet_saves_the_undone_source_and_history() -> TestResult {
    // The separated sequence is a positive control using exactly the same real UI/IO.
    for batched in [false, true] {
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        let path = f.directory.path("ordered-history-save-化学.cdxml");
        f.command("bond.create",json!({"page_id":1,"begin_x_pt":220,"begin_y_pt":380,"end_x_pt":300,"end_y_pt":380,"order":"single"}))?;
        f.command("document.save_cdxml", json!({"path":path}))?;
        f.command("document.open_cdxml", json!({"path":path}))?;
        f.command("ui.view", json!({"action":"set_zoom","percent":150}))?;
        f.choose("Single bond")?;
        let output = f.warm();
        let original = f.state()?;
        let raw = fs::read(&path)?;
        let view = f.view()?;
        let revision = original["revision"].as_u64().ok_or("source revision")?;
        assert_eq!(original["undo_depth"], 0);
        assert_eq!(original["file"]["dirty"], false);
        let page = [210.0 * 72.0 / 25.4, 297.0 * 72.0 / 25.4];
        let p = projected(canvas(&output)?, page, 150.0, None, [260.0, 380.0]);
        f.frame(vec![
            Event::PointerMoved(p),
            button(p, true),
            button(p, false),
        ]);
        let edited = f.state()?;
        assert_eq!(
            edited["document"]["pages"][0]["fragments"][0]["bonds"][0]["order"],
            "double"
        );
        assert_eq!(edited["revision"], revision + 1);
        assert_eq!(edited["undo_depth"], 1);
        assert_eq!(edited["file"]["dirty"], true);
        let undo = vec![
            key(Key::Z, true, Modifiers::COMMAND),
            key(Key::Z, false, Modifiers::COMMAND),
        ];
        let save = vec![
            key(Key::S, true, Modifiers::COMMAND),
            key(Key::S, false, Modifiers::COMMAND),
        ];
        if batched {
            let mut events = undo;
            events.extend(save);
            f.frame(events);
        } else {
            f.frame(undo);
            f.frame(save);
        }
        finish_actual_file_workflow(&mut f)?;
        let actual = f.state()?;
        let saved_graph = independently_read_actual_saved_graph(&path)?;
        assert_eq!(
            saved_graph, original["document"]["pages"][0]["fragments"],
            "batched={batched}: actual saved file must contain the undone literal Single graph; revision={}, undo={}, redo={}",
            actual["revision"], actual["undo_depth"], actual["redo_depth"]
        );
        assert_eq!(actual["document"], original["document"]);
        assert_eq!(actual["revision"], revision + 2);
        assert_eq!(actual["undo_depth"], 0);
        assert_eq!(actual["redo_depth"], 1);
        assert_eq!(actual["file"]["dirty"], false);
        assert_eq!(actual["source"]["modified"], false);
        assert_eq!(
            fs::read(&path)?,
            raw,
            "Undo+Save retains the exact associated native source bytes"
        );
        assert_eq!(f.view()?, view);
        assert!(
            f.app.error.is_none(),
            "batched={batched}: {:?}",
            f.app.error
        );
    }
    Ok(())
}

#[path = "sdf_inspector_tests.rs"]
mod sdf_inspector;

#[path = "sdf_editor_tests.rs"]
mod sdf_editor;

#[path = "sdf_view_tests.rs"]
mod sdf_view;

#[path = "fit_zoom_tests.rs"]
mod fit_zoom;

#[path = "smiles_record_tests.rs"]
mod smiles_record;

#[path = "sdf_record_remove_tests.rs"]
mod sdf_record_remove_tests;
