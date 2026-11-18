//! Existing public registry APIs only, declared before command48 production.
use bondline_commands::Command;
#[path = "../../../xtask/oracles/registry_fixture_support.rs"]
mod registry_fixture;
use serde_json::{Value, json};
type TestResult = Result<(), Box<dyn std::error::Error>>;
#[test]
fn boxed_atom_command48_parses_the_closed_old_target_shape_with_the_qualified_registry_prefix()
-> TestResult {
    let command = Command::parse(
        "atom.set_authored_label",
        json!({"expected_revision":0,"page_id":1,"fragment_id":1,"atom_id":1,"element":7,"explicit_hydrogens":2}),
    )?;
    assert_eq!(
        serde_json::to_value(command)?,
        json!({"command":"atom.set_authored_label","params":{"expected_revision":0,"page_id":1,"fragment_id":1,"atom_id":1,"element":7,"explicit_hydrogens":2}})
    );
    let rows = serde_json::to_value(bondline_commands::registry())?;
    let rows = rows.as_array().ok_or("registry rows")?;
    let old: Vec<Value> = serde_json::from_str(&registry_fixture::with_cdx_caption_edit(
        include_str!("../../cli/tests/support/atom-child-old47.json"),
    ))?;
    assert_eq!(rows.len(), 49);
    assert_eq!(old.len(), 47);
    assert_eq!(&rows[..47], old.as_slice());
    assert_eq!(rows[47]["name"], "atom.set_authored_label");
    assert_eq!(rows[47]["read_only"], false);
    let schema = &rows[47]["input_schema"];
    assert_eq!(
        schema,
        &rows
            .iter()
            .find(|r| r["name"] == "atom.set_label")
            .ok_or("old atom row")?["input_schema"]
    );
    Ok(())
}
use bondline_commands::Engine;
use bondline_io::StoredDocument;
use bondline_model::PageSize;
use proptest::prelude::*;
use std::{borrow::Cow, sync::Arc};
#[path = "../../../xtask/oracles/boxed_label_support.rs"]
#[allow(
    dead_code,
    reason = "the shared actual-file vectors also serve IO/adapters/App tests"
)]
mod boxed;

proptest! {
    #![proptest_config(ProptestConfig {
        cases:24,
        rng_seed:proptest::test_runner::RngSeed::Fixed(671308429),
        failure_persistence:None,
        ..ProptestConfig::default()
    })]
    #[test]
    fn generated_literal_label_edits_restore_complete_source_snapshots(
        choices in prop::collection::vec((1_u8..=118,any::<u16>()),1..5)
    ) {
        let mut engine=boxed_engine(boxed::SOURCE)
            .map_err(|error|TestCaseError::fail(error.to_string()))?;
        let mut snapshots=vec![engine.content().clone()];
        let mut encoded=vec![boxed::SOURCE.as_bytes().to_vec()];
        for (element,hydrogens) in choices {
            let h=if element==1 {0} else {hydrogens};
            let result=engine.dispatch("atom.set_authored_label",
                boxed::params(engine.state().revision,element,h))?;
            let raw=engine.stored_document()?.to_cdxml()?.into_owned();
            let facts=boxed::read(&raw).map_err(|error|TestCaseError::fail(error.to_string()))?;
            prop_assert_eq!(&facts["element"],&json!(element));
            prop_assert_eq!(&facts["hydrogens"],&json!(h));
            prop_assert_eq!(&facts["point"],&json!(boxed::POINT));
            if result.changed {
                snapshots.push(engine.content().clone());
                encoded.push(raw);
            } else {
                prop_assert_eq!(engine.content(),snapshots.last().ok_or_else(||TestCaseError::fail("snapshot"))?);
            }
        }
        for (snapshot,raw) in snapshots.iter().zip(&encoded).rev().skip(1) {
            engine.dispatch("history.undo",json!({}))?;
            prop_assert_eq!(engine.content(),snapshot);
            prop_assert_eq!(&engine.stored_document()?.to_cdxml()?.into_owned(),raw);
        }
        for (snapshot,raw) in snapshots.iter().zip(&encoded).skip(1) {
            engine.dispatch("history.redo",json!({}))?;
            prop_assert_eq!(engine.content(),snapshot);
            prop_assert_eq!(&engine.stored_document()?.to_cdxml()?.into_owned(),raw);
        }
    }
}

fn boxed_engine(raw: &str) -> boxed::Result<Engine> {
    Ok(Engine::from_document(StoredDocument::from_cdxml(
        raw.as_bytes().into(),
        PageSize::A4,
    )?))
}
fn unchanged_error(e: &mut Engine, name: &str, params: Value, code: &str) -> TestResult {
    let state = serde_json::to_value(e.state())?;
    let content = e.content().clone();
    let stamp = e.capture_stamp();
    let file = e.ui_file_context_stamp();
    let model = e.stored_document()?.model_snapshot();
    let error = e
        .dispatch(name, params)
        .err()
        .ok_or_else(|| format!("{name} unexpectedly acquired authority"))?;
    assert_eq!(error.info().code, code, "{name}: {error}");
    assert_eq!(serde_json::to_value(e.state())?, state);
    assert_eq!(e.content(), &content);
    assert_eq!(e.capture_stamp(), stamp);
    assert!(e.ui_file_context_stamp_matches(&file));
    assert!(Arc::ptr_eq(&model, &e.stored_document()?.model_snapshot()));
    Ok(())
}

#[test]
fn authored_command_couples_numeric_body_and_box_in_one_source_preserving_history_edit()
-> TestResult {
    assert_eq!(boxed::read(boxed::SOURCE.as_bytes())?, boxed::facts(8, 1));
    assert_eq!(boxed::read(boxed::edited().as_bytes())?, boxed::facts(7, 2));
    let mut e = boxed_engine(boxed::SOURCE)?;
    assert!(e.stored_document()?.has_boxed_atom_labels());
    assert!(!e.stored_document()?.is_fully_mapped());
    let baseline = e.content().clone();
    let initial_model = e.stored_document()?.model_snapshot();
    let initial_stamp = e.capture_stamp();
    let initial_file = e.ui_file_context_stamp();
    let no = e.dispatch("atom.set_authored_label", boxed::params(0, 8, 1))?;
    assert!(!no.changed);
    assert_eq!((no.revision, no.undo_depth, no.redo_depth), (0, 0, 0));
    assert!(Arc::ptr_eq(
        &initial_model,
        &e.stored_document()?.model_snapshot()
    ));
    assert_eq!(e.capture_stamp(), initial_stamp);
    assert!(e.ui_file_context_stamp_matches(&initial_file));
    let result = e.dispatch("atom.set_authored_label", boxed::params(0, 7, 2))?;
    assert!(result.changed && result.file.dirty);
    assert_eq!(
        (result.revision, result.undo_depth, result.redo_depth),
        (1, 1, 0)
    );
    let model = serde_json::to_value(e.document()?)?;
    let atom = &model["pages"][0]["fragments"][0]["atoms"][0];
    assert_eq!(atom["id"], 1);
    assert_eq!(atom["element"], 7);
    assert_eq!(atom["explicit_hydrogens"], 2);
    assert_eq!(
        atom["position"],
        json!({"x_pt":boxed::POINT[0],"y_pt":boxed::POINT[1]})
    );
    assert_eq!(atom["authored_label"], boxed::label(7, 2));
    assert_eq!(
        e.stored_document()?.to_cdxml()?.as_ref(),
        boxed::edited().as_bytes()
    );
    assert_eq!(
        boxed::read(e.stored_document()?.to_cdxml()?.as_ref())?,
        boxed::facts(7, 2)
    );
    let changed = e.content().clone();
    e.dispatch("history.undo", json!({}))?;
    assert_eq!(e.content(), &baseline);
    assert!(matches!(e.stored_document()?.to_cdxml()?, Cow::Borrowed(_)));
    assert_eq!(
        e.stored_document()?.export_cdxml()?,
        boxed::SOURCE.as_bytes()
    );
    let after_undo = e.stored_document()?.model_snapshot();
    let stamp = e.capture_stamp();
    let file = e.ui_file_context_stamp();
    let result = e.dispatch("atom.set_authored_label", boxed::params(2, 8, 1))?;
    assert!(!result.changed);
    assert_eq!(
        (result.revision, result.undo_depth, result.redo_depth),
        (2, 0, 1)
    );
    assert!(Arc::ptr_eq(
        &after_undo,
        &e.stored_document()?.model_snapshot()
    ));
    assert_eq!(e.capture_stamp(), stamp);
    assert!(e.ui_file_context_stamp_matches(&file));
    e.dispatch("history.redo", json!({}))?;
    assert_eq!(e.content(), &changed);
    assert_eq!(
        e.stored_document()?.to_cdxml()?.as_ref(),
        boxed::edited().as_bytes()
    );
    Ok(())
}

#[test]
fn command48_retains_chemical_guards_while_selection_and_zero_motion_are_qualified() -> TestResult {
    let mut e = boxed_engine(boxed::SOURCE)?;
    for (name, p) in [
        ("atom.set_label", boxed::params(0, 8, 1)),
        ("atom.set_label", boxed::params(0, 7, 2)),
        (
            "page.set_size",
            json!({"page_id":1,"width_pt":512,"height_pt":512}),
        ),
        (
            "page.set_size",
            json!({"page_id":1,"width_pt":400,"height_pt":500}),
        ),
        (
            "bond.create",
            json!({"page_id":1,"begin_x_pt":160,"begin_y_pt":200,"end_x_pt":200,"end_y_pt":200}),
        ),
        (
            "bond.extend",
            json!({"expected_revision":0,"page_id":1,"fragment_id":1,"begin_atom_id":1,"end_x_pt":160,"end_y_pt":100,"hydrogen_policy":"rectify_terminal_carbon"}),
        ),
        (
            "bond.set_order",
            json!({"expected_revision":0,"page_id":1,"fragment_id":1,"bond_id":1,"order":"single","hydrogen_policy":"rectify_neutral_carbon"}),
        ),
        (
            "document.set_drawing_settings",
            json!({"expected_revision":0,"bond_length_pt":null,"chain_angle_degrees":null}),
        ),
        (
            "document.set_document_settings",
            json!({"expected_revision":0,"bond_length_pt":null,"chain_angle_degrees":null,"line_width_pt":null}),
        ),
        (
            "text.set_run",
            json!({"expected_revision":0,"page_id":1,"text_id":1,"run_index":0,"text":"OH"}),
        ),
    ] {
        unchanged_error(&mut e, name, p, "unsupported_content")?;
    }
    unchanged_error(
        &mut e,
        "selection.translate",
        json!({"expected_revision":0,"expected_selection_epoch":0,"delta_x_pt":0,"delta_y_pt":0}),
        "stale_selection",
    )?;
    let content = e.content().clone();
    let model = e.stored_document()?.model_snapshot();
    let stamp = e.capture_stamp();
    let selected = e.dispatch(
        "ui.selection",
        json!({"expected_revision":0,"action":"all","page_id":1}),
    )?;
    assert!(!selected.changed);
    assert_eq!(e.selection().atom_ids, vec![1]);
    assert_eq!(e.content(), &content);
    assert_eq!(e.capture_stamp(), stamp);
    assert!(Arc::ptr_eq(&model, &e.stored_document()?.model_snapshot()));
    let selected_state = serde_json::to_value(e.state())?;
    assert!(!e.dispatch(
        "selection.translate",
        json!({"expected_revision":0,"expected_selection_epoch":e.selection().epoch,"delta_x_pt":0,"delta_y_pt":0}),
    )?.changed);
    assert_eq!(serde_json::to_value(e.state())?, selected_state);
    assert!(Arc::ptr_eq(&model, &e.stored_document()?.model_snapshot()));
    unchanged_error(
        &mut e,
        "atom.set_authored_label",
        boxed::params(1, 7, 2),
        "stale_revision",
    )?;
    assert_eq!(
        e.stored_document()?.to_cdxml()?.as_ref(),
        boxed::SOURCE.as_bytes()
    );
    Ok(())
}

#[test]
fn unsupported_child_or_source_cannot_gain_authored_noop_authority_and_keeps_raw_bytes()
-> TestResult {
    let mut variants = vec![
        boxed::SOURCE.replace(">OH</s>", ">N</s>"),
        boxed::SOURCE.replace("font=\"42\"", "font=\"43\""),
        boxed::SOURCE.replace("face=\"0\"", "face=\"96\""),
        boxed::SOURCE.replace("size=\"12\"", "size=\"24\""),
        boxed::SOURCE.replace("Justification=\"Left\"", "Justification=\"Auto\""),
        boxed::SOURCE.replace("<t id=\"21\"", "<t id=\"21\" p='80 100'"),
        boxed::SOURCE.replace(
            "80.69140625 91.3515625 95.796875 100.1171875",
            "80.69140625 91.3515625 95.79690625 100.1171875",
        ),
        boxed::SOURCE.replace(
            " BoundingBox=\"80.69140625 91.3515625 95.796875 100.1171875\"",
            "",
        ),
        boxed::SOURCE.replace("<CDXML ", "<CDXML Future='opaque' "),
        boxed::SOURCE.replace("<t id=\"21\"", "<t id=\"21\" Future='opaque'"),
        boxed::SOURCE.replace("<s font=\"42\"", "<s Unknown='opaque' font=\"42\""),
        boxed::SOURCE.replace("</fragment>", "<n id='97' NodeType='Element' Element='7' Charge='0' Radical='None' NumHydrogens='0' p='220 220'/></fragment>"),
        boxed::SOURCE.replace("</fragment>", "<n id='97' NodeType='Element' Element='8' Charge='0' Radical='None' NumHydrogens='1' p='184.1220703125 95.734375'><t id='21' BoundingBox='180.69140625 91.3515625 195.796875 100.1171875' Justification='Left' LabelJustification='Left'><s font='42' size='12' face='0' color='4'>OH</s></t></n><b id='900' B='97' E='73' Order='1'/></fragment>"),
    ];
    variants.push(boxed::SOURCE.replace("<CDXML ", "<CDXML xmlns:x='urn:unused' "));
    for raw in variants {
        let mut e = boxed_engine(&raw)?;
        assert!(!e.stored_document()?.has_boxed_atom_labels());
        unchanged_error(
            &mut e,
            "atom.set_authored_label",
            boxed::params(0, 8, 1),
            "unsupported_content",
        )?;
        assert_eq!(e.stored_document()?.to_cdxml()?.as_ref(), raw.as_bytes());
    }
    Ok(())
}

#[test]
fn source_scoped_id_and_lexical_variants_keep_each_original_byte_outside_owned_numeric_body_and_box_fields()
-> TestResult {
    let variants = [
        boxed::SOURCE.to_owned(),
        boxed::SOURCE
            .replace('"', "'")
            .replace("Element='8'", "Element='08'")
            .replace("NumHydrogens='1'", "NumHydrogens='01'")
            .replace("<page ", "<!-- source comment -->\r\n<page "),
        boxed::SOURCE.replace("<t id=\"21\"", "<t id=\"73\""),
        boxed::SOURCE
            .replace("<font id=\"42\"", "<font id=\"73\"")
            .replace("font=\"42\"", "font=\"73\""),
    ];
    for raw in variants {
        let mut e = boxed_engine(&raw)?;
        assert!(e.stored_document()?.has_boxed_atom_labels(), "{raw}");
        let before = e.content().clone();
        let no = e.dispatch("atom.set_authored_label", boxed::params(0, 8, 1))?;
        assert!(!no.changed);
        assert_eq!(e.content(), &before);
        assert_eq!(e.stored_document()?.to_cdxml()?.as_ref(), raw.as_bytes());
        e.dispatch("atom.set_authored_label", boxed::params(0, 7, 2))?;
        let expected = raw
            .replace("Element=\"8\"", "Element=\"7\"")
            .replace("Element='08'", "Element='7'")
            .replace("NumHydrogens=\"1\"", "NumHydrogens=\"2\"")
            .replace("NumHydrogens='01'", "NumHydrogens='2'")
            .replace(
                "80.69140625 91.3515625 95.796875 100.1171875",
                "80.84375 91.3515625 103.26171875 100",
            )
            .replace(">OH</s>", ">NH2</s>");
        assert_eq!(
            e.stored_document()?.to_cdxml()?.as_ref(),
            expected.as_bytes()
        );
        assert_eq!(boxed::read(expected.as_bytes())?, boxed::facts(7, 2));
        e.dispatch("history.undo", json!({}))?;
        assert_eq!(e.content(), &before);
        assert_eq!(e.stored_document()?.to_cdxml()?.as_ref(), raw.as_bytes());
    }
    Ok(())
}

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse original isolated actual-file directory")]
mod boxed_files;
#[test]
fn captured_boxed_save_holds_its_old_complete_snapshot_and_never_acknowledges_the_newer_authored_edit()
-> TestResult {
    let directory = boxed_files::FileTestDirectory::new()?;
    let input = directory.path("boxed-lease-original-λ.cdxml");
    let output = directory.path("boxed-lease-captured-保存.cdxml");
    let unused = directory.path("boxed-lease-unused.cdxml");
    std::fs::write(&input, boxed::SOURCE)?;
    let mut e = Engine::default();
    e.dispatch("document.open_cdxml", json!({"path":input}))?;
    let pending = e.prepare(Command::parse(
        "document.save_cdxml",
        json!({"path":output}),
    )?)?;
    e.dispatch("atom.set_authored_label", boxed::params(1, 7, 2))?;
    let current = e.stored_document()?.model_snapshot();
    let another = || Command::parse("document.save_cdxml", json!({"path":unused}));
    assert_eq!(
        e.prepare(another()?)
            .err()
            .ok_or("captured save lost reservation")?
            .info()
            .code,
        "busy"
    );
    let completed = pending.complete()?;
    let raw = std::fs::read(&output)?;
    assert_eq!(raw, boxed::SOURCE.as_bytes());
    assert_eq!(boxed::read(&raw)?, boxed::facts(8, 1));
    assert_eq!(
        e.prepare(another()?)
            .err()
            .ok_or("completed save dropped reservation before finish")?
            .info()
            .code,
        "busy"
    );
    let result = e.finish(completed)?;
    assert!(!result.changed && result.file.dirty && result.source.ok_or("source")?.modified);
    let receipt = result.file_operation.ok_or("save receipt")?;
    assert_eq!(
        (receipt.input_revision, receipt.byte_length),
        (1, boxed::SOURCE.len())
    );
    assert!(receipt.ui_request_id.is_none() && receipt.ui_request_current.is_none());
    assert!(Arc::ptr_eq(
        &current,
        &e.stored_document()?.model_snapshot()
    ));
    assert_eq!(
        e.stored_document()?.to_cdxml()?.as_ref(),
        boxed::edited().as_bytes()
    );
    e.dispatch("atom.set_authored_label", boxed::params(2, 8, 1))?;
    assert!(!e.state().file.dirty, "captured OH is the saved baseline");
    drop(e.prepare(another()?)?);
    assert!(!unused.exists());
    assert_eq!(std::fs::read(&input)?, boxed::SOURCE.as_bytes());
    Ok(())
}
