//! Real-App recovery/navigation for the unchanged ordinary two-record source.
use super::{Fixture, TestResult, set_slider};
use eframe::egui;
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    fs,
    time::{Duration, Instant},
};

const SOURCE: &[u8] =
    include_bytes!("../../io/tests/fixtures/sdf-preview/valence-absent-mixed.sdf");

fn painted_texture(fixture: &Fixture, output: &egui::FullOutput) -> Option<egui::TextureId> {
    fixture
        .context
        .tessellate(output.shapes.clone(), output.pixels_per_point)
        .iter()
        .find_map(|paint| match &paint.primitive {
            egui::epaint::Primitive::Mesh(mesh)
                if mesh.texture_id != egui::TextureId::Managed(0)
                    && mesh.vertices.len() == 4
                    && mesh.indices.len() == 6 =>
            {
                Some(mesh.texture_id)
            }
            _ => None,
        })
}

fn preview_refusal(output: &egui::FullOutput) -> Option<String> {
    output.shapes.iter().find_map(|paint| match &paint.shape {
        egui::Shape::Text(text) if text.galley.text().starts_with("Preview unavailable:") => {
            Some(text.galley.text().to_owned())
        }
        _ => None,
    })
}

#[test]
fn ordinary_sdf_actual_size_and_existing_pan_controls_render_without_authoring() -> TestResult {
    assert_eq!(SOURCE.len(), 740);
    let mut fixture = Fixture::new(1160.0, 800.0, 1.0)?;
    let path = fixture.directory.path("ordinary-view-化学.sdf");
    fs::write(&path, SOURCE)?;
    fixture.command("document.open_sdf", json!({"path":path}))?;
    let before = fixture.state()?;
    assert_eq!(before["document"], Value::Null);
    assert_eq!(before["collection"]["record_count"], 2);
    assert_eq!(before["collection"]["records"][0]["mol_version"], "v3000");
    assert_eq!(before["collection"]["records"][1]["mol_version"], "v2000");
    let (owner, stamp) = {
        let engine = fixture.app.engine.lock().map_err(|_| "engine")?;
        (engine.content().clone(), engine.capture_stamp())
    };
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let output = fixture.frame(vec![]);
        let texture = painted_texture(&fixture, &output);
        let refusal = preview_refusal(&output);
        if !fixture.app.preview.is_busy()
            && !fixture.app.executor.has_active_work()
            && (texture.is_some() || refusal.is_some())
        {
            println!("initial ordinary SDF preview: texture={texture:?}; refusal={refusal:?}");
            break;
        }
        if Instant::now() >= deadline {
            return Err(
                "initial registered SDF preview neither painted nor reported its actual refusal"
                    .into(),
            );
        }
        std::thread::yield_now();
    }
    assert_eq!(fixture.state()?, before);
    assert_eq!(fs::read(&path)?, SOURCE);

    // The baseline must fail at this existing disabled control, not a future Rust API.
    fixture.choose("View")?;
    fixture.choose("Actual size")?;
    assert_eq!(fixture.view()?["zoom_percent"], 100.0);
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let output = fixture.frame(vec![]);
        if !fixture.app.preview.is_busy()
            && !fixture.app.executor.has_active_work()
            && painted_texture(&fixture, &output).is_some()
            && fixture
                .node("Horizontal position", &[egui::accesskit::Role::Slider])
                .is_ok()
            && fixture
                .node("Vertical position", &[egui::accesskit::Role::Slider])
                .is_ok()
        {
            break;
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "Actual capture did not prepare reachable pan controls: {:?}",
                preview_refusal(&output)
            )
            .into());
        }
        std::thread::yield_now();
    }
    let (horizontal, _) = fixture.node("Horizontal position", &[egui::accesskit::Role::Slider])?;
    fixture.frame(vec![set_slider(horizontal, 100.0)]);
    let (vertical, _) = fixture.node("Vertical position", &[egui::accesskit::Role::Slider])?;
    fixture.frame(vec![set_slider(vertical, 144.0)]);
    let view = fixture.view()?;
    assert_eq!(view["zoom_percent"], 100.0);
    assert_eq!(view["center_x_pt"], 100.0);
    assert_eq!(view["center_y_pt"], 144.0);
    let mut output = fixture.frame(vec![]);
    let mut uploads = HashMap::new();
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        for (id, delta) in &output.textures_delta.set {
            if let egui::ImageData::Color(image) = &delta.image {
                assert!(
                    delta.pos.is_none(),
                    "the registered preview uploads a complete image"
                );
                uploads.insert(*id, image.as_ref().clone());
            }
        }
        if !fixture.app.preview.is_busy()
            && !fixture.app.executor.has_active_work()
            && let Some(id) = painted_texture(&fixture, &output)
            && let Some(image) = uploads.get(&id)
        {
            assert!(image.size.into_iter().all(|side| side > 0 && side <= 2048));
            assert!(
                image.pixels.iter().any(|color| color.a() == 255
                    && color.r() < 80
                    && color.g() < 80
                    && color.b() < 80),
                "the actual registered image contains ink"
            );
            break;
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "100% and center(100,144) did not paint a fresh registered SDF image: {:?}",
                preview_refusal(&output)
            )
            .into());
        }
        std::thread::yield_now();
        output = fixture.frame(vec![]);
    }
    assert_eq!(fixture.state()?, before);
    assert_eq!(fs::read(&path)?, SOURCE);
    let engine = fixture.app.engine.lock().map_err(|_| "engine")?;
    assert_eq!(engine.content(), &owner);
    assert_eq!(engine.content().sdf()?.as_bytes(), SOURCE);
    assert!(engine.capture_stamp_matches(&stamp));
    assert!(fixture.app.error.is_none(), "{:?}", fixture.app.error);
    Ok(())
}

fn wait_image(fixture: &mut Fixture) -> TestResult<egui::FullOutput> {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let output = fixture.frame(vec![]);
        if !fixture.app.preview.is_busy()
            && !fixture.app.executor.has_active_work()
            && painted_texture(fixture, &output).is_some()
        {
            return Ok(output);
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "registered image did not settle: {:?}",
                preview_refusal(&output)
            )
            .into());
        }
        std::thread::yield_now();
    }
}

fn ready_ordinary(dpi: f32) -> TestResult<Fixture> {
    let mut fixture = Fixture::new(1160.0, 800.0, dpi)?;
    let path = fixture.directory.path("ordinary-camera-source.sdf");
    fs::write(&path, SOURCE)?;
    fixture.command("document.open_sdf", json!({"path":path}))?;
    fixture.choose("View")?;
    fixture.choose("Actual size")?;
    wait_image(&mut fixture)?;
    let (horizontal, _) = fixture.node("Horizontal position", &[egui::accesskit::Role::Slider])?;
    fixture.frame(vec![set_slider(horizontal, 100.0)]);
    let (vertical, _) = fixture.node("Vertical position", &[egui::accesskit::Role::Slider])?;
    fixture.frame(vec![set_slider(vertical, 144.0)]);
    wait_image(&mut fixture)?;
    Ok(fixture)
}

fn image_rect(fixture: &Fixture, output: &egui::FullOutput) -> TestResult<egui::Rect> {
    fixture
        .context
        .tessellate(output.shapes.clone(), output.pixels_per_point)
        .iter()
        .find_map(|paint| match &paint.primitive {
            egui::epaint::Primitive::Mesh(mesh)
                if mesh.texture_id != egui::TextureId::Managed(0)
                    && mesh.vertices.len() == 4
                    && mesh.indices.len() == 6 =>
            {
                let bounds = mesh
                    .vertices
                    .iter()
                    .fold(egui::Rect::NOTHING, |rect, vertex| {
                        rect.union(egui::Rect::from_min_max(vertex.pos, vertex.pos))
                    });
                Some(bounds.intersect(paint.clip_rect))
            }
            _ => None,
        })
        .ok_or_else(|| "actual painted SDF image rectangle".into())
}

fn wheel_event(
    unit: egui::MouseWheelUnit,
    delta: [f32; 2],
    modifiers: egui::Modifiers,
) -> egui::Event {
    egui::Event::MouseWheel {
        unit,
        delta: egui::vec2(delta[0], delta[1]),
        modifiers,
    }
}

#[test]
fn repeated_sdf_wheel_uses_literal_logical_deltas_and_respects_input_gates() -> TestResult {
    let mut fixture = ready_ordinary(1.0)?;
    let before = fixture.state()?;
    let stamp = fixture
        .app
        .engine
        .lock()
        .map_err(|_| "engine")?
        .capture_stamp();
    let output = wait_image(&mut fixture)?;
    let area = image_rect(&fixture, &output)?;
    let point = area.center();
    for label in ["Horizontal position", "Vertical position"] {
        let (_, bounds) = fixture.node(label, &[egui::accesskit::Role::Slider])?;
        assert!(
            !bounds.contains(point),
            "the wheel target is real image, not {label}"
        );
    }
    let epoch = fixture.view()?["epoch"].as_u64().ok_or("view epoch")?;
    fixture.frame(vec![
        egui::Event::PointerMoved(point),
        wheel_event(
            egui::MouseWheelUnit::Line,
            [-1.0, -1.0],
            egui::Modifiers::NONE,
        ),
    ]);
    let first = fixture.view()?;
    assert_eq!(
        (first["center_x_pt"].clone(), first["center_y_pt"].clone()),
        (json!(124.0), json!(168.0)),
        "one Line(-1,-1) is +24 authored points at100%, without DPI multiplication"
    );
    assert_eq!(first["epoch"], epoch + 1);
    fixture.frame(vec![wheel_event(
        egui::MouseWheelUnit::Line,
        [-1.0, -1.0],
        egui::Modifiers::NONE,
    )]);
    let second = fixture.view()?;
    assert_eq!(
        (second["center_x_pt"].clone(), second["center_y_pt"].clone()),
        (json!(148.0), json!(192.0)),
        "ordinary continued wheel input needs no new pointer movement"
    );
    assert_eq!(second["epoch"], epoch + 2);
    fixture.frame(vec![wheel_event(
        egui::MouseWheelUnit::Point,
        [12.0, -18.0],
        egui::Modifiers::NONE,
    )]);
    let fixed = fixture.view()?;
    assert_eq!(
        (fixed["center_x_pt"].clone(), fixed["center_y_pt"].clone()),
        (json!(136.0), json!(210.0))
    );
    assert_eq!(fixed["epoch"], epoch + 3);
    fixture.frame(vec![wheel_event(
        egui::MouseWheelUnit::Point,
        [0.0, 0.0],
        egui::Modifiers::NONE,
    )]);
    assert_eq!(
        fixture.view()?,
        fixed,
        "zero wheel must not advance the camera epoch"
    );
    fixture.frame(vec![
        egui::Event::PointerMoved(egui::pos2(1.0, 1.0)),
        wheel_event(
            egui::MouseWheelUnit::Line,
            [-1.0, -1.0],
            egui::Modifiers::NONE,
        ),
    ]);
    assert_eq!(fixture.view()?, fixed, "outside image input cannot pan");
    for modifiers in [
        egui::Modifiers::SHIFT,
        egui::Modifiers::CTRL,
        egui::Modifiers::ALT,
        egui::Modifiers::COMMAND,
    ] {
        fixture.frame(vec![
            egui::Event::PointerMoved(point),
            wheel_event(egui::MouseWheelUnit::Line, [-1.0, -1.0], modifiers),
        ]);
        assert_eq!(fixture.view()?, fixed, "modified wheel is not ordinary pan");
    }
    let (zoom, _) = fixture.node(
        "Zoom",
        &[
            egui::accesskit::Role::SpinButton,
            egui::accesskit::Role::TextInput,
        ],
    )?;
    fixture.frame(vec![egui::Event::AccessKitActionRequest(
        egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::Focus,
            target: zoom,
            data: None,
        },
    )]);
    let focused = fixture
        .context
        .memory(|memory| memory.focused())
        .ok_or("actual Zoom focus")?;
    assert!(egui::TextEdit::load_state(&fixture.context, focused).is_some());
    fixture.frame(vec![
        egui::Event::PointerMoved(point),
        wheel_event(
            egui::MouseWheelUnit::Point,
            [12.0, -18.0],
            egui::Modifiers::NONE,
        ),
    ]);
    assert_eq!(
        fixture.view()?,
        fixed,
        "text focus blocks wheel despite a fresh image point"
    );
    fixture.command("ui.file", json!({"expected_revision":1,"action":"open"}))?;
    fixture.warm();
    let modal = fixture.state()?;
    assert_eq!(modal["ui_file"]["request"]["phase"], "accepted");
    fixture.frame(vec![
        egui::Event::PointerMoved(point),
        wheel_event(
            egui::MouseWheelUnit::Line,
            [-1.0, -1.0],
            egui::Modifiers::NONE,
        ),
    ]);
    assert_eq!(
        fixture.view()?,
        fixed,
        "an actual file picker blocks canvas wheel"
    );
    assert_eq!(fixture.state()?, modal);
    let request = modal["ui_file"]["request"]["id"]
        .as_u64()
        .ok_or("picker request")?;
    fixture.command(
        "ui.file",
        json!({"expected_revision":1,"action":"cancel","request_id":request}),
    )?;
    fixture.frame(vec![]);
    let mut after = fixture.state()?;
    assert!(after["ui_file"]["request"].is_null());
    assert_eq!(after["ui_file"]["epoch"], 2);
    after["ui_file"] = before["ui_file"].clone();
    assert_eq!(
        after, before,
        "only the explicitly tested picker intent may differ"
    );
    let engine = fixture.app.engine.lock().map_err(|_| "engine")?;
    assert!(engine.capture_stamp_matches(&stamp));
    assert_eq!(engine.content().sdf()?.as_bytes(), SOURCE);
    assert!(fixture.app.error.is_none(), "{:?}", fixture.app.error);
    Ok(())
}

fn fresh_capture(
    fixture: &mut Fixture,
    mut output: egui::FullOutput,
) -> TestResult<(egui::ColorImage, egui::FullOutput)> {
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut uploads = HashMap::new();
    loop {
        for (id, delta) in &output.textures_delta.set {
            if *id != egui::TextureId::Managed(0)
                && let egui::ImageData::Color(image) = &delta.image
            {
                assert!(delta.pos.is_none());
                uploads.insert(*id, image.as_ref().clone());
            }
        }
        if !fixture.app.preview.is_busy()
            && !fixture.app.executor.has_active_work()
            && let Some(id) = painted_texture(fixture, &output)
            && let Some(image) = uploads.remove(&id)
        {
            return Ok((image, output));
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "no fresh registered upload for current paint: {:?}",
                preview_refusal(&output)
            )
            .into());
        }
        std::thread::yield_now();
        output = fixture.frame(vec![]);
    }
}

fn ordinary_capture() -> TestResult<(Fixture, egui::ColorImage, egui::FullOutput)> {
    let mut fixture = Fixture::new(1160.0, 800.0, 1.0)?;
    let path = fixture.directory.path("ordinary-current-view.sdf");
    fs::write(&path, SOURCE)?;
    fixture.command("document.open_sdf", json!({"path":path}))?;
    fixture.choose("View")?;
    fixture.choose("Actual size")?;
    wait_image(&mut fixture)?;
    let (horizontal, _) = fixture.node("Horizontal position", &[egui::accesskit::Role::Slider])?;
    fixture.frame(vec![set_slider(horizontal, 100.0)]);
    let (vertical, _) = fixture.node("Vertical position", &[egui::accesskit::Role::Slider])?;
    fixture.frame(vec![set_slider(vertical, 144.0)]);
    let output = fixture.frame(vec![]);
    let (image, output) = fresh_capture(&mut fixture, output)?;
    Ok((fixture, image, output))
}

fn note_edited_source() -> TestResult<Vec<u8>> {
    assert_eq!(SOURCE.get(380..388), Some(b"ethanol\n".as_slice()));
    let expected = [
        SOURCE.get(..380).ok_or("literal NOTE prefix")?,
        b"reviewed ethanol\n",
        SOURCE.get(388..).ok_or("literal NOTE suffix")?,
    ]
    .concat();
    assert_eq!(expected.len(), 749);
    Ok(expected)
}

#[test]
fn view_dpi_and_source_aba_require_fresh_uploads_instead_of_old_equal_pixels() -> TestResult {
    let (mut fixture, first, _) = ordinary_capture()?;
    let before = fixture.state()?;
    let stamp = fixture
        .app
        .engine
        .lock()
        .map_err(|_| "engine")?
        .capture_stamp();
    let (zoom, _) = fixture.node(
        "Zoom",
        &[
            egui::accesskit::Role::SpinButton,
            egui::accesskit::Role::TextInput,
        ],
    )?;
    let changed = fixture.frame(vec![set_slider(zoom, 150.0)]);
    assert!(
        painted_texture(&fixture, &changed).is_none(),
        "old100% texture clears before painting150%"
    );
    let (magnified, _) = fresh_capture(&mut fixture, changed)?;
    assert_ne!(
        magnified.pixels, first.pixels,
        "view changes affect real registered pixels"
    );
    let (zoom, _) = fixture.node(
        "Zoom",
        &[
            egui::accesskit::Role::SpinButton,
            egui::accesskit::Role::TextInput,
        ],
    )?;
    let changed = fixture.frame(vec![set_slider(zoom, 100.0)]);
    assert!(
        painted_texture(&fixture, &changed).is_none(),
        "returning to A cannot revive the old A texture"
    );
    let (again, _) = fresh_capture(&mut fixture, changed)?;
    assert_eq!(
        again, first,
        "equal A pixels require a fresh current upload"
    );
    let view = fixture.view()?;
    assert_eq!(view["zoom_percent"], 100.0);
    assert_eq!(view["center_x_pt"], 100.0);
    assert_eq!(view["center_y_pt"], 144.0);
    fixture.dpi = 2.0;
    let changed = fixture.frame(vec![]);
    assert_eq!(changed.pixels_per_point, 2.0);
    assert!(
        painted_texture(&fixture, &changed).is_none(),
        "old1x texture cannot paint after native-pixel-scale input changes"
    );
    let (dense, output) = fresh_capture(&mut fixture, changed)?;
    let rect = image_rect(&fixture, &output)?;
    assert_eq!(
        dense.size,
        [
            (rect.width() * 2.0).round() as usize,
            (rect.height() * 2.0).round() as usize
        ]
    );
    assert!(dense.size[0] > first.size[0] && dense.size[1] > first.size[1]);
    assert_eq!(fixture.view()?, view, "DPI is not semantic zoom");
    fixture.dpi = 1.0;
    let changed = fixture.frame(vec![]);
    assert_eq!(changed.pixels_per_point, 1.0);
    assert!(painted_texture(&fixture, &changed).is_none());
    let (returned, _) = fresh_capture(&mut fixture, changed)?;
    assert_eq!(returned, first);
    assert_eq!(fixture.state()?, before);
    assert_eq!(fixture.view()?, view);
    assert!(
        fixture
            .app
            .engine
            .lock()
            .map_err(|_| "engine")?
            .capture_stamp_matches(&stamp)
    );

    // Source B changes only a literal data value, so chemistry pixels may be equal;
    // new source identities still require fresh registered work in both directions.
    let alternate = note_edited_source()?;
    let alternate_path = fixture.directory.path("source-B.sdf");
    fs::write(&alternate_path, &alternate)?;
    fixture.command("document.open_sdf", json!({"path":alternate_path}))?;
    let source_b = fixture.state()?;
    let changed = fixture.frame(vec![]);
    assert!(painted_texture(&fixture, &changed).is_none());
    let (alternate_image, _) = fresh_capture(&mut fixture, changed)?;
    assert_eq!(alternate_image, first);
    assert_eq!(fixture.state()?, source_b);
    assert_eq!(
        fixture
            .app
            .engine
            .lock()
            .map_err(|_| "engine")?
            .content()
            .sdf()?
            .as_bytes(),
        alternate
    );
    assert!(
        !fixture
            .app
            .engine
            .lock()
            .map_err(|_| "engine")?
            .capture_stamp_matches(&stamp)
    );
    let original_path = before["file"]["path"]
        .as_str()
        .ok_or("original associated path")?;
    fixture.command("document.open_sdf", json!({"path":original_path}))?;
    let source_a = fixture.state()?;
    let changed = fixture.frame(vec![]);
    assert!(painted_texture(&fixture, &changed).is_none());
    let (original_image, _) = fresh_capture(&mut fixture, changed)?;
    assert_eq!(original_image, first);
    assert_eq!(fixture.state()?, source_a);
    assert_eq!(source_a["revision"], 3);
    assert_eq!(source_a["undo_depth"], 0);
    assert_eq!(source_a["file"]["dirty"], false);
    assert_eq!(
        fixture
            .app
            .engine
            .lock()
            .map_err(|_| "engine")?
            .content()
            .sdf()?
            .as_bytes(),
        SOURCE
    );
    assert!(
        !fixture
            .app
            .engine
            .lock()
            .map_err(|_| "engine")?
            .capture_stamp_matches(&stamp)
    );
    assert_eq!(fs::read(original_path)?, SOURCE);
    assert_eq!(fs::read(alternate_path)?, alternate);
    Ok(())
}

#[path = "../../../xtask/oracles/sdf_test_support.rs"]
#[allow(
    dead_code,
    clippy::duplicate_mod,
    reason = "reuse unchanged independent per-MOL consumers"
)]
mod view_sdf_support;

fn field_text(fixture: &Fixture, label: &str) -> TestResult<String> {
    let (id, _) = fixture.node(
        label,
        &[
            egui::accesskit::Role::TextInput,
            egui::accesskit::Role::MultilineTextInput,
        ],
    )?;
    let mut value = String::new();
    let mut stack = vec![id];
    while let Some(id) = stack.pop() {
        let node = fixture.nodes.get(&id).ok_or("current field text")?;
        if node.role() == egui::accesskit::Role::InlineTextBox {
            value.push_str(node.value().ok_or("actual inline value")?);
        }
        stack.extend(node.children().iter().rev().copied());
    }
    Ok(value)
}

#[test]
fn explicit_view_survives_note_apply_fresh_preview_and_actual_file_save_reopen() -> TestResult {
    let (mut fixture, _, _) = ordinary_capture()?;
    let path = fixture.state()?["file"]["path"]
        .as_str()
        .ok_or("associated SDF path")?
        .to_owned();
    let expected = note_edited_source()?;
    fixture.choose("Records")?;
    let output = fixture.frame(vec![]);
    let (before_image, _) = fresh_capture(&mut fixture, output)?;
    fixture.choose("Inspect record 1 fields")?;
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        fixture.frame(vec![]);
        if fixture
            .node("Field 1: NOTE", &[egui::accesskit::Role::Button])
            .is_ok()
        {
            break;
        }
        if Instant::now() >= deadline {
            return Err("real NOTE inspection did not finish".into());
        }
        std::thread::yield_now();
    }
    fixture.choose("Field 1: NOTE")?;
    assert_eq!(
        field_text(&fixture, "Record 1 field 1 value")?,
        "ethanol\nvertical O bond"
    );
    fixture.choose("Edit record 1 field 1")?;
    let label = "Record 1 field 1 draft";
    let (id, _) = fixture.node(
        label,
        &[
            egui::accesskit::Role::TextInput,
            egui::accesskit::Role::MultilineTextInput,
        ],
    )?;
    fixture.frame(vec![egui::Event::AccessKitActionRequest(
        egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::Focus,
            target: id,
            data: None,
        },
    )]);
    fixture.frame(vec![
        super::key(egui::Key::A, true, egui::Modifiers::COMMAND),
        super::key(egui::Key::A, false, egui::Modifiers::COMMAND),
        egui::Event::Text("reviewed ethanol\nvertical O bond".into()),
    ]);
    assert_eq!(
        field_text(&fixture, label)?,
        "reviewed ethanol\nvertical O bond"
    );
    assert_eq!(
        fixture
            .app
            .engine
            .lock()
            .map_err(|_| "engine")?
            .content()
            .sdf()?
            .as_bytes(),
        SOURCE
    );
    fixture.choose("Apply field value")?;
    let output = fixture.frame(vec![]);
    let (after_image, _) = fresh_capture(&mut fixture, output)?;
    assert_eq!(
        after_image, before_image,
        "metadata edits preserve geometry/H depiction but still require a fresh source-bound capture"
    );
    let applied = fixture.state()?;
    assert_eq!(applied["revision"], 2);
    assert_eq!(applied["undo_depth"], 1);
    assert_eq!(applied["source"]["byte_length"], 740);
    assert_eq!(applied["source"]["modified"], true);
    assert_eq!(applied["file"]["dirty"], true);
    assert_eq!(fixture.view()?["zoom_percent"], 100.0);
    assert_eq!(fixture.view()?["center_x_pt"], 100.0);
    assert_eq!(fixture.view()?["center_y_pt"], 144.0);
    assert_eq!(fs::read(&path)?, SOURCE);
    fixture.frame(vec![
        super::key(egui::Key::S, true, egui::Modifiers::COMMAND),
        super::key(egui::Key::S, false, egui::Modifiers::COMMAND),
    ]);
    super::finish_actual_file_workflow(&mut fixture)?;
    let bytes = fs::read(&path)?;
    assert_eq!(bytes, expected);
    assert_eq!(bytes.get(..380), SOURCE.get(..380));
    assert_eq!(bytes.get(397..), SOURCE.get(388..));
    assert_eq!(bytes.get(419..), SOURCE.get(410..));
    let first = view_sdf_support::describe_source(
        "v3000_oracle.py",
        bytes.get(..371).ok_or("unchanged V3000 MOL")?,
    )?;
    let second = view_sdf_support::describe_source(
        "mol_oracle.py",
        bytes.get(419..722).ok_or("unchanged V2000 MOL")?,
    )?;
    assert_eq!(first["atoms"].as_array().ok_or("V3000 atoms")?.len(), 3);
    assert_eq!(first["atoms"][2]["element"], 8);
    assert_eq!(second["smiles"], "CN");
    assert_eq!(second["formula"], "CH5N");
    assert_eq!(fixture.state()?["file"]["dirty"], false);
    fixture.command("document.open_sdf", json!({"path":path}))?;
    let output = fixture.frame(vec![]);
    assert!(painted_texture(&fixture, &output).is_none());
    let (reopened, _) = fresh_capture(&mut fixture, output)?;
    assert_eq!(reopened, before_image);
    let state = fixture.state()?;
    assert_eq!(state["source"]["byte_length"], 749);
    assert_eq!(state["source"]["modified"], false);
    assert_eq!(state["file"]["dirty"], false);
    assert_eq!(state["undo_depth"], 0);
    assert_eq!(
        fixture
            .app
            .engine
            .lock()
            .map_err(|_| "engine")?
            .content()
            .sdf()?
            .as_bytes(),
        expected
    );
    Ok(())
}
