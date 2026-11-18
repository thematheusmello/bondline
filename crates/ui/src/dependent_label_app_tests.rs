//! Runtime declarations adopted beneath native_atom_label_tests.rs; no new product hooks.
use super::{
    Fixture, TestResult, canvas, click_alt, close_form, finish_actual_file_workflow, key, painted,
    projected, set, shortcut, value, wait,
};
use eframe::egui::{self, Event, Key, Modifiers, PointerButton, Pos2, accesskit::Role};
use serde_json::{Value, json};
use std::{fs, path::PathBuf, sync::Arc};
#[path = "../../../xtask/oracles/dependent_label_support.rs"]
#[allow(dead_code, reason = "shared frozen independent source/facts reader")]
mod independent;

fn point(name: &str) -> [f64; 2] {
    if name.starts_with("boxed") {
        [84.1220703125, 95.734375]
    } else {
        [160.0, 160.0]
    }
}
fn pointer(pos: Pos2, pressed: bool, modifiers: Modifiers) -> Event {
    Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers,
    }
}
fn at(f: &mut Fixture, point: [f64; 2]) -> TestResult<Pos2> {
    let shown = f.warm();
    let view = f.view()?;
    assert_eq!(view["zoom_percent"], 100.0);
    let center = match (view["center_x_pt"].as_f64(), view["center_y_pt"].as_f64()) {
        (Some(x), Some(y)) => Some([x, y]),
        _ => None,
    };
    let area = canvas(&shown)?;
    let p = projected(area, [512.0, 512.0], 100.0, center, point);
    assert!(
        area.contains(p),
        "declared point must be inside the actual canvas"
    );
    Ok(p)
}
fn click(f: &mut Fixture, p: [f64; 2]) -> TestResult {
    let p = at(f, p)?;
    f.frame(vec![
        Event::PointerMoved(p),
        pointer(p, true, Modifiers::NONE),
    ]);
    f.frame(vec![pointer(p, false, Modifiers::NONE)]);
    Ok(())
}
fn activate(f: &mut Fixture) -> TestResult {
    f.warm();
    let (id, rect) = f.node("Marquee selection", &[Role::Button, Role::CheckBox])?;
    assert!(
        !f.nodes.get(&id).ok_or("marquee control")?.is_disabled(),
        "existing Marquee control must be enabled for the qualified label source"
    );
    f.frame(vec![
        Event::PointerMoved(rect.center()),
        pointer(rect.center(), true, Modifiers::NONE),
    ]);
    f.frame(vec![pointer(rect.center(), false, Modifiers::NONE)]);
    assert_eq!(f.state()?["ui_selection"]["active"], true);
    Ok(())
}
fn open(f: &mut Fixture, name: &str, retained: &str) -> TestResult<PathBuf> {
    match retained {
        "ring" => {
            f.command("ui.ring_tool", json!({"size":6}))?;
        }
        "chain" => {
            f.command("ui.chain_tool", json!({"action":"select","kind":"acyclic"}))?;
        }
        "selection" => {
            f.command(
                "ui.selection",
                json!({"action":"activate","expected_revision":0}),
            )?;
        }
        "bond" => {}
        _ => return Err("declared retained mode".into()),
    }
    let mode = f.app.engine.lock().map_err(|_| "engine")?.effective_tool();
    let input = f.directory.path(&format!("dependent-{name}-原-λ.cdxml"));
    fs::write(&input, independent::source(name))?;
    f.warm();
    shortcut(f, Key::O, false);
    wait(f, |f, _| f.node("File name", &[Role::TextInput]).is_ok())?;
    set(f, "File name", input.to_str().ok_or("input path")?)?;
    f.choose("Open")?;
    finish_actual_file_workflow(f)?;
    f.command("ui.view", json!({"action":"actual"}))?;
    f.warm();
    assert_eq!(
        f.app.engine.lock().map_err(|_| "engine")?.effective_tool(),
        mode
    );
    assert_eq!(f.state()?["file"]["dirty"], false);
    Ok(input)
}
fn authored_unchanged(f: &Fixture, before: &Value) -> TestResult {
    let after = f.state()?;
    for key in [
        "document",
        "revision",
        "undo_depth",
        "redo_depth",
        "source",
        "file",
    ] {
        assert_eq!(
            after[key], before[key],
            "transient canvas operation authored {key}"
        );
    }
    Ok(())
}
fn encoded(f: &Fixture) -> TestResult<Vec<u8>> {
    Ok(f.app
        .engine
        .lock()
        .map_err(|_| "engine")?
        .stored_document()?
        .to_cdxml()?
        .into_owned())
}
fn blue_spans(out: &egui::FullOutput) -> TestResult<Vec<[f32; 4]>> {
    let area = canvas(out)?;
    let spans = out
        .shapes
        .iter()
        .filter_map(|paint| match &paint.shape {
            egui::Shape::Rect(r)
                if r.fill == egui::Color32::from_rgb(51, 102, 204)
                    && area.contains_rect(r.rect)
                    && paint.clip_rect.contains_rect(r.rect) =>
            {
                Some([r.rect.min.x, r.rect.min.y, r.rect.width(), r.rect.height()])
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(
        !spans.is_empty(),
        "actual blue glyph spans must be painted, not a state-only preview"
    );
    Ok(spans)
}

#[test]
fn dependent_labels_actual_marquee_drag_nudges_history_unicode_save_reopen_and_alt_cancel()
-> TestResult {
    for name in ["boxed", "connected"] {
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        let input = open(&mut f, name, "ring")?;
        let original = fs::read(&input)?;
        activate(&mut f)?;
        click(&mut f, point(name))?;
        assert_eq!(f.state()?["ui_selection"]["atom_ids"], json!([1]));
        shortcut(&mut f, Key::A, false);
        let ids = if name == "boxed" {
            vec![1]
        } else {
            vec![1, 2, 3]
        };
        assert_eq!(f.state()?["ui_selection"]["atom_ids"], json!(ids));
        let before = f.state()?;
        let camera = f.view()?;
        let start = at(&mut f, point(name))?;
        let end = start + egui::vec2(24.0, -12.0);
        let painted_before = blue_spans(&f.warm())?;
        let (stamp, model) = {
            let engine = f.app.engine.lock().map_err(|_| "engine")?;
            (
                engine.capture_stamp(),
                engine.stored_document()?.model_snapshot(),
            )
        };
        f.frame(vec![
            Event::PointerMoved(start),
            pointer(start, true, Modifiers::NONE),
        ]);
        let preview = f.frame(vec![Event::PointerMoved(end)]);
        authored_unchanged(&f, &before)?;
        assert_eq!(
            fs::read(&input)?,
            original,
            "preview must not publish the source"
        );
        {
            let engine = f.app.engine.lock().map_err(|_| "engine")?;
            assert_eq!(engine.capture_stamp(), stamp);
            assert!(Arc::ptr_eq(
                &model,
                &engine.stored_document()?.model_snapshot()
            ));
        }
        assert!(f.app.drawing.selection.is_dragging());
        let expected_pixels = painted_before
            .iter()
            .map(|r| [r[0] + 24.0, r[1] - 12.0, r[2], r[3]])
            .collect::<Vec<_>>();
        assert_eq!(
            blue_spans(&preview)?,
            expected_pixels,
            "fixed100%/1x preview must move every original glyph span by the declared integer delta"
        );
        f.frame(vec![pointer(end, false, Modifiers::NONE)]);
        let moved = independent::model_after(&before["document"], &ids, 24.0, -12.0)?;
        assert_eq!(f.state()?["document"], moved);
        assert_eq!(f.state()?["undo_depth"], 1);
        assert_eq!(
            f.state()?["revision"],
            before["revision"].as_u64().ok_or("revision")? + 1
        );
        let moved_tag = format!("{name}_all_p24_m12");
        assert_eq!(encoded(&f)?, independent::source(&moved_tag).as_bytes());
        assert_eq!(
            independent::read(&encoded(&f)?)?,
            independent::facts(&moved_tag)
        );
        f.frame(vec![
            key(Key::ArrowRight, true, Modifiers::NONE),
            key(Key::ArrowRight, false, Modifiers::NONE),
        ]);
        let shift = Modifiers {
            shift: true,
            ..Modifiers::NONE
        };
        f.frame(vec![
            key(Key::ArrowDown, true, shift),
            key(Key::ArrowDown, false, shift),
        ]);
        let final_doc = independent::model_after(&before["document"], &ids, 25.0, -2.0)?;
        assert_eq!(f.state()?["document"], final_doc);
        assert_eq!(f.state()?["undo_depth"], 3);
        assert_eq!(f.view()?, camera);
        let final_tag = format!("{name}_nudge_p25_m2");
        assert_eq!(encoded(&f)?, independent::source(&final_tag).as_bytes());
        for _ in 0..3 {
            shortcut(&mut f, Key::Z, false);
        }
        assert_eq!(f.state()?["document"], before["document"]);
        assert_eq!(encoded(&f)?, original);
        for _ in 0..3 {
            shortcut(&mut f, Key::Z, true);
        }
        assert_eq!(f.state()?["document"], final_doc);
        let output = f.directory.path(&format!("dependent-{name}-移動-λ.cdxml"));
        shortcut(&mut f, Key::S, true);
        wait(&mut f, |f, _| {
            f.node("File name", &[Role::TextInput]).is_ok()
        })?;
        set(&mut f, "File name", output.to_str().ok_or("output")?)?;
        f.choose("Save")?;
        finish_actual_file_workflow(&mut f)?;
        let raw = fs::read(&output)?;
        assert_eq!(raw, independent::source(&final_tag).as_bytes());
        assert_eq!(independent::read(&raw)?, independent::facts(&final_tag));
        assert_eq!(f.state()?["file"]["dirty"], false);
        shortcut(&mut f, Key::O, false);
        wait(&mut f, |f, _| {
            f.node("File name", &[Role::TextInput]).is_ok()
        })?;
        set(&mut f, "File name", output.to_str().ok_or("output")?)?;
        f.choose("Open")?;
        finish_actual_file_workflow(&mut f)?;
        f.command("ui.view", json!({"action":"actual"}))?;
        assert_eq!(f.state()?["document"], final_doc);
        let after_reopen = f.state()?;
        let p = point(name);
        click_alt(&mut f, [p[0] + 25.0, p[1] - 2.0])?;
        wait(&mut f, |f, o| {
            painted(o, "Atom label") && value(f, "Element").is_ok()
        })?;
        assert_eq!(value(&f, "Element")?, "O");
        set(&mut f, "Element", "N")?;
        close_form(&mut f, "Cancel")?;
        authored_unchanged(&f, &after_reopen)?;
        assert_eq!(fs::read(&input)?, original);
        assert_eq!(fs::read(&output)?, raw);
    }
    Ok(())
}

#[test]
fn dependent_connected_double_click_component_move_preserves_same_fragment_island_and_refuses_offpaper_preview()
-> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let input = open(&mut f, "connected_component", "selection")?;
    activate(&mut f)?;
    let p = at(&mut f, [160.0, 160.0])?;
    f.frame(vec![
        Event::PointerMoved(p),
        pointer(p, true, Modifiers::NONE),
        pointer(p, false, Modifiers::NONE),
        pointer(p, true, Modifiers::NONE),
        pointer(p, false, Modifiers::NONE),
    ]);
    assert_eq!(
        f.state()?["ui_selection"]["atom_ids"],
        json!([1, 2, 3]),
        "double-click selects only the connected water component, not isolated appAtom4 in the same fragment"
    );
    let before = f.state()?;
    let original = fs::read(&input)?;
    let (stamp, model, island) = {
        let engine = f.app.engine.lock().map_err(|_| "engine")?;
        let model = engine.stored_document()?.model_snapshot();
        let island = Arc::clone(
            model.pages()[0].fragments()[0].atoms()[3]
                .authored_label
                .as_ref()
                .ok_or("island label")?,
        );
        (engine.capture_stamp(), model, island)
    };
    // Pointer stays on paper while selected H atx80 would move tox-20.
    let invalid = at(&mut f, [60.0, 160.0])?;
    f.frame(vec![
        Event::PointerMoved(p),
        pointer(p, true, Modifiers::NONE),
    ]);
    f.frame(vec![Event::PointerMoved(invalid)]);
    authored_unchanged(&f, &before)?;
    assert!(
        f.app.error.is_some(),
        "the final-page physical failure must be reported during preview"
    );
    f.frame(vec![pointer(invalid, false, Modifiers::NONE)]);
    authored_unchanged(&f, &before)?;
    {
        let engine = f.app.engine.lock().map_err(|_| "engine")?;
        assert_eq!(engine.capture_stamp(), stamp);
        assert!(Arc::ptr_eq(
            &model,
            &engine.stored_document()?.model_snapshot()
        ));
    }
    assert_eq!(fs::read(&input)?, original);
    let start = at(&mut f, [160.0, 160.0])?;
    let end = start + egui::vec2(24.0, -12.0);
    f.frame(vec![
        Event::PointerMoved(start),
        pointer(start, true, Modifiers::NONE),
        Event::ModifiersChanged(Modifiers::SHIFT),
        Event::PointerMoved(end),
        Event::ModifiersChanged(Modifiers::NONE),
        pointer(end, false, Modifiers::NONE),
    ]);
    let expected = independent::model_after(&before["document"], &[1, 2, 3], 24.0, -12.0)?;
    assert_eq!(f.state()?["document"], expected);
    assert_eq!(f.state()?["undo_depth"], 1);
    {
        let engine = f.app.engine.lock().map_err(|_| "engine")?;
        assert!(Arc::ptr_eq(
            &island,
            engine.document()?.pages()[0].fragments()[0].atoms()[3]
                .authored_label
                .as_ref()
                .ok_or("island label")?
        ));
    }
    let raw = encoded(&f)?;
    assert_eq!(
        raw,
        independent::source("connected_component_p24_m12").as_bytes()
    );
    assert_eq!(
        independent::read(&raw)?,
        independent::facts("connected_component_p24_m12")
    );
    assert_eq!(fs::read(&input)?, original);
    Ok(())
}

#[test]
fn dependent_retained_modes_stay_readonly_and_selection_keeps_exact_alt_and_digit_fences()
-> TestResult {
    for name in ["boxed", "connected"] {
        for mode in ["ring", "chain"] {
            let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
            open(&mut f, name, mode)?;
            let before = f.state()?;
            let retained = f.app.engine.lock().map_err(|_| "engine")?.effective_tool();
            let start = at(&mut f, point(name))?;
            let end = start + egui::vec2(24.0, -12.0);
            f.frame(vec![
                Event::PointerMoved(start),
                pointer(start, true, Modifiers::NONE),
                Event::PointerMoved(end),
                pointer(end, false, Modifiers::NONE),
            ]);
            authored_unchanged(&f, &before)?;
            assert_eq!(
                f.app.engine.lock().map_err(|_| "engine")?.effective_tool(),
                retained
            );
            let controls = f.command("ui.controls", json!({}))?;
            for id in ["tool.ring.6", "tool.ring.chair_1", "chain.acyclic"] {
                assert!(
                    !controls["controls"]
                        .as_array()
                        .ok_or("controls")?
                        .iter()
                        .any(|c| c["id"] == id && c["enabled"] == true),
                    "selection authority enabled unrelated {id}"
                );
            }
            click_alt(&mut f, point(name))?;
            wait(&mut f, |f, o| {
                painted(o, "Atom label") && value(f, "Element").is_ok()
            })?;
            close_form(&mut f, "Cancel")?;
            authored_unchanged(&f, &before)?;
            activate(&mut f)?;
            click(&mut f, point(name))?;
            let selected = f.state()?;
            let p = at(&mut f, point(name))?;
            let invalid = Modifiers {
                alt: true,
                mac_cmd: true,
                ..Modifiers::NONE
            };
            f.frame(vec![
                Event::PointerMoved(p),
                pointer(p, true, invalid),
                Event::ModifiersChanged(Modifiers::NONE),
                pointer(p, false, Modifiers::NONE),
            ]);
            let shown = f.warm();
            assert!(!painted(&shown, "Atom label"));
            assert!(f.node("Element", &[Role::TextInput]).is_err());
            authored_unchanged(&f, &selected)?;
            assert_eq!(f.state()?["ui_selection"], selected["ui_selection"]);
            assert!(!f.app.drawing.selection.is_dragging());
            // Exact Alt press wins even if a later ordered report/release has no Alt.
            let alt = Modifiers {
                alt: true,
                ..Modifiers::NONE
            };
            f.frame(vec![
                pointer(p, true, alt),
                Event::ModifiersChanged(Modifiers::NONE),
                pointer(p, false, Modifiers::NONE),
            ]);
            wait(&mut f, |f, o| {
                painted(o, "Atom label") && value(f, "Element").is_ok()
            })?;
            close_form(&mut f, "Cancel")?;
            authored_unchanged(&f, &selected)?;
            assert_eq!(f.state()?["ui_selection"], selected["ui_selection"]);
            let hotspot = if name == "connected" {
                [200.0, 190.0]
            } else {
                point(name)
            };
            let hotspot = at(&mut f, hotspot)?;
            f.frame(vec![Event::PointerMoved(hotspot)]);
            for digit in [Key::Num1, Key::Num2, Key::Num3, Key::Num9, Key::Num0] {
                f.frame(vec![
                    key(digit, true, Modifiers::NONE),
                    key(digit, false, Modifiers::NONE),
                ]);
                authored_unchanged(&f, &selected)?;
                assert_eq!(f.state()?["ui_selection"], selected["ui_selection"]);
                assert!(
                    f.app.error.is_none(),
                    "blocked digit must not dispatch an unsupported order/chair command"
                );
            }
        }
    }
    Ok(())
}
