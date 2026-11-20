//! Actual App frames using independent CDX and CDXML caption witnesses.
#[path = "native_caption_creation_tests.rs"]
mod native_caption_creation_tests;

use super::{Fixture, TestResult, button, canvas, finish_actual_file_workflow, key, projected};
use eframe::egui::{
    self, Event, Key, Modifiers, PointerButton, Pos2,
    accesskit::{Action, ActionData, ActionRequest, Role},
};
use serde_json::{Value, json};
use std::{fs, path::Path};

const MULTI: &[u8] = include_bytes!("../../io/tests/fixtures/cdx-caption-edit/multi_before.cdx");
const SHIFTED: &[u8] = include_bytes!("../../io/tests/fixtures/cdx-caption-edit/multi_after.cdx");
const TAIL: &[u8] = include_bytes!("../../io/tests/fixtures/cdx-caption-edit/tail_after.cdx");
const WITNESSES: &str = include_str!("../../io/tests/fixtures/cdx-caption-edit/cases.json");
const CDXML: &str = include_str!("../../io/tests/fixtures/captions/explicit.cdxml");

// Interior Roboto ink points from the original font oracle, in paper points.
// The H lies inside both overprinted captions. The space run has no ink.
const OVERLAP: [f64; 2] = [100.5, 96.375];
const FIRST_RUN: [f64; 2] = [127.5, 223.125];
const SPACE_RUN: [f64; 2] = [151.9, 217.5];
const LAST_RUN: [f64; 2] = [188.0, 216.5];
const BONDLINE: [f64; 2] = [82.5, 96.3125];
const PHASE: [f64; 2] = [73.5, 91.5];

fn paper_point(f: &mut Fixture, point: [f64; 2]) -> TestResult<Pos2> {
    let paper = canvas(&f.warm())?;
    let state = f.state()?;
    let size = &state["document"]["pages"][0]["size"];
    let width = size["width_pt"].as_f64().ok_or("paper width")?;
    let height = size["height_pt"].as_f64().ok_or("paper height")?;
    assert_eq!(f.view()?["zoom_percent"], 100.0);
    let position = projected(paper, [width, height], 100.0, None, point);
    assert!(paper.contains(position));
    Ok(position)
}

fn click(f: &mut Fixture, position: Pos2, modifiers: Modifiers) {
    f.frame(vec![
        Event::PointerMoved(position),
        Event::PointerButton {
            pos: position,
            button: PointerButton::Primary,
            pressed: true,
            modifiers,
        },
    ]);
    f.frame(vec![Event::PointerButton {
        pos: position,
        button: PointerButton::Primary,
        pressed: false,
        modifiers,
    }]);
    f.warm();
}

fn shortcut(f: &mut Fixture, code: Key, shift: bool) {
    let modifiers = Modifiers {
        shift,
        ..Modifiers::COMMAND
    };
    f.frame(vec![
        key(code, true, modifiers),
        key(code, false, modifiers),
    ]);
}

fn field_value(f: &Fixture, label: &str) -> TestResult<String> {
    let (id, _) = f.node(label, &[Role::TextInput, Role::MultilineTextInput])?;
    Ok(f.nodes
        .get(&id)
        .and_then(|node| node.value())
        .ok_or("accessible text value")?
        .to_owned())
}

fn set_field(f: &mut Fixture, label: &str, text: &str) -> TestResult {
    let (id, _) = f.node(label, &[Role::TextInput, Role::MultilineTextInput])?;
    f.frame(vec![Event::AccessKitActionRequest(ActionRequest {
        action: Action::SetValue,
        target: id,
        data: Some(ActionData::Value(text.into())),
    })]);
    f.warm();
    assert_eq!(field_value(f, label)?, text);
    Ok(())
}

fn open_run(
    f: &mut Fixture,
    point: [f64; 2],
    text_id: u32,
    run_index: usize,
    run_count: usize,
    text: &str,
) -> TestResult<Value> {
    let position = paper_point(f, point)?;
    f.frame(vec![Event::PointerMoved(position), button(position, true)]);
    assert!(
        f.state()?["ui_caption"].is_null(),
        "press alone opened the editor"
    );
    f.frame(vec![button(position, false)]);
    let shown = f.warm();
    let state = f.state()?;
    assert!(
        shown.shapes.iter().any(|paint| {
            matches!(&paint.shape, egui::Shape::Text(shape)
                 if shape.galley.text() == "Edit caption"
                 && paint.clip_rect.contains_rect(paint.shape.visual_bounding_rect()))
        }),
        "page point {point:?}, screen {position:?}, density {}, request {}, error {:?}",
        f.dpi,
        state["ui_caption"],
        f.app.error
    );
    let request = state["ui_caption"].clone();
    assert!(request["id"].as_u64().is_some_and(|id| id > 0));
    assert_eq!(request["input_revision"], state["revision"]);
    assert_eq!(request["page_id"], 1);
    assert_eq!(request["text_id"], text_id);
    assert_eq!(request["run_index"], run_index);
    assert_eq!(request["run_count"], run_count);
    assert_eq!(request["text"], text);
    assert_eq!(request["current"], true);
    assert_eq!(field_value(f, "Caption text")?, text);
    Ok(request)
}

fn closed(f: &mut Fixture) -> TestResult {
    f.warm();
    assert!(f.state()?["ui_caption"].is_null());
    assert!(
        f.node("Caption text", &[Role::TextInput, Role::MultilineTextInput])
            .is_err()
    );
    Ok(())
}

fn open_pinned(f: &mut Fixture, path: &Path, format: &str, bytes: &[u8]) -> TestResult {
    fs::write(path, bytes)?;
    f.command(&format!("document.open_{format}"), json!({"path":path}))?;
    f.command("ui.view", json!({"action":"actual"}))?;
    f.warm();
    Ok(())
}

fn current_cdx(f: &Fixture) -> TestResult<Vec<u8>> {
    let engine = f.app.engine.lock().map_err(|_| "engine")?;
    Ok(engine.stored_document()?.export_cdx()?.to_vec())
}

fn choose_file(f: &mut Fixture, path: &Path, action: &str) -> TestResult {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while f.node("File name", &[Role::TextInput]).is_err() {
        f.frame(vec![]);
        if std::time::Instant::now() >= deadline {
            return Err("caption file picker did not open".into());
        }
        std::thread::yield_now();
    }
    set_field(f, "File name", path.to_str().ok_or("Unicode file path")?)?;
    f.choose(action)?;
    finish_actual_file_workflow(f)
}

#[test]
fn mixed_cdx_paints_caption_over_label_and_keeps_normal_chemical_gestures_locked() -> TestResult {
    let raw = include_bytes!("../../io/tests/fixtures/cdx-mixed-caption/overlap_before.cdx");
    let manifest: Value = serde_json::from_str(include_str!(
        "../../io/tests/fixtures/cdx-mixed-caption/cases.json"
    ))?;
    let case = &manifest["overlap_before"];
    for retained_text_tool in [false, true] {
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        let input = f.directory.path("structure-and-caption-原.cdx");
        let output = f.directory.path("structure-and-caption-保存-β.cdx");
        fs::write(&input, raw)?;
        f.warm();
        if retained_text_tool {
            f.choose("Text")?;
        }
        shortcut(&mut f, Key::O, false);
        choose_file(&mut f, &input, "Open")?;
        let opened = f.state()?;
        assert_eq!(opened["document"], case["model"]);
        assert_eq!(opened["file"]["dirty"], false);
        {
            let engine = f.app.engine.lock().map_err(|_| "engine")?;
            assert_eq!(
                engine.effective_tool() == bondline_commands::EffectiveTool::Caption,
                retained_text_tool
            );
            let controls = engine.controls();
            assert!(controls.iter().any(|c| c.id == "tool.caption" && c.enabled));
            for id in ["tool.bond.single", "page.rotate", "file.document_settings"] {
                let control = controls.iter().find(|c| c.id == id).ok_or("tool control")?;
                assert!(!control.enabled, "{id}");
                assert_eq!(control.disabled_reason, Some("unsupported_content"));
            }
        }
        f.frame(super::tap(Key::F5));
        let shown = f.warm();
        assert_eq!(f.view()?["zoom_percent"], 100.0);
        let instruction = if retained_text_tool {
            "Click blank paper to add a caption, or click a caption run to edit its text."
        } else {
            "Choose Text to add or edit a caption."
        };
        for label in [instruction, "1 caption"] {
            assert!(
                shown.shapes.iter().any(|paint| {
                    matches!(&paint.shape, egui::Shape::Text(text) if text.galley.text() == label)
                }),
                "missing inspector label {label}"
            );
        }
        let paper = canvas(&shown)?;
        for sample in case["optical_probes"][0]["samples"]
            .as_array()
            .ok_or("ink probes")?
        {
            let point = [
                sample["xy"][0].as_f64().ok_or("x")? + 0.5,
                sample["xy"][1].as_f64().ok_or("y")? + 0.5,
            ];
            let position = projected(paper, [512.0, 512.0], 100.0, None, point);
            let color = shown
                .shapes
                .iter()
                .rev()
                .find_map(|paint| match &paint.shape {
                    egui::Shape::Rect(rect)
                        if paint.clip_rect.contains(position) && rect.rect.contains(position) =>
                    {
                        Some(rect.fill)
                    }
                    _ => None,
                })
                .ok_or("painted paper pixel")?;
            assert_eq!(
                json!([color.r(), color.g(), color.b()]),
                sample["rgb"],
                "{sample}"
            );
        }
        let blank = paper_point(&mut f, [350.0, 350.0])?;
        let end = blank + egui::vec2(40.0, 0.0);
        f.frame(vec![Event::PointerMoved(blank), button(blank, true)]);
        f.frame(vec![Event::PointerMoved(end)]);
        assert!(
            f.app.drawing.gesture.is_none(),
            "read-only paper started a bond"
        );
        f.frame(vec![button(end, false)]);
        for (point, modifiers) in [
            ([166.5, 122.5], Modifiers::ALT),
            ([159.5, 123.5], Modifiers::NONE),
        ] {
            let position = paper_point(&mut f, point)?;
            click(&mut f, position, modifiers);
            assert_eq!(f.state()?, opened);
        }
        assert!(f.node("Caption text", &[Role::TextInput]).is_err());
        assert!(f.node("Element", &[Role::TextInput]).is_err());
        if retained_text_tool {
            native_caption_creation_tests::open_creation_at(
                &mut f,
                [350.0, 350.0],
                egui::Vec2::ZERO,
            )?;
            f.choose("Cancel")?;
            closed(&mut f)?;
            assert_eq!(f.state()?, opened);
            open_run(&mut f, [166.5, 122.5], 1, 0, 1, "H")?;
            f.choose("Cancel")?;
            closed(&mut f)?;
        } else {
            click(&mut f, blank, Modifiers::NONE);
            assert_eq!(f.state()?, opened);
            let overlap = paper_point(&mut f, [166.5, 122.5])?;
            click(&mut f, overlap, Modifiers::NONE);
        }
        assert_eq!(f.state()?, opened);
        assert!(f.app.error.is_none(), "{:?}", f.app.error);
        f.frame(super::tap(Key::F7));
        assert_eq!(f.view()?["zoom_percent"], 125.0);
        assert_eq!(f.state()?, opened);
        shortcut(&mut f, Key::S, true);
        choose_file(&mut f, &output, "Save")?;
        assert_eq!(fs::read(&output)?, raw);
        assert_eq!(fs::read(&input)?, raw);
        shortcut(&mut f, Key::O, false);
        choose_file(&mut f, &output, "Open")?;
        let reopened = f.state()?;
        assert_eq!(reopened["document"], case["model"]);
        assert_eq!(reopened["undo_depth"], 0);
        assert_eq!(reopened["file"]["dirty"], false);
    }
    Ok(())
}

#[test]
fn mixed_cdx_alt_click_edits_only_exposed_atoms_in_their_own_fragments() -> TestResult {
    let raw = include_bytes!("../../io/tests/fixtures/cdx-chemical-caption-create/mixed_first.cdx");
    let oh = include_bytes!("../../io/tests/fixtures/cdx-mixed-label-edit/mixed_first_oh.cdx");
    let both = include_bytes!("../../io/tests/fixtures/cdx-mixed-label-edit/mixed_first_both.cdx");
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let input = f.directory.path("mixed-labels-原.cdx");
    let output = f.directory.path("mixed-labels-保存-β.cdx");
    open_pinned(&mut f, &input, "cdx", raw)?;
    let initial = f.state()?;

    // Independent blue caption-only and overlap samples both screen the nearby red NH2.
    for point in [[161.5, 124.5], [166.5, 122.5], [350., 350.]] {
        let pos = paper_point(&mut f, point)?;
        click(&mut f, pos, Modifiers::ALT);
        assert_eq!(f.state()?, initial);
        assert!(f.node("Element", &[Role::TextInput]).is_err());
    }
    for (point, id, old, symbol, hydrogens, expected) in [
        ([159.5, 123.5], 1, "N", "O", "1", oh.as_slice()),
        ([350.5, 277.75], 2, "Cl", "Br", "0", both.as_slice()),
    ] {
        let pos = paper_point(&mut f, point)?;
        click(&mut f, pos, Modifiers::ALT);
        let state = f.state()?;
        let draft = &state["ui_atom_label"];
        assert_eq!(draft["fragment_id"], id, "exposed atom at {point:?}");
        assert_eq!(draft["atom_id"], id);
        assert_eq!(draft["current"], true);
        assert!(draft["creation_style"].is_null());
        assert_eq!(field_value(&f, "Element")?, old);
        set_field(&mut f, "Element", symbol)?;
        set_field(&mut f, "Attached hydrogens", hydrogens)?;
        f.choose("Apply")?;
        f.warm();
        assert!(f.state()?["ui_atom_label"].is_null());
        assert_eq!(current_cdx(&f)?, expected);
        assert_eq!(
            f.state()?["document"]["pages"][0]["texts"],
            initial["document"]["pages"][0]["texts"]
        );
    }
    shortcut(&mut f, Key::Z, false);
    assert_eq!(current_cdx(&f)?, oh);
    shortcut(&mut f, Key::Z, false);
    assert_eq!(current_cdx(&f)?, raw);
    assert_eq!(f.state()?["file"]["dirty"], false);
    shortcut(&mut f, Key::Z, true);
    shortcut(&mut f, Key::Z, true);
    assert_eq!(current_cdx(&f)?, both);
    shortcut(&mut f, Key::S, true);
    choose_file(&mut f, &output, "Save")?;
    assert_eq!(fs::read(&input)?, raw);
    assert_eq!(fs::read(&output)?, both);
    shortcut(&mut f, Key::O, false);
    choose_file(&mut f, &output, "Open")?;
    assert_eq!(current_cdx(&f)?, both);
    assert_eq!(f.state()?["undo_depth"], 0);
    assert_eq!(f.state()?["file"]["dirty"], false);
    f.choose("Text")?;
    open_run(&mut f, [166.5, 122.5], 19, 0, 2, "H ")?;
    f.choose("Cancel")?;
    closed(&mut f)?;
    Ok(())
}

#[test]
fn mixed_cdx_text_tool_edits_the_painted_page_run_and_reopens_its_sparse_id() -> TestResult {
    let before = include_bytes!("../../io/tests/fixtures/cdx-mixed-caption-edit/before.cdx");
    let prefix = include_bytes!("../../io/tests/fixtures/cdx-mixed-caption-edit/after_prefix.cdx");
    let both = include_bytes!("../../io/tests/fixtures/cdx-mixed-caption-edit/after_both.cdx");
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let input = f.directory.path("mixed-caption-原.cdx");
    let output = f.directory.path("mixed-caption-保存-β.cdx");
    fs::write(&input, before)?;
    f.warm();
    shortcut(&mut f, Key::O, false);
    choose_file(&mut f, &input, "Open")?;
    f.frame(super::tap(Key::F5));
    f.choose("Text")?;
    let opened = f.state()?;
    let label = paper_point(&mut f, [159.5, 123.5])?;
    click(&mut f, label, Modifiers::NONE);
    closed(&mut f)?;
    assert_eq!(f.state()?, opened);
    native_caption_creation_tests::open_creation_at(&mut f, [350.0, 350.0], egui::Vec2::ZERO)?;
    f.choose("Cancel")?;
    closed(&mut f)?;
    assert_eq!(f.state()?, opened);
    open_run(&mut f, [166.5, 122.5], 19, 0, 2, "H ")?;
    set_field(&mut f, "Caption text", "H H ")?;
    f.choose("Apply")?;
    closed(&mut f)?;
    assert_eq!(current_cdx(&f)?, prefix);
    // FontTools' Roboto advances place the second run's λ at x=205.048828125 pt.
    let tail_ink = [208.5, 126.5];
    open_run(&mut f, tail_ink, 19, 1, 2, "λΩé")?;
    set_field(&mut f, "Caption text", "λéπΩ")?;
    f.choose("Apply")?;
    closed(&mut f)?;
    let edited = f.state()?;
    assert_eq!(edited["undo_depth"], 2);
    assert_eq!(current_cdx(&f)?, both);
    shortcut(&mut f, Key::Z, false);
    assert_eq!(current_cdx(&f)?, prefix);
    shortcut(&mut f, Key::Z, false);
    assert_eq!(current_cdx(&f)?, before);
    assert_eq!(f.state()?["file"]["dirty"], false);
    shortcut(&mut f, Key::Z, true);
    shortcut(&mut f, Key::Z, true);
    assert_eq!(current_cdx(&f)?, both);
    shortcut(&mut f, Key::S, true);
    choose_file(&mut f, &output, "Save")?;
    assert_eq!(fs::read(&input)?, before);
    assert_eq!(fs::read(&output)?, both);
    shortcut(&mut f, Key::O, false);
    choose_file(&mut f, &output, "Open")?;
    let reopened = f.state()?;
    assert_eq!(reopened["document"], edited["document"]);
    assert_eq!(reopened["file"]["dirty"], false);
    assert_eq!(reopened["undo_depth"], 0);
    f.frame(super::tap(Key::F5));
    open_run(&mut f, tail_ink, 19, 1, 2, "λéπΩ")?;
    f.choose("Cancel")?;
    closed(&mut f)?;
    assert_eq!(current_cdx(&f)?, both);
    Ok(())
}

#[test]
fn text_tool_targets_top_painted_caption_and_ordered_runs_at_both_densities() -> TestResult {
    let witnesses: Value = serde_json::from_str(WITNESSES)?;
    let case = &witnesses["cases"]["multi_shift"];
    for density in [1.0, 2.0] {
        let mut f = Fixture::new(1160.0, 800.0, density)?;
        let input = f.directory.path("caption-runs-原.cdx");
        open_pinned(&mut f, &input, "cdx", MULTI)?;
        let initial = f.state()?;
        assert_eq!(initial["document"], case["before_model"]);
        let overlap = paper_point(&mut f, OVERLAP)?;
        click(&mut f, overlap, Modifiers::NONE);
        assert_eq!(f.state()?, initial);
        f.choose("Text")?;
        let original = f.state()?;
        let stamp = f.app.engine.lock().map_err(|_| "engine")?.capture_stamp();

        let background = paper_point(&mut f, [350.0, 350.0])?;
        let spacing = paper_point(&mut f, SPACE_RUN)?;
        click(&mut f, background, Modifiers::NONE);
        let creation = f.state()?["ui_caption"].clone();
        assert!(creation["id"].as_u64().is_some_and(|id| id > 0));
        assert_eq!(creation["input_revision"], original["revision"]);
        assert_eq!(creation["page_id"], 1);
        assert_eq!(creation["position"], json!({"x_pt":350.0,"y_pt":350.0}));
        assert_eq!(creation["text"], "");
        assert_eq!(creation["current"], true);
        assert!(creation.get("text_id").is_none());
        assert_eq!(field_value(&f, "Caption text")?, "");
        f.choose("Cancel")?;
        closed(&mut f)?;
        assert_eq!(f.state()?, original);
        click(&mut f, spacing, Modifiers::NONE);
        click(&mut f, overlap, Modifiers::ALT);
        f.frame(vec![Event::PointerMoved(overlap), button(overlap, true)]);
        f.frame(vec![Event::PointerMoved(background)]);
        f.frame(vec![Event::PointerMoved(overlap), button(overlap, false)]);
        closed(&mut f)?;
        assert_eq!(f.state()?, original);

        open_run(&mut f, OVERLAP, 4, 0, 1, "H")?;
        f.choose("Cancel")?;
        closed(&mut f)?;
        open_run(&mut f, LAST_RUN, 2, 2, 3, "λΩé")?;
        f.choose("Cancel")?;
        closed(&mut f)?;
        open_run(&mut f, FIRST_RUN, 2, 0, 3, "AV")?;
        f.choose("Apply")?;
        closed(&mut f)?;
        assert_eq!(f.state()?, original);
        assert!(
            f.app
                .engine
                .lock()
                .map_err(|_| "engine")?
                .capture_stamp_matches(&stamp)
        );
        assert_eq!(current_cdx(&f)?, MULTI);

        open_run(&mut f, FIRST_RUN, 2, 0, 3, "AV")?;
        set_field(&mut f, "Caption text", "λ")?;
        f.choose("Apply")?;
        f.warm();
        assert_eq!(field_value(&f, "Caption text")?, "λ");
        assert_eq!(f.state()?["revision"], original["revision"]);
        assert_eq!(current_cdx(&f)?, MULTI);
        f.frame(vec![
            key(Key::Escape, true, Modifiers::NONE),
            key(Key::Escape, false, Modifiers::NONE),
        ]);
        closed(&mut f)?;
        assert_eq!(f.state()?, original);

        open_run(&mut f, FIRST_RUN, 2, 0, 3, "AV")?;
        set_field(&mut f, "Caption text", "AVAV")?;
        f.choose("Apply")?;
        closed(&mut f)?;
        let edited = f.state()?;
        assert_eq!(edited["document"], case["after_model"]);
        assert_eq!(edited["undo_depth"], 1);
        assert_eq!(edited["file"]["dirty"], true);
        assert_eq!(current_cdx(&f)?, SHIFTED);
        shortcut(&mut f, Key::Z, false);
        assert_eq!(f.state()?["document"], case["before_model"]);
        assert_eq!(f.state()?["file"]["dirty"], false);
        assert_eq!(current_cdx(&f)?, MULTI);
        shortcut(&mut f, Key::Z, true);
        assert_eq!(f.state()?["document"], case["after_model"]);
        assert_eq!(current_cdx(&f)?, SHIFTED);
        assert_eq!(fs::read(&input)?, MULTI);
    }
    Ok(())
}

#[test]
fn visible_caption_edit_saves_reopens_exact_unicode_cdx_and_cdxml() -> TestResult {
    let witnesses: Value = serde_json::from_str(WITNESSES)?;
    let tail = &witnesses["cases"]["tail_unicode"];
    let edited_xml = CDXML.replace(">Phase A</s>", ">Phase β &amp; &lt;</s>");
    for (format, before, after, point, ids, old, new) in [
        ("cdx", MULTI, TAIL, LAST_RUN, [2, 2, 3], "λΩé", "λéπΩ"),
        (
            "cdxml",
            CDXML.as_bytes(),
            edited_xml.as_bytes(),
            PHASE,
            [1, 0, 1],
            "Phase A",
            "Phase β & <",
        ),
    ] {
        let mut f = Fixture::new(1160.0, 800.0, 2.0)?;
        let input = f.directory.path(&format!("before-原-λ.{format}"));
        let output = f.directory.path(&format!("caption-保存-β.{format}"));
        open_pinned(&mut f, &input, format, before)?;
        f.choose("Text")?;
        let original = f.state()?;
        let [text, run, count] = ids;
        let request = open_run(&mut f, point, text, run as usize, count as usize, old)?;
        f.frame(vec![
            key(Key::Enter, true, Modifiers::NONE),
            key(Key::Enter, false, Modifiers::NONE),
        ]);
        assert_eq!(f.state()?["ui_caption"]["id"], request["id"]);
        assert_eq!(f.state()?["revision"], original["revision"]);
        let (id, _) = f.node("Caption text", &[Role::TextInput, Role::MultilineTextInput])?;
        f.frame(vec![Event::AccessKitActionRequest(ActionRequest {
            action: Action::Focus,
            target: id,
            data: None,
        })]);
        shortcut(&mut f, Key::A, false);
        f.frame(vec![Event::Paste(new.into())]);
        f.warm();
        assert_eq!(field_value(&f, "Caption text")?, new);
        f.choose("Apply")?;
        closed(&mut f)?;
        let edited = f.state()?;
        assert_eq!(edited["file"]["dirty"], true);
        if format == "cdx" {
            assert_eq!(edited["document"], tail["after_model"]);
        }

        shortcut(&mut f, Key::S, true);
        choose_file(&mut f, &output, "Save")?;
        assert_eq!(fs::read(&input)?, before);
        assert_eq!(fs::read(&output)?, after);
        assert_eq!(f.state()?["file"]["dirty"], false);
        shortcut(&mut f, Key::O, false);
        choose_file(&mut f, &output, "Open")?;
        let reopened = f.state()?;
        assert_eq!(reopened["document"], edited["document"]);
        assert_eq!(reopened["undo_depth"], 0);
        assert_eq!(reopened["file"]["dirty"], false);
        assert_eq!(
            reopened["source"],
            json!({"format":format,"byte_length":after.len(),"modified":false})
        );
    }
    Ok(())
}

#[test]
fn caption_pointer_and_draft_refuse_equal_view_and_source_replacements() -> TestResult {
    let original = include_bytes!("../../io/tests/fixtures/cdx-caption/caption_ascii.cdx");
    let distinct = include_bytes!("../../io/tests/fixtures/cdx-caption/caption_ascii_explicit.cdx");
    for source_changes in [false, true] {
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        let a = f.directory.path("same-model-a-原.cdx");
        let b = f.directory.path("same-model-b-λ.cdx");
        fs::write(&b, distinct)?;
        open_pinned(&mut f, &a, "cdx", original)?;
        let replace_context = |f: &mut Fixture| -> TestResult {
            if source_changes {
                f.command("document.open_cdx", json!({"path":b}))?;
                f.command("document.open_cdx", json!({"path":a}))?;
            } else {
                f.command("ui.view", json!({"action":"set_zoom","percent":125}))?;
            }
            f.command("ui.view", json!({"action":"actual"}))?;
            Ok(())
        };
        f.choose("Text")?;
        let point = paper_point(&mut f, BONDLINE)?;
        f.frame(vec![Event::PointerMoved(point), button(point, true)]);
        assert!(f.state()?["ui_caption"].is_null());
        let before_press_change = f.state()?["document"].clone();
        replace_context(&mut f)?;
        assert_eq!(paper_point(&mut f, BONDLINE)?, point);
        f.frame(vec![button(point, false)]);
        closed(&mut f)?;
        assert_eq!(f.state()?["document"], before_press_change);

        f.choose("Text")?;
        let opened = open_run(&mut f, BONDLINE, 1, 0, 1, "Bondline")?;
        let draft = "Draft λ retained";
        set_field(&mut f, "Caption text", draft)?;
        let modal_state = f.state()?;
        let modal_view = f.view()?;
        shortcut(&mut f, Key::O, false);
        f.frame(vec![
            key(Key::F7, true, Modifiers::NONE),
            key(Key::F7, false, Modifiers::NONE),
        ]);
        f.warm();
        assert_eq!(f.state()?, modal_state);
        assert_eq!(f.view()?, modal_view);
        assert_eq!(field_value(&f, "Caption text")?, draft);

        replace_context(&mut f)?;
        f.warm();
        let detached = f.state()?;
        assert_eq!(detached["document"], before_press_change);
        assert_eq!(detached["ui_caption"]["id"], opened["id"]);
        assert_eq!(detached["ui_caption"]["current"], false);
        assert_eq!(field_value(&f, "Caption text")?, draft);
        let button = f
            .nodes
            .iter()
            .find(|(_, node)| {
                node.role() == Role::Button && node.name() == Some("Apply") && node.is_disabled()
            })
            .map(|(id, _)| *id)
            .ok_or("disabled Apply for a detached caption draft")?;
        f.frame(vec![Event::AccessKitActionRequest(ActionRequest {
            action: Action::Default,
            target: button,
            data: None,
        })]);
        let error = f
            .app
            .engine
            .lock()
            .map_err(|_| "engine")?
            .dispatch(
                "ui.caption",
                json!({"action":"apply","expected_revision":detached["revision"],
                       "request_id":opened["id"],"text":draft}),
            )
            .err()
            .ok_or("detached caption request accepted Apply")?;
        assert_eq!(error.info().code, "ui_stale_request");
        assert_eq!(f.state()?, detached);
        assert_eq!(field_value(&f, "Caption text")?, draft);
        assert_eq!(current_cdx(&f)?, original);
        f.choose("Cancel")?;
        closed(&mut f)?;
        assert_eq!(fs::read(&a)?, original);
        assert_eq!(fs::read(&b)?, distinct);
    }
    Ok(())
}

#[test]
fn text_mode_does_not_author_bonds_or_open_on_chemical_hits() -> TestResult {
    use bondline_commands::Engine;
    use bondline_model::{Point2, StyledText, TextId, TextRun, TextStyle};

    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    f.command(
        "page.set_size",
        json!({"page_id":1,"width_pt":512.0,"height_pt":512.0}),
    )?;
    f.command(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":240.0,"begin_y_pt":300.0,
               "end_x_pt":280.0,"end_y_pt":300.0}),
    )?;
    {
        let mut engine = f.app.engine.lock().map_err(|_| "engine")?;
        let mut stored = engine.stored_document()?.clone();
        let page_id = stored.model().pages().first().ok_or("paper")?.id();
        let caption = StyledText::new(
            TextId::new(1).ok_or("caption id")?,
            Point2::new(96.25, 100.75)?,
            vec![TextRun::new(
                "H".into(),
                TextStyle::new("Roboto".into(), 12.0, 0, [0.0; 3])?,
            )?],
        )?;
        stored.model_mut().append_texts(page_id, vec![caption])?;
        *engine = Engine::from_document(stored);
        engine.set_ui_file_host(true);
    }
    f.command("ui.view", json!({"action":"actual"}))?;
    f.choose("Text")?;
    let original = f.state()?;
    // The open area inside the H outline envelope remains owned by its existing caption.
    for point in [[240.0, 300.0], [260.0, 300.0], [100.5, 93.5]] {
        let hit = paper_point(&mut f, point)?;
        click(&mut f, hit, Modifiers::NONE);
        closed(&mut f)?;
        assert_eq!(f.state()?, original);
    }
    open_run(&mut f, OVERLAP, 1, 0, 1, "H")?;
    f.choose("Cancel")?;
    closed(&mut f)?;
    assert_eq!(f.state()?, original);
    Ok(())
}

#[test]
fn plain_t_activates_text_only_when_the_draft_does_not_own_typing() -> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let input = f.directory.path("shortcut-runs.cdx");
    open_pinned(&mut f, &input, "cdx", MULTI)?;
    f.frame(vec![
        key(Key::T, true, Modifiers::NONE),
        Event::Text("t".into()),
        key(Key::T, false, Modifiers::NONE),
    ]);
    let request = open_run(&mut f, FIRST_RUN, 2, 0, 3, "AV")?;
    let (id, _) = f.node("Caption text", &[Role::TextInput])?;
    f.frame(vec![Event::AccessKitActionRequest(ActionRequest {
        action: Action::Focus,
        target: id,
        data: None,
    })]);
    shortcut(&mut f, Key::A, false);
    f.frame(vec![
        key(Key::T, true, Modifiers::NONE),
        Event::Text("t".into()),
        key(Key::T, false, Modifiers::NONE),
    ]);
    f.warm();
    assert_eq!(field_value(&f, "Caption text")?, "t");
    assert_eq!(f.state()?["ui_caption"], request);
    assert_eq!(current_cdx(&f)?, MULTI);
    f.choose("Cancel")?;
    closed(&mut f)
}

#[test]
fn apply_consumes_only_the_ordered_field_input_before_its_activation() -> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let input = f.directory.path("prefixed-runs.cdx");
    open_pinned(&mut f, &input, "cdx", MULTI)?;
    f.choose("Text")?;
    open_run(&mut f, FIRST_RUN, 2, 0, 3, "AV")?;
    let (field, _) = f.node("Caption text", &[Role::TextInput])?;
    let (apply, _) = f.node("Apply", &[Role::Button])?;
    let set = |value: &str| {
        Event::AccessKitActionRequest(ActionRequest {
            action: Action::SetValue,
            target: field,
            data: Some(ActionData::Value(value.into())),
        })
    };
    f.frame(vec![
        set("AVAV"),
        Event::AccessKitActionRequest(ActionRequest {
            action: Action::Default,
            target: apply,
            data: None,
        }),
        set("λ"),
    ]);
    closed(&mut f)?;
    assert_eq!(current_cdx(&f)?, SHIFTED);
    assert_eq!(f.state()?["undo_depth"], 1);
    assert_eq!(fs::read(&input)?, MULTI);
    Ok(())
}

#[test]
fn rejected_apply_retains_a_bounded_later_replacement_for_explicit_retry() -> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let input = f.directory.path("retry-runs.cdx");
    open_pinned(&mut f, &input, "cdx", MULTI)?;
    f.choose("Text")?;
    let request = open_run(&mut f, FIRST_RUN, 2, 0, 3, "AV")?;
    let (field, _) = f.node("Caption text", &[Role::TextInput])?;
    let (apply, _) = f.node("Apply", &[Role::Button])?;
    let set = |value: &str| {
        Event::AccessKitActionRequest(ActionRequest {
            action: Action::SetValue,
            target: field,
            data: Some(ActionData::Value(value.into())),
        })
    };
    f.frame(vec![
        set("λ"),
        Event::AccessKitActionRequest(ActionRequest {
            action: Action::Default,
            target: apply,
            data: None,
        }),
        set("AVAV"),
    ]);
    f.warm();
    assert_eq!(f.state()?["ui_caption"], request);
    assert_eq!(f.state()?["undo_depth"], 0);
    assert_eq!(current_cdx(&f)?, MULTI);
    assert_eq!(field_value(&f, "Caption text")?, "AVAV");
    f.choose("Apply")?;
    closed(&mut f)?;
    assert_eq!(current_cdx(&f)?, SHIFTED);
    Ok(())
}

#[test]
fn text_tool_uses_the_shared_wheel_camera() -> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let input = f.directory.path("wheel-runs.cdx");
    open_pinned(&mut f, &input, "cdx", MULTI)?;
    f.choose("Text")?;
    let before = f.state()?;
    let pos = paper_point(&mut f, [260.0, 256.0])?;
    let epoch = f.view()?["epoch"].as_u64().ok_or("view epoch")?;
    f.frame(vec![
        Event::PointerMoved(pos),
        super::wheel([0.0, 40.0], Modifiers::NONE),
    ]);
    let view = f.view()?;
    assert_eq!(view["zoom_percent"], 100.0);
    assert_eq!(
        (view["center_x_pt"].clone(), view["center_y_pt"].clone()),
        (json!(256.0), json!(216.0))
    );
    assert_eq!(view["epoch"], epoch + 1);
    assert_eq!(f.state()?, before);
    Ok(())
}
