//! Independent read-only SDF inspection through the real App and existing frame harness.
use super::{Fixture, TestResult, key};
use eframe::egui;
use egui::{Event, Key, Modifiers};
use serde_json::{Value, json};
use std::{
    fs,
    time::{Duration, Instant},
};

const SOURCE: &[u8] = include_bytes!("../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf");

fn wait_for(
    fixture: &mut Fixture,
    condition: impl Fn(&Fixture, &egui::FullOutput) -> bool,
) -> TestResult<egui::FullOutput> {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let output = fixture.frame(vec![]);
        if condition(fixture, &output) {
            return Ok(output);
        }
        if Instant::now() >= deadline {
            let text = output
                .shapes
                .iter()
                .filter_map(|shape| match &shape.shape {
                    egui::Shape::Text(text) => Some(text.galley.text()),
                    _ => None,
                })
                .collect::<Vec<_>>();
            return Err(format!(
                "real inspector/preview did not become ready: {:?}; {text:?}",
                fixture.app.error
            )
            .into());
        }
        std::thread::sleep(Duration::from_millis(2));
    }
}

fn value(fixture: &Fixture, label: &str) -> TestResult<String> {
    let (id, _) = fixture.node(
        label,
        &[
            egui::accesskit::Role::TextInput,
            egui::accesskit::Role::MultilineTextInput,
        ],
    )?;
    let mut text = String::new();
    let mut stack = vec![id];
    let mut found = false;
    while let Some(id) = stack.pop() {
        let node = fixture.nodes.get(&id).ok_or("current text node")?;
        if node.role() == egui::accesskit::Role::InlineTextBox {
            found = true;
            text.push_str(node.value().ok_or("actual inline text value")?);
        }
        stack.extend(node.children().iter().rev().copied());
    }
    if !found {
        return Err("read-only value has no actual inline text".into());
    }
    Ok(text)
}

fn visible_text(output: &egui::FullOutput, expected: &str) -> bool {
    output.shapes.iter().any(|paint| {
        matches!(&paint.shape,
        egui::Shape::Text(text) if text.galley.text() == expected
            && paint.clip_rect.contains_rect(paint.shape.visual_bounding_rect()))
    })
}

fn painted_preview(fixture: &Fixture, output: &egui::FullOutput) -> bool {
    !fixture.app.preview.is_busy()
        && fixture
            .context
            .tessellate(output.shapes.clone(), output.pixels_per_point)
            .iter()
            .any(|paint| {
                matches!(&paint.primitive, egui::epaint::Primitive::Mesh(mesh)
                if mesh.texture_id != egui::TextureId::Managed(0)
                    && mesh.vertices.len() == 4 && mesh.indices.len() == 6)
            })
}

#[test]
fn explicit_record_occurrence_inspection_is_readonly_and_keeps_the_real_dpi2_preview() -> TestResult
{
    assert_eq!(SOURCE.len(), 777);
    let mut fixture = Fixture::new(1160.0, 800.0, 2.0)?;
    let path = fixture.directory.path("inspector-777.sdf");
    fs::write(&path, SOURCE)?;
    fixture.command("document.open_sdf", json!({"path":path}))?;
    let before = fixture.state()?;
    let owner = fixture
        .app
        .engine
        .lock()
        .map_err(|_| "engine")?
        .content()
        .clone();
    assert_eq!(before["document"], Value::Null);
    assert_eq!(before["collection"]["record_count"], 2);
    assert_eq!(before["collection"]["records"][0]["mol_version"], "v3000");
    assert_eq!(before["collection"]["records"][1]["mol_version"], "v2000");

    fixture.choose("Records")?;
    fixture.choose("Inspect record 1 fields")?;
    let output = wait_for(&mut fixture, |fixture, output| {
        visible_text(output, "Record 1 fields")
            && fixture
                .node("Field 3: DUP", &[egui::accesskit::Role::Button])
                .is_ok()
    })?;
    assert!(visible_text(&output, "Record 1 fields"));
    let mut previous_y = f32::NEG_INFINITY;
    for label in [
        "Field 1: EMPTY",
        "Field 2: DUP",
        "Field 3: DUP",
        "Field 4: MASS",
        "Field 5: DT0012",
        "Field 6: SPACES",
    ] {
        let (_, bounds) = fixture.node(label, &[egui::accesskit::Role::Button])?;
        assert!(bounds.center().y > previous_y, "ordered occurrence {label}");
        previous_y = bounds.center().y;
    }
    fixture.choose("Field 3: DUP")?;
    let label = "Record 1 field 3 value";
    let output = wait_for(&mut fixture, |fixture, output| {
        value(fixture, label).is_ok_and(|value| value == "second")
            && visible_text(output, "> <DUP>")
            && visible_text(output, "Value: 7 bytes, 1 line")
    })?;
    assert!(visible_text(&output, "> <DUP>"));
    let (id, bounds) = fixture.node(
        label,
        &[
            egui::accesskit::Role::TextInput,
            egui::accesskit::Role::MultilineTextInput,
        ],
    )?;
    assert!(
        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1160.0, 800.0))
            .contains_rect(bounds)
    );
    fixture.frame(vec![Event::AccessKitActionRequest(
        egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::Focus,
            target: id,
            data: None,
        },
    )]);
    let copied = fixture.frame(vec![
        key(Key::A, true, Modifiers::COMMAND),
        key(Key::A, false, Modifiers::COMMAND),
        Event::Copy,
    ]);
    assert_eq!(
        copied.platform_output.copied_text, "second",
        "value is actually selectable/copyable"
    );
    fixture.frame(vec![Event::Text("SHOULD_NOT_REPLACE".into())]);
    assert_eq!(
        value(&fixture, label)?,
        "second",
        "the value widget is genuinely read-only"
    );
    wait_for(&mut fixture, painted_preview)?;
    assert_eq!(fixture.state()?, before);
    let engine = fixture.app.engine.lock().map_err(|_| "engine")?;
    assert_eq!(engine.content(), &owner);
    assert_eq!(engine.content().sdf()?.as_bytes(), SOURCE);
    assert_eq!(fs::read(&path)?, SOURCE);
    assert!(
        engine.state().sdf_record.is_none(),
        "explicit fields do not enter automatic state"
    );
    assert!(!format!("{:?}", engine.state()).contains("second"));
    Ok(())
}

#[test]
fn empty_duplicate_spaces_and_record_replacement_stay_literal_and_source_switch_clears_values()
-> TestResult {
    let mut fixture = Fixture::new(1160.0, 800.0, 2.0)?;
    let path = fixture.directory.path("inspector-literal.sdf");
    fs::write(&path, SOURCE)?;
    fixture.command("document.open_sdf", json!({"path":path}))?;
    let before = fixture.state()?;
    fixture.choose("Records")?;
    fixture.choose("Inspect record 1 fields")?;
    wait_for(&mut fixture, |fixture, _| {
        fixture
            .node("Field 1: EMPTY", &[egui::accesskit::Role::Button])
            .is_ok()
    })?;
    fixture.choose("Field 1: EMPTY")?;
    wait_for(&mut fixture, |_, output| {
        visible_text(output, "> <EMPTY>") && visible_text(output, "Value: 0 bytes, 0 lines")
    })?;
    fixture.choose("Field 1: EMPTY")?;
    for (row, label, expected, metadata) in [
        (
            "Field 2: DUP",
            "Record 1 field 2 value",
            "first",
            "Value: 6 bytes, 1 line",
        ),
        (
            "Field 6: SPACES",
            "Record 1 field 6 value",
            "   ",
            "Value: 4 bytes, 1 line",
        ),
    ] {
        fixture.choose(row)?;
        wait_for(&mut fixture, |fixture, output| {
            value(fixture, label).is_ok_and(|value| value == expected)
                && visible_text(output, metadata)
        })?;
        fixture.choose(row)?;
    }
    fixture.choose("Inspect record 2 fields")?;
    let output = wait_for(&mut fixture, |fixture, output| {
        visible_text(output, "Record 2 fields")
            && fixture
                .node("Field 1: ORDINARY", &[egui::accesskit::Role::Button])
                .is_ok()
    })?;
    assert!(!visible_text(&output, "Field 2: DUP"));
    assert!(
        fixture
            .node("Field 3: DUP", &[egui::accesskit::Role::Button])
            .is_err()
    );
    fixture.choose("Field 1: ORDINARY")?;
    wait_for(&mut fixture, |fixture, output| {
        value(fixture, "Record 2 field 1 value").is_ok_and(|value| value == "second record")
            && visible_text(output, "> <ORDINARY>")
            && visible_text(output, "Value: 14 bytes, 1 line")
    })?;
    wait_for(&mut fixture, painted_preview)?;
    assert_eq!(fixture.state()?, before);
    assert_eq!(fs::read(&path)?, SOURCE);

    let replacement = include_bytes!("../../io/tests/fixtures/sdf/ordered-fields-v2000.sdf");
    let next = fixture.directory.path("inspector-replacement.sdf");
    fs::write(&next, replacement)?;
    fixture.command("document.open_sdf", json!({"path":next}))?;
    let current = fixture.state()?;
    assert_eq!(current["collection"]["byte_length"], 690);
    assert_eq!(current["collection"]["records"][1]["field_count"], 0);
    let output = fixture.frame(vec![]);
    assert!(
        !visible_text(&output, "second record"),
        "old explicit data must clear before painting a new source"
    );
    assert!(
        fixture
            .node(
                "Record 2 field 1 value",
                &[
                    egui::accesskit::Role::TextInput,
                    egui::accesskit::Role::MultilineTextInput,
                ]
            )
            .is_err(),
        "the new record 2 has no fields, so its old value cannot remain current"
    );
    wait_for(&mut fixture, painted_preview)?;
    assert_eq!(fixture.state()?, current);
    let engine = fixture.app.engine.lock().map_err(|_| "engine")?;
    assert!(engine.state().sdf_record.is_none());
    assert_eq!(engine.content().sdf()?.as_bytes(), replacement);
    assert_eq!(fs::read(&next)?, replacement);
    assert!(!format!("{:?}", engine.state()).contains("second record"));
    Ok(())
}

#[test]
fn selected_field_accessibility_declares_readonly_without_set_value_authority() -> TestResult {
    let mut fixture = Fixture::new(1160.0, 800.0, 2.0)?;
    let path = fixture.directory.path("inspector-accessibility.sdf");
    fs::write(&path, SOURCE)?;
    fixture.command("document.open_sdf", json!({"path":path}))?;
    fixture.choose("Records")?;
    fixture.choose("Inspect record 1 fields")?;
    wait_for(&mut fixture, |fixture, _| {
        fixture
            .node("Field 3: DUP", &[egui::accesskit::Role::Button])
            .is_ok()
    })?;
    fixture.choose("Field 3: DUP")?;
    let label = "Record 1 field 3 value";
    wait_for(&mut fixture, |fixture, _| {
        value(fixture, label).is_ok_and(|value| value == "second")
    })?;
    let (id, _) = fixture.node(
        label,
        &[
            egui::accesskit::Role::TextInput,
            egui::accesskit::Role::MultilineTextInput,
        ],
    )?;
    let node = fixture.nodes.get(&id).ok_or("current value input")?;
    assert!(
        node.is_read_only(),
        "the native accessibility bridge must see the actual value input as read-only"
    );
    assert!(node.supports_action(egui::accesskit::Action::Focus));
    assert!(
        !node.supports_action(egui::accesskit::Action::SetValue),
        "inspection does not grant native SetValue authority"
    );
    Ok(())
}

#[test]
fn page_budget_navigation_follows_returned_offsets_and_omission_is_not_an_empty_value() -> TestResult
{
    let original = include_bytes!("../../io/tests/fixtures/sdf/ordered-fields-v2000.sdf");
    let mut source = original
        .get(..282)
        .ok_or("original literal MOL span")?
        .to_vec();
    // 323 literal LF lines: 64,883 raw bytes, 65,530 bytes as a compact JSON vector.
    // Sixteen vectors alone consume 1,048,480 bytes, leaving only 96 bytes for all
    // field/page metadata, so the 1 MiB response budget permits only fifteen.
    let mut lines = vec!["a".repeat(200); 322];
    lines.push("a".repeat(160));
    assert_eq!(
        lines.iter().map(|line| line.len() + 1).sum::<usize>(),
        64_883
    );
    assert_eq!(serde_json::to_vec(&lines)?.len(), 65_530);
    for number in 1..=37 {
        source.extend_from_slice(format!("> <P{number:02}>\n").as_bytes());
        for line in &lines {
            source.extend_from_slice(line.as_bytes());
            source.push(b'\n');
        }
        source.push(b'\n');
    }
    source.extend_from_slice(b"> <OMITTED>\n");
    for _ in 0..328 {
        source.extend(std::iter::repeat_n(b'z', 200));
        source.push(b'\n');
    }
    source.extend_from_slice(b"\n$$$$\n");
    let mut fixture = Fixture::new(1160.0, 800.0, 2.0)?;
    let path = fixture.directory.path("inspector-page-budget.sdf");
    fs::write(&path, &source)?;
    fixture.command("document.open_sdf", json!({"path":path}))?;
    let before = fixture.state()?;
    assert_eq!(before["collection"]["record_count"], 1);
    assert_eq!(before["collection"]["records"][0]["field_count"], 38);
    fixture.choose("Records")?;
    fixture.choose("Inspect record 1 fields")?;
    wait_for(&mut fixture, |fixture, _| {
        fixture
            .node("Field 1: P01", &[egui::accesskit::Role::Button])
            .is_ok()
    })?;
    assert!(
        fixture
            .node("Field 16: P16", &[egui::accesskit::Role::Button])
            .is_err()
    );
    fixture.choose("Next fields")?;
    wait_for(&mut fixture, |fixture, _| {
        fixture
            .node("Field 16: P16", &[egui::accesskit::Role::Button])
            .is_ok()
    })?;
    assert!(
        fixture
            .node("Field 1: P01", &[egui::accesskit::Role::Button])
            .is_err()
    );
    fixture.choose("Next fields")?;
    wait_for(&mut fixture, |fixture, _| {
        fixture
            .node("Field 31: P31", &[egui::accesskit::Role::Button])
            .is_ok()
    })?;
    assert!(
        fixture
            .node("Next fields", &[egui::accesskit::Role::Button])
            .is_err()
    );
    fixture.choose("Field 38: OMITTED")?;
    wait_for(&mut fixture, |_, output| {
        visible_text(output, "Value: 65928 bytes, 328 lines")
            && visible_text(
                output,
                "Value omitted by the inspection limit; it is not an empty value.",
            )
    })?;
    assert!(
        fixture
            .node(
                "Record 1 field 38 value",
                &[
                    egui::accesskit::Role::TextInput,
                    egui::accesskit::Role::MultilineTextInput,
                ]
            )
            .is_err(),
        "omitted null is never presented as an empty text input"
    );
    fixture.choose("Previous fields")?;
    wait_for(&mut fixture, |fixture, _| {
        fixture
            .node("Field 16: P16", &[egui::accesskit::Role::Button])
            .is_ok()
    })?;
    fixture.choose("Previous fields")?;
    wait_for(&mut fixture, |fixture, _| {
        fixture
            .node("Field 1: P01", &[egui::accesskit::Role::Button])
            .is_ok()
    })?;
    assert!(
        fixture
            .node("Previous fields", &[egui::accesskit::Role::Button])
            .is_err()
    );
    wait_for(&mut fixture, painted_preview)?;
    assert_eq!(fixture.state()?, before);
    let engine = fixture.app.engine.lock().map_err(|_| "engine")?;
    assert!(engine.state().sdf_record.is_none());
    assert_eq!(engine.content().sdf()?.as_bytes(), source);
    assert_eq!(fs::read(&path)?, source);
    Ok(())
}

#[test]
fn focused_readonly_value_does_not_persist_payload_in_text_edit_undo_state() -> TestResult {
    let mut fixture = Fixture::new(1160.0, 800.0, 2.0)?;
    let path = fixture.directory.path("inspector-no-text-history.sdf");
    fs::write(&path, SOURCE)?;
    fixture.command("document.open_sdf", json!({"path":path}))?;
    fixture.choose("Records")?;
    fixture.choose("Inspect record 1 fields")?;
    wait_for(&mut fixture, |fixture, _| {
        fixture
            .node("Field 3: DUP", &[egui::accesskit::Role::Button])
            .is_ok()
    })?;
    fixture.choose("Field 3: DUP")?;
    let label = "Record 1 field 3 value";
    wait_for(&mut fixture, |fixture, _| {
        value(fixture, label).is_ok_and(|value| value == "second")
    })?;
    let (id, _) = fixture.node(
        label,
        &[
            egui::accesskit::Role::TextInput,
            egui::accesskit::Role::MultilineTextInput,
        ],
    )?;
    fixture.frame(vec![Event::AccessKitActionRequest(
        egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::Focus,
            target: id,
            data: None,
        },
    )]);
    let focused = fixture
        .context
        .memory(|memory| memory.focused())
        .ok_or("actual focused value widget")?;
    let empty = (
        egui::text::CCursorRange::one(egui::text::CCursor::new(0)),
        String::new(),
    );
    let state = egui::TextEdit::load_state(&fixture.context, focused)
        .ok_or("actual value TextEdit state")?;
    assert!(
        !state.undoer().has_undo(&empty),
        "a read-only field must not retain its disclosed payload in TextEdit undo history"
    );
    assert!(!state.undoer().is_in_flux());
    fixture.choose("Close fields")?;
    let state = egui::TextEdit::load_state(&fixture.context, focused)
        .ok_or("retained value widget state")?;
    assert!(!state.undoer().has_undo(&empty));
    assert!(!state.undoer().is_in_flux());
    Ok(())
}
