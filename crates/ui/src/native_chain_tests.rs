//! Independent initial actual-App chain control declaration; no future Rust API.
use super::{Fixture, TestResult, button};
use eframe::egui;

#[test]
fn acyclic_chain_is_an_actionable_accessible_native_tool_without_authoring() -> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let shown = f.warm();
    let before = f.state()?;
    let (id, rect) = f.node(
        "Acyclic chain",
        &[
            egui::accesskit::Role::Button,
            egui::accesskit::Role::CheckBox,
        ],
    )?;
    assert!(
        !f.nodes
            .get(&id)
            .ok_or("current Acyclic chain control")?
            .is_disabled()
    );
    assert!(
        rect.is_positive()
            && egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1160.0, 800.0))
                .contains_rect(rect)
    );
    assert!(
        shown.shapes.iter().any(|paint| {
            if !paint.clip_rect.intersects(rect) {
                return false;
            }
            match &paint.shape {
                egui::Shape::LineSegment { points, stroke } => {
                    stroke.width > 0.0 && points.iter().all(|point| rect.contains(*point))
                }
                egui::Shape::Path(path) => {
                    path.stroke.width > 0.0
                        && !path.points.is_empty()
                        && path.points.iter().all(|point| rect.contains(*point))
                }
                _ => false,
            }
        }),
        "the accessible chain control must have visible in-bounds procedural strokes"
    );
    f.frame(vec![
        egui::Event::PointerMoved(rect.center()),
        button(rect.center(), true),
    ]);
    f.frame(vec![button(rect.center(), false)]);
    let controls = f.command("ui.controls", serde_json::json!({}))?;
    let selected = controls["controls"]
        .as_array()
        .ok_or("central controls")?
        .iter()
        .find(|row| row["id"] == "chain.acyclic")
        .ok_or("central Acyclic chain binding")?;
    assert_eq!(selected["enabled"], true);
    assert_eq!(selected["selected"], true);
    assert_eq!(
        selected["command"],
        serde_json::json!({
            "command":"ui.chain_tool","params":{"action":"select","kind":"acyclic"}
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
            "tool selection authored {field}"
        );
    }
    Ok(())
}

// Four independent complete interaction families declared after the locked interface.
use super::{canvas, key, projected};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command as Process, Stdio},
    time::{Duration, Instant},
};

fn chain_wait(
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
                "chain App condition did not become ready: {:?}",
                f.app.error
            )
            .into());
        }
        std::thread::yield_now();
    }
}
fn chain_text(output: &egui::FullOutput, text: &str) -> bool {
    output.shapes.iter().any(|paint| {
        matches!(&paint.shape,
        egui::Shape::Text(shape) if shape.galley.text().contains(text))
    })
}
fn chain_seed(f: &mut Fixture, kind: &str) -> TestResult<PathBuf> {
    f.command(
        "document.set_drawing_settings",
        json!({
            "expected_revision":f.state()?["revision"],
            "bond_length_pt":48,"chain_angle_degrees":90
        }),
    )?;
    let source = f.directory.path("original-chain-λ.cdxml");
    f.command("document.save_cdxml", json!({"path":source}))?;
    f.command("document.open_cdxml", json!({"path":source}))?;
    f.command("ui.view", json!({"action":"actual"}))?;
    f.warm();
    chain_choose(
        f,
        if kind == "acyclic" {
            "Acyclic chain"
        } else {
            "Snaking chain"
        },
    )?;
    f.command(
        "ui.chain_tool",
        json!({
            "action":"constraints","fixed_lengths":true,"fixed_angles":true
        }),
    )?;
    f.warm();
    assert_eq!(f.state()?["undo_depth"], 0);
    assert_eq!(f.state()?["file"]["dirty"], false);
    Ok(source)
}
fn chain_choose(f: &mut Fixture, label: &str) -> TestResult {
    chain_wait(f, |f, _| {
        f.node(
            label,
            &[
                egui::accesskit::Role::Button,
                egui::accesskit::Role::CheckBox,
            ],
        )
        .is_ok_and(|(id, _)| f.nodes.get(&id).is_some_and(|node| !node.is_disabled()))
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
    f.warm();
    Ok(())
}
fn chain_at(f: &Fixture, area: egui::Rect, point: [f64; 2]) -> TestResult<egui::Pos2> {
    let state = f.state()?;
    let size = &state["document"]["pages"][0]["size"];
    let view = f.view()?;
    let center = match (view["center_x_pt"].as_f64(), view["center_y_pt"].as_f64()) {
        (Some(x), Some(y)) => Some([x, y]),
        _ => None,
    };
    let screen = projected(
        area,
        [
            size["width_pt"].as_f64().ok_or("width")?,
            size["height_pt"].as_f64().ok_or("height")?,
        ],
        view["zoom_percent"]
            .as_f64()
            .ok_or("explicit chain test camera")?,
        center,
        point,
    );
    assert!(
        area.contains(screen),
        "literal point {point:?} must be on the actual canvas"
    );
    Ok(screen)
}
fn chain_pointer(pos: egui::Pos2, pressed: bool, modifiers: egui::Modifiers) -> egui::Event {
    egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers,
    }
}
fn chain_press(f: &mut Fixture, area: egui::Rect, point: [f64; 2]) -> TestResult {
    let pos = chain_at(f, area, point)?;
    f.frame(vec![
        egui::Event::PointerMoved(pos),
        chain_pointer(pos, true, egui::Modifiers::NONE),
    ]);
    Ok(())
}
fn chain_move(f: &mut Fixture, area: egui::Rect, point: [f64; 2]) -> TestResult<egui::FullOutput> {
    let pos = chain_at(f, area, point)?;
    Ok(f.frame(vec![egui::Event::PointerMoved(pos)]))
}
fn chain_release(
    f: &mut Fixture,
    area: egui::Rect,
    point: [f64; 2],
) -> TestResult<egui::FullOutput> {
    let pos = chain_at(f, area, point)?;
    Ok(f.frame(vec![chain_pointer(pos, false, egui::Modifiers::NONE)]))
}
fn chain_click(f: &mut Fixture, point: [f64; 2], modifiers: egui::Modifiers) -> TestResult {
    let area = canvas(&f.warm())?;
    let pos = chain_at(f, area, point)?;
    f.frame(vec![
        egui::Event::PointerMoved(pos),
        chain_pointer(pos, true, modifiers),
    ]);
    f.frame(vec![chain_pointer(pos, false, modifiers)]);
    Ok(())
}
fn chain_literal(fragment: u32, first_atom: u32, first_bond: u32, points: &[[f64; 2]]) -> Value {
    json!({"id":fragment,
        "atoms":points.iter().enumerate().map(|(i,p)| json!({
            "id":first_atom+i as u32,"element":6,"position":{"x_pt":p[0],"y_pt":p[1]},
            "isotope":null,"charge":0,"radical":"none",
            "explicit_hydrogens":if i==0 || i+1==points.len() {3} else {2},
            "show_terminal_carbon_labels":false,"show_non_terminal_carbon_labels":false
        })).collect::<Vec<_>>(),
        "bonds":points.windows(2).enumerate().map(|(i,_)| json!({
            "id":first_bond+i as u32,"begin":first_atom+i as u32,
            "end":first_atom+i as u32+1,"order":"single"
        })).collect::<Vec<_>>()})
}
fn chain_expect_document(f: &Fixture, before: &Value, fragments: &[Value]) -> TestResult<Value> {
    let mut expected = before["document"].clone();
    expected["pages"][0]["fragments"] = json!(fragments);
    let mut actual = f.state()?["document"].clone();
    let actual_fragments = actual["pages"][0]["fragments"]
        .as_array_mut()
        .ok_or("fragments")?;
    assert_eq!(actual_fragments.len(), fragments.len());
    for (actual, expected) in actual_fragments.iter_mut().zip(fragments) {
        let atoms = actual["atoms"].as_array_mut().ok_or("atoms")?;
        let expected_atoms = expected["atoms"].as_array().ok_or("literal atoms")?;
        assert_eq!(atoms.len(), expected_atoms.len());
        for (a, e) in atoms.iter_mut().zip(expected_atoms) {
            for axis in ["x_pt", "y_pt"] {
                let x = a["position"][axis].as_f64().ok_or("actual point")?;
                let y = e["position"][axis].as_f64().ok_or("literal point")?;
                assert!((x - y).abs() <= 1e-9, "literal {axis}: {x} != {y}");
            }
            a["position"] = e["position"].clone();
        }
    }
    assert_eq!(
        actual, expected,
        "all IDs/directed edges/H/flags/settings/pages must be literal"
    );
    Ok(expected)
}
fn chain_authored_same(f: &Fixture, before: &Value) -> TestResult {
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
fn chain_one_edit(f: &Fixture, before: &Value, source: &Path, raw: &[u8]) -> TestResult<Value> {
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
    assert_eq!(
        fs::read(source)?,
        raw,
        "a gesture must never overwrite its associated source"
    );
    assert!(f.app.error.is_none(), "{:?}", f.app.error);
    Ok(after)
}
fn chain_preview_axes(
    f: &Fixture,
    shown: &egui::FullOutput,
    area: egui::Rect,
    points: &[[f64; 2]],
    count: usize,
) -> TestResult {
    assert_eq!(
        canvas(shown)?,
        area,
        "the counter must not reflow the actual canvas"
    );
    assert!(
        chain_text(shown, &format!("Carbons: {count} (including anchor)")),
        "painted live count"
    );
    let mut axes = shown
        .shapes
        .iter()
        .filter_map(|paint| {
            if !paint.clip_rect.intersects(area) {
                return None;
            }
            match &paint.shape {
                egui::Shape::Path(path)
                    if path.closed
                        && path.fill == egui::Color32::BLACK
                        && path.points.len() == 4
                        && path.points.iter().all(|p| area.contains(*p)) =>
                {
                    Some([
                        path.points[0].lerp(path.points[3], 0.5),
                        path.points[1].lerp(path.points[2], 0.5),
                    ])
                }
                _ => None,
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(
        axes.len(),
        points.len() - 1,
        "all and only the literal candidate Single strokes"
    );
    let tolerance = 1.0 / (256.0 * f.dpi);
    for pair in points.windows(2) {
        let a = chain_at(f, area, pair[0])?;
        let b = chain_at(f, area, pair[1])?;
        let found = axes.iter().position(|[x, y]| {
            (x.distance(a) <= tolerance && y.distance(b) <= tolerance)
                || (x.distance(b) <= tolerance && y.distance(a) <= tolerance)
        });
        let found = found
            .ok_or("preview stroke axis differs from independently projected literal endpoints")?;
        axes.swap_remove(found);
    }
    Ok(())
}
fn chain_history(f: &mut Fixture, before: &Value, edited: &Value) -> TestResult {
    f.frame(vec![
        key(egui::Key::Z, true, egui::Modifiers::COMMAND),
        key(egui::Key::Z, false, egui::Modifiers::COMMAND),
    ]);
    let undo = f.state()?;
    assert_eq!(undo["document"], before["document"]);
    assert_eq!(undo["source"], before["source"]);
    assert_eq!(undo["file"], before["file"]);
    assert_eq!(undo["undo_depth"], before["undo_depth"]);
    assert_eq!(undo["redo_depth"], 1);
    let shifted = egui::Modifiers {
        shift: true,
        ..egui::Modifiers::COMMAND
    };
    f.frame(vec![
        key(egui::Key::Z, true, shifted),
        key(egui::Key::Z, false, shifted),
    ]);
    let redo = f.state()?;
    assert_eq!(redo["document"], edited["document"]);
    assert_eq!(redo["source"], edited["source"]);
    assert_eq!(redo["file"], edited["file"]);
    assert_eq!(redo["undo_depth"], edited["undo_depth"]);
    assert_eq!(redo["redo_depth"], 0);
    assert_eq!(
        redo["revision"],
        edited["revision"].as_u64().ok_or("revision")? + 2
    );
    Ok(())
}
fn chain_consume(path: &Path, fragments: &[Value], formulas: &[&str]) -> TestResult {
    let expected = json!({"fragments":fragments,"formulas":formulas});
    let mut child=Process::new(std::env::var_os("BONDLINE_PYTHON").ok_or("pinned external Python")?)
        .args(["-B","-c",r#"
import json,subprocess,sys,xml.parsers.expat as expat
from pathlib import Path
from decimal import Decimal
from rdkit import Chem,rdBase
from rdkit.Chem import rdMolDescriptors
assert rdBase.rdkitVersion=='2026.09.2'
expected=json.load(sys.stdin); source=Path(sys.argv[1]).read_bytes()
want=expected['fragments']; plain=json.loads(json.dumps(want))
for fragment in plain:
 for atom in fragment['atoms']:
  assert atom.pop('show_terminal_carbon_labels') is False
  assert atom.pop('show_non_terminal_carbon_labels') is False
read=subprocess.run([sys.executable,'-B',sys.argv[2]],input=source,capture_output=True,check=True)
actual=json.loads(read.stdout)
assert len(actual)==len(plain)
for a,e in zip(actual,plain):
 assert len(a['atoms'])==len(e['atoms'])
 for aa,ee in zip(a['atoms'],e['atoms']):
  for axis in ('x_pt','y_pt'):
   assert abs(Decimal(str(aa['position'][axis]))-Decimal(str(ee['position'][axis])))<=Decimal('1e-9')
  aa['position']=ee['position']
assert actual==plain,'independent complete raw directed graph/H/metadata'
nodes=[]; root=[]
def start(name,attrs):
 if name=='CDXML': root.append(attrs)
 if name=='n': nodes.append(attrs)
p=expat.ParserCreate();p.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
p.StartElementHandler=start;p.Parse(source,True)
assert len(root)==1 and Decimal(root[0]['BondLength'])==48 and Decimal(root[0]['ChainAngle'])==90
flat=[a for f in want for a in f['atoms']];assert len(nodes)==len(flat)
for node,a in zip(nodes,flat):
 assert node['ShowTerminalCarbonLabels']=='no' and node['ShowNonTerminalCarbonLabels']=='no'
 values=list(map(Decimal,node['p'].split()));assert len(values)==2
 for v,axis in zip(values,('x_pt','y_pt')):
  assert v.is_finite() and abs(v-Decimal(str(a['position'][axis])))<=Decimal('1e-9')
ms=Chem.MolsFromCDXML(source.decode('utf-8'))
assert len(ms)==len(want)==len(expected['formulas'])
for m,g,formula in zip(ms,want,expected['formulas']):
 assert m.GetNumAtoms()==len(g['atoms']) and m.GetNumBonds()==len(g['bonds'])
 assert [a.GetAtomicNum() for a in m.GetAtoms()]==[6]*len(g['atoms'])
 assert [a.GetTotalNumHs() for a in m.GetAtoms()]==[a['explicit_hydrogens'] for a in g['atoms']]
 assert all(a.GetFormalCharge()==0 and a.GetIsotope()==0 and a.GetNumRadicalElectrons()==0 for a in m.GetAtoms())
 assert all(b.GetBondTypeAsDouble()==1 for b in m.GetBonds())
 assert Chem.MolToSmiles(m)=='C'*len(g['atoms'])
 assert rdMolDescriptors.CalcMolFormula(m)==formula
"#])
        .arg(path).arg(concat!(env!("CARGO_MANIFEST_DIR"),"/../../xtask/oracles/cdxml_graph.py"))
        .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
    serde_json::to_writer(
        child.stdin.take().ok_or("literal consumer stdin")?,
        &expected,
    )?;
    let result = child.wait_with_output()?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(())
}
fn chain_save_reopen(
    f: &mut Fixture,
    name: &str,
    fragments: &[Value],
    formulas: &[&str],
) -> TestResult {
    let output = f.directory.path(name);
    let before = f.state()?;
    f.command("document.save_cdxml", json!({"path":output}))?;
    assert_eq!(f.state()?["document"], before["document"]);
    assert_eq!(f.state()?["file"]["dirty"], false);
    chain_consume(&output, fragments, formulas)?;
    f.command("document.open_cdxml", json!({"path":output}))?;
    assert_eq!(f.state()?["file"]["dirty"], false);
    assert_eq!(f.state()?["undo_depth"], 0);
    assert_eq!(f.state()?["redo_depth"], 0);
    chain_expect_document(f, &before, fragments)?;
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
fn acyclic_fixed_drag_previews_literal_four_carbon_geometry_then_one_edit_and_native_history()
-> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let source = chain_seed(&mut f, "acyclic")?;
    let raw = fs::read(&source)?;
    let area = canvas(&f.warm())?;
    let before = f.state()?;
    let view = f.view()?;
    let model = f
        .app
        .engine
        .lock()
        .map_err(|_| "engine")?
        .stored_document()?
        .model_snapshot();
    let points = [
        [144.0, 144.0],
        [192.0, 144.0],
        [192.0, 192.0],
        [240.0, 192.0],
    ];
    chain_press(&mut f, area, [144.0, 144.0])?;
    chain_move(&mut f, area, [168.0, 168.0])?;
    let preview = chain_move(&mut f, area, [216.0, 216.0])?;
    assert_eq!(
        f.state()?,
        before,
        "preview must not mutate even transient Engine state"
    );
    assert_eq!(f.view()?, view);
    assert!(std::sync::Arc::ptr_eq(
        &model,
        &f.app
            .engine
            .lock()
            .map_err(|_| "engine")?
            .stored_document()?
            .model_snapshot()
    ));
    chain_preview_axes(&f, &preview, area, &points, 4)?;
    chain_release(&mut f, area, [216.0, 216.0])?;
    let edited = chain_one_edit(&f, &before, &source, &raw)?;
    let expected = chain_literal(1, 1, 1, &points);
    chain_expect_document(&f, &before, std::slice::from_ref(&expected))?;
    assert_eq!(f.view()?, view);
    chain_history(&mut f, &before, &edited)?;
    chain_save_reopen(&mut f, "chain-acyclic-λ.cdxml", &[expected], &["C4H10"])
}

#[test]
fn acyclic_free_length_drag_normalizes_the_actual_terminal_instead_of_projected_step_span()
-> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let source = chain_seed(&mut f, "acyclic")?;
    f.command(
        "ui.chain_tool",
        json!({"action":"constraints","fixed_lengths":false,"fixed_angles":false}),
    )?;
    let raw = fs::read(&source)?;
    let area = canvas(&f.warm())?;
    let before = f.state()?;
    let points = [
        [144.0, 144.0],
        [184.0, 124.0],
        [204.0, 164.0],
        [244.0, 144.0],
    ];
    chain_press(&mut f, area, points[0])?;
    let preview = chain_move(&mut f, area, [244.0, 144.0])?;
    assert_eq!(f.state()?, before);
    chain_preview_axes(&f, &preview, area, &points, 4)?;
    chain_release(&mut f, area, [244.0, 144.0])?;
    chain_one_edit(&f, &before, &source, &raw)?;
    let expected = chain_literal(1, 1, 1, &points);
    chain_expect_document(&f, &before, std::slice::from_ref(&expected))?;
    chain_save_reopen(&mut f, "chain-endpoint-λ.cdxml", &[expected], &["C4H10"])
}

#[test]
fn snaking_preserves_a_literal_turning_walk_and_backtracks_before_advancing_in_one_edit()
-> TestResult {
    let full = [
        [144.0, 144.0],
        [192.0, 144.0],
        [192.0, 192.0],
        [240.0, 192.0],
        [240.0, 144.0],
    ];
    for retract in [false, true] {
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        let source = chain_seed(&mut f, "snaking")?;
        let raw = fs::read(&source)?;
        let area = canvas(&f.warm())?;
        let before = f.state()?;
        chain_press(&mut f, area, full[0])?;
        let mut preview = chain_move(&mut f, area, full[1])?;
        for point in &full[2..] {
            preview = chain_move(&mut f, area, *point)?;
        }
        assert_eq!(f.state()?, before);
        chain_preview_axes(&f, &preview, area, &full, 5)?;
        let (points, last, formula): (&[[f64; 2]], [f64; 2], &str) = if retract {
            chain_move(&mut f, area, full[3])?;
            preview = chain_move(&mut f, area, full[2])?;
            chain_preview_axes(&f, &preview, area, &full[..3], 3)?;
            (&full[..3], full[2], "C3H8")
        } else {
            (&full, full[4], "C5H12")
        };
        assert_eq!(
            f.state()?,
            before,
            "backtracking is still only a disposable preview"
        );
        chain_release(&mut f, area, last)?;
        let edited = chain_one_edit(&f, &before, &source, &raw)?;
        let expected = chain_literal(1, 1, 1, points);
        chain_expect_document(&f, &before, std::slice::from_ref(&expected))?;
        chain_history(&mut f, &before, &edited)?;
        chain_save_reopen(
            &mut f,
            if retract {
                "chain-backtrack-λ.cdxml"
            } else {
                "chain-snaking-λ.cdxml"
            },
            &[expected],
            &[formula],
        )?;
    }
    Ok(())
}

fn chain_count_node(f: &Fixture) -> TestResult<egui::accesskit::NodeId> {
    Ok(f.node(
        "Carbon count (including anchor)",
        &[egui::accesskit::Role::TextInput],
    )?
    .0)
}
fn chain_count(f: &Fixture) -> TestResult<String> {
    Ok(f.nodes
        .get(&chain_count_node(f)?)
        .and_then(|n| n.value())
        .ok_or("count AX value")?
        .to_owned())
}
fn chain_form(f: &mut Fixture) -> TestResult {
    chain_wait(f, |f, out| {
        chain_text(out, "Add Chain") && chain_count_node(f).is_ok()
    })?;
    Ok(())
}
fn chain_closed(f: &mut Fixture) -> TestResult {
    chain_wait(f, |f, _| chain_count_node(f).is_err())?;
    Ok(())
}
fn chain_set_count(f: &mut Fixture, value: &str) -> TestResult {
    let id = chain_count_node(f)?;
    f.frame(vec![egui::Event::AccessKitActionRequest(
        egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::SetValue,
            target: id,
            data: Some(egui::accesskit::ActionData::Value(value.to_owned().into())),
        },
    )]);
    chain_wait(f, |f, _| chain_count(f).is_ok_and(|actual| actual == value))?;
    Ok(())
}
fn chain_add_enabled(f: &Fixture) -> TestResult<bool> {
    let (id, _) = f.node("Add", &[egui::accesskit::Role::Button])?;
    Ok(!f.nodes.get(&id).ok_or("Add node")?.is_disabled())
}
#[test]
fn numeric_add_is_bounded_cancel_is_noop_and_only_a_current_successful_add_repeats_its_count()
-> TestResult {
    for change in ["none", "unrelated_edit", "new", "view_aba"] {
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        let source = chain_seed(&mut f, "acyclic")?;
        let raw = fs::read(&source)?;
        let before = f.state()?;
        chain_click(&mut f, [144.0, 144.0], egui::Modifiers::NONE)?;
        chain_form(&mut f)?;
        assert_eq!(chain_count(&f)?, "6");
        chain_authored_same(&f, &before)?;
        if change == "none" {
            for bad in ["1", "65", "2.0", "-2", "１２"] {
                chain_set_count(&mut f, bad)?;
                assert!(!chain_add_enabled(&f)?, "strict count refuses {bad}");
                chain_authored_same(&f, &before)?;
            }
            chain_set_count(&mut f, "6")?;
            let field = chain_count_node(&f)?;
            let (add, _) = f.node("Add", &[egui::accesskit::Role::Button])?;
            f.frame(vec![
                egui::Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
                    action: egui::accesskit::Action::SetValue,
                    target: field,
                    data: Some(egui::accesskit::ActionData::Value("9".repeat(385).into())),
                }),
                egui::Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
                    action: egui::accesskit::Action::Default,
                    target: add,
                    data: None,
                }),
            ]);
            chain_form(&mut f)?;
            assert_eq!(
                chain_count(&f)?,
                "6",
                "oversize input rejects the whole edit"
            );
            chain_authored_same(&f, &before)?;
            f.choose("Cancel")?;
            chain_closed(&mut f)?;
            chain_authored_same(&f, &before)?;
            chain_click(&mut f, [144.0, 144.0], egui::Modifiers::NONE)?;
            chain_form(&mut f)?;
            assert_eq!(chain_count(&f)?, "6");
        }
        chain_set_count(&mut f, "4")?;
        assert!(chain_add_enabled(&f)?);
        f.choose("Add")?;
        chain_closed(&mut f)?;
        chain_one_edit(&f, &before, &source, &raw)?;
        let q = 24.0 * 2.0_f64.sqrt();
        let first = chain_literal(
            1,
            1,
            1,
            &[
                [144.0, 144.0],
                [144.0 + q, 144.0 - q],
                [144.0 + 2.0 * q, 144.0],
                [144.0 + 3.0 * q, 144.0 - q],
            ],
        );
        chain_expect_document(&f, &before, std::slice::from_ref(&first))?;
        if change == "none" {
            let previous = f.state()?;
            chain_click(&mut f, [144.0, 312.0], egui::Modifiers::ALT)?;
            chain_closed(&mut f)?;
            chain_one_edit(&f, &previous, &source, &raw)?;
            let second = chain_literal(
                2,
                5,
                4,
                &[
                    [144.0, 312.0],
                    [144.0 + q, 312.0 - q],
                    [144.0 + 2.0 * q, 312.0],
                    [144.0 + 3.0 * q, 312.0 - q],
                ],
            );
            chain_expect_document(&f, &before, &[first.clone(), second.clone()])?;
            chain_save_reopen(
                &mut f,
                "chain-add-repeat-λ.cdxml",
                &[first, second],
                &["C4H10", "C4H10"],
            )?;
        } else {
            match change {
                "unrelated_edit" => {
                    f.command(
                        "bond.create",
                        json!({"page_id":1,
                    "begin_x_pt":350,"begin_y_pt":480,"end_x_pt":430,"end_y_pt":480}),
                    )?;
                }
                "new" => {
                    f.command(
                        "document.new",
                        json!({"expected_revision":f.state()?["revision"],"discard_unsaved":true}),
                    )?;
                    // Existing New resets Fit; explicit Actual only qualifies the fresh recovery surface.
                    f.command("ui.view", json!({"action":"actual"}))?;
                }
                "view_aba" => {
                    f.command("ui.view", json!({"action":"set_zoom","percent":125}))?;
                    f.command("ui.view", json!({"action":"actual"}))?;
                }
                _ => return Err("declared repeat disarm".into()),
            }
            f.warm();
            let current = f.state()?;
            chain_click(&mut f, [144.0, 312.0], egui::Modifiers::ALT)?;
            chain_form(&mut f)?;
            assert_eq!(
                chain_count(&f)?,
                "6",
                "{change} must open a fresh Add, not reuse count4"
            );
            chain_authored_same(&f, &current)?;
            f.choose("Cancel")?;
            chain_closed(&mut f)?;
            chain_authored_same(&f, &current)?;
            assert_eq!(fs::read(&source)?, raw);
        }
    }
    Ok(())
}

// Independent post-controller regression declaration; no execution or private workflow API.
#[test]
fn a_real_zero_viewport_frame_disarms_add_repeat_before_an_identical_surface_returns() -> TestResult
{
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let source = chain_seed(&mut f, "acyclic")?;
    let raw = fs::read(&source)?;
    let original = f.state()?;
    chain_click(&mut f, [144.0, 144.0], egui::Modifiers::NONE)?;
    chain_form(&mut f)?;
    chain_set_count(&mut f, "4")?;
    f.choose("Add")?;
    chain_closed(&mut f)?;
    chain_one_edit(&f, &original, &source, &raw)?;
    let q = 24.0 * 2.0_f64.sqrt();
    chain_expect_document(
        &f,
        &original,
        &[chain_literal(
            1,
            1,
            1,
            &[
                [144.0, 144.0],
                [144.0 + q, 144.0 - q],
                [144.0 + 2.0 * q, 144.0],
                [144.0 + 3.0 * q, 144.0 - q],
            ],
        )],
    )?;
    let area = canvas(&f.warm())?;
    let before = f.state()?;
    let view = f.view()?;
    let width = f.width;
    f.width = 0.0;
    let unavailable = f.frame(vec![]);
    assert_eq!(
        f.context.screen_rect().width(),
        0.0,
        "setup must deliver an actual zero viewport, not a positive clamped canvas"
    );
    assert!(
        canvas(&unavailable).is_err(),
        "there is no usable drawing surface on this frame"
    );
    chain_authored_same(&f, &before)?;
    assert_eq!(f.view()?, view);
    f.width = width;
    let restored = f.frame(vec![]);
    assert_eq!(
        canvas(&restored)?,
        area,
        "restore the identical logical drawing surface"
    );
    assert_eq!(f.view()?, view);
    chain_authored_same(&f, &before)?;
    chain_click(&mut f, [144.0, 312.0], egui::Modifiers::ALT)?;
    // Fail at the actual unauthorized repeat edit, before any readiness timeout.
    chain_authored_same(&f, &before)?;
    chain_form(&mut f)?;
    assert_eq!(
        chain_count(&f)?,
        "6",
        "physical loss requires a fresh Add; old count4 is not permission"
    );
    f.choose("Cancel")?;
    chain_closed(&mut f)?;
    chain_authored_same(&f, &before)?;
    assert_eq!(fs::read(&source)?, raw);
    Ok(())
}

// Independent one-packet refusal regression; no warm frame may manufacture its feedback.
#[test]
fn an_unsupported_chain_chord_stays_visibly_refused_after_later_pointer_events_in_the_same_packet()
-> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let source = chain_seed(&mut f, "acyclic")?;
    let raw = fs::read(&source)?;
    let area = canvas(&f.warm())?;
    let a = chain_at(&f, area, [144.0, 144.0])?;
    let q = chain_at(&f, area, [216.0, 216.0])?;
    let before = f.state()?;
    assert!(
        f.app.error.is_none(),
        "the refusal must originate in this packet"
    );
    let chord = egui::Modifiers {
        ctrl: true,
        alt: true,
        ..egui::Modifiers::NONE
    };
    let shown = f.frame(vec![
        chain_pointer(a, true, egui::Modifiers::NONE),
        egui::Event::ModifiersChanged(chord),
        egui::Event::PointerMoved(q),
        chain_pointer(q, false, chord),
    ]);
    chain_authored_same(&f, &before)?;
    assert!(
        chain_count_node(&f).is_err(),
        "an unsupported drag cannot fall back to Add"
    );
    assert!(f.state()?["ui_add_chain"].is_null());
    let message = f
        .app
        .error
        .as_deref()
        .filter(|message| !message.trim().is_empty())
        .ok_or("the same-packet unsupported chord refusal was erased")?;
    assert!(
        shown.shapes.iter().any(|paint| matches!(&paint.shape,
        egui::Shape::Text(text) if text.galley.text().contains(message)
            && paint.clip_rect.intersects(paint.shape.visual_bounding_rect()))),
        "the same returned frame must visibly paint its nonempty refusal"
    );
    assert_eq!(fs::read(&source)?, raw);
    Ok(())
}

// Independent post-first-controller families. Expectations never call chain/pose math.
fn chain2_ctrl() -> egui::Modifiers {
    egui::Modifiers {
        ctrl: true,
        ..egui::Modifiers::NONE
    }
}
fn chain2_chord() -> egui::Modifiers {
    egui::Modifiers {
        ctrl: true,
        alt: true,
        ..egui::Modifiers::NONE
    }
}
fn chain2_refusal(f: &Fixture, shown: &egui::FullOutput) -> TestResult {
    let message = f
        .app
        .error
        .as_deref()
        .filter(|m| !m.trim().is_empty())
        .ok_or("visible chain refusal")?;
    assert!(shown.shapes.iter().any(|paint| matches!(&paint.shape,
        egui::Shape::Text(t) if t.galley.text().contains(message)
            && paint.clip_rect.intersects(paint.shape.visual_bounding_rect()))));
    Ok(())
}
#[test]
fn control_conversion_obeys_mouse_release_order_and_identical_stream_frame_partition() -> TestResult
{
    let prior = [
        [144.0, 144.0],
        [192.0, 144.0],
        [192.0, 192.0],
        [240.0, 192.0],
    ];
    let full = [
        [144.0, 144.0],
        [192.0, 144.0],
        [192.0, 192.0],
        [240.0, 192.0],
        [240.0, 240.0],
    ];
    let mut committed: Option<Value> = None;
    for split in [false, true] {
        for mouse_first in [true, false] {
            let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
            let source = chain_seed(&mut f, "acyclic")?;
            let raw = fs::read(&source)?;
            let area = canvas(&f.warm())?;
            let a = chain_at(&f, area, prior[0])?;
            let q = chain_at(&f, area, [216.0, 216.0])?;
            let turn = chain_at(&f, area, full[4])?;
            let before = f.state()?;
            let prefix = vec![
                egui::Event::PointerMoved(a),
                chain_pointer(a, true, egui::Modifiers::NONE),
                egui::Event::PointerMoved(q),
            ];
            let end = if mouse_first {
                vec![
                    chain_pointer(turn, false, chain2_ctrl()),
                    egui::Event::ModifiersChanged(egui::Modifiers::NONE),
                ]
            } else {
                vec![
                    egui::Event::ModifiersChanged(egui::Modifiers::NONE),
                    chain_pointer(turn, false, egui::Modifiers::NONE),
                ]
            };
            if split {
                f.frame(prefix);
                let report = f.frame(vec![egui::Event::ModifiersChanged(chain2_ctrl())]);
                chain_authored_same(&f, &before)?;
                chain_preview_axes(&f, &report, area, &prior, 4)?;
                let turning = f.frame(vec![egui::Event::PointerMoved(turn)]);
                chain_authored_same(&f, &before)?;
                chain_preview_axes(&f, &turning, area, &full, 5)?;
                f.frame(end);
            } else {
                let mut events = prefix;
                events.push(egui::Event::ModifiersChanged(chain2_ctrl()));
                events.push(egui::Event::PointerMoved(turn));
                events.extend(end);
                f.frame(events);
            }
            assert_eq!(
                f.context.input(|input| input.modifiers),
                egui::Modifiers::NONE,
                "both causal streams have the same final raw modifier snapshot"
            );
            assert!(
                chain_count_node(&f).is_err(),
                "conversion/cancellation cannot open Add"
            );
            if mouse_first {
                chain_one_edit(&f, &before, &source, &raw)?;
                chain_expect_document(&f, &before, &[chain_literal(1, 1, 1, &full)])?;
                let document = f.state()?["document"].clone();
                if let Some(previous) = &committed {
                    assert_eq!(
                        &document, previous,
                        "partitioning the same event stream must preserve every represented point"
                    );
                } else {
                    committed = Some(document);
                }
            } else {
                chain_authored_same(&f, &before)?;
                assert_eq!(fs::read(&source)?, raw);
            }
        }
    }
    Ok(())
}

#[test]
fn off_length_reversal_uses_one_chord_rise_and_late_alt_cannot_reinterpret_a_converted_walk()
-> TestResult {
    let mirrored = [
        [144.0, 144.0],
        [184.0, 164.0],
        [204.0, 124.0],
        [244.0, 144.0],
    ];
    let mut first: Option<Value> = None;
    for duplicates in [0, 1] {
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        let source = chain_seed(&mut f, "acyclic")?;
        f.command(
            "ui.chain_tool",
            json!({"action":"constraints","fixed_lengths":false,"fixed_angles":true}),
        )?;
        let raw = fs::read(&source)?;
        let area = canvas(&f.warm())?;
        let before = f.state()?;
        chain_press(&mut f, area, [144.0, 144.0])?;
        chain_move(&mut f, area, [244.0, 144.0])?;
        let q = chain_at(&f, area, [244.0, 144.0])?;
        let mut events = vec![
            egui::Event::ModifiersChanged(chain2_ctrl()),
            egui::Event::ModifiersChanged(chain2_chord()),
        ];
        for _ in 0..duplicates {
            events.push(egui::Event::ModifiersChanged(chain2_chord()));
        }
        events.push(egui::Event::PointerMoved(q));
        let reversed = f.frame(events);
        chain_authored_same(&f, &before)?;
        chain_preview_axes(&f, &reversed, area, &mirrored, 4)?;
        f.frame(vec![
            chain_pointer(q, false, chain2_chord()),
            egui::Event::ModifiersChanged(egui::Modifiers::NONE),
        ]);
        chain_one_edit(&f, &before, &source, &raw)?;
        chain_expect_document(&f, &before, &[chain_literal(1, 1, 1, &mirrored)])?;
        let document = f.state()?["document"].clone();
        if let Some(previous) = &first {
            assert_eq!(
                &document, previous,
                "same-state reports cannot toggle twice"
            );
        } else {
            first = Some(document);
        }
    }
    for later in [chain2_chord(), egui::Modifiers::ALT] {
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        let source = chain_seed(&mut f, "acyclic")?;
        f.command(
            "ui.chain_tool",
            json!({"action":"constraints","fixed_lengths":false,"fixed_angles":true}),
        )?;
        let raw = fs::read(&source)?;
        let area = canvas(&f.warm())?;
        let before = f.state()?;
        chain_press(&mut f, area, [144.0, 144.0])?;
        chain_move(&mut f, area, [244.0, 144.0])?;
        // A real Control-only MOVE converts before Alt arrives. The six-point count
        // consists of the unchanged four-point OFF prefix, one48pt step and a finite tip.
        let turn = chain_at(&f, area, [264.0, 224.0])?;
        let converted = f.frame(vec![
            egui::Event::ModifiersChanged(chain2_ctrl()),
            egui::Event::PointerMoved(turn),
        ]);
        assert!(chain_text(&converted, "Carbons: 6 (including anchor)"));
        assert!(
            f.app.error.is_none(),
            "the conversion setup must be renderable: {:?}",
            f.app.error
        );
        chain_authored_same(&f, &before)?;
        let refused = f.frame(vec![
            egui::Event::ModifiersChanged(later),
            egui::Event::PointerMoved(turn),
            chain_pointer(turn, false, later),
            egui::Event::ModifiersChanged(egui::Modifiers::NONE),
        ]);
        chain_authored_same(&f, &before)?;
        assert!(chain_count_node(&f).is_err());
        chain2_refusal(&f, &refused)?;
        assert_eq!(fs::read(&source)?, raw);
    }
    Ok(())
}

fn chain2_close_packet(f: &mut Fixture) -> egui::FullOutput {
    let mut input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(f.width, f.height),
        )),
        focused: true,
        ..Default::default()
    };
    let viewport = input.viewports.entry(egui::ViewportId::ROOT).or_default();
    viewport.native_pixels_per_point = Some(f.dpi);
    viewport.events.push(egui::ViewportEvent::Close);
    let context = f.context.clone();
    let output = context.run(input, |context| f.app.update_workspace(context));
    if let Some(update) = &output.platform_output.accesskit_update {
        f.nodes.extend(update.nodes.iter().cloned());
    }
    output
}
fn chain2_spare() -> Value {
    json!({"id":1,"atoms":[
        {"id":1,"element":6,"position":{"x_pt":350.0,"y_pt":480.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":3},
        {"id":2,"element":6,"position":{"x_pt":430.0,"y_pt":480.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":3}],
        "bonds":[{"id":1,"begin":1,"end":2,"order":"single"}]})
}
#[test]
fn add_drafts_latch_context_loss_and_remote_cancel_fences_the_whole_dismissal_packet() -> TestResult
{
    for change in ["view_aba", "close_aba", "canvas_aba", "source_same"] {
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        let source = chain_seed(&mut f, "acyclic")?;
        let raw = fs::read(&source)?;
        // There is real Undo/Save work behind this form, not an empty-history precondition.
        f.command(
            "bond.create",
            json!({"page_id":1,"begin_x_pt":350,"begin_y_pt":480,
            "end_x_pt":430,"end_y_pt":480}),
        )?;
        f.warm();
        let before = f.state()?;
        chain_click(&mut f, [144.0, 144.0], egui::Modifiers::NONE)?;
        chain_form(&mut f)?;
        chain_set_count(&mut f, "4")?;
        let request = f.state()?["ui_add_chain"]["id"]
            .as_u64()
            .ok_or("exact Add request ID")?;
        let closing = chain2_close_packet(&mut f);
        assert!(
            closing
                .viewport_output
                .get(&egui::ViewportId::ROOT)
                .is_some_and(|viewport| viewport
                    .commands
                    .iter()
                    .any(|command| matches!(command, egui::ViewportCommand::CancelClose)))
        );
        chain_authored_same(&f, &before)?;
        assert_eq!(chain_count(&f)?, "4");
        assert!(
            chain_add_enabled(&f)?,
            "holding native Close does not discard or stale a current draft"
        );
        match change {
            "view_aba" => {
                f.command("ui.view", json!({"action":"set_zoom","percent":125}))?;
                f.command("ui.view", json!({"action":"actual"}))?;
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
            "canvas_aba" => {
                let width = f.width;
                f.width = width + 24.0;
                f.frame(vec![]);
                f.width = width;
                f.frame(vec![]);
            }
            "source_same" => {
                f.command(
                    "document.open_cdxml",
                    json!({"path":source,"discard_unsaved":true}),
                )?;
                f.command("ui.view", json!({"action":"actual"}))?;
            }
            _ => return Err("declared Add context".into()),
        }
        let current = f.state()?;
        chain_wait(&mut f, |f, _| {
            chain_count(f).is_ok_and(|v| v == "4")
                && chain_add_enabled(f).is_ok_and(|enabled| !enabled)
        })?;
        let field = chain_count_node(&f)?;
        assert!(
            !f.nodes
                .get(&field)
                .ok_or("copyable stale field")?
                .is_disabled()
        );
        if change != "canvas_aba" {
            assert_eq!(f.state()?["ui_add_chain"]["current"], false);
        }
        let (old_add, _) = f.node("Add", &[egui::accesskit::Role::Button])?;
        f.frame(vec![egui::Event::AccessKitActionRequest(
            egui::accesskit::ActionRequest {
                action: egui::accesskit::Action::Default,
                target: old_add,
                data: None,
            },
        )]);
        chain_authored_same(&f, &current)?;
        let area = canvas(&f.warm())?;
        let a = chain_at(&f, area, [144.0, 312.0])?;
        let q = chain_at(&f, area, [216.0, 384.0])?;
        f.command("ui.chain_tool",json!({"action":"cancel_add","expected_revision":f.state()?["revision"],"request_id":request}))?;
        f.frame(vec![
            egui::Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
                action: egui::accesskit::Action::Default,
                target: old_add,
                data: None,
            }),
            egui::Event::PointerMoved(a),
            chain_pointer(a, true, egui::Modifiers::NONE),
            egui::Event::PointerMoved(q),
            chain_pointer(q, false, egui::Modifiers::NONE),
            key(egui::Key::Z, true, egui::Modifiers::COMMAND),
            key(egui::Key::Z, false, egui::Modifiers::COMMAND),
            key(egui::Key::S, true, egui::Modifiers::COMMAND),
            key(egui::Key::S, false, egui::Modifiers::COMMAND),
        ]);
        chain_authored_same(&f, &current)?;
        assert!(f.state()?["ui_add_chain"].is_null());
        assert!(
            f.state()?["ui_file"]["request"].is_null(),
            "no Save may be queued by the dismissal suffix"
        );
        chain_closed(&mut f)?;
        chain_authored_same(&f, &current)?;
        assert_eq!(fs::read(&source)?, raw);
        let area = canvas(&f.warm())?;
        let recovery = f.state()?;
        chain_press(&mut f, area, [144.0, 312.0])?;
        chain_move(&mut f, area, [216.0, 384.0])?;
        chain_release(&mut f, area, [216.0, 384.0])?;
        chain_one_edit(&f, &recovery, &source, &raw)?;
        let points = [
            [144.0, 312.0],
            [192.0, 312.0],
            [192.0, 360.0],
            [240.0, 360.0],
        ];
        let expected = if change == "source_same" {
            vec![chain_literal(1, 1, 1, &points)]
        } else {
            vec![chain2_spare(), chain_literal(2, 3, 2, &points)]
        };
        chain_expect_document(&f, &recovery, &expected)?;
    }
    Ok(())
}

#[test]
fn invisible_add_atom_and_settings_open_cancel_epochs_cannot_revive_an_older_drag() -> TestResult {
    for owner in ["add", "atom", "settings"] {
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        f.command(
            "bond.create",
            json!({"page_id":1,"begin_x_pt":350,"begin_y_pt":480,
            "end_x_pt":430,"end_y_pt":480}),
        )?;
        let source = chain_seed(&mut f, "acyclic")?;
        let raw = fs::read(&source)?;
        let area = canvas(&f.warm())?;
        let before = f.state()?;
        chain_press(&mut f, area, [144.0, 144.0])?;
        let shown = chain_move(&mut f, area, [216.0, 216.0])?;
        assert!(chain_text(&shown, "Carbons: 4 (including anchor)"));
        chain_authored_same(&f, &before)?;
        let revision = f.state()?["revision"].clone();
        // No App frame is allowed between Open and Cancel; final Option equality is insufficient.
        match owner {
            "add" => {
                let result = f.command(
                    "ui.chain_tool",
                    json!({"action":"open_add","expected_revision":revision,
                    "page_id":1,"target":{"mode":"free","x_pt":144,"y_pt":312}}),
                )?;
                let id = result["ui_add_chain"]["id"].as_u64().ok_or("Add ID")?;
                f.command(
                    "ui.chain_tool",
                    json!({"action":"cancel_add","expected_revision":revision,"request_id":id}),
                )?;
            }
            "atom" => {
                let result = f.command(
                    "ui.atom_label",
                    json!({"action":"open","expected_revision":revision,
                    "page_id":1,"fragment_id":1,"atom_id":1}),
                )?;
                let id = result["ui_atom_label"]["id"].as_u64().ok_or("Atom ID")?;
                f.command(
                    "ui.atom_label",
                    json!({"action":"cancel","expected_revision":revision,"request_id":id}),
                )?;
            }
            "settings" => {
                let result = f.command(
                    "ui.drawing_settings",
                    json!({"action":"open","expected_revision":revision}),
                )?;
                let id = result["ui_drawing_settings"]["id"]
                    .as_u64()
                    .ok_or("settings ID")?;
                f.command(
                    "ui.drawing_settings",
                    json!({"action":"cancel","expected_revision":revision,"request_id":id}),
                )?;
            }
            _ => return Err("declared foreground epoch".into()),
        }
        assert!(f.state()?["ui_add_chain"].is_null());
        assert!(f.state()?["ui_atom_label"].is_null());
        assert!(f.state()?["ui_drawing_settings"].is_null());
        chain_release(&mut f, area, [216.0, 216.0])?;
        f.warm();
        chain_authored_same(&f, &before)?;
        assert!(
            chain_count_node(&f).is_err(),
            "a cancelled old drag cannot become an Add click"
        );
        assert!(
            f.node("Element", &[egui::accesskit::Role::TextInput])
                .is_err()
        );
        assert!(
            f.node("Bond length (pt)", &[egui::accesskit::Role::TextInput])
                .is_err()
        );
        assert_eq!(fs::read(&source)?, raw);
        chain_press(&mut f, area, [144.0, 144.0])?;
        chain_move(&mut f, area, [216.0, 216.0])?;
        chain_release(&mut f, area, [216.0, 216.0])?;
        chain_one_edit(&f, &before, &source, &raw)?;
        chain_expect_document(
            &f,
            &before,
            &[
                chain2_spare(),
                chain_literal(
                    2,
                    3,
                    2,
                    &[
                        [144.0, 144.0],
                        [192.0, 144.0],
                        [192.0, 192.0],
                        [240.0, 192.0],
                    ],
                ),
            ],
        )?;
    }
    Ok(())
}

fn chain2_native_source(f: &mut Fixture, fragments: &[Value]) -> TestResult<PathBuf> {
    let mut document = bondline_model::Document::default();
    document.set_drawing_settings(bondline_model::DrawingSettings::new(
        Some(48.0),
        Some(90.0),
    )?);
    let page = bondline_model::PageId::new(1).ok_or("page")?;
    let fragments = fragments
        .iter()
        .map(|fragment| {
            Ok(bondline_model::Fragment::new(
                bondline_model::FragmentId::new(u32::try_from(
                    fragment["id"].as_u64().ok_or("fragment ID")?,
                )?)
                .ok_or("fragment")?,
                serde_json::from_value::<Vec<bondline_model::Atom>>(fragment["atoms"].clone())?,
                serde_json::from_value::<Vec<bondline_model::Bond>>(fragment["bonds"].clone())?,
            )?)
        })
        .collect::<TestResult<Vec<_>>>()?;
    document.append_fragments(page, fragments)?;
    let mut stored = bondline_commands::Engine::default()
        .stored_document()?
        .clone();
    *stored.model_mut() = document;
    {
        let mut engine = f.app.engine.lock().map_err(|_| "engine")?;
        *engine = bondline_commands::Engine::from_document(stored);
        engine.set_ui_file_host(true);
    }
    let source = f.directory.path("weighted-chain-source-λ.cdxml");
    f.command("document.save_cdxml", json!({"path":source}))?;
    f.command("document.open_cdxml", json!({"path":source}))?;
    f.command("ui.view", json!({"action":"actual"}))?;
    f.warm();
    chain_choose(f, "Acyclic chain")?;
    f.command(
        "ui.chain_tool",
        json!({"action":"constraints","fixed_lengths":true,"fixed_angles":true}),
    )?;
    f.warm();
    Ok(source)
}
fn chain2_consume(
    path: &Path,
    fragments: &[Value],
    smiles: &[&str],
    formulas: &[&str],
) -> TestResult {
    let expected = json!({"fragments":fragments,"smiles":smiles,"formulas":formulas});
    let mut child=Process::new(std::env::var_os("BONDLINE_PYTHON").ok_or("pinned external Python")?)
        .args(["-B","-c",r#"
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
assert actual==plain
nodes=[];bonds=[]
def start(name,attrs):
 if name=='n':nodes.append(attrs)
 if name=='b':bonds.append(attrs)
p=expat.ParserCreate();p.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
p.StartElementHandler=start;p.Parse(source,True)
atom_wants=[a for f in want for a in f['atoms']];bond_wants=[b for f in want for b in f['bonds']]
assert len(nodes)==len(atom_wants) and len(bonds)==len(bond_wants)
for n,a in zip(nodes,atom_wants):
 for model,xml in [('show_terminal_carbon_labels','ShowTerminalCarbonLabels'),('show_non_terminal_carbon_labels','ShowNonTerminalCarbonLabels')]:
  assert n.get(xml)==({True:'yes',False:'no'}[a[model]] if model in a else None)
 for value,axis in zip(map(Decimal,n['p'].split()),('x_pt','y_pt')):
  assert value.is_finite() and abs(value-Decimal(str(a['position'][axis])))<=Decimal('1e-9')
for b,w in zip(bonds,bond_wants):
 assert b.get('DoublePosition')==(w['double_position'].capitalize() if 'double_position' in w else None)
 assert 'Display' not in b
ms=Chem.MolsFromCDXML(source.decode('utf-8'))
assert len(ms)==len(want)==len(e['smiles'])==len(e['formulas'])
for m,g,s,formula in zip(ms,want,e['smiles'],e['formulas']):
 assert m.GetNumAtoms()==len(g['atoms']) and m.GetNumBonds()==len(g['bonds'])
 assert [a.GetAtomicNum() for a in m.GetAtoms()]==[a['element'] for a in g['atoms']]
 assert [a.GetTotalNumHs() for a in m.GetAtoms()]==[a['explicit_hydrogens'] for a in g['atoms']]
 assert all(a.GetFormalCharge()==0 and a.GetIsotope()==0 and a.GetNumRadicalElectrons()==0 for a in m.GetAtoms())
 assert [b.GetBondTypeAsDouble() for b in m.GetBonds()]==[{'single':1,'double':2,'triple':3}[b['order']] for b in g['bonds']]
 reference=Chem.MolFromSmiles(s);assert reference is not None
 assert Chem.MolToSmiles(m)==Chem.MolToSmiles(reference)
 assert rdMolDescriptors.CalcMolFormula(m)==formula
"#])
        .arg(path).arg(concat!(env!("CARGO_MANIFEST_DIR"),"/../../xtask/oracles/cdxml_graph.py"))
        .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
    serde_json::to_writer(child.stdin.take().ok_or("consumer stdin")?, &expected)?;
    let result = child.wait_with_output()?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(())
}
#[test]
fn numeric_triple_attachment_is_linear_and_no_h_cumulene_cannot_fall_back_to_free() -> TestResult {
    let styled = json!({"id":1,"atoms":[
        {"id":1,"element":6,"position":{"x_pt":350.0,"y_pt":480.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":2,
            "show_terminal_carbon_labels":false,"show_non_terminal_carbon_labels":true},
        {"id":2,"element":6,"position":{"x_pt":430.0,"y_pt":480.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":2,
            "show_terminal_carbon_labels":true,"show_non_terminal_carbon_labels":false}],
        "bonds":[{"id":1,"begin":1,"end":2,"order":"double","double_position":"right"}]});
    let triple = json!({"id":2,"atoms":[
        {"id":3,"element":6,"position":{"x_pt":144.0,"y_pt":288.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":1,
            "show_terminal_carbon_labels":true},
        {"id":4,"element":6,"position":{"x_pt":192.0,"y_pt":288.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":1,
            "show_non_terminal_carbon_labels":false}],
        "bonds":[{"id":2,"begin":3,"end":4,"order":"triple"}]});
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let source = chain2_native_source(&mut f, &[styled.clone(), triple.clone()])?;
    chain2_consume(
        &source,
        &[styled.clone(), triple.clone()],
        &["C=C", "C#C"],
        &["C2H4", "C2H2"],
    )?;
    let raw = fs::read(&source)?;
    let before = f.state()?;
    let kept = std::sync::Arc::clone(
        &f.app
            .engine
            .lock()
            .map_err(|_| "engine")?
            .document()?
            .pages()[0]
            .fragments()[0],
    );
    chain_click(&mut f, [192.0, 288.0], egui::Modifiers::NONE)?;
    chain_form(&mut f)?;
    assert_eq!(
        f.state()?["ui_add_chain"]["target"],
        json!({"mode":"atom","fragment_id":2,"atom_id":4})
    );
    chain_set_count(&mut f, "2")?;
    f.choose("Add")?;
    chain_closed(&mut f)?;
    let edited = chain_one_edit(&f, &before, &source, &raw)?;
    let mut expected = triple;
    expected["atoms"][1]["explicit_hydrogens"] = json!(0);
    expected["atoms"].as_array_mut().ok_or("atoms")?.push(json!({
        "id":5,"element":6,"position":{"x_pt":240.0,"y_pt":288.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":3,
        "show_terminal_carbon_labels":false,"show_non_terminal_carbon_labels":false}));
    expected["bonds"]
        .as_array_mut()
        .ok_or("bonds")?
        .push(json!({"id":3,"begin":4,"end":5,"order":"single"}));
    chain_expect_document(&f, &before, &[styled.clone(), expected.clone()])?;
    assert!(std::sync::Arc::ptr_eq(
        &kept,
        &f.app
            .engine
            .lock()
            .map_err(|_| "engine")?
            .document()?
            .pages()[0]
            .fragments()[0]
    ));
    chain_history(&mut f, &before, &edited)?;
    let output = f.directory.path("linear-propyne-λ.cdxml");
    f.command("document.save_cdxml", json!({"path":output}))?;
    chain2_consume(
        &output,
        &[styled.clone(), expected.clone()],
        &["C=C", "C#CC"],
        &["C2H4", "C3H4"],
    )?;
    f.command("document.open_cdxml", json!({"path":output}))?;
    chain_expect_document(&f, &before, &[styled, expected])?;
    assert_eq!(f.state()?["file"]["dirty"], false);
    assert_eq!(f.state()?["undo_depth"], 0);

    let mut cumulene = chain_literal(1, 1, 1, &[[144.0, 288.0], [192.0, 288.0], [240.0, 288.0]]);
    for (i, h) in [2, 0, 2].into_iter().enumerate() {
        cumulene["atoms"][i]["explicit_hydrogens"] = json!(h);
    }
    for i in 0..2 {
        cumulene["bonds"][i]["order"] = json!("double");
        cumulene["bonds"][i]["double_position"] = json!("center");
    }
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let source = chain2_native_source(&mut f, std::slice::from_ref(&cumulene))?;
    chain2_consume(
        &source,
        std::slice::from_ref(&cumulene),
        &["C=C=C"],
        &["C3H4"],
    )?;
    let raw = fs::read(&source)?;
    let before = f.state()?;
    let area = canvas(&f.warm())?;
    let center = chain_at(&f, area, [192.0, 288.0])?;
    let picked = {
        let engine = f.app.engine.lock().map_err(|_| "engine")?;
        let page = bondline_model::PageId::new(1).ok_or("page")?;
        let view = engine.page_view(
            area.width().floor() as u32,
            area.height().floor() as u32,
            page,
        )?;
        let graph = engine.drawing_scene_with_view(
            &view,
            page,
            &bondline_render::Cancellation::default(),
        )?;
        let local = center - area.min;
        graph.hit_test([local.x, local.y], 5.0)?
    };
    assert!(
        matches!(picked,Some(bondline_render::HitTarget::Atom {atom_id,..}) if atom_id.get()==2),
        "setup must actually pick the mandatory central C/H0, not a free fallback"
    );
    chain_click(&mut f, [192.0, 288.0], egui::Modifiers::NONE)?;
    f.warm();
    chain_authored_same(&f, &before)?;
    assert!(chain_count_node(&f).is_err());
    assert!(f.state()?["ui_add_chain"].is_null());
    assert!(f.app.error.as_ref().is_some_and(|m| !m.trim().is_empty()));
    assert_eq!(fs::read(&source)?, raw);
    Ok(())
}

// Synthetic2x/fractional-origin App evidence, not measured native-display acceptance.
// App input is f32. First cast the chosen screen points, then independently invert
// the documented logical camera to obtain the expected authored anchor. No ideal
// decimal pointer or production chain/pose helper is used as an oracle.
fn chain3_frame(f: &mut Fixture, origin: egui::Pos2, events: Vec<egui::Event>) -> egui::FullOutput {
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
    let output = context.run(input, |context| f.app.update_workspace(context));
    if let Some(update) = &output.platform_output.accesskit_update {
        f.nodes.extend(update.nodes.iter().cloned());
    }
    output
}
fn chain3_affine(f: &Fixture, area: egui::Rect) -> TestResult<([f64; 2], f64)> {
    let view = f.view()?;
    let scale = view["zoom_percent"]
        .as_f64()
        .ok_or("explicit physical-test camera")?
        / 100.0;
    let center = [
        view["center_x_pt"].as_f64().ok_or("explicit camera x")?,
        view["center_y_pt"].as_f64().ok_or("explicit camera y")?,
    ];
    let offset = [
        (f64::from(area.width().floor()) / 2.0 - center[0] * scale).round(),
        (f64::from(area.height().floor()) / 2.0 - center[1] * scale).round(),
    ];
    Ok((
        [
            f64::from(area.left()) + offset[0],
            f64::from(area.top()) + offset[1],
        ],
        scale,
    ))
}
fn chain3_screen(base: [f64; 2], scale: f64, point: [f64; 2]) -> egui::Pos2 {
    egui::pos2(
        (base[0] + point[0] * scale) as f32,
        (base[1] + point[1] * scale) as f32,
    )
}
fn chain3_inverse(base: [f64; 2], scale: f64, screen: egui::Pos2) -> [f64; 2] {
    [
        (f64::from(screen.x) - base[0]) / scale,
        (f64::from(screen.y) - base[1]) / scale,
    ]
}
fn chain3_inside(quad: &[egui::Pos2], point: egui::Pos2) -> bool {
    let mut positive = false;
    let mut negative = false;
    for i in 0..quad.len() {
        let a = quad[i];
        let b = quad[(i + 1) % quad.len()];
        let cross = (b.x - a.x) * (point.y - a.y) - (b.y - a.y) * (point.x - a.x);
        positive |= cross > 0.0;
        negative |= cross < 0.0;
    }
    !(positive && negative)
}
fn chain3_physical_ink(
    f: &Fixture,
    shown: &egui::FullOutput,
    area: egui::Rect,
    base: [f64; 2],
    scale: f64,
    points: &[[f64; 2]],
) -> TestResult {
    let density = f64::from(f.dpi);
    let quads = shown
        .shapes
        .iter()
        .filter_map(|paint| match &paint.shape {
            egui::Shape::Path(path)
                if path.closed
                    && path.fill == egui::Color32::BLACK
                    && path.points.len() == 4
                    && path.points.iter().all(|p| area.contains(*p)) =>
            {
                Some(path.points.clone())
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        quads.len(),
        3,
        "literal four-carbon chain has exactly three Single components"
    );
    let mut remaining = quads.clone();
    let order = |a: &[f64; 2], b: &[f64; 2]| a[0].total_cmp(&b[0]).then(a[1].total_cmp(&b[1]));
    for pair in points.windows(2) {
        let a = [
            (base[0] + pair[0][0] * scale) * density,
            (base[1] + pair[0][1] * scale) * density,
        ];
        let b = [
            (base[0] + pair[1][0] * scale) * density,
            (base[1] + pair[1][1] * scale) * density,
        ];
        let delta = [b[0] - a[0], b[1] - a[1]];
        let length = delta[0].hypot(delta[1]);
        // Existing authored stroke width is1pt, unchanged by the chain preferences.
        let half = scale * density / 2.0;
        let normal = [-delta[1] / length, delta[0] / length];
        let mut expected = vec![
            [a[0] - normal[0] * half, a[1] - normal[1] * half],
            [b[0] - normal[0] * half, b[1] - normal[1] * half],
            [b[0] + normal[0] * half, b[1] + normal[1] * half],
            [a[0] + normal[0] * half, a[1] + normal[1] * half],
        ];
        expected.sort_by(order);
        let matched=remaining.iter().position(|quad| {
            let mut actual=quad.iter().map(|p|[f64::from(p.x)*density,f64::from(p.y)*density]).collect::<Vec<_>>();
            actual.sort_by(order);
            actual.iter().zip(&expected).all(|(a,b)|(a[0]-b[0]).abs()<=1.0/256.0&&(a[1]-b[1]).abs()<=1.0/256.0)
        }).ok_or("actual global device quad differs from the independently transformed literal stroke")?;
        remaining.swap_remove(matched);
        let middle = [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0];
        let ink = middle.map(|v| v.floor() + 0.5);
        let white = [
            (middle[0] + normal[0] * 3.0).floor() + 0.5,
            (middle[1] + normal[1] * 3.0).floor() + 0.5,
        ];
        let ink_logical = egui::pos2((ink[0] / density) as f32, (ink[1] / density) as f32);
        let white_logical = egui::pos2((white[0] / density) as f32, (white[1] / density) as f32);
        assert!(
            quads.iter().any(|q| chain3_inside(q, ink_logical)),
            "predicted global device center must carry bond ink"
        );
        assert!(
            quads.iter().all(|q| !chain3_inside(q, white_logical)),
            "the independent exterior pixel must stay clear"
        );
        assert!(shown.shapes.iter().any(|paint|matches!(&paint.shape,egui::Shape::Rect(r)
            if r.fill==egui::Color32::WHITE && r.rect.contains(white_logical) && paint.clip_rect.contains(white_logical))),
            "clear corridor must have actual white paper below it");
    }
    let glyphs = shown
        .shapes
        .iter()
        .filter_map(|paint| match &paint.shape {
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
        "the original O/H2 must remain in the WHOLE preview"
    );
    for glyph in glyphs {
        for coordinate in [glyph.left(), glyph.top(), glyph.right(), glyph.bottom()] {
            let device = f64::from(coordinate) * density;
            assert!(
                (device - device.round()).abs() <= 1.0 / 256.0,
                "global device glyph phase"
            );
        }
    }
    Ok(())
}

#[test]
fn actual_chain_two_x_and_fractional_origin_preserve_device_ink_pick_history_files_and_guards()
-> TestResult {
    let water = json!({"id":1,"atoms":[{"id":1,"element":8,"position":{"x_pt":288.0,"y_pt":432.0},
        "isotope":null,"charge":0,"radical":"none","explicit_hydrogens":2}],"bonds":[]});
    for (index, origin) in [egui::Pos2::ZERO, egui::pos2(0.125, 0.375)]
        .into_iter()
        .enumerate()
    {
        let mut f = Fixture::new(1160.0, 800.0, 2.0)?;
        let source = chain2_native_source(&mut f, std::slice::from_ref(&water))?;
        chain2_consume(&source, std::slice::from_ref(&water), &["O"], &["H2O"])?;
        let raw = fs::read(&source)?;
        f.command(
            "ui.view",
            json!({"action":"pan","center_x_pt":288,"center_y_pt":432}),
        )?;
        chain3_frame(&mut f, origin, vec![]);
        let warm = chain3_frame(&mut f, origin, vec![]);
        assert_eq!(warm.pixels_per_point, 2.0);
        let area = canvas(&warm)?;
        let device_origin = [f64::from(area.left()) * 2.0, f64::from(area.top()) * 2.0];
        let phase = device_origin.map(|v| v - v.floor());
        if index == 1 {
            assert!(
                phase.iter().any(|v| (0.2..=0.8).contains(v)),
                "the declared fractional raw origin must actually produce substantial device phase"
            );
        }
        let (base, scale) = chain3_affine(&f, area)?;
        assert_eq!(scale, 1.0);
        let press = chain3_screen(base, scale, [144.0, 144.0]);
        let first_move = chain3_screen(base, scale, [168.0, 168.0]);
        let release = chain3_screen(base, scale, [216.0, 216.0]);
        let anchor = chain3_inverse(base, scale, press);
        let end = chain3_inverse(base, scale, release);
        assert!(
            ((end[0] - anchor[0]) - 72.0).abs() <= 1e-5
                && ((end[1] - anchor[1]) - 72.0).abs() <= 1e-5,
            "actual f32 inputs must qualify the interior45-degree/count4 witness"
        );
        let points = [
            [anchor[0], anchor[1]],
            [anchor[0] + 48.0, anchor[1]],
            [anchor[0] + 48.0, anchor[1] + 48.0],
            [anchor[0] + 96.0, anchor[1] + 48.0],
        ];
        let before = f.state()?;
        let view = f.view()?;
        chain3_frame(
            &mut f,
            origin,
            vec![
                egui::Event::PointerMoved(press),
                chain_pointer(press, true, egui::Modifiers::NONE),
            ],
        );
        chain3_frame(&mut f, origin, vec![egui::Event::PointerMoved(first_move)]);
        let preview = chain3_frame(&mut f, origin, vec![egui::Event::PointerMoved(release)]);
        assert_eq!(canvas(&preview)?, area);
        assert!(chain_text(&preview, "Carbons: 4 (including anchor)"));
        assert_eq!(f.state()?, before, "physical preview is disposable");
        assert_eq!(f.view()?, view);
        chain3_physical_ink(&f, &preview, area, base, scale, &points)?;
        chain3_frame(
            &mut f,
            origin,
            vec![chain_pointer(release, false, egui::Modifiers::NONE)],
        );
        let edited = chain_one_edit(&f, &before, &source, &raw)?;
        let chain = chain_literal(2, 2, 1, &points);
        chain_expect_document(&f, &before, &[water.clone(), chain.clone()])?;
        // Fresh logical hover must hit the same physical component; no click or edit.
        f.command("ui.bond_tool", json!({"order":"single"}))?;
        let pick = chain3_screen(base, scale, [anchor[0] + 24.0, anchor[1]]);
        chain3_frame(&mut f, origin, vec![egui::Event::PointerMoved(pick)]);
        assert!(
            matches!(f.app.drawing.hover_target(),Some(bondline_render::HitTarget::Bond {fragment_id,bond_id})
            if fragment_id.get()==2&&bond_id.get()==1)
        );
        chain_authored_same(&f, &edited)?;
        chain3_frame(
            &mut f,
            origin,
            vec![
                key(egui::Key::Z, true, egui::Modifiers::COMMAND),
                key(egui::Key::Z, false, egui::Modifiers::COMMAND),
            ],
        );
        assert_eq!(f.state()?["document"], before["document"]);
        assert_eq!(f.state()?["source"], before["source"]);
        assert_eq!(f.state()?["file"], before["file"]);
        assert_eq!(f.state()?["redo_depth"], 1);
        let shift = egui::Modifiers {
            shift: true,
            ..egui::Modifiers::COMMAND
        };
        chain3_frame(
            &mut f,
            origin,
            vec![
                key(egui::Key::Z, true, shift),
                key(egui::Key::Z, false, shift),
            ],
        );
        assert_eq!(f.state()?["document"], edited["document"]);
        assert_eq!(f.state()?["undo_depth"], 1);
        assert_eq!(f.state()?["redo_depth"], 0);
        if index == 0 {
            let capture_before = f.state()?;
            let capture_view = f.view()?;
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
                "complete CPU/GPU PNG bytes must agree"
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
            assert_eq!(f.state()?, capture_before);
            assert_eq!(f.view()?, capture_view);
        }
        let output = f.directory.path(if index == 0 {
            "physical-2x-chain-λ.cdxml"
        } else {
            "physical-fraction-chain-λ.cdxml"
        });
        let document = f.state()?["document"].clone();
        f.command("document.save_cdxml", json!({"path":output}))?;
        assert_eq!(f.state()?["document"], document);
        assert_eq!(f.state()?["file"]["dirty"], false);
        chain2_consume(
            &output,
            &[water.clone(), chain.clone()],
            &["O", "CCCC"],
            &["H2O", "C4H10"],
        )?;
        assert_eq!(fs::read(&source)?, raw);
        f.command("document.open_cdxml", json!({"path":output}))?;
        chain_expect_document(&f, &before, &[water.clone(), chain])?;
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
        // Existing Open resets Fit/center. Qualify a fresh valid source surface before
        // the negative guard so namespace discard cannot be the reason input is refused.
        f.command("ui.view", json!({"action":"actual"}))?;
        f.command(
            "ui.view",
            json!({"action":"pan","center_x_pt":288,"center_y_pt":432}),
        )?;
        let reopened = chain3_frame(&mut f, origin, vec![]);
        let reopened_area = canvas(&reopened)?;
        let (reopened_base, reopened_scale) = chain3_affine(&f, reopened_area)?;
        chain3_physical_ink(
            &f,
            &reopened,
            reopened_area,
            reopened_base,
            reopened_scale,
            &points,
        )?;

        // Physical font pixels alone must not make a logically tiny existing label editable.
        f.command("ui.chain_tool", json!({"action":"select","kind":"acyclic"}))?;
        f.command("ui.view", json!({"action":"set_zoom","percent":50}))?;
        let guarded = f.state()?;
        let tiny = chain3_frame(&mut f, origin, vec![]);
        let guard = bondline_render::Error::LabelScale.to_string();
        assert!(
            chain_text(&tiny, &guard),
            "logical8-em minimum is independent of physical2x"
        );
        let small_area = canvas(&tiny)?;
        let (small_base, small_scale) = chain3_affine(&f, small_area)?;
        let a = chain3_screen(small_base, small_scale, [144.0, 144.0]);
        let b = chain3_screen(small_base, small_scale, [216.0, 216.0]);
        chain3_frame(
            &mut f,
            origin,
            vec![
                chain_pointer(a, true, egui::Modifiers::NONE),
                egui::Event::PointerMoved(b),
                chain_pointer(b, false, egui::Modifiers::NONE),
            ],
        );
        chain_authored_same(&f, &guarded)?;
        assert!(chain_count_node(&f).is_err());
        f.command("ui.view", json!({"action":"actual"}))?;
        f.command(
            "ui.view",
            json!({"action":"pan","center_x_pt":288,"center_y_pt":432}),
        )?;
        let restored = chain3_frame(&mut f, origin, vec![]);
        let area = canvas(&restored)?;
        let (base, scale) = chain3_affine(&f, area)?;
        // This start is inside the actual canvas but10pt outside the authored paper.
        let outside = chain3_screen(base, scale, [-10.0, 288.0]);
        let inside = chain3_screen(base, scale, [62.0, 360.0]);
        assert!(area.contains(outside) && area.contains(inside));
        let paper_before = f.state()?;
        chain3_frame(
            &mut f,
            origin,
            vec![
                chain_pointer(outside, true, egui::Modifiers::NONE),
                egui::Event::PointerMoved(inside),
                chain_pointer(inside, false, egui::Modifiers::NONE),
            ],
        );
        chain_authored_same(&f, &paper_before)?;
        assert!(
            chain_count_node(&f).is_err(),
            "outside-paper press cannot become an Add or free fallback"
        );
        if index == 0 {
            let mol = Path::new(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../io/tests/fixtures/mol/ethane-v2000.mol"
            ));
            let raw_mol = fs::read(mol)?;
            f.command(
                "document.open_mol",
                json!({"path":mol,"discard_unsaved":true}),
            )?;
            let foreign = f.state()?;
            let error = f
                .app
                .engine
                .lock()
                .map_err(|_| "engine")?
                .dispatch("ui.chain_tool", json!({"action":"select","kind":"acyclic"}))
                .err()
                .ok_or("foreign source must refuse even same mode")?;
            assert_eq!(error.info().code, "unsupported_content");
            chain_authored_same(&f, &foreign)?;
            chain3_frame(&mut f, origin, vec![]);
            chain3_frame(
                &mut f,
                origin,
                vec![
                    chain_pointer(press, true, egui::Modifiers::NONE),
                    egui::Event::PointerMoved(release),
                    chain_pointer(release, false, egui::Modifiers::NONE),
                ],
            );
            chain_authored_same(&f, &foreign)?;
            assert_eq!(fs::read(mol)?, raw_mol);
        }
    }
    Ok(())
}
