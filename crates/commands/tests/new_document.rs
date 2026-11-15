//! Independent New-document acceptance through the existing shared dispatcher.
use bondline_commands::Engine;
use serde_json::json;
use std::fs;

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse only the existing isolated test directory")]
mod support;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const SDF: &[u8] = include_bytes!("../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf");

#[test]
fn new_from_complete_sdf_returns_a_clean_source_free_drawing_without_writing_the_source()
-> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("whole-source.sdf");
    fs::write(&input, SDF)?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_sdf", json!({"path":input}))?;
    let result = engine.dispatch("document.new", json!({"expected_revision":1}))?;
    assert!(result.changed);
    assert_eq!(result.revision, 2);
    assert!(result.source.is_none() && result.file.path.is_none());
    assert!(!result.file.dirty);
    assert_eq!(engine.document()?, &bondline_model::Document::default());
    assert_eq!(fs::read(input)?, SDF);
    Ok(())
}

const SMILES: &[u8] = b"CC first\r\n\n \xffignored\n[13CH3][OH] \0\xffPRIVATE\nO last";
const NATIVE: &[u8] = include_bytes!("../../io/tests/fixtures/bondline-v0.cdxml");
const CDX: &[u8] = include_bytes!("../../io/tests/fixtures/cdx/metadata-header.cdx");
const MOL: &[u8] = include_bytes!("../../io/tests/fixtures/mol/ethane-v2000.mol");
const V3000: &[u8] = include_bytes!("../../io/tests/fixtures/v3000/noncontiguous-cc.mol");

fn new_params(revision: u64, discard: bool) -> serde_json::Value {
    json!({"expected_revision":revision,"discard_unsaved":discard})
}

fn pair(engine: &mut Engine, index: usize) -> TestResult<bondline_commands::CommandResult> {
    Ok(engine.dispatch(
        "bond.create",
        json!({
            "page_id":1,"begin_x_pt":100.0,"begin_y_pt":200.0+100.0*index as f64,
            "end_x_pt":140.0,"end_y_pt":200.0+100.0*index as f64
        }),
    )?)
}

fn assert_blank(engine: &Engine, revision: u64, host: bool) -> TestResult {
    let state = engine.state();
    assert_eq!(state.revision, revision);
    assert_eq!((state.undo_depth, state.redo_depth), (0, 0));
    assert!(state.source.is_none());
    assert!(state.file.path.is_none() && !state.file.dirty);
    assert_eq!(state.file.format, bondline_commands::FileFormat::Cdxml);
    assert_eq!(state.ui_file.available, host);
    assert!(state.ui_file.request.is_none());
    assert_eq!(state.close, bondline_commands::CloseState::Open);
    assert_eq!(engine.document()?, &bondline_model::Document::default());
    let page = engine.document()?.pages().first().ok_or("default page")?;
    assert_eq!(page.id().get(), 1);
    assert_eq!(page.size().width_pt(), 210.0 * 72.0 / 25.4);
    assert_eq!(page.size().height_pt(), 297.0 * 72.0 / 25.4);
    assert!(page.fragments().is_empty());
    assert_eq!(engine.view_page_size()?, bondline_model::PageSize::A4);
    let wire = serde_json::to_value(state)?;
    for key in [
        "collection",
        "smiles_file",
        "sdf_record",
        "smiles_record",
        "file_operation",
    ] {
        assert!(wire.get(key).is_none(), "stale {key} must not survive New");
    }
    Ok(())
}

fn assert_local_ethane(engine: &Engine) -> TestResult {
    let page = engine.document()?.pages().first().ok_or("page")?;
    assert_eq!(page.fragments().len(), 1);
    let fragment = page.fragments().first().ok_or("fresh fragment")?;
    assert_eq!(fragment.id().get(), 1);
    assert_eq!(
        fragment
            .atoms()
            .iter()
            .map(|atom| atom.id.get())
            .collect::<Vec<_>>(),
        [1, 2]
    );
    let bond = fragment.bonds().first().ok_or("bond")?;
    assert_eq!(fragment.bonds().len(), 1);
    assert_eq!((bond.id.get(), bond.begin.get(), bond.end.get()), (1, 1, 2));
    assert_eq!(bond.order, bondline_model::BondOrder::Single);
    for atom in fragment.atoms() {
        assert_eq!(atom.element.get(), 6);
        assert_eq!(atom.explicit_hydrogens, Some(3));
        assert_eq!(atom.charge, 0);
    }
    Ok(())
}

#[test]
fn new_schema_adds_only_the_thirtieth_command_and_a_qualified_file_binding() -> TestResult {
    let names = bondline_commands::registry()
        .into_iter()
        .map(|info| info.name)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        names,
        [
            "document.inspect",
            "document.close",
            "document.new",
            "ui.file",
            "ui.controls",
            "ui.bond_tool",
            "ui.view",
            "document.open_cdxml",
            "document.save_cdxml",
            "document.open_cdx",
            "document.save_cdx",
            "document.open_mol",
            "document.save_mol",
            "document.open_sdf",
            "document.save_sdf",
            "document.open_smiles",
            "document.save_smiles",
            "file.convert_smiles_to_cdxml",
            "file.export_smiles_fragment",
            "sdf.record.inspect",
            "smiles.record.inspect",
            "structure.paste_smiles",
            "sdf.field.set_value",
            "sdf.record.remove",
            "page.set_size",
            "bond.create",
            "bond.extend",
            "bond.set_order",
            "history.undo",
            "history.redo",
            "render.capture"
        ]
        .into_iter()
        .collect()
    );
    let info = bondline_commands::registry()
        .into_iter()
        .find(|info| info.name == "document.new")
        .ok_or("New schema")?;
    assert!(!info.read_only);
    let schema = info.input_schema.as_value();
    assert_eq!(schema["additionalProperties"], false);
    assert_eq!(schema["required"], json!(["expected_revision"]));
    assert_eq!(
        schema["properties"].as_object().ok_or("properties")?.len(),
        2
    );
    assert_eq!(schema["properties"]["discard_unsaved"]["type"], "boolean");
    assert_eq!(schema["properties"]["discard_unsaved"]["default"], false);
    for params in [
        json!({}),
        json!({"expected_revision":-1}),
        json!({"expected_revision":"1"}),
        json!({"expected_revision":1.5}),
        json!({"expected_revision":1,"discard_unsaved":null}),
        json!({"expected_revision":1,"discard_unsaved":"yes"}),
        json!({"expected_revision":1,"path":"ignored.cdxml"}),
    ] {
        assert_eq!(
            bondline_commands::Command::parse("document.new", params)
                .err()
                .ok_or("closed New parameters")?
                .info()
                .code,
            "invalid_params"
        );
    }
    let mut engine = Engine::default();
    engine.set_ui_file_host(true);
    let control = engine
        .controls()
        .into_iter()
        .find(|control| control.id == "file.new")
        .ok_or("central New control")?;
    assert_eq!(control.menu, "File");
    assert!(control.enabled);
    assert_eq!(
        serde_json::to_value(control.command)?,
        json!({"command":"ui.file",
        "params":{"expected_revision":0,"action":"new"}})
    );
    assert_eq!(
        serde_json::to_value(control.shortcut)?,
        json!({"key":"n","primary":true,"shift":false})
    );
    Ok(())
}

#[test]
fn every_content_kind_returns_to_fresh_a4_and_repeated_new_namespaces_stay_independent()
-> TestResult {
    let directory = support::FileTestDirectory::new()?;
    for (index, input) in [
        None,
        Some(("document.open_cdxml", NATIVE)),
        Some(("document.open_cdx", CDX)),
        Some(("document.open_mol", MOL)),
        Some(("document.open_mol", V3000)),
        Some(("document.open_sdf", SDF)),
        Some(("document.open_smiles", SMILES)),
    ]
    .into_iter()
    .enumerate()
    {
        let mut engine = Engine::default();
        engine.set_ui_file_host(true);
        engine.dispatch("ui.bond_tool", json!({"order":"triple"}))?;
        let path = directory.path(&format!("content-{index}.data"));
        if let Some((command, bytes)) = input {
            fs::write(&path, bytes)?;
            engine.dispatch(command, json!({"path":path}))?;
        }
        engine.set_view_canvas_size(800, 600)?;
        if matches!(
            engine.content(),
            bondline_commands::StoredContent::Sdf { .. }
        ) || engine
            .stored_document()
            .is_ok_and(|stored| stored.is_fully_mapped())
        {
            engine.dispatch("ui.view", json!({"action":"set_zoom","percent":150}))?;
        }
        let tool = engine.bond_tool();
        let view = engine.view();
        let revision = engine.state().revision;
        let old_stamp = engine.capture_stamp();
        let old_content = engine.content().clone();
        let old_result = engine.state();
        let old_wire = serde_json::to_value(&old_result)?;
        let result = engine.dispatch("document.new", json!({"expected_revision":revision}))?;
        assert!(result.changed);
        assert_blank(&engine, revision + 1, true)?;
        assert_ne!(engine.capture_stamp(), old_stamp);
        assert!(!engine.capture_stamp_matches(&old_stamp));
        assert_eq!(engine.bond_tool(), tool);
        assert_eq!(engine.view().zoom_percent, None);
        assert_eq!(engine.view().center, None);
        assert_eq!(engine.view().epoch, view.epoch + 1);
        assert!(
            engine.view_percent().is_some(),
            "actual canvas size must survive"
        );
        assert_eq!(serde_json::to_value(&old_result)?, old_wire);
        if let Some((_, bytes)) = input {
            assert_eq!(fs::read(&path)?, bytes);
            assert_eq!(
                old_content.source_info().ok_or("old source")?.byte_length,
                bytes.len()
            );
        }
    }
    let mut engine = Engine::default();
    engine.set_ui_file_host(true);
    pair(&mut engine, 0)?;
    let old_result = pair(&mut engine, 1)?;
    let old_wire = serde_json::to_value(&old_result)?;
    engine.dispatch("history.undo", json!({}))?;
    assert_eq!(
        (engine.state().undo_depth, engine.state().redo_depth),
        (1, 1)
    );
    engine.dispatch("ui.bond_tool", json!({"order":"triple"}))?;
    let tool = engine.bond_tool();
    engine.dispatch("ui.view", json!({"action":"actual"}))?;
    engine.dispatch(
        "ui.view",
        json!({"action":"pan","center_x_pt":100,"center_y_pt":200}),
    )?;
    let view = engine.view();
    engine.dispatch("document.new", new_params(3, true))?;
    assert_blank(&engine, 4, true)?;
    assert_eq!(engine.view().center, None);
    assert_eq!(engine.view().zoom_percent, None);
    assert_eq!(engine.view().epoch, view.epoch + 1);
    assert_eq!(engine.bond_tool(), tool);
    let first_new = engine.capture_stamp();
    engine.dispatch("document.new", json!({"expected_revision":4}))?;
    assert_blank(&engine, 5, true)?;
    assert_ne!(engine.capture_stamp(), first_new);
    pair(&mut engine, 0)?;
    assert_local_ethane(&engine)?;
    assert!(engine.state().file.dirty);
    engine.dispatch("history.undo", json!({}))?;
    assert_eq!(engine.document()?, &bondline_model::Document::default());
    assert!(
        !engine.state().file.dirty,
        "saved baseline must be the new blank"
    );
    engine.dispatch("history.redo", json!({}))?;
    assert_local_ethane(&engine)?;
    let output = directory.path("new-document-化学.cdxml");
    engine.dispatch("document.save_cdxml", json!({"path":output}))?;
    support::independently_read_file(&output, 1)?;
    let mut reopened = Engine::default();
    reopened.dispatch("document.open_cdxml", json!({"path":output}))?;
    assert_local_ethane(&reopened)?;
    assert_eq!(
        reopened
            .document()?
            .pages()
            .first()
            .ok_or("reopened page")?
            .size(),
        bondline_model::PageSize::A4
    );
    assert_eq!(serde_json::to_value(old_result)?, old_wire);
    Ok(())
}

#[test]
fn dirty_stale_invalid_and_pending_or_committed_close_refusals_are_atomic() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    for sdf in [false, true] {
        let mut engine = Engine::default();
        let path = directory.path("dirty-source.sdf");
        if sdf {
            fs::write(&path, SDF)?;
            engine.dispatch("document.open_sdf", json!({"path":path}))?;
            engine.dispatch(
                "sdf.field.set_value",
                json!({"expected_revision":1,"record_id":1,
                "field_index":2,"value_lines":["changed"]}),
            )?;
        } else {
            pair(&mut engine, 0)?;
        }
        let revision = engine.state().revision;
        let before = serde_json::to_value(engine.state())?;
        let content = engine.content().clone();
        for (params, code) in [
            (new_params(revision, false), "unsaved_changes"),
            (json!({"expected_revision":revision}), "unsaved_changes"),
            (new_params(revision - 1, true), "stale_revision"),
            (
                json!({"expected_revision":revision,"discard_unsaved":true,"path":"bad"}),
                "invalid_params",
            ),
        ] {
            assert_eq!(
                engine
                    .dispatch("document.new", params)
                    .err()
                    .ok_or("New refusal")?
                    .info()
                    .code,
                code
            );
            assert_eq!(serde_json::to_value(engine.state())?, before);
            assert_eq!(engine.content(), &content);
        }
        engine.dispatch("document.new", new_params(revision, true))?;
        assert_blank(&engine, revision + 1, false)?;
        if sdf {
            assert_eq!(fs::read(path)?, SDF);
        }
    }
    for close in ["approved", "needs_decision", "committed"] {
        let mut engine = Engine::default();
        if close == "needs_decision" {
            pair(&mut engine, 0)?;
        }
        let revision = engine.state().revision;
        engine.dispatch(
            "document.close",
            json!({"expected_revision":revision,"action":"request"}),
        )?;
        if close == "committed" {
            assert!(engine.complete_close());
        }
        assert_eq!(serde_json::to_value(engine.state())?["close"], close);
        let before = serde_json::to_value(engine.state())?;
        let content = engine.content().clone();
        assert_eq!(
            engine
                .dispatch("document.new", new_params(revision, true))
                .err()
                .ok_or("close-qualified New refusal")?
                .info()
                .code,
            if close == "committed" {
                "close_committed"
            } else {
                "ui_busy"
            }
        );
        assert_eq!(serde_json::to_value(engine.state())?, before);
        assert_eq!(engine.content(), &content);
    }
    Ok(())
}

#[test]
fn prior_open_and_sdf_candidates_go_stale_while_an_old_query_remains_historical() -> TestResult {
    use bondline_commands::Command;
    let directory = support::FileTestDirectory::new()?;
    let path = directory.path("previous-complete.sdf");
    fs::write(&path, SDF)?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_sdf", json!({"path":path}))?;
    let query = engine.prepare(Command::parse(
        "sdf.record.inspect",
        json!({"expected_revision":1,"record_id":2}),
    )?)?;
    let old_open = engine.prepare(Command::parse("document.open_sdf", json!({"path":path}))?)?;
    let field = engine.prepare(Command::parse(
        "sdf.field.set_value",
        json!({"expected_revision":1,
        "record_id":1,"field_index":2,"value_lines":["obsolete"]}),
    )?)?;
    let removal = engine.prepare(Command::parse(
        "sdf.record.remove",
        json!({"expected_revision":1,"record_id":1}),
    )?)?;
    assert!(old_open.is_deferred() && field.is_deferred() && removal.is_deferred());
    let completed = [old_open.complete()?, field.complete()?, removal.complete()?];
    engine.dispatch("document.new", json!({"expected_revision":1}))?;
    let before = serde_json::to_value(engine.state())?;
    for old in completed {
        assert_eq!(
            engine
                .finish(old)
                .err()
                .ok_or("old authored candidate")?
                .info()
                .code,
            "stale_revision"
        );
        assert_eq!(serde_json::to_value(engine.state())?, before);
        assert_blank(&engine, 2, false)?;
    }
    let historical = serde_json::to_value(engine.finish(query.complete()?)?)?;
    assert_eq!(historical["revision"], 1);
    assert_eq!(historical["document"], serde_json::Value::Null);
    assert_eq!(historical["collection"]["byte_length"], 777);
    assert_eq!(historical["collection"]["record_count"], 2);
    assert_eq!(historical["sdf_record"]["record_id"], 2);
    assert_eq!(
        historical["sdf_record"]["fields"][0]["value_lines"],
        json!(["second record"])
    );
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert_eq!(fs::read(path)?, SDF);
    Ok(())
}

#[test]
fn old_published_save_keeps_its_lease_but_cannot_associate_or_clean_a_new_session() -> TestResult {
    use bondline_commands::Command;
    let directory = support::FileTestDirectory::new()?;
    let old_output = directory.path("old-session-化学.cdxml");
    let fresh_output = directory.path("fresh-session-λ.cdxml");
    let mut engine = Engine::default();
    engine.set_ui_file_host(true);
    pair(&mut engine, 0)?;
    pair(&mut engine, 1)?;
    let save = engine.prepare(Command::parse(
        "document.save_cdxml",
        json!({"path":old_output}),
    )?)?;
    engine.dispatch("document.new", new_params(2, true))?;
    engine.dispatch("document.new", json!({"expected_revision":3}))?;
    assert_blank(&engine, 4, true)?;
    let before = serde_json::to_value(engine.state())?;
    let completed = save.complete()?;
    support::independently_read_file(&old_output, 2)?;
    assert_eq!(
        engine
            .prepare(Command::parse(
                "document.save_cdxml",
                json!({"path":fresh_output})
            )?)
            .err()
            .ok_or("published old Save reservation")?
            .info()
            .code,
        "busy"
    );
    assert!(!fresh_output.exists());
    assert_eq!(serde_json::to_value(engine.state())?, before);
    let receipt = engine.finish(completed)?;
    let published = receipt
        .file_operation
        .ok_or("historical publication receipt")?;
    assert_eq!(published.input_revision, 2);
    assert_eq!(published.byte_length, fs::read(&old_output)?.len());
    assert_eq!(
        published.path,
        fs::canonicalize(&old_output)?.to_str().ok_or("old path")?
    );
    assert!(published.ui_request_id.is_none() && published.ui_request_current.is_none());
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert_blank(&engine, 4, true)?;
    assert!(!engine.finish_ui_file(1, false));
    assert!(!engine.complete_close());
    pair(&mut engine, 0)?;
    assert_local_ethane(&engine)?;
    engine.dispatch("document.save_cdxml", json!({"path":fresh_output}))?;
    support::independently_read_file(&fresh_output, 1)?;
    assert!(!engine.state().file.dirty);
    assert_eq!(
        engine.state().file.path.as_deref(),
        fs::canonicalize(&fresh_output)?.to_str()
    );
    support::independently_read_file(&old_output, 2)?;
    Ok(())
}
