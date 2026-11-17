//! Independent actual-App acceptance for the locked original native ring tool.
use super::{Fixture, TestResult};
use eframe::egui;

#[test]
fn cyclohexane_is_an_actionable_accessible_native_tool_without_authoring() -> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    f.warm();
    let before = f.state()?;
    let (id, rect) = f.node(
        "Cyclohexane",
        &[
            egui::accesskit::Role::Button,
            egui::accesskit::Role::CheckBox,
        ],
    )?;
    assert!(
        !f.nodes
            .get(&id)
            .ok_or("current Cyclohexane tool")?
            .is_disabled()
    );
    assert!(
        rect.is_positive()
            && egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1160.0, 800.0))
                .contains_rect(rect)
    );
    f.frame(vec![
        egui::Event::PointerMoved(rect.center()),
        egui::Event::PointerButton {
            pos: rect.center(),
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: egui::Modifiers::NONE,
        },
    ]);
    f.frame(vec![egui::Event::PointerButton {
        pos: rect.center(),
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: egui::Modifiers::NONE,
    }]);
    f.warm();
    let controls = f.command("ui.controls", serde_json::json!({}))?;
    let selected = controls["controls"]
        .as_array()
        .ok_or("controls")?
        .iter()
        .find(|row| row["id"] == "tool.ring.6")
        .ok_or("central Cyclohexane binding")?;
    assert_eq!(selected["selected"], true);
    assert_eq!(
        selected["command"],
        serde_json::json!({"command":"ui.ring_tool","params":{"size":6}})
    );
    let (id, _) = f.node(
        "Cyclohexane",
        &[
            egui::accesskit::Role::Button,
            egui::accesskit::Role::CheckBox,
        ],
    )?;
    assert_eq!(
        f.nodes.get(&id).ok_or("selected Cyclohexane")?.toggled(),
        Some(egui::accesskit::Toggled::True)
    );
    let after = f.state()?;
    for field in [
        "document",
        "revision",
        "undo_depth",
        "redo_depth",
        "source",
        "file",
        "close",
        "close_epoch",
    ] {
        assert_eq!(after[field], before[field], "tool choice authored {field}");
    }
    Ok(())
}

// First independently declared real gestures; palette draft exists, gesture routing does not.
use super::{button, canvas, projected};
use egui::{Event, Pos2, accesskit::Role};
use serde_json::{Value, json};
use std::{
    fs,
    path::PathBuf,
    time::{Duration, Instant},
};

fn ring_wait(
    f: &mut Fixture,
    ready: impl Fn(&Fixture, &egui::FullOutput) -> bool,
) -> TestResult<egui::FullOutput> {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let out = f.frame(vec![]);
        if ready(f, &out) {
            return Ok(out);
        }
        if Instant::now() >= deadline {
            return Err("native ring App state did not become ready".into());
        }
        std::thread::yield_now();
    }
}
fn ring_choose(f: &mut Fixture, name: &str) -> TestResult {
    ring_wait(f, |f, _| {
        f.node(name, &[Role::Button, Role::CheckBox]).is_ok()
    })?;
    let (_, r) = f.node(name, &[Role::Button, Role::CheckBox])?;
    f.frame(vec![
        Event::PointerMoved(r.center()),
        button(r.center(), true),
    ]);
    f.frame(vec![button(r.center(), false)]);
    Ok(())
}
fn ring_point(f: &mut Fixture, point: [f64; 2]) -> TestResult<Pos2> {
    let out = f.warm();
    let state = f.state()?;
    let page = &state["document"]["pages"][0]["size"];
    let view = f.view()?;
    let center = match (view["center_x_pt"].as_f64(), view["center_y_pt"].as_f64()) {
        (Some(x), Some(y)) => Some([x, y]),
        _ => None,
    };
    let p = projected(
        canvas(&out)?,
        [
            page["width_pt"].as_f64().ok_or("width")?,
            page["height_pt"].as_f64().ok_or("height")?,
        ],
        view["zoom_percent"].as_f64().ok_or("explicit camera")?,
        center,
        point,
    );
    assert!(
        canvas(&out)?.contains(p),
        "declared point must be on the actual canvas"
    );
    Ok(p)
}
fn ring_pointer(p: Pos2, pressed: bool, modifiers: egui::Modifiers) -> Event {
    Event::PointerButton {
        pos: p,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers,
    }
}
fn ring_click(f: &mut Fixture, point: [f64; 2]) -> TestResult<egui::FullOutput> {
    let p = ring_point(f, point)?;
    f.frame(vec![
        Event::PointerMoved(p),
        ring_pointer(p, true, egui::Modifiers::NONE),
    ]);
    Ok(f.frame(vec![ring_pointer(p, false, egui::Modifiers::NONE)]))
}
fn ring_seed(f: &mut Fixture, pair: Option<[[f64; 2]; 2]>) -> TestResult<PathBuf> {
    f.command(
        "page.set_size",
        json!({"page_id":1,"width_pt":288,"height_pt":288}),
    )?;
    if let Some([a, b]) = pair {
        f.command("bond.create",json!({"page_id":1,"begin_x_pt":a[0],"begin_y_pt":a[1],"end_x_pt":b[0],"end_y_pt":b[1]}))?;
    }
    let path = f.directory.path("native-ring-original-原.cdxml");
    f.command("document.save_cdxml", json!({"path":path}))?;
    f.command("document.open_cdxml", json!({"path":path}))?;
    f.command("ui.view", json!({"action":"actual"}))?;
    f.warm();
    assert_eq!(f.state()?["undo_depth"], 0);
    Ok(path)
}
fn ring_graph(points: &[[f64; 2]], hydrogens: &[u16], edges: &[[u32; 2]]) -> Value {
    assert_eq!(points.len(), hydrogens.len());
    json!({"id":1,"atoms":points.iter().zip(hydrogens).enumerate().map(|(i,(p,h))|json!({"id":i+1,"element":6,"position":{"x_pt":p[0],"y_pt":p[1]},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":h})).collect::<Vec<_>>(),
        "bonds":edges.iter().enumerate().map(|(i,[a,b])|json!({"id":i+1,"begin":a,"end":b,"order":"single"})).collect::<Vec<_>>()})
}
fn ring_hex(side: f64) -> Value {
    let q = 24.0 * 3.0_f64.sqrt();
    ring_graph(
        &[
            [120.0, 168.0],
            [120.0, 120.0],
            [120.0 + side * q, 96.0],
            [120.0 + side * 2.0 * q, 120.0],
            [120.0 + side * 2.0 * q, 168.0],
            [120.0 + side * q, 192.0],
        ],
        &[2; 6],
        &[[1, 2], [2, 3], [3, 4], [4, 5], [5, 6], [6, 1]],
    )
}
fn ring_assert_graph(f: &Fixture, expected: &Value) -> TestResult {
    let state = f.state()?;
    let pages = state["document"]["pages"].as_array().ok_or("pages")?;
    assert_eq!(pages.len(), 1);
    let fragments = pages[0]["fragments"].as_array().ok_or("fragments")?;
    assert_eq!(fragments.len(), 1);
    let mut actual = fragments[0].clone();
    let atoms = actual["atoms"].as_array_mut().ok_or("atoms")?;
    let wants = expected["atoms"].as_array().ok_or("expected atoms")?;
    assert_eq!(atoms.len(), wants.len());
    for (a, e) in atoms.iter_mut().zip(wants) {
        for axis in ["x_pt", "y_pt"] {
            let x = a["position"][axis].as_f64().ok_or("actual position")?;
            let y = e["position"][axis].as_f64().ok_or("literal position")?;
            assert!((x - y).abs() <= 1e-9, "{axis}: actual{x} != literal{y}");
        }
        a["position"] = e["position"].clone();
    }
    assert_eq!(&actual, expected);
    Ok(())
}
fn ring_one_edit(f: &Fixture, before: &Value, path: &std::path::Path, raw: &[u8]) -> TestResult {
    let after = f.state()?;
    assert_eq!(
        after["revision"],
        before["revision"].as_u64().ok_or("revision")? + 1
    );
    assert_eq!(
        after["undo_depth"],
        before["undo_depth"].as_u64().ok_or("undo")? + 1
    );
    assert_eq!(after["redo_depth"], 0);
    assert_eq!(after["source"]["format"], "cdxml");
    assert_eq!(after["source"]["modified"], true);
    assert_eq!(after["file"]["dirty"], true);
    assert_eq!(after["file"]["path"], before["file"]["path"]);
    assert_eq!(
        after["document"]["pages"][0]["size"],
        before["document"]["pages"][0]["size"]
    );
    assert_eq!(fs::read(path)?, raw);
    assert!(f.app.error.is_none(), "{:?}", f.app.error);
    Ok(())
}

#[test]
fn native_free_click_creates_the_locked_literal_six_ring_in_one_edit() -> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let path = ring_seed(&mut f, None)?;
    let raw = fs::read(&path)?;
    ring_choose(&mut f, "Cyclohexane")?;
    let before = f.state()?;
    ring_click(&mut f, [120.0, 168.0])?;
    f.warm();
    ring_assert_graph(&f, &ring_hex(1.0))?;
    ring_one_edit(&f, &before, &path, &raw)
}

#[test]
fn native_shared_terminal_atom_click_adds_a_real_methylcyclohexane_without_fallback() -> TestResult
{
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let path = ring_seed(&mut f, Some([[144.0, 144.0], [192.0, 144.0]]))?;
    let raw = fs::read(&path)?;
    ring_choose(&mut f, "Cyclohexane")?;
    let before = f.state()?;
    ring_click(&mut f, [144.0, 144.0])?;
    f.warm();
    let q = 24.0 * 3.0_f64.sqrt();
    let expected = ring_graph(
        &[
            [144.0, 144.0],
            [192.0, 144.0],
            [120.0, 144.0 + q],
            [72.0, 144.0 + q],
            [48.0, 144.0],
            [72.0, 144.0 - q],
            [120.0, 144.0 - q],
        ],
        &[1, 3, 2, 2, 2, 2, 2],
        &[[1, 2], [1, 3], [3, 4], [4, 5], [5, 6], [6, 7], [7, 1]],
    );
    ring_assert_graph(&f, &expected)?;
    ring_one_edit(&f, &before, &path, &raw)
}

#[test]
fn native_shared_bond_click_preserves_the_authored_edge_and_closes_a_real_ring() -> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let path = ring_seed(&mut f, Some([[120.0, 168.0], [120.0, 120.0]]))?;
    let raw = fs::read(&path)?;
    ring_choose(&mut f, "Cyclohexane")?;
    let before = f.state()?;
    ring_click(&mut f, [120.0, 144.0])?;
    f.warm();
    ring_assert_graph(&f, &ring_hex(1.0))?;
    ring_one_edit(&f, &before, &path, &raw)
}

// Independent expansion after the first core/palette/gesture draft; earlier declarations stay exact.
use super::{finish_actual_file_workflow, key};
use egui::{Key, Modifiers};
use std::{
    path::Path,
    process::{Command as Process, Stdio},
    sync::Arc,
};

fn ring_keys(code: Key, m: Modifiers) -> Vec<Event> {
    vec![key(code, true, m), key(code, false, m)]
}
fn ring_shortcut(f: &mut Fixture, code: Key, shift: bool) -> egui::FullOutput {
    f.frame(ring_keys(
        code,
        Modifiers {
            shift,
            ..Modifiers::COMMAND
        },
    ))
}
fn ring_authored_same(f: &Fixture, before: &Value) -> TestResult {
    let state = f.state()?;
    for field in [
        "document",
        "revision",
        "undo_depth",
        "redo_depth",
        "source",
        "file",
        "close",
        "close_epoch",
    ] {
        assert_eq!(
            state[field], before[field],
            "native ring input changed {field}"
        );
    }
    Ok(())
}
fn ring_model(graphs: &[Value], size: [f64; 2]) -> TestResult<bondline_model::Document> {
    let mut document = bondline_model::Document::default();
    let page = bondline_model::PageId::new(1).ok_or("page")?;
    document.set_page_size(page, bondline_model::PageSize::new(size[0], size[1])?)?;
    let fragments = graphs
        .iter()
        .map(|g| {
            let id = u32::try_from(g["id"].as_u64().ok_or("fragment ID")?)?;
            Ok(bondline_model::Fragment::new(
                bondline_model::FragmentId::new(id).ok_or("fragment")?,
                serde_json::from_value::<Vec<bondline_model::Atom>>(g["atoms"].clone())?,
                serde_json::from_value::<Vec<bondline_model::Bond>>(g["bonds"].clone())?,
            )?)
        })
        .collect::<TestResult<Vec<_>>>()?;
    document.append_fragments(page, fragments)?;
    Ok(document)
}
fn ring_literal_source(f: &mut Fixture, graphs: &[Value], size: [f64; 2]) -> TestResult<PathBuf> {
    let mut stored = bondline_commands::Engine::default()
        .stored_document()?
        .clone();
    *stored.model_mut() = ring_model(graphs, size)?;
    {
        let mut engine = f.app.engine.lock().map_err(|_| "engine")?;
        *engine = bondline_commands::Engine::from_document(stored);
        engine.set_ui_file_host(true);
    }
    let path = f.directory.path("literal-ring-source-化学.cdxml");
    f.command("document.save_cdxml", json!({"path":path}))?;
    f.command("document.open_cdxml", json!({"path":path}))?;
    f.command("ui.view", json!({"action":"actual"}))?;
    f.warm();
    Ok(path)
}
fn ring_fused() -> Value {
    let q = 24.0 * 3.0_f64.sqrt();
    ring_graph(
        &[
            [120.0, 168.0],
            [120.0, 120.0],
            [120.0 + q, 96.0],
            [120.0 + 2.0 * q, 120.0],
            [120.0 + 2.0 * q, 168.0],
            [120.0 + q, 192.0],
            [120.0 - q, 96.0],
            [120.0 - 2.0 * q, 120.0],
            [120.0 - 2.0 * q, 168.0],
            [120.0 - q, 192.0],
        ],
        &[1, 1, 2, 2, 2, 2, 2, 2, 2, 2],
        &[
            [1, 2],
            [2, 3],
            [3, 4],
            [4, 5],
            [5, 6],
            [6, 1],
            [2, 7],
            [7, 8],
            [8, 9],
            [9, 10],
            [10, 1],
        ],
    )
}
fn ring_spiro() -> Value {
    let q = 24.0 * 3.0_f64.sqrt();
    ring_graph(
        &[
            [120.0, 168.0],
            [120.0, 120.0],
            [120.0 + q, 96.0],
            [120.0 + 2.0 * q, 120.0],
            [120.0 + 2.0 * q, 168.0],
            [120.0 + q, 192.0],
            [120.0, 216.0],
            [120.0 - q, 240.0],
            [120.0 - 2.0 * q, 216.0],
            [120.0 - 2.0 * q, 168.0],
            [120.0 - q, 144.0],
        ],
        &[0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2],
        &[
            [1, 2],
            [2, 3],
            [3, 4],
            [4, 5],
            [5, 6],
            [6, 1],
            [1, 7],
            [7, 8],
            [8, 9],
            [9, 10],
            [10, 11],
            [11, 1],
        ],
    )
}
fn ring_assert_old_prefix(f: &Fixture, before: &Value, hydrogens: &[(u32, u16)]) -> TestResult {
    let state = f.state()?;
    let old = &before["document"]["pages"][0]["fragments"][0];
    let new = &state["document"]["pages"][0]["fragments"][0];
    let a = old["atoms"].as_array().ok_or("old atoms")?;
    let b = new["atoms"].as_array().ok_or("new atoms")?;
    for (i, atom) in a.iter().enumerate() {
        let mut expected = atom.clone();
        for &(id, h) in hydrogens {
            if expected["id"] == id {
                expected["explicit_hydrogens"] = json!(h);
            }
        }
        assert_eq!(
            b[i], expected,
            "old shared/unrelated atom data must remain exact"
        );
    }
    let old_bonds = old["bonds"].as_array().ok_or("old bonds")?;
    assert_eq!(
        &new["bonds"].as_array().ok_or("new bonds")?[..old_bonds.len()],
        old_bonds.as_slice()
    );
    assert_eq!(new["id"], old["id"]);
    Ok(())
}
fn ring_compare_document(actual: &Value, expected: &Value) -> TestResult {
    let mut normalized = actual.clone();
    let pages = normalized["pages"].as_array_mut().ok_or("actual pages")?;
    let expected_pages = expected["pages"].as_array().ok_or("expected pages")?;
    assert_eq!(pages.len(), expected_pages.len());
    for (page, want) in pages.iter_mut().zip(expected_pages) {
        let fragments = page["fragments"].as_array_mut().ok_or("fragments")?;
        let wants = want["fragments"].as_array().ok_or("expected fragments")?;
        assert_eq!(fragments.len(), wants.len());
        for (fragment, want) in fragments.iter_mut().zip(wants) {
            let atoms = fragment["atoms"].as_array_mut().ok_or("atoms")?;
            let wants = want["atoms"].as_array().ok_or("expected atoms")?;
            assert_eq!(atoms.len(), wants.len());
            for (a, e) in atoms.iter_mut().zip(wants) {
                for axis in ["x_pt", "y_pt"] {
                    assert!(
                        (a["position"][axis].as_f64().ok_or("position")?
                            - e["position"][axis].as_f64().ok_or("expected position")?)
                        .abs()
                            <= 1e-9
                    );
                }
                a["position"] = e["position"].clone();
            }
        }
    }
    assert_eq!(&normalized, expected);
    Ok(())
}
fn ring_ink_compare(
    mut actual: Vec<Vec<f32>>,
    mut expected: Vec<Vec<f32>>,
    density: f32,
) -> TestResult {
    let order = |a: &Vec<f32>, b: &Vec<f32>| {
        a.iter()
            .zip(b)
            .map(|(a, b)| a.total_cmp(b))
            .find(|o| !o.is_eq())
            .unwrap_or_else(|| a.len().cmp(&b.len()))
    };
    actual.sort_by(order);
    expected.sort_by(order);
    assert_eq!(
        actual.len(),
        expected.len(),
        "ordinary candidate primitive count"
    );
    for (a, e) in actual.iter().zip(expected) {
        assert_eq!(a.len(), e.len());
        for (a, e) in a.iter().zip(e) {
            assert!(
                (a - e).abs() * density <= 1.0 / 256.0,
                "candidate device geometry{a} != literal{e}"
            );
        }
    }
    Ok(())
}
fn ring_paint_matches(
    f: &Fixture,
    shown: &egui::FullOutput,
    graphs: &[Value],
    size: [f64; 2],
) -> TestResult {
    let area = canvas(shown)?;
    let density = f.context.pixels_per_point();
    assert_eq!(density, f.dpi);
    let page = bondline_model::PageId::new(1).ok_or("page")?;
    let logical = f.app.engine.lock().map_err(|_| "engine")?.page_view(
        area.width().floor() as u32,
        area.height().floor() as u32,
        page,
    )?;
    let (origin, view) = if density == 1.0 {
        (area.min, logical)
    } else {
        let device = [
            f64::from(area.min.x) * f64::from(density),
            f64::from(area.min.y) * f64::from(density),
        ];
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
    *stored.model_mut() = ring_model(graphs, size)?;
    let reference = bondline_commands::Engine::from_document(stored);
    let graph = reference.drawing_scene_with_view(
        &view,
        page,
        &bondline_render::Cancellation::default(),
    )?;
    let mut rectangles = Vec::new();
    let mut quads = Vec::new();
    for primitive in &graph.scene().primitives {
        match primitive {
            bondline_render::Primitive::Rectangle(r) if r.color == [0, 0, 0, 255] => rectangles
                .push(vec![
                    origin.x + r.x as f32 / density,
                    origin.y + r.y as f32 / density,
                    origin.x + (r.x + r.width) as f32 / density,
                    origin.y + (r.y + r.height) as f32 / density,
                ]),
            bondline_render::Primitive::Stroke(s) if s.color == [0, 0, 0, 255] => quads.push(
                s.corners()?
                    .into_iter()
                    .flat_map(|[x, y]| [origin.x + x / density, origin.y + y / density])
                    .collect(),
            ),
            _ => {}
        }
    }
    assert!(
        !rectangles.is_empty(),
        "literal H/C glyph ink is required, not a polygon outline"
    );
    assert!(!quads.is_empty());
    let actual_rectangles = shown
        .shapes
        .iter()
        .filter_map(|s| match &s.shape {
            egui::Shape::Rect(r)
                if r.fill == egui::Color32::BLACK && area.contains_rect(r.rect) =>
            {
                Some(vec![
                    r.rect.left(),
                    r.rect.top(),
                    r.rect.right(),
                    r.rect.bottom(),
                ])
            }
            _ => None,
        })
        .collect();
    let actual_quads = shown
        .shapes
        .iter()
        .filter_map(|s| match &s.shape {
            egui::Shape::Path(p)
                if p.fill == egui::Color32::BLACK
                    && p.points.len() == 4
                    && p.points.iter().all(|v| area.contains(*v)) =>
            {
                Some(p.points.iter().flat_map(|p| [p.x, p.y]).collect())
            }
            _ => None,
        })
        .collect();
    ring_ink_compare(actual_rectangles, rectangles, density)?;
    ring_ink_compare(actual_quads, quads, density)
}
fn ring_consume(path: &Path, document: &Value, formulas: &[&str]) -> TestResult {
    // Compare unchanged independent Expat output in Python's exact JSON float bridge.
    // No Rust parsing of its numeric tokens and no tolerance/field relaxation.
    let mut child=Process::new(std::env::var_os("BONDLINE_PYTHON").ok_or("pinned external Python")?).args(["-B","-c",r#"
from pathlib import Path
from rdkit import Chem,rdBase
from rdkit.Chem import rdMolDescriptors
import json,subprocess,sys
assert rdBase.rdkitVersion=='2026.09.2'
expected=json.load(sys.stdin)
source=Path(sys.argv[1]).read_bytes()
raw=subprocess.run([sys.executable,'-B',sys.argv[2]],input=source,capture_output=True,check=True)
assert json.loads(raw.stdout)==expected,'exact complete independent Expat graph differs from serialized model'
ms=Chem.MolsFromCDXML(source.decode('utf-8'))
formulas=json.loads(sys.argv[3]);assert len(ms)==len(expected)==len(formulas)
for m,g,formula in zip(ms,expected,formulas):
 assert m.GetNumAtoms()==len(g['atoms']) and m.GetNumBonds()==len(g['bonds'])
 assert [a.GetAtomicNum() for a in m.GetAtoms()]==[a['element'] for a in g['atoms']]
 assert [a.GetTotalNumHs() for a in m.GetAtoms()]==[a['explicit_hydrogens'] for a in g['atoms']]
 assert all(a.GetFormalCharge()==0 and a.GetIsotope()==0 and a.GetNumRadicalElectrons()==0 for a in m.GetAtoms())
 assert all(b.GetBondTypeAsDouble()==1 for b in m.GetBonds())
 assert rdMolDescriptors.CalcMolFormula(m)==formula
"#]).arg(path).arg(concat!(env!("CARGO_MANIFEST_DIR"),"/../../xtask/oracles/cdxml_graph.py"))
        .arg(serde_json::to_string(formulas)?).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
    serde_json::to_writer(
        child.stdin.take().ok_or("expected graph stdin")?,
        &document["pages"][0]["fragments"],
    )?;
    let out = child.wait_with_output()?;
    assert!(
        out.status.success(),
        "actual Unicode ring file raw graph/chemistry: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    Ok(())
}

#[test]
fn free_drag_paints_the_complete_literal_candidate_before_commit_and_saves_real_unicode_history()
-> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let path = ring_seed(&mut f, None)?;
    let raw = fs::read(&path)?;
    ring_choose(&mut f, "Cyclohexane")?;
    let before = f.state()?;
    let q = 24.0 * 3.0_f64.sqrt();
    let expected = ring_graph(
        &[
            [120.0, 120.0],
            [168.0, 120.0],
            [192.0, 120.0 + q],
            [168.0, 120.0 + 2.0 * q],
            [120.0, 120.0 + 2.0 * q],
            [96.0, 120.0 + q],
        ],
        &[2; 6],
        &[[1, 2], [2, 3], [3, 4], [4, 5], [5, 6], [6, 1]],
    );
    let a = ring_point(&mut f, [120.0, 120.0])?;
    let b = ring_point(&mut f, [168.0, 120.0])?;
    let (stamp, model) = {
        let e = f.app.engine.lock().map_err(|_| "engine")?;
        (e.capture_stamp(), e.stored_document()?.model_snapshot())
    };
    f.frame(vec![
        Event::PointerMoved(a),
        ring_pointer(a, true, Modifiers::NONE),
    ]);
    let held = f.frame(vec![Event::PointerMoved(b)]);
    assert_eq!(f.state()?, before);
    assert_eq!(fs::read(&path)?, raw);
    {
        let e = f.app.engine.lock().map_err(|_| "engine")?;
        assert_eq!(e.capture_stamp(), stamp);
        assert!(Arc::ptr_eq(&model, &e.stored_document()?.model_snapshot()));
    }
    ring_paint_matches(&f, &held, std::slice::from_ref(&expected), [288.0, 288.0])?;
    f.frame(vec![ring_pointer(b, false, Modifiers::NONE)]);
    let shown = f.warm();
    ring_assert_graph(&f, &expected)?;
    ring_one_edit(&f, &before, &path, &raw)?;
    ring_paint_matches(&f, &shown, std::slice::from_ref(&expected), [288.0, 288.0])?;
    let committed = f.state()?;
    ring_shortcut(&mut f, Key::S, false);
    finish_actual_file_workflow(&mut f)?;
    assert_eq!(f.state()?["file"]["dirty"], false);
    assert_eq!(
        f.state()?["file"]["path"],
        fs::canonicalize(&path)?.to_str().ok_or("Unicode path")?
    );
    ring_consume(&path, &committed["document"], &["C6H12"])?;
    ring_shortcut(&mut f, Key::Z, false);
    assert_eq!(f.state()?["document"], before["document"]);
    assert_eq!(f.state()?["redo_depth"], 1);
    assert_eq!(f.state()?["file"]["dirty"], true);
    ring_shortcut(&mut f, Key::Z, true);
    ring_compare_document(&f.state()?["document"], &committed["document"])?;
    assert_eq!(f.state()?["file"]["dirty"], false);
    f.command("document.open_cdxml", json!({"path":path}))?;
    f.warm();
    ring_compare_document(&f.state()?["document"], &committed["document"])?;
    assert_eq!(f.state()?["undo_depth"], 0);
    Ok(())
}

#[test]
fn actual_isolated_and_existing_ring_atoms_keep_shared_data_and_create_the_locked_spiro_pose()
-> TestResult {
    let isolated = ring_graph(&[[120.0, 168.0]], &[4], &[]);
    for (name, source, expected, h, formula) in [
        ("isolated", isolated, ring_hex(1.0), 2, "C6H12"),
        ("spiro", ring_hex(1.0), ring_spiro(), 0, "C11H20"),
    ] {
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        let path = ring_literal_source(&mut f, std::slice::from_ref(&source), [288.0, 288.0])?;
        let raw = fs::read(&path)?;
        ring_choose(&mut f, "Cyclohexane")?;
        let before = f.state()?;
        ring_click(&mut f, [120.0, 168.0])?;
        let shown = f.warm();
        ring_assert_graph(&f, &expected)?;
        ring_assert_old_prefix(&f, &before, &[(1, h)])?;
        ring_one_edit(&f, &before, &path, &raw)?;
        ring_paint_matches(&f, &shown, std::slice::from_ref(&expected), [288.0, 288.0])?;
        ring_shortcut(&mut f, Key::S, false);
        finish_actual_file_workflow(&mut f)?;
        ring_consume(&path, &f.state()?["document"], &[formula])?;
        assert!(f.app.error.is_none(), "{name}: {:?}", f.app.error);
    }
    Ok(())
}

#[test]
fn actual_hex_bond_click_chooses_the_outside_and_explicit_bond_drags_keep_both_anchors()
-> TestResult {
    for direction in ["fused_click", "positive_drag", "negative_drag"] {
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        let source = if direction == "fused_click" {
            ring_hex(1.0)
        } else {
            ring_graph(&[[120.0, 168.0], [120.0, 120.0]], &[3, 3], &[[1, 2]])
        };
        let path = ring_literal_source(&mut f, std::slice::from_ref(&source), [288.0, 288.0])?;
        let raw = fs::read(&path)?;
        ring_choose(&mut f, "Cyclohexane")?;
        let before = f.state()?;
        let expected = match direction {
            "fused_click" => {
                ring_click(&mut f, [120.0, 144.0])?;
                ring_fused()
            }
            _ => {
                let a = ring_point(&mut f, [120.0, 144.0])?;
                let b = ring_point(
                    &mut f,
                    [
                        if direction == "positive_drag" {
                            150.0
                        } else {
                            90.0
                        },
                        144.0,
                    ],
                )?;
                f.frame(vec![
                    Event::PointerMoved(a),
                    ring_pointer(a, true, Modifiers::NONE),
                ]);
                let held = f.frame(vec![Event::PointerMoved(b)]);
                assert_eq!(f.state()?, before);
                let expected = ring_hex(if direction == "positive_drag" {
                    1.0
                } else {
                    -1.0
                });
                ring_paint_matches(&f, &held, std::slice::from_ref(&expected), [288.0, 288.0])?;
                f.frame(vec![ring_pointer(b, false, Modifiers::NONE)]);
                expected
            }
        };
        let shown = f.warm();
        ring_assert_graph(&f, &expected)?;
        ring_assert_old_prefix(
            &f,
            &before,
            &[
                (1, if direction == "fused_click" { 1 } else { 2 }),
                (2, if direction == "fused_click" { 1 } else { 2 }),
            ],
        )?;
        ring_one_edit(&f, &before, &path, &raw)?;
        ring_paint_matches(&f, &shown, std::slice::from_ref(&expected), [288.0, 288.0])?;
        ring_shortcut(&mut f, Key::S, false);
        finish_actual_file_workflow(&mut f)?;
        ring_consume(
            &path,
            &f.state()?["document"],
            &[if direction == "fused_click" {
                "C10H18"
            } else {
                "C6H12"
            }],
        )?;
    }
    Ok(())
}

#[test]
fn a_new_candidate_checks_logical_labels_and_uses_real_physical_vector_geometry() -> TestResult {
    for density in [2.0, 4.0] {
        let mut f = Fixture::new(1160.0, 800.0, density)?;
        ring_seed(&mut f, None)?;
        ring_choose(&mut f, "Cyclohexane")?;
        let before = f.state()?;
        let a = ring_point(&mut f, [120.0, 168.0])?;
        let held = f.frame(vec![
            Event::PointerMoved(a),
            ring_pointer(a, true, Modifiers::NONE),
        ]);
        assert_eq!(held.pixels_per_point, density);
        assert_eq!(f.state()?, before);
        let expected = ring_hex(1.0);
        ring_paint_matches(&f, &held, std::slice::from_ref(&expected), [288.0, 288.0])?;
        if density == 4.0 {
            let area = canvas(&held)?;
            assert!(
                area.width() * density > 2048.0,
                "positive control must exceed readback width without allocating a bitmap"
            );
        }
        f.frame(vec![ring_pointer(a, false, Modifiers::NONE)]);
        ring_assert_graph(&f, &expected)?;
    }
    let mut f = Fixture::new(1160.0, 800.0, 2.0)?;
    let path = ring_seed(&mut f, None)?;
    let raw = fs::read(&path)?;
    f.command("ui.view", json!({"action":"set_zoom","percent":50}))?;
    f.warm();
    ring_choose(&mut f, "Cyclohexane")?;
    let before = f.state()?;
    let a = ring_point(&mut f, [120.0, 168.0])?;
    f.frame(vec![
        Event::PointerMoved(a),
        ring_pointer(a, true, Modifiers::NONE),
    ]);
    let shown = f.frame(vec![ring_pointer(a, false, Modifiers::NONE)]);
    let guard = bondline_render::Error::LabelScale.to_string();
    assert!(
        f.app.error.as_ref().is_some_and(|s| s.contains(&guard))
            || shown.shapes.iter().any(
                |s| matches!(&s.shape,egui::Shape::Text(t)if t.galley.text().contains(&guard))
            ),
        "new logical6-em H labels must refuse despite physical12-em at2x"
    );
    assert_eq!(f.state()?, before);
    assert_eq!(fs::read(&path)?, raw);
    Ok(())
}

// Independent lifecycle/chronology expansion after the first shared native gesture draft.
fn ring_activation(f: &Fixture, name: &str) -> TestResult<Event> {
    let (id, _) = f.node(name, &[Role::Button, Role::CheckBox])?;
    Ok(Event::AccessKitActionRequest(
        egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::Default,
            target: id,
            data: None,
        },
    ))
}
fn ring_selected(f: &Fixture) -> TestResult<Vec<String>> {
    let c = f.command("ui.controls", json!({}))?;
    Ok(c["controls"]
        .as_array()
        .ok_or("controls")?
        .iter()
        .filter(|c| c["menu"] == "Tools" && c["selected"] == true)
        .map(|c| c["id"].as_str().unwrap_or("").to_owned())
        .collect())
}
fn ring_shift(
    mut graph: Value,
    fragment: u32,
    atom_offset: u32,
    bond_offset: u32,
    delta: [f64; 2],
) -> TestResult<Value> {
    graph["id"] = json!(fragment);
    for a in graph["atoms"].as_array_mut().ok_or("atoms")? {
        a["id"] = json!(a["id"].as_u64().ok_or("ID")? + u64::from(atom_offset));
        for (axis, d) in ["x_pt", "y_pt"].into_iter().zip(delta) {
            a["position"][axis] = json!(a["position"][axis].as_f64().ok_or("position")? + d);
        }
    }
    for b in graph["bonds"].as_array_mut().ok_or("bonds")? {
        b["id"] = json!(b["id"].as_u64().ok_or("ID")? + u64::from(bond_offset));
        for endpoint in ["begin", "end"] {
            b[endpoint] = json!(b[endpoint].as_u64().ok_or("endpoint")? + u64::from(atom_offset));
        }
    }
    Ok(graph)
}

#[test]
fn all_six_bindings_use_one_effective_mode_and_same_ring_noop_preserves_an_armed_gesture()
-> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let path = ring_seed(&mut f, Some([[120.0, 168.0], [120.0, 120.0]]))?;
    let raw = fs::read(&path)?;
    let before = f.state()?;
    ring_choose(&mut f, "Marquee selection")?;
    ring_click(&mut f, [120.0, 168.0])?;
    assert_eq!(f.state()?["ui_selection"]["atom_ids"], json!([1]));
    for (n, name) in [
        (3, "Cyclopropane"),
        (4, "Cyclobutane"),
        (5, "Cyclopentane"),
        (6, "Cyclohexane"),
        (7, "Cycloheptane"),
        (8, "Cyclooctane"),
    ] {
        ring_choose(&mut f, name)?;
        f.warm();
        assert_eq!(ring_selected(&f)?, vec![format!("tool.ring.{n}")]);
        let controls = f.command("ui.controls", json!({}))?;
        let rows = controls["controls"].as_array().ok_or("controls")?;
        assert_eq!(rows.len(), 36);
        let row = rows
            .iter()
            .find(|r| r["id"] == format!("tool.ring.{n}"))
            .ok_or("ring binding")?;
        assert_eq!(
            row["command"],
            json!({"command":"ui.ring_tool","params":{"size":n}})
        );
        assert_eq!(row["shortcut"], Value::Null);
        assert_eq!(f.state()?["ui_selection"]["active"], false);
        assert_eq!(f.state()?["ui_selection"]["atom_ids"], json!([]));
    }
    let epoch = f.app.engine.lock().map_err(|_| "engine")?.bond_tool().epoch;
    ring_choose(&mut f, "Single bond")?;
    f.warm();
    assert_eq!(
        f.app.engine.lock().map_err(|_| "engine")?.bond_tool().epoch,
        epoch + 1,
        "remembered Single is a real Ring-to-Bond switch"
    );
    assert_eq!(ring_selected(&f)?, vec!["tool.bond.single"]);
    ring_authored_same(&f, &before)?;
    assert_eq!(fs::read(&path)?, raw);
    // Separate empty native page: top horizontal first edge for a non-six click.
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let path = ring_seed(&mut f, None)?;
    let raw = fs::read(&path)?;
    ring_choose(&mut f, "Cyclobutane")?;
    let before = f.state()?;
    ring_click(&mut f, [96.0, 96.0])?;
    f.warm();
    let square = ring_graph(
        &[[96.0, 96.0], [144.0, 96.0], [144.0, 144.0], [96.0, 144.0]],
        &[2; 4],
        &[[1, 2], [2, 3], [3, 4], [4, 1]],
    );
    ring_assert_graph(&f, &square)?;
    ring_one_edit(&f, &before, &path, &raw)?;
    // A no-op selected-tool AX activation between press/release cannot cancel the gesture.
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let path = ring_seed(&mut f, None)?;
    let raw = fs::read(&path)?;
    ring_choose(&mut f, "Cyclohexane")?;
    let before = f.state()?;
    let epoch = f.app.engine.lock().map_err(|_| "engine")?.bond_tool().epoch;
    let a = ring_point(&mut f, [120.0, 168.0])?;
    f.frame(vec![
        Event::PointerMoved(a),
        ring_pointer(a, true, Modifiers::NONE),
    ]);
    let same = ring_activation(&f, "Cyclohexane")?;
    f.frame(vec![same, ring_pointer(a, false, Modifiers::NONE)]);
    f.warm();
    ring_assert_graph(&f, &ring_hex(1.0))?;
    ring_one_edit(&f, &before, &path, &raw)?;
    assert_eq!(
        f.app.engine.lock().map_err(|_| "engine")?.bond_tool().epoch,
        epoch
    );
    Ok(())
}

#[test]
fn ring_cancellation_and_source_contexts_never_revive_an_old_preview_or_guess_a_click() -> TestResult
{
    for change in [
        "escape",
        "focus",
        "pointer_gone",
        "outside",
        "view_aba",
        "ring_size_aba",
        "bond_ring_aba",
        "open_aba",
        "file_aba",
        "close_aba",
        "dpi",
        "out_and_back",
        "raw_sdf",
        "raw_smiles",
        "retained_smiles",
    ] {
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        let path = ring_seed(&mut f, None)?;
        let raw = fs::read(&path)?;
        ring_choose(&mut f, "Cyclohexane")?;
        let a = ring_point(&mut f, [120.0, 120.0])?;
        let b = ring_point(&mut f, [168.0, 120.0])?;
        let area = canvas(&f.warm())?;
        let before = f.state()?;
        f.frame(vec![
            Event::PointerMoved(a),
            ring_pointer(a, true, Modifiers::NONE),
        ]);
        f.frame(vec![Event::PointerMoved(b)]);
        assert_eq!(f.state()?, before);
        let mut release = Vec::new();
        let mut end = b;
        let mut replacement = None;
        match change {
            "escape" => release.extend(ring_keys(Key::Escape, Modifiers::NONE)),
            "focus" => {
                f.frame(vec![Event::WindowFocused(false)]);
                release.push(Event::WindowFocused(true));
            }
            "pointer_gone" => {
                f.frame(vec![Event::PointerGone]);
            }
            "outside" => {
                let outside = egui::pos2(area.left() - 12.0, area.center().y);
                assert!(!area.contains(outside));
                assert!(outside.x > 0.0);
                f.frame(vec![Event::PointerMoved(outside)]);
                f.frame(vec![Event::PointerMoved(b)]);
            }
            "view_aba" => {
                f.command("ui.view", json!({"action":"set_zoom","percent":125}))?;
                f.command("ui.view", json!({"action":"set_zoom","percent":100}))?;
            }
            "ring_size_aba" => {
                f.command("ui.ring_tool", json!({"size":7}))?;
                f.command("ui.ring_tool", json!({"size":6}))?;
            }
            "bond_ring_aba" => {
                f.command("ui.bond_tool", json!({"order":"double"}))?;
                f.command("ui.ring_tool", json!({"size":6}))?;
            }
            "open_aba" => {
                f.command("document.open_cdxml", json!({"path":path}))?;
            }
            "file_aba" => {
                let other = f.directory.path("association-B-λ.cdxml");
                f.command("document.save_cdxml", json!({"path":other}))?;
                f.command("document.save_cdxml", json!({"path":path,"overwrite":true}))?;
                assert_eq!(fs::read(other)?, raw);
            }
            "close_aba" => {
                f.command(
                    "document.close",
                    json!({"expected_revision":f.state()?["revision"],"action":"request"}),
                )?;
                f.command(
                    "document.close",
                    json!({"expected_revision":f.state()?["revision"],"action":"cancel"}),
                )?;
            }
            "dpi" => {
                f.dpi = 2.0;
            }
            "out_and_back" => {
                f.frame(vec![Event::PointerMoved(a)]);
                end = a;
            }
            "raw_sdf" => {
                let input = f.directory.path("replacement-原.sdf");
                let bytes = include_bytes!("../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf");
                fs::write(&input, bytes)?;
                f.command("document.open_sdf", json!({"path":input}))?;
                replacement = Some((input, bytes.to_vec(), 15));
            }
            "raw_smiles" | "retained_smiles" => {
                let input = f.directory.path("replacement-原.smi");
                let bytes = b"CC first\r\nCC second\n";
                fs::write(&input, bytes)?;
                f.command("document.open_smiles", json!({"path":input}))?;
                if change == "retained_smiles" {
                    f.command("document.import_smiles_records",json!({"expected_revision":f.state()?["revision"],"selection":{"mode":"all"},"bond_length_pt":64,"origin_x_pt":72,"origin_y_pt":72,"page_width_pt":288,"page_height_pt":288}))?;
                }
                replacement = Some((
                    input,
                    bytes.to_vec(),
                    if change == "retained_smiles" { 21 } else { 11 },
                ));
            }
            _ => return Err("undeclared ring cancellation".into()),
        }
        let after_context = f.state()?;
        release.extend([
            Event::PointerMoved(end),
            ring_pointer(end, false, Modifiers::NONE),
        ]);
        f.frame(release);
        f.warm();
        ring_authored_same(&f, &after_context)?;
        assert_eq!(fs::read(&path)?, raw);
        if let Some((input, bytes, count)) = replacement {
            assert_eq!(fs::read(input)?, bytes);
            let c = f.command("ui.controls", json!({}))?;
            let rows = c["controls"].as_array().ok_or("controls")?;
            assert_eq!(rows.len(), count);
            let settings = rows
                .iter()
                .find(|row| row["id"] == "file.document_settings")
                .ok_or("Document Settings remains visible but unavailable")?;
            assert_eq!(settings["enabled"], false);
            assert_eq!(settings["disabled_reason"], "unsupported_content");
            assert!(
                rows.iter()
                    .all(|r| !r["id"].as_str().unwrap_or("").starts_with("tool.ring."))
            );
            f.command("document.open_cdxml", json!({"path":path}))?;
            f.warm();
        }
        f.command("ui.view", json!({"action":"actual"}))?;
        f.warm();
        ring_choose(&mut f, "Cyclohexane")?;
        let fresh = f.state()?;
        ring_click(&mut f, [120.0, 168.0])?;
        f.warm();
        ring_assert_graph(&f, &ring_hex(1.0))?;
        ring_one_edit(&f, &fresh, &path, &raw)?;
    }
    Ok(())
}

#[test]
fn ring_releases_tools_history_and_file_save_resolve_at_their_actual_raw_boundaries() -> TestResult
{
    for action in ["tool", "undo", "save"] {
        for action_first in [false, true] {
            let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
            let path = ring_literal_source(&mut f, &[], [400.0, 500.0])?;
            let raw = fs::read(&path)?;
            ring_choose(&mut f, "Cyclohexane")?;
            let first = ring_shift(ring_hex(1.0), 1, 0, 0, [-24.0, 0.0])?;
            if action == "undo" {
                ring_click(&mut f, [96.0, 168.0])?;
                f.warm();
            }
            let before = f.state()?;
            let start = if action == "undo" {
                [264.0, 360.0]
            } else {
                [96.0, 168.0]
            };
            let a = ring_point(&mut f, start)?;
            f.frame(vec![
                Event::PointerMoved(a),
                ring_pointer(a, true, Modifiers::NONE),
            ]);
            let acts = match action {
                "tool" => vec![ring_activation(&f, "Single bond")?],
                "undo" => ring_keys(Key::Z, Modifiers::COMMAND),
                "save" => ring_keys(Key::S, Modifiers::COMMAND),
                _ => return Err("action".into()),
            };
            let release = vec![ring_pointer(a, false, Modifiers::NONE)];
            f.frame(if action_first {
                [acts, release].concat()
            } else {
                [release, acts].concat()
            });
            if action == "save" {
                finish_actual_file_workflow(&mut f)?;
            }
            f.warm();
            let expected_graphs = if !action_first {
                vec![first.clone()]
            } else {
                vec![]
            };
            let expected = serde_json::to_value(ring_model(&expected_graphs, [400.0, 500.0])?)?;
            ring_compare_document(&f.state()?["document"], &expected)?;
            let revisions = if action == "undo" {
                if action_first { 1 } else { 2 }
            } else if action_first {
                0
            } else {
                1
            };
            assert_eq!(
                f.state()?["revision"],
                before["revision"].as_u64().ok_or("revision")? + revisions
            );
            assert_eq!(
                f.state()?["undo_depth"],
                if expected_graphs.is_empty() { 0 } else { 1 }
            );
            assert_eq!(
                f.state()?["redo_depth"],
                if action == "undo" { 1 } else { 0 }
            );
            if action == "tool" {
                assert_eq!(ring_selected(&f)?, vec!["tool.bond.single"]);
            } else {
                assert_eq!(ring_selected(&f)?, vec!["tool.ring.6"]);
            }
            if action == "save" {
                assert_eq!(f.state()?["file"]["dirty"], false);
                if action_first {
                    assert_eq!(fs::read(&path)?, raw);
                } else {
                    ring_consume(&path, &f.state()?["document"], &["C6H12"])?;
                }
            } else {
                assert_eq!(fs::read(&path)?, raw);
            }
            assert!(
                f.app.error.is_none(),
                "action={action} first={action_first}: {:?}",
                f.app.error
            );
        }
    }
    Ok(())
}

#[test]
fn two_completed_ring_clicks_in_one_packet_are_two_distinct_current_source_commits() -> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let path = ring_literal_source(&mut f, &[], [400.0, 500.0])?;
    let raw = fs::read(&path)?;
    ring_choose(&mut f, "Cyclohexane")?;
    let a = ring_point(&mut f, [96.0, 168.0])?;
    let b = ring_point(&mut f, [264.0, 360.0])?;
    let before = f.state()?;
    f.frame(vec![
        Event::PointerMoved(a),
        ring_pointer(a, true, Modifiers::NONE),
        ring_pointer(a, false, Modifiers::NONE),
        Event::PointerMoved(b),
        ring_pointer(b, true, Modifiers::NONE),
        ring_pointer(b, false, Modifiers::NONE),
    ]);
    let shown = f.warm();
    let graphs = [
        ring_shift(ring_hex(1.0), 1, 0, 0, [-24.0, 0.0])?,
        ring_shift(ring_hex(1.0), 2, 6, 6, [144.0, 192.0])?,
    ];
    let expected = serde_json::to_value(ring_model(&graphs, [400.0, 500.0])?)?;
    ring_compare_document(&f.state()?["document"], &expected)?;
    assert_eq!(
        f.state()?["revision"],
        before["revision"].as_u64().ok_or("revision")? + 2
    );
    assert_eq!(f.state()?["undo_depth"], 2);
    assert_eq!(f.state()?["redo_depth"], 0);
    assert_eq!(fs::read(&path)?, raw);
    ring_paint_matches(&f, &shown, &graphs, [400.0, 500.0])?;
    ring_shortcut(&mut f, Key::Z, false);
    assert_eq!(
        f.state()?["document"]["pages"][0]["fragments"]
            .as_array()
            .ok_or("fragments")?
            .len(),
        1
    );
    ring_shortcut(&mut f, Key::Z, true);
    ring_compare_document(&f.state()?["document"], &expected)?;
    ring_shortcut(&mut f, Key::S, false);
    finish_actual_file_workflow(&mut f)?;
    ring_consume(&path, &f.state()?["document"], &["C6H12", "C6H12"])?;
    Ok(())
}

#[test]
fn ring_modified_primary_and_ineligible_hits_never_open_another_form_or_fall_back() -> TestResult {
    let mods = [
        Modifiers::SHIFT,
        Modifiers {
            alt: true,
            ..Modifiers::NONE
        },
        Modifiers {
            ctrl: true,
            ..Modifiers::NONE
        },
        Modifiers::COMMAND,
    ];
    for modifiers in mods {
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        let path = ring_seed(&mut f, Some([[120.0, 168.0], [120.0, 120.0]]))?;
        let raw = fs::read(&path)?;
        ring_choose(&mut f, "Cyclohexane")?;
        let before = f.state()?;
        for p in [[120.0, 168.0], [72.0, 240.0]] {
            let a = ring_point(&mut f, p)?;
            f.frame(vec![
                Event::PointerMoved(a),
                ring_pointer(a, true, modifiers),
            ]);
            f.frame(vec![ring_pointer(a, false, modifiers)]);
            f.warm();
            assert_eq!(f.state()?, before);
            assert!(f.node("Element", &[Role::TextInput]).is_err());
            assert_eq!(f.state()?["ui_atom_label"], Value::Null);
            assert!(
                f.app.error.is_some(),
                "unsupported Ring modifier must be reported"
            );
            assert_eq!(fs::read(&path)?, raw);
        }
    }
    for kind in ["oxygen", "inconsistent_h"] {
        let mut source = ring_graph(
            &[[120.0, 168.0], [120.0, 120.0]],
            &[3, if kind == "oxygen" { 1 } else { 0 }],
            &[[1, 2]],
        );
        if kind == "oxygen" {
            source["atoms"][1]["element"] = json!(8);
        }
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        let path = ring_literal_source(&mut f, &[source], [288.0, 288.0])?;
        let raw = fs::read(&path)?;
        ring_choose(&mut f, "Cyclohexane")?;
        let before = f.state()?;
        ring_click(&mut f, [120.0, 120.0])?;
        f.warm();
        assert_eq!(f.state()?, before);
        assert!(f.app.error.is_some());
        assert_eq!(fs::read(&path)?, raw);
        ring_click(&mut f, [120.0, 144.0])?;
        f.warm();
        assert_eq!(f.state()?, before);
        assert!(f.app.error.is_some());
        assert_eq!(fs::read(&path)?, raw);
    }
    Ok(())
}

#[test]
fn an_actual_atom_modal_and_its_cancel_frame_fence_armed_ring_history_and_save() -> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let path = ring_seed(&mut f, Some([[120.0, 168.0], [120.0, 120.0]]))?;
    let raw = fs::read(&path)?;
    ring_choose(&mut f, "Cyclohexane")?;
    ring_click(&mut f, [120.0, 144.0])?;
    f.warm();
    ring_assert_graph(&f, &ring_hex(1.0))?;
    let before = f.state()?;
    assert_eq!(before["undo_depth"], 1);
    let a = ring_point(&mut f, [120.0, 168.0])?;
    f.frame(vec![
        Event::PointerMoved(a),
        ring_pointer(a, true, Modifiers::NONE),
    ]);
    f.command("ui.atom_label",json!({"action":"open","expected_revision":before["revision"],"page_id":1,"fragment_id":1,"atom_id":1}))?;
    ring_wait(&mut f, |f, _| f.node("Element", &[Role::TextInput]).is_ok())?;
    let (element, _) = f.node("Element", &[Role::TextInput])?;
    f.frame(vec![Event::AccessKitActionRequest(
        egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::Focus,
            target: element,
            data: None,
        },
    )]);
    let mut text = ring_keys(Key::A, Modifiers::COMMAND);
    text.push(Event::Text("Cl".into()));
    text.extend(ring_keys(Key::ArrowDown, Modifiers::SHIFT));
    f.frame(text);
    ring_authored_same(&f, &before)?;
    assert_eq!(ring_selected(&f)?, vec!["tool.ring.6"]);
    let (cancel, _) = f.node("Cancel", &[Role::Button])?;
    let mut packet = vec![
        Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::Default,
            target: cancel,
            data: None,
        }),
        ring_pointer(a, false, Modifiers::NONE),
        Event::PointerMoved(a),
        ring_pointer(a, true, Modifiers::NONE),
        ring_pointer(a, false, Modifiers::NONE),
    ];
    packet.extend(ring_keys(Key::Z, Modifiers::COMMAND));
    packet.extend(ring_keys(Key::S, Modifiers::COMMAND));
    f.frame(packet);
    f.warm();
    ring_authored_same(&f, &before)?;
    assert_eq!(f.state()?["ui_atom_label"], Value::Null);
    assert_eq!(f.state()?["ui_file"]["request"], Value::Null);
    assert!(!f.app.files.is_busy());
    assert_eq!(fs::read(&path)?, raw);
    ring_click(&mut f, [120.0, 168.0])?;
    f.warm();
    ring_assert_graph(&f, &ring_spiro())?;
    assert_eq!(f.state()?["undo_depth"], 2);
    ring_shortcut(&mut f, Key::S, false);
    finish_actual_file_workflow(&mut f)?;
    ring_consume(&path, &f.state()?["document"], &["C11H20"])?;
    Ok(())
}

// Bounded role/pose loop requested separately; no duplicate full polygon solver matrix.
#[test]
fn each_accessible_ring_button_has_its_own_visible_polygon_and_creates_the_selected_size()
-> TestResult {
    for (n, name) in [
        (3_u32, "Cyclopropane"),
        (4, "Cyclobutane"),
        (5, "Cyclopentane"),
        (6, "Cyclohexane"),
        (7, "Cycloheptane"),
        (8, "Cyclooctane"),
    ] {
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        // Roomy original paper isolates the locked fixed click pose from paper-edge guards.
        let path = ring_literal_source(&mut f, &[], [400.0, 500.0])?;
        let raw = fs::read(&path)?;
        ring_choose(&mut f, name)?;
        let shown = f.warm();
        let (id, r) = f.node(name, &[Role::Button, Role::CheckBox])?;
        assert_eq!(
            f.nodes.get(&id).ok_or("ring button")?.toggled(),
            Some(egui::accesskit::Toggled::True)
        );
        let inner = r.shrink(3.0);
        let polygon = shown.shapes.iter().any(|s| {
            if !s.clip_rect.intersects(r) {
                return false;
            }
            match &s.shape {
                egui::Shape::Path(path) => {
                    path.stroke.width > 0.0
                        && path.closed
                        && path.points.len() == usize::try_from(n).unwrap_or(0)
                        && path.points.iter().all(|p| inner.contains(*p))
                }
                egui::Shape::Rect(rect) => {
                    n == 4 && rect.stroke.width > 0.0 && inner.contains_rect(rect.rect)
                }
                _ => false,
            }
        });
        assert!(
            polygon,
            "{name} needs its visible procedural{n}-gon inside the actual tool response"
        );
        assert_eq!(ring_selected(&f)?, vec![format!("tool.ring.{n}")]);
        let c = f.command("ui.controls", json!({}))?;
        let row = c["controls"]
            .as_array()
            .ok_or("controls")?
            .iter()
            .find(|c| c["id"] == format!("tool.ring.{n}"))
            .ok_or("typed ring binding")?;
        assert_eq!(
            row["command"],
            json!({"command":"ui.ring_tool","params":{"size":n}})
        );
        let before = f.state()?;
        ring_click(&mut f, [120.0, 168.0])?;
        f.warm();
        let state = f.state()?;
        let fragments = state["document"]["pages"][0]["fragments"]
            .as_array()
            .ok_or("fragments")?;
        assert_eq!(fragments.len(), 1);
        let atoms = fragments[0]["atoms"].as_array().ok_or("atoms")?;
        let bonds = fragments[0]["bonds"].as_array().ok_or("bonds")?;
        assert_eq!(atoms.len(), usize::try_from(n)?);
        assert_eq!(bonds.len(), usize::try_from(n)?);
        assert_eq!(atoms[0]["position"], json!({"x_pt":120.0,"y_pt":168.0}));
        assert_eq!(
            atoms[1]["position"],
            if n == 6 {
                json!({"x_pt":120.0,"y_pt":120.0})
            } else {
                json!({"x_pt":168.0,"y_pt":168.0})
            }
        );
        for (i, atom) in atoms.iter().enumerate() {
            assert_eq!(atom["id"], i + 1);
            assert_eq!(atom["element"], 6);
            assert_eq!(atom["explicit_hydrogens"], 2);
            assert_eq!(atom["charge"], 0);
            assert_eq!(atom["isotope"], Value::Null);
            assert_eq!(atom["radical"], "none");
        }
        for (i, bond) in bonds.iter().enumerate() {
            assert_eq!(
                bond,
                &json!({"id":i+1,"begin":i+1,"end":if i+1==atoms.len(){1}else{i+2},"order":"single"})
            );
        }
        ring_one_edit(&f, &before, &path, &raw)?;
    }
    Ok(())
}

// Independent source-review regressions. Append to the existing native_ring test child.
fn ring_review_seed_large(
    f: &mut Fixture,
    width: f64,
    height: f64,
    percent: f64,
    center: [f64; 2],
) -> TestResult<PathBuf> {
    f.command(
        "page.set_size",
        json!({"page_id":1,"width_pt":width,"height_pt":height}),
    )?;
    let path = f.directory.path("native-ring-boundary-original-原.cdxml");
    f.command("document.save_cdxml", json!({"path":path}))?;
    f.command("document.open_cdxml", json!({"path":path}))?;
    f.command("ui.view", json!({"action":"set_zoom","percent":percent}))?;
    f.command(
        "ui.view",
        json!({"action":"pan","center_x_pt":center[0],"center_y_pt":center[1]}),
    )?;
    let shown = f.warm();
    assert!(canvas(&shown)?.is_positive());
    assert_eq!(f.state()?["undo_depth"], 0);
    assert_eq!(f.state()?["redo_depth"], 0);
    assert_eq!(f.state()?["file"]["dirty"], false);
    assert_eq!(f.state()?["source"]["modified"], false);
    assert!(
        f.state()?["document"]["pages"][0]["fragments"]
            .as_array()
            .ok_or("fragments")?
            .is_empty()
    );
    assert!(
        f.app.error.is_none(),
        "initial large vector viewport: {:?}",
        f.app.error
    );
    Ok(path)
}
fn ring_review_owner_same(
    f: &Fixture,
    stamp: &bondline_commands::CaptureStamp,
    model: &Arc<bondline_model::Document>,
    file: &bondline_commands::UiFileContextStamp,
) -> TestResult {
    let e = f.app.engine.lock().map_err(|_| "engine")?;
    assert!(e.capture_stamp_matches(stamp));
    assert!(Arc::ptr_eq(model, &e.stored_document()?.model_snapshot()));
    assert!(e.ui_file_context_stamp_matches(file));
    Ok(())
}
fn ring_review_shift_graph(mut graph: Value, fragment: u32, offset: u32) -> TestResult<Value> {
    graph["id"] = json!(fragment);
    for a in graph["atoms"].as_array_mut().ok_or("atoms")? {
        a["id"] = json!(a["id"].as_u64().ok_or("id")? + u64::from(offset));
    }
    for b in graph["bonds"].as_array_mut().ok_or("bonds")? {
        for field in ["id", "begin", "end"] {
            b[field] = json!(b[field].as_u64().ok_or("bond field")? + u64::from(offset));
        }
    }
    Ok(graph)
}

#[test]
fn review_legal_leftward_drag_does_not_require_an_out_of_domain_click_default() -> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let path = ring_review_seed_large(&mut f, 100000.0, 1000.0, 99.0, [99950.0, 500.0])?;
    let raw = fs::read(&path)?;
    ring_choose(&mut f, "Cyclopropane")?;
    let before = f.state()?;
    let (stamp, model, file) = {
        let e = f.app.engine.lock().map_err(|_| "engine")?;
        (
            e.capture_stamp(),
            e.stored_document()?.model_snapshot(),
            e.ui_file_context_stamp(),
        )
    };
    // At99%, these quarter-device phases are exactly representable even in the
    // large virtual source. The default x+48 is invalid, but this explicit50pt
    // leftward edge and every literal triangle vertex are within the point domain.
    let a = ring_point(&mut f, [99975.0, 500.0])?;
    let b = ring_point(&mut f, [99925.0, 500.0])?;
    let page_width = f.state()?["document"]["pages"][0]["size"]["width_pt"]
        .as_f64()
        .ok_or("actual page width")?;
    assert_eq!(page_width, 100000.0);
    assert!(99975.0 + 48.0 > page_width);
    assert!(a.distance(b) > 10.0);
    let first = ring_graph(
        &[
            [99975.0, 500.0],
            [99925.0, 500.0],
            [99950.0, 500.0 - 25.0 * 3.0_f64.sqrt()],
        ],
        &[2; 3],
        &[[1, 2], [2, 3], [3, 1]],
    );
    f.frame(vec![
        Event::PointerMoved(a),
        ring_pointer(a, true, Modifiers::NONE),
    ]);
    let held = f.frame(vec![Event::PointerMoved(b)]);
    ring_authored_same(&f, &before)?;
    ring_review_owner_same(&f, &stamp, &model, &file)?;
    assert_eq!(fs::read(&path)?, raw);
    f.frame(vec![ring_pointer(b, false, Modifiers::NONE)]);
    let shown = f.warm();
    ring_assert_graph(&f, &first)?;
    ring_one_edit(&f, &before, &path, &raw)?;
    ring_paint_matches(&f, &shown, std::slice::from_ref(&first), [100000.0, 1000.0])?;
    // The held frame is ordinary actual-App evidence; no direct setter or fake preview.
    assert_eq!(held.pixels_per_point, 1.0);
    // Fresh gesture after the boundary operation must work with the next stable IDs.
    let next_before = f.state()?;
    let a = ring_point(&mut f, [99875.0, 650.0])?;
    let b = ring_point(&mut f, [99825.0, 650.0])?;
    f.frame(vec![
        Event::PointerMoved(a),
        ring_pointer(a, true, Modifiers::NONE),
    ]);
    f.frame(vec![Event::PointerMoved(b)]);
    f.frame(vec![ring_pointer(b, false, Modifiers::NONE)]);
    let shown = f.warm();
    let second = ring_review_shift_graph(
        ring_graph(
            &[
                [99875.0, 650.0],
                [99825.0, 650.0],
                [99850.0, 650.0 - 25.0 * 3.0_f64.sqrt()],
            ],
            &[2; 3],
            &[[1, 2], [2, 3], [3, 1]],
        ),
        2,
        3,
    )?;
    let expected = serde_json::to_value(ring_model(
        &[first.clone(), second.clone()],
        [100000.0, 1000.0],
    )?)?;
    ring_compare_document(&f.state()?["document"], &expected)?;
    ring_one_edit(&f, &next_before, &path, &raw)?;
    ring_paint_matches(&f, &shown, &[first, second], [100000.0, 1000.0])?;
    Ok(())
}

#[test]
fn review_physical_viewport_refusal_permanently_cancels_the_old_ring_gesture() -> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let path = ring_review_seed_large(&mut f, 60000.0, 1000.0, 100.0, [30000.0, 500.0])?;
    let raw = fs::read(&path)?;
    ring_choose(&mut f, "Cyclohexane")?;
    let before = f.state()?;
    let view = f.view()?;
    let (stamp, model, file) = {
        let e = f.app.engine.lock().map_err(|_| "engine")?;
        (
            e.capture_stamp(),
            e.stored_document()?.model_snapshot(),
            e.ui_file_context_stamp(),
        )
    };
    let a = ring_point(&mut f, [30000.0, 500.0])?;
    let q = 24.0 * 3.0_f64.sqrt();
    let expected = ring_graph(
        &[
            [30000.0, 500.0],
            [30000.0, 452.0],
            [30000.0 + q, 428.0],
            [30000.0 + 2.0 * q, 452.0],
            [30000.0 + 2.0 * q, 500.0],
            [30000.0 + q, 524.0],
        ],
        &[2; 6],
        &[[1, 2], [2, 3], [3, 4], [4, 5], [5, 6], [6, 1]],
    );
    let held = f.frame(vec![
        Event::PointerMoved(a),
        ring_pointer(a, true, Modifiers::NONE),
    ]);
    ring_authored_same(&f, &before)?;
    ring_review_owner_same(&f, &stamp, &model, &file)?;
    ring_paint_matches(
        &f,
        &held,
        std::slice::from_ref(&expected),
        [60000.0, 1000.0],
    )?;
    //60064 logical source units fit at1x; the unchanged2x full-source bound refuses.
    // DPI is native-input metadata here, not semantic zoom or an Engine revision.
    f.dpi = 2.0;
    let refused = f.frame(vec![]);
    assert_eq!(refused.pixels_per_point, 2.0);
    let guard = bondline_render::Error::InvalidViewport.to_string();
    assert!(
        f.app.error.as_ref().is_some_and(|s| s.contains(&guard))
            || refused.shapes.iter().any(
                |s| matches!(&s.shape,egui::Shape::Text(t)if t.galley.text().contains(&guard))
            ),
        "setup must reach the real physical viewport refusal"
    );
    ring_authored_same(&f, &before)?;
    ring_review_owner_same(&f, &stamp, &model, &file)?;
    assert_eq!(f.view()?, view);
    assert_eq!(fs::read(&path)?, raw);
    f.dpi = 1.0;
    let returned = f.frame(vec![]);
    assert_eq!(returned.pixels_per_point, 1.0);
    f.frame(vec![ring_pointer(a, false, Modifiers::NONE)]);
    f.warm();
    ring_authored_same(&f, &before)?;
    ring_review_owner_same(&f, &stamp, &model, &file)?;
    assert_eq!(f.view()?, view);
    assert_eq!(fs::read(&path)?, raw);
    // A new press/release after retirement proves cancellation did not disable authoring.
    ring_click(&mut f, [30000.0, 500.0])?;
    let shown = f.warm();
    ring_assert_graph(&f, &expected)?;
    ring_one_edit(&f, &before, &path, &raw)?;
    ring_paint_matches(
        &f,
        &shown,
        std::slice::from_ref(&expected),
        [60000.0, 1000.0],
    )?;
    Ok(())
}

#[path = "native_ring_tests/unsaturated.rs"]
mod unsaturated;
