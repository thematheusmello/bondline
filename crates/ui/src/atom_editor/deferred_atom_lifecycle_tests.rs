//! Independent private AtomEditor lifecycle: bounded deferred AX text, real widgets.
use super::*;
use serde_json::json;
use std::fs;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn frame(
    editor: &mut AtomEditor,
    engine: &mut Engine,
    context: &egui::Context,
    events: Vec<egui::Event>,
    time: f64,
) -> egui::FullOutput {
    context.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1160.0, 800.0),
            )),
            events,
            time: Some(time),
            focused: true,
            ..Default::default()
        },
        |context| {
            // Read this pass's actual events, including egui's empty discard passes.
            let events = context.input(|input| input.events.clone());
            editor.show(context, Some(&mut *engine), &events);
        },
    )
}

fn set_value(target: egui::Id, value: &str) -> egui::Event {
    egui::Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
        action: egui::accesskit::Action::SetValue,
        target: egui::accesskit::NodeId(target.value()),
        data: Some(egui::accesskit::ActionData::Value(value.into())),
    })
}

fn apply(target: egui::Id) -> egui::Event {
    egui::Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
        action: egui::accesskit::Action::Default,
        target: egui::accesskit::NodeId(target.value()),
        data: None,
    })
}

fn retained_text_is_bounded(form: &Form) -> TestResult {
    let mut bytes = 0;
    for event in &form.deferred_text {
        let egui::Event::AccessKitActionRequest(request) = event else {
            return Err("non-AX event retained as deferred text".into());
        };
        assert_eq!(request.action, egui::accesskit::Action::SetValue);
        let Some(egui::accesskit::ActionData::Value(value)) = &request.data else {
            return Err("non-Value request retained as deferred text".into());
        };
        bytes += value.len();
        let limit = if request.target.0 == field_id(form.request.id, 0).value() {
            2
        } else if request.target.0 == field_id(form.request.id, 1).value() {
            5
        } else {
            return Err("unrelated target retained as deferred text".into());
        };
        assert!(
            value.len() <= limit,
            "retained {} bytes for limit {limit}",
            value.len()
        );
    }
    assert!(
        form.deferred_text.len() <= 2,
        "retained {} requests / {bytes} Value bytes",
        form.deferred_text.len()
    );
    assert!(bytes <= 7, "retained {bytes} Value bytes");
    Ok(())
}

#[test]
fn refused_apply_bounds_targeted_deferred_text_and_fresh_apply_commits_once() -> TestResult {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../io/tests/fixtures/bondline-v0.cdxml"
    );
    let original_bytes = fs::read(path)?;
    for huge_last_element in [true, false] {
        let mut engine = Engine::default();
        engine.dispatch("document.open_cdxml", json!({"path":path}))?;
        assert!(engine.stored_document()?.is_fully_mapped());
        let opened = serde_json::to_value(engine.state())?;
        assert_eq!(opened["source"]["format"], "cdxml");
        let atoms = &opened["document"]["pages"][0]["fragments"][0]["atoms"];
        assert_eq!(atoms.as_array().ok_or("native C2 atoms")?.len(), 2);
        assert_eq!(atoms[0]["element"], 6);
        assert_eq!(atoms[1]["element"], 6);
        engine.dispatch(
            "atom.set_label",
            json!({
                "expected_revision":engine.state().revision,"page_id":1,
                "fragment_id":1,"atom_id":2,"element":17,"explicit_hydrogens":0
            }),
        )?;
        engine.set_ui_file_host(true);
        let request = engine
            .dispatch(
                "ui.atom_label",
                json!({
                    "action":"open","expected_revision":engine.state().revision,
                    "page_id":1,"fragment_id":1,"atom_id":2
                }),
            )?
            .ui_atom_label_operation
            .ok_or("atom request")?;
        let before = serde_json::to_value(engine.state())?;
        let content = engine.content().clone();
        let owner = engine.capture_stamp();
        let revision = engine.state().revision;
        let undo_depth = engine.state().undo_depth;
        let context = egui::Context::default();
        crate::configure_context(&context);
        context.enable_accesskit();
        let mut editor = AtomEditor::default();
        let mut error = None;
        editor.sync(&engine, &mut error);
        assert!(error.is_none(), "{error:?}");
        // First layout establishes the Window; the second supplies its displayed IDs.
        frame(&mut editor, &mut engine, &context, vec![], 0.0);
        // At t=0 Window fade-out returns before Area creation; then one sizing pass is hidden.
        frame(&mut editor, &mut engine, &context, vec![], 1.0 / 120.0);
        frame(&mut editor, &mut engine, &context, vec![], 1.0 / 60.0);
        let form = editor.form.as_ref().ok_or("actual form")?;
        assert_eq!(form.request.id, request);
        assert_eq!(form.element.as_str(), "Cl");
        assert_eq!(form.hydrogens.as_str(), "0");
        let button = form.buttons[0].ok_or("displayed Apply ID")?;
        assert!(
            context
                .read_response(button)
                .ok_or("Apply response")?
                .enabled()
        );
        let element = field_id(request, 0);
        let hydrogens = field_id(request, 1);
        let mut events = vec![set_value(element, "xx"), apply(button)];
        for index in 0..64 {
            events.push(set_value(element, if index % 2 == 0 { "Cl" } else { "Br" }));
            events.push(set_value(
                hydrogens,
                if index % 2 == 0 { "12" } else { "0" },
            ));
        }
        events.push(set_value(hydrogens, "0"));
        let final_element = if huge_last_element {
            "X".repeat(65537)
        } else {
            "F".to_owned()
        };
        events.push(set_value(element, &final_element));
        let output = frame(&mut editor, &mut engine, &context, events, 2.0 / 60.0);
        let form = editor.form.as_ref().ok_or("refused Apply retains form")?;
        assert_eq!(
            form.element.as_str(),
            "xx",
            "Apply must see its prefix, not later replacements"
        );
        assert_eq!(form.hydrogens.as_str(), "0");
        assert!(output.shapes.iter().any(|paint| matches!(&paint.shape,
            egui::Shape::Text(text) if text.galley.text().contains("Enter an exact-case element symbol")
                && paint.clip_rect.contains_rect(paint.shape.visual_bounding_rect()))),
            "the invalid prefix must have a visible validation error");
        assert_eq!(serde_json::to_value(engine.state())?, before);
        assert_eq!(engine.content(), &content);
        assert!(engine.capture_stamp_matches(&owner));
        assert_eq!(fs::read(path)?, original_bytes);
        retained_text_is_bounded(form)?;
        if huge_last_element {
            continue;
        }

        frame(&mut editor, &mut engine, &context, vec![], 3.0 / 60.0);
        let form = editor.form.as_ref().ok_or("recovered draft")?;
        assert_eq!(form.element.as_str(), "F");
        assert_eq!(form.hydrogens.as_str(), "0");
        assert!(
            form.deferred_text.is_empty(),
            "the deferred replacements run once"
        );
        assert_eq!(serde_json::to_value(engine.state())?, before);
        frame(&mut editor, &mut engine, &context, vec![], 4.0 / 60.0);
        let form = editor
            .form
            .as_ref()
            .ok_or("draft remains until a fresh Apply")?;
        assert_eq!(form.element.as_str(), "F");
        assert_eq!(form.hydrogens.as_str(), "0");
        assert!(form.deferred_text.is_empty());
        assert_eq!(serde_json::to_value(engine.state())?, before);
        assert_eq!(engine.content(), &content);
        assert!(engine.capture_stamp_matches(&owner));
        let button = form.buttons[0].ok_or("fresh Apply ID")?;
        assert!(
            context
                .read_response(button)
                .ok_or("fresh Apply response")?
                .enabled()
        );
        frame(
            &mut editor,
            &mut engine,
            &context,
            vec![apply(button)],
            5.0 / 60.0,
        );
        assert!(!editor.present());
        let committed = serde_json::to_value(engine.state())?;
        let mut expected = before["document"].clone();
        expected["pages"][0]["fragments"][0]["atoms"][1]["element"] = json!(9);
        expected["pages"][0]["fragments"][0]["atoms"][1]["explicit_hydrogens"] = json!(0);
        assert_eq!(committed["document"], expected);
        assert_eq!(engine.state().revision, revision + 1);
        assert_eq!(engine.state().undo_depth, undo_depth + 1);
        assert_eq!(engine.state().redo_depth, 0);
        assert!(engine.state().ui_atom_label.is_none());
        assert_eq!(committed["source"], before["source"]);
        assert_eq!(committed["file"], before["file"]);
        frame(&mut editor, &mut engine, &context, vec![], 6.0 / 60.0);
        assert_eq!(serde_json::to_value(engine.state())?, committed);
        assert_eq!(fs::read(path)?, original_bytes);
    }
    Ok(())
}
