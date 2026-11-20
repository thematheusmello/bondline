//! Original explicit creation contract; exercise the existing dispatch and modal APIs.
#[path = "../../../xtask/oracles/registry_fixture_support.rs"]
mod registry_fixture;
use bondline_commands::Engine;
use bondline_io::StoredDocument;
use bondline_model::PageSize;
use serde_json::{Value, json};
use std::sync::Arc;
#[path = "../../../xtask/oracles/created_label_support.rs"]
#[allow(dead_code, reason = "independent frozen source/model/history vectors")]
mod f;
#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "existing isolated real-file directory")]
mod files;
#[path = "../../../xtask/oracles/mixed_label_support.rs"]
#[allow(dead_code, reason = "shared original source and independent reader")]
mod mixed;
type TestResult = mixed::Result;

#[test]
fn creation_appends_one_closed_typed_command_and_preserves_all_old48_records() -> TestResult {
    let rows = serde_json::to_value(bondline_commands::registry())?;
    let rows = rows.as_array().ok_or("registry")?;
    let old: Vec<Value> =
        serde_json::from_str(&registry_fixture::with_cdx_caption_edit(mixed::METADATA48))?;
    assert_eq!((old.len(), rows.len()), (48, 53));
    assert_eq!(&rows[..48], old.as_slice());
    assert_eq!(rows[48]["name"], "atom.create_authored_label");
    assert_eq!(rows[48]["read_only"], false);
    let schema = &rows[48]["input_schema"];
    assert_eq!(schema["additionalProperties"], false);
    assert_eq!(
        schema["required"],
        json!([
            "expected_revision",
            "page_id",
            "fragment_id",
            "atom_id",
            "element",
            "explicit_hydrogens",
            "style"
        ])
    );
    assert_eq!(
        schema["$defs"]["AtomLabelStyle"]["additionalProperties"],
        false
    );
    let e = engine()?;
    let valid = params(&e, 3);
    for key in [
        "expected_revision",
        "page_id",
        "fragment_id",
        "atom_id",
        "element",
        "explicit_hydrogens",
        "style",
    ] {
        let mut invalid = valid.clone();
        invalid.as_object_mut().ok_or("params")?.remove(key);
        assert!(bondline_commands::Command::parse("atom.create_authored_label", invalid).is_err());
    }
    for invalid in [json!([]), Value::Null, json!({"extra":true})] {
        let mut p = valid.clone();
        p["style"] = invalid;
        assert!(bondline_commands::Command::parse("atom.create_authored_label", p).is_err());
    }
    Ok(())
}

fn engine() -> mixed::Result<Engine> {
    Ok(Engine::from_document(StoredDocument::from_cdxml(
        mixed::source("amino_alcohol").as_bytes().into(),
        PageSize::A4,
    )?))
}

fn original(name: &str) -> mixed::Result<Engine> {
    Ok(Engine::from_document(StoredDocument::from_cdxml(
        f::source(name).as_bytes().into(),
        PageSize::A4,
    )?))
}

fn exact(e: &Engine, baseline: &Value, name: &str) -> TestResult {
    let document = e.stored_document()?;
    assert!(!document.is_fully_mapped());
    assert!(!document.has_boxed_atom_labels());
    assert!(!document.has_connected_atom_labels());
    assert!(document.has_display_projection());
    assert_eq!(
        document.has_flagged_atom_labels(),
        f::facts(name)["flagged_expected"] == true
    );
    let bytes = document.to_cdxml()?;
    assert_eq!(bytes.as_ref(), f::source(name).as_bytes());
    assert_eq!(f::read(bytes.as_ref())?, f::facts(name));
    assert_eq!(
        serde_json::to_value(document.model())?,
        f::model_for_case(baseline, name)?
    );
    Ok(())
}

fn create(e: &mut Engine, id: u32) -> TestResult {
    let p = params(e, id);
    assert!(e.dispatch("atom.create_authored_label", p)?.changed);
    Ok(())
}

#[test]
fn frozen_source_permutations_reopen_and_generated_history_keep_stable_child_ids() -> TestResult {
    for (input, partial, two, all) in [
        (
            "base",
            "created_3_black",
            "created_3_2_black",
            "created_all_black",
        ),
        (
            "dense_base",
            "dense_created_3",
            "dense_created_3_2",
            "dense_created_all",
        ),
    ] {
        let mut e = original(input)?;
        let baseline = serde_json::to_value(e.stored_document()?.model())?;
        let mut seed = 0x2805_7341_u32;
        // Fixed frozen endpoints; only history order is generated, never expected geometry/bytes.
        for index in 0..10 {
            create(&mut e, 3)?;
            exact(&e, &baseline, partial)?;
            let captured = e.stored_document()?.to_cdxml()?.into_owned();
            let mut reopened =
                Engine::from_document(StoredDocument::from_cdxml(captured.into(), PageSize::A4)?);
            create(&mut reopened, 2)?;
            exact(&reopened, &baseline, two)?;
            create(&mut reopened, 4)?;
            exact(&reopened, &baseline, all)?;
            create(&mut e, 2)?;
            exact(&e, &baseline, two)?;
            create(&mut e, 4)?;
            exact(&e, &baseline, all)?;
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let cycles = if index == 0 { 1 } else { 1 + seed % 3 };
            for _ in 0..cycles {
                for name in [two, partial, input] {
                    e.dispatch("history.undo", json!({}))?;
                    exact(&e, &baseline, name)?;
                }
                for name in [partial, two, all] {
                    e.dispatch("history.redo", json!({}))?;
                    exact(&e, &baseline, name)?;
                }
            }
            for _ in 0..3 {
                e.dispatch("history.undo", json!({}))?;
            }
            assert_eq!(e.state().undo_depth, 0);
        }
        let mut ordered = original(input)?;
        for id in [2, 3, 4] {
            create(&mut ordered, id)?;
        }
        exact(&ordered, &baseline, all)?;
    }
    for (input, output) in [
        ("paired_base", "paired_created_3"),
        ("caption_base", "caption_created_3"),
    ] {
        let mut e = original(input)?;
        let baseline = serde_json::to_value(e.stored_document()?.model())?;
        create(&mut e, 3)?;
        exact(&e, &baseline, output)?;
    }
    Ok(())
}

#[test]
fn explicit_styles_chemistry_and_point_edits_compose_against_immutable_source() -> TestResult {
    for (style, z, h, output) in [
        (f::black_style(), 6, 2, "created_3_black"),
        (f::blue_style(), 6, 2, "created_3_blue14"),
        (f::black_style(), 7, 1, "created_3_nh"),
    ] {
        let mut e = original("base")?;
        let baseline = serde_json::to_value(e.stored_document()?.model())?;
        e.dispatch("atom.create_authored_label", f::params(0, 3, z, h, style))?;
        exact(&e, &baseline, output)?;
        assert_eq!(e.state().undo_depth, 1);
    }
    let mut e = original("base")?;
    let baseline = serde_json::to_value(e.stored_document()?.model())?;
    create(&mut e, 3)?;
    e.dispatch("atom.set_authored_label", f::label_params(1, 3, 7, 1))?;
    exact(&e, &baseline, "created_3_old48_nh")?;
    e.dispatch("history.undo", json!({}))?;
    let redo = e.state().redo_depth;
    let owner = e.capture_stamp();
    let noop = e.dispatch(
        "atom.set_authored_label",
        f::label_params(e.state().revision, 3, 6, 2),
    )?;
    assert!(!noop.changed);
    assert_eq!(e.state().redo_depth, redo);
    assert_eq!(e.capture_stamp(), owner);
    for (ids, dx, dy, output) in [
        (vec![3, 4], 12.5, -7.25, "created_3_selected_p12_5_m7_25"),
        (vec![1, 2, 3, 4, 5], 24., -12., "created_3_all_p24_m12"),
    ] {
        e.dispatch("ui.selection",json!({"action":"set","expected_revision":e.state().revision,"page_id":1,"atom_ids":ids}))?;
        let p = f::motion(e.state().revision, e.selection().epoch, dx, dy);
        e.dispatch("selection.translate", p)?;
        exact(&e, &baseline, output)?;
        e.dispatch("history.undo", json!({}))?;
        exact(&e, &baseline, "created_3_black")?;
    }
    let mut endpoint = original("base")?;
    endpoint.dispatch("atom.set_authored_label", f::label_params(0, 1, 7, 2))?;
    create(&mut endpoint, 3)?;
    exact(&endpoint, &baseline, "endpoint_edited_created_3")?;
    Ok(())
}
fn style() -> Value {
    json!({"font_family":"Roboto","size_pt":12.,"face":0,"rgb":[0.,0.,0.]})
}
fn params(e: &Engine, atom: u32) -> Value {
    let mut p = mixed::label_params(e.state().revision, atom, 6, 2);
    p["style"] = style();
    p
}

fn refused(e: &mut Engine, name: &str, p: Value) -> TestResult {
    let state = serde_json::to_value(e.state())?;
    let snapshot = e.stored_document()?.model_snapshot();
    let content = e.content().clone();
    let bytes = e.stored_document()?.to_cdxml()?.into_owned();
    let owner = e.capture_stamp();
    let file = e.ui_file_context_stamp();
    assert!(
        e.dispatch(name, p).is_err(),
        "{name} unexpectedly succeeded"
    );
    assert_eq!(serde_json::to_value(e.state())?, state);
    assert_eq!(e.content(), &content);
    assert!(Arc::ptr_eq(
        &snapshot,
        &e.stored_document()?.model_snapshot()
    ));
    assert_eq!(e.stored_document()?.to_cdxml()?.as_ref(), bytes);
    assert_eq!(e.capture_stamp(), owner);
    assert!(e.ui_file_context_stamp_matches(&file));
    Ok(())
}

#[test]
fn source_target_style_and_physical_page_refusals_preserve_every_owner() -> TestResult {
    let mut e = engine()?;
    let valid = params(&e, 3);
    for (key, value) in [
        ("expected_revision", json!(1)),
        ("page_id", json!(0)),
        ("page_id", json!(2)),
        ("fragment_id", json!(2)),
        ("atom_id", json!(1)),
        ("atom_id", json!(0)),
        ("atom_id", json!(99)),
        ("element", json!(0)),
        ("element", json!(119)),
        ("explicit_hydrogens", json!(65536)),
    ] {
        let mut p = valid.clone();
        p[key] = value;
        refused(&mut e, "atom.create_authored_label", p)?;
    }
    for (key, value) in [
        ("font_family", json!("Other")),
        ("font_family", json!("")),
        ("face", json!(1)),
        ("size_pt", json!(0)),
        ("size_pt", json!(-1)),
        ("size_pt", json!(100001)),
        ("size_pt", json!(100000)),
        ("rgb", json!([0., 0., 1.])),
        ("rgb", json!([-0.1, 0., 0.])),
        ("rgb", json!([0., 1.1, 0.])),
    ] {
        let mut p = valid.clone();
        p["style"][key] = value;
        refused(&mut e, "atom.create_authored_label", p)?;
    }
    refused(
        &mut e,
        "atom.set_authored_label",
        mixed::label_params(0, 3, 6, 2),
    )?;
    refused(&mut e, "atom.set_label", mixed::label_params(0, 3, 6, 2))?;
    let opaque = mixed::source("amino_alcohol").replace("<CDXML ", "<CDXML Future='retain' ");
    let mut foreign = Engine::from_document(StoredDocument::from_cdxml(
        opaque.as_bytes().into(),
        PageSize::A4,
    )?);
    refused(&mut foreign, "atom.create_authored_label", valid)?;
    Ok(())
}

#[test]
fn native_data_codec_cannot_bypass_current_resource_or_hidden_atom_authoring_proofs() -> TestResult
{
    let raw = bondline_io::cdxml::write_document(
        original("created_3_black")?.stored_document()?.model(),
    )?;
    let mut e = Engine::from_document(StoredDocument::from_cdxml(raw.into(), PageSize::A4)?);
    assert!(e.stored_document()?.has_flagged_atom_labels());
    assert!(!e.stored_document()?.is_fully_mapped());
    let mut p = params(&e, 2);
    p["style"]["rgb"] = json!([0.1, 0.2, 0.3]);
    refused(&mut e, "atom.create_authored_label", p)?;
    e.dispatch(
        "ui.selection",
        json!({"action":"set","expected_revision":0,"page_id":1,"atom_ids":[2]}),
    )?;
    let page = &e.stored_document()?.model().pages()[0];
    let fragment = &page.fragments()[0];
    let mut atoms = fragment.atoms().to_vec();
    let target = atoms
        .iter_mut()
        .find(|a| a.id.get() == 2)
        .ok_or("hidden target")?;
    target.position = Some(bondline_model::Point2::new(168., 160.)?);
    let mut codec = e.stored_document()?.clone();
    codec.model_mut().replace_fragment(
        page.id(),
        bondline_model::Fragment::new(fragment.id(), atoms, fragment.bonds().to_vec())?,
    )?;
    assert!(!codec.has_display_projection());
    assert!(
        codec.to_cdxml().is_ok(),
        "old exact-native data codec is distinct from authoring"
    );
    let movement = f::motion(e.state().revision, e.selection().epoch, 0., -48.);
    refused(&mut e, "selection.translate", movement)?;
    Ok(())
}

#[test]
fn explicit_creation_is_one_edit_even_when_numeric_element_and_h_are_unchanged() -> TestResult {
    let mut e = engine()?;
    let before = e.content().clone();
    let snapshot = e.stored_document()?.model_snapshot();
    let original = e.stored_document()?.to_cdxml()?.into_owned();
    let p = params(&e, 3);
    assert!(e.dispatch("atom.create_authored_label", p)?.changed);
    assert_eq!(e.state().undo_depth, 1);
    assert_eq!(e.state().redo_depth, 0);
    let after = e.content().clone();
    let atoms = e.stored_document()?.model().pages()[0].fragments()[0].atoms();
    assert_eq!(atoms.len(), 5);
    for (a, b) in snapshot.pages()[0].fragments()[0].atoms().iter().zip(atoms) {
        assert_eq!(a.id, b.id);
        assert_eq!(a.position, b.position);
        assert_eq!(a.element, b.element);
        assert_eq!(a.explicit_hydrogens, b.explicit_hydrogens);
        assert_eq!(a.show_terminal_carbon_labels, b.show_terminal_carbon_labels);
        assert_eq!(
            a.show_non_terminal_carbon_labels,
            b.show_non_terminal_carbon_labels
        );
        if b.id.get() == 3 {
            let label = b.authored_label.as_ref().ok_or("created label")?;
            assert_eq!(label.runs()[0].text(), "CH2");
            assert_eq!(serde_json::to_value(label.runs()[0].style())?, style());
        } else {
            assert_eq!(a.authored_label, b.authored_label);
        }
    }
    let created = e.stored_document()?.to_cdxml()?.into_owned();
    let state = serde_json::to_value(e.state())?;
    let share = e.stored_document()?.model_snapshot();
    let duplicate = params(&e, 3);
    assert!(e.dispatch("atom.create_authored_label", duplicate).is_err());
    assert_eq!(serde_json::to_value(e.state())?, state);
    assert!(Arc::ptr_eq(&share, &e.stored_document()?.model_snapshot()));
    e.dispatch("history.undo", json!({}))?;
    assert_eq!(e.content(), &before);
    assert_eq!(e.stored_document()?.to_cdxml()?.as_ref(), original);
    e.dispatch("history.redo", json!({}))?;
    assert_eq!(e.content(), &after);
    assert_eq!(e.stored_document()?.to_cdxml()?.as_ref(), created);
    Ok(())
}

#[test]
fn hidden_target_form_declares_its_original_style_and_cancel_never_creates_data() -> TestResult {
    let mut e = engine()?;
    e.set_ui_file_host(true);
    let before = e.content().clone();
    let p = json!({"action":"open","expected_revision":e.state().revision,"page_id":1,"fragment_id":1,"atom_id":3});
    assert!(!e.dispatch("ui.atom_label", p)?.changed);
    let request = e.ui_atom_label_request().ok_or("creation form")?;
    assert_eq!(serde_json::to_value(&request)?["creation_style"], style());
    assert_eq!(e.content(), &before);
    assert!(!e.dispatch("ui.atom_label",json!({"action":"cancel","expected_revision":e.state().revision,"request_id":request.id}))?.changed);
    assert_eq!(e.content(), &before);
    assert_eq!(e.state().undo_depth, 0);
    assert!(!e.ui_atom_label_request_current(request.id));
    Ok(())
}

#[test]
fn captured_old_save_and_stale_creation_forms_never_author_or_acknowledge_newer_labels()
-> TestResult {
    let directory = files::FileTestDirectory::new()?;
    let input = directory.path("created-original-原-λ.cdxml");
    let output = directory.path("created-older-save-保存.cdxml");
    std::fs::write(&input, f::source("base"))?;
    let mut e = Engine::default();
    e.dispatch("document.open_cdxml", json!({"path":input}))?;
    e.set_ui_file_host(true);
    let baseline = serde_json::to_value(e.stored_document()?.model())?;
    let pending = e.prepare(bondline_commands::Command::parse(
        "document.save_cdxml",
        json!({"path":output}),
    )?)?;
    create(&mut e, 3)?;
    let snapshot = e.stored_document()?.model_snapshot();
    let saved = e.finish(pending.complete()?)?;
    assert!(!saved.changed && saved.file.dirty && saved.source.ok_or("source")?.modified);
    assert!(Arc::ptr_eq(
        &snapshot,
        &e.stored_document()?.model_snapshot()
    ));
    assert_eq!(std::fs::read(&output)?, f::source("base").as_bytes());
    assert_eq!(std::fs::read(&input)?, f::source("base").as_bytes());
    exact(&e, &baseline, "created_3_black")?;
    e.dispatch(
        "ui.selection",
        json!({"action":"activate","expected_revision":e.state().revision}),
    )?;
    let context = e.selection_context()?;
    let open = json!({"action":"open","expected_revision":e.state().revision,"page_id":1,"fragment_id":1,"atom_id":2});
    e.dispatch("ui.atom_label", open.clone())?;
    assert!(!context.is_current(&e));
    let request = e.ui_atom_label_request().ok_or("creation request")?;
    assert_eq!(
        serde_json::to_value(&request)?["creation_style"],
        f::black_style()
    );
    e.dispatch(
        "ui.atom_label",
        json!({"action":"cancel","expected_revision":e.state().revision,"request_id":request.id}),
    )?;
    assert!(!context.is_current(&e));
    assert!(e.apply_ui_atom_label(request.id, 7, 1).is_err());
    let fresh = e.selection_context()?;
    assert!(fresh.is_current(&e));
    e.dispatch("ui.atom_label", open)?;
    let request = e.ui_atom_label_request().ok_or("second creation request")?;
    e.dispatch("ui.view", json!({"action":"set_zoom","percent":125}))?;
    let before = serde_json::to_value(e.state())?;
    assert!(e.apply_ui_atom_label(request.id, 7, 1).is_err());
    assert_eq!(serde_json::to_value(e.state())?, before);
    e.dispatch(
        "ui.atom_label",
        json!({"action":"cancel","expected_revision":e.state().revision,"request_id":request.id}),
    )?;
    exact(&e, &baseline, "created_3_black")?;
    Ok(())
}
