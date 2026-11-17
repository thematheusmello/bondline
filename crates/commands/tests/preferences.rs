//! Independent preferences JSON declaration before preferences production.
use bondline_commands::Engine;
use serde_json::json;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn preferences_inspect_and_set_are_session_only_and_initially_keep_spiro() -> TestResult {
    let mut engine = Engine::default();
    let before = serde_json::to_value(engine.state())?;
    let inspected =
        serde_json::to_value(engine.dispatch("ui.preferences", json!({"action":"inspect"}))?)?;
    assert_eq!(
        inspected["preferences"],
        json!({"sprout_rings_when_clicking":false,"epoch":0})
    );
    assert_eq!(inspected["changed"], false);
    assert_eq!(serde_json::to_value(engine.state())?, before);
    let changed = serde_json::to_value(engine.dispatch(
        "ui.preferences",
        json!({
        "action":"set","expected_epoch":0,"sprout_rings_when_clicking":true}),
    )?)?;
    assert_eq!(
        changed["preferences"],
        json!({"sprout_rings_when_clicking":true,"epoch":1})
    );
    assert_eq!(
        changed["changed"], false,
        "an application preference is not an authored edit"
    );
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert!(before.get("preferences").is_none());
    Ok(())
}

// Additive bounded independent acceptance; the initial declaration above is immutable.
use bondline_commands::{ChainContext, Command, RingContext, SelectionContext};
use bondline_io::StoredDocument;
use bondline_model::PageSize;
use serde_json::Value;
use std::{collections::BTreeSet, fs, sync::Arc};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(
    dead_code,
    reason = "reuse only the unchanged isolated-directory helper"
)]
mod support;
const NAME: &str = "ui.preferences";
const NATIVE: &[u8] = include_bytes!("../../io/tests/fixtures/bondline-v0.cdxml");

fn native() -> TestResult<Engine> {
    let stored = StoredDocument::from_cdxml(NATIVE.into(), PageSize::A4)?;
    assert!(stored.is_fully_mapped());
    Ok(Engine::from_document(stored))
}
fn prefs(engine: &mut Engine) -> TestResult<Value> {
    let before = serde_json::to_value(engine.state())?;
    let value = serde_json::to_value(engine.dispatch(NAME, json!({"action":"inspect"}))?)?;
    assert!(value.get("ui_preferences_operation").is_none());
    assert_eq!(value["changed"], false);
    assert_eq!(serde_json::to_value(engine.state())?, before);
    Ok(value["preferences"].clone())
}
fn set(engine: &mut Engine, epoch: u64, sprout: bool) -> TestResult<Value> {
    let value = serde_json::to_value(engine.dispatch(
        NAME,
        json!({"action":"set",
        "expected_epoch":epoch,"sprout_rings_when_clicking":sprout}),
    )?)?;
    assert_eq!(value["changed"], false);
    assert!(value.get("file_operation").is_none() && value.get("ui_file_operation").is_none());
    Ok(value)
}
fn opened(engine: &mut Engine) -> TestResult<(u64, Value)> {
    let value = serde_json::to_value(engine.dispatch(
        NAME,
        json!({"action":"open",
        "expected_revision":engine.state().revision}),
    )?)?;
    let id = value["ui_preferences"]["id"]
        .as_u64()
        .ok_or("preferences request")?;
    assert!(id > 0);
    assert_eq!(value["changed"], false);
    assert!(value.get("file_operation").is_none() && value.get("ui_file_operation").is_none());
    Ok((id, value))
}
fn cancel(engine: &mut Engine, id: u64) -> TestResult<Value> {
    let value = serde_json::to_value(engine.dispatch(
        NAME,
        json!({"action":"cancel",
        "expected_revision":engine.state().revision,"request_id":id}),
    )?)?;
    assert_eq!(value["changed"], false);
    assert!(value.get("file_operation").is_none() && value.get("ui_file_operation").is_none());
    Ok(value)
}
fn refuse(engine: &mut Engine, name: &str, params: Value, code: &str) -> TestResult {
    let before = serde_json::to_value(engine.state())?;
    let content = engine.content().clone();
    let model = engine
        .stored_document()
        .ok()
        .map(StoredDocument::model_snapshot);
    let stamp = engine.capture_stamp();
    let file = engine.ui_file_context_stamp();
    let view = engine.view();
    let tool = (engine.bond_tool(), engine.effective_tool());
    let selected = engine.selection().clone();
    let generation = engine.session_generation();
    let error = engine
        .dispatch(name, params)
        .err()
        .ok_or("required refusal accepted")?;
    assert_eq!(error.info().code, code, "{error}");
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert_eq!(engine.content(), &content);
    assert_eq!(engine.capture_stamp(), stamp);
    assert!(engine.ui_file_context_stamp_matches(&file));
    assert_eq!(engine.view(), view);
    assert_eq!((engine.bond_tool(), engine.effective_tool()), tool);
    assert_eq!(engine.selection(), &selected);
    assert_eq!(engine.session_generation(), generation);
    if let Some(model) = model {
        assert!(Arc::ptr_eq(
            &model,
            &engine.stored_document()?.model_snapshot()
        ));
    }
    Ok(())
}
fn schema_node<'a>(root: &'a Value, node: &'a Value) -> TestResult<&'a Value> {
    match node.get("$ref").and_then(Value::as_str) {
        Some(reference) => Ok(root
            .pointer(reference.strip_prefix('#').ok_or("local schema ref")?)
            .ok_or("schema target")?),
        None => Ok(node),
    }
}

#[test]
fn five_closed_action_objects_reject_null_missing_unknown_and_wrong_scalar_fields() -> TestResult {
    let rows = bondline_commands::registry();
    assert_eq!(rows.len(), 46);
    assert_eq!(rows.get(44).ok_or("preferences registry")?.name, NAME);
    assert_eq!(
        rows.last().ok_or("last row")?.name,
        "document.set_document_settings"
    );
    let schema = rows.get(44).ok_or("schema")?.input_schema.as_value();
    assert_eq!(schema["type"], "object");
    let branches = schema["oneOf"].as_array().ok_or("closed action branches")?;
    assert_eq!(branches.len(), 5);
    let mut actions = BTreeSet::new();
    for branch in branches {
        let branch = schema_node(schema, branch)?;
        assert_eq!(branch["type"], "object");
        assert_eq!(branch["additionalProperties"], false);
        let fields = branch["properties"].as_object().ok_or("branch fields")?;
        let action = fields["action"]["const"].as_str().ok_or("action const")?;
        assert!(actions.insert(action));
        let expected = match action {
            "inspect" => BTreeSet::from(["action"]),
            "set" => BTreeSet::from(["action", "expected_epoch", "sprout_rings_when_clicking"]),
            "open" => BTreeSet::from(["action", "expected_revision"]),
            "apply" => BTreeSet::from([
                "action",
                "expected_revision",
                "request_id",
                "sprout_rings_when_clicking",
            ]),
            "cancel" => BTreeSet::from(["action", "expected_revision", "request_id"]),
            _ => return Err("undeclared preferences action".into()),
        };
        assert_eq!(
            fields.keys().map(String::as_str).collect::<BTreeSet<_>>(),
            expected
        );
        assert_eq!(
            branch["required"]
                .as_array()
                .ok_or("required fields")?
                .iter()
                .map(|value| value.as_str().ok_or("required key"))
                .collect::<Result<BTreeSet<_>, _>>()?,
            expected
        );
        if fields.contains_key("sprout_rings_when_clicking") {
            assert_eq!(fields["sprout_rings_when_clicking"]["type"], "boolean");
        }
    }
    assert_eq!(
        actions,
        BTreeSet::from(["inspect", "set", "open", "apply", "cancel"])
    );
    let mut engine = Engine::default();
    for bad in [
        Value::Null,
        json!([]),
        json!(["inspect"]),
        json!({}),
        json!({"action":"unknown"}),
        json!({"action":null}),
        json!({"action":"inspect","expected_epoch":0}),
        json!({"action":"inspect","extra":true}),
        json!({"action":"set","expected_epoch":0}),
        json!({"action":"set","sprout_rings_when_clicking":true}),
        json!({"action":"set","expected_epoch":null,"sprout_rings_when_clicking":true}),
        json!({"action":"set","expected_epoch":-1,"sprout_rings_when_clicking":true}),
        json!({"action":"set","expected_epoch":0.5,"sprout_rings_when_clicking":true}),
        json!({"action":"set","expected_epoch":0,"sprout_rings_when_clicking":null}),
        json!({"action":"set","expected_epoch":0,"sprout_rings_when_clicking":1}),
        json!({"action":"set","expected_epoch":0,"sprout_rings_when_clicking":"true"}),
        json!({"action":"set","expected_epoch":0,"sprout_rings_when_clicking":true,"request_id":1}),
        json!({"action":"open"}),
        json!({"action":"open","expected_revision":null}),
        json!({"action":"open","expected_revision":0,"sprout_rings_when_clicking":true}),
        json!({"action":"apply","expected_revision":0,"request_id":1}),
        json!({"action":"apply","expected_revision":0,"request_id":null,"sprout_rings_when_clicking":false}),
        json!({"action":"cancel","expected_revision":0}),
        json!({"action":"cancel","expected_revision":0,"request_id":1,"sprout_rings_when_clicking":false}),
    ] {
        assert!(
            Command::parse(NAME, bad.clone()).is_err(),
            "closed parse accepted {bad}"
        );
        refuse(&mut engine, NAME, bad, "invalid_params")?;
    }
    Ok(())
}

#[test]
fn real_preference_changes_and_noops_preserve_source_redo_selection_close_and_model_sharing()
-> TestResult {
    let mut engine = native()?;
    engine.set_ui_file_host(true);
    for y in [144, 240] {
        engine.dispatch(
            "bond.create",
            json!({"page_id":1,"begin_x_pt":240,"begin_y_pt":y,
            "end_x_pt":288,"end_y_pt":y}),
        )?;
    }
    engine.dispatch("history.undo", json!({}))?;
    engine.dispatch("ui.bond_tool", json!({"order":"triple"}))?;
    engine.dispatch(
        "ui.selection",
        json!({"action":"set","expected_revision":engine.state().revision,
        "page_id":1,"atom_ids":[1,2]}),
    )?;
    engine.dispatch(
        "document.close",
        json!({"action":"request","expected_revision":engine.state().revision}),
    )?;
    let before = serde_json::to_value(engine.state())?;
    assert_eq!(before["redo_depth"], 1);
    assert_eq!(before["close"], "needs_decision");
    let revision = engine.state().revision;
    refuse(
        &mut engine,
        NAME,
        json!({"action":"open","expected_revision":revision}),
        "ui_busy",
    )?;
    let model = engine.stored_document()?.model_snapshot();
    let bytes = engine.stored_document()?.to_cdxml()?.into_owned();
    let source = engine.capture_stamp();
    let file = engine.ui_file_context_stamp();
    let view = engine.view();
    let tool = engine.bond_tool();
    let mode = engine.effective_tool();
    let selection = engine.selection().clone();
    let generation = engine.session_generation();
    let noop = set(&mut engine, 0, false)?;
    assert!(noop.get("ui_preferences_operation").is_none());
    assert_eq!(engine.bond_tool(), tool);
    for (expected_epoch, value, wanted_epoch) in [(0, true, 1), (1, false, 2)] {
        let reply = set(&mut engine, expected_epoch, value)?;
        assert_eq!(
            reply["preferences"],
            json!({"sprout_rings_when_clicking":value,"epoch":wanted_epoch})
        );
        assert_eq!(
            reply["ui_preferences_operation"],
            json!({"action":"set","changed":true})
        );
        assert_eq!(reply["changed"], false);
        assert_eq!(serde_json::to_value(engine.state())?, before);
        assert_eq!(engine.bond_tool().epoch, tool.epoch + wanted_epoch);
        assert_eq!(engine.bond_tool().order, tool.order);
        assert_eq!(engine.effective_tool(), mode);
        assert_eq!(engine.selection(), &selection);
        assert_eq!(engine.view(), view);
        assert_eq!(engine.session_generation(), generation);
        assert_eq!(engine.capture_stamp(), source);
        assert!(engine.ui_file_context_stamp_matches(&file));
        assert!(Arc::ptr_eq(
            &model,
            &engine.stored_document()?.model_snapshot()
        ));
        assert_eq!(
            engine.stored_document()?.to_cdxml()?.as_ref(),
            bytes.as_slice()
        );
    }
    refuse(
        &mut engine,
        NAME,
        json!({"action":"set","expected_epoch":1,
        "sprout_rings_when_clicking":false}),
        "ui_stale_request",
    )?;
    assert_eq!(
        prefs(&mut engine)?,
        json!({"sprout_rings_when_clicking":false,"epoch":2})
    );
    let tool = engine.bond_tool();
    let noop = set(&mut engine, 2, false)?;
    assert!(noop.get("ui_preferences_operation").is_none());
    assert_eq!(engine.bond_tool(), tool);
    assert_eq!(serde_json::to_value(engine.state())?, before);
    Ok(())
}

#[test]
fn session_preferences_survive_new_open_save_history_host_changes_and_foreign_content() -> TestResult
{
    let directory = support::FileTestDirectory::new()?;
    let mut engine = native()?;
    set(&mut engine, 0, true)?;
    let target = directory.path("preferences-do-not-become-file-data-λ.cdxml");
    engine.dispatch("document.save_cdxml", json!({"path":target}))?;
    assert_eq!(fs::read(&target)?, NATIVE);
    engine.dispatch(
        "document.new",
        json!({"expected_revision":engine.state().revision}),
    )?;
    engine.dispatch("document.open_cdxml", json!({"path":target}))?;
    engine.dispatch(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":240,"begin_y_pt":144,
        "end_x_pt":288,"end_y_pt":144}),
    )?;
    for action in ["history.undo", "history.redo"] {
        engine.dispatch(action, json!({}))?;
        assert_eq!(
            prefs(&mut engine)?,
            json!({"sprout_rings_when_clicking":true,"epoch":1})
        );
    }
    engine.set_ui_file_host(true);
    engine.set_ui_file_host(false);
    engine.set_ui_file_host(true);
    assert_eq!(
        prefs(&mut engine)?,
        json!({"sprout_rings_when_clicking":true,"epoch":1})
    );
    let mut epoch = 1;
    for (name, raw, open) in [
        (
            "original.mol",
            include_bytes!("../../io/tests/fixtures/mol/ethane-v2000.mol").as_slice(),
            "document.open_mol",
        ),
        (
            "original.cdx",
            include_bytes!("../../io/tests/fixtures/cdx/metadata-explicit.cdx").as_slice(),
            "document.open_cdx",
        ),
        (
            "original.sdf",
            include_bytes!("../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf").as_slice(),
            "document.open_sdf",
        ),
        (
            "original.smi",
            b"CC first\r\nO\topaque\xff\0".as_slice(),
            "document.open_smiles",
        ),
    ] {
        let path = directory.path(name);
        fs::write(&path, raw)?;
        engine.dispatch(open, json!({"path":path,"discard_unsaved":true}))?;
        assert_eq!(
            prefs(&mut engine)?,
            json!({"sprout_rings_when_clicking":true,"epoch":epoch})
        );
        let before = serde_json::to_value(engine.state())?;
        let content = engine.content().clone();
        set(&mut engine, epoch, false)?;
        set(&mut engine, epoch + 1, true)?;
        epoch += 2;
        assert_eq!(serde_json::to_value(engine.state())?, before);
        assert_eq!(engine.content(), &content);
        assert_eq!(fs::read(&path)?, raw);
        assert_eq!(
            prefs(&mut engine)?,
            json!({"sprout_rings_when_clicking":true,"epoch":epoch})
        );
        let (request, _) = opened(&mut engine)?;
        cancel(&mut engine, request)?;
        assert_eq!(serde_json::to_value(engine.state())?, before);
    }
    let mut foreign = String::from_utf8(NATIVE.to_vec())?;
    foreign = foreign.replace("<CDXML ", "<CDXML Private=\"retain\" ");
    let mut opaque = Engine::from_document(StoredDocument::from_cdxml(
        foreign.as_bytes().into(),
        PageSize::A4,
    )?);
    assert!(!opaque.stored_document()?.is_fully_mapped());
    set(&mut opaque, 0, true)?;
    assert_eq!(
        opaque.stored_document()?.export_cdxml()?,
        foreign.as_bytes()
    );
    assert_eq!(
        prefs(&mut Engine::default())?,
        json!({"sprout_rings_when_clicking":false,"epoch":0})
    );
    Ok(())
}

#[test]
fn presentation_ownership_exact_ids_apply_and_cancel_never_author_or_acknowledge_save() -> TestResult
{
    let mut engine = Engine::default();
    refuse(
        &mut engine,
        NAME,
        json!({"action":"open","expected_revision":0}),
        "ui_unavailable",
    )?;
    engine.set_ui_file_host(true);
    engine.dispatch(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":144,"begin_y_pt":144,
        "end_x_pt":192,"end_y_pt":144}),
    )?;
    let before = serde_json::to_value(engine.state())?;
    let tool = engine.bond_tool();
    refuse(
        &mut engine,
        NAME,
        json!({"action":"open","expected_revision":0}),
        "stale_revision",
    )?;
    let (id, open) = opened(&mut engine)?;
    assert_eq!(
        open["ui_preferences"],
        json!({"id":id,"input_revision":1,
        "sprout_rings_when_clicking":false,"current":true})
    );
    assert_eq!(
        open["ui_preferences_operation"],
        json!({"action":"open","changed":false,"request_id":id})
    );
    assert_eq!(
        open["preferences"],
        json!({"sprout_rings_when_clicking":false,"epoch":0})
    );
    assert!(!open["changed"].as_bool().ok_or("changed")?);
    assert!(open.get("file_operation").is_none() && open.get("ui_file_operation").is_none());
    assert_eq!(engine.bond_tool(), tool);
    for (name, input) in [
        (NAME, json!({"action":"open","expected_revision":1})),
        ("ui.file", json!({"action":"open","expected_revision":1})),
        (
            "ui.atom_label",
            json!({"action":"open","expected_revision":1,"page_id":1,"fragment_id":1,"atom_id":2}),
        ),
        (
            "ui.drawing_settings",
            json!({"action":"open","expected_revision":1}),
        ),
    ] {
        refuse(&mut engine, name, input, "ui_busy")?;
    }
    refuse(
        &mut engine,
        NAME,
        json!({"action":"apply","expected_revision":0,"request_id":id,
        "sprout_rings_when_clicking":true}),
        "stale_revision",
    )?;
    refuse(
        &mut engine,
        NAME,
        json!({"action":"cancel","expected_revision":0,"request_id":id}),
        "stale_revision",
    )?;
    refuse(
        &mut engine,
        NAME,
        json!({"action":"cancel","expected_revision":1,"request_id":id+1}),
        "ui_stale_request",
    )?;
    let cancelled = cancel(&mut engine, id)?;
    assert_eq!(
        cancelled["ui_preferences_operation"],
        json!({"action":"cancel","changed":false,"request_id":id})
    );
    assert!(cancelled.get("ui_preferences").is_none());
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert_eq!(engine.bond_tool(), tool);
    let (next, _) = opened(&mut engine)?;
    assert!(next > id);
    refuse(
        &mut engine,
        NAME,
        json!({"action":"cancel","expected_revision":1,"request_id":id}),
        "ui_stale_request",
    )?;
    let applied = serde_json::to_value(engine.dispatch(
        NAME,
        json!({"action":"apply","expected_revision":1,
        "request_id":next,"sprout_rings_when_clicking":false}),
    )?)?;
    assert_eq!(
        applied["ui_preferences_operation"],
        json!({"action":"apply","changed":false,"request_id":next})
    );
    assert!(applied.get("ui_preferences").is_none());
    assert_eq!(serde_json::to_value(engine.state())?, before);
    let (next, _) = opened(&mut engine)?;
    let applied = serde_json::to_value(engine.dispatch(
        NAME,
        json!({"action":"apply","expected_revision":1,
        "request_id":next,"sprout_rings_when_clicking":true}),
    )?)?;
    assert_eq!(
        applied["ui_preferences_operation"],
        json!({"action":"apply","changed":true,"request_id":next})
    );
    assert_eq!(applied["changed"], false);
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert_eq!(engine.bond_tool().epoch, tool.epoch + 1);
    engine.dispatch(
        "document.close",
        json!({"action":"discard","expected_revision":1}),
    )?;
    assert!(engine.complete_close());
    for action in [
        json!({"action":"inspect"}),
        json!({"action":"set","expected_epoch":1,"sprout_rings_when_clicking":true}),
        json!({"action":"open","expected_revision":1}),
    ] {
        refuse(&mut engine, NAME, action, "close_committed")?;
    }
    Ok(())
}

#[test]
fn stale_preference_drafts_never_rebase_and_only_the_exact_current_id_can_cancel() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    for change in [
        "view_aba",
        "association_aba",
        "new_source",
        "preference_aba",
        "close_aba",
    ] {
        let mut engine = native()?;
        engine.set_ui_file_host(true);
        let a = directory.path(&format!("request-{change}-A-λ.cdxml"));
        let b = directory.path(&format!("request-{change}-B-λ.cdxml"));
        engine.dispatch("document.save_cdxml", json!({"path":a}))?;
        engine.dispatch("ui.view", json!({"action":"actual"}))?;
        let (id, _) = opened(&mut engine)?;
        match change {
            "view_aba" => {
                engine.dispatch("ui.view", json!({"action":"set_zoom","percent":150}))?;
                engine.dispatch("ui.view", json!({"action":"actual"}))?;
            }
            "association_aba" => {
                engine.dispatch("document.save_cdxml", json!({"path":b}))?;
                engine.dispatch("document.save_cdxml", json!({"path":a,"overwrite":true}))?;
            }
            "new_source" => {
                engine.dispatch(
                    "document.new",
                    json!({"expected_revision":engine.state().revision}),
                )?;
            }
            "preference_aba" => {
                set(&mut engine, 0, true)?;
                set(&mut engine, 1, false)?;
            }
            "close_aba" => {
                engine.dispatch(
                    "document.close",
                    json!({"action":"request","expected_revision":engine.state().revision}),
                )?;
                engine.dispatch(
                    "document.close",
                    json!({"action":"cancel","expected_revision":engine.state().revision}),
                )?;
            }
            _ => return Err("undeclared context change".into()),
        }
        let state = serde_json::to_value(engine.state())?;
        assert_eq!(state["ui_preferences"]["id"], id);
        assert_eq!(
            state["ui_preferences"]["current"], false,
            "stale draft revived after {change}"
        );
        let current = prefs(&mut engine)?;
        let revision = engine.state().revision;
        refuse(
            &mut engine,
            NAME,
            json!({"action":"apply","expected_revision":revision,
            "request_id":id,"sprout_rings_when_clicking":true}),
            "ui_stale_request",
        )?;
        assert_eq!(prefs(&mut engine)?, current);
        let cancelled = cancel(&mut engine, id)?;
        assert!(cancelled.get("ui_preferences").is_none());
        assert_eq!(
            cancelled["ui_preferences_operation"],
            json!({"action":"cancel","changed":false,"request_id":id})
        );
        assert_eq!(prefs(&mut engine)?, current);
        let (next, _) = opened(&mut engine)?;
        assert!(next > id);
        let revision = engine.state().revision;
        refuse(
            &mut engine,
            NAME,
            json!({"action":"cancel","expected_revision":revision,
            "request_id":id}),
            "ui_stale_request",
        )?;
        cancel(&mut engine, next)?;
    }
    let mut engine = native()?;
    engine.set_ui_file_host(true);
    let (id, _) = opened(&mut engine)?;
    engine.set_ui_file_host(false);
    engine.set_ui_file_host(true);
    assert!(
        serde_json::to_value(engine.state())?
            .get("ui_preferences")
            .is_none()
    );
    assert_eq!(
        prefs(&mut engine)?,
        json!({"sprout_rings_when_clicking":false,"epoch":0})
    );
    refuse(
        &mut engine,
        NAME,
        json!({"action":"cancel","expected_revision":0,"request_id":id}),
        "ui_stale_request",
    )?;
    let (next, _) = opened(&mut engine)?;
    assert!(next > id);
    cancel(&mut engine, next)?;
    Ok(())
}

enum Proof {
    Ring(RingContext),
    Selection(SelectionContext),
    Chain(ChainContext),
}
impl Proof {
    fn current(&self, engine: &Engine) -> bool {
        match self {
            Self::Ring(p) => p.is_current(engine),
            Self::Selection(p) => p.is_current(engine),
            Self::Chain(p) => p.is_current(engine),
        }
    }
}
fn proof(engine: &Engine, mode: &str) -> TestResult<Proof> {
    Ok(match mode {
        "ring" => Proof::Ring(engine.ring_context(1)?),
        "selection" => Proof::Selection(engine.selection_context()?),
        "chain" => Proof::Chain(engine.chain_context(1)?),
        _ => return Err("mode".into()),
    })
}
#[test]
fn hidden_preferences_and_real_preference_aba_stale_all_three_gesture_proofs_without_tool_switches()
-> TestResult {
    for mode in ["ring", "selection", "chain"] {
        for change in ["modal", "preference"] {
            let mut engine = Engine::default();
            engine.set_ui_file_host(true);
            engine.dispatch(
                "bond.create",
                json!({"page_id":1,"begin_x_pt":144,"begin_y_pt":144,
                "end_x_pt":192,"end_y_pt":144}),
            )?;
            match mode {
                "ring" => {
                    engine.dispatch("ui.ring_tool", json!({"size":6}))?;
                }
                "selection" => {
                    engine.dispatch(
                        "ui.selection",
                        json!({"action":"set","expected_revision":1,
                    "page_id":1,"atom_ids":[1,2]}),
                    )?;
                }
                "chain" => {
                    engine
                        .dispatch("ui.chain_tool", json!({"action":"select","kind":"acyclic"}))?;
                }
                _ => return Err("mode".into()),
            }
            let before = serde_json::to_value(engine.state())?;
            let tool = engine.bond_tool();
            let selected = engine.selection().clone();
            let view = engine.view();
            let model = engine.stored_document()?.model_snapshot();
            let source = engine.capture_stamp();
            let file = engine.ui_file_context_stamp();
            let context = proof(&engine, mode)?;
            assert!(context.current(&engine));
            prefs(&mut engine)?;
            let noop = set(&mut engine, 0, false)?;
            assert!(noop.get("ui_preferences_operation").is_none());
            assert!(context.current(&engine), "inspect/no-op invalidated {mode}");
            if change == "modal" {
                let (id, _) = opened(&mut engine)?;
                assert!(
                    !context.current(&engine),
                    "active Preferences failed to fence {mode}"
                );
                cancel(&mut engine, id)?;
                assert_eq!(
                    engine.bond_tool(),
                    tool,
                    "modal epoch must not masquerade as a tool switch"
                );
            } else {
                set(&mut engine, 0, true)?;
                set(&mut engine, 1, false)?;
                assert_eq!(engine.bond_tool().epoch, tool.epoch + 2);
                assert_eq!(engine.bond_tool().order, tool.order);
            }
            assert!(
                !context.current(&engine),
                "restored values revived old {mode} proof after {change}"
            );
            assert!(
                proof(&engine, mode)?.current(&engine),
                "fresh proof must remain possible"
            );
            assert_eq!(serde_json::to_value(engine.state())?, before);
            assert_eq!(engine.selection(), &selected);
            assert_eq!(engine.view(), view);
            assert!(Arc::ptr_eq(
                &model,
                &engine.stored_document()?.model_snapshot()
            ));
            assert_eq!(engine.capture_stamp(), source);
            assert!(engine.ui_file_context_stamp_matches(&file));
        }
    }
    Ok(())
}
