//! Independent actual-App caption declarations; child of view_tests.rs.
//! These frames do not establish native OS interaction or full Text-tool parity.
use super::*;
use bondline_model::{Document, PageSize, Point2, StyledText, TextId, TextRun, TextStyle};
use bondline_render::{Cancellation, PageView};
use egui::accesskit::{Action, ActionData, ActionRequest, Role};
use std::time::{Duration, Instant};

const CAPTION_SOURCE: &str = include_str!("../../io/tests/fixtures/captions/explicit.cdxml");

#[track_caller]
fn caption_wait(
    f: &mut Fixture,
    ready: impl Fn(&Fixture, &egui::FullOutput) -> TestResult<bool>,
) -> TestResult<egui::FullOutput> {
    let caller = std::panic::Location::caller();
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let shown = f.frame(vec![]);
        if ready(f, &shown)? {
            return Ok(shown);
        }
        if Instant::now() >= deadline {
            return Err(format!("caption wait at {caller}: {:?}", f.app.error).into());
        }
        std::thread::yield_now();
    }
}

fn caption_ink(shown: &egui::FullOutput) -> Vec<egui::Rect> {
    shown
        .shapes
        .iter()
        .filter_map(|paint| match &paint.shape {
            egui::Shape::Rect(rect) if rect.fill == egui::Color32::from_rgb(51, 102, 204) => {
                Some(rect.rect)
            }
            _ => None,
        })
        .collect()
}

fn caption_open(f: &mut Fixture) -> TestResult<egui::FullOutput> {
    fs::write(f.directory.path("caption-λ.cdxml"), CAPTION_SOURCE)?;
    f.warm();
    f.frame(vec![
        key(Key::O, true, Modifiers::COMMAND),
        key(Key::O, false, Modifiers::COMMAND),
    ]);
    caption_wait(f, |f, _| {
        Ok(f.node("File name", &[Role::TextInput]).is_ok())
    })?;
    let (id, _) = f.node("File name", &[Role::TextInput])?;
    f.frame(vec![Event::AccessKitActionRequest(ActionRequest {
        action: Action::SetValue,
        target: id,
        data: Some(ActionData::Value("caption-λ.cdxml".into())),
    })]);
    assert_eq!(
        f.nodes.get(&id).and_then(|node| node.value()),
        Some("caption-λ.cdxml")
    );
    f.choose("Open")?;
    caption_wait(f, |f, _| {
        Ok(!f.app.files.is_busy()
            && !f.app.executor.has_active_work()
            && f.state()?["ui_file"]["request"].is_null()
            && f.state()?["document"]["pages"][0]["texts"][0]["runs"][0]["text"] == "Phase A")
    })?;
    let shown = f.warm();
    assert!(f.app.error.is_none(), "{:?}", f.app.error);
    Ok(shown)
}

fn caption_capture(f: &Fixture) -> TestResult<Value> {
    let result = f.command(
        "render.capture",
        json!({"page_id":1,"width_px":576,"height_px":576,"backend":"cpu"}),
    )?;
    let png = result["capture"]["png_base64"].clone();
    assert!(png.as_str().is_some_and(|value| !value.is_empty()));
    Ok(png)
}

#[test]
fn caption_logical_readability_checks_small_run_at_double_density() -> TestResult {
    for (small, accepted) in [(3.0, false), (4.0, true)] {
        let mut document = Document::default();
        let id = document.pages().first().ok_or("page")?.id();
        document.set_page_size(id, PageSize::new(256.0, 256.0)?)?;
        let runs = [small, 12.0]
            .into_iter()
            .map(|size| {
                TextRun::new(
                    "H".into(),
                    TextStyle::new("Roboto".into(), size, 0, [0.2, 0.4, 0.8])?,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        document.append_texts(
            id,
            vec![StyledText::new(
                TextId::new(1).ok_or("text id")?,
                Point2::new(32.0, 96.0)?,
                runs,
            )?],
        )?;
        let page = document.pages().first().ok_or("page")?;
        let graph = PageView::fit(576, 576, page.size())?
            .at_display_scale(2.0, [0.0, 0.0])?
            .graph_scene(page, &Cancellation::default())?;
        assert_eq!(
            graph.label_em_px(),
            None,
            "chemical metadata stays separate"
        );
        assert_eq!(graph.text_em_range_px(), Some([small as f32 * 4.0, 48.0]));
        let checked = readable_graph(graph, 2.0);
        if accepted {
            assert!(checked.is_ok(), "logical boundary em8 must remain readable");
        } else {
            assert!(matches!(
                checked,
                Err(bondline_commands::Error::Render(
                    bondline_render::Error::LabelScale
                ))
            ));
        }
    }
    Ok(())
}

#[test]
fn actual_file_open_paints_foreign_caption_without_granting_chemical_edit_authority() -> TestResult
{
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let shown = caption_open(&mut f)?;
    assert!(
        !caption_ink(&shown).is_empty(),
        "actual canvas needs caption ink"
    );
    {
        let engine = f.app.engine.lock().map_err(|_| "engine")?;
        let stored = engine.stored_document()?;
        assert!(stored.has_caption_projection());
        assert!(!stored.is_fully_mapped());
        let single = engine
            .controls()
            .into_iter()
            .find(|control| control.id == "tool.bond.single")
            .ok_or("single bond control")?;
        assert!(!single.enabled);
        assert_eq!(single.disabled_reason, Some("unsupported_content"));
    }
    let before = f.state()?;
    assert_eq!(before["file"]["dirty"], false);
    assert_eq!(before["source"]["modified"], false);
    let area = canvas(&shown)?;
    let start = area.center() + egui::vec2(-20.0, 50.0);
    let end = start + egui::vec2(40.0, 0.0);
    f.frame(vec![Event::PointerMoved(start), button(start, true)]);
    f.frame(vec![Event::PointerMoved(end)]);
    f.frame(vec![button(end, false)]);
    f.frame(vec![Event::PointerMoved(start), button(start, true)]);
    f.frame(vec![button(start, false)]);
    f.warm();
    assert_eq!(f.state()?, before, "read-only canvas cannot author a bond");
    assert!(f.app.error.is_none(), "{:?}", f.app.error);
    assert!(!caption_capture(&f)?.is_null());
    f.frame(tap(Key::F5));
    assert_eq!(f.view()?["zoom_percent"], 100.0);
    f.frame(tap(Key::F7));
    assert_eq!(f.view()?["zoom_percent"], 125.0);
    f.command(
        "ui.view",
        json!({"action":"pan","center_x_pt":120,"center_y_pt":120}),
    )?;
    assert_eq!(f.view()?["center_x_pt"], 120.0);
    assert_eq!(f.view()?["center_y_pt"], 120.0);
    assert!(!caption_ink(&f.warm()).is_empty());
    assert_eq!(f.state()?, before, "camera and capture cannot dirty source");
    assert_eq!(
        fs::read(f.directory.path("caption-λ.cdxml"))?,
        CAPTION_SOURCE.as_bytes()
    );
    Ok(())
}

#[test]
fn actual_app_remote_run_edit_repaints_history_and_rejects_old_view_capture_epoch() -> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let shown = caption_open(&mut f)?;
    let original_ink = caption_ink(&shown);
    assert!(!original_ink.is_empty());
    let before = f.state()?;
    let original_capture = caption_capture(&f)?;
    let old_source = f.app.engine.lock().map_err(|_| "engine")?.capture_stamp();
    let revision = before["revision"].as_u64().ok_or("revision")?;
    f.command(
        "text.set_run",
        json!({"expected_revision":revision,"page_id":1,"text_id":1,"run_index":0,
            "text":"Phase B & μ"}),
    )?;
    let edited = f.state()?;
    assert_eq!(
        edited["document"]["pages"][0]["texts"][0]["runs"][0]["text"],
        "Phase B & μ"
    );
    assert_eq!(edited["revision"], revision + 1);
    assert_eq!(edited["undo_depth"], 1);
    assert_eq!(edited["redo_depth"], 0);
    assert_eq!(edited["file"]["dirty"], true);
    assert_eq!(edited["source"]["modified"], true);
    let changed_ink = caption_ink(&f.warm());
    assert!(!changed_ink.is_empty());
    assert_ne!(changed_ink, original_ink);
    assert_ne!(caption_capture(&f)?, original_capture);
    assert!(
        !f.app
            .engine
            .lock()
            .map_err(|_| "engine")?
            .capture_stamp_matches(&old_source)
    );
    f.command("history.undo", json!({}))?;
    assert_eq!(f.state()?["document"], before["document"]);
    assert_eq!(caption_ink(&f.warm()), original_ink);
    assert_eq!(caption_capture(&f)?, original_capture);
    assert!(
        !f.app
            .engine
            .lock()
            .map_err(|_| "engine")?
            .capture_stamp_matches(&old_source)
    );
    f.command("history.redo", json!({}))?;
    assert_eq!(f.state()?["document"], edited["document"]);
    assert_eq!(caption_ink(&f.warm()), changed_ink);
    let old_view = f.view()?["epoch"].as_u64().ok_or("view epoch")?;
    f.command("ui.view", json!({"action":"set_zoom","percent":125}))?;
    let current_state = f.state()?;
    let rejected = f.app.engine.lock().map_err(|_| "engine")?.dispatch(
        "render.capture",
        json!({"page_id":1,"width_px":576,"height_px":576,"backend":"cpu",
            "view":{"logical_width":576,"logical_height":576,"expected_view_epoch":old_view}}),
    );
    let error = rejected.err().ok_or("old view capture must refuse")?;
    assert_eq!(error.info().code, "stale_view");
    assert_eq!(f.state()?, current_state);
    assert_eq!(
        fs::read(f.directory.path("caption-λ.cdxml"))?,
        CAPTION_SOURCE.as_bytes()
    );
    assert!(f.app.error.is_none(), "{:?}", f.app.error);
    Ok(())
}

#[test]
fn actual_caption_read_only_gate_preserves_keyboard_history_and_save() -> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let shown = caption_open(&mut f)?;
    let original_ink = caption_ink(&shown);
    let original = f.state()?["document"].clone();
    let revision = f.state()?["revision"].as_u64().ok_or("revision")?;
    f.command(
        "text.set_run",
        json!({"expected_revision":revision,"page_id":1,"text_id":1,"run_index":0,
            "text":"Phase B"}),
    )?;
    let edited = f.state()?["document"].clone();
    let edited_ink = caption_ink(&f.warm());
    assert_ne!(edited_ink, original_ink);
    f.frame(vec![
        key(Key::Z, true, Modifiers::COMMAND),
        key(Key::Z, false, Modifiers::COMMAND),
    ]);
    assert_eq!(f.state()?["document"], original);
    assert_eq!(caption_ink(&f.warm()), original_ink);
    let redo = Modifiers {
        shift: true,
        ..Modifiers::COMMAND
    };
    f.frame(vec![key(Key::Z, true, redo), key(Key::Z, false, redo)]);
    assert_eq!(f.state()?["document"], edited);
    assert_eq!(caption_ink(&f.warm()), edited_ink);
    assert_eq!(f.state()?["file"]["dirty"], true);
    f.frame(vec![
        key(Key::S, true, Modifiers::COMMAND),
        key(Key::S, false, Modifiers::COMMAND),
    ]);
    caption_wait(&mut f, |f, _| {
        Ok(!f.app.files.is_busy()
            && !f.app.executor.has_active_work()
            && f.state()?["ui_file"]["request"].is_null()
            && f.state()?["file"]["dirty"] == false)
    })?;
    assert_eq!(f.state()?["document"], edited);
    assert_eq!(
        f.state()?["source"]["modified"],
        true,
        "Save retains original backing; dirty compares the saved baseline"
    );
    assert_eq!(
        fs::read(f.directory.path("caption-λ.cdxml"))?,
        CAPTION_SOURCE.replace("Phase A", "Phase B").as_bytes()
    );
    assert!(f.app.error.is_none(), "{:?}", f.app.error);
    Ok(())
}
