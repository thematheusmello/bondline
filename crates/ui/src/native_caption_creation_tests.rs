//! Actual App frames against independent native caption-creation witnesses.
use super::{
    CDXML, FIRST_RUN, Fixture, MULTI, PHASE, TestResult, button, choose_file, click, closed,
    current_cdx, field_value, open_pinned, open_run, paper_point, set_field, shortcut,
};
use bondline_commands::Engine;
use bondline_model::PageSize;
use eframe::egui::{self, Event, Key, Modifiers, accesskit::Role};
use serde_json::{Value, json};
use std::fs;

const BLANK: &[u8] = include_bytes!("../../io/tests/fixtures/caption-create/blank.cdxml");
const FIRST: &[u8] = include_bytes!("../../io/tests/fixtures/caption-create/first.cdxml");
const AFTER_UNDO: &[u8] = include_bytes!("../../io/tests/fixtures/caption-create/after_undo.cdxml");
const MIXED: &[u8] = include_bytes!("../../io/tests/fixtures/caption-create/mixed_before.cdxml");
const MIXED_AFTER: &[u8] =
    include_bytes!("../../io/tests/fixtures/caption-create/mixed_after.cdxml");
const NONCANONICAL: &[u8] =
    include_bytes!("../../io/tests/fixtures/caption-create/mixed_noncanonical.cdxml");
const WITNESSES: &str = include_str!("../../io/tests/fixtures/caption-create/cases.json");
const CDX_BEFORE: &[u8] =
    include_bytes!("../../io/tests/fixtures/cdx-caption-edit/single_before.cdx");
const CDX_FIRST: &[u8] = include_bytes!("../../io/tests/fixtures/cdx-caption-create/first.cdx");
const CDX_AFTER_UNDO: &[u8] =
    include_bytes!("../../io/tests/fixtures/cdx-caption-create/after_undo.cdx");
const CDX_WITNESSES: &str = include_str!("../../io/tests/fixtures/cdx-caption-edit/cases.json");
const BASELINE: [f64; 2] = [144.25, 216.75];
const BODY: &str = "Phase β & q";
// FontTools interior probes from the independent fixture generator.
const ORIGINAL_INK: [f64; 2] = [85.0, 119.25];
const NEW_INK: [f64; 2] = [148.25, 212.75];
// Each FontTools probe and its eight +/-0.4 pt neighbours lie inside this glyph.
const CDX_OLD_INK: [f64; 2] = [81.75, 95.75];
const CDX_NEW_INK: [f64; 2] = [145.75, 211.75];

fn blank_source_free(f: &mut Fixture) -> TestResult {
    let mut engine = f.app.engine.lock().map_err(|_| "engine")?;
    let mut document = engine.stored_document()?.clone();
    let page = document.model().pages().first().ok_or("paper")?.id();
    document
        .model_mut()
        .set_page_size(page, PageSize::new(512.0, 512.0)?)?;
    *engine = Engine::from_document(document);
    engine.set_ui_file_host(true);
    Ok(())
}

fn current_xml(f: &Fixture) -> TestResult<Vec<u8>> {
    let engine = f.app.engine.lock().map_err(|_| "engine")?;
    Ok(engine.stored_document()?.to_cdxml()?.into_owned())
}

fn open_creation(f: &mut Fixture, release_offset: egui::Vec2) -> TestResult {
    open_creation_at(f, BASELINE, release_offset)
}

fn open_creation_at(f: &mut Fixture, baseline: [f64; 2], release_offset: egui::Vec2) -> TestResult {
    let press = paper_point(f, baseline)?;
    f.frame(vec![Event::PointerMoved(press), button(press, true)]);
    assert!(f.state()?["ui_caption"].is_null());
    let release = press + release_offset;
    f.frame(vec![Event::PointerMoved(release), button(release, false)]);
    let shown = f.warm();
    assert!(shown.shapes.iter().any(|paint| {
        matches!(&paint.shape, egui::Shape::Text(shape)
                 if shape.galley.text() == "New caption"
                 && paint.clip_rect.contains_rect(paint.shape.visual_bounding_rect()))
    }));
    let state = f.state()?;
    let request = &state["ui_caption"];
    let id = request["id"].as_u64().ok_or("new caption request ID")?;
    assert!(id > 0);
    assert_eq!(
        request,
        &json!({"id":id,"input_revision":state["revision"],"page_id":1,
                "position":{"x_pt":baseline[0],"y_pt":baseline[1]},"text":"","current":true})
    );
    assert_eq!(field_value(f, "Caption text")?, "");
    Ok(())
}

fn apply_body(f: &mut Fixture, body: &str) -> TestResult {
    set_field(f, "Caption text", body)?;
    f.choose("Apply")?;
    closed(f)
}

fn cdx_created_model(id: u32) -> TestResult<Value> {
    let cdx: Value = serde_json::from_str(CDX_WITNESSES)?;
    let xml: Value = serde_json::from_str(WITNESSES)?;
    let mut expected = cdx["cases"]["single_grow"]["before_model"].clone();
    let mut caption = xml["cases"]["first"]["after_model"]["pages"][0]["texts"][0].clone();
    caption["id"] = json!(id);
    expected["pages"][0]["texts"]
        .as_array_mut()
        .ok_or("independent existing captions")?
        .push(caption);
    Ok(expected)
}

#[test]
fn empty_cdx_paper_creates_its_first_caption_and_reopens_the_sparse_undo_branch() -> TestResult {
    const BEFORE: &[u8] =
        include_bytes!("../../io/tests/fixtures/cdx-caption-empty/implicit_before.cdx");
    const FIRST: &[u8] =
        include_bytes!("../../io/tests/fixtures/cdx-caption-empty/implicit_first.cdx");
    const AFTER_UNDO: &[u8] =
        include_bytes!("../../io/tests/fixtures/cdx-caption-empty/implicit_after_undo.cdx");
    let witnesses: Value = serde_json::from_str(include_str!(
        "../../io/tests/fixtures/cdx-caption-empty/cases.json"
    ))?;
    let case = &witnesses["cases"]["implicit"];
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let input = f.directory.path("empty-caption-paper-原.cdx");
    let output = f.directory.path("first-caption-保存-β.cdx");
    fs::write(&input, BEFORE)?;
    shortcut(&mut f, Key::O, false);
    choose_file(&mut f, &input, "Open")?;
    let imported = f.state()?;
    assert_eq!(imported["document"], case["before"]["model"]);
    assert_eq!(
        imported["document"]["pages"][0]["size"],
        json!({"width_pt":512.25,"height_pt":384.5})
    );
    assert!(imported["document"]["pages"][0].get("texts").is_none());
    assert_eq!(imported["file"]["dirty"], false);
    assert_eq!(current_cdx(&f)?, BEFORE);
    f.command("ui.view", json!({"action":"actual"}))?;
    f.choose("Text")?;
    let original = f.state()?;

    open_creation(&mut f, egui::vec2(1.0, 1.0))?;
    apply_body(&mut f, BODY)?;
    assert_eq!(f.state()?["document"], case["first"]["model"]);
    assert_eq!(f.state()?["undo_depth"], 1);
    assert_eq!(f.state()?["file"]["dirty"], true);
    assert_eq!(current_cdx(&f)?, FIRST);
    shortcut(&mut f, Key::Z, false);
    assert_eq!(f.state()?["document"], original["document"]);
    assert_eq!(f.state()?["file"]["dirty"], false);
    assert_eq!(current_cdx(&f)?, BEFORE);
    shortcut(&mut f, Key::Z, true);
    assert_eq!(f.state()?["document"], case["first"]["model"]);
    assert_eq!(current_cdx(&f)?, FIRST);
    shortcut(&mut f, Key::Z, false);

    open_creation(&mut f, egui::Vec2::ZERO)?;
    apply_body(&mut f, BODY)?;
    let branch = f.state()?;
    assert_eq!(branch["document"], case["after_undo"]["model"]);
    assert_eq!(branch["undo_depth"], 1);
    assert_eq!(branch["redo_depth"], 0);
    assert_eq!(current_cdx(&f)?, AFTER_UNDO);
    shortcut(&mut f, Key::S, true);
    choose_file(&mut f, &output, "Save")?;
    assert_eq!(fs::read(&input)?, BEFORE);
    assert_eq!(fs::read(&output)?, AFTER_UNDO);
    assert_eq!(f.state()?["file"]["dirty"], false);
    shortcut(&mut f, Key::O, false);
    choose_file(&mut f, &output, "Open")?;
    let reopened = f.state()?;
    assert_eq!(reopened["document"], case["after_undo"]["model"]);
    assert_eq!(reopened["undo_depth"], 0);
    assert_eq!(reopened["file"]["dirty"], false);
    assert_eq!(
        reopened["source"],
        json!({"format":"cdx","byte_length":AFTER_UNDO.len(),"modified":false})
    );
    assert_eq!(current_cdx(&f)?, AFTER_UNDO);
    f.command("ui.view", json!({"action":"actual"}))?;
    f.choose("Text")?;
    open_run(&mut f, CDX_NEW_INK, 2, 0, 1, BODY)?;
    f.choose("Cancel")?;
    closed(&mut f)?;
    assert_eq!(f.state()?["document"], case["after_undo"]["model"]);
    assert_eq!(f.state()?["undo_depth"], 0);
    assert_eq!(f.state()?["file"]["dirty"], false);
    assert_eq!(current_cdx(&f)?, AFTER_UNDO);
    Ok(())
}

#[test]
fn cdx_paper_creation_keeps_sparse_history_and_unicode_save_reopens_both_editable_captions()
-> TestResult {
    let witnesses: Value = serde_json::from_str(CDX_WITNESSES)?;
    for dpi in [1.0, 2.0] {
        let mut f = Fixture::new(1160.0, 800.0, dpi)?;
        let input = f.directory.path("existing-caption-原.cdx");
        let output = f.directory.path("new-caption-保存-β.cdx");
        open_pinned(&mut f, &input, "cdx", CDX_BEFORE)?;
        assert!(f.warm().shapes.iter().any(|paint| {
            matches!(&paint.shape, egui::Shape::Text(shape)
                if shape.galley.text() == "Choose Text to add or edit a caption.")
        }));
        f.choose("Text")?;
        let original = f.state()?;
        assert_eq!(
            original["document"],
            witnesses["cases"]["single_grow"]["before_model"]
        );
        assert_eq!(original["undo_depth"], 0);
        assert_eq!(
            original["source"],
            json!({"format":"cdx","byte_length":CDX_BEFORE.len(),"modified":false})
        );
        let stamp = f.app.engine.lock().map_err(|_| "engine")?.capture_stamp();

        open_creation(&mut f, egui::vec2(1.0, 1.0))?;
        set_field(&mut f, "Caption text", "Discard this CDX draft")?;
        f.choose("Cancel")?;
        closed(&mut f)?;
        assert_eq!(f.state()?, original);
        assert_eq!(current_cdx(&f)?, CDX_BEFORE);
        assert!(
            f.app
                .engine
                .lock()
                .map_err(|_| "engine")?
                .capture_stamp_matches(&stamp)
        );

        open_creation(&mut f, egui::Vec2::ZERO)?;
        set_field(&mut f, "Caption text", &"W".repeat(80))?;
        f.choose("Apply")?;
        f.warm();
        assert_eq!(field_value(&f, "Caption text")?, "W".repeat(80));
        assert_eq!(f.state()?["revision"], original["revision"]);
        assert_eq!(f.state()?["undo_depth"], 0);
        assert_eq!(f.state()?["document"], original["document"]);
        assert_eq!(current_cdx(&f)?, CDX_BEFORE);
        f.choose("Cancel")?;
        closed(&mut f)?;
        assert_eq!(f.state()?, original);

        open_creation(&mut f, egui::vec2(1.0, 1.0))?;
        apply_body(&mut f, BODY)?;
        assert_eq!(f.state()?["document"], cdx_created_model(2)?);
        assert_eq!(f.state()?["undo_depth"], 1);
        assert_eq!(f.state()?["file"]["dirty"], true);
        assert_eq!(current_cdx(&f)?, CDX_FIRST);
        shortcut(&mut f, Key::Z, false);
        assert_eq!(f.state()?["document"], original["document"]);
        assert_eq!(f.state()?["file"]["dirty"], false);
        assert_eq!(current_cdx(&f)?, CDX_BEFORE);
        shortcut(&mut f, Key::Z, true);
        assert_eq!(f.state()?["document"], cdx_created_model(2)?);
        assert_eq!(current_cdx(&f)?, CDX_FIRST);
        shortcut(&mut f, Key::Z, false);
        open_creation(&mut f, egui::Vec2::ZERO)?;
        apply_body(&mut f, BODY)?;
        let second = f.state()?;
        assert_eq!(second["document"], cdx_created_model(3)?);
        assert_eq!(second["redo_depth"], 0);
        assert_eq!(current_cdx(&f)?, CDX_AFTER_UNDO);

        shortcut(&mut f, Key::S, true);
        assert!(f.warm().shapes.iter().any(|paint| {
            matches!(&paint.shape, egui::Shape::Text(shape)
                if shape.galley.text() == "CDX files (*.cdx). Saves the imported source with supported caption and atom label edits.")
        }));
        choose_file(&mut f, &output, "Save")?;
        assert_eq!(fs::read(&input)?, CDX_BEFORE);
        assert_eq!(fs::read(&output)?, CDX_AFTER_UNDO);
        assert_eq!(f.state()?["file"]["dirty"], false);
        shortcut(&mut f, Key::O, false);
        choose_file(&mut f, &output, "Open")?;
        let reopened = f.state()?;
        assert_eq!(reopened["document"], second["document"]);
        assert_eq!(reopened["undo_depth"], 0);
        assert_eq!(
            reopened["source"],
            json!({"format":"cdx","byte_length":CDX_AFTER_UNDO.len(),"modified":false})
        );
        assert_eq!(current_cdx(&f)?, CDX_AFTER_UNDO);
        f.command("ui.view", json!({"action":"actual"}))?;
        f.choose("Text")?;

        for (point, id, old, new) in [
            (CDX_NEW_INK, 3, BODY, "Phase β & <"),
            (CDX_OLD_INK, 1, "Bondline", "Bondline λ notes"),
        ] {
            open_run(&mut f, point, id, 0, 1, old)?;
            apply_body(&mut f, new)?;
            let mut expected = second["document"].clone();
            let caption = expected["pages"][0]["texts"]
                .as_array_mut()
                .and_then(|texts| texts.iter_mut().find(|text| text["id"] == id))
                .ok_or("independent edited caption ID")?;
            caption["runs"][0]["text"] = json!(new);
            assert_eq!(f.state()?["document"], expected);
            assert_eq!(f.state()?["undo_depth"], 1);
            assert_eq!(f.state()?["file"]["dirty"], true);
            shortcut(&mut f, Key::Z, false);
            assert_eq!(f.state()?["document"], second["document"]);
            assert_eq!(f.state()?["file"]["dirty"], false);
            assert_eq!(current_cdx(&f)?, CDX_AFTER_UNDO);
        }
    }
    Ok(())
}

#[test]
fn cdx_creation_press_and_visible_draft_refuse_equal_view_restoration() -> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let input = f.directory.path("creation-weak-owner-原.cdx");
    open_pinned(&mut f, &input, "cdx", CDX_BEFORE)?;
    f.choose("Text")?;
    let original = f.state()?;
    let press = paper_point(&mut f, BASELINE)?;
    f.frame(vec![Event::PointerMoved(press), button(press, true)]);
    f.command("ui.view", json!({"action":"set_zoom","percent":125}))?;
    f.command("ui.view", json!({"action":"actual"}))?;
    assert_eq!(paper_point(&mut f, BASELINE)?, press);
    f.frame(vec![button(press, false)]);
    closed(&mut f)?;
    assert_eq!(f.state()?["document"], original["document"]);
    assert_eq!(current_cdx(&f)?, CDX_BEFORE);

    open_creation(&mut f, egui::Vec2::ZERO)?;
    set_field(&mut f, "Caption text", BODY)?;
    let request = f.state()?["ui_caption"].clone();
    f.command("ui.view", json!({"action":"set_zoom","percent":125}))?;
    f.command("ui.view", json!({"action":"actual"}))?;
    f.warm();
    let detached = f.state()?;
    assert_eq!(detached["ui_caption"]["id"], request["id"]);
    assert_eq!(detached["ui_caption"]["current"], false);
    assert_eq!(field_value(&f, "Caption text")?, BODY);
    assert!(f.nodes.values().any(|node| node.role() == Role::Button
        && node.name() == Some("Apply")
        && node.is_disabled()));
    let error = f
        .app
        .engine
        .lock()
        .map_err(|_| "engine")?
        .dispatch(
            "ui.caption",
            json!({"action":"apply","expected_revision":detached["revision"],
               "request_id":request["id"],"text":BODY}),
        )
        .err()
        .ok_or("detached CDX creation request accepted")?;
    assert_eq!(error.info().code, "ui_stale_request");
    assert_eq!(f.state()?, detached);
    assert_eq!(current_cdx(&f)?, CDX_BEFORE);
    f.choose("Cancel")?;
    closed(&mut f)?;
    assert_eq!(f.state()?["document"], original["document"]);
    assert_eq!(f.state()?["undo_depth"], 0);
    assert_eq!(fs::read(&input)?, CDX_BEFORE);
    Ok(())
}

#[test]
fn blank_paper_click_uses_the_press_baseline_and_saves_sparse_caption_after_undo() -> TestResult {
    let witnesses: Value = serde_json::from_str(WITNESSES)?;
    let cases = &witnesses["cases"];
    for dpi in [1.0, 2.0] {
        let mut f = Fixture::new(1160.0, 800.0, dpi)?;
        blank_source_free(&mut f)?;
        f.command("ui.view", json!({"action":"actual"}))?;
        f.choose("Text")?;
        let original = f.state()?;
        assert!(original["source"].is_null());
        assert_eq!(original["document"], cases["first"]["before_model"]);
        assert_eq!(current_xml(&f)?, BLANK);
        let stamp = f.app.engine.lock().map_err(|_| "engine")?.capture_stamp();

        let start = paper_point(&mut f, BASELINE)?;
        let out_of_paper = paper_point(&mut f, [-12.0, BASELINE[1]])?;
        click(&mut f, out_of_paper, Modifiers::NONE);
        click(&mut f, start, Modifiers::ALT);
        f.frame(vec![Event::PointerMoved(start), button(start, true)]);
        f.frame(vec![Event::PointerMoved(start + egui::vec2(12.0, 0.0))]);
        f.frame(vec![Event::PointerMoved(start), button(start, false)]);
        closed(&mut f)?;
        assert_eq!(f.state()?, original);

        open_creation(&mut f, egui::vec2(1.0, 1.0))?;
        set_field(&mut f, "Caption text", "Discard this draft")?;
        f.choose("Cancel")?;
        closed(&mut f)?;
        assert_eq!(f.state()?, original);
        assert!(
            f.app
                .engine
                .lock()
                .map_err(|_| "engine")?
                .capture_stamp_matches(&stamp)
        );

        open_creation(&mut f, egui::vec2(1.0, 1.0))?;
        apply_body(&mut f, BODY)?;
        let first = f.state()?;
        assert_eq!(first["document"], cases["first"]["after_model"]);
        assert_eq!(first["undo_depth"], 1);
        assert_eq!(first["file"]["dirty"], true);
        assert_eq!(current_xml(&f)?, FIRST);
        shortcut(&mut f, Key::Z, false);
        assert_eq!(f.state()?["document"], original["document"]);
        assert_eq!(f.state()?["file"]["dirty"], false);
        assert_eq!(current_xml(&f)?, BLANK);
        shortcut(&mut f, Key::Z, true);
        assert_eq!(f.state()?["document"], first["document"]);
        assert_eq!(current_xml(&f)?, FIRST);
        shortcut(&mut f, Key::Z, false);
        f.command("ui.view", json!({"action":"actual"}))?;
        open_creation(&mut f, egui::Vec2::ZERO)?;
        apply_body(&mut f, BODY)?;
        let second = f.state()?;
        assert_eq!(second["document"], cases["after_undo"]["after_model"]);
        assert_eq!(second["redo_depth"], 0);
        assert_eq!(current_xml(&f)?, AFTER_UNDO);

        let output = f.directory.path("new-caption-保存-β.cdxml");
        shortcut(&mut f, Key::S, true);
        choose_file(&mut f, &output, "Save")?;
        assert_eq!(fs::read(&output)?, AFTER_UNDO);
        assert_eq!(f.state()?["file"]["dirty"], false);
        shortcut(&mut f, Key::O, false);
        choose_file(&mut f, &output, "Open")?;
        let reopened = f.state()?;
        assert_eq!(reopened["document"], second["document"]);
        assert_eq!(reopened["undo_depth"], 0);
        assert_eq!(reopened["file"]["dirty"], false);
        f.command("ui.view", json!({"action":"actual"}))?;
        f.choose("Text")?;
        open_run(&mut f, NEW_INK, 2, 0, 1, BODY)?;
        f.choose("Cancel")?;
        closed(&mut f)?;
    }
    Ok(())
}

#[test]
fn exact_native_caption_append_keeps_the_old_graph_runs_and_resources() -> TestResult {
    let witnesses: Value = serde_json::from_str(WITNESSES)?;
    let mixed = &witnesses["cases"]["mixed_append"];
    let mut f = Fixture::new(1160.0, 800.0, 2.0)?;
    let input = f.directory.path("styled-original-原.cdxml");
    let output = f.directory.path("styled-with-caption-保存-λ.cdxml");
    open_pinned(&mut f, &input, "cdxml", MIXED)?;
    f.choose("Text")?;
    let original = f.state()?;
    assert_eq!(original["document"], mixed["before_model"]);
    let old = &mixed["before_model"]["pages"][0]["fragments"][0]["atoms"];
    let position = |index: usize| -> TestResult<[f64; 2]> {
        Ok([
            old[index]["position"]["x_pt"].as_f64().ok_or("atom x")?,
            old[index]["position"]["y_pt"].as_f64().ok_or("atom y")?,
        ])
    };
    let a = position(0)?;
    let b = position(1)?;
    let bond = [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0];
    for target in [a, bond] {
        let hit = paper_point(&mut f, target)?;
        click(&mut f, hit, Modifiers::NONE);
        closed(&mut f)?;
        assert_eq!(f.state()?, original);
    }
    open_run(&mut f, ORIGINAL_INK, 1, 0, 2, "Sample ")?;
    f.choose("Cancel")?;
    closed(&mut f)?;
    assert_eq!(f.state()?, original);

    open_creation(&mut f, egui::Vec2::ZERO)?;
    apply_body(&mut f, BODY)?;
    let edited = f.state()?;
    assert_eq!(edited["document"], mixed["after_model"]);
    assert_eq!(edited["undo_depth"], 1);
    assert_eq!(edited["file"]["dirty"], true);
    assert_eq!(
        edited["source"],
        json!({"format":"cdxml","byte_length":MIXED.len(),"modified":true})
    );
    assert_eq!(current_xml(&f)?, MIXED_AFTER);
    shortcut(&mut f, Key::S, true);
    choose_file(&mut f, &output, "Save")?;
    assert_eq!(fs::read(&input)?, MIXED);
    assert_eq!(fs::read(&output)?, MIXED_AFTER);
    assert_eq!(f.state()?["file"]["dirty"], false);
    shortcut(&mut f, Key::O, false);
    choose_file(&mut f, &output, "Open")?;
    let reopened = f.state()?;
    assert_eq!(reopened["document"], mixed["after_model"]);
    assert_eq!(
        reopened["source"],
        json!({"format":"cdxml","byte_length":MIXED_AFTER.len(),"modified":false})
    );
    assert_eq!(reopened["undo_depth"], 0);
    Ok(())
}

fn refuse_new_caption(f: &mut Fixture, blank: [f64; 2]) -> TestResult {
    let original = f.state()?;
    let position = paper_point(f, blank)?;
    click(f, position, Modifiers::NONE);
    closed(f)?;
    assert_eq!(f.state()?, original);
    refuse_open_new(f, blank, &original)
}

fn refuse_open_new(f: &Fixture, blank: [f64; 2], original: &Value) -> TestResult {
    let error = f
        .app
        .engine
        .lock()
        .map_err(|_| "engine")?
        .dispatch(
            "ui.caption",
            json!({"action":"open_new","expected_revision":original["revision"],"page_id":1,
                   "x_pt":blank[0],"y_pt":blank[1]}),
        )
        .err()
        .ok_or("unqualified caption creation request accepted")?;
    assert_eq!(error.info().code, "unsupported_content");
    assert_eq!(&f.state()?, original);
    Ok(())
}

#[test]
fn imported_caption_sources_keep_cdx_creation_and_foreign_cdxml_edit_only_authority() -> TestResult
{
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let cdx = f.directory.path("imported-caption.cdx");
    open_pinned(&mut f, &cdx, "cdx", MULTI)?;
    f.choose("Text")?;
    let imported = f.state()?;
    open_creation_at(&mut f, [350.0, 350.0], egui::Vec2::ZERO)?;
    f.choose("Cancel")?;
    closed(&mut f)?;
    assert_eq!(f.state()?, imported);
    open_run(&mut f, FIRST_RUN, 2, 0, 3, "AV")?;
    f.choose("Cancel")?;
    closed(&mut f)?;
    assert_eq!(current_cdx(&f)?, MULTI);

    let foreign = f.directory.path("foreign-caption.cdxml");
    open_pinned(&mut f, &foreign, "cdxml", CDXML.as_bytes())?;
    f.choose("Text")?;
    refuse_new_caption(&mut f, BASELINE)?;
    open_run(&mut f, PHASE, 1, 0, 1, "Phase A")?;
    f.choose("Cancel")?;
    closed(&mut f)?;
    assert_eq!(current_xml(&f)?, CDXML.as_bytes());

    let native = f.directory.path("mixed-native.cdxml");
    let noncanonical = f.directory.path("mixed-opaque.cdxml");
    open_pinned(&mut f, &native, "cdxml", MIXED)?;
    f.choose("Text")?;
    let before = f.state()?["document"].clone();
    let prior_paper = paper_point(&mut f, BASELINE)?;
    fs::write(&noncanonical, NONCANONICAL)?;
    f.command("document.open_cdxml", json!({"path":noncanonical}))?;
    assert!(f.command("ui.view", json!({"action":"actual"})).is_err());
    f.warm();
    assert_eq!(f.state()?["document"], before);
    let retained = f.state()?;
    click(&mut f, prior_paper, Modifiers::NONE);
    closed(&mut f)?;
    assert_eq!(f.state()?, retained);
    refuse_open_new(&f, BASELINE, &retained)?;
    assert_eq!(current_xml(&f)?, NONCANONICAL);
    Ok(())
}

#[test]
fn external_caption_apply_retires_the_presented_pristine_form_and_allows_close() -> TestResult {
    for (body, local_matches_apply) in [("AV", false), ("AVAV", false), ("AVAV", true)] {
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        let input = f.directory.path("external-caption.cdx");
        open_pinned(&mut f, &input, "cdx", MULTI)?;
        f.choose("Text")?;
        let request = open_run(&mut f, FIRST_RUN, 2, 0, 3, "AV")?;
        if local_matches_apply {
            set_field(&mut f, "Caption text", body)?;
        }
        let result = f.command(
            "ui.caption",
            json!({"action":"apply",
            "expected_revision":request["input_revision"],"request_id":request["id"],"text":body}),
        )?;
        assert_eq!(result["changed"], body != "AV");
        assert!(result["ui_caption"].is_null());
        // Close can arrive before the first repaint acknowledging the external Apply.
        let close = f.command(
            "document.close",
            json!({"expected_revision":result["revision"]}),
        )?;
        if body == "AVAV" {
            assert_eq!(close["close"], "needs_decision");
            f.command(
                "document.close",
                json!({"action":"discard","expected_revision":result["revision"]}),
            )?;
        } else {
            assert_eq!(close["close"], "approved");
        }
        closed(&mut f)?;
        assert!(!f.app.drawing.authoring_modal());
        assert_eq!(f.state()?["close"], "committed");
    }
    Ok(())
}

#[test]
fn external_caption_apply_keeps_local_unsent_text_for_explicit_cancel() -> TestResult {
    for input_on_next_frame in [false, true] {
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        let input = f.directory.path("divergent-caption.cdx");
        open_pinned(&mut f, &input, "cdx", MULTI)?;
        f.choose("Text")?;
        let request = open_run(&mut f, FIRST_RUN, 2, 0, 3, "AV")?;
        if !input_on_next_frame {
            set_field(&mut f, "Caption text", "my unsent text")?;
        }
        let result = f.command(
            "ui.caption",
            json!({"action":"apply",
        "expected_revision":request["input_revision"],"request_id":request["id"],"text":"AVAV"}),
        )?;
        assert!(result["ui_caption"].is_null());
        if input_on_next_frame {
            set_field(&mut f, "Caption text", "my unsent text")?;
        } else {
            f.warm();
        }
        assert_eq!(field_value(&f, "Caption text")?, "my unsent text");
        assert!(f.app.drawing.authoring_modal());
        // Returning to the original visible value does not implicitly dismiss a detached draft.
        set_field(&mut f, "Caption text", "AV")?;
        f.choose("Cancel")?;
        closed(&mut f)?;
        assert_eq!(f.state()?["revision"], result["revision"]);
    }
    Ok(())
}
