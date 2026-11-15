//! Public-guide order conversions through actual App frames; no native execution.
use super::*;
use egui::{Event, Key, Modifiers, PointerButton, Pos2};
use serde_json::{Value, json};
use std::{collections::HashMap, fs, process::Command as Process};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(
    dead_code,
    clippy::duplicate_mod,
    reason = "reuse original atomic temporary-directory helper without editing frozen tests"
)]
mod support;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

struct Fixture {
    app: BondlineApp,
    context: egui::Context,
    directory: support::FileTestDirectory,
    nodes: HashMap<egui::accesskit::NodeId, egui::accesskit::Node>,
    dpi: f32,
    origin: Pos2,
    cover: Option<egui::Rect>,
    editor: bool,
    text: String,
}

impl Fixture {
    fn new(chain: bool) -> TestResult<Self> {
        let directory = support::FileTestDirectory::new()?;
        let mut engine = Engine::default();
        engine.set_ui_file_host(true);
        engine.dispatch(
            "page.set_size",
            json!({"page_id":1,"width_pt":128,"height_pt":128}),
        )?;
        if chain {
            engine.dispatch(
                "bond.create",
                json!({"page_id":1,"begin_x_pt":25,"begin_y_pt":25,"end_x_pt":25,"end_y_pt":75}),
            )?;
            engine.dispatch("bond.extend", json!({"expected_revision":engine.state().revision,"page_id":1,"fragment_id":1,"begin_atom_id":2,"end_x_pt":100,"end_y_pt":75,"hydrogen_policy":"rectify_terminal_carbon"}))?;
        } else {
            for y in [35, 90] {
                engine.dispatch(
                    "bond.create",
                    json!({"page_id":1,"begin_x_pt":20,"begin_y_pt":y,"end_x_pt":100,"end_y_pt":y}),
                )?;
            }
        }
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
        context.enable_accesskit();
        Ok(Self {
            app,
            context,
            directory,
            nodes: HashMap::new(),
            dpi: 1.0,
            origin: egui::pos2(100.0, 100.0),
            cover: None,
            editor: false,
            text: String::new(),
        })
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
        let viewport = input.viewports.entry(egui::ViewportId::ROOT).or_default();
        viewport.native_pixels_per_point = Some(self.dpi);
        viewport.inner_rect = Some(egui::Rect::from_min_size(
            self.origin,
            egui::vec2(1600.0, 1400.0),
        ));
        let context = self.context.clone();
        let output = context.run(input, |ctx| {
            if let Some(cover) = self.cover {
                egui::Area::new(egui::Id::new("order-foreground"))
                    .order(egui::Order::Foreground)
                    .fixed_pos(cover.min)
                    .show(ctx, |ui| {
                        ui.allocate_exact_size(cover.size(), egui::Sense::hover());
                    });
            }
            if self.editor {
                egui::Area::new(egui::Id::new("order-text-area"))
                    .order(egui::Order::Foreground)
                    .fixed_pos(egui::pos2(400.0, 5.0))
                    .show(ctx, |ui| {
                        egui::TextEdit::singleline(&mut self.text)
                            .id(egui::Id::new("order-editor"))
                            .show(ui)
                            .response
                            .request_focus();
                    });
            }
            self.app.update_workspace(ctx);
        });
        if let Some(update) = &output.platform_output.accesskit_update {
            self.nodes.extend(update.nodes.iter().cloned());
        }
        output
    }

    fn warm(&mut self) -> egui::FullOutput {
        self.frame(vec![]);
        self.frame(vec![])
    }

    fn state(&self) -> TestResult<Value> {
        Ok(serde_json::to_value(
            self.app.engine.lock().map_err(|_| "engine")?.state(),
        )?)
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

    fn node(&self, label: &str) -> TestResult<(egui::accesskit::NodeId, egui::Rect)> {
        self.nodes
            .iter()
            .find_map(|(id, node)| {
                (node.name() == Some(label)
                    && node.role() == egui::accesskit::Role::Button
                    && node.supports_action(egui::accesskit::Action::Focus))
                .then(|| {
                    node.bounds().map(|r| {
                        (
                            *id,
                            egui::Rect::from_min_max(
                                egui::pos2(r.x0 as f32, r.y0 as f32),
                                egui::pos2(r.x1 as f32, r.y1 as f32),
                            ),
                        )
                    })
                })?
            })
            .ok_or_else(|| format!("actual accessible {label} button missing").into())
    }

    fn choose(&mut self, label: &str) -> TestResult {
        self.warm();
        let (_, bounds) = self.node(label)?;
        self.frame(vec![
            Event::PointerMoved(bounds.center()),
            button(bounds.center(), true),
        ]);
        self.frame(vec![button(bounds.center(), false)]);
        Ok(())
    }

    fn mode(&self) -> TestResult<(BondDrawOrder, u64)> {
        let tool = self.app.engine.lock().map_err(|_| "engine")?.bond_tool();
        Ok((tool.order, tool.epoch))
    }

    fn point(&self, output: &egui::FullOutput, p: [f64; 2]) -> TestResult<Pos2> {
        let canvas = canvas(output)?;
        let engine = self.app.engine.lock().map_err(|_| "engine")?;
        let size = engine.document()?.pages().first().ok_or("page")?.size();
        let viewport = bondline_render::PageViewport::new(
            canvas.width().floor() as u32,
            canvas.height().floor() as u32,
            size,
        )?;
        let [x, y] = viewport.canvas_position(Point2::new(p[0], p[1])?)?;
        Ok(canvas.min + egui::vec2(x, y))
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

fn click(pos: Pos2) -> Vec<Event> {
    vec![
        Event::PointerMoved(pos),
        button(pos, true),
        button(pos, false),
    ]
}

fn key(key: Key, pressed: bool) -> Event {
    Event::Key {
        key,
        physical_key: Some(key),
        pressed,
        repeat: false,
        modifiers: Modifiers::NONE,
    }
}

fn tap(key_code: Key) -> Vec<Event> {
    vec![key(key_code, true), key(key_code, false)]
}

fn canvas(output: &egui::FullOutput) -> TestResult<egui::Rect> {
    output
        .shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            egui::Shape::Rect(rect) if rect.fill == egui::Color32::from_gray(196) => {
                Some(shape.clip_rect)
            }
            _ => None,
        })
        .ok_or_else(|| "actual paper canvas missing".into())
}

fn assert_row_components(output: &egui::FullOutput, y: f32, count: usize) -> TestResult {
    let canvas = canvas(output)?;
    let paths = output
        .shapes
        .iter()
        .filter(|shape| shape.clip_rect == canvas)
        .filter_map(|shape| match &shape.shape {
            egui::Shape::Path(path)
                if path.closed && path.fill == egui::Color32::BLACK && path.points.len() == 4 =>
            {
                let rect = egui::Rect::from_points(&path.points);
                (rect.width() > rect.height() * 2.0 && (rect.center().y - y).abs() < 40.0)
                    .then_some(rect)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        paths.len(),
        count,
        "every new visible bond component must be refreshed"
    );
    Ok(())
}

fn expected_chain(mut document: Value, order: &str, weight: u16) -> Value {
    document["pages"][0]["fragments"][0]["bonds"][1]["order"] = json!(order);
    document["pages"][0]["fragments"][0]["atoms"][1]["explicit_hydrogens"] = json!(3 - weight);
    document["pages"][0]["fragments"][0]["atoms"][2]["explicit_hydrogens"] = json!(4 - weight);
    document
}

fn independent_saved(path: &std::path::Path, expected: &Value, weight: u16) -> TestResult {
    use std::process::Stdio;
    let python =
        std::env::var_os("BONDLINE_PYTHON").ok_or("pinned external interpreter required")?;
    let mut raw = Process::new(&python)
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../xtask/oracles/cdxml_graph.py"
        ))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    raw.stdin
        .take()
        .ok_or("Expat input")?
        .write_all(&fs::read(path)?)?;
    let output = raw.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let fragments: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(
        fragments, expected["pages"][0]["fragments"],
        "unchanged Expat/Decimal consumer must recover all actual saved IDs/positions/H/orders"
    );
    let output=Process::new(python).args(["-B","-c",r#"
import sys,json
from pathlib import Path
from rdkit import Chem,rdBase
from rdkit.Chem import rdMolDescriptors
assert rdBase.rdkitVersion=='2026.09.2'
ms=Chem.MolsFromCDXML(Path(sys.argv[1]).read_text(encoding='utf-8')); assert len(ms)==1
m=ms[0]; w=int(sys.argv[2]); assert m.GetNumAtoms()==3 and m.GetNumBonds()==2
assert all(a.GetAtomicNum()==6 and a.GetFormalCharge()==0 and a.GetIsotope()==0 for a in m.GetAtoms())
assert [b.GetBondTypeAsDouble() for b in m.GetBonds()]==[1,w]
assert [a.GetTotalNumHs() for a in m.GetAtoms()]==[3,3-w,4-w]
assert rdMolDescriptors.CalcMolFormula(m)=={2:'C3H6',3:'C3H4'}[w]
print(Chem.MolToSmiles(m))
"#]).arg(path).arg(weight.to_string()).output()?;
    assert!(
        output.status.success(),
        "actual saved CDXML/RDKit: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

#[test]
fn palette_click_converts_existing_single_without_new_ids_and_survives_history_save_reopen()
-> TestResult {
    for (tool, order, weight) in [
        ("Single bond", "double", 2_u16),
        ("Double bond", "double", 2),
        ("Triple bond", "triple", 3),
    ] {
        let mut f = Fixture::new(true)?;
        if tool == "Double bond" {
            let original = f.directory.path("original-native.cdxml");
            f.command("document.save_cdxml", json!({"path":original}))?;
            f.command("document.open_cdxml", json!({"path":original}))?;
        }
        f.choose(tool)?;
        let mode = f.mode()?;
        let output = f.warm();
        let point = f.point(&output, [62.5, 75.0])?;
        let before = f.state()?;
        let expected = expected_chain(before["document"].clone(), order, weight);
        let output = f.frame(click(point));
        assert_eq!(
            f.state()?["document"],
            expected,
            "public tool click must convert existing Bond2 rather than ignore the click, reset it or add a fragment"
        );
        assert!(f.app.error.is_none(), "{:?}", f.app.error);
        assert_eq!(
            f.state()?["revision"].as_u64(),
            before["revision"].as_u64().map(|r| r + 1)
        );
        assert_eq!(
            f.state()?["undo_depth"].as_u64(),
            before["undo_depth"].as_u64().map(|d| d + 1)
        );
        assert_eq!(
            f.mode()?,
            mode,
            "conversion does not change selected tool or epoch"
        );
        assert_row_components(&output, point.y, usize::from(weight))?;
        let xml = f.directory.path(&format!("converted-{order}-λ.cdxml"));
        f.command("document.save_cdxml", json!({"path":xml}))?;
        independent_saved(&xml, &expected, weight)?;
        f.command("history.undo", json!({}))?;
        assert_eq!(f.state()?["document"], before["document"]);
        assert_eq!(f.state()?["file"]["dirty"], true);
        f.command("history.redo", json!({}))?;
        assert_eq!(f.state()?["document"], expected);
        assert_eq!(f.state()?["file"]["dirty"], false);
        f.command("document.open_cdxml", json!({"path":xml}))?;
        assert_eq!(f.state()?["document"], expected);
        assert_eq!(f.mode()?, mode);
        let output = f.warm();
        assert_row_components(
            &output,
            f.point(&output, [62.5, 75.0])?.y,
            usize::from(weight),
        )?;
    }
    Ok(())
}

#[test]
fn absolute_hotspot_keys_use_event_order_and_allow_real_toolbar_focus_without_changing_mode()
-> TestResult {
    let mut f = Fixture::new(false)?;
    f.choose("Triple bond")?;
    let (id, _) = f.node("Triple bond")?;
    let output = f.frame(vec![Event::AccessKitActionRequest(
        egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::Focus,
            target: id,
            data: None,
        },
    )]);
    assert_eq!(
        output
            .platform_output
            .accesskit_update
            .as_ref()
            .ok_or("actual focus update")?
            .focus,
        id
    );
    assert!(
        f.context.wants_keyboard_input(),
        "focused actual palette button is not text editing"
    );
    let a = f.point(&output, [60.0, 35.0])?;
    let b = f.point(&output, [60.0, 90.0])?;
    f.frame(vec![Event::PointerMoved(a)]);
    assert!(
        f.context.wants_keyboard_input(),
        "the actual toolbar button remains focused over the canvas"
    );
    let mode = f.mode()?;
    let before = f.state()?;
    let output = f.frame(vec![
        key(Key::Num2, true),
        key(Key::Num2, false),
        Event::PointerMoved(b),
        key(Key::Num3, true),
        key(Key::Num3, false),
    ]);
    let mut expected = before["document"].clone();
    for (index, order, h) in [(0, "double", 2), (1, "triple", 1)] {
        expected["pages"][0]["fragments"][index]["bonds"][0]["order"] = json!(order);
        for atom in expected["pages"][0]["fragments"][index]["atoms"]
            .as_array_mut()
            .ok_or("pair")?
        {
            atom["explicit_hydrogens"] = json!(h);
        }
    }
    assert_eq!(
        f.state()?["document"],
        expected,
        "Key2 before MoveB must affect A; later Key3 affects B using freshly reacquired current graph"
    );
    assert_eq!(
        f.state()?["undo_depth"].as_u64(),
        before["undo_depth"].as_u64().map(|d| d + 2)
    );
    assert_eq!(f.mode()?, mode);
    assert_row_components(&output, a.y, 2)?;
    assert_row_components(&output, b.y, 3)?;
    // Hit actual new Double ink instead of its intentionally white center gap.
    let viewport = bondline_render::PageViewport::new(
        canvas(&output)?.width().floor() as u32,
        canvas(&output)?.height().floor() as u32,
        bondline_model::PageSize::new(128.0, 128.0)?,
    )?;
    let ink = a - egui::vec2(0.0, 1.5 * viewport.stroke_width());
    f.frame(vec![Event::PointerMoved(ink)]);
    assert!(
        matches!(f.app.drawing.hover_target(),Some(bondline_render::HitTarget::Bond{bond_id,..})if bond_id.get()==1)
    );
    f.frame(tap(Key::Num1));
    expected["pages"][0]["fragments"][0] = before["document"]["pages"][0]["fragments"][0].clone();
    assert_eq!(
        f.state()?["document"],
        expected,
        "hotspot1 is absolute Single, not click cycling or tool selection"
    );
    assert_eq!(f.mode()?, mode);
    f.frame(vec![Event::PointerMoved(a)]);
    assert!(
        matches!(f.app.drawing.hover_target(),Some(bondline_render::HitTarget::Bond{bond_id,..})if bond_id.get()==1)
    );
    let before = f.state()?;
    f.frame(tap(Key::Num1));
    assert_eq!(
        f.state()?,
        before,
        "same-order absolute setter must be a validated no-op"
    );
    Ok(())
}

impl Fixture {
    fn ink_point(&self, output: &egui::FullOutput, p: [f64; 2], weight: u16) -> TestResult<Pos2> {
        let mut point = self.point(output, p)?;
        let rect = canvas(output)?;
        let engine = self.app.engine.lock().map_err(|_| "engine")?;
        let size = engine.document()?.pages().first().ok_or("page")?.size();
        let viewport = bondline_render::PageViewport::new(
            rect.width().floor() as u32,
            rect.height().floor() as u32,
            size,
        )?;
        point.y -= match weight {
            1 => 0.0,
            2 => 1.5 * viewport.stroke_width(),
            3 => 3.0 * viewport.stroke_width(),
            _ => return Err("test order".into()),
        };
        Ok(point)
    }
}

#[test]
fn hotspot_guards_require_fresh_uncovered_context_and_plain_keys_before_positive_rearm()
-> TestResult {
    for guard in [
        "no_fresh",
        "escape",
        "focus",
        "gone",
        "tool_epoch",
        "revision",
        "dpi",
        "origin",
        "cover",
        "atom",
        "non_plain",
        "close",
        "text",
        "held",
    ] {
        let mut f = Fixture::new(false)?;
        let output = f.warm();
        let point = f.point(&output, [60.0, 35.0])?;
        if guard != "no_fresh" {
            f.frame(vec![Event::PointerMoved(point)]);
        }
        let mut events = tap(Key::Num2);
        match guard {
            "no_fresh" => {}
            "escape" => {
                f.frame(tap(Key::Escape));
            }
            "focus" => {
                f.frame(vec![
                    Event::WindowFocused(false),
                    Event::WindowFocused(true),
                ]);
            }
            "gone" => {
                f.frame(vec![Event::PointerGone]);
            }
            "tool_epoch" => {
                f.command("ui.bond_tool", json!({"order":"double"}))?;
                f.command("ui.bond_tool", json!({"order":"single"}))?;
            }
            "revision" => {
                f.command(
                    "page.set_size",
                    json!({"page_id":1,"width_pt":124,"height_pt":128}),
                )?;
            }
            "dpi" => {
                f.dpi = 2.0;
            }
            "origin" => {
                f.origin += egui::vec2(11.0, 7.0);
            }
            "cover" => {
                f.cover = Some(egui::Rect::from_center_size(point, egui::vec2(40.0, 40.0)));
                f.warm();
            }
            "atom" => {
                f.frame(vec![Event::PointerMoved(f.point(&output, [20.0, 35.0])?)]);
            }
            "non_plain" => {
                let setup_before = f.state()?;
                let setup_mode = f.mode()?;
                let off_hotspot = f.point(&output, [10.0, 10.0])?;
                f.frame(vec![Event::PointerMoved(off_hotspot), key(Key::Num2, true)]);
                f.frame(vec![Event::PointerMoved(point)]);
                assert_eq!(
                    f.state()?,
                    setup_before,
                    "off-hotspot held-key setup must not edit"
                );
                assert_eq!(
                    f.mode()?,
                    setup_mode,
                    "off-hotspot number is not global tool selection"
                );
                assert!(f.context.input(|input| input.key_down(Key::Num2)));
                events = vec![
                    Event::Key {
                        key: Key::Num2,
                        physical_key: Some(Key::Num2),
                        pressed: true,
                        repeat: false, // Pinned egui infers true from the actual held key.
                        modifiers: Modifiers::NONE,
                    },
                    key(Key::Num2, false),
                    key(Key::Num2, false), // Also preserve an unmatched key-up negative.
                ];
                for modifiers in [
                    Modifiers::CTRL,
                    Modifiers::ALT,
                    Modifiers::SHIFT,
                    Modifiers::MAC_CMD,
                ] {
                    events.extend([
                        Event::Key {
                            key: Key::Num2,
                            physical_key: Some(Key::Num2),
                            pressed: true,
                            repeat: false,
                            modifiers,
                        },
                        Event::Key {
                            key: Key::Num2,
                            physical_key: Some(Key::Num2),
                            pressed: false,
                            repeat: false,
                            modifiers,
                        },
                    ]);
                }
            }
            "close" => {
                f.command(
                    "document.close",
                    json!({"expected_revision":f.state()?["revision"],"action":"request"}),
                )?;
            }
            "text" => {
                f.editor = true;
                f.warm();
                events.push(Event::Text("2".to_owned()));
            }
            "held" => {
                let anchor = f.point(&output, [100.0, 35.0])?;
                let other = f.point(&output, [60.0, 90.0])?;
                f.frame(vec![
                    Event::PointerMoved(anchor),
                    button(anchor, true),
                    Event::PointerMoved(other),
                ]);
                assert!(f.app.drawing.gesture.is_some());
            }
            _ => return Err("test guard".into()),
        }
        let before = f.state()?;
        let mode = f.mode()?;
        f.frame(events);
        if guard == "non_plain" {
            let repeats = f.context.input(|input| {
                input
                    .raw
                    .events
                    .iter()
                    .filter_map(|event| match event {
                        Event::Key {
                            key: Key::Num2,
                            pressed: true,
                            repeat,
                            ..
                        } => Some(*repeat),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
            });
            assert_eq!(
                repeats,
                [true, false, false, false, false],
                "actual normalized repeat precedes the four original modified presses"
            );
        }
        assert_eq!(
            f.state()?,
            before,
            "guard {guard} must prevent a numeric edit, history item or baseline change"
        );
        assert_eq!(
            f.mode()?,
            mode,
            "numeric input must not change tool mode under {guard}"
        );
        if guard == "text" {
            assert!(
                f.text.contains('2'),
                "real TextEdit must receive the character instead of the canvas"
            );
        }
        if guard == "close" {
            f.command(
                "document.close",
                json!({"expected_revision":f.state()?["revision"],"action":"cancel"}),
            )?;
        }
        if guard == "held" {
            f.frame(tap(Key::Escape));
            f.frame(vec![button(point, false)]);
        }
        f.cover = None;
        f.editor = false;
        f.context
            .memory_mut(|memory| memory.surrender_focus(egui::Id::new("order-editor")));
        f.dpi = 1.0;
        let output = f.warm();
        let point = f.point(&output, [60.0, 35.0])?;
        let before = f.state()?;
        let mut events = vec![Event::PointerMoved(point)];
        events.extend(tap(Key::Num2));
        f.frame(events);
        let mut expected = before["document"].clone();
        expected["pages"][0]["fragments"][0]["bonds"][0]["order"] = json!("double");
        for atom in expected["pages"][0]["fragments"][0]["atoms"]
            .as_array_mut()
            .ok_or("pair")?
        {
            atom["explicit_hydrogens"] = json!(2);
        }
        assert_eq!(
            f.state()?["document"],
            expected,
            "fresh ordered input must rearm after guard {guard}; do not satisfy negative cases by disabling all hotspot keys"
        );
    }
    Ok(())
}

#[test]
fn unsupported_click_combinations_show_diagnostics_and_mol_backing_keeps_save_authority()
-> TestResult {
    for (old, weight, label, alignment) in [
        ("double", 2, "Single bond", true),
        ("double", 2, "Double bond", true),
        ("triple", 3, "Single bond", false),
        ("triple", 3, "Double bond", false),
        ("double", 2, "Triple bond", false),
        ("triple", 3, "Triple bond", false),
    ] {
        let mut f = Fixture::new(false)?;
        f.command("history.undo", json!({}))?;
        f.command("history.undo", json!({}))?;
        f.command("bond.create",json!({"page_id":1,"order":old,"begin_x_pt":20,"begin_y_pt":35,"end_x_pt":100,"end_y_pt":35}))?;
        f.choose(label)?;
        let output = f.warm();
        let point = f.ink_point(&output, [60.0, 35.0], weight)?;
        let before = f.state()?;
        let output = f.frame(click(point));
        assert_eq!(
            f.state()?,
            before,
            "unsupported {old} + {label} must not reset/cycle order or commit history"
        );
        let error = f
            .app
            .error
            .as_ref()
            .ok_or("unsupported click must show a diagnostic, not a successful no-op")?;
        if alignment {
            assert!(
                error.to_lowercase().contains("align"),
                "alignment gap must be explicit: {error}"
            );
        }
        assert!(output.shapes.iter().any(|shape|matches!(&shape.shape,egui::Shape::Text(text) if text.galley.text().contains(error) && shape.clip_rect.contains_rect(shape.shape.visual_bounding_rect()))),"unsupported flow diagnostic must actually be visible");
    }
    let mut f = Fixture::new(false)?;
    let original = include_bytes!("../../io/tests/fixtures/mol/ethane-v2000.mol");
    let mol = f.directory.path("original-source.mol");
    fs::write(&mol, original)?;
    f.command(
        "document.open_mol",
        json!({"path":mol,"discard_unsaved":true}),
    )?;
    f.choose("Triple bond")?;
    let output = f.warm();
    let point = f.point(&output, [90.0, 72.0])?;
    let before = f.state()?;
    let owner = f.app.engine.lock().map_err(|_| "engine")?.content().clone();
    f.frame(click(point));
    assert_eq!(
        f.state()?,
        before,
        "faithful MOL display does not authorize a source edit"
    );
    assert!(f.app.error.is_some());
    let mut events = vec![Event::PointerMoved(point)];
    events.extend(tap(Key::Num3));
    f.frame(events);
    assert_eq!(f.state()?, before);
    assert_eq!(
        f.app.engine.lock().map_err(|_| "engine")?.content(),
        &owner,
        "retain complete original owner, not a projection"
    );
    let copy = f.directory.path("source-copy.mol");
    f.command("document.save_mol", json!({"path":copy}))?;
    assert_eq!(fs::read(copy)?, original);
    Ok(())
}

#[test]
fn single_tool_endpoint_drag_across_double_converts_only_that_bond_in_both_directions() -> TestResult
{
    for reverse in [false, true] {
        let mut f = Fixture::new(true)?;
        f.command("history.undo", json!({}))?;
        f.command("bond.extend",json!({"expected_revision":f.state()?["revision"],"page_id":1,"fragment_id":1,"begin_atom_id":2,"end_x_pt":100,"end_y_pt":75,"order":"double","hydrogen_policy":"rectify_terminal_carbon"}))?;
        f.choose("Single bond")?;
        let output = f.warm();
        let mut a = f.point(&output, [25.0, 75.0])?;
        let mut b = f.point(&output, [100.0, 75.0])?;
        if reverse {
            std::mem::swap(&mut a, &mut b);
        }
        let before = f.state()?;
        let expected = expected_chain(before["document"].clone(), "triple", 3);
        let output = f.frame(vec![
            Event::PointerMoved(a),
            button(a, true),
            Event::PointerMoved(b),
            button(b, false),
        ]);
        assert_eq!(
            f.state()?["document"],
            expected,
            "documented Double endpoint drag must convert the existing edge, not join nodes/add an edge; reverse={reverse}"
        );
        assert!(f.app.error.is_none(), "{:?}", f.app.error);
        assert_eq!(
            f.state()?["undo_depth"].as_u64(),
            before["undo_depth"].as_u64().map(|d| d + 1)
        );
        assert_row_components(&output, a.y, 3)?;
        f.command("history.undo", json!({}))?;
        assert_eq!(f.state()?["document"], before["document"]);
        f.command("history.redo", json!({}))?;
        assert_eq!(f.state()?["document"], expected);
    }
    Ok(())
}

#[test]
fn stationary_hotspot_absolute_keys_keep_the_same_bond_when_edited_ink_moves_away() -> TestResult {
    for (outer_triple, one_packet) in [(false, false), (false, true), (true, false), (true, true)] {
        let mut f = Fixture::new(false)?;
        if outer_triple {
            f.command("history.undo", json!({}))?;
            f.command("history.undo", json!({}))?;
            f.command("bond.create",json!({"page_id":1,"order":"triple","begin_x_pt":20,"begin_y_pt":35,"end_x_pt":100,"end_y_pt":35}))?;
        }
        f.choose("Triple bond")?;
        let mode = f.mode()?;
        let output = f.warm();
        let axis = f.point(&output, [60.0, 35.0])?;
        let pointer = f.ink_point(&output, [60.0, 35.0], if outer_triple { 3 } else { 1 })?;
        let rect = canvas(&output)?;
        let width = bondline_render::PageViewport::new(
            rect.width().floor() as u32,
            rect.height().floor() as u32,
            bondline_model::PageSize::new(128.0, 128.0)?,
        )?
        .stroke_width();
        assert!(
            width > 5.0,
            "fixture's Double center gap must exceed the five-unit hit tolerance"
        );
        f.frame(vec![Event::PointerMoved(pointer)]);
        let before = f.state()?;
        let bond_id = before["document"]["pages"][0]["fragments"][0]["bonds"][0]["id"]
            .as_u64()
            .ok_or("bond ID")?;
        assert!(
            matches!(f.app.drawing.hover_target(),Some(bondline_render::HitTarget::Bond{bond_id:id,..})if u64::from(id.get())==bond_id)
        );
        let revision = before["revision"].as_u64().ok_or("revision")?;
        let undo = before["undo_depth"].as_u64().ok_or("undo")?;
        let sequence: Vec<(Key, &str, u16)> = if outer_triple {
            vec![(Key::Num1, "single", 1), (Key::Num2, "double", 2)]
        } else {
            vec![
                (Key::Num2, "double", 2),
                (Key::Num3, "triple", 3),
                (Key::Num1, "single", 1),
            ]
        };
        let mut expected = before["document"].clone();
        let mut completed = 0_u64;
        if one_packet {
            let events = sequence.iter().flat_map(|(key, _, _)| tap(*key)).collect();
            // No PointerMoved/press is allowed after the single initial hotspot entry.
            let output = f.frame(events);
            for (_, order, weight) in &sequence {
                expected["pages"][0]["fragments"][0]["bonds"][0]["order"] = json!(order);
                for atom in expected["pages"][0]["fragments"][0]["atoms"]
                    .as_array_mut()
                    .ok_or("pair")?
                {
                    atom["explicit_hydrogens"] = json!(4 - weight);
                }
                completed += 1;
            }
            let final_weight = sequence.last().ok_or("sequence")?.2;
            assert_eq!(
                f.state()?["document"],
                expected,
                "same-packet stationary hotspot must retain logical bond identity; outerTriple={outer_triple}"
            );
            assert_row_components(&output, axis.y, usize::from(final_weight))?;
        } else {
            for (key, order, weight) in sequence {
                let output = f.frame(tap(key));
                expected["pages"][0]["fragments"][0]["bonds"][0]["order"] = json!(order);
                for atom in expected["pages"][0]["fragments"][0]["atoms"]
                    .as_array_mut()
                    .ok_or("pair")?
                {
                    atom["explicit_hydrogens"] = json!(4 - weight);
                }
                completed += 1;
                assert_eq!(
                    f.state()?["document"],
                    expected,
                    "stationary key must keep the original logical bond even after its ink moves; key={key:?}, outerTriple={outer_triple}"
                );
                assert_eq!(f.state()?["revision"], json!(revision + completed));
                assert_eq!(f.state()?["undo_depth"], json!(undo + completed));
                assert_row_components(&output, axis.y, usize::from(weight))?;
            }
        }
        assert_eq!(f.state()?["revision"], json!(revision + completed));
        assert_eq!(
            f.state()?["undo_depth"],
            json!(undo + completed),
            "exactly one history item per actual order change"
        );
        assert_eq!(
            f.mode()?,
            mode,
            "absolute hotspot edits never select a different tool/epoch"
        );
        assert!(f.app.error.is_none(), "{:?}", f.app.error);
    }
    Ok(())
}
