use super::*;

#[test]
fn document_settings_has_a_fourth_spacing_field_without_authoring_defaults() -> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    settings_seed(&mut f)?;
    let before = settings_state(&f)?;
    settings_open(&mut f)?;
    f.node("Bond spacing (%)", &[Role::TextInput])?;
    assert_eq!(settings_value(&f, "Bond spacing (%)")?, "");
    assert_eq!(settings_state(&f)?, before);
    f.choose("Cancel")?;
    settings_closed(&mut f)?;
    assert_eq!(settings_state(&f)?, before);
    Ok(())
}

// Additive actual-App acceptance; the initial fourth-field declaration is exact.
#[test]
fn fourth_spacing_field_obeys_apply_prefix_and_retirement_suffix_fences() -> TestResult {
    for early in [true, false] {
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        let path = settings_seed(&mut f)?;
        let raw = fs::read(&path)?;
        let before = settings_state(&f)?;
        settings_open(&mut f)?;
        f.node("Bond spacing (%)", &[Role::TextInput])?;
        settings_set(&mut f, "Bond length (pt)", "40")?;
        settings_set(&mut f, "Chain angle (degrees)", "120")?;
        settings_set(&mut f, "Line width (pt)", "0.6")?;
        settings_set(&mut f, "Bond spacing (%)", "18")?;
        assert!(settings_apply_enabled(&f)?);
        let spacing = f.node("Bond spacing (%)", &[Role::TextInput])?.0;
        let width = f.node("Line width (pt)", &[Role::TextInput])?.0;
        let apply = f.node("Apply", &[Role::Button])?.0;
        let edit = settings_action(
            spacing,
            Action::SetValue,
            Some(ActionData::Value("30".into())),
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
            width,
            Action::SetValue,
            Some(ActionData::Value("2".into())),
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
        let mut expected = settings_document(&before["document"], Some(40.0), Some(120.0))?;
        expected["drawing_style"] =
            json!({"line_width_pt":0.6,"bond_spacing_percent":if early {30.0}else{18.0}});
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
            "post-Apply Save must not reach the canvas/file route"
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
fn spacing_quota_stale_draft_and_exact_remote_cancel_preserve_source_and_history() -> TestResult {
    for paste in [false, true] {
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        let path = settings_seed(&mut f)?;
        let raw = fs::read(&path)?;
        let before = settings_state(&f)?;
        settings_open(&mut f)?;
        f.node("Bond spacing (%)", &[Role::TextInput])?;
        settings_set(&mut f, "Bond length (pt)", "40")?;
        settings_set(&mut f, "Chain angle (degrees)", "120")?;
        settings_set(&mut f, "Line width (pt)", "0.6")?;
        let maximum = format!("{}18", "0".repeat(382));
        let oversized = format!("{}18", "0".repeat(383));
        assert_eq!((maximum.len(), oversized.len()), (384, 385));
        settings_set(&mut f, "Bond spacing (%)", &maximum)?;
        assert!(settings_apply_enabled(&f)?);
        if paste {
            settings_focus_all(&mut f, "Bond spacing (%)")?;
        }
        let field = f.node("Bond spacing (%)", &[Role::TextInput])?.0;
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
            "rejected prefix cannot apply the prior valid fourth value"
        );
        assert_eq!(settings_value(&f, "Bond spacing (%)")?, maximum);
        settings_set(&mut f, "Bond spacing (%)", "18")?;
        f.command("ui.view", json!({"action":"fit"}))?;
        f.command("ui.view", json!({"action":"actual"}))?;
        settings_wait(&mut f, |f, _| {
            settings_apply_enabled(f).is_ok_and(|yes| !yes)
        })?;
        assert_eq!(settings_value(&f, "Bond spacing (%)")?, "18");
        assert_eq!(settings_value(&f, "Line width (pt)")?, "0.6");
        assert_eq!(settings_value(&f, "Bond length (pt)")?, "40");
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
