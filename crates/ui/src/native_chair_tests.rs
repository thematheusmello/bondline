//! Independent actual-App chair control declaration; existing APIs only.
use super::{Fixture, TestResult, button};
use eframe::egui;

#[test]
fn cyclohexane_chair_one_is_actionable_and_selects_its_central_binding_without_authoring()
-> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    f.warm();
    let before = f.state()?;
    let (id, rect) = f.node(
        "Cyclohexane chair 1",
        &[
            egui::accesskit::Role::Button,
            egui::accesskit::Role::CheckBox,
        ],
    )?;
    assert!(
        !f.nodes
            .get(&id)
            .ok_or("current chair1 control")?
            .is_disabled(),
        "the actual Cyclohexane chair1 control must be actionable"
    );
    assert!(
        rect.is_positive()
            && egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1160.0, 800.0))
                .contains_rect(rect)
    );
    f.frame(vec![
        egui::Event::PointerMoved(rect.center()),
        button(rect.center(), true),
    ]);
    f.frame(vec![button(rect.center(), false)]);
    let controls = f.command("ui.controls", serde_json::json!({}))?;
    let selected = controls["controls"]
        .as_array()
        .ok_or("controls")?
        .iter()
        .find(|control| control["id"] == "tool.ring.chair_1")
        .ok_or("central chair1 binding")?;
    assert_eq!(selected["enabled"], true);
    assert_eq!(selected["selected"], true);
    assert_eq!(
        selected["command"],
        serde_json::json!({
            "command":"ui.ring_tool","params":{"kind":"cyclohexane_chair_1"}
        })
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
        assert_eq!(
            after[field], before[field],
            "tool activation authored {field}"
        );
    }
    Ok(())
}

// Independent actual-App camera/history and external-reader harness.
use super::{canvas, key, projected};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command as Process, Stdio},
    time::{Duration, Instant},
};

fn chair_wait(
    f: &mut Fixture,
    ready: impl Fn(&Fixture, &egui::FullOutput) -> bool,
) -> TestResult<egui::FullOutput> {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let output = f.frame(vec![]);
        if ready(f, &output) {
            return Ok(output);
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "chair App condition did not become ready: {:?}",
                f.app.error
            )
            .into());
        }
        std::thread::yield_now();
    }
}
fn chair_select(f: &mut Fixture, kind: usize) -> TestResult {
    let (label, wire, id) = if kind == 1 {
        (
            "Cyclohexane chair 1",
            "cyclohexane_chair_1",
            "tool.ring.chair_1",
        )
    } else {
        (
            "Cyclohexane chair 2",
            "cyclohexane_chair_2",
            "tool.ring.chair_2",
        )
    };
    chair_wait(f, |f, _| {
        f.node(
            label,
            &[
                egui::accesskit::Role::Button,
                egui::accesskit::Role::CheckBox,
            ],
        )
        .is_ok_and(|(id, _)| f.nodes.get(&id).is_some_and(|n| !n.is_disabled()))
    })?;
    let (_, rect) = f.node(
        label,
        &[
            egui::accesskit::Role::Button,
            egui::accesskit::Role::CheckBox,
        ],
    )?;
    f.frame(vec![
        egui::Event::PointerMoved(rect.center()),
        button(rect.center(), true),
    ]);
    f.frame(vec![button(rect.center(), false)]);
    let controls = f.command("ui.controls", json!({}))?;
    let selected = controls["controls"]
        .as_array()
        .ok_or("controls")?
        .iter()
        .find(|c| c["id"] == id)
        .ok_or("selected chair")?;
    assert_eq!(selected["selected"], true);
    assert_eq!(
        selected["command"],
        json!({"command":"ui.ring_tool","params":{"kind":wire}})
    );
    let shown = f.warm();
    let (_, rect) = f.node(
        label,
        &[
            egui::accesskit::Role::Button,
            egui::accesskit::Role::CheckBox,
        ],
    )?;
    assert!(
        shown.shapes.iter().any(|paint| {
            let egui::Shape::Path(path) = &paint.shape else {
                return false;
            };
            if !path.closed
                || path.points.len() != 6
                || path.stroke.width <= 0.0
                || !path.points.iter().all(|p| rect.contains(*p))
            {
                return false;
            }
            let mut positive = false;
            let mut negative = false;
            for i in 0..6 {
                let a = path.points[i];
                let b = path.points[(i + 1) % 6];
                let c = path.points[(i + 2) % 6];
                let cross = (b.x - a.x) * (c.y - b.y) - (b.y - a.y) * (c.x - b.x);
                positive |= cross > 1e-6;
                negative |= cross < -1e-6;
            }
            positive && negative
        }),
        "the actual tool must paint an original nonconvex chair glyph, not a regular hexagon"
    );
    Ok(())
}
fn chair_seed(f: &mut Fixture, kind: usize, length: Option<f64>) -> TestResult<PathBuf> {
    f.command(
        "document.set_drawing_settings",
        json!({"expected_revision":f.state()?["revision"],
        "bond_length_pt":length,"chain_angle_degrees":137.5}),
    )?;
    f.command("ui.chain_tool", json!({"action":"select","kind":"snaking"}))?;
    f.command(
        "ui.chain_tool",
        json!({"action":"constraints","fixed_lengths":false,"fixed_angles":false}),
    )?;
    let source = f.directory.path("original-chair-λ.cdxml");
    f.command("document.save_cdxml", json!({"path":source}))?;
    f.command("document.open_cdxml", json!({"path":source}))?;
    f.command("ui.view", json!({"action":"actual"}))?;
    f.warm();
    chair_select(f, kind)?;
    f.warm();
    assert_eq!(f.state()?["undo_depth"], 0);
    assert_eq!(f.state()?["file"]["dirty"], false);
    Ok(source)
}
fn chair_at(f: &Fixture, area: egui::Rect, point: [f64; 2]) -> TestResult<egui::Pos2> {
    let state = f.state()?;
    let size = &state["document"]["pages"][0]["size"];
    let view = f.view()?;
    let center = match (view["center_x_pt"].as_f64(), view["center_y_pt"].as_f64()) {
        (Some(x), Some(y)) => Some([x, y]),
        _ => None,
    };
    let p = projected(
        area,
        [
            size["width_pt"].as_f64().ok_or("width")?,
            size["height_pt"].as_f64().ok_or("height")?,
        ],
        view["zoom_percent"].as_f64().ok_or("explicit camera")?,
        center,
        point,
    );
    assert!(
        area.contains(p),
        "literal point {point:?} must lie on the actual canvas"
    );
    Ok(p)
}
fn chair_pointer(pos: egui::Pos2, pressed: bool, modifiers: egui::Modifiers) -> egui::Event {
    egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers,
    }
}
fn chair_literal(points: &[[f64; 2]]) -> Value {
    json!({"id":1,"atoms":points.iter().enumerate().map(|(i,p)|json!({"id":i+1,"element":6,
        "position":{"x_pt":p[0],"y_pt":p[1]},"isotope":null,"charge":0,"radical":"none",
        "explicit_hydrogens":2,"show_non_terminal_carbon_labels":false})).collect::<Vec<_>>(),
        "bonds":points.iter().enumerate().map(|(i,_)|json!({"id":i+1,"begin":i+1,"end":(i+1)%points.len()+1,"order":"single"})).collect::<Vec<_>>()})
}
fn chair_compare(f: &Fixture, before: &Value, fragments: &[Value]) -> TestResult {
    let mut expected = before["document"].clone();
    expected["pages"][0]["fragments"] = json!(fragments);
    let mut actual = f.state()?["document"].clone();
    let actual_fragments = actual["pages"][0]["fragments"]
        .as_array_mut()
        .ok_or("fragments")?;
    assert_eq!(actual_fragments.len(), fragments.len());
    for (a, e) in actual_fragments.iter_mut().zip(fragments) {
        let atoms = a["atoms"].as_array_mut().ok_or("atoms")?;
        let wants = e["atoms"].as_array().ok_or("expected atoms")?;
        assert_eq!(atoms.len(), wants.len());
        for (a, e) in atoms.iter_mut().zip(wants) {
            for axis in ["x_pt", "y_pt"] {
                assert!(
                    (a["position"][axis].as_f64().ok_or("actual position")?
                        - e["position"][axis].as_f64().ok_or("expected position")?)
                    .abs()
                        <= 1e-9
                );
            }
            a["position"] = e["position"].clone();
        }
    }
    assert_eq!(
        actual, expected,
        "complete literal arrays/H/metadata/flag presence"
    );
    Ok(())
}
fn chair_same(f: &Fixture, before: &Value) -> TestResult {
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
        assert_eq!(after[field], before[field], "presentation changed {field}");
    }
    Ok(())
}
fn chair_one_edit(f: &Fixture, before: &Value, source: &Path, raw: &[u8]) -> TestResult<Value> {
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
    assert_eq!(after["file"]["path"], before["file"]["path"]);
    assert_eq!(after["file"]["dirty"], true);
    assert_eq!(after["source"]["modified"], true);
    assert_eq!(fs::read(source)?, raw);
    assert!(f.app.error.is_none(), "{:?}", f.app.error);
    Ok(after)
}
fn chair_history(f: &mut Fixture, before: &Value, after: &Value) -> TestResult {
    f.frame(vec![
        key(egui::Key::Z, true, egui::Modifiers::COMMAND),
        key(egui::Key::Z, false, egui::Modifiers::COMMAND),
    ]);
    assert_eq!(f.state()?["document"], before["document"]);
    assert_eq!(f.state()?["source"], before["source"]);
    assert_eq!(f.state()?["file"], before["file"]);
    assert_eq!(f.state()?["undo_depth"], before["undo_depth"]);
    assert_eq!(f.state()?["redo_depth"], 1);
    let m = egui::Modifiers {
        shift: true,
        ..egui::Modifiers::COMMAND
    };
    f.frame(vec![
        key(egui::Key::Z, true, m),
        key(egui::Key::Z, false, m),
    ]);
    assert_eq!(f.state()?["document"], after["document"]);
    assert_eq!(f.state()?["source"], after["source"]);
    assert_eq!(f.state()?["file"], after["file"]);
    assert_eq!(f.state()?["undo_depth"], after["undo_depth"]);
    assert_eq!(f.state()?["redo_depth"], 0);
    Ok(())
}
fn chair_consume(
    path: &Path,
    fragments: &[Value],
    smiles: &[&str],
    formulas: &[&str],
) -> TestResult {
    let expected = json!({"fragments":fragments,"smiles":smiles,"formulas":formulas});
    let mut child=Process::new(std::env::var_os("BONDLINE_PYTHON").ok_or("pinned external Python")?).args(["-B","-c",r#"
import json,subprocess,sys,xml.parsers.expat as expat
from pathlib import Path
from decimal import Decimal
from rdkit import Chem,rdBase
from rdkit.Chem import rdMolDescriptors
assert rdBase.rdkitVersion=='2026.09.2'
e=json.load(sys.stdin);want=e['fragments'];source=Path(sys.argv[1]).read_bytes()
plain=json.loads(json.dumps(want))
for f in plain:
 for a in f['atoms']:
  a.pop('show_terminal_carbon_labels',None);a.pop('show_non_terminal_carbon_labels',None)
 for b in f['bonds']:b.pop('double_position',None)
r=subprocess.run([sys.executable,'-B',sys.argv[2]],input=source,capture_output=True,check=True)
actual=json.loads(r.stdout);assert len(actual)==len(plain)
for a,w in zip(actual,plain):
 assert len(a['atoms'])==len(w['atoms'])
 for aa,ww in zip(a['atoms'],w['atoms']):
  for axis in ('x_pt','y_pt'):
   assert abs(Decimal(str(aa['position'][axis]))-Decimal(str(ww['position'][axis])))<=Decimal('1e-9')
  aa['position']=ww['position']
assert actual==plain,'independent directed raw graph/H/metadata'
nodes=[];bonds=[]
def start(name,attrs):
 if name=='n':nodes.append(attrs)
 if name=='b':bonds.append(attrs)
p=expat.ParserCreate();p.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
p.StartElementHandler=start;p.Parse(source,True)
atoms=[a for f in want for a in f['atoms']];edges=[b for f in want for b in f['bonds']]
assert len(nodes)==len(atoms) and len(bonds)==len(edges)
for n,a in zip(nodes,atoms):
 for field,xml in [('show_terminal_carbon_labels','ShowTerminalCarbonLabels'),('show_non_terminal_carbon_labels','ShowNonTerminalCarbonLabels')]:
  assert n.get(xml)==({True:'yes',False:'no'}[a[field]] if field in a else None)
 for value,axis in zip(map(Decimal,n['p'].split()),('x_pt','y_pt')):
  assert value.is_finite() and abs(value-Decimal(str(a['position'][axis])))<=Decimal('1e-9')
for b,w in zip(bonds,edges):
 assert b.get('DoublePosition')==(w['double_position'].capitalize() if 'double_position' in w else None)
 assert 'Display' not in b
ms=Chem.MolsFromCDXML(source.decode('utf-8'));assert len(ms)==len(want)==len(e['smiles'])==len(e['formulas'])
for m,g,s,formula in zip(ms,want,e['smiles'],e['formulas']):
 assert m.GetNumAtoms()==len(g['atoms']) and m.GetNumBonds()==len(g['bonds'])
 assert [a.GetAtomicNum() for a in m.GetAtoms()]==[a['element'] for a in g['atoms']]
 assert [a.GetTotalNumHs() for a in m.GetAtoms()]==[a['explicit_hydrogens'] for a in g['atoms']]
 assert [b.GetBondTypeAsDouble() for b in m.GetBonds()]==[{'single':1,'double':2,'triple':3}[b['order']] for b in g['bonds']]
 assert all(a.GetFormalCharge()==0 and a.GetIsotope()==0 and a.GetNumRadicalElectrons()==0 for a in m.GetAtoms())
 reference=Chem.MolFromSmiles(s);assert reference is not None
 assert Chem.MolToSmiles(m)==Chem.MolToSmiles(reference)
 assert rdMolDescriptors.CalcMolFormula(m)==formula
"#]).arg(path).arg(concat!(env!("CARGO_MANIFEST_DIR"),"/../../xtask/oracles/cdxml_graph.py"))
        .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
    serde_json::to_writer(child.stdin.take().ok_or("literal oracle stdin")?, &expected)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

// Final locked same-shape supporting-edge reference, independently written from public tuples.
fn chair_projection(kind: usize) -> [[f64; 2]; 6] {
    let h = 2.0_f64.sqrt() / 8.0;
    let q = 3.0_f64.sqrt() / 8.0;
    let g = 4.0 / 17.0_f64.sqrt();
    if kind == 1 {
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
    }
}
fn chair_affine(kind: usize, a: [f64; 2], b: [f64; 2], side: f64) -> [[f64; 2]; 6] {
    let p = chair_projection(kind);
    let e = [p[1][0] - p[0][0], p[1][1] - p[0][1]];
    let d = (7.0 + 6.0_f64.sqrt()) / 17.0;
    let edge = [b[0] - a[0], b[1] - a[1]];
    let origin = p[0];
    let mut result = p.map(|p| {
        let w = [p[0] - origin[0], p[1] - origin[1]];
        let t = [
            (w[0] * e[0] + w[1] * e[1]) / d,
            (e[0] * w[1] - e[1] * w[0]) / d,
        ];
        [
            a[0] + t[0] * edge[0] - side * t[1] * edge[1],
            a[1] + t[0] * edge[1] + side * t[1] * edge[0],
        ]
    });
    result[0] = a;
    result[1] = b;
    result
}
fn chair_horizontal(kind: usize, a: [f64; 2], length: f64) -> [[f64; 2]; 6] {
    let p = chair_projection(kind);
    let norm = ((7.0 + 6.0_f64.sqrt()) / 17.0).sqrt();
    let b = [
        a[0] + length * (p[1][0] - p[0][0]) / norm,
        a[1] + length * (p[1][1] - p[0][1]) / norm,
    ];
    chair_affine(kind, a, b, 1.0)
}
fn chair_outward_right(kind: usize, a: [f64; 2], length: f64) -> [[f64; 2]; 6] {
    let t = chair_affine(kind, [0.0, 0.0], [1.0, 0.0], 1.0);
    let c = [
        t.iter().map(|p| p[0]).sum::<f64>() / 6.0,
        t.iter().map(|p| p[1]).sum::<f64>() / 6.0,
    ];
    let norm = c[0].hypot(c[1]);
    chair_affine(
        kind,
        a,
        [a[0] + length * c[0] / norm, a[1] - length * c[1] / norm],
        1.0,
    )
}
fn chair_preview(
    f: &Fixture,
    out: &egui::FullOutput,
    area: egui::Rect,
    points: &[[f64; 2]],
) -> TestResult {
    assert_eq!(canvas(out)?, area);
    let mut strokes = out
        .shapes
        .iter()
        .filter_map(|paint| match &paint.shape {
            egui::Shape::Path(p)
                if p.closed
                    && p.fill == egui::Color32::BLACK
                    && p.points.len() == 4
                    && p.points.iter().all(|q| area.contains(*q)) =>
            {
                Some([
                    p.points[0].lerp(p.points[3], 0.5),
                    p.points[1].lerp(p.points[2], 0.5),
                ])
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        strokes.len(),
        6,
        "six complete actual chair strokes, not a regular polygon/ghost"
    );
    for i in 0..6 {
        let a = chair_at(f, area, points[i])?;
        let b = chair_at(f, area, points[(i + 1) % 6])?;
        let found = strokes
            .iter()
            .position(|[x, y]| {
                (x.distance(a) <= 1.0 / 256.0 && y.distance(b) <= 1.0 / 256.0)
                    || (x.distance(b) <= 1.0 / 256.0 && y.distance(a) <= 1.0 / 256.0)
            })
            .ok_or("actual chair preview axis differs from literal projection")?;
        strokes.swap_remove(found);
    }
    Ok(())
}
fn chair_click(f: &mut Fixture, point: [f64; 2]) -> TestResult {
    let area = canvas(&f.warm())?;
    let p = chair_at(f, area, point)?;
    f.frame(vec![
        egui::Event::PointerMoved(p),
        chair_pointer(p, true, egui::Modifiers::NONE),
    ]);
    f.frame(vec![chair_pointer(p, false, egui::Modifiers::NONE)]);
    Ok(())
}
fn chair_save(
    f: &mut Fixture,
    name: &str,
    before: &Value,
    fragments: &[Value],
    smiles: &[&str],
    formulas: &[&str],
) -> TestResult {
    let output = f.directory.path(name);
    let current = f.state()?;
    f.command("document.save_cdxml", json!({"path":output}))?;
    assert_eq!(f.state()?["document"], current["document"]);
    assert_eq!(f.state()?["file"]["dirty"], false);
    chair_consume(&output, fragments, smiles, formulas)?;
    f.command("document.open_cdxml", json!({"path":output}))?;
    chair_compare(f, before, fragments)?;
    assert_eq!(f.state()?["file"]["dirty"], false);
    assert_eq!(f.state()?["undo_depth"], 0);
    assert!(
        f.app
            .engine
            .lock()
            .map_err(|_| "engine")?
            .stored_document()?
            .is_fully_mapped()
    );
    Ok(())
}
#[test]
fn both_horizontal_chairs_preview_the_final_literal_pose_and_read_only_bond_length_preference()
-> TestResult {
    for kind in [1, 2] {
        for length in [Some(64.0), None] {
            let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
            let source = chair_seed(&mut f, kind, length)?;
            let raw = fs::read(&source)?;
            let area = canvas(&f.warm())?;
            let before = f.state()?;
            let view = f.view()?;
            let a = [180.0, 300.0];
            let pos = chair_at(&f, area, a)?;
            let points = chair_horizontal(kind, a, length.unwrap_or(48.0));
            let held = f.frame(vec![
                egui::Event::PointerMoved(pos),
                chair_pointer(pos, true, egui::Modifiers::NONE),
            ]);
            assert_eq!(f.state()?, before);
            assert_eq!(f.view()?, view);
            chair_preview(&f, &held, area, &points)?;
            f.frame(vec![chair_pointer(pos, false, egui::Modifiers::NONE)]);
            let after = chair_one_edit(&f, &before, &source, &raw)?;
            let literal = chair_literal(&points);
            chair_compare(&f, &before, std::slice::from_ref(&literal))?;
            assert_eq!(f.view()?, view);
            chair_history(&mut f, &before, &after)?;
            chair_save(
                &mut f,
                if kind == 1 {
                    "chair-one-horizontal-λ.cdxml"
                } else {
                    "chair-two-horizontal-λ.cdxml"
                },
                &before,
                &[literal],
                &["C1CCCCC1"],
                &["C6H12"],
            )?;
        }
    }
    Ok(())
}
#[test]
fn both_chairs_use_near_vertical_whole_chair_drag_for_the_same_literal_preview_and_commit()
-> TestResult {
    let near_vertical = |points: &[[f64; 2]]| {
        let center = [
            points.iter().map(|p| p[0]).sum::<f64>() / points.len() as f64,
            points.iter().map(|p| p[1]).sum::<f64>() / points.len() as f64,
        ];
        let (xx, yy, xy) = points
            .iter()
            .fold((0.0_f64, 0.0_f64, 0.0_f64), |(xx, yy, xy), p| {
                let x = p[0] - center[0];
                let y = p[1] - center[1];
                (xx + x * x, yy + y * y, xy + x * y)
            });
        let axis = 0.5 * (2.0 * xy).atan2(xx - yy);
        let deviation = axis.cos().abs().atan2(axis.sin().abs()).to_degrees();
        assert!(
            deviation <= 0.5,
            "WHOLE projected chair principal axis must be near vertical: {deviation} degrees off"
        );
    };
    for kind in [1, 2] {
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        let source = chair_seed(&mut f, kind, Some(64.0))?;
        let raw = fs::read(&source)?;
        let area = canvas(&f.warm())?;
        let before = f.state()?;
        let a = [180.0, 300.0];
        // Pref64 is retained. The actual reference-edge length is sqrt(48²+43²),
        // not64; these integer inputs rotate the WHOLE horizontal chair near90°.
        let delta = if kind == 1 {
            [-48.0, 43.0]
        } else {
            [48.0, 43.0]
        };
        let b = [a[0] + delta[0], a[1] + delta[1]];
        let halfway = [a[0] + delta[0] / 2.0, a[1] + delta[1] / 2.0];
        let press = chair_at(&f, area, a)?;
        let midway = chair_at(&f, area, halfway)?;
        let release = chair_at(&f, area, b)?;
        f.frame(vec![
            egui::Event::PointerMoved(press),
            chair_pointer(press, true, egui::Modifiers::NONE),
        ]);
        f.frame(vec![egui::Event::PointerMoved(midway)]);
        let preview = f.frame(vec![egui::Event::PointerMoved(release)]);
        assert_eq!(f.state()?, before);
        let points = chair_affine(kind, a, b, 1.0);
        near_vertical(&points);
        chair_preview(&f, &preview, area, &points)?;
        f.frame(vec![chair_pointer(release, false, egui::Modifiers::NONE)]);
        let after = chair_one_edit(&f, &before, &source, &raw)?;
        let literal = chair_literal(&points);
        chair_compare(&f, &before, std::slice::from_ref(&literal))?;
        let actual = f.state()?;
        let actual_points = actual["document"]["pages"][0]["fragments"][0]["atoms"]
            .as_array()
            .ok_or("chair atoms")?
            .iter()
            .map(|atom| {
                Ok([
                    atom["position"]["x_pt"].as_f64().ok_or("x")?,
                    atom["position"]["y_pt"].as_f64().ok_or("y")?,
                ])
            })
            .collect::<TestResult<Vec<_>>>()?;
        near_vertical(&actual_points);
        chair_history(&mut f, &before, &after)?;
        chair_save(
            &mut f,
            if kind == 1 {
                "chair-one-vertical-λ.cdxml"
            } else {
                "chair-two-vertical-λ.cdxml"
            },
            &before,
            &[literal],
            &["C1CCCCC1"],
            &["C6H12"],
        )?;
    }
    Ok(())
}
fn chair_old(points: &[[f64; 2]], h: &[u16], edges: &[[u32; 2]]) -> Value {
    json!({"id":1,"atoms":points.iter().zip(h).enumerate().map(|(i,(p,h))|json!({"id":i+1,"element":6,
        "position":{"x_pt":p[0],"y_pt":p[1]},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":h,
        "show_non_terminal_carbon_labels":false})).collect::<Vec<_>>(),
        "bonds":edges.iter().enumerate().map(|(i,[a,b])|json!({"id":i+1,"begin":a,"end":b,"order":"single"})).collect::<Vec<_>>()})
}
fn chair_native_source(
    f: &mut Fixture,
    graph: &Value,
    kind: usize,
    length: f64,
) -> TestResult<PathBuf> {
    let mut document = bondline_model::Document::default();
    document.set_drawing_settings(bondline_model::DrawingSettings::new(
        Some(length),
        Some(137.5),
    )?);
    document.append_fragments(
        bondline_model::PageId::new(1).ok_or("page")?,
        vec![bondline_model::Fragment::new(
            bondline_model::FragmentId::new(1).ok_or("fragment")?,
            serde_json::from_value(graph["atoms"].clone())?,
            serde_json::from_value(graph["bonds"].clone())?,
        )?],
    )?;
    let mut stored = bondline_commands::Engine::default()
        .stored_document()?
        .clone();
    *stored.model_mut() = document;
    {
        let mut engine = f.app.engine.lock().map_err(|_| "engine")?;
        *engine = bondline_commands::Engine::from_document(stored);
        engine.set_ui_file_host(true);
    }
    let source = f.directory.path("original-chair-target-λ.cdxml");
    f.command("document.save_cdxml", json!({"path":source}))?;
    f.command("document.open_cdxml", json!({"path":source}))?;
    f.command("ui.view", json!({"action":"actual"}))?;
    f.warm();
    chair_select(f, kind)?;
    f.warm();
    Ok(source)
}
fn chair_join(
    mut old: Value,
    points: &[[f64; 2]; 6],
    perimeter: [u32; 6],
    shared: usize,
    first_bond: u32,
) -> TestResult<Value> {
    for i in shared..6 {
        old["atoms"].as_array_mut().ok_or("atoms")?.push(json!({"id":perimeter[i],"element":6,
        "position":{"x_pt":points[i][0],"y_pt":points[i][1]},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":2,
        "show_non_terminal_carbon_labels":false}));
    }
    let mut id = first_bond;
    for i in 0..6 {
        if shared == 2 && i == 0 {
            continue;
        }
        old["bonds"]
            .as_array_mut()
            .ok_or("bonds")?
            .push(json!({"id":id,"begin":perimeter[i],"end":perimeter[(i+1)%6],"order":"single"}));
        id += 1;
    }
    Ok(old)
}
#[test]
fn both_chairs_attach_to_ordinary_terminal_spiro_and_fused_targets_without_losing_old_data()
-> TestResult {
    let q = 32.0 * 3.0_f64.sqrt();
    let hex = [
        [208.0, 300.0],
        [176.0, 300.0 - q],
        [112.0, 300.0 - q],
        [80.0, 300.0],
        [112.0, 300.0 + q],
        [176.0, 300.0 + q],
    ];
    for kind in [1, 2] {
        for target in ["terminal", "spiro", "fused"] {
            let old = if target == "terminal" {
                chair_old(&[[144.0, 300.0], [208.0, 300.0]], &[3, 3], &[[1, 2]])
            } else {
                chair_old(
                    &hex,
                    &[2; 6],
                    &[[1, 2], [2, 3], [3, 4], [4, 5], [5, 6], [6, 1]],
                )
            };
            let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
            let source = chair_native_source(&mut f, &old, kind, 64.0)?;
            let raw = fs::read(&source)?;
            chair_consume(
                &source,
                std::slice::from_ref(&old),
                &[if target == "terminal" {
                    "CC"
                } else {
                    "C1CCCCC1"
                }],
                &[if target == "terminal" {
                    "C2H6"
                } else {
                    "C6H12"
                }],
            )?;
            let before = f.state()?;
            let mut expected = old.clone();
            let (hit, points, perimeter, shared, next, smiles, formula) = match target {
                "terminal" => {
                    expected["atoms"][1]["explicit_hydrogens"] = json!(1);
                    (
                        [208.0, 300.0],
                        chair_outward_right(kind, [208.0, 300.0], 64.0),
                        [2, 3, 4, 5, 6, 7],
                        1,
                        2,
                        "CC1CCCCC1",
                        "C7H14",
                    )
                }
                "spiro" => {
                    expected["atoms"][0]["explicit_hydrogens"] = json!(0);
                    (
                        [208.0, 300.0],
                        chair_outward_right(kind, [208.0, 300.0], 64.0),
                        [1, 7, 8, 9, 10, 11],
                        1,
                        7,
                        "C1CCC2(CC1)CCCCC2",
                        "C11H20",
                    )
                }
                "fused" => {
                    expected["atoms"][0]["explicit_hydrogens"] = json!(1);
                    expected["atoms"][1]["explicit_hydrogens"] = json!(1);
                    (
                        [192.0, 300.0 - q / 2.0],
                        chair_affine(kind, hex[0], hex[1], 1.0),
                        [1, 2, 7, 8, 9, 10],
                        2,
                        7,
                        "C1CCC2CCCCC2C1",
                        "C10H18",
                    )
                }
                _ => return Err("declared attachment".into()),
            };
            chair_click(&mut f, hit)?;
            let after = chair_one_edit(&f, &before, &source, &raw)?;
            let expected = chair_join(expected, &points, perimeter, shared, next)?;
            chair_compare(&f, &before, std::slice::from_ref(&expected))?;
            chair_history(&mut f, &before, &after)?;
            chair_save(
                &mut f,
                &format!("chair-{kind}-{target}-λ.cdxml"),
                &before,
                &[expected],
                &[smiles],
                &[formula],
            )?;
        }
    }
    Ok(())
}
#[test]
fn nine_and_zero_are_ordered_fresh_bond_hotspots_not_global_or_repeating_tool_shortcuts()
-> TestResult {
    for (kind, code) in [(1, egui::Key::Num9), (2, egui::Key::Num0)] {
        // A reversed old B/E direction proves the contextual builder does not sort IDs.
        let old = chair_old(&[[144.0, 300.0], [208.0, 300.0]], &[3, 3], &[[2, 1]]);
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        let source = chair_native_source(&mut f, &old, kind, 96.0)?;
        let raw = fs::read(&source)?;
        f.command("ui.bond_tool", json!({"order":"triple"}))?;
        f.warm();
        let tool = f.app.engine.lock().map_err(|_| "engine")?.effective_tool();
        let bond = f.app.engine.lock().map_err(|_| "engine")?.bond_tool();
        let before = f.state()?;
        let area = canvas(&f.warm())?;
        let middle = chair_at(&f, area, [176.0, 300.0])?;
        // Future movement in this packet cannot supply an earlier key's target.
        // egui derives repeat from held keys, so keep this press down across packets.
        f.frame(vec![
            key(code, true, egui::Modifiers::NONE),
            egui::Event::PointerMoved(middle),
        ]);
        assert_eq!(
            f.state()?["revision"],
            before["revision"],
            "future pointer must not supply earlier chair key target"
        );
        chair_same(&f, &before)?;
        let repeated = egui::Event::Key {
            key: code,
            physical_key: Some(code),
            pressed: true,
            repeat: true,
            modifiers: egui::Modifiers::NONE,
        };
        f.frame(vec![repeated, key(code, false, egui::Modifiers::NONE)]);
        assert_eq!(
            f.state()?["revision"],
            before["revision"],
            "repeat stage must not author chair"
        );
        chair_same(&f, &before)?;
        f.frame(vec![
            egui::Event::PointerMoved(middle),
            key(code, true, egui::Modifiers::NONE),
            key(code, false, egui::Modifiers::NONE),
        ]);
        let after = chair_one_edit(&f, &before, &source, &raw)?;
        assert_eq!(
            f.app.engine.lock().map_err(|_| "engine")?.effective_tool(),
            tool
        );
        assert_eq!(f.app.engine.lock().map_err(|_| "engine")?.bond_tool(), bond);
        let points = chair_affine(kind, [208.0, 300.0], [144.0, 300.0], 1.0);
        let mut expected = old;
        expected["atoms"][0]["explicit_hydrogens"] = json!(2);
        expected["atoms"][1]["explicit_hydrogens"] = json!(2);
        let expected = chair_join(expected, &points, [2, 1, 3, 4, 5, 6], 2, 2)?;
        chair_compare(&f, &before, std::slice::from_ref(&expected))?;
        // The winning edit clears the hotspot; the SDK's cached position cannot re-arm it.
        f.frame(vec![
            key(code, true, egui::Modifiers::NONE),
            key(code, false, egui::Modifiers::NONE),
        ]);
        chair_same(&f, &after)?;
        chair_history(&mut f, &before, &after)?;
        chair_save(
            &mut f,
            &format!("chair-hotspot-{kind}-λ.cdxml"),
            &before,
            &[expected],
            &["C1CCCCC1"],
            &["C6H12"],
        )?;
    }
    Ok(())
}

// Separate ordinary native-bond label coverage; no hidden-label seed substitution.
#[test]
fn both_chairs_accept_default_native_labelled_terminal_and_fresh_hotspot_targets() -> TestResult {
    for kind in [1, 2] {
        for target in ["terminal", "hotspot"] {
            let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
            f.command("bond.create",json!({"page_id":1,"begin_x_pt":144,"begin_y_pt":300,"end_x_pt":208,"end_y_pt":300}))?;
            let source = chair_seed(&mut f, kind, Some(64.0))?;
            let raw = fs::read(&source)?;
            let mut old = chair_old(&[[144.0, 300.0], [208.0, 300.0]], &[3, 3], &[[1, 2]]);
            for atom in old["atoms"].as_array_mut().ok_or("atoms")? {
                atom.as_object_mut()
                    .ok_or("atom")?
                    .remove("show_non_terminal_carbon_labels");
            }
            assert_eq!(
                f.state()?["document"]["pages"][0]["fragments"],
                json!([old.clone()])
            );
            for atom in f.state()?["document"]["pages"][0]["fragments"][0]["atoms"]
                .as_array()
                .ok_or("atoms")?
            {
                assert!(
                    atom.get("show_terminal_carbon_labels").is_none()
                        && atom.get("show_non_terminal_carbon_labels").is_none()
                );
            }
            chair_consume(&source, std::slice::from_ref(&old), &["CC"], &["C2H6"])?;
            if target == "hotspot" {
                f.command("ui.bond_tool", json!({"order":"triple"}))?;
                f.warm();
            }
            let before = f.state()?;
            let tool = f.app.engine.lock().map_err(|_| "engine")?.effective_tool();
            let mut expected = old;
            let (points, perimeter, shared, first_bond, smiles, formula) = if target == "terminal" {
                expected["atoms"][1]["explicit_hydrogens"] = json!(1);
                chair_click(&mut f, [208.0, 300.0])?;
                (
                    chair_outward_right(kind, [208.0, 300.0], 64.0),
                    [2, 3, 4, 5, 6, 7],
                    1,
                    2,
                    "CC1CCCCC1",
                    "C7H14",
                )
            } else {
                expected["atoms"][0]["explicit_hydrogens"] = json!(2);
                expected["atoms"][1]["explicit_hydrogens"] = json!(2);
                let area = canvas(&f.warm())?;
                let middle = chair_at(&f, area, [176.0, 300.0])?;
                let code = if kind == 1 {
                    egui::Key::Num9
                } else {
                    egui::Key::Num0
                };
                // Coherent fresh down/up; no reliance on a supplied raw repeat flag.
                f.frame(vec![
                    egui::Event::PointerMoved(middle),
                    key(code, true, egui::Modifiers::NONE),
                    key(code, false, egui::Modifiers::NONE),
                ]);
                (
                    chair_affine(kind, [144.0, 300.0], [208.0, 300.0], 1.0),
                    [1, 2, 3, 4, 5, 6],
                    2,
                    2,
                    "C1CCCCC1",
                    "C6H12",
                )
            };
            let after = chair_one_edit(&f, &before, &source, &raw)?;
            assert_eq!(
                f.app.engine.lock().map_err(|_| "engine")?.effective_tool(),
                tool
            );
            let expected = chair_join(expected, &points, perimeter, shared, first_bond)?;
            chair_compare(&f, &before, std::slice::from_ref(&expected))?;
            let shown = f.warm();
            let area = canvas(&shown)?;
            assert!(
                shown
                    .shapes
                    .iter()
                    .any(|paint| matches!(&paint.shape,egui::Shape::Rect(r)
            if r.fill==egui::Color32::BLACK&&area.contains_rect(r.rect))),
                "old None carbon/H labels must remain actual visible literal glyphs"
            );
            assert!(
                f.app.error.is_none(),
                "the complete labelled candidate must pass unchanged guards: {:?}",
                f.app.error
            );
            chair_history(&mut f, &before, &after)?;
            chair_save(
                &mut f,
                &format!("chair-default-label-{kind}-{target}-λ.cdxml"),
                &before,
                &[expected],
                &[smiles],
                &[formula],
            )?;
        }
    }
    Ok(())
}

fn chair_declared_pair() -> Value {
    chair_old(&[[144.0, 300.0], [208.0, 300.0]], &[3, 3], &[[1, 2]])
}
fn chair_no_modal(f: &Fixture) -> TestResult {
    let state = f.state()?;
    assert!(state["ui_atom_label"].is_null());
    assert!(state["ui_drawing_settings"].is_null());
    assert!(state["ui_add_chain"].is_null());
    assert!(
        f.node("Element", &[egui::accesskit::Role::TextInput])
            .is_err()
    );
    assert!(
        f.node(
            "Carbon count (including anchor)",
            &[egui::accesskit::Role::TextInput]
        )
        .is_err()
    );
    Ok(())
}
fn chair_visible_error(f: &Fixture, out: &egui::FullOutput) -> TestResult {
    let message = f
        .app
        .error
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .ok_or("visible chair refusal")?;
    assert!(out.shapes.iter().any(|p|matches!(&p.shape,egui::Shape::Text(t)
        if t.galley.text().contains(message)&&p.clip_rect.intersects(p.shape.visual_bounding_rect()))));
    Ok(())
}
#[test]
fn chairs_refuse_unsupported_modifiers_and_protect_focused_modal_and_file_surfaces() -> TestResult {
    let modifiers = [
        egui::Modifiers::SHIFT,
        egui::Modifiers {
            ctrl: true,
            ..egui::Modifiers::NONE
        },
        egui::Modifiers::ALT,
        egui::Modifiers::COMMAND,
        egui::Modifiers {
            mac_cmd: true,
            ..egui::Modifiers::NONE
        },
    ];
    for kind in [1, 2] {
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        let old = chair_declared_pair();
        let source = chair_native_source(&mut f, &old, kind, 64.0)?;
        let raw = fs::read(&source)?;
        for modifier in modifiers {
            chair_select(&mut f, kind)?;
            let area = canvas(&f.warm())?;
            let atom = chair_at(&f, area, [208.0, 300.0])?;
            let moved = chair_at(&f, area, [208.0, 260.0])?;
            let before = f.state()?;
            assert!(
                f.app.error.is_none(),
                "the refusal must originate in this packet"
            );
            let out = f.frame(vec![
                egui::Event::PointerMoved(atom),
                chair_pointer(atom, true, modifier),
                egui::Event::PointerMoved(moved),
                chair_pointer(moved, false, modifier),
            ]);
            if modifier == egui::Modifiers::ALT {
                let points = chair_outward_right(kind, [0.0, 0.0], 40.0)
                    .map(|[x, y]| [208.0 + y, 260.0 - x]);
                let mut expected = old.clone();
                expected["atoms"][1]["explicit_hydrogens"] = json!(2);
                expected["bonds"]
                    .as_array_mut()
                    .ok_or("old bonds")?
                    .push(json!({"id":2,"begin":2,"end":3,"order":"single"}));
                let mut expected = chair_join(expected, &points, [3, 4, 5, 6, 7, 8], 0, 3)?;
                expected["atoms"][2]["explicit_hydrogens"] = json!(1);
                chair_compare(&f, &before, &[expected])?;
                chair_one_edit(&f, &before, &source, &raw)?;
                f.command("history.undo", json!({}))?;
                assert_eq!(f.state()?["document"], before["document"]);
                assert_eq!(f.state()?["file"], before["file"]);
                assert_eq!(f.state()?["source"], before["source"]);
                assert_eq!(f.state()?["redo_depth"], 1);
            } else {
                chair_same(&f, &before)?;
                chair_visible_error(&f, &out)?;
            }
            chair_no_modal(&f)?;
            assert_eq!(fs::read(&source)?, raw);
        }
        chair_select(&mut f, kind)?;
        let area = canvas(&f.warm())?;
        let middle = chair_at(&f, area, [176.0, 300.0])?;
        f.frame(vec![egui::Event::PointerMoved(middle)]);
        let before = f.state()?;
        let revision = before["revision"].clone();
        let opened = f.command(
            "ui.drawing_settings",
            json!({"action":"open","expected_revision":revision}),
        )?;
        let request = opened["ui_drawing_settings"]["id"]
            .as_u64()
            .ok_or("settings ID")?;
        chair_wait(&mut f, |f, _| {
            f.node("Bond length (pt)", &[egui::accesskit::Role::TextInput])
                .is_ok()
        })?;
        let (field, _) = f.node("Bond length (pt)", &[egui::accesskit::Role::TextInput])?;
        f.frame(vec![egui::Event::AccessKitActionRequest(
            egui::accesskit::ActionRequest {
                action: egui::accesskit::Action::Focus,
                target: field,
                data: None,
            },
        )]);
        f.frame(vec![
            egui::Event::PointerMoved(middle),
            key(egui::Key::Num9, true, egui::Modifiers::NONE),
            egui::Event::Text("9".into()),
            key(egui::Key::Num9, false, egui::Modifiers::NONE),
        ]);
        chair_same(&f, &before)?;
        assert_eq!(f.state()?["ui_drawing_settings"]["id"], request);
        f.command(
            "ui.drawing_settings",
            json!({"action":"cancel","expected_revision":revision,"request_id":request}),
        )?;
        // Explicit remote dismissal may not expose this packet's later chair hotspot key.
        f.frame(vec![
            egui::Event::PointerMoved(middle),
            key(egui::Key::Num0, true, egui::Modifiers::NONE),
            key(egui::Key::Num0, false, egui::Modifiers::NONE),
        ]);
        chair_same(&f, &before)?;
        chair_wait(&mut f, |f, _| {
            f.node("Bond length (pt)", &[egui::accesskit::Role::TextInput])
                .is_err()
        })?;
        chair_same(&f, &before)?;
        // A real File Open chooser owns the surface; no fabricated running worker is claimed.
        f.command(
            "ui.file",
            json!({"action":"open","expected_revision":f.state()?["revision"]}),
        )?;
        let busy = f.state()?;
        f.frame(vec![
            egui::Event::PointerMoved(middle),
            key(egui::Key::Num9, true, egui::Modifiers::NONE),
            key(egui::Key::Num9, false, egui::Modifiers::NONE),
        ]);
        chair_same(&f, &busy)?;
        assert!(f.state()?["ui_file"]["request"].is_object());
        f.choose("Cancel")?;
        chair_wait(&mut f, |f, _| {
            f.state().is_ok_and(|s| s["ui_file"]["request"].is_null())
        })?;
        chair_same(&f, &before)?;
        assert_eq!(fs::read(&source)?, raw);
    }
    Ok(())
}
#[test]
fn chair_gestures_never_revive_after_modals_context_aba_or_surface_loss() -> TestResult {
    for (index, change) in [
        "settings",
        "atom",
        "add_mode_aba",
        "view",
        "file",
        "close",
        "dpi",
        "zero",
        "pointer_gone",
        "focus",
        "escape",
    ]
    .into_iter()
    .enumerate()
    {
        let kind = if index % 2 == 0 { 1 } else { 2 };
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        let old = chair_declared_pair();
        let source = chair_native_source(&mut f, &old, kind, 64.0)?;
        let raw = fs::read(&source)?;
        let area = canvas(&f.warm())?;
        let a = [360.0, 300.0];
        let press = chair_at(&f, area, a)?;
        let before = f.state()?;
        f.frame(vec![
            egui::Event::PointerMoved(press),
            chair_pointer(press, true, egui::Modifiers::NONE),
        ]);
        chair_same(&f, &before)?;
        assert!(
            f.app.error.is_none(),
            "the original candidate must be valid"
        );
        let revision = before["revision"].clone();
        match change {
            "settings" => {
                let open = f.command(
                    "ui.drawing_settings",
                    json!({"action":"open","expected_revision":revision}),
                )?;
                f.command("ui.drawing_settings",json!({"action":"cancel","expected_revision":revision,"request_id":open["ui_drawing_settings"]["id"]}))?;
            }
            "atom" => {
                let open=f.command("ui.atom_label",json!({"action":"open","expected_revision":revision,"page_id":1,"fragment_id":1,"atom_id":1}))?;
                f.command("ui.atom_label",json!({"action":"cancel","expected_revision":revision,"request_id":open["ui_atom_label"]["id"]}))?;
            }
            "add_mode_aba" => {
                // Public Add requires Chain mode: this case honestly includes a tool ABA.
                f.command("ui.chain_tool", json!({"action":"select","kind":"acyclic"}))?;
                let open = f.command(
                    "ui.chain_tool",
                    json!({"action":"open_add","expected_revision":revision,"page_id":1,
                    "target":{"mode":"free","x_pt":360,"y_pt":500}}),
                )?;
                f.command("ui.chain_tool",json!({"action":"cancel_add","expected_revision":revision,"request_id":open["ui_add_chain"]["id"]}))?;
                f.command(
                    "ui.ring_tool",
                    json!({"kind":if kind==1 {"cyclohexane_chair_1"} else {"cyclohexane_chair_2"}}),
                )?;
            }
            "view" => {
                f.command("ui.view", json!({"action":"set_zoom","percent":125}))?;
                f.command("ui.view", json!({"action":"actual"}))?;
            }
            "file" => {
                let other = f.directory.path("chair-other.cdxml");
                f.command("document.save_cdxml", json!({"path":other}))?;
                f.command(
                    "document.save_cdxml",
                    json!({"path":source,"overwrite":true}),
                )?;
            }
            "close" => {
                f.command(
                    "document.close",
                    json!({"expected_revision":revision,"action":"request"}),
                )?;
                f.command(
                    "document.close",
                    json!({"expected_revision":revision,"action":"cancel"}),
                )?;
            }
            "dpi" => {
                f.dpi = 2.0;
                f.frame(vec![]);
                f.dpi = 1.0;
                f.frame(vec![]);
            }
            "zero" => {
                let width = f.width;
                f.width = 0.0;
                f.frame(vec![]);
                assert_eq!(f.context.screen_rect().width(), 0.0);
                f.width = width;
                f.frame(vec![]);
            }
            "pointer_gone" => {
                f.frame(vec![egui::Event::PointerGone]);
            }
            "focus" => {
                f.frame(vec![egui::Event::WindowFocused(false)]);
                f.frame(vec![egui::Event::WindowFocused(true)]);
            }
            "escape" => {
                f.frame(vec![
                    key(egui::Key::Escape, true, egui::Modifiers::NONE),
                    key(egui::Key::Escape, false, egui::Modifiers::NONE),
                ]);
            }
            _ => return Err("declared chair invalidation".into()),
        }
        let current = f.state()?;
        // Close/File setup may legitimately advance close_epoch; the held input
        // must preserve that NEW context rather than undoing its cancellation.
        for field in [
            "document",
            "revision",
            "undo_depth",
            "redo_depth",
            "source",
            "file",
            "close",
        ] {
            assert_eq!(
                current[field], before[field],
                "context setup unexpectedly authored {field}"
            );
        }
        f.frame(vec![chair_pointer(press, false, egui::Modifiers::NONE)]);
        f.warm();
        chair_same(&f, &current)?;
        chair_no_modal(&f)?;
        assert_eq!(fs::read(&source)?, raw);
        // A fresh click succeeds after cancellation without spending IDs for the old gesture.
        chair_click(&mut f, a)?;
        chair_one_edit(&f, &current, &source, &raw)?;
        let mut fresh = chair_literal(&chair_horizontal(kind, a, 64.0));
        fresh["id"] = json!(2);
        for atom in fresh["atoms"].as_array_mut().ok_or("atoms")? {
            atom["id"] = json!(atom["id"].as_u64().ok_or("ID")? + 2);
        }
        for bond in fresh["bonds"].as_array_mut().ok_or("bonds")? {
            bond["id"] = json!(bond["id"].as_u64().ok_or("ID")? + 1);
            bond["begin"] = json!(bond["begin"].as_u64().ok_or("begin")? + 2);
            bond["end"] = json!(bond["end"].as_u64().ok_or("end")? + 2);
        }
        chair_compare(&f, &before, &[old, fresh])?;
    }
    Ok(())
}
#[test]
fn chair_source_replacement_cancels_the_old_surface_and_foreign_sources_never_gain_authority()
-> TestResult {
    for (kind, format) in [(1, "mol"), (2, "smiles")] {
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        let old = chair_declared_pair();
        let source = chair_native_source(&mut f, &old, kind, 64.0)?;
        let original = fs::read(&source)?;
        let area = canvas(&f.warm())?;
        let p = chair_at(&f, area, [360.0, 300.0])?;
        f.frame(vec![
            egui::Event::PointerMoved(p),
            chair_pointer(p, true, egui::Modifiers::NONE),
        ]);
        let foreign = if format == "mol" {
            PathBuf::from(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../io/tests/fixtures/mol/ethane-v2000.mol"
            ))
        } else {
            let path = f.directory.path("original-chair-refusal.smi");
            fs::write(&path, b"CC\nCCC\n")?;
            path
        };
        let raw = fs::read(&foreign)?;
        f.command(
            if format == "mol" {
                "document.open_mol"
            } else {
                "document.open_smiles"
            },
            json!({"path":foreign,"discard_unsaved":true}),
        )?;
        let current = f.state()?;
        f.frame(vec![chair_pointer(p, false, egui::Modifiers::NONE)]);
        f.warm();
        chair_same(&f, &current)?;
        let error = f
            .app
            .engine
            .lock()
            .map_err(|_| "engine")?
            .dispatch(
                "ui.ring_tool",
                json!({"kind":if kind==1 {"cyclohexane_chair_1"} else {"cyclohexane_chair_2"}}),
            )
            .err()
            .ok_or("foreign no-op select must refuse")?;
        assert_eq!(error.info().code, "unsupported_content");
        chair_same(&f, &current)?;
        f.frame(vec![
            egui::Event::PointerMoved(p),
            key(egui::Key::Num9, true, egui::Modifiers::NONE),
            key(egui::Key::Num9, false, egui::Modifiers::NONE),
        ]);
        chair_same(&f, &current)?;
        assert_eq!(fs::read(&foreign)?, raw);
        assert_eq!(fs::read(&source)?, original);
        f.command(
            "document.open_cdxml",
            json!({"path":source,"discard_unsaved":true}),
        )?;
        f.command("ui.view", json!({"action":"actual"}))?;
        f.warm();
        chair_select(&mut f, kind)?;
        let before = f.state()?;
        chair_click(&mut f, [360.0, 300.0])?;
        chair_one_edit(&f, &before, &source, &original)?;
        let mut fresh = chair_literal(&chair_horizontal(kind, [360.0, 300.0], 64.0));
        fresh["id"] = json!(2);
        for atom in fresh["atoms"].as_array_mut().ok_or("atoms")? {
            atom["id"] = json!(atom["id"].as_u64().ok_or("ID")? + 2);
        }
        for bond in fresh["bonds"].as_array_mut().ok_or("bonds")? {
            bond["id"] = json!(bond["id"].as_u64().ok_or("ID")? + 1);
            bond["begin"] = json!(bond["begin"].as_u64().ok_or("begin")? + 2);
            bond["end"] = json!(bond["end"].as_u64().ok_or("end")? + 2);
        }
        chair_compare(&f, &before, &[old, fresh])?;
    }
    Ok(())
}

fn chair_physical_frame(
    f: &mut Fixture,
    origin: egui::Pos2,
    events: Vec<egui::Event>,
) -> egui::FullOutput {
    let mut input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            origin,
            egui::vec2(f.width, f.height),
        )),
        events,
        focused: true,
        ..Default::default()
    };
    input
        .viewports
        .entry(egui::ViewportId::ROOT)
        .or_default()
        .native_pixels_per_point = Some(f.dpi);
    let context = f.context.clone();
    let out = context.run(input, |c| f.app.update_workspace(c));
    if let Some(update) = &out.platform_output.accesskit_update {
        f.nodes.extend(update.nodes.iter().cloned());
    }
    out
}
fn chair_global_base(f: &Fixture, area: egui::Rect) -> TestResult<([f64; 2], f64)> {
    let view = f.view()?;
    let s = view["zoom_percent"].as_f64().ok_or("explicit zoom")? / 100.0;
    let c = [
        view["center_x_pt"].as_f64().ok_or("camera x")?,
        view["center_y_pt"].as_f64().ok_or("camera y")?,
    ];
    Ok((
        [
            f64::from(area.left()) + (f64::from(area.width().floor()) / 2.0 - c[0] * s).round(),
            f64::from(area.top()) + (f64::from(area.height().floor()) / 2.0 - c[1] * s).round(),
        ],
        s,
    ))
}
fn chair_screen(base: [f64; 2], s: f64, p: [f64; 2]) -> egui::Pos2 {
    egui::pos2((base[0] + p[0] * s) as f32, (base[1] + p[1] * s) as f32)
}
fn chair_device_ink(
    f: &Fixture,
    out: &egui::FullOutput,
    area: egui::Rect,
    base: [f64; 2],
    s: f64,
    points: &[[f64; 2]; 6],
) -> TestResult {
    let d = f64::from(f.dpi);
    let mut quads = out
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::Shape::Path(p)
                if p.closed
                    && p.fill == egui::Color32::BLACK
                    && p.points.len() == 4
                    && p.points.iter().all(|p| area.contains(*p)) =>
            {
                Some(p.points.clone())
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(quads.len(), 6);
    let all = quads.clone();
    let order = |a: &[f64; 2], b: &[f64; 2]| a[0].total_cmp(&b[0]).then(a[1].total_cmp(&b[1]));
    let inside = |q: &[egui::Pos2], p: egui::Pos2| {
        let mut pos = false;
        let mut neg = false;
        for i in 0..4 {
            let a = q[i];
            let b = q[(i + 1) % 4];
            let c = (b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x);
            pos |= c > 0.0;
            neg |= c < 0.0;
        }
        !(pos && neg)
    };
    for i in 0..6 {
        let a = [
            (base[0] + points[i][0] * s) * d,
            (base[1] + points[i][1] * s) * d,
        ];
        let b = [
            (base[0] + points[(i + 1) % 6][0] * s) * d,
            (base[1] + points[(i + 1) % 6][1] * s) * d,
        ];
        let delta = [b[0] - a[0], b[1] - a[1]];
        let length = delta[0].hypot(delta[1]);
        let n = [-delta[1] / length, delta[0] / length];
        let half = s * d / 2.0;
        let mut expected = vec![
            [a[0] - n[0] * half, a[1] - n[1] * half],
            [b[0] - n[0] * half, b[1] - n[1] * half],
            [b[0] + n[0] * half, b[1] + n[1] * half],
            [a[0] + n[0] * half, a[1] + n[1] * half],
        ];
        expected.sort_by(order);
        let found = quads
            .iter()
            .position(|q| {
                let mut a = q
                    .iter()
                    .map(|p| [f64::from(p.x) * d, f64::from(p.y) * d])
                    .collect::<Vec<_>>();
                a.sort_by(order);
                a.iter().zip(&expected).all(|(a, e)| {
                    (a[0] - e[0]).abs() <= 1.0 / 256.0 && (a[1] - e[1]).abs() <= 1.0 / 256.0
                })
            })
            .ok_or("global device chair corners disagree with independent affine rectangle")?;
        quads.swap_remove(found);
        let m = [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0];
        let ink = egui::pos2(
            ((m[0].floor() + 0.5) / d) as f32,
            ((m[1].floor() + 0.5) / d) as f32,
        );
        let white = egui::pos2(
            (((m[0] + n[0] * 3.0).floor() + 0.5) / d) as f32,
            (((m[1] + n[1] * 3.0).floor() + 0.5) / d) as f32,
        );
        assert!(
            all.iter().any(|q| inside(q, ink)),
            "predicted global pixel carries actual ink"
        );
        assert!(
            all.iter().all(|q| !inside(q, white)),
            "the independent exterior pixel stays clear"
        );
        assert!(out.shapes.iter().any(|p|matches!(&p.shape,egui::Shape::Rect(r) if r.fill==egui::Color32::WHITE&&r.rect.contains(white)&&p.clip_rect.contains(white))));
    }
    let glyphs = out
        .shapes
        .iter()
        .filter_map(|p| match &p.shape {
            egui::Shape::Rect(r)
                if r.fill == egui::Color32::BLACK && area.contains_rect(r.rect) =>
            {
                Some(r.rect)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(
        !glyphs.is_empty(),
        "the whole preview keeps the original O/H2 glyphs"
    );
    for r in glyphs {
        for x in [r.left(), r.top(), r.right(), r.bottom()] {
            let p = f64::from(x) * d;
            assert!((p - p.round()).abs() <= 1.0 / 256.0, "global glyph lattice");
        }
    }
    Ok(())
}
#[test]
fn chair_two_x_and_fractional_origin_preview_pick_history_files_and_logical_guards_are_coherent()
-> TestResult {
    let water = json!({"id":1,"atoms":[{"id":1,"element":8,"position":{"x_pt":288.0,"y_pt":480.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":2}],"bonds":[]});
    for (kind, origin) in [(1, egui::Pos2::ZERO), (2, egui::pos2(0.125, 0.375))] {
        let mut f = Fixture::new(1160.0, 800.0, 2.0)?;
        let source = chair_native_source(&mut f, &water, kind, 64.0)?;
        let raw = fs::read(&source)?;
        f.command(
            "ui.view",
            json!({"action":"pan","center_x_pt":288,"center_y_pt":432}),
        )?;
        chair_physical_frame(&mut f, origin, vec![]);
        let ready = chair_physical_frame(&mut f, origin, vec![]);
        assert_eq!(ready.pixels_per_point, 2.0);
        let area = canvas(&ready)?;
        let phase =
            [f64::from(area.left()) * 2.0, f64::from(area.top()) * 2.0].map(|x| x - x.floor());
        if kind == 2 {
            assert!(
                phase.iter().any(|x| (0.2..=0.8).contains(x)),
                "actual canvas must retain substantial fractional device origin"
            );
        }
        let (base, s) = chair_global_base(&f, area)?;
        assert_eq!(s, 1.0);
        let screen = chair_screen(base, s, [300.0, 300.0]);
        // Invert the ACTUAL f32 event coordinates, not ideal decimal input, independently.
        let a = [
            (f64::from(screen.x) - base[0]) / s,
            (f64::from(screen.y) - base[1]) / s,
        ];
        let points = chair_horizontal(kind, a, 64.0);
        let before = f.state()?;
        let view = f.view()?;
        let held = chair_physical_frame(
            &mut f,
            origin,
            vec![
                egui::Event::PointerMoved(screen),
                chair_pointer(screen, true, egui::Modifiers::NONE),
            ],
        );
        assert_eq!(f.state()?, before);
        assert_eq!(f.view()?, view);
        chair_device_ink(&f, &held, area, base, s, &points)?;
        chair_physical_frame(
            &mut f,
            origin,
            vec![chair_pointer(screen, false, egui::Modifiers::NONE)],
        );
        let after = chair_one_edit(&f, &before, &source, &raw)?;
        let mut literal = chair_literal(&points);
        literal["id"] = json!(2);
        for atom in literal["atoms"].as_array_mut().ok_or("atoms")? {
            atom["id"] = json!(atom["id"].as_u64().ok_or("ID")? + 1);
        }
        for bond in literal["bonds"].as_array_mut().ok_or("bonds")? {
            bond["begin"] = json!(bond["begin"].as_u64().ok_or("ID")? + 1);
            bond["end"] = json!(bond["end"].as_u64().ok_or("ID")? + 1);
        }
        chair_compare(&f, &before, &[water.clone(), literal.clone()])?;
        f.command("ui.bond_tool", json!({"order":"single"}))?;
        let midpoint = [
            (points[0][0] + points[1][0]) / 2.0,
            (points[0][1] + points[1][1]) / 2.0,
        ];
        let p = chair_screen(base, s, midpoint);
        chair_physical_frame(&mut f, origin, vec![egui::Event::PointerMoved(p)]);
        assert!(
            matches!(f.app.drawing.hover_target(),Some(bondline_render::HitTarget::Bond {fragment_id,bond_id}) if fragment_id.get()==2&&bond_id.get()==1)
        );
        chair_same(&f, &after)?;
        chair_physical_frame(
            &mut f,
            origin,
            vec![
                key(egui::Key::Z, true, egui::Modifiers::COMMAND),
                key(egui::Key::Z, false, egui::Modifiers::COMMAND),
            ],
        );
        assert_eq!(f.state()?["document"], before["document"]);
        assert_eq!(f.state()?["file"], before["file"]);
        assert_eq!(f.state()?["redo_depth"], 1);
        let m = egui::Modifiers {
            shift: true,
            ..egui::Modifiers::COMMAND
        };
        chair_physical_frame(
            &mut f,
            origin,
            vec![key(egui::Key::Z, true, m), key(egui::Key::Z, false, m)],
        );
        assert_eq!(f.state()?["document"], after["document"]);
        assert_eq!(f.state()?["undo_depth"], 1);
        assert_eq!(f.state()?["redo_depth"], 0);
        if kind == 1 {
            let saved = f.state()?;
            let view = f.view()?;
            let cpu = f.command(
                "render.capture",
                json!({"page_id":1,"width_px":1024,"height_px":1024,"backend":"cpu"}),
            )?;
            let gpu = f.command(
                "render.capture",
                json!({"page_id":1,"width_px":1024,"height_px":1024,"backend":"gpu"}),
            )?;
            assert!(
                cpu["capture"]["png_base64"]
                    .as_str()
                    .is_some_and(|s| !s.is_empty())
            );
            assert!(
                cpu["capture"]["png_base64"] == gpu["capture"]["png_base64"],
                "actual complete CPU/GPU PNG equality"
            );
            assert!(
                gpu["capture"]["adapter"]["name"]
                    .as_str()
                    .is_some_and(|s| !s.is_empty())
            );
            assert!(
                gpu["capture"]["adapter"]["backend"]
                    .as_str()
                    .is_some_and(|s| !s.is_empty())
            );
            assert_eq!(f.state()?, saved);
            assert_eq!(f.view()?, view);
        }
        let output = f.directory.path(&format!("physical-chair-{kind}-λ.cdxml"));
        f.command("document.save_cdxml", json!({"path":output}))?;
        chair_consume(
            &output,
            &[water.clone(), literal.clone()],
            &["O", "C1CCCCC1"],
            &["H2O", "C6H12"],
        )?;
        assert_eq!(fs::read(&source)?, raw);
        f.command("document.open_cdxml", json!({"path":output}))?;
        chair_compare(&f, &before, &[water.clone(), literal])?;
        assert_eq!(f.state()?["file"]["dirty"], false);
        assert_eq!(f.state()?["undo_depth"], 0);
        // Qualify a new valid surface after Open, before the negative label guard.
        f.command("ui.view", json!({"action":"actual"}))?;
        f.command(
            "ui.view",
            json!({"action":"pan","center_x_pt":288,"center_y_pt":432}),
        )?;
        chair_physical_frame(&mut f, origin, vec![]);
        f.command(
            "ui.ring_tool",
            json!({"kind":if kind==1 {"cyclohexane_chair_1"} else {"cyclohexane_chair_2"}}),
        )?;
        f.command("ui.view", json!({"action":"set_zoom","percent":50}))?;
        let guarded = f.state()?;
        let out = chair_physical_frame(&mut f, origin, vec![]);
        let error = bondline_render::Error::LabelScale.to_string();
        assert!(
            out.shapes.iter().any(
                |p| matches!(&p.shape,egui::Shape::Text(t) if t.galley.text().contains(&error))
            ),
            "physical pixels do not waive logical label readability"
        );
        let area = canvas(&out)?;
        let (base, s) = chair_global_base(&f, area)?;
        let p = chair_screen(base, s, [300.0, 300.0]);
        chair_physical_frame(
            &mut f,
            origin,
            vec![
                chair_pointer(p, true, egui::Modifiers::NONE),
                chair_pointer(p, false, egui::Modifiers::NONE),
            ],
        );
        chair_same(&f, &guarded)?;
        f.command("ui.view", json!({"action":"actual"}))?;
        let out = chair_physical_frame(&mut f, origin, vec![]);
        let area = canvas(&out)?;
        let (base, s) = chair_global_base(&f, area)?;
        let outside = chair_screen(base, s, [-10.0, 360.0]);
        let inside = chair_screen(base, s, [60.0, 430.0]);
        assert!(area.contains(outside) && area.contains(inside));
        let before = f.state()?;
        chair_physical_frame(
            &mut f,
            origin,
            vec![
                chair_pointer(outside, true, egui::Modifiers::NONE),
                egui::Event::PointerMoved(inside),
                chair_pointer(inside, false, egui::Modifiers::NONE),
            ],
        );
        chair_same(&f, &before)?;
    }
    Ok(())
}
