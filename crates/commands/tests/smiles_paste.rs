use bondline_commands::Command;
use serde_json::json;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn editable_smiles_paste_is_a_named_typed_authoring_command() -> TestResult {
    let command = Command::parse(
        "structure.paste_smiles",
        json!({
            "expected_revision":0,
            "page_id":1,
            "notation":"c1ccccc1",
            "bond_length_pt":64.0,
            "origin_x_pt":96.0,
            "origin_y_pt":96.0
        }),
    )?;
    assert_eq!(command.name(), "structure.paste_smiles");
    Ok(())
}

use bondline_commands::Engine;
use bondline_io::StoredDocument;
use bondline_model::{Fragment, PageSize};
use serde_json::Value;
use std::{
    fs,
    io::Write,
    process::{Command as Process, Stdio},
    sync::Arc,
};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse the existing isolated file directory")]
mod support;

const NAME: &str = "structure.paste_smiles";

fn params(revision: u64, notation: &str) -> Value {
    json!({"expected_revision":revision,"page_id":1,"notation":notation,
        "bond_length_pt":64.0,"origin_x_pt":96.0,"origin_y_pt":96.0})
}

fn native_pair() -> TestResult<(Engine, Vec<u8>)> {
    let mut engine = Engine::default();
    engine.dispatch(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":72.0,
        "begin_y_pt":480.0,"end_x_pt":120.0,"end_y_pt":480.0}),
    )?;
    let original = engine.stored_document()?.to_cdxml()?.into_owned();
    let stored = StoredDocument::from_cdxml(original.as_slice().into(), PageSize::A4)?;
    assert!(stored.is_fully_mapped());
    Ok((Engine::from_document(stored), original))
}

fn consume(fragment: &Fragment, source: &str, edited: bool) -> TestResult {
    let mut atoms = vec![json!([6, null, 1]); 6];
    let mut bonds = json!([
        [1, 2, 2],
        [2, 3, 1],
        [3, 4, 2],
        [4, 5, 1],
        [5, 6, 2],
        [1, 6, 1]
    ]);
    if edited {
        atoms[0] = json!([6, null, 2]);
        atoms[1] = json!([6, null, 2]);
        bonds[0][2] = json!(1);
    }
    let expected = json!({"atoms":atoms,"bonds":bonds,
        "formula":if edited {"C6H8"} else {"C6H6"},
        "components":[[1,2,3,4,5,6]]});
    let mut child = Process::new(std::env::var_os("BONDLINE_PYTHON").ok_or("BONDLINE_PYTHON")?)
        .arg("-B")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../io/tests/smiles_authoring_oracle.py"
        ))
        .arg(source)
        .arg(expected.to_string())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("consumer stdin")?
        .write_all(&serde_json::to_vec(fragment)?)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(value["raw_graph"]["atoms"], expected["atoms"]);
    assert_eq!(value["raw_graph"]["bonds"], expected["bonds"]);
    assert_eq!(value["no_assigned_stereo"], true);
    assert_eq!(value["no_radicals"], true);
    Ok(())
}

#[test]
fn deferred_benzene_paste_preserves_complete_native_source_and_one_step_history_then_saves()
-> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let (mut engine, original) = native_pair()?;
    let before = serde_json::to_value(engine.state())?;
    let owner = engine.content().clone();
    let stamp = engine.capture_stamp();
    let old_fragment = Arc::clone(&engine.document()?.pages()[0].fragments()[0]);
    let prepared = engine.prepare(Command::parse(NAME, params(0, "c1ccccc1"))?)?;
    assert_eq!(serde_json::to_value(engine.state())?, before);
    let completed = prepared.complete()?;
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert!(engine.capture_stamp_matches(&stamp));
    let result = engine.finish(completed)?;
    assert!(result.changed);
    assert_eq!(result.revision, 1);
    assert_eq!(result.undo_depth, 1);
    assert!(result.file.dirty);
    assert!(result.file_operation.is_none());
    assert!(result.conversion.is_none());
    let page = &engine.document()?.pages()[0];
    assert_eq!(page.size(), PageSize::A4);
    assert_eq!(page.fragments().len(), 2);
    assert!(Arc::ptr_eq(&old_fragment, &page.fragments()[0]));
    let pasted = &page.fragments()[1];
    assert_eq!(pasted.id().get(), 2);
    assert_eq!(
        pasted
            .atoms()
            .iter()
            .map(|a| a.id.get())
            .collect::<Vec<_>>(),
        (3..=8).collect::<Vec<_>>()
    );
    assert_eq!(
        pasted
            .bonds()
            .iter()
            .map(|b| b.id.get())
            .collect::<Vec<_>>(),
        (2..=7).collect::<Vec<_>>()
    );
    assert_eq!(pasted.atoms().len(), 6);
    assert_eq!(pasted.bonds().len(), 6);
    assert!(
        pasted
            .atoms()
            .iter()
            .all(|a| a.position.is_some() && a.explicit_hydrogens == Some(1))
    );
    consume(pasted, "c1ccccc1", false)?;
    let whole = engine.content().clone();
    let bytes = engine.stored_document()?.to_cdxml()?.into_owned();
    engine.dispatch("history.undo", json!({}))?;
    assert_eq!(engine.content(), &owner);
    assert!(!engine.state().file.dirty);
    assert_eq!(engine.stored_document()?.to_cdxml()?.as_ref(), original);
    engine.dispatch("history.redo", json!({}))?;
    assert_eq!(engine.content(), &whole);
    assert_eq!(engine.stored_document()?.to_cdxml()?.as_ref(), bytes);
    engine.dispatch(
        "bond.set_order",
        json!({"expected_revision":engine.state().revision,
        "page_id":1,"fragment_id":2,"bond_id":2,"order":"single",
        "hydrogen_policy":"rectify_neutral_carbon"}),
    )?;
    let edited = engine.document()?.clone();
    consume(&edited.pages()[0].fragments()[1], "C1CC=CC=C1", true)?;
    let output = directory.path("pasted-λ.cdxml");
    engine.dispatch("document.save_cdxml", json!({"path":output}))?;
    let saved = fs::read(&output)?;
    assert_ne!(saved, original);
    engine.dispatch("document.open_cdxml", json!({"path":output}))?;
    assert_eq!(engine.document()?, &edited);
    assert!(engine.stored_document()?.is_fully_mapped());
    assert!(!engine.state().file.dirty);
    consume(
        &engine.document()?.pages()[0].fragments()[1],
        "C1CC=CC=C1",
        true,
    )?;
    assert!(Arc::ptr_eq(
        &old_fragment,
        &edited.pages()[0].fragments()[0]
    ));
    Ok(())
}

#[test]
fn paste_keeps_all_components_and_new_branches_never_recycle_pasted_ids() -> TestResult {
    let (mut engine, _) = native_pair()?;
    engine.dispatch(NAME, params(0, "c1ccccc1.CC"))?;
    let page = &engine.document()?.pages()[0];
    assert_eq!(page.fragments().len(), 2);
    let pasted = &page.fragments()[1];
    assert_eq!(pasted.atoms().len(), 8);
    assert_eq!(pasted.bonds().len(), 7);
    assert_eq!(
        pasted
            .atoms()
            .iter()
            .map(|a| a.explicit_hydrogens)
            .collect::<Vec<_>>(),
        vec![
            Some(1),
            Some(1),
            Some(1),
            Some(1),
            Some(1),
            Some(1),
            Some(3),
            Some(3)
        ]
    );
    let high_atom = pasted
        .atoms()
        .iter()
        .map(|a| a.id.get())
        .max()
        .ok_or("atom")?;
    let high_bond = pasted
        .bonds()
        .iter()
        .map(|b| b.id.get())
        .max()
        .ok_or("bond")?;
    let high_fragment = pasted.id().get();
    engine.dispatch("history.undo", json!({}))?;
    engine.dispatch(NAME, params(engine.state().revision, "CC"))?;
    let fresh = &engine.document()?.pages()[0].fragments()[1];
    assert!(fresh.id().get() > high_fragment);
    assert!(fresh.atoms().iter().all(|a| a.id.get() > high_atom));
    assert!(fresh.bonds().iter().all(|b| b.id.get() > high_bond));
    assert_eq!(engine.state().redo_depth, 0);
    Ok(())
}

#[test]
fn cancelled_stale_foreign_and_close_aba_candidates_never_mutate_their_target() -> TestResult {
    let (mut engine, _) = native_pair()?;
    let before = serde_json::to_value(engine.state())?;
    let prepared = engine.prepare(Command::parse(NAME, params(0, "c1ccccc1"))?)?;
    let cancel = bondline_commands::Cancellation::default();
    cancel.cancel();
    let error = prepared
        .complete_with_cancel(&cancel)
        .err()
        .ok_or("cancelled work succeeded")?;
    assert_eq!(error.info().code, "cancelled");
    assert_eq!(serde_json::to_value(engine.state())?, before);
    let completed = engine
        .prepare(Command::parse(NAME, params(0, "c1ccccc1"))?)?
        .complete()?;
    let (mut foreign, _) = native_pair()?;
    assert_eq!(
        foreign
            .finish(completed)
            .err()
            .ok_or("foreign candidate")?
            .info()
            .code,
        "stale_revision"
    );
    assert_eq!(serde_json::to_value(foreign.state())?, before);
    let completed = engine
        .prepare(Command::parse(NAME, params(0, "c1ccccc1"))?)?
        .complete()?;
    engine.dispatch(
        "document.close",
        json!({"expected_revision":0,"action":"request"}),
    )?;
    engine.dispatch(
        "document.close",
        json!({"expected_revision":0,"action":"cancel"}),
    )?;
    let after_close = serde_json::to_value(engine.state())?;
    assert!(
        engine.finish(completed).is_err(),
        "close ABA cannot revive work"
    );
    assert_eq!(serde_json::to_value(engine.state())?, after_close);
    let completed = engine
        .prepare(Command::parse(NAME, params(0, "c1ccccc1"))?)?
        .complete()?;
    engine.dispatch(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":320.0,
        "begin_y_pt":480.0,"end_x_pt":380.0,"end_y_pt":480.0}),
    )?;
    let authored = serde_json::to_value(engine.state())?;
    assert_eq!(
        engine
            .finish(completed)
            .err()
            .ok_or("stale candidate")?
            .info()
            .code,
        "stale_revision"
    );
    assert_eq!(serde_json::to_value(engine.state())?, authored);
    Ok(())
}

#[test]
fn unsupported_invalid_overlapping_and_wrong_source_pastes_are_atomic() -> TestResult {
    let (mut engine, original) = native_pair()?;
    let before = serde_json::to_value(engine.state())?;
    for source in [
        "CC PRIVATE",
        "CC\nO",
        "c1cccc1",
        "[NH4+]",
        "[nH]1CCCC1",
        "[13cH]1ccccc1",
    ] {
        assert!(
            engine.dispatch(NAME, params(0, source)).is_err(),
            "{source}"
        );
        assert_eq!(serde_json::to_value(engine.state())?, before);
        assert_eq!(engine.stored_document()?.to_cdxml()?.as_ref(), original);
    }
    let mut overlapping = params(0, "CC");
    overlapping["origin_x_pt"] = json!(72.0);
    overlapping["origin_y_pt"] = json!(480.0);
    assert!(
        engine.dispatch(NAME, overlapping).is_err(),
        "occupied old atoms must refuse"
    );
    let mut outside = params(0, "c1ccccc1");
    outside["origin_x_pt"] = json!(580.0);
    assert!(
        engine.dispatch(NAME, outside).is_err(),
        "whole ring must fit"
    );
    let mut oversized = params(0, "C");
    oversized["notation"] = json!("C".repeat(65_537));
    assert_eq!(
        engine
            .dispatch(NAME, oversized)
            .err()
            .ok_or("notation bound")?
            .info()
            .code,
        "resource_limit"
    );
    assert_eq!(serde_json::to_value(engine.state())?, before);
    let opaque =
        String::from_utf8(original)?.replacen("<CDXML ", "<CDXML Future=\"retain me\" ", 1);
    let mut foreign = Engine::from_document(StoredDocument::from_cdxml(
        opaque.as_bytes().into(),
        PageSize::A4,
    )?);
    assert!(!foreign.stored_document()?.is_fully_mapped());
    let before = serde_json::to_value(foreign.state())?;
    assert_eq!(
        foreign
            .dispatch(NAME, params(0, "c1ccccc1"))
            .err()
            .ok_or("opaque source")?
            .info()
            .code,
        "unsupported_content"
    );
    assert_eq!(serde_json::to_value(foreign.state())?, before);
    assert_eq!(
        foreign.stored_document()?.to_cdxml()?.as_ref(),
        opaque.as_bytes()
    );
    let directory = support::FileTestDirectory::new()?;
    let path = directory.path("complete.smi");
    fs::write(&path, b"CC\tPRIVATE\xff\0\r\nO\n")?;
    let mut collection = Engine::default();
    collection.dispatch("document.open_smiles", json!({"path":path}))?;
    let before = serde_json::to_value(collection.state())?;
    assert_eq!(
        collection
            .dispatch(NAME, params(collection.state().revision, "c1ccccc1"))
            .err()
            .ok_or("SMILES source replaced")?
            .info()
            .code,
        "unsupported_content"
    );
    assert_eq!(serde_json::to_value(collection.state())?, before);
    Ok(())
}

#[test]
fn paste_schema_is_closed_and_private_notation_is_not_in_debug_or_automatic_state() -> TestResult {
    let registry = bondline_commands::registry();
    let entry = registry
        .iter()
        .find(|entry| entry.name == NAME)
        .ok_or("paste registry")?;
    assert!(!entry.read_only);
    let schema = entry.input_schema.as_value();
    assert_eq!(schema["additionalProperties"], false);
    let mut required = schema["required"]
        .as_array()
        .ok_or("required")?
        .iter()
        .map(|value| value.as_str().ok_or("required key"))
        .collect::<Result<Vec<_>, _>>()?;
    required.sort_unstable();
    assert_eq!(
        required,
        [
            "bond_length_pt",
            "expected_revision",
            "notation",
            "origin_x_pt",
            "origin_y_pt",
            "page_id"
        ]
    );
    let command = Command::parse(NAME, params(0, "PRIVATE_NOTATION"))?;
    assert!(!format!("{command:?}").contains("PRIVATE_NOTATION"));
    for key in ["file_path", "profile", "hydrogen_policy"] {
        let mut value = params(0, "c1ccccc1");
        value[key] = json!("not declared");
        assert_eq!(
            Command::parse(NAME, value)
                .err()
                .ok_or("unknown field")?
                .info()
                .code,
            "invalid_params"
        );
    }
    let (mut engine, _) = native_pair()?;
    let prepared = engine.prepare(Command::parse(NAME, params(0, "c1ccccc1"))?)?;
    assert!(!format!("{prepared:?}").contains("c1ccccc1"));
    let completed = prepared.complete()?;
    assert!(!format!("{completed:?}").contains("c1ccccc1"));
    let state = serde_json::to_string(&engine.state())?;
    assert!(!state.contains("c1ccccc1"));
    Ok(())
}
