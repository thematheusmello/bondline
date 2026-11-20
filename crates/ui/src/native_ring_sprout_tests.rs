//! Independent actual-App Alt-atom sprout declaration; existing APIs only.
use super::{Fixture, TestResult, button, canvas, projected};
use eframe::egui;
use serde_json::json;

#[test]
fn alt_atom_with_cyclohexane_sprouts_a_distinct_six_ring_in_one_history_edit() -> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    f.command(
        "ring.create",
        json!({"expected_revision":0,"page_id":1,"size":6,
        "hydrogen_policy":"rectify_neutral_carbon","placement":{
            "mode":"free","begin_x_pt":240,"begin_y_pt":350,
            "end_x_pt":240,"end_y_pt":302,"side":"positive"}}),
    )?;
    f.command("ui.view", json!({"action":"actual"}))?;
    f.warm();
    let (id, tool) = f.node(
        "Cyclohexane",
        &[
            egui::accesskit::Role::Button,
            egui::accesskit::Role::CheckBox,
        ],
    )?;
    assert!(!f.nodes.get(&id).ok_or("Cyclohexane control")?.is_disabled());
    f.frame(vec![
        egui::Event::PointerMoved(tool.center()),
        button(tool.center(), true),
    ]);
    f.frame(vec![button(tool.center(), false)]);
    let shown = f.warm();
    assert!(
        f.app.error.is_none(),
        "the original C6 must render without a guard: {:?}",
        f.app.error
    );
    let before = f.state()?;
    let old = &before["document"]["pages"][0]["fragments"][0];
    assert_eq!(old["atoms"].as_array().ok_or("seed atoms")?.len(), 6);
    assert_eq!(old["bonds"].as_array().ok_or("seed bonds")?.len(), 6);
    let size = &before["document"]["pages"][0]["size"];
    let view = f.view()?;
    assert_eq!(view["zoom_percent"], 100.0);
    let center = match (view["center_x_pt"].as_f64(), view["center_y_pt"].as_f64()) {
        (Some(x), Some(y)) => Some([x, y]),
        _ => None,
    };
    let bounds = canvas(&shown)?;
    let target = projected(
        bounds,
        [
            size["width_pt"].as_f64().ok_or("width")?,
            size["height_pt"].as_f64().ok_or("height")?,
        ],
        100.0,
        center,
        [240.0, 350.0],
    );
    assert!(bounds.contains(target));
    f.frame(vec![egui::Event::PointerMoved(target)]);
    assert!(
        matches!(f.app.drawing.hover_target(),
        Some(bondline_render::HitTarget::Atom {fragment_id,atom_id})
        if fragment_id.get()==1 && atom_id.get()==1),
        "fresh actual atom target required"
    );
    let alt = egui::Modifiers {
        alt: true,
        ..egui::Modifiers::NONE
    };
    f.frame(vec![egui::Event::PointerButton {
        pos: target,
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: alt,
    }]);
    f.frame(vec![egui::Event::PointerButton {
        pos: target,
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: alt,
    }]);
    let after = f.state()?;
    let fragments = after["document"]["pages"][0]["fragments"]
        .as_array()
        .ok_or("fragments")?;
    assert_eq!(fragments.len(), 1, "sprout extends the original Fragment");
    let atoms = fragments[0]["atoms"].as_array().ok_or("atoms")?;
    let bonds = fragments[0]["bonds"].as_array().ok_or("bonds")?;
    assert_eq!(
        (atoms.len(), bonds.len()),
        (12, 13),
        "Alt-atom must sprout a distinct C6 joined by Single, rather than refuse or make an 11-atom spiro ring"
    );
    let mut old_atoms = old["atoms"].as_array().ok_or("old atoms")?.clone();
    old_atoms[0]["explicit_hydrogens"] = json!(1);
    assert_eq!(&atoms[..6], old_atoms.as_slice());
    assert_eq!(
        &bonds[..6],
        old["bonds"].as_array().ok_or("old bonds")?.as_slice()
    );
    assert_eq!(bonds[6], json!({"id":7,"begin":1,"end":7,"order":"single"}));
    for (i, atom) in atoms[6..].iter().enumerate() {
        assert_eq!(atom["id"], i + 7);
        assert_eq!(atom["element"], 6);
        assert_eq!(atom["explicit_hydrogens"], if i == 0 { 1 } else { 2 });
        assert_eq!(
            bonds[i + 7],
            json!({"id":i+8,"begin":i+7,
            "end":if i==5 {7} else {i+8},"order":"single"})
        );
    }
    assert_eq!(
        after["revision"],
        before["revision"].as_u64().ok_or("revision")? + 1
    );
    assert_eq!(
        after["undo_depth"],
        before["undo_depth"].as_u64().ok_or("history")? + 1
    );
    assert_eq!(after["redo_depth"], 0);
    assert_eq!(after["source"], before["source"]);
    assert_eq!(
        after["document"]["pages"][0]["size"],
        before["document"]["pages"][0]["size"]
    );
    f.command("history.undo", json!({}))?;
    assert_eq!(f.state()?["document"], before["document"]);
    Ok(())
}

// Independent stage2 expansion. Initial Alt-atom declaration remains byte-exact.
use super::key;
use egui::{
    Event, Key, Modifiers, Pos2,
    accesskit::{Action, ActionRequest, NodeId, Role, Toggled},
};
use serde_json::Value;
use std::{
    fs,
    path::PathBuf,
    time::{Duration, Instant},
};
const PREF_CHECK: &str = "Sprout rings instead of spiro when clicking";
const ROOT_POINT: [f64; 2] = [240.0, 350.0];
const TOOLS: [(&str, usize); 10] = [
    ("Cyclopropane", 3),
    ("Cyclobutane", 4),
    ("Cyclopentane", 5),
    ("Cyclohexane", 6),
    ("Cycloheptane", 7),
    ("Cyclooctane", 8),
    ("Benzene", 6),
    ("Cyclopentadiene", 5),
    ("Cyclohexane chair 1", 6),
    ("Cyclohexane chair 2", 6),
];
#[track_caller]
fn rs_wait(
    f: &mut Fixture,
    ready: impl Fn(&Fixture, &egui::FullOutput) -> bool,
) -> TestResult<egui::FullOutput> {
    let caller = std::panic::Location::caller();
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let shown = f.frame(vec![]);
        if ready(f, &shown) {
            return Ok(shown);
        }
        if Instant::now() >= deadline {
            return Err(format!("sprout App condition at {caller}: {:?}", f.app.error).into());
        }
        std::thread::yield_now();
    }
}
fn rs_text(shown: &egui::FullOutput, text: &str) -> bool {
    shown.shapes.iter().any(|paint|matches!(&paint.shape,egui::Shape::Text(t)
        if t.galley.text().contains(text) && paint.clip_rect.contains_rect(paint.shape.visual_bounding_rect())))
}
fn rs_action(id: NodeId) -> Event {
    Event::AccessKitActionRequest(ActionRequest {
        action: Action::Default,
        target: id,
        data: None,
    })
}
fn rs_button(f: &Fixture, name: &str) -> TestResult<Event> {
    let (id, _) = f.node(name, &[Role::Button])?;
    assert!(
        !f.nodes.get(&id).ok_or("button")?.is_disabled(),
        "{name} must be enabled"
    );
    Ok(rs_action(id))
}
fn rs_select(f: &mut Fixture, name: &str) -> TestResult {
    rs_wait(f, |f, _| {
        f.node(name, &[Role::Button, Role::CheckBox])
            .is_ok_and(|(id, _)| f.nodes.get(&id).is_some_and(|n| !n.is_disabled()))
    })?;
    let (_, rect) = f.node(name, &[Role::Button, Role::CheckBox])?;
    assert!(rect.is_positive());
    f.frame(vec![
        Event::PointerMoved(rect.center()),
        button(rect.center(), true),
    ]);
    f.frame(vec![button(rect.center(), false)]);
    let controls = f.command("ui.controls", json!({}))?;
    let selected = controls["controls"]
        .as_array()
        .ok_or("controls")?
        .iter()
        .filter(|row| row["menu"] == "Tools" && row["selected"] == true)
        .collect::<Vec<_>>();
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0]["label"], name);
    Ok(())
}
fn rs_seed(f: &mut Fixture) -> TestResult<PathBuf> {
    f.command(
        "ring.create",
        json!({"expected_revision":0,"page_id":1,"size":6,
        "hydrogen_policy":"rectify_neutral_carbon","placement":{"mode":"free",
        "begin_x_pt":240,"begin_y_pt":350,"end_x_pt":240,"end_y_pt":302,"side":"positive"}}),
    )?;
    let path = f.directory.path("sprout-original-λ.cdxml");
    f.command("document.save_cdxml", json!({"path":path}))?;
    f.command("document.open_cdxml", json!({"path":path}))?;
    f.command("ui.view", json!({"action":"actual"}))?;
    f.warm();
    assert_eq!(f.state()?["undo_depth"], 0);
    assert_eq!(f.state()?["file"]["dirty"], false);
    Ok(path)
}
fn rs_pref(f: &mut Fixture, value: bool) -> TestResult {
    let old = f.state()?;
    let epoch = f.command("ui.preferences", json!({"action":"inspect"}))?["preferences"]["epoch"]
        .as_u64()
        .ok_or("preferences epoch")?;
    let result = f.command(
        "ui.preferences",
        json!({"action":"set","expected_epoch":epoch,
        "sprout_rings_when_clicking":value}),
    )?;
    assert_eq!(result["changed"], false);
    assert_eq!(result["preferences"]["sprout_rings_when_clicking"], value);
    assert_eq!(f.state()?, old);
    Ok(())
}
fn rs_at(f: &Fixture, area: egui::Rect, point: [f64; 2]) -> TestResult<Pos2> {
    let state = f.state()?;
    let size = &state["document"]["pages"][0]["size"];
    let view = f.view()?;
    assert_eq!(view["zoom_percent"], 100.0);
    let center = match (view["center_x_pt"].as_f64(), view["center_y_pt"].as_f64()) {
        (Some(x), Some(y)) => Some([x, y]),
        _ => None,
    };
    let point = projected(
        area,
        [
            size["width_pt"].as_f64().ok_or("width")?,
            size["height_pt"].as_f64().ok_or("height")?,
        ],
        100.0,
        center,
        point,
    );
    assert!(area.contains(point), "declared point on actual canvas");
    Ok(point)
}
fn rs_pointer(pos: Pos2, pressed: bool, modifiers: Modifiers) -> Event {
    Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers,
    }
}
fn rs_alt() -> Modifiers {
    Modifiers {
        alt: true,
        ..Modifiers::NONE
    }
}
fn rs_click(
    f: &mut Fixture,
    point: [f64; 2],
    press: Modifiers,
    release: Modifiers,
) -> TestResult<egui::FullOutput> {
    let area = canvas(&f.warm())?;
    let at = rs_at(f, area, point)?;
    f.frame(vec![Event::PointerMoved(at), rs_pointer(at, true, press)]);
    Ok(f.frame(vec![rs_pointer(at, false, release)]))
}
fn rs_same(f: &Fixture, before: &Value) -> TestResult {
    let after = f.state()?;
    for key in [
        "document",
        "revision",
        "undo_depth",
        "redo_depth",
        "source",
        "file",
        "close",
        "close_epoch",
    ] {
        assert_eq!(after[key], before[key], "presentation authored {key}");
    }
    Ok(())
}
fn rs_compare(f: &Fixture, expected: &Value) -> TestResult {
    let mut actual = f.state()?["document"].clone();
    let graphs = actual["pages"][0]["fragments"]
        .as_array_mut()
        .ok_or("fragments")?;
    let wanted = expected["pages"][0]["fragments"]
        .as_array()
        .ok_or("expected fragments")?;
    assert_eq!(graphs.len(), wanted.len());
    for (a, e) in graphs.iter_mut().zip(wanted) {
        let atoms = a["atoms"].as_array_mut().ok_or("atoms")?;
        let wants = e["atoms"].as_array().ok_or("expected atoms")?;
        assert_eq!(atoms.len(), wants.len());
        for (a, e) in atoms.iter_mut().zip(wants) {
            for axis in ["x_pt", "y_pt"] {
                assert!(
                    (a["position"][axis].as_f64().ok_or("position")?
                        - e["position"][axis].as_f64().ok_or("literal coordinate")?)
                    .abs()
                        <= 1e-9,
                    "{axis}"
                );
            }
            a["position"] = e["position"].clone();
        }
    }
    assert_eq!(
        actual, *expected,
        "whole literal model/arrays/H/metadata/flag presence"
    );
    Ok(())
}
fn rs_edit(f: &Fixture, before: &Value, path: &std::path::Path, raw: &[u8]) -> TestResult {
    let after = f.state()?;
    assert_eq!(
        after["revision"],
        before["revision"].as_u64().ok_or("revision")? + 1
    );
    assert_eq!(
        after["undo_depth"],
        before["undo_depth"].as_u64().ok_or("history")? + 1
    );
    assert_eq!(after["redo_depth"], 0);
    assert_eq!(after["file"]["dirty"], true);
    assert_eq!(after["file"]["path"], before["file"]["path"]);
    assert_eq!(after["source"]["modified"], true);
    assert_eq!(after["source"]["format"], "cdxml");
    assert_eq!(fs::read(path)?, raw);
    Ok(())
}
fn rs_points(
    kind: &str,
    n: usize,
    anchor: [f64; 2],
    outward: [f64; 2],
    length: f64,
) -> Vec<[f64; 2]> {
    if kind.starts_with("Cyclohexane chair") {
        // Original public analytic seed, independently normalized by its projected reference edge.
        let h = 2.0_f64.sqrt() / 8.0;
        let q = 3.0_f64.sqrt() / 8.0;
        let g = 4.0 / 17.0_f64.sqrt();
        let p = if kind.ends_with('1') {
            [
                [0.5, -g * (h + q)],
                [1.0, g * h],
                [0.5, g * (q - h)],
                [-0.5, g * (q + h)],
                [-1.0, -g * h],
                [-0.5, g * (h - q)],
            ]
        } else {
            [
                [-1.0, g * h],
                [-0.5, -g * (h + q)],
                [0.5, g * (h - q)],
                [1.0, -g * h],
                [0.5, g * (h + q)],
                [-0.5, g * (q - h)],
            ]
        };
        let c = [-p[0][0], -p[0][1]];
        let cn = c[0].hypot(c[1]);
        let scale = length / (p[1][0] - p[0][0]).hypot(p[1][1] - p[0][1]);
        let base = p[0];
        return p
            .iter()
            .map(|point| {
                let v = [point[0] - base[0], point[1] - base[1]];
                let along = (v[0] * c[0] + v[1] * c[1]) / cn;
                let across = (c[0] * v[1] - c[1] * v[0]) / cn;
                [
                    anchor[0] + scale * (along * outward[0] - across * outward[1]),
                    anchor[1] + scale * (along * outward[1] + across * outward[0]),
                ]
            })
            .collect();
    }
    let first =
        outward[1].atan2(outward[0]) - (n as f64 - 2.0) * std::f64::consts::PI / (2.0 * n as f64);
    let mut points = vec![anchor];
    for i in 0..n - 1 {
        let angle = first + i as f64 * std::f64::consts::TAU / n as f64;
        let previous = points[i];
        points.push([
            previous[0] + length * angle.cos(),
            previous[1] + length * angle.sin(),
        ]);
    }
    points
}
fn rs_weights(kind: &str, n: usize, single_first: bool) -> Vec<u8> {
    match kind {
        "Benzene" => {
            if single_first {
                vec![1, 2, 1, 2, 1, 2]
            } else {
                vec![2, 1, 2, 1, 2, 1]
            }
        }
        "Cyclopentadiene" => {
            if single_first {
                vec![1, 2, 1, 1, 2]
            } else {
                vec![2, 1, 2, 1, 1]
            }
        }
        _ => vec![1; n],
    }
}
fn rs_atom(id: usize, point: [f64; 2], h: u16) -> Value {
    json!({"id":id,"element":6,"isotope":null,"charge":0,"radical":"none",
        "position":{"x_pt":point[0],"y_pt":point[1]},"explicit_hydrogens":h,
        "show_non_terminal_carbon_labels":false})
}
fn rs_expected(
    before: &Value,
    kind: &str,
    n: usize,
    single: bool,
    connector: [f64; 2],
) -> TestResult<Value> {
    let length = connector[0].hypot(connector[1]);
    assert!(length > 0.0);
    let outward = [connector[0] / length, connector[1] / length];
    let anchor = [ROOT_POINT[0] + connector[0], ROOT_POINT[1] + connector[1]];
    let points = rs_points(kind, n, anchor, outward, length);
    let weights = rs_weights(kind, n, single);
    let mut expected = before["document"].clone();
    let graph = &mut expected["pages"][0]["fragments"][0];
    let atoms = graph["atoms"].as_array_mut().ok_or("old atoms")?;
    assert_eq!(atoms.len(), 6);
    atoms[0]["explicit_hydrogens"] = json!(1);
    for (i, p) in points.iter().enumerate() {
        let w = u16::from(weights[(i + n - 1) % n]) + u16::from(weights[i]) + u16::from(i == 0);
        atoms.push(rs_atom(i + 7, *p, 4 - w));
    }
    let bonds = graph["bonds"].as_array_mut().ok_or("old bonds")?;
    assert_eq!(bonds.len(), 6);
    bonds.push(json!({"id":7,"begin":1,"end":7,"order":"single"}));
    for (i, w) in weights.iter().enumerate() {
        let mut b = json!({"id":i+8,"begin":i+7,"end":7+(i+1)%n,
        "order":if *w==2 {"double"}else{"single"}});
        if *w == 2 {
            b["double_position"] = json!("right");
        }
        bonds.push(b);
    }
    Ok(expected)
}
fn rs_click_vector() -> [f64; 2] {
    [-24.0 * 3.0_f64.sqrt(), 24.0]
}
fn rs_spiro(before: &Value) -> TestResult<Value> {
    let mut expected = before["document"].clone();
    let graph = &mut expected["pages"][0]["fragments"][0];
    let points = rs_points(
        "Cyclohexane",
        6,
        ROOT_POINT,
        [-3.0_f64.sqrt() / 2.0, 0.5],
        48.0,
    );
    let atoms = graph["atoms"].as_array_mut().ok_or("atoms")?;
    atoms[0]["explicit_hydrogens"] = json!(0);
    for (i, p) in points.iter().enumerate().skip(1) {
        let mut atom = rs_atom(i + 6, *p, 2);
        atom.as_object_mut()
            .ok_or("spiro atom")?
            .remove("show_non_terminal_carbon_labels");
        atoms.push(atom);
    }
    let ids = [1, 7, 8, 9, 10, 11];
    let bonds = graph["bonds"].as_array_mut().ok_or("bonds")?;
    for i in 0..6 {
        bonds.push(json!({"id":i+7,"begin":ids[i],"end":ids[(i+1)%6],"order":"single"}));
    }
    Ok(expected)
}
fn rs_paint(f: &Fixture, shown: &egui::FullOutput, expected: &Value) -> TestResult {
    let area = canvas(shown)?;
    let page = bondline_model::PageId::new(1).ok_or("page")?;
    let logical = f.app.engine.lock().map_err(|_| "engine")?.page_view(
        area.width().floor() as u32,
        area.height().floor() as u32,
        page,
    )?;
    let density = f.context.pixels_per_point();
    assert_eq!(density, f.dpi);
    let device = [
        f64::from(area.min.x) * f64::from(density),
        f64::from(area.min.y) * f64::from(density),
    ];
    let (origin, view) = if density == 1.0 {
        (area.min, logical)
    } else {
        (
            egui::pos2(
                (device[0].floor() / f64::from(density)) as f32,
                (device[1].floor() / f64::from(density)) as f32,
            ),
            logical.at_display_scale(f64::from(density), device.map(|v| v - v.floor()))?,
        )
    };
    let mut stored = bondline_commands::Engine::default()
        .stored_document()?
        .clone();
    let mut fragments = Vec::new();
    for value in expected["pages"][0]["fragments"]
        .as_array()
        .ok_or("expected fragments")?
    {
        fragments.push(bondline_model::Fragment::new(
            bondline_model::FragmentId::new(value["id"].as_u64().ok_or("fragment ID")?.try_into()?)
                .ok_or("fragment ID")?,
            serde_json::from_value(value["atoms"].clone())?,
            serde_json::from_value(value["bonds"].clone())?,
        )?);
    }
    stored.model_mut().append_fragments(page, fragments)?;
    assert_eq!(serde_json::to_value(stored.model())?, *expected);
    let reference = bondline_commands::Engine::from_document(stored);
    let graph = reference.drawing_scene_with_view(
        &view,
        page,
        &bondline_render::Cancellation::default(),
    )?;
    let mut wants = Vec::new();
    for primitive in &graph.scene().primitives {
        match primitive {
            bondline_render::Primitive::Rectangle(r) if r.color == [0, 0, 0, 255] => {
                wants.push(vec![
                    0.0,
                    origin.x + r.x as f32 / density,
                    origin.y + r.y as f32 / density,
                    origin.x + (r.x + r.width) as f32 / density,
                    origin.y + (r.y + r.height) as f32 / density,
                ])
            }
            bondline_render::Primitive::Stroke(s) if s.color == [0, 0, 0, 255] => {
                let mut points = vec![1.0];
                points.extend(
                    s.corners()?
                        .into_iter()
                        .flat_map(|[x, y]| [origin.x + x / density, origin.y + y / density]),
                );
                wants.push(points);
            }
            _ => {}
        }
    }
    let mut got = Vec::new();
    for paint in &shown.shapes {
        match &paint.shape {
            egui::Shape::Rect(r)
                if r.fill == egui::Color32::BLACK && area.contains_rect(r.rect) =>
            {
                got.push(vec![
                    0.0,
                    r.rect.left(),
                    r.rect.top(),
                    r.rect.right(),
                    r.rect.bottom(),
                ])
            }
            egui::Shape::Path(p)
                if p.fill == egui::Color32::BLACK
                    && p.points.len() == 4
                    && p.points.iter().all(|p| area.contains(*p)) =>
            {
                let mut points = vec![1.0];
                points.extend(p.points.iter().flat_map(|p| [p.x, p.y]));
                got.push(points);
            }
            _ => {}
        }
    }
    let order = |a: &Vec<f32>, b: &Vec<f32>| {
        a.iter()
            .zip(b)
            .map(|(a, b)| a.total_cmp(b))
            .find(|o| !o.is_eq())
            .unwrap_or_else(|| a.len().cmp(&b.len()))
    };
    got.sort_by(order);
    wants.sort_by(order);
    assert!(!wants.is_empty());
    assert_eq!(got.len(), wants.len(), "whole candidate ink");
    for (a, e) in got.iter().zip(&wants) {
        assert_eq!(a.len(), e.len());
        for (a, e) in a.iter().zip(e) {
            assert!(
                (a - e).abs() * density <= 1.0 / 256.0,
                "physical candidate primitive {a} != {e}"
            );
        }
    }
    Ok(())
}
fn rs_form(f: &mut Fixture) -> TestResult {
    rs_wait(f, |f, shown| {
        rs_text(shown, "Building/Display")
            && f.node(PREF_CHECK, &[Role::CheckBox])
                .is_ok_and(|(id, _)| f.nodes.get(&id).is_some_and(|n| !n.is_disabled()))
    })?;
    for name in ["Use Defaults", "OK", "Cancel"] {
        f.node(name, &[Role::Button])?;
    }
    Ok(())
}
fn rs_closed(f: &mut Fixture) -> TestResult {
    rs_wait(f, |f, _| f.node(PREF_CHECK, &[Role::CheckBox]).is_err())?;
    Ok(())
}
fn rs_inspect(f: &Fixture) -> TestResult<Value> {
    Ok(f.command("ui.preferences", json!({"action":"inspect"}))?["preferences"].clone())
}

#[test]
fn preferences_file_menu_and_ordered_checkbox_defaults_ok_cancel_are_real_session_drafts()
-> TestResult {
    // Every subcase opens the actual File menu; no future Rust APIs or direct form setters.
    for (initial, operations, wanted) in [
        (false, vec!["toggle", "ok"], true),
        (false, vec!["ok", "toggle"], false),
        (false, vec!["toggle", "toggle", "ok"], false),
        (false, vec!["toggle", "defaults", "ok"], false),
        (true, vec!["defaults", "ok"], false),
        (true, vec!["ok", "defaults"], true),
        (false, vec!["toggle", "cancel"], false),
    ] {
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        let path = rs_seed(&mut f)?;
        let raw = fs::read(&path)?;
        if initial {
            rs_pref(&mut f, true)?;
        }
        let cancelling = operations.contains(&"cancel");
        if cancelling {
            f.command(
                "bond.create",
                json!({"page_id":1,"begin_x_pt":420,"begin_y_pt":600,
            "end_x_pt":468,"end_y_pt":600}),
            )?;
        }
        let before = f.state()?;
        f.choose("File")?;
        rs_wait(&mut f, |f, _| {
            f.node("Preferences…", &[Role::Button])
                .is_ok_and(|(id, _)| f.nodes.get(&id).is_some_and(|n| !n.is_disabled()))
        })?;
        f.choose("Preferences…")?;
        rs_form(&mut f)?;
        let (check, _) = f.node(PREF_CHECK, &[Role::CheckBox])?;
        assert_eq!(
            f.nodes.get(&check).ok_or("checkbox")?.toggled(),
            Some(if initial {
                Toggled::True
            } else {
                Toggled::False
            })
        );
        let id = f.state()?["ui_preferences"]["id"]
            .as_u64()
            .ok_or("preferences request")?;
        assert_eq!(f.state()?["ui_preferences"]["current"], true);
        assert_eq!(
            f.state()?["ui_preferences"]["sprout_rings_when_clicking"],
            initial
        );
        rs_same(&f, &before)?;
        let mut packet = Vec::new();
        for operation in operations {
            packet.push(match operation {
                "toggle" => rs_action(check),
                "defaults" => rs_button(&f, "Use Defaults")?,
                "ok" => rs_button(&f, "OK")?,
                "cancel" => rs_button(&f, "Cancel")?,
                _ => return Err("operation".into()),
            });
        }
        if cancelling {
            let area = canvas(&f.warm())?;
            let a = rs_at(&f, area, [320.0, 600.0])?;
            let b = rs_at(&f, area, [368.0, 600.0])?;
            packet.extend([
                Event::PointerMoved(a),
                rs_pointer(a, true, Modifiers::NONE),
                Event::PointerMoved(b),
                rs_pointer(b, false, Modifiers::NONE),
                key(Key::Z, true, Modifiers::COMMAND),
                key(Key::Z, false, Modifiers::COMMAND),
                key(Key::S, true, Modifiers::COMMAND),
                key(Key::S, false, Modifiers::COMMAND),
            ]);
        }
        f.frame(packet);
        rs_closed(&mut f)?;
        let state = f.state()?;
        assert!(state.get("ui_preferences").is_none());
        assert_eq!(
            rs_inspect(&f)?,
            json!({"sprout_rings_when_clicking":wanted,
            "epoch":u64::from(initial)+u64::from(wanted!=initial)})
        );
        assert!(id > 0);
        rs_same(&f, &before)?;
        assert_eq!(fs::read(&path)?, raw);
        assert!(f.state()?["ui_file"]["request"].is_null());
        assert!(!f.app.files.is_busy());
        let controls = f.command("ui.controls", json!({}))?;
        let controls = controls["controls"].as_array().ok_or("controls")?;
        assert_eq!(controls.len(), 41);
        assert!(
            controls
                .iter()
                .any(|row| row["id"] == "tool.caption" && row["enabled"] == true)
        );
        let control = controls
            .iter()
            .find(|row| row["id"] == "file.preferences")
            .ok_or("Preferences central binding")?;
        assert_eq!(control["label"], "Preferences…");
        assert_eq!(control["menu"], "File");
        assert_eq!(control["enabled"], true);
        assert_eq!(
            control["command"],
            json!({"command":"ui.preferences","params":{"action":"open",
            "expected_revision":before["revision"]}})
        );
    }
    Ok(())
}

#[test]
fn all_ten_alt_atom_tools_paint_the_literal_sprout_cancel_and_then_commit_one_distinct_cycle()
-> TestResult {
    for (index, (kind, n)) in TOOLS.into_iter().enumerate() {
        let mut f = Fixture::new(1160.0, 800.0, if index % 2 == 0 { 1.0 } else { 2.0 })?;
        let path = rs_seed(&mut f)?;
        let raw = fs::read(&path)?;
        rs_select(&mut f, kind)?;
        let shown = f.warm();
        let area = canvas(&shown)?;
        let before = f.state()?;
        let target = rs_at(&f, area, ROOT_POINT)?;
        let expected = rs_expected(&before, kind, n, false, rs_click_vector())?;
        f.frame(vec![Event::PointerMoved(target)]);
        assert!(
            matches!(f.app.drawing.hover_target(),Some(bondline_render::HitTarget::Atom{fragment_id,atom_id})
            if fragment_id.get()==1 && atom_id.get()==1)
        );
        let preview = f.frame(vec![rs_pointer(target, true, rs_alt())]);
        rs_same(&f, &before)?;
        rs_paint(&f, &preview, &expected)?;
        f.frame(vec![
            key(Key::Escape, true, Modifiers::NONE),
            key(Key::Escape, false, Modifiers::NONE),
            rs_pointer(target, false, rs_alt()),
        ]);
        rs_same(&f, &before)?;
        assert_eq!(fs::read(&path)?, raw);
        let committed = rs_click(
            &mut f,
            ROOT_POINT,
            rs_alt(),
            if index % 2 == 0 {
                rs_alt()
            } else {
                Modifiers::NONE
            },
        )?;
        rs_compare(&f, &expected)?;
        rs_edit(&f, &before, &path, &raw)?;
        rs_paint(&f, &committed, &expected)?;
        assert!(f.state()?["ui_atom_label"].is_null());
        let next = expected["pages"][0]["fragments"][0]["atoms"][6]["position"].clone();
        let shown = f.warm();
        let area = canvas(&shown)?;
        let at = rs_at(
            &f,
            area,
            [
                next["x_pt"].as_f64().ok_or("x")?,
                next["y_pt"].as_f64().ok_or("y")?,
            ],
        )?;
        f.frame(vec![Event::PointerMoved(at)]);
        assert!(
            matches!(f.app.drawing.hover_target(),Some(bondline_render::HitTarget::Atom{fragment_id,atom_id})
            if fragment_id.get()==1 && atom_id.get()==7),
            "new cycle remains logically pickable"
        );
        f.command("history.undo", json!({}))?;
        assert_eq!(f.state()?["document"], before["document"]);
        assert_eq!(f.state()?["file"], before["file"]);
        f.command("history.redo", json!({}))?;
        rs_compare(&f, &expected)?;
    }
    Ok(())
}

#[test]
fn preference_plain_vs_spiro_alt_force_and_press_release_attachment_latches_are_distinct()
-> TestResult {
    for (preference, press, release, sprout) in [
        (false, Modifiers::NONE, Modifiers::NONE, false),
        (true, Modifiers::NONE, Modifiers::NONE, true),
        (true, rs_alt(), Modifiers::NONE, true),
        (false, rs_alt(), Modifiers::NONE, true),
    ] {
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        let path = rs_seed(&mut f)?;
        let raw = fs::read(&path)?;
        rs_pref(&mut f, preference)?;
        rs_select(&mut f, "Cyclohexane")?;
        let before = f.state()?;
        let expected = if sprout {
            rs_expected(&before, "Cyclohexane", 6, false, rs_click_vector())?
        } else {
            rs_spiro(&before)?
        };
        let shown = rs_click(&mut f, ROOT_POINT, press, release)?;
        rs_compare(&f, &expected)?;
        rs_edit(&f, &before, &path, &raw)?;
        rs_paint(&f, &shown, &expected)?;
    }
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let path = rs_seed(&mut f)?;
    let raw = fs::read(&path)?;
    rs_select(&mut f, "Cyclohexane")?;
    let before = f.state()?;
    rs_click(&mut f, ROOT_POINT, Modifiers::NONE, rs_alt())?;
    rs_same(&f, &before)?;
    assert!(
        f.app.error.is_some(),
        "late Alt must refuse, not sprout or silently commit spiro"
    );
    assert_eq!(fs::read(path)?, raw);
    Ok(())
}

#[test]
fn named_shift_phase_is_latched_before_moves_and_releases_in_preference_sprout() -> TestResult {
    for kind in ["Benzene", "Cyclopentadiene"] {
        for (pressed, released, single) in [
            (Modifiers::NONE, Modifiers::SHIFT, false),
            (Modifiers::SHIFT, Modifiers::NONE, true),
        ] {
            let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
            let path = rs_seed(&mut f)?;
            let raw = fs::read(&path)?;
            rs_pref(&mut f, true)?;
            rs_select(&mut f, kind)?;
            let before = f.state()?;
            let n = if kind == "Benzene" { 6 } else { 5 };
            let expected = rs_expected(&before, kind, n, single, rs_click_vector())?;
            let shown = rs_click(&mut f, ROOT_POINT, pressed, released)?;
            rs_compare(&f, &expected)?;
            rs_edit(&f, &before, &path, &raw)?;
            rs_paint(&f, &shown, &expected)?;
        }
    }
    Ok(())
}

#[test]
fn alt_nonatom_and_unsupported_chords_refuse_visibly_without_atom_editor_or_free_fallback()
-> TestResult {
    let ctrl = Modifiers {
        ctrl: true,
        ..Modifiers::NONE
    };
    let command = Modifiers::COMMAND;
    for (kind, point, modifiers) in [
        ("Cyclohexane", [440.0, 220.0], rs_alt()),
        ("Cyclohexane", [240.0, 326.0], rs_alt()),
        ("Cyclohexane", ROOT_POINT, ctrl),
        ("Cyclohexane", ROOT_POINT, command),
        (
            "Benzene",
            ROOT_POINT,
            Modifiers {
                alt: true,
                shift: true,
                ..Modifiers::NONE
            },
        ),
        (
            "Cyclopentadiene",
            ROOT_POINT,
            Modifiers {
                alt: true,
                ctrl: true,
                ..Modifiers::NONE
            },
        ),
        ("Cyclohexane", ROOT_POINT, Modifiers::SHIFT),
        ("Cyclohexane chair 1", ROOT_POINT, Modifiers::SHIFT),
    ] {
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        let path = rs_seed(&mut f)?;
        let raw = fs::read(&path)?;
        rs_select(&mut f, kind)?;
        let before = f.state()?;
        rs_click(&mut f, point, modifiers, modifiers)?;
        let shown = f.warm();
        rs_same(&f, &before)?;
        assert_eq!(fs::read(path)?, raw);
        assert!(f.state()?["ui_atom_label"].is_null());
        assert!(f.node("Element", &[Role::TextInput]).is_err());
        let message = f
            .app
            .error
            .as_deref()
            .ok_or("unsupported ring input must be visibly reported")?;
        assert!(
            rs_text(&shown, message),
            "the refusal is painted in the inspector"
        );
    }
    Ok(())
}

#[test]
fn zero_return_focus_pointer_escape_and_hidden_preferences_never_reuse_an_old_sprout_preview()
-> TestResult {
    for reason in [
        "zero",
        "focus",
        "gone",
        "escape",
        "preference_aba",
        "modal_aba",
    ] {
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        let path = rs_seed(&mut f)?;
        let raw = fs::read(&path)?;
        rs_select(&mut f, "Cyclohexane")?;
        let area = canvas(&f.warm())?;
        let before = f.state()?;
        let a = rs_at(&f, area, ROOT_POINT)?;
        let b = rs_at(&f, area, [192.0, 374.0])?;
        f.frame(vec![Event::PointerMoved(a), rs_pointer(a, true, rs_alt())]);
        let preview = f.frame(vec![Event::PointerMoved(b)]);
        rs_same(&f, &before)?;
        let expected = rs_expected(&before, "Cyclohexane", 6, false, [-48.0, 24.0])?;
        rs_paint(&f, &preview, &expected)?;
        match reason {
            "zero" => {
                let shown = f.frame(vec![Event::PointerMoved(a)]);
                rs_paint(&f, &shown, &before["document"])?;
                f.frame(vec![rs_pointer(a, false, rs_alt())]);
            }
            "focus" => {
                f.frame(vec![
                    Event::WindowFocused(false),
                    rs_pointer(b, false, rs_alt()),
                    Event::WindowFocused(true),
                ]);
            }
            "gone" => {
                f.frame(vec![Event::PointerGone]);
                f.frame(vec![Event::PointerMoved(b), rs_pointer(b, false, rs_alt())]);
            }
            "escape" => {
                f.frame(vec![
                    key(Key::Escape, true, Modifiers::NONE),
                    key(Key::Escape, false, Modifiers::NONE),
                    rs_pointer(b, false, rs_alt()),
                ]);
            }
            "preference_aba" => {
                rs_pref(&mut f, true)?;
                rs_pref(&mut f, false)?;
                f.frame(vec![rs_pointer(b, false, rs_alt())]);
            }
            "modal_aba" => {
                let opened = f.command(
                    "ui.preferences",
                    json!({"action":"open","expected_revision":before["revision"]}),
                )?;
                let id = opened["ui_preferences"]["id"]
                    .as_u64()
                    .ok_or("modal request")?;
                f.command("ui.preferences",json!({"action":"cancel","expected_revision":before["revision"],"request_id":id}))?;
                f.frame(vec![rs_pointer(b, false, rs_alt())]);
            }
            _ => return Err("reason".into()),
        }
        rs_same(&f, &before)?;
        assert!(f.state()?["ui_file"]["request"].is_null());
        assert_eq!(fs::read(&path)?, raw);
        let expected = rs_expected(&before, "Cyclohexane", 6, false, rs_click_vector())?;
        rs_click(&mut f, ROOT_POINT, rs_alt(), Modifiers::NONE)?;
        rs_compare(&f, &expected)?;
        rs_edit(&f, &before, &path, &raw)?;
    }
    Ok(())
}

#[test]
fn preference_and_ring_alt_support_do_not_steal_existing_atom_edit_or_chain_repeat_owners()
-> TestResult {
    for selection in [false, true] {
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        let path = rs_seed(&mut f)?;
        let raw = fs::read(&path)?;
        rs_pref(&mut f, true)?;
        if selection {
            f.command(
                "ui.selection",
                json!({"action":"activate","expected_revision":f.state()?["revision"]}),
            )?;
        } else {
            rs_select(&mut f, "Single bond")?;
        }
        let before = f.state()?;
        rs_click(&mut f, ROOT_POINT, rs_alt(), rs_alt())?;
        rs_wait(&mut f, |f, _| f.node("Element", &[Role::TextInput]).is_ok())?;
        assert_eq!(f.state()?["ui_atom_label"]["atom_id"], 1);
        rs_same(&f, &before)?;
        let cancel = rs_button(&f, "Cancel")?;
        f.frame(vec![cancel]);
        rs_wait(&mut f, |f, _| {
            f.node("Element", &[Role::TextInput]).is_err()
        })?;
        rs_same(&f, &before)?;
        assert_eq!(fs::read(path)?, raw);
    }
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    rs_pref(&mut f, true)?;
    f.command("ui.view", json!({"action":"actual"}))?;
    rs_select(&mut f, "Acyclic chain")?;
    rs_click(&mut f, [144.0, 300.0], Modifiers::NONE, Modifiers::NONE)?;
    rs_wait(&mut f, |f, _| {
        f.node("Carbon count (including anchor)", &[Role::TextInput])
            .is_ok()
    })?;
    let (count, _) = f.node("Carbon count (including anchor)", &[Role::TextInput])?;
    f.frame(vec![Event::AccessKitActionRequest(ActionRequest {
        action: Action::SetValue,
        target: count,
        data: Some(egui::accesskit::ActionData::Value("4".into())),
    })]);
    rs_wait(&mut f, |f, _| {
        f.nodes.get(&count).and_then(|n| n.value()) == Some("4")
    })?;
    let add = rs_button(&f, "Add")?;
    f.frame(vec![add]);
    rs_wait(&mut f, |f, _| {
        f.node("Carbon count (including anchor)", &[Role::TextInput])
            .is_err()
    })?;
    let before = f.state()?;
    assert_eq!(before["undo_depth"], 1);
    assert_eq!(
        before["document"]["pages"][0]["fragments"][0]["atoms"]
            .as_array()
            .ok_or("chain atoms")?
            .len(),
        4
    );
    rs_click(&mut f, [144.0, 500.0], rs_alt(), rs_alt())?;
    assert!(
        f.node("Carbon count (including anchor)", &[Role::TextInput])
            .is_err()
    );
    assert!(f.node("Element", &[Role::TextInput]).is_err());
    let after = f.state()?;
    let fragments = after["document"]["pages"][0]["fragments"]
        .as_array()
        .ok_or("chains")?;
    assert_eq!(fragments.len(), 2);
    assert_eq!(after["undo_depth"], 2);
    assert_eq!(
        after["revision"],
        before["revision"].as_u64().ok_or("revision")? + 1
    );
    assert_eq!(fragments[0], before["document"]["pages"][0]["fragments"][0]);
    for fragment in fragments {
        let atoms = fragment["atoms"].as_array().ok_or("atoms")?;
        assert_eq!(atoms.len(), 4);
        assert_eq!(fragment["bonds"].as_array().ok_or("bonds")?.len(), 3);
        assert_eq!(
            atoms
                .iter()
                .map(|a| a["explicit_hydrogens"].as_u64().ok_or("H"))
                .collect::<Result<Vec<_>, _>>()?,
            vec![3, 2, 2, 3]
        );
    }
    assert_eq!(rs_inspect(&f)?["sprout_rings_when_clicking"], true);
    Ok(())
}
