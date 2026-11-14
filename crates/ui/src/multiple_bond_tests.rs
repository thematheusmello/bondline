//! Independent real-frame tool/gesture and centered-bundle acceptance; no native proof.
use super::*;
use bondline_model::{BondOrder, PageSize};
use egui::{Event, Modifiers, PointerButton, Pos2};
use serde_json::{Value, json};
use std::{collections::HashMap, fs, process::Command as Process};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(
    dead_code,
    clippy::duplicate_mod,
    reason = "reuse unchanged private test directory helper without editing sibling tests"
)]
mod support;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const TOOLS: [(&str, &str); 3] = [
    ("single", "Single bond"),
    ("double", "Double bond"),
    ("triple", "Triple bond"),
];
const EPS: f32 = 0.001;
const HOVER: egui::Color32 = egui::Color32::from_rgb(40, 120, 180);

struct Fixture {
    app: BondlineApp,
    context: egui::Context,
    directory: support::FileTestDirectory,
    nodes: HashMap<egui::accesskit::NodeId, egui::accesskit::Node>,
}
impl Fixture {
    fn new() -> TestResult<Self> {
        let directory = support::FileTestDirectory::new()?;
        let mut stored = Engine::default().stored_document()?.clone();
        stored
            .model_mut()
            .set_page_size(PageId::new(1).ok_or("page")?, PageSize::new(128.0, 128.0)?)?;
        let mut engine = Engine::from_document(stored);
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
        };
        let context = egui::Context::default();
        configure_context(&context);
        context.enable_accesskit();
        Ok(Self {
            app,
            context,
            directory,
            nodes: HashMap::new(),
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
        input
            .viewports
            .entry(egui::ViewportId::ROOT)
            .or_default()
            .native_pixels_per_point = Some(1.0);
        let context = self.context.clone();
        let output = context.run(input, |ctx| self.app.update_workspace(ctx));
        if let Some(update) = &output.platform_output.accesskit_update {
            for (id, node) in &update.nodes {
                self.nodes.insert(*id, node.clone());
            }
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
    fn controls(&self) -> TestResult<Vec<Value>> {
        Ok(self.command("ui.controls", json!({}))?["controls"]
            .as_array()
            .ok_or("controls")?
            .clone())
    }
    fn tool(&self, label: &str) -> TestResult<(egui::accesskit::NodeId, egui::Rect)> {
        self.nodes
            .iter()
            .find_map(|(id, node)| {
                if node.name() != Some(label) {
                    return None;
                }
                let b = node.bounds()?;
                Some((
                    *id,
                    egui::Rect::from_min_max(
                        egui::pos2(b.x0 as f32, b.y0 as f32),
                        egui::pos2(b.x1 as f32, b.y1 as f32),
                    ),
                ))
            })
            .ok_or_else(|| format!("accessible actual palette button {label} missing").into())
    }
    fn click_tool(&mut self, order: &str) -> TestResult<egui::FullOutput> {
        self.frame(vec![]);
        let label = TOOLS
            .iter()
            .find(|(name, _)| *name == order)
            .ok_or("tool")?
            .1;
        let (_, bounds) = self.tool(label)?;
        let point = bounds.center();
        self.frame(vec![Event::PointerMoved(point), button(point, true)]);
        let output = self.frame(vec![button(point, false)]);
        assert_selected(&self.controls()?, order)?;
        Ok(output)
    }
    fn operation(&self, order: &str) -> TestResult<Value> {
        let result = self.command("ui.bond_tool", json!({"order":order}))?;
        assert_eq!(result["changed"], false, "mode is not an authored edit");
        let op = result
            .get("ui_bond_tool_operation")
            .ok_or("explicit mode receipt")?
            .clone();
        assert_eq!(op["order"], order);
        Ok(op)
    }
    fn point(&self, canvas: egui::Rect, p: [f64; 2]) -> TestResult<Pos2> {
        let viewport = bondline_render::PageViewport::new(
            canvas.width().floor() as u32,
            canvas.height().floor() as u32,
            PageSize::new(128.0, 128.0)?,
        )?;
        let [x, y] = viewport.canvas_position(Point2::new(p[0], p[1])?)?;
        Ok(canvas.min + egui::vec2(x, y))
    }
    fn graph(&self) -> TestResult<(Vec<BondOrder>, Vec<u16>)> {
        let engine = self.app.engine.lock().map_err(|_| "engine")?;
        let fragments = engine
            .document()?
            .pages()
            .first()
            .ok_or("page")?
            .fragments();
        assert_eq!(fragments.len(), 1);
        let f = fragments.first().ok_or("fragment")?;
        Ok((
            f.bonds().iter().map(|b| b.order).collect(),
            f.atoms()
                .iter()
                .map(|a| a.explicit_hydrogens.ok_or("literal H"))
                .collect::<Result<Vec<_>, _>>()?,
        ))
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
fn drag(a: Pos2, b: Pos2) -> Vec<Event> {
    vec![
        Event::PointerMoved(a),
        button(a, true),
        Event::PointerMoved(b),
        button(b, false),
    ]
}
fn canvas(output: &egui::FullOutput) -> TestResult<egui::Rect> {
    output
        .shapes
        .iter()
        .find_map(|s| match &s.shape {
            egui::Shape::Rect(r) if r.fill == egui::Color32::from_gray(196) => Some(s.clip_rect),
            _ => None,
        })
        .ok_or_else(|| "actual paper/clip missing".into())
}
fn assert_selected(rows: &[Value], selected: &str) -> TestResult {
    for (order, label) in TOOLS {
        let id = format!("tool.bond.{order}");
        let row = rows
            .iter()
            .find(|r| r["id"] == id)
            .ok_or_else(|| format!("central binding {id} missing"))?;
        assert_eq!(row["label"], label);
        assert_eq!(row["shortcut"], Value::Null);
        assert_eq!(row["selected"], order == selected);
    }
    Ok(())
}
fn black_bundles(output: &egui::FullOutput, canvas: egui::Rect) -> Vec<egui::Rect> {
    let mut result = output
        .shapes
        .iter()
        .filter(|s| s.clip_rect == canvas)
        .filter_map(|s| match &s.shape {
            egui::Shape::Path(p) if p.fill == egui::Color32::BLACK && p.closed => {
                assert_eq!(p.points.len(), 4);
                let r = egui::Rect::from_points(&p.points);
                (r.width() > r.height() * 2.0).then_some(r)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    result.sort_by(|a, b| a.center().y.total_cmp(&b.center().y));
    result
}
fn assert_bundle(
    fixture: &Fixture,
    output: &egui::FullOutput,
    a: [f64; 2],
    b: [f64; 2],
    order: usize,
) -> TestResult<Vec<egui::Rect>> {
    let canvas = canvas(output)?;
    let a = fixture.point(canvas, a)?;
    let b = fixture.point(canvas, b)?;
    let scale = bondline_render::PageViewport::new(
        canvas.width().floor() as u32,
        canvas.height().floor() as u32,
        PageSize::new(128.0, 128.0)?,
    )?
    .stroke_width();
    let offsets: &[f32] = match order {
        1 => &[0.0],
        2 => &[-1.5, 1.5],
        3 => &[-3.0, 0.0, 3.0],
        _ => return Err("test order".into()),
    };
    let rects = black_bundles(output, canvas);
    assert_eq!(
        rects.len(),
        order,
        "one actual polygon per visible component"
    );
    for (r, offset) in rects.iter().zip(offsets) {
        assert!(
            (r.left() - a.x).abs() < EPS && (r.right() - b.x).abs() < EPS,
            "authored endpoints moved"
        );
        assert!((r.height() - scale).abs() < EPS);
        assert!(
            (r.center().y - (a.y + offset * scale)).abs() < EPS,
            "component spacing is not the declared3w"
        );
    }
    Ok(rects)
}
fn assert_hover(
    output: &egui::FullOutput,
    canvas: egui::Rect,
    components: &[egui::Rect],
) -> TestResult {
    let mut outlines = output
        .shapes
        .iter()
        .filter(|s| s.clip_rect == canvas)
        .filter_map(|s| match &s.shape {
            egui::Shape::Path(p) if p.stroke.color == egui::epaint::ColorMode::Solid(HOVER) => {
                Some(egui::Rect::from_points(&p.points))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    outlines.sort_by(|a, b| a.center().y.total_cmp(&b.center().y));
    assert_eq!(
        outlines.len(),
        components.len(),
        "hover must outline every actual component, not an axis"
    );
    for (o, c) in outlines.iter().zip(components) {
        assert!((o.center().y - c.center().y).abs() < EPS);
        assert!((o.left() - c.left()).abs() < EPS && (o.right() - c.right()).abs() < EPS);
        assert!((o.height() - c.height() - 4.0).abs() < EPS);
    }
    Ok(())
}
fn icon_lines(output: &egui::FullOutput, bounds: egui::Rect) -> usize {
    output
        .shapes
        .iter()
        .filter(|s| match &s.shape {
            egui::Shape::LineSegment { points, .. } => points.iter().all(|p| bounds.contains(*p)),
            egui::Shape::Path(p) if p.points.len() == 2 => {
                p.points.iter().all(|p| bounds.contains(*p))
            }
            _ => false,
        })
        .count()
}
fn consume_saved(path: &std::path::Path, orders: &[u8], hydrogens: &[u16]) -> TestResult {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let output=Process::new(python).args(["-B","-c",r#"
import sys,json
from pathlib import Path
from rdkit import Chem,rdBase
assert rdBase.rdkitVersion=='2026.09.2'
mols=Chem.MolsFromCDXML(Path(sys.argv[1]).read_text(encoding='utf-8'))
assert len(mols)==1
m=mols[0];assert all(a.GetAtomicNum()==6 and a.GetFormalCharge()==0 and a.GetIsotope()==0 for a in m.GetAtoms())
assert [b.GetBondTypeAsDouble() for b in m.GetBonds()]==json.loads(sys.argv[2])
assert [a.GetTotalNumHs() for a in m.GetAtoms()]==json.loads(sys.argv[3])
print(Chem.MolToSmiles(m))
"#]).arg(path).arg(serde_json::to_string(orders)?).arg(serde_json::to_string(hydrogens)?).output()?;
    assert!(
        output.status.success(),
        "independent actual-CDXML consumer: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}
fn pixel_strips(
    fixture: &Fixture,
    output: &egui::FullOutput,
    a: [f64; 2],
    b: [f64; 2],
    order: usize,
) -> TestResult {
    let canvas = canvas(output)?;
    let path = fixture.directory.path("actual-created-bond.png");
    let result=fixture.app.engine.lock().map_err(|_|"engine")?.dispatch("render.capture",json!({"page_id":1,"width_px":canvas.width().floor()as u32,"height_px":canvas.height().floor()as u32,"backend":"cpu"}))?;
    fs::write(
        &path,
        result.capture.ok_or("actual registered PNG")?.png_bytes()?,
    )?;
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let result=Process::new(python).args(["-B","-c",r#"
import sys,math,PIL
from PIL import Image
assert PIL.__version__=='12.3.0'
im=Image.open(sys.argv[1]);assert im.mode=='RGBA'
w,h=im.size;p=min(w-64,h-64);s=p/128;ox,oy=(w-p)//2,(h-p)//2
x=math.floor(ox+(float(sys.argv[2])+float(sys.argv[3]))/2*s);y=oy+float(sys.argv[4])*s;n=int(sys.argv[5])
offsets={1:[0],2:[-1.5,1.5],3:[-3,0,3]}[n]
for d in offsets:assert im.getpixel((x,math.floor(y+d*s)))==(0,0,0,255),('missing component',n,d)
for d in ({1:[],2:[0],3:[-1.5,1.5]}[n]):assert im.getpixel((x,math.floor(y+d*s)))==(255,255,255,255),('collapsed/gap ink',n,d)
"#]).arg(path).arg(a[0].to_string()).arg(b[0].to_string()).arg(a[1].to_string()).arg(order.to_string()).output()?;
    assert!(
        result.status.success(),
        "independent analytic PNG strips: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(())
}

#[test]
fn central_palette_controls_icons_and_actual_clicks_have_ephemeral_order_epochs() -> TestResult {
    let mut f = Fixture::new()?;
    let before = f.state()?;
    let output = f.warm();
    assert_selected(&f.controls()?, "single")?;
    for (index, (_, label)) in TOOLS.iter().enumerate() {
        let (_, bounds) = f.tool(label)?;
        assert_eq!(
            icon_lines(&output, bounds),
            index + 1,
            "original procedural icon role"
        );
    }
    assert_eq!(
        f.operation("single")?,
        json!({"order":"single","epoch":0,"changed":false})
    );
    for (order, epoch) in [("double", 1), ("triple", 2)] {
        f.click_tool(order)?;
        assert_eq!(
            f.operation(order)?,
            json!({"order":order,"epoch":epoch,"changed":false})
        );
        assert_eq!(
            f.state()?,
            before,
            "tool mode dirtied normal authored/session state"
        );
    }
    f.click_tool("triple")?;
    assert_eq!(f.operation("triple")?["epoch"], 2);
    assert_eq!(f.state()?, before);
    Ok(())
}

#[test]
fn real_palette_creates_paints_and_hovers_double_triple_with_save_history_and_reopen() -> TestResult
{
    for (order, count, bond, h) in [
        ("double", 2, BondOrder::Double, 2),
        ("triple", 3, BondOrder::Triple, 1),
    ] {
        let mut f = Fixture::new()?;
        let output = f.warm();
        let canvas = canvas(&output)?;
        f.click_tool(order)?;
        let a = [20.0, 50.0];
        let b = [100.0, 50.0];
        let begin = f.point(canvas, a)?;
        let end = f.point(canvas, b)?;
        let empty = f.state()?["document"].clone();
        let output = f.frame(drag(begin, end));
        assert!(f.app.error.is_none(), "{:?}", f.app.error);
        assert_eq!(f.graph()?, (vec![bond], vec![h, h]));
        assert_eq!(f.state()?["undo_depth"], 1);
        let rects = assert_bundle(&f, &output, a, b, count)?;
        pixel_strips(&f, &output, a, b, count)?;
        for r in &rects {
            let output = f.frame(vec![Event::PointerMoved(r.center())]);
            assert!(
                matches!(f.app.drawing.hover_target(),Some(bondline_render::HitTarget::Bond{bond_id,..})if bond_id.get()==1)
            );
            assert_hover(&output, canvas, &rects)?;
        }
        let saved = f.directory.path("multiple.cdxml");
        f.command("document.save_cdxml", json!({"path":saved}))?;
        consume_saved(&saved, &[count as u8], &[h, h])?;
        f.command("history.undo", json!({}))?;
        assert_eq!(f.state()?["document"], empty);
        assert_selected(&f.controls()?, order)?;
        f.command("history.redo", json!({}))?;
        assert_eq!(f.graph()?, (vec![bond], vec![h, h]));
        assert_eq!(f.state()?["file"]["dirty"], false);
        f.command("document.open_cdxml", json!({"path":saved}))?;
        assert_selected(&f.controls()?, order)?;
        assert_eq!(f.operation(order)?["epoch"], 1);
        let output = f.frame(vec![]);
        assert_bundle(&f, &output, a, b, count)?;
    }
    Ok(())
}

#[test]
fn actual_mode_activation_cancels_old_gesture_and_same_frame_fresh_press_uses_new_order()
-> TestResult {
    let mut f = Fixture::new()?;
    let initial = f.warm();
    assert_selected(&f.controls()?, "single")?;
    let canvas = canvas(&initial)?;
    let old = f.point(canvas, [20.0, 25.0])?;
    let old_end = f.point(canvas, [100.0, 25.0])?;
    f.frame(vec![Event::PointerMoved(old), button(old, true)]);
    assert!(f.app.drawing.gesture.is_some());
    let (id, _) = f.tool("Double bond")?;
    let a = [20.0, 80.0];
    let b = [100.0, 80.0];
    let begin = f.point(canvas, a)?;
    let end = f.point(canvas, b)?;
    let mut events = vec![
        Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::Default,
            target: id,
            data: None,
        }),
        Event::PointerMoved(old_end),
        button(old_end, false),
    ];
    events.extend(drag(begin, end));
    let output = f.frame(events);
    assert!(f.app.error.is_none(), "{:?}", f.app.error);
    assert_eq!(f.graph()?, (vec![BondOrder::Double], vec![2, 2]));
    assert_eq!(f.state()?["undo_depth"], 1);
    assert_bundle(&f, &output, a, b, 2)?;
    // A->B->A has unchanged final order but a new epoch; it cannot revive an
    // armed gesture or hover from the old context.
    let empty = f.point(canvas, [20.0, 105.0])?;
    let end = f.point(canvas, [100.0, 105.0])?;
    f.frame(vec![Event::PointerMoved(empty), button(empty, true)]);
    let before = f.state()?;
    assert_eq!(f.operation("triple")?["changed"], true);
    assert_eq!(f.operation("double")?["changed"], true);
    f.frame(vec![Event::PointerMoved(end), button(end, false)]);
    assert_eq!(f.state()?, before);
    assert!(f.app.drawing.gesture.is_none());
    assert!(f.app.drawing.hover_target().is_none());
    let rects = black_bundles(&f.frame(vec![]), canvas);
    let point = rects.first().ok_or("component")?.center();
    f.frame(vec![Event::PointerMoved(point)]);
    assert!(f.app.drawing.hover_target().is_some());
    f.operation("single")?;
    for _ in 0..2 {
        f.frame(vec![]);
        assert!(f.app.drawing.hover_target().is_none());
    }
    Ok(())
}

#[test]
fn existing_single_anchor_extends_with_selected_multiple_order_and_refreshes_every_component()
-> TestResult {
    for (order, count, bond, hs) in [
        ("double", 2, BondOrder::Double, [3, 1, 2]),
        ("triple", 3, BondOrder::Triple, [3, 0, 1]),
    ] {
        let mut f = Fixture::new()?;
        let output = f.warm();
        let canvas = canvas(&output)?;
        let a = f.point(canvas, [25.0, 25.0])?;
        let anchor = f.point(canvas, [25.0, 75.0])?;
        f.frame(drag(a, anchor));
        assert_eq!(f.graph()?, (vec![BondOrder::Single], vec![3, 3]));
        f.click_tool(order)?;
        let end = f.point(canvas, [100.0, 75.0])?;
        let output = f.frame(drag(anchor, end));
        assert!(f.app.error.is_none(), "{:?}", f.app.error);
        assert_eq!(f.graph()?, (vec![BondOrder::Single, bond], hs.to_vec()));
        assert_eq!(f.state()?["undo_depth"], 2);
        let rects = assert_bundle(&f, &output, [25.0, 75.0], [100.0, 75.0], count)?;
        for r in &rects {
            let output = f.frame(vec![Event::PointerMoved(r.center())]);
            assert!(
                matches!(f.app.drawing.hover_target(),Some(bondline_render::HitTarget::Bond{bond_id,..})if bond_id.get()==2)
            );
            assert_hover(&output, canvas, &rects)?;
        }
        let saved = f.directory.path("extended.cdxml");
        f.command("document.save_cdxml", json!({"path":saved}))?;
        consume_saved(&saved, &[1, count as u8], &hs)?;
        f.command("history.undo", json!({}))?;
        assert_eq!(f.graph()?, (vec![BondOrder::Single], vec![3, 3]));
        assert_selected(&f.controls()?, order)?;
        f.command("history.redo", json!({}))?;
        assert_eq!(f.graph()?, (vec![BondOrder::Single, bond], hs.to_vec()));
    }
    Ok(())
}

#[test]
fn held_double_triple_previews_have_real_components_and_cancel_without_authored_changes()
-> TestResult {
    let preview_color = egui::Color32::from_rgb(70, 105, 155);
    for (order, offsets) in [
        ("double", &[-1.5_f32, 1.5][..]),
        ("triple", &[-3.0_f32, 0.0, 3.0][..]),
    ] {
        let mut fixture = Fixture::new()?;
        fixture.warm();
        let output = fixture.click_tool(order)?;
        let canvas = canvas(&output)?;
        let begin = fixture.point(canvas, [20.0, 50.0])?;
        let end = fixture.point(canvas, [100.0, 50.0])?;
        let before = fixture.state()?;
        fixture.frame(vec![Event::PointerMoved(begin), button(begin, true)]);
        let output = fixture.frame(vec![Event::PointerMoved(end)]);
        assert!(fixture.app.drawing.gesture.is_some());
        assert_eq!(fixture.state()?, before);
        let mut components = output
            .shapes
            .iter()
            .filter(|s| s.clip_rect == canvas)
            .filter_map(|s| match &s.shape {
                egui::Shape::Path(path) if path.fill == preview_color && path.closed => {
                    assert_eq!(path.points.len(), 4);
                    Some(egui::Rect::from_points(&path.points))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        components.sort_by(|a, b| a.center().y.total_cmp(&b.center().y));
        assert_eq!(
            components.len(),
            offsets.len(),
            "held {order} preview collapsed to an axis"
        );
        let width = bondline_render::PageViewport::new(
            canvas.width().floor() as u32,
            canvas.height().floor() as u32,
            PageSize::new(128.0, 128.0)?,
        )?
        .stroke_width();
        for (component, offset) in components.iter().zip(offsets) {
            assert!(
                (component.left() - begin.x).abs() < EPS && (component.right() - end.x).abs() < EPS
            );
            assert!((component.height() - width).abs() < EPS);
            assert!((component.center().y - (begin.y + offset * width)).abs() < EPS);
        }
        let output = fixture.frame(vec![Event::Key {
            key: egui::Key::Escape,
            physical_key: Some(egui::Key::Escape),
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }]);
        assert!(fixture.app.drawing.gesture.is_none());
        assert!(!output.shapes.iter().any(|s| matches!(&s.shape,
            egui::Shape::Path(path) if s.clip_rect == canvas && path.fill == preview_color)));
        assert_eq!(
            fixture.state()?,
            before,
            "cancelled preview authored data/history"
        );
        fixture.frame(vec![button(end, false)]);
        assert_eq!(fixture.state()?, before);
    }
    Ok(())
}

#[test]
fn mode_activations_follow_raw_event_order_and_cannot_retroactively_change_a_press() -> TestResult {
    let activate = |id| {
        Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::Default,
            target: id,
            data: None,
        })
    };
    let mut failures = Vec::new();
    {
        let mut fixture = Fixture::new()?;
        let initial = fixture.warm();
        let canvas = canvas(&initial)?;
        let begin = fixture.point(canvas, [20.0, 50.0])?;
        let end = fixture.point(canvas, [100.0, 50.0])?;
        let (double, _) = fixture.tool("Double bond")?;
        let before = fixture.state()?;
        fixture.frame(vec![
            Event::PointerMoved(begin),
            button(begin, true),
            activate(double),
            Event::PointerMoved(end),
            button(end, false),
        ]);
        if fixture.state()? != before {
            failures.push(
                "a Single press before Double activation authored on its old release".to_owned(),
            );
        }
        if fixture.app.drawing.gesture.is_some() {
            failures.push("mode activation left the earlier press armed".to_owned());
        }
        let op = fixture.operation("double")?;
        if op != json!({"order":"double","epoch":1,"changed":false}) {
            failures.push(format!("one activation has wrong receipt: {op}"));
        }
    }
    {
        let mut fixture = Fixture::new()?;
        fixture.warm();
        let (triple, _) = fixture.tool("Triple bond")?;
        let (double, _) = fixture.tool("Double bond")?;
        let before = fixture.state()?;
        fixture.frame(vec![activate(triple), activate(double)]);
        let rows = fixture.controls()?;
        let selected = rows
            .iter()
            .filter(|row| row["selected"] == true)
            .filter_map(|row| row["id"].as_str()?.strip_prefix("tool.bond."))
            .collect::<Vec<_>>();
        if selected != ["double"] {
            failures.push(format!(
                "Triple then Double ended with selected {selected:?}"
            ));
        }
        // Query the actual selected mode without correcting it: a same-mode
        // operation is a no-op and must expose the two transitions' epoch.
        if let [actual] = selected.as_slice() {
            let op = fixture.operation(actual)?;
            if op["epoch"] != 2 || op["changed"] != false {
                failures.push(format!("two ordered activations have wrong receipt: {op}"));
            }
        }
        if fixture.state()? != before {
            failures.push("tool activation order changed authored/file/close state".to_owned());
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("; "));
    Ok(())
}
