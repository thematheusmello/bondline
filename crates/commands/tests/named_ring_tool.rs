//! First independent JSON-only native named-tool declaration; no new Rust type is used.
use bondline_commands::Engine;
use serde_json::json;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn named_benzene_tool_is_transient_with_the_exact_existing_receipt_shape() -> TestResult {
    let mut engine = Engine::default();
    let before = serde_json::to_value(engine.state())?;
    let epoch = engine.bond_tool().epoch;
    let result = engine.dispatch("ui.ring_tool", json!({"kind":"benzene"}))?;
    assert!(!result.changed);
    let mut result = serde_json::to_value(result)?;
    assert_eq!(
        result["ui_ring_tool_operation"],
        json!({"size":6,"epoch":epoch+1,"changed":true})
    );
    result
        .as_object_mut()
        .ok_or("result")?
        .remove("ui_ring_tool_operation");
    assert_eq!(result, before);
    assert_eq!(serde_json::to_value(engine.state())?, before);
    let controls = engine.controls();
    let selected = controls
        .iter()
        .filter(|c| c.menu == "Tools" && c.selected == Some(true))
        .map(|c| c.id)
        .collect::<Vec<_>>();
    assert_eq!(selected, vec!["tool.ring.benzene"]);
    Ok(())
}

// Stage2 preserves the exact first1,080-byte existing-JSON declaration.
use serde_json::Value;
use std::sync::Arc;

fn authored(engine: &Engine) -> TestResult<Value> {
    let s = serde_json::to_value(engine.state())?;
    Ok(
        json!({"revision":s["revision"],"document":s["document"],"collection":s["collection"],
        "smiles_file":s["smiles_file"],"source":s["source"],"file":s["file"],
        "undo_depth":s["undo_depth"],"redo_depth":s["redo_depth"],"close":s["close"],
        "close_epoch":s["close_epoch"],"ui_file":s["ui_file"]}),
    )
}
fn active(engine: &Engine, id: &str) -> TestResult {
    assert_eq!(
        engine
            .controls()
            .into_iter()
            .filter(|c| c.menu == "Tools" && c.selected == Some(true))
            .map(|c| c.id)
            .collect::<Vec<_>>(),
        vec![id]
    );
    Ok(())
}
fn choose(
    engine: &mut Engine,
    params: Value,
    size: u8,
    epoch: u64,
    changed: bool,
    id: &str,
) -> TestResult {
    let before = authored(engine)?;
    let stamp = engine.capture_stamp();
    let model = engine.stored_document()?.model_snapshot();
    let order = engine.bond_tool().order;
    let generation = engine.session_generation();
    let view = engine.view();
    let file = engine.ui_file_context_stamp();
    let result = serde_json::to_value(engine.dispatch("ui.ring_tool", params)?)?;
    assert_eq!(result["changed"], false);
    assert_eq!(
        result["ui_ring_tool_operation"],
        json!({"size":size,"epoch":epoch,"changed":changed})
    );
    assert!(result.get("file_operation").is_none() && result.get("close_operation").is_none());
    assert_eq!(authored(engine)?, before);
    assert_eq!(engine.capture_stamp(), stamp);
    assert!(Arc::ptr_eq(
        &model,
        &engine.stored_document()?.model_snapshot()
    ));
    assert_eq!(engine.bond_tool().order, order);
    assert_eq!(engine.bond_tool().epoch, epoch);
    assert_eq!(engine.session_generation(), generation);
    assert_eq!(engine.view(), view);
    assert!(engine.ui_file_context_stamp_matches(&file));
    active(engine, id)?;
    assert!(
        serde_json::to_value(engine.state())?
            .get("ui_ring_tool_operation")
            .is_none()
    );
    Ok(())
}

#[test]
fn named_sized_and_kind_aba_are_exclusive_modes_while_noops_preserve_current_context() -> TestResult
{
    let mut e = Engine::default();
    e.dispatch(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":72,"begin_y_pt":72,"end_x_pt":120,"end_y_pt":72}),
    )?;
    e.dispatch("ui.bond_tool", json!({"order":"triple"}))?;
    e.dispatch(
        "ui.selection",
        json!({"action":"set","expected_revision":1,"page_id":1,"atom_ids":[2,1]}),
    )?;
    assert_eq!(e.bond_tool().epoch, 2);
    let original = authored(&e)?;
    choose(
        &mut e,
        json!({"kind":"benzene"}),
        6,
        3,
        true,
        "tool.ring.benzene",
    )?;
    assert!(!e.selection().active && e.selection().atom_ids.is_empty());
    let first = e.ring_context(1)?;
    assert!(first.is_current(&e));
    choose(
        &mut e,
        json!({"kind":"benzene"}),
        6,
        3,
        false,
        "tool.ring.benzene",
    )?;
    assert!(
        first.is_current(&e),
        "a fully validated named no-op must not cancel an armed gesture"
    );
    choose(&mut e, json!({"size":6}), 6, 4, true, "tool.ring.6")?;
    assert!(!first.is_current(&e));
    let sized = e.ring_context(1)?;
    choose(
        &mut e,
        json!({"kind":"benzene"}),
        6,
        5,
        true,
        "tool.ring.benzene",
    )?;
    assert!(
        !first.is_current(&e) && !sized.is_current(&e),
        "equal size is not an exact-mode proof"
    );
    let benzene = e.ring_context(1)?;
    choose(
        &mut e,
        json!({"kind":"cyclopentadiene"}),
        5,
        6,
        true,
        "tool.ring.cyclopentadiene",
    )?;
    assert!(!benzene.is_current(&e));
    let cp = e.ring_context(1)?;
    choose(
        &mut e,
        json!({"kind":"benzene"}),
        6,
        7,
        true,
        "tool.ring.benzene",
    )?;
    assert!(
        !cp.is_current(&e) && !benzene.is_current(&e),
        "kind A-B-A never revives its first context"
    );
    let returned = e.dispatch("ui.bond_tool", json!({"order":"triple"}))?;
    assert_eq!(
        serde_json::to_value(returned)?["ui_bond_tool_operation"],
        json!({"order":"triple","epoch":8,"changed":true})
    );
    active(&e, "tool.bond.triple")?;
    assert_eq!(authored(&e)?, original);
    Ok(())
}

// Stage3: existing APIs only; exact first4,850 bytes remain unchanged.
use bondline_commands::Command;
use bondline_io::StoredDocument;
use bondline_model::PageSize;
use std::fs;
#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(
    dead_code,
    reason = "reuse only the original isolated file-directory helper"
)]
mod support;
const NATIVE: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../io/tests/fixtures/bondline-v0.cdxml"
));

fn refused(e: &mut Engine, params: Value, code: &str) -> TestResult {
    let before = serde_json::to_value(e.state())?;
    let content = e.content().clone();
    let stamp = e.capture_stamp();
    let mode = e.effective_tool();
    let tool = e.bond_tool();
    let view = e.view();
    let file = e.ui_file_context_stamp();
    let generation = e.session_generation();
    let model = e.stored_document().ok().map(StoredDocument::model_snapshot);
    let error = e
        .dispatch("ui.ring_tool", params)
        .err()
        .ok_or("named tool refusal accepted")?;
    assert_eq!(error.info().code, code, "{error}");
    assert_eq!(serde_json::to_value(e.state())?, before);
    assert_eq!(e.content(), &content);
    assert_eq!(e.capture_stamp(), stamp);
    assert_eq!(e.effective_tool(), mode);
    assert_eq!(e.bond_tool(), tool);
    assert_eq!(e.view(), view);
    assert_eq!(e.session_generation(), generation);
    assert!(e.ui_file_context_stamp_matches(&file));
    if let Some(model) = model {
        assert!(Arc::ptr_eq(&model, &e.stored_document()?.model_snapshot()));
    }
    Ok(())
}

#[test]
fn named_contexts_keep_all_existing_source_view_file_close_and_aba_qualifiers() -> TestResult {
    for change in [
        "kind",
        "sized",
        "view",
        "file-request",
        "association",
        "source",
        "close",
        "host",
    ] {
        let directory = support::FileTestDirectory::new()?;
        let a = directory.path("named-context-A-原.cdxml");
        let mut e = Engine::from_document(StoredDocument::from_cdxml(NATIVE.into(), PageSize::A4)?);
        e.set_ui_file_host(true);
        e.dispatch("document.save_cdxml", json!({"path":a}))?;
        e.dispatch("ui.view", json!({"action":"actual"}))?;
        e.dispatch("ui.ring_tool", json!({"kind":"benzene"}))?;
        let model = serde_json::to_value(e.document()?)?;
        let bytes = fs::read(&a)?;
        let proof = e.ring_context(1)?;
        assert!(proof.is_current(&e));
        choose(
            &mut e,
            json!({"kind":"benzene"}),
            6,
            1,
            false,
            "tool.ring.benzene",
        )?;
        e.dispatch("document.inspect", json!({}))?;
        e.dispatch("ui.controls", json!({}))?;
        e.dispatch("ui.view", json!({"action":"inspect"}))?;
        e.dispatch("document.save_cdxml", json!({"path":a,"overwrite":true}))?;
        e.set_view_canvas_size(848, 728)?;
        assert!(
            proof.is_current(&e),
            "no-op/inspection/same-association Save and canvas size do not change the proof"
        );
        match change {
            "kind" => {
                e.dispatch("ui.ring_tool", json!({"kind":"cyclopentadiene"}))?;
                e.dispatch("ui.ring_tool", json!({"kind":"benzene"}))?;
            }
            "sized" => {
                e.dispatch("ui.ring_tool", json!({"size":6}))?;
                e.dispatch("ui.ring_tool", json!({"kind":"benzene"}))?;
            }
            "view" => {
                e.dispatch("ui.view", json!({"action":"set_zoom","percent":150}))?;
                e.dispatch("ui.view", json!({"action":"actual"}))?;
            }
            "file-request" => {
                let id = e
                    .dispatch(
                        "ui.file",
                        json!({"expected_revision":e.state().revision,"action":"save"}),
                    )?
                    .ui_file_operation
                    .ok_or("file request")?;
                assert!(e.ring_context(1).is_err());
                refused(&mut e, json!({"kind":"benzene"}), "ui_busy")?;
                let plan = e.claim_ui_file(id)?;
                assert!(e.ring_context(1).is_err());
                refused(&mut e, json!({"kind":"benzene"}), "ui_busy")?;
                let bondline_commands::UiFilePlan::SaveExisting { params, .. } = plan else {
                    return Err("captured SaveExisting plan required".into());
                };
                let work = e.prepare_ui_file(id, Command::SaveCdxml(params))?;
                assert!(
                    e.ring_context(1).is_err(),
                    "Running Save still blocks a native gesture"
                );
                choose(
                    &mut e,
                    json!({"kind":"benzene"}),
                    6,
                    1,
                    false,
                    "tool.ring.benzene",
                )?;
                drop(work);
                assert!(!e.finish_ui_file(id, false));
            }
            "association" => {
                let b = directory.path("named-context-B-λ.cdxml");
                e.dispatch("document.save_cdxml", json!({"path":b}))?;
                e.dispatch("document.save_cdxml", json!({"path":a,"overwrite":true}))?;
                assert_eq!(fs::read(b)?, bytes);
            }
            "source" => {
                e.dispatch("document.open_cdxml", json!({"path":a}))?;
                e.dispatch("ui.view", json!({"action":"actual"}))?;
            }
            "close" => {
                e.dispatch(
                    "document.close",
                    json!({"expected_revision":e.state().revision,"action":"request"}),
                )?;
                assert!(e.ring_context(1).is_err());
                refused(&mut e, json!({"kind":"benzene"}), "ui_busy")?;
                e.dispatch(
                    "document.close",
                    json!({"expected_revision":e.state().revision,"action":"cancel"}),
                )?;
            }
            _ => {
                e.set_ui_file_host(false);
                e.set_ui_file_host(true);
            }
        }
        assert_eq!(serde_json::to_value(e.document()?)?, model);
        assert_eq!(fs::read(&a)?, bytes);
        active(&e, "tool.ring.benzene")?;
        assert!(
            !proof.is_current(&e),
            "{change} ABA cannot revive old authority"
        );
        assert!(e.ring_context(1)?.is_current(&e));
    }
    Ok(())
}

fn schema_node<'a>(root: &'a Value, node: &'a Value) -> TestResult<&'a Value> {
    match node.get("$ref").and_then(Value::as_str) {
        None => Ok(node),
        Some(reference) => root
            .pointer(reference.strip_prefix('#').ok_or("nonlocal reference")?)
            .ok_or_else(|| "missing schema definition".into()),
    }
}
#[test]
fn named_union_schema_and_native31_foreign20_14_10_controls_preserve_closed_source_authority()
-> TestResult {
    let registry = bondline_commands::registry();
    assert_eq!(registry.len(), 53);
    let row = registry
        .iter()
        .find(|r| r.name == "ui.ring_tool")
        .ok_or("ring tool")?;
    assert!(!row.read_only);
    let schema = serde_json::to_value(&row.input_schema)?;
    assert_eq!(schema["type"], "object");
    let variants = schema
        .get("anyOf")
        .or_else(|| schema.get("oneOf"))
        .and_then(Value::as_array)
        .ok_or("selector alternatives")?;
    assert_eq!(variants.len(), 3);
    let mut selectors = std::collections::BTreeSet::new();
    for branch in variants {
        let branch = schema_node(&schema, branch)?;
        assert_eq!(branch["type"], "object");
        assert_eq!(branch["additionalProperties"], false);
        let props = branch["properties"].as_object().ok_or("fields")?;
        assert_eq!(props.len(), 1);
        if props.contains_key("size") {
            assert!(selectors.insert("size"));
            assert_eq!(branch["required"], json!(["size"]));
            assert_eq!(props["size"]["type"], "integer");
            assert_eq!(props["size"]["minimum"].as_f64(), Some(3.0));
            assert_eq!(props["size"]["maximum"].as_f64(), Some(8.0));
        } else if schema_node(&schema, props.get("kind").ok_or("kind selector")?)?["enum"]
            == json!(["cyclohexane_chair_1", "cyclohexane_chair_2"])
        {
            assert!(selectors.insert("chair"));
            assert_eq!(branch["required"], json!(["kind"]));
            assert_eq!(schema_node(&schema, &props["kind"])?["type"], "string");
        } else {
            assert!(selectors.insert("kind"));
            assert_eq!(branch["required"], json!(["kind"]));
            let kind = schema_node(&schema, props.get("kind").ok_or("kind")?)?;
            assert_eq!(kind["type"], "string");
            assert_eq!(kind["enum"], json!(["benzene", "cyclopentadiene"]));
        }
    }
    assert_eq!(
        selectors,
        std::collections::BTreeSet::from(["size", "kind", "chair"])
    );
    let mut e = Engine::default();
    assert_eq!(e.controls().len(), 40);
    for (kind, label, size) in [
        ("benzene", "Benzene", 6),
        ("cyclopentadiene", "Cyclopentadiene", 5),
    ] {
        let controls = e.controls();
        let id = format!("tool.ring.{kind}");
        let c = controls
            .iter()
            .find(|c| c.id == id)
            .ok_or("named binding")?;
        assert_eq!(c.label, label);
        assert_eq!(c.menu, "Tools");
        assert!(c.enabled && c.shortcut.is_none());
        assert_eq!(c.selected, Some(false));
        assert_eq!(
            serde_json::to_value(&c.command)?,
            json!({"command":"ui.ring_tool","params":{"kind":kind}})
        );
        assert_eq!(if kind == "benzene" { 6 } else { 5 }, size);
    }
    for value in [
        json!({"size":6}),
        json!({"kind":"benzene"}),
        json!({"kind":"cyclopentadiene"}),
    ] {
        assert_eq!(
            serde_json::to_value(Command::parse("ui.ring_tool", value.clone())?)?,
            json!({"command":"ui.ring_tool","params":value})
        );
    }
    for params in [
        json!({}),
        json!(null),
        json!([]),
        json!({"size":2}),
        json!({"size":9}),
        json!({"size":6,"kind":"benzene"}),
        json!({"kind":null}),
        json!({"kind":false}),
        json!({"kind":"Benzene"}),
        json!({"kind":{"benzene":null}}),
        json!({"kind":"benzene","phase":"single_first"}),
        json!({"kind":"benzene","extra":true}),
    ] {
        refused(&mut e, params, "invalid_params")?;
    }
    let directory = support::FileTestDirectory::new()?;
    for projected in [false, true] {
        let path = directory.path(if projected {
            "projected-原.smi"
        } else {
            "readonly-原.smi"
        });
        let bytes = b"CC first\r\nO\topaque\xff\0";
        let mut foreign = Engine::default();
        fs::write(&path, bytes)?;
        foreign.dispatch("document.open_smiles", json!({"path":path}))?;
        if projected {
            foreign.dispatch("document.import_smiles_records",json!({"expected_revision":1,"selection":{"mode":"all"},
            "bond_length_pt":48,"origin_x_pt":72,"origin_y_pt":72,"page_width_pt":210.0*72.0/25.4,"page_height_pt":297.0*72.0/25.4}))?;
        }
        let controls = foreign.controls();
        assert_eq!(controls.len(), if projected { 23 } else { 13 });
        let settings = controls
            .iter()
            .find(|row| row.id == "file.document_settings")
            .ok_or("Document Settings binding")?;
        assert!(!settings.enabled);
        assert_eq!(settings.disabled_reason, Some("ui_unavailable"));
        assert!(!controls.iter().any(|c| c.id.starts_with("tool.ring.")));
        refused(
            &mut foreign,
            json!({"kind":"benzene"}),
            "unsupported_content",
        )?;
        assert_eq!(fs::read(path)?, bytes);
    }
    let path = directory.path("readonly-全.sdf");
    let bytes = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf"
    ));
    fs::write(&path, bytes)?;
    let mut sdf = Engine::default();
    sdf.dispatch("document.open_sdf", json!({"path":path}))?;
    assert_eq!(sdf.controls().len(), 17);
    let settings = sdf
        .controls()
        .into_iter()
        .find(|row| row.id == "file.document_settings")
        .ok_or("Document Settings binding")?;
    assert!(!settings.enabled);
    assert_eq!(settings.disabled_reason, Some("ui_unavailable"));
    assert!(
        !sdf.controls()
            .iter()
            .any(|c| c.id.starts_with("tool.ring."))
    );
    refused(
        &mut sdf,
        json!({"kind":"cyclopentadiene"}),
        "unsupported_content",
    )?;
    assert_eq!(fs::read(path)?, bytes);
    let opaque = String::from_utf8(NATIVE.to_vec())?
        .replace("<CDXML ", "<CDXML Private=\"named-tool-source\" ");
    let mut opaque_engine = Engine::from_document(StoredDocument::from_cdxml(
        opaque.as_bytes().into(),
        PageSize::A4,
    )?);
    refused(
        &mut opaque_engine,
        json!({"kind":"benzene"}),
        "unsupported_content",
    )?;
    assert_eq!(
        opaque_engine.stored_document()?.export_cdxml()?,
        opaque.as_bytes()
    );
    e.dispatch(
        "document.close",
        json!({"expected_revision":0,"action":"discard"}),
    )?;
    assert!(e.complete_close());
    refused(&mut e, json!({"kind":"benzene"}), "close_committed")?;
    Ok(())
}
