//! Actual App frames against independent native caption-creation witnesses.
use super::{
    CDXML, FIRST_RUN, Fixture, MULTI, PHASE, TestResult, button, choose_file, click, closed,
    current_cdx, field_value, open_pinned, open_run, paper_point, set_field, shortcut,
};
use bondline_commands::Engine;
use bondline_model::PageSize;
use eframe::egui::{self, Event, Key, Modifiers};
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
const BASELINE: [f64; 2] = [144.25, 216.75];
const BODY: &str = "Phase β & q";
// FontTools interior probes from the independent fixture generator.
const ORIGINAL_INK: [f64; 2] = [85.0, 119.25];
const NEW_INK: [f64; 2] = [148.25, 212.75];

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
    let press = paper_point(f, BASELINE)?;
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
                "position":{"x_pt":BASELINE[0],"y_pt":BASELINE[1]},"text":"","current":true})
    );
    assert_eq!(field_value(f, "Caption text")?, "");
    Ok(())
}

fn apply_body(f: &mut Fixture, body: &str) -> TestResult {
    set_field(f, "Caption text", body)?;
    f.choose("Apply")?;
    closed(f)
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
fn imported_caption_sources_keep_editing_without_new_caption_creation() -> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let cdx = f.directory.path("imported-caption.cdx");
    open_pinned(&mut f, &cdx, "cdx", MULTI)?;
    f.choose("Text")?;
    refuse_new_caption(&mut f, [350.0, 350.0])?;
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
