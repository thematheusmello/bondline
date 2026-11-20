//! Directed CDX bond data retains source identity without granting drawing authority.
#[path = "../../../xtask/oracles/registry_fixture_support.rs"]
mod registry_fixture;
use bondline_commands::{Command, Engine};
use serde_json::{Value, json};
use std::{fs, sync::Arc};
#[path = "../../../xtask/oracles/cdx_bond_support.rs"]
#[allow(
    dead_code,
    reason = "independent frozen source/model/file expectations"
)]
mod f;
#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "existing isolated file directory")]
mod files;
type TestResult = f::Result;

fn exact(e: &Engine, name: &str) -> TestResult {
    assert_eq!(serde_json::to_value(e.document()?)?, f::model(name)?);
    let stored = e.stored_document()?;
    assert!(!stored.is_fully_mapped());
    assert!(!stored.has_display_projection());
    assert_eq!(stored.to_cdx()?.as_ref(), f::source(name));
    Ok(())
}

#[test]
fn directed_bonds_survive_registered_unicode_files_with_all_authoring_guards() -> TestResult {
    let commands = bondline_commands::registry();
    assert_eq!(commands.len(), 55);
    assert_eq!(commands[49].name, "ui.caption");
    assert_eq!(
        serde_json::to_value(&commands[..49])?,
        serde_json::from_str::<Value>(&registry_fixture::with_cdx_caption_edit(f::METADATA49))?
    );
    for name in [
        "ethene",
        "ethyne",
        "manual_right",
        "reversed_right",
        "embedded_right",
    ] {
        let dir = files::FileTestDirectory::new()?;
        let input = dir.path("bond-原-λ.cdx");
        let output = dir.path("bond-保存-λ.cdx");
        let refused = dir.path("unrelated.dat");
        fs::write(&input, f::source(name))?;
        fs::write(&refused, b"preserve unrelated bytes")?;
        let mut e = Engine::default();
        e.dispatch("document.open_cdx", json!({"path":input}))?;
        exact(&e, name)?;
        for (command, params) in [
            (
                "bond.set_order",
                json!({"expected_revision":1,"page_id":1,"fragment_id":1,"bond_id":1,"order":"double","hydrogen_policy":"rectify_neutral_carbon"}),
            ),
            (
                "atom.set_label",
                json!({"expected_revision":1,"page_id":1,"fragment_id":1,"atom_id":1,"element":6,"explicit_hydrogens":2}),
            ),
            (
                "ui.selection",
                json!({"action":"set","expected_revision":1,"page_id":1,"atom_ids":[1]}),
            ),
            (
                "render.capture",
                json!({"page_id":1,"width_px":1088,"height_px":1088,"backend":"cpu"}),
            ),
            (
                "render.capture",
                json!({"page_id":1,"width_px":1088,"height_px":1088,"backend":"gpu"}),
            ),
            (
                "document.save_cdxml",
                json!({"path":refused,"overwrite":true}),
            ),
            (
                "document.save_mol",
                json!({"path":refused,"overwrite":true}),
            ),
        ] {
            let before = serde_json::to_value(e.state())?;
            let model = e.stored_document()?.model_snapshot();
            let stamp = e.capture_stamp();
            assert!(e.dispatch(command, params).is_err(), "{command}: {name}");
            assert_eq!(serde_json::to_value(e.state())?, before);
            assert!(e.capture_stamp_matches(&stamp));
            assert!(Arc::ptr_eq(&model, &e.stored_document()?.model_snapshot()));
            assert_eq!(fs::read(&refused)?, b"preserve unrelated bytes");
        }
        let original = e.stored_document()?.model_snapshot();
        assert!(
            !e.dispatch(
                "page.set_size",
                json!({"page_id":1,"width_pt":210.0*72.0/25.4,"height_pt":297.0*72.0/25.4})
            )?
            .changed
        );
        assert!(Arc::ptr_eq(
            &original,
            &e.stored_document()?.model_snapshot()
        ));
        e.dispatch(
            "page.set_size",
            json!({"page_id":1,"width_pt":612,"height_pt":792}),
        )?;
        assert!(e.state().file.dirty);
        assert_eq!(e.state().undo_depth, 1);
        assert!(
            e.dispatch(
                "document.save_cdx",
                json!({"path":refused,"overwrite":true})
            )
            .is_err()
        );
        assert_eq!(fs::read(&refused)?, b"preserve unrelated bytes");
        e.dispatch("history.undo", json!({}))?;
        exact(&e, name)?;
        e.dispatch("history.redo", json!({}))?;
        assert!(e.stored_document()?.to_cdx().is_err());
        e.dispatch("history.undo", json!({}))?;
        e.dispatch("document.save_cdx", json!({"path":output}))?;
        let actual = fs::read(&output)?;
        assert_eq!(actual, f::source(name));
        assert_eq!(f::read(&actual)?, f::facts(name));
        e.dispatch("document.open_cdx", json!({"path":output}))?;
        exact(&e, name)?;
        assert_eq!((e.state().undo_depth, e.state().redo_depth), (0, 0));
        assert!(!e.state().file.dirty);
        assert_eq!(fs::read(input)?, f::source(name));
    }
    Ok(())
}

#[test]
fn equal_side_automatic_manual_sources_do_not_share_save_or_capture_authority() -> TestResult {
    assert_eq!(f::model("auto_right")?, f::model("manual_right")?);
    assert_ne!(f::source("auto_right"), f::source("manual_right"));
    let dir = files::FileTestDirectory::new()?;
    let a = dir.path("automatic-原.cdx");
    let b = dir.path("manual-λ.cdx");
    let output = dir.path("captured-保存.cdx");
    fs::write(&a, f::source("auto_right"))?;
    fs::write(&b, f::source("manual_right"))?;
    let mut e = Engine::default();
    e.dispatch("document.open_cdx", json!({"path":a}))?;
    exact(&e, "auto_right")?;
    let capture = e.capture_stamp();
    let file = e.ui_file_context_stamp();
    let save = e
        .prepare(Command::parse("document.save_cdx", json!({"path":output}))?)?
        .complete()?;
    e.dispatch("document.open_cdx", json!({"path":b}))?;
    exact(&e, "manual_right")?;
    assert!(!e.capture_stamp_matches(&capture));
    assert!(!e.ui_file_context_stamp_matches(&file));
    let before = serde_json::to_value(e.state())?;
    e.finish(save)?;
    assert_eq!(serde_json::to_value(e.state())?, before);
    let actual = fs::read(output)?;
    assert_eq!(actual, f::source("auto_right"));
    assert_eq!(f::read(&actual)?, f::facts("auto_right"));
    assert_eq!(fs::read(b)?, f::source("manual_right"));
    Ok(())
}
