//! Existing actual-App harness; no proposed Rust API in the first runtime case.
use super::{Fixture, TestResult, ring_activation, ring_authored_same};
use eframe::egui;
use serde_json::json;

#[test]
fn benzene_is_a_current_actionable_native_control_and_selecting_it_never_authors() -> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    f.warm();
    let before = f.state()?;
    let (id, rect) = f.node(
        "Benzene",
        &[
            egui::accesskit::Role::Button,
            egui::accesskit::Role::CheckBox,
        ],
    )?;
    assert!(!f.nodes.get(&id).ok_or("Benzene control")?.is_disabled());
    assert!(
        rect.is_positive()
            && egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1160.0, 800.0))
                .contains_rect(rect)
    );
    let activate = ring_activation(&f, "Benzene")?;
    f.frame(vec![activate]);
    f.warm();
    let result = f.command("ui.controls", json!({}))?;
    let row = result["controls"]
        .as_array()
        .ok_or("controls")?
        .iter()
        .find(|r| r["id"] == "tool.ring.benzene")
        .ok_or("central Benzene binding")?;
    assert_eq!(row["selected"], true);
    assert_eq!(
        row["command"],
        json!({"command":"ui.ring_tool","params":{"kind":"benzene"}})
    );
    ring_authored_same(&f, &before)?;
    Ok(())
}

use super::super::{canvas, finish_actual_file_workflow};
use super::{
    ring_assert_graph, ring_assert_old_prefix, ring_choose, ring_compare_document, ring_graph,
    ring_hex, ring_ink_compare, ring_keys, ring_literal_source, ring_model, ring_one_edit,
    ring_point, ring_pointer, ring_review_owner_same, ring_review_seed_large, ring_seed,
    ring_selected, ring_shift, ring_shortcut, ring_wait,
};
use egui::{Event, Key, Modifiers, Pos2, accesskit::Role};
use serde_json::Value;
use std::{
    collections::BTreeSet,
    fs,
    io::Write,
    path::Path,
    process::{Command as Process, Stdio},
    sync::Arc,
};

fn pattern(
    mut graph: Value,
    weights: &[u8],
    h: &[u16],
    new_start: usize,
    side: &str,
) -> TestResult<Value> {
    let atoms = graph["atoms"].as_array_mut().ok_or("atoms")?;
    assert_eq!(atoms.len(), h.len());
    for (i, (a, h)) in atoms.iter_mut().zip(h).enumerate() {
        a["explicit_hydrogens"] = json!(h);
        if i >= new_start {
            a["show_non_terminal_carbon_labels"] = json!(false);
        }
    }
    let bonds = graph["bonds"].as_array_mut().ok_or("bonds")?;
    assert_eq!(bonds.len(), weights.len());
    for (b, w) in bonds.iter_mut().zip(weights) {
        b["order"] = json!(if *w == 2 { "double" } else { "single" });
        if *w == 2 {
            b["double_position"] = json!(side);
        } else {
            b.as_object_mut().ok_or("bond")?.remove("double_position");
        }
    }
    Ok(graph)
}
fn named_free(kind: &str, single: bool, horizontal: bool) -> TestResult<Value> {
    let q = 24.0 * 3.0_f64.sqrt();
    if kind == "Benzene" {
        let g = if horizontal {
            ring_graph(
                &[
                    [120.0, 120.0],
                    [168.0, 120.0],
                    [192.0, 120.0 + q],
                    [168.0, 120.0 + 2.0 * q],
                    [120.0, 120.0 + 2.0 * q],
                    [96.0, 120.0 + q],
                ],
                &[1; 6],
                &[[1, 2], [2, 3], [3, 4], [4, 5], [5, 6], [6, 1]],
            )
        } else {
            ring_hex(1.0)
        };
        pattern(
            g,
            if single {
                &[1, 2, 1, 2, 1, 2]
            } else {
                &[2, 1, 2, 1, 2, 1]
            },
            &[1; 6],
            0,
            "right",
        )
    } else {
        let r = 5.0_f64.sqrt();
        let dx = 12.0 * (10.0 + 2.0 * r).sqrt();
        let far = 12.0 * ((10.0 + 2.0 * r).sqrt() + (10.0 - 2.0 * r).sqrt());
        let t = 12.0 * (r - 1.0);
        pattern(
            ring_graph(
                &[
                    [96.0, 120.0],
                    [144.0, 120.0],
                    [144.0 + t, 120.0 + dx],
                    [120.0, 120.0 + far],
                    [96.0 - t, 120.0 + dx],
                ],
                &[1, 1, 1, 1, 2],
                &[[1, 2], [2, 3], [3, 4], [4, 5], [5, 1]],
            ),
            if single {
                &[1, 2, 1, 1, 2]
            } else {
                &[2, 1, 2, 1, 1]
            },
            if single {
                &[1, 1, 1, 2, 1]
            } else {
                &[1, 1, 1, 1, 2]
            },
            0,
            "right",
        )
    }
}
fn named_glyph(
    f: &Fixture,
    shown: &egui::FullOutput,
    name: &str,
    n: usize,
    doubles: usize,
) -> TestResult {
    let (_, rect) = f.node(name, &[Role::Button, Role::CheckBox])?;
    let inner = rect.shrink(3.0);
    let polygon = shown
        .shapes
        .iter()
        .find_map(|s| match &s.shape {
            egui::Shape::Path(p)
                if s.clip_rect.intersects(rect)
                    && p.closed
                    && p.points.len() == n
                    && p.stroke.width > 0.0
                    && p.points.iter().all(|v| inner.contains(*v)) =>
            {
                Some(&p.points)
            }
            _ => None,
        })
        .ok_or("named tool needs its actual closed polygon")?;
    let mut edges = BTreeSet::new();
    for s in &shown.shapes {
        if !s.clip_rect.intersects(rect) {
            continue;
        }
        let pair = match &s.shape {
            egui::Shape::LineSegment { points, stroke } if stroke.width > 0.0 => Some(*points),
            egui::Shape::Path(p) if !p.closed && p.points.len() == 2 && p.stroke.width > 0.0 => {
                Some([p.points[0], p.points[1]])
            }
            _ => None,
        };
        let Some([a, b]) = pair else {
            continue;
        };
        if !inner.contains(a) || !inner.contains(b) || a.distance(b) <= 1.0 {
            continue;
        }
        let inside = |p: Pos2| {
            let signs = polygon
                .iter()
                .zip(polygon.iter().cycle().skip(1))
                .map(|(a, b)| (b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x))
                .collect::<Vec<_>>();
            signs.iter().all(|s| *s > 1.0 / 1024.0) || signs.iter().all(|s| *s < -1.0 / 1024.0)
        };
        if !inside(a) || !inside(b) {
            continue;
        }
        let v = b - a;
        let mid = a + (b - a) * 0.5;
        let edge = polygon
            .iter()
            .zip(polygon.iter().cycle().skip(1))
            .enumerate()
            .filter_map(|(i, (c, d))| {
                let u = *d - *c;
                let parallel = (u.x * v.y - u.y * v.x).abs() / (u.length() * v.length());
                let distance = (u.x * (mid.y - c.y) - u.y * (mid.x - c.x)).abs() / u.length();
                (parallel <= 1.0 / 1024.0).then_some((i, distance))
            })
            .min_by(|(_, a), (_, b)| a.total_cmp(b))
            .map(|(i, _)| i)
            .ok_or("inset icon stroke is not parallel to a polygon edge")?;
        assert!(edges.insert(edge), "duplicate inset icon edge");
    }
    assert_eq!(
        edges.len(),
        doubles,
        "actual separate inward Double strokes"
    );
    for i in &edges {
        assert!(
            !edges.contains(&((i + 1) % n)),
            "icon Doubles must not be adjacent"
        );
    }
    Ok(())
}
fn named_paint(
    f: &Fixture,
    shown: &egui::FullOutput,
    graphs: &[Value],
    size: [f64; 2],
) -> TestResult<bondline_render::GraphScene> {
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
    for p in &graph.scene().primitives {
        match p {
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
    // Reuse existing exact primitive comparison, without its Single-only glyph-required precondition.
    ring_ink_compare(actual_rectangles, rectangles, density)?;
    ring_ink_compare(actual_quads, quads, density)?;
    Ok(graph)
}
fn named_consume(path: &Path, expected: &Value, smiles: &str, formula: &str) -> TestResult {
    // Extract only from independently authored EXPECTATIONS, never from actual output.
    let mut raw_expected = expected.clone();
    let atoms = raw_expected["atoms"].as_array_mut().ok_or("atoms")?;
    let flags = atoms
        .iter_mut()
        .map(|a| {
            a.as_object_mut().ok_or("atom").map(|a| {
                a.remove("show_non_terminal_carbon_labels")
                    .unwrap_or(Value::Null)
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let sides = raw_expected["bonds"]
        .as_array_mut()
        .ok_or("bonds")?
        .iter_mut()
        .map(|b| {
            b.as_object_mut()
                .ok_or("bond")
                .map(|b| b.remove("double_position").unwrap_or(Value::Null))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let args = serde_json::to_string(
        &json!({"graph":[raw_expected],"smiles":smiles,"formula":formula,"flags":flags,"double_positions":sides}),
    )?;
    let mut child =
        Process::new(std::env::var_os("BONDLINE_PYTHON").ok_or("pinned external Python")?)
            .args([
                "-B",
                concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../commands/tests/ring_create/named_ring_oracle.py"
                ),
                &args,
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
    child
        .stdin
        .take()
        .ok_or("consumer stdin")?
        .write_all(&fs::read(path)?)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

#[test]
fn named_tools_have_original_inward_icons_and_preview_the_complete_literal_candidate_before_one_release_and_unicode_save()
-> TestResult {
    for (kind, dpi, n, doubles, start, end, smiles, formula) in [
        (
            "Benzene",
            1.0,
            6,
            3,
            [120.0, 120.0],
            [168.0, 120.0],
            "c1ccccc1",
            "C6H6",
        ),
        (
            "Cyclopentadiene",
            2.0,
            5,
            2,
            [96.0, 120.0],
            [144.0, 120.0],
            "C1=CC=CC1",
            "C5H6",
        ),
    ] {
        let mut f = Fixture::new(1160.0, 800.0, dpi)?;
        let path = ring_seed(&mut f, None)?;
        let raw = fs::read(&path)?;
        ring_choose(&mut f, kind)?;
        let icon = f.warm();
        named_glyph(&f, &icon, kind, n, doubles)?;
        let before = f.state()?;
        let expected = named_free(kind, false, true)?;
        let a = ring_point(&mut f, start)?;
        let b = ring_point(&mut f, end)?;
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
        let scene = named_paint(&f, &held, std::slice::from_ref(&expected), [288.0, 288.0])?;
        assert_eq!(scene.label_em_px(), None);
        assert_eq!(
            scene
                .scene()
                .primitives
                .iter()
                .filter(|p| matches!(p, bondline_render::Primitive::Stroke(_)))
                .count(),
            n + doubles
        );
        f.frame(vec![ring_pointer(b, false, Modifiers::NONE)]);
        let shown = f.warm();
        ring_assert_graph(&f, &expected)?;
        ring_one_edit(&f, &before, &path, &raw)?;
        named_paint(&f, &shown, std::slice::from_ref(&expected), [288.0, 288.0])?;
        let committed = f.state()?;
        ring_shortcut(&mut f, Key::S, false);
        finish_actual_file_workflow(&mut f)?;
        assert_eq!(f.state()?["file"]["dirty"], false);
        named_consume(&path, &expected, smiles, formula)?;
        ring_shortcut(&mut f, Key::Z, false);
        assert_eq!(f.state()?["document"], before["document"]);
        assert_eq!(f.state()?["redo_depth"], 1);
        ring_shortcut(&mut f, Key::Z, true);
        ring_compare_document(&f.state()?["document"], &committed["document"])?;
        assert_eq!(f.state()?["file"]["dirty"], false);
        f.command("document.open_cdxml", json!({"path":path}))?;
        ring_compare_document(&f.state()?["document"], &committed["document"])?;
        assert_eq!(f.state()?["undo_depth"], 0);
        let reopened = f.state()?;
        ring_choose(&mut f, "Marquee selection")?;
        super::ring_click(&mut f, start)?;
        assert_eq!(f.state()?["ui_selection"]["atom_ids"], json!([1]));
        ring_authored_same(&f, &reopened)?;
    }
    Ok(())
}

#[test]
fn named_shift_is_latched_at_primary_press_across_frames_and_packets_while_legacy_shift_stays_refused()
-> TestResult {
    for (packet, press, release, single) in [
        (false, Modifiers::SHIFT, Modifiers::NONE, true),
        (false, Modifiers::NONE, Modifiers::SHIFT, false),
        (true, Modifiers::SHIFT, Modifiers::NONE, true),
        (true, Modifiers::NONE, Modifiers::SHIFT, false),
    ] {
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        let path = ring_seed(&mut f, None)?;
        let raw = fs::read(&path)?;
        ring_choose(&mut f, "Benzene")?;
        let before = f.state()?;
        let a = ring_point(&mut f, [120.0, 168.0])?;
        if packet {
            f.frame(vec![
                Event::PointerMoved(a),
                ring_pointer(a, true, press),
                ring_pointer(a, false, release),
            ]);
        } else {
            let held = f.frame(vec![Event::PointerMoved(a), ring_pointer(a, true, press)]);
            ring_authored_same(&f, &before)?;
            named_paint(
                &f,
                &held,
                &[named_free("Benzene", single, false)?],
                [288.0, 288.0],
            )?;
            f.frame(vec![ring_pointer(a, false, release)]);
        }
        f.warm();
        ring_assert_graph(&f, &named_free("Benzene", single, false)?)?;
        ring_one_edit(&f, &before, &path, &raw)?;
    }
    for (name, modifiers) in [
        ("Cyclohexane", Modifiers::SHIFT),
        ("Benzene", Modifiers::COMMAND),
        (
            "Benzene",
            Modifiers {
                ctrl: true,
                ..Modifiers::NONE
            },
        ),
        (
            "Benzene",
            Modifiers {
                alt: true,
                ..Modifiers::NONE
            },
        ),
    ] {
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        let path = ring_seed(&mut f, None)?;
        let raw = fs::read(&path)?;
        ring_choose(&mut f, name)?;
        let before = f.state()?;
        let a = ring_point(&mut f, [120.0, 168.0])?;
        f.frame(vec![
            Event::PointerMoved(a),
            ring_pointer(a, true, modifiers),
            ring_pointer(a, false, modifiers),
        ]);
        f.warm();
        ring_authored_same(&f, &before)?;
        assert_eq!(fs::read(&path)?, raw);
        assert!(f.app.error.is_some());
        assert_eq!(f.state()?["ui_atom_label"], Value::Null);
    }
    Ok(())
}

// Further independent acceptance appended after the complete gesture/Shift checkpoint.
#[test]
fn real_terminal_fusion_spiro_and_inward_shared_double_keep_old_data_and_never_fall_back_from_phase_conflict()
-> TestResult {
    let q = 24.0 * 3.0_f64.sqrt();
    let r = 5.0_f64.sqrt();
    let dx = 12.0 * (10.0 + 2.0 * r).sqrt();
    let far = 12.0 * ((10.0 + 2.0 * r).sqrt() + (10.0 - 2.0 * r).sqrt());
    let t = 12.0 * (r - 1.0);
    for case in ["terminal", "fused", "spiro", "double"] {
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        let path = if case == "terminal" {
            ring_seed(&mut f, Some([[144.0, 144.0], [192.0, 144.0]]))?
        } else if case == "double" {
            let mut seed = ring_graph(&[[120.0, 168.0], [120.0, 120.0]], &[2, 2], &[[1, 2]]);
            seed["bonds"][0]["order"] = json!("double");
            seed["bonds"][0]["double_position"] = json!("right");
            for a in seed["atoms"].as_array_mut().ok_or("atoms")? {
                a["show_non_terminal_carbon_labels"] = json!(false);
            }
            ring_literal_source(&mut f, &[seed], [288.0, 288.0])?
        } else {
            ring_literal_source(&mut f, &[ring_hex(1.0)], [288.0, 288.0])?
        };
        let raw = fs::read(&path)?;
        ring_choose(
            &mut f,
            if case == "spiro" {
                "Cyclopentadiene"
            } else {
                "Benzene"
            },
        )?;
        let before = f.state()?;
        if case == "double" {
            let at = ring_point(&mut f, [120.0, 144.0])?;
            f.frame(vec![
                Event::PointerMoved(at),
                ring_pointer(at, true, Modifiers::SHIFT),
                ring_pointer(at, false, Modifiers::NONE),
            ]);
            f.warm();
            ring_authored_same(&f, &before)?;
            assert!(
                f.app.error.is_some(),
                "Single-first must not retry contextual against preserved Double"
            );
            assert_eq!(fs::read(&path)?, raw);
        }
        let (expected, shared, smiles, formula) = match case {
            "terminal" => (
                pattern(
                    ring_graph(
                        &[
                            [144.0, 144.0],
                            [192.0, 144.0],
                            [120.0, 144.0 + q],
                            [72.0, 144.0 + q],
                            [48.0, 144.0],
                            [72.0, 144.0 - q],
                            [120.0, 144.0 - q],
                        ],
                        &[0, 3, 1, 1, 1, 1, 1],
                        &[[1, 2], [1, 3], [3, 4], [4, 5], [5, 6], [6, 7], [7, 1]],
                    ),
                    &[1, 2, 1, 2, 1, 2, 1],
                    &[0, 3, 1, 1, 1, 1, 1],
                    2,
                    "right",
                )?,
                vec![(1, 0)],
                "Cc1ccccc1",
                "C7H8",
            ),
            "fused" => (
                pattern(
                    super::ring_fused(),
                    &[1, 1, 1, 1, 1, 1, 2, 1, 2, 1, 2],
                    &[0, 0, 2, 2, 2, 2, 1, 1, 1, 1],
                    6,
                    "left",
                )?,
                vec![(1, 0), (2, 0)],
                "c1ccc2CCCCc2c1",
                "C10H12",
            ),
            "spiro" => (
                pattern(
                    ring_graph(
                        &[
                            [120.0, 168.0],
                            [120.0, 120.0],
                            [120.0 + q, 96.0],
                            [120.0 + 2.0 * q, 120.0],
                            [120.0 + 2.0 * q, 168.0],
                            [120.0 + q, 192.0],
                            [120.0, 216.0],
                            [120.0 - dx, 216.0 + t],
                            [120.0 - far, 192.0],
                            [120.0 - dx, 168.0 - t],
                        ],
                        &[0, 2, 2, 2, 2, 2, 1, 1, 1, 1],
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
                            [10, 1],
                        ],
                    ),
                    &[1, 1, 1, 1, 1, 1, 1, 2, 1, 2, 1],
                    &[0, 2, 2, 2, 2, 2, 1, 1, 1, 1],
                    6,
                    "right",
                )?,
                vec![(1, 0)],
                "C1CCC2(CC1)C=CC=C2",
                "C10H14",
            ),
            _ => (
                named_free("Benzene", false, false)?,
                vec![(1, 1), (2, 1)],
                "c1ccccc1",
                "C6H6",
            ),
        };
        if case == "spiro" {
            let a = ring_point(&mut f, [120.0, 168.0])?;
            let b = ring_point(&mut f, [120.0, 216.0])?;
            f.frame(vec![
                Event::PointerMoved(a),
                ring_pointer(a, true, Modifiers::NONE),
            ]);
            let held = f.frame(vec![Event::PointerMoved(b)]);
            ring_authored_same(&f, &before)?;
            named_paint(&f, &held, std::slice::from_ref(&expected), [288.0, 288.0])?;
            f.frame(vec![ring_pointer(b, false, Modifiers::NONE)]);
        } else {
            super::ring_click(
                &mut f,
                if case == "terminal" {
                    [144.0, 144.0]
                } else {
                    [120.0, 144.0]
                },
            )?;
        }
        let shown = f.warm();
        ring_assert_graph(&f, &expected)?;
        ring_assert_old_prefix(&f, &before, &shared)?;
        ring_one_edit(&f, &before, &path, &raw)?;
        named_paint(&f, &shown, std::slice::from_ref(&expected), [288.0, 288.0])?;
        let committed = f.state()?;
        ring_shortcut(&mut f, Key::S, false);
        finish_actual_file_workflow(&mut f)?;
        named_consume(&path, &expected, smiles, formula)?;
        ring_shortcut(&mut f, Key::Z, false);
        assert_eq!(f.state()?["document"], before["document"]);
        ring_shortcut(&mut f, Key::Z, true);
        ring_compare_document(&f.state()?["document"], &committed["document"])?;
        f.command("document.open_cdxml", json!({"path":path}))?;
        ring_assert_graph(&f, &expected)?;
    }
    Ok(())
}

#[test]
fn named_mode_aba_noop_raw_order_future_shift_modal_and_physical_refusal_keep_gestures_source_qualified()
-> TestResult {
    for action in [
        "same",
        "kind_aba",
        "sized_aba",
        "release_first",
        "view_aba",
        "escape",
    ] {
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        let path = ring_seed(&mut f, None)?;
        let raw = fs::read(&path)?;
        ring_choose(&mut f, "Benzene")?;
        let before = f.state()?;
        let a = ring_point(&mut f, [120.0, 168.0])?;
        let epoch = f.app.engine.lock().map_err(|_| "engine")?.bond_tool().epoch;
        f.frame(vec![
            Event::PointerMoved(a),
            ring_pointer(a, true, Modifiers::NONE),
        ]);
        if action == "view_aba" {
            f.command("ui.view", json!({"action":"set_zoom","percent":150}))?;
            f.command("ui.view", json!({"action":"actual"}))?;
        }
        let same = ring_activation(&f, "Benzene")?;
        let cp = ring_activation(&f, "Cyclopentadiene")?;
        let sized = ring_activation(&f, "Cyclohexane")?;
        let packet = match action {
            "same" => vec![same, ring_pointer(a, false, Modifiers::NONE)],
            "kind_aba" => vec![cp, same, ring_pointer(a, false, Modifiers::NONE)],
            "sized_aba" => vec![sized, same, ring_pointer(a, false, Modifiers::NONE)],
            "release_first" => vec![ring_pointer(a, false, Modifiers::NONE), cp],
            "view_aba" => vec![ring_pointer(a, false, Modifiers::NONE)],
            _ => [
                ring_keys(Key::Escape, Modifiers::NONE),
                vec![ring_pointer(a, false, Modifiers::NONE)],
            ]
            .concat(),
        };
        f.frame(packet);
        f.warm();
        if action == "same" || action == "release_first" {
            ring_assert_graph(&f, &named_free("Benzene", false, false)?)?;
            ring_one_edit(&f, &before, &path, &raw)?;
            assert_eq!(
                f.app.engine.lock().map_err(|_| "engine")?.bond_tool().epoch,
                epoch + u64::from(action == "release_first")
            );
        } else {
            ring_authored_same(&f, &before)?;
            if action == "kind_aba" || action == "sized_aba" {
                assert_eq!(
                    f.app.engine.lock().map_err(|_| "engine")?.bond_tool().epoch,
                    epoch + 2
                );
            }
            super::ring_click(&mut f, [120.0, 168.0])?;
            ring_assert_graph(&f, &named_free("Benzene", false, false)?)?;
        }
        assert_eq!(
            ring_selected(&f)?,
            vec![if action == "release_first" {
                "tool.ring.cyclopentadiene"
            } else {
                "tool.ring.benzene"
            }]
        );
    }
    // Later Shift in the same raw packet must not leak into the already completed first gesture.
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let path = ring_literal_source(&mut f, &[], [500.0, 700.0])?;
    ring_choose(&mut f, "Benzene")?;
    let before = f.state()?;
    let a = ring_point(&mut f, [96.0, 168.0])?;
    let b = ring_point(&mut f, [300.0, 420.0])?;
    f.frame(vec![
        Event::PointerMoved(a),
        ring_pointer(a, true, Modifiers::NONE),
        ring_pointer(a, false, Modifiers::NONE),
        Event::PointerMoved(b),
        ring_pointer(b, true, Modifiers::SHIFT),
        ring_pointer(b, false, Modifiers::NONE),
    ]);
    f.warm();
    let first = ring_shift(named_free("Benzene", false, false)?, 1, 0, 0, [-24.0, 0.0])?;
    let second = ring_shift(named_free("Benzene", true, false)?, 2, 6, 6, [180.0, 252.0])?;
    let expected = serde_json::to_value(ring_model(&[first.clone(), second], [500.0, 700.0])?)?;
    ring_compare_document(&f.state()?["document"], &expected)?;
    assert_eq!(
        f.state()?["revision"],
        before["revision"].as_u64().ok_or("revision")? + 2
    );
    assert_eq!(f.state()?["undo_depth"], 2);
    let mut history_save = ring_keys(Key::Z, Modifiers::COMMAND);
    history_save.extend(ring_keys(Key::S, Modifiers::COMMAND));
    f.frame(history_save);
    finish_actual_file_workflow(&mut f)?;
    let expected = serde_json::to_value(ring_model(std::slice::from_ref(&first), [500.0, 700.0])?)?;
    ring_compare_document(&f.state()?["document"], &expected)?;
    assert_eq!(f.state()?["undo_depth"], 1);
    assert_eq!(f.state()?["redo_depth"], 1);
    assert_eq!(f.state()?["file"]["dirty"], false);
    named_consume(&path, &first, "c1ccccc1", "C6H6")?;

    // Existing actual Atom-label modal and its Cancel frame fence every stale release/Save/history suffix.
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let path = ring_seed(&mut f, Some([[120.0, 168.0], [120.0, 120.0]]))?;
    let raw = fs::read(&path)?;
    ring_choose(&mut f, "Benzene")?;
    let before = f.state()?;
    let a = ring_point(&mut f, [120.0, 168.0])?;
    f.frame(vec![
        Event::PointerMoved(a),
        ring_pointer(a, true, Modifiers::NONE),
    ]);
    f.command("ui.atom_label",json!({"action":"open","expected_revision":before["revision"],"page_id":1,"fragment_id":1,"atom_id":1}))?;
    ring_wait(&mut f, |f, _| {
        f.node("Element", &[Role::TextInput]).is_ok()
            && f.node("Cancel", &[Role::Button]).is_ok_and(|(id, rect)| {
                rect.is_positive()
                    && f.nodes.get(&id).is_some_and(|node| {
                        !node.is_disabled()
                            && node.default_action_verb()
                                == Some(egui::accesskit::DefaultActionVerb::Click)
                    })
            })
    })?;
    let (cancel, _) = f.node("Cancel", &[Role::Button])?;
    assert!(
        !f.nodes
            .get(&cancel)
            .ok_or("current Cancel node")?
            .is_disabled(),
        "the first surfaced Cancel node is disabled"
    );
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

    // The existing full-source physical bound must retire named input too, without an Engine revision.
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let path = ring_review_seed_large(&mut f, 60000.0, 1000.0, 100.0, [30000.0, 500.0])?;
    let raw = fs::read(&path)?;
    ring_choose(&mut f, "Benzene")?;
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
    f.frame(vec![
        Event::PointerMoved(a),
        ring_pointer(a, true, Modifiers::NONE),
    ]);
    f.dpi = 2.0;
    let refused = f.frame(vec![]);
    let guard = bondline_render::Error::InvalidViewport.to_string();
    assert!(
        f.app.error.as_ref().is_some_and(|s| s.contains(&guard))
            || refused.shapes.iter().any(
                |s| matches!(&s.shape,egui::Shape::Text(t)if t.galley.text().contains(&guard))
            )
    );
    f.dpi = 1.0;
    f.frame(vec![]);
    f.frame(vec![ring_pointer(a, false, Modifiers::NONE)]);
    f.warm();
    ring_authored_same(&f, &before)?;
    ring_review_owner_same(&f, &stamp, &model, &file)?;
    assert_eq!(f.view()?, view);
    assert_eq!(fs::read(&path)?, raw);
    super::ring_click(&mut f, [30000.0, 500.0])?;
    ring_assert_graph(
        &f,
        &ring_shift(
            named_free("Benzene", false, false)?,
            1,
            0,
            0,
            [29880.0, 332.0],
        )?,
    )?;
    Ok(())
}
