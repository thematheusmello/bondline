//! Child of native_drawing_settings_tests.rs; uses its existing actual-App helpers.
use super::*;

#[test]
fn actual_document_settings_has_a_third_field_without_authored_defaults() -> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    settings_seed(&mut f)?;
    let before = settings_state(&f)?;
    settings_open(&mut f)?;
    f.node("Line width (pt)", &[Role::TextInput])?;
    assert_eq!(settings_value(&f, "Line width (pt)")?, "");
    assert_eq!(
        f.state()?["ui_drawing_settings"].get("line_width_pt"),
        Some(&Value::Null)
    );
    assert_eq!(settings_state(&f)?, before);
    f.choose("Cancel")?;
    settings_closed(&mut f)?;
    assert_eq!(settings_state(&f)?, before);
    Ok(())
}

#[test]
fn third_field_obeys_raw_apply_prefix_and_modal_suffix_fences() -> TestResult {
    for early in [true, false] {
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        let path = settings_seed(&mut f)?;
        let raw = fs::read(&path)?;
        let before = settings_state(&f)?;
        settings_open(&mut f)?;
        f.node("Line width (pt)", &[Role::TextInput])?;
        settings_set(&mut f, "Bond length (pt)", "24.5")?;
        settings_set(&mut f, "Chain angle (degrees)", "120.5")?;
        settings_set(&mut f, "Line width (pt)", "2")?;
        assert!(settings_apply_enabled(&f)?);
        let width = f.node("Line width (pt)", &[Role::TextInput])?.0;
        let angle = f.node("Chain angle (degrees)", &[Role::TextInput])?.0;
        let apply = f.node("Apply", &[Role::Button])?.0;
        let edit = settings_action(
            width,
            Action::SetValue,
            Some(ActionData::Value("0.5".into())),
        );
        let mut packet = Vec::new();
        if early {
            packet.push(edit.clone());
        }
        packet.push(settings_action(apply, Action::Default, None));
        if !early {
            packet.push(edit);
        }
        packet.push(settings_action(
            angle,
            Action::SetValue,
            Some(ActionData::Value("90".into())),
        ));
        packet.extend([
            key(Key::Z, true, Modifiers::COMMAND),
            key(Key::Z, false, Modifiers::COMMAND),
            key(Key::S, true, Modifiers::COMMAND),
            key(Key::S, false, Modifiers::COMMAND),
        ]);
        f.frame(packet);
        settings_closed(&mut f)?;
        let after = settings_state(&f)?;
        let mut expected = settings_document(&before["document"], Some(24.5), Some(120.5))?;
        expected["drawing_style"] = json!({"line_width_pt":if early {0.5}else{2.0}});
        assert_eq!(after["document"], expected, "early={early}");
        assert_eq!(
            after["revision"].as_u64(),
            Some(before["revision"].as_u64().ok_or("revision")? + 1)
        );
        assert_eq!(
            (after["undo_depth"].clone(), after["redo_depth"].clone()),
            (json!(1), json!(0))
        );
        assert_eq!(after["file"]["dirty"], true);
        assert_eq!(after["source"]["modified"], true);
        assert_eq!(after["ui_file"], before["ui_file"]);
        assert_eq!(
            fs::read(&path)?,
            raw,
            "later Save cannot fall through dismissal"
        );
        assert!(!f.app.files.is_busy() && !f.app.executor.has_active_work());
        settings_shortcut(&mut f, Key::Z, false);
        assert_eq!(settings_state(&f)?["document"], before["document"]);
        settings_shortcut(&mut f, Key::Z, true);
        assert_eq!(settings_state(&f)?["document"], expected);
    }
    Ok(())
}
#[test]
fn third_field_quota_stale_draft_and_exact_cancel_cannot_author_or_save() -> TestResult {
    for paste in [false, true] {
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        let path = settings_seed(&mut f)?;
        let raw = fs::read(&path)?;
        let before = settings_state(&f)?;
        settings_open(&mut f)?;
        f.node("Line width (pt)", &[Role::TextInput])?;
        settings_set(&mut f, "Bond length (pt)", "24.5")?;
        settings_set(&mut f, "Chain angle (degrees)", "120.5")?;
        let maximum = format!("{}2", "0".repeat(383));
        let oversized = format!("{}2", "0".repeat(384));
        assert_eq!((maximum.len(), oversized.len()), (384, 385));
        settings_set(&mut f, "Line width (pt)", &maximum)?;
        assert!(settings_apply_enabled(&f)?);
        if paste {
            settings_focus_all(&mut f, "Line width (pt)")?;
        }
        let field = f.node("Line width (pt)", &[Role::TextInput])?.0;
        let apply = f.node("Apply", &[Role::Button])?.0;
        let edit = if paste {
            Event::Paste(oversized)
        } else {
            settings_action(
                field,
                Action::SetValue,
                Some(ActionData::Value(oversized.into())),
            )
        };
        f.frame(vec![edit, settings_action(apply, Action::Default, None)]);
        assert_eq!(
            settings_state(&f)?,
            before,
            "a rejected prefix cannot Apply the old valid triple"
        );
        assert_eq!(settings_value(&f, "Line width (pt)")?, maximum);
        settings_set(&mut f, "Line width (pt)", "2")?;
        f.command("ui.view", json!({"action":"fit"}))?;
        f.command("ui.view", json!({"action":"actual"}))?;
        settings_wait(&mut f, |f, _| {
            settings_apply_enabled(f).is_ok_and(|yes| !yes)
        })?;
        assert_eq!(settings_value(&f, "Line width (pt)")?, "2");
        assert_eq!(settings_value(&f, "Bond length (pt)")?, "24.5");
        assert_eq!(settings_value(&f, "Chain angle (degrees)")?, "120.5");
        let stale = settings_state(&f)?;
        let id = f.state()?["ui_drawing_settings"]["id"]
            .as_u64()
            .ok_or("request")?;
        f.command(
            "ui.drawing_settings",
            json!({"action":"cancel",
            "expected_revision":f.state()?["revision"],"request_id":id}),
        )?;
        settings_closed(&mut f)?;
        assert_eq!(settings_state(&f)?, stale);
        assert_eq!(fs::read(&path)?, raw);
        assert!(!f.app.files.is_busy() && !f.app.executor.has_active_work());
    }
    Ok(())
}
