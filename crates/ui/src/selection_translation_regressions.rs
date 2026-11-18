//! Child of view_tests::selection_translation; original fixture and helpers only.
use super::*;

fn selection_guard_axis(
    f: &Fixture,
    shown: &egui::FullOutput,
    point: [f64; 2],
) -> TestResult<bool> {
    let area = canvas(shown)?;
    let state = f.state()?;
    let size = &state["document"]["pages"][0]["size"];
    let view = f.view()?;
    assert_eq!(view["zoom_percent"], 100.0);
    let center = match (view["center_x_pt"].as_f64(), view["center_y_pt"].as_f64()) {
        (Some(x), Some(y)) => Some([x, y]),
        _ => None,
    };
    let at = projected(
        area,
        [
            size["width_pt"].as_f64().ok_or("width")?,
            size["height_pt"].as_f64().ok_or("height")?,
        ],
        100.0,
        center,
        point,
    );
    assert!(area.contains(at));
    Ok(shown.shapes.iter().any(|paint| {
        let egui::Shape::Path(path) = &paint.shape else {
            return false;
        };
        if path.fill != egui::Color32::BLACK
            || path.points.len() != 4
            || !paint.clip_rect.contains(at)
        {
            return false;
        }
        let mut positive = false;
        let mut negative = false;
        for i in 0..4 {
            let a = path.points[i];
            let b = path.points[(i + 1) % 4];
            let cross = (b.x - a.x) * (at.y - a.y) - (b.y - a.y) * (at.x - a.x);
            positive |= cross > 0.0;
            negative |= cross < 0.0;
        }
        !(positive && negative)
    }))
}
fn selection_guard_arm(f: &mut Fixture) -> TestResult<(Pos2, Pos2, Value, egui::FullOutput)> {
    selection_quiet_clicks(f)?;
    let a = selection_point(f, [180.0, 320.0])?;
    let b = selection_point(f, [204.0, 308.0])?;
    assert!(
        a.distance(b) > 6.0,
        "real path exceeds the SDK click threshold"
    );
    let before = f.state()?;
    f.frame(vec![
        Event::PointerMoved(a),
        selection_pointer(a, true, false),
    ]);
    let preview = f.frame(vec![Event::PointerMoved(b)]);
    assert_eq!(
        f.state()?,
        before,
        "a demonstrated preview is not an authored edit"
    );
    assert!(f.app.drawing.selection.is_dragging());
    assert!(
        selection_guard_axis(f, &preview, [244.0, 308.0])?,
        "the moved C–O bond must actually be painted"
    );
    assert!(
        !selection_guard_axis(f, &preview, [220.0, 320.0])?,
        "the old C–O bond axis must be absent during preview"
    );
    Ok((a, b, before, preview))
}
fn selection_guard_unchanged(
    f: &Fixture,
    before: &Value,
    stamp: &bondline_commands::CaptureStamp,
    model: &std::sync::Arc<bondline_model::Document>,
    path: &Path,
    raw: &[u8],
) -> TestResult {
    assert_eq!(
        f.state()?,
        *before,
        "all ordinary state/selection/IDs/history/source must remain exact"
    );
    selection_expect(f, &[1, 2])?;
    let engine = f.app.engine.lock().map_err(|_| "engine")?;
    assert_eq!(&engine.capture_stamp(), stamp);
    assert!(std::sync::Arc::ptr_eq(
        model,
        &engine.stored_document()?.model_snapshot()
    ));
    assert_eq!(fs::read(path)?, raw);
    Ok(())
}
fn selection_guard_commit(
    f: &mut Fixture,
    before: &Value,
    end: Pos2,
    path: &Path,
    raw: &[u8],
) -> TestResult {
    f.frame(vec![selection_pointer(end, false, false)]);
    let expected =
        selection_expected_positions(&before["document"], &[(1, 204.0, 308.0), (2, 284.0, 308.0)])?;
    let after = f.state()?;
    assert_eq!(after["document"], expected);
    assert_eq!(
        after["revision"],
        before["revision"].as_u64().ok_or("revision")? + 1
    );
    assert_eq!(
        after["undo_depth"],
        before["undo_depth"].as_u64().ok_or("history")? + 1
    );
    assert_eq!(after["redo_depth"], 0);
    selection_expect(f, &[1, 2])?;
    assert_eq!(after["file"]["dirty"], true);
    assert_eq!(after["source"]["modified"], true);
    assert_eq!(fs::read(path)?, raw);
    Ok(())
}

#[test]
fn hidden_atom_and_settings_open_cancel_cannot_restore_an_armed_selection_preview() -> TestResult {
    // A no-loss run executes the SAME proven drag and must commit, so cancellation is not vacuous.
    for modal in ["none", "atom", "settings"] {
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        let path = selection_seed(&mut f)?;
        let raw = fs::read(&path)?;
        selection_choose(&mut f, "Marquee selection")?;
        selection_click_at(&mut f, [220.0, 320.0], false)?;
        selection_expect(&f, &[1, 2])?;
        let (_, end, before, _) = selection_guard_arm(&mut f)?;
        let (stamp, model, tool, view, mode) = {
            let engine = f.app.engine.lock().map_err(|_| "engine")?;
            (
                engine.capture_stamp(),
                engine.stored_document()?.model_snapshot(),
                engine.bond_tool(),
                engine.view(),
                engine.effective_tool(),
            )
        };
        if modal != "none" {
            let (command, field, params) = if modal == "atom" {
                (
                    "ui.atom_label",
                    "ui_atom_label",
                    json!({"action":"open",
                "expected_revision":before["revision"],"page_id":1,"fragment_id":1,"atom_id":1}),
                )
            } else {
                (
                    "ui.drawing_settings",
                    "ui_drawing_settings",
                    json!({"action":"open","expected_revision":before["revision"]}),
                )
            };
            // No App frame or tool/view/source change occurs during this real hidden modal cycle.
            let opened = f.command(command, params)?;
            let id = opened[field]["id"].as_u64().ok_or("real modal request")?;
            assert_eq!(opened[field]["current"], true);
            f.command(
                command,
                json!({"action":"cancel","expected_revision":before["revision"],"request_id":id}),
            )?;
            assert!(f.state()?[field].is_null());
        }
        {
            let engine = f.app.engine.lock().map_err(|_| "engine")?;
            assert_eq!(engine.bond_tool(), tool);
            assert_eq!(engine.view(), view);
            assert_eq!(engine.effective_tool(), mode);
        }
        selection_guard_unchanged(&f, &before, &stamp, &model, &path, &raw)?;
        let after = f.frame(vec![]);
        if modal == "none" {
            assert!(selection_guard_axis(&f, &after, [244.0, 308.0])?);
            selection_guard_commit(&mut f, &before, end, &path, &raw)?;
        } else {
            assert!(
                !selection_guard_axis(&f, &after, [244.0, 308.0])?,
                "hidden {modal} cycle must remove the old preview"
            );
            assert!(selection_guard_axis(&f, &after, [220.0, 320.0])?);
            assert!(!f.app.drawing.selection.is_dragging());
            f.frame(vec![
                Event::PointerMoved(end),
                selection_pointer(end, false, false),
            ]);
            selection_guard_unchanged(&f, &before, &stamp, &model, &path, &raw)?;
            let (_, fresh_end, fresh, _) = selection_guard_arm(&mut f)?;
            selection_guard_commit(&mut f, &fresh, fresh_end, &path, &raw)?;
        }
    }
    Ok(())
}

#[test]
fn an_actual_zero_surface_frame_cancels_selection_before_identical_viewport_and_release()
-> TestResult {
    for lose_surface in [false, true] {
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        let path = selection_seed(&mut f)?;
        let raw = fs::read(&path)?;
        selection_choose(&mut f, "Marquee selection")?;
        selection_click_at(&mut f, [220.0, 320.0], false)?;
        selection_expect(&f, &[1, 2])?;
        let (_, end, before, preview) = selection_guard_arm(&mut f)?;
        let area = canvas(&preview)?;
        let view = f.view()?;
        let (stamp, model) = {
            let engine = f.app.engine.lock().map_err(|_| "engine")?;
            (
                engine.capture_stamp(),
                engine.stored_document()?.model_snapshot(),
            )
        };
        let width = f.width;
        if lose_surface {
            f.width = 0.0;
        }
        let missing = f.frame(vec![]);
        if lose_surface {
            assert_eq!(
                f.context.screen_rect().width(),
                0.0,
                "the SDK must receive an actual zero viewport"
            );
            assert!(
                canvas(&missing).is_err(),
                "the lost frame must have no drawing surface"
            );
        } else {
            assert_eq!(canvas(&missing)?, area);
            assert!(selection_guard_axis(&f, &missing, [244.0, 308.0])?);
        }
        selection_guard_unchanged(&f, &before, &stamp, &model, &path, &raw)?;
        assert_eq!(f.view()?, view);
        f.width = width;
        let restored = f.frame(vec![]);
        assert_eq!(
            canvas(&restored)?,
            area,
            "restore the identical surface, not a different hover qualifier"
        );
        assert_eq!(f.view()?, view);
        selection_guard_unchanged(&f, &before, &stamp, &model, &path, &raw)?;
        if lose_surface {
            assert!(
                !selection_guard_axis(&f, &restored, [244.0, 308.0])?,
                "surface loss cannot revive the moved preview"
            );
            assert!(selection_guard_axis(&f, &restored, [220.0, 320.0])?);
            assert!(!f.app.drawing.selection.is_dragging());
            f.frame(vec![
                Event::PointerMoved(end),
                selection_pointer(end, false, false),
            ]);
            selection_guard_unchanged(&f, &before, &stamp, &model, &path, &raw)?;
            let (_, fresh_end, fresh, _) = selection_guard_arm(&mut f)?;
            selection_guard_commit(&mut f, &fresh, fresh_end, &path, &raw)?;
        } else {
            selection_guard_commit(&mut f, &before, end, &path, &raw)?;
        }
    }
    Ok(())
}
