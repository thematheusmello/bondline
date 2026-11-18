//! Three grouped independent current-API acceptance declarations, before Mixed callers.
use bondline_commands::{Command, Engine, SelectionTranslate};
use bondline_io::StoredDocument;
use bondline_model::{Fragment, PageSize, Point2};
use serde_json::{Value, json};
use std::{borrow::Cow, sync::Arc};
#[path = "../../../xtask/oracles/mixed_label_support.rs"]
#[allow(dead_code, reason = "shared actual-file/native mixed acceptance")]
mod f;
#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "existing actual-file isolated directory")]
mod files;
type TestResult = f::Result;
fn engine(name: &str) -> f::Result<Engine> {
    Ok(Engine::from_document(StoredDocument::from_cdxml(
        f::source(name).as_bytes().into(),
        PageSize::A4,
    )?))
}
fn encoded(e: &Engine) -> f::Result<Vec<u8>> {
    Ok(e.stored_document()?.to_cdxml()?.into_owned())
}
fn exact(e: &Engine, name: &str) -> TestResult {
    assert!(!e.stored_document()?.is_fully_mapped());
    assert!(!e.stored_document()?.has_boxed_atom_labels());
    assert!(!e.stored_document()?.has_connected_atom_labels());
    assert!(e.stored_document()?.has_display_projection());
    let raw = encoded(e)?;
    assert_eq!(raw, f::source(name).as_bytes());
    let facts = f::facts(name);
    assert_eq!(f::read(&raw)?, facts);
    let model = serde_json::to_value(e.stored_document()?.model())?;
    let atoms = model["pages"][0]["fragments"][0]["atoms"]
        .as_array()
        .ok_or("model atoms")?;
    let nodes = facts["nodes"].as_array().ok_or("independent nodes")?;
    assert_eq!(atoms.len(), nodes.len());
    for (atom, node) in atoms.iter().zip(nodes) {
        assert_eq!(atom["id"], node["model_id"]);
        assert_eq!(atom["element"], node["element"]);
        assert_eq!(atom["explicit_hydrogens"], node["hydrogens"]);
        assert_eq!(
            atom["position"],
            json!({"x_pt":node["point"][0],"y_pt":node["point"][1]})
        );
        assert_eq!(atom["charge"], node["charge"]);
        assert_eq!(atom["radical"], "none");
        assert_eq!(atom["isotope"], Value::Null);
        for (xml, field) in [
            ("ShowTerminalCarbonLabels", "show_terminal_carbon_labels"),
            (
                "ShowNonTerminalCarbonLabels",
                "show_non_terminal_carbon_labels",
            ),
        ] {
            assert_eq!(atom[field], node["flags"][xml]);
        }
        if node["label"].is_null() {
            assert!(atom.get("authored_label").is_none_or(Value::is_null));
        } else {
            let label = &atom["authored_label"];
            assert_eq!(label["bounding_box"], node["label"]["bounding_box"]);
            assert_eq!(label["runs"][0]["text"], node["label"]["text"]);
            assert_eq!(
                label["runs"][0]["style"],
                json!({"font_family":node["label"]["font"],"size_pt":node["label"]["size"],"face":node["label"]["face"],"rgb":node["label"]["rgb"]})
            );
        }
    }
    let bonds = model["pages"][0]["fragments"][0]["bonds"]
        .as_array()
        .ok_or("model bonds")?;
    let expected = facts["bonds"].as_array().ok_or("independent bonds")?;
    assert_eq!(bonds.len(), expected.len());
    for (bond, source) in bonds.iter().zip(expected) {
        assert_eq!(bond["id"], source["model_id"]);
        for (from, to) in [("begin", "begin"), ("end", "end")] {
            let key = source[from].as_u64().ok_or("source endpoint")?.to_string();
            assert_eq!(bond[to], facts["source_to_model_ids"][key]);
        }
        assert_eq!(bond["order"], "single");
    }
    Ok(())
}
fn select(e: &mut Engine, ids: &[u32]) -> TestResult {
    assert!(!e.dispatch("ui.selection",json!({"action":"set","expected_revision":e.state().revision,"page_id":1,"atom_ids":ids}))?.changed);
    Ok(())
}
fn move_params(e: &Engine, dx: f64, dy: f64) -> Value {
    f::motion(e.state().revision, e.selection().epoch, dx, dy)
}
fn move_atoms(e: &mut Engine, dx: f64, dy: f64) -> TestResult {
    let p = move_params(e, dx, dy);
    assert!(e.dispatch("selection.translate", p)?.changed);
    Ok(())
}
fn edit(e: &mut Engine, atom: u32, z: u8, h: u16) -> TestResult {
    assert!(e.dispatch("atom.set_authored_label",json!({"expected_revision":e.state().revision,"page_id":1,"fragment_id":1,"atom_id":atom,"element":z,"explicit_hydrogens":h}))?.changed);
    Ok(())
}
fn failed(e: &mut Engine, name: &str, p: Value) -> TestResult {
    let state = serde_json::to_value(e.state())?;
    let content = e.content().clone();
    let model = e.stored_document()?.model_snapshot();
    let raw = encoded(e)?;
    let stamp = e.capture_stamp();
    let file = e.ui_file_context_stamp();
    let view = e.view();
    let tool = e.bond_tool();
    let generation = e.session_generation();
    assert!(
        e.dispatch(name, p).is_err(),
        "{name} unexpectedly succeeded"
    );
    assert_eq!(serde_json::to_value(e.state())?, state);
    assert_eq!(e.content(), &content);
    assert!(Arc::ptr_eq(&model, &e.stored_document()?.model_snapshot()));
    assert_eq!(encoded(e)?, raw);
    assert_eq!(e.capture_stamp(), stamp);
    assert!(e.ui_file_context_stamp_matches(&file));
    assert_eq!(e.view(), view);
    assert_eq!(e.bond_tool(), tool);
    assert_eq!(e.session_generation(), generation);
    Ok(())
}
fn literal_move(base: &StoredDocument, ids: &[u32], dx: f64, dy: f64) -> f::Result<StoredDocument> {
    let page = base.model().pages().first().ok_or("page")?;
    let mut replacements = Vec::new();
    for fragment in page.fragments() {
        let mut atoms = fragment.atoms().to_vec();
        for atom in &mut atoms {
            if ids.contains(&atom.id.get()) {
                let p = atom.position.ok_or("point")?;
                let q = Point2::new(p.x_pt() + dx, p.y_pt() + dy)?;
                if p != q {
                    atom.position = Some(q);
                    if let Some(old) = &atom.authored_label {
                        let [l, t, r, b] = old.bounding_box().ok_or("box")?;
                        atom.authored_label = Some(Arc::new(old.with_bounding_box(Some([
                            l + dx,
                            t + dy,
                            r + dx,
                            b + dy,
                        ]))?));
                    }
                }
            }
        }
        replacements.push(Fragment::new(
            fragment.id(),
            atoms,
            fragment.bonds().to_vec(),
        )?);
    }
    let mut result = base.clone();
    result
        .model_mut()
        .replace_fragments(page.id(), replacements)?;
    Ok(result)
}

#[test]
fn mixed_frozen_transitions_keep_exact_source_history_sharing_and_generated_noop_redo() -> TestResult
{
    let old: Vec<Value> = serde_json::from_str(f::METADATA48)?;
    assert_eq!(
        serde_json::to_value(
            bondline_commands::registry()
                .into_iter()
                .take(48)
                .collect::<Vec<_>>()
        )?,
        json!(old)
    );
    let mut e = engine("amino_alcohol")?;
    exact(&e, "amino_alcohol")?;
    let original = e.content().clone();
    let mut seed = 0x07a3_c519_u32;
    // The first five cover every frozen transition; later choices change history order,
    // never generate geometry/expected source from the producer under test.
    for index in 0..21 {
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let choice = if index < 5 {
            index
        } else {
            (seed % 5) as usize
        };
        let (after, ids, delta, has_edit) = match choice {
            0 => ("amino_diamine", vec![], None, true),
            1 => (
                "hidden_middle_p12_5_m7_25",
                vec![3],
                Some((12.5, -7.25)),
                false,
            ),
            2 => ("all_p24_m12", vec![1, 2, 3, 4, 5], Some((24., -12.)), false),
            3 => (
                "subset_1_3_5_p12_5_m7_25",
                vec![1, 3, 5],
                Some((12.5, -7.25)),
                false,
            ),
            _ => (
                "edited_all_p24_m12",
                vec![1, 2, 3, 4, 5],
                Some((24., -12.)),
                true,
            ),
        };
        let mut steps = 0;
        if has_edit {
            edit(&mut e, 1, 7, 2)?;
            steps += 1;
            exact(&e, "amino_diamine")?;
        }
        if let Some((dx, dy)) = delta {
            select(&mut e, &ids)?;
            let before = e.content().clone();
            let model = e.stored_document()?.model_snapshot();
            let stamp = e.capture_stamp();
            let context = e.selection_context()?;
            let epoch = e.selection().epoch;
            let revision = e.state().revision;
            let preview = e.preview_selection_translation(&SelectionTranslate {
                expected_revision: revision,
                expected_selection_epoch: epoch,
                delta_x_pt: dx,
                delta_y_pt: dy,
            })?;
            assert_eq!(
                serde_json::to_value(preview.model())?,
                f::model_for_case(&serde_json::to_value(&model)?, after)?
            );
            assert_eq!(preview.to_cdxml()?.as_ref(), f::source(after).as_bytes());
            assert_eq!(e.content(), &before);
            assert_eq!(e.capture_stamp(), stamp);
            assert!(context.is_current(&e));
            for (old, new) in model.pages()[0].fragments()[0]
                .atoms()
                .iter()
                .zip(preview.model().pages()[0].fragments()[0].atoms())
            {
                assert_eq!(old.explicit_hydrogens, new.explicit_hydrogens);
                assert_eq!(
                    old.show_terminal_carbon_labels,
                    new.show_terminal_carbon_labels
                );
                assert_eq!(
                    old.show_non_terminal_carbon_labels,
                    new.show_non_terminal_carbon_labels
                );
                match (&old.authored_label, &new.authored_label) {
                    (Some(a), Some(b)) => {
                        assert_eq!(Arc::ptr_eq(a, b), !ids.contains(&old.id.get()))
                    }
                    (None, None) => {}
                    _ => return Err("movement changed child presence".into()),
                }
            }
            move_atoms(&mut e, dx, dy)?;
            steps += 1;
            assert!(!context.is_current(&e));
            assert_eq!(e.selection().atom_ids, ids);
        }
        exact(&e, after)?;
        let final_content = e.content().clone();
        assert_eq!(e.state().undo_depth, steps);
        for _ in 0..steps {
            e.dispatch("history.undo", json!({}))?;
        }
        assert_eq!(e.content(), &original);
        exact(&e, "amino_alcohol")?;
        assert_eq!(e.state().redo_depth, steps);
        select(&mut e, &[1, 2, 3, 4, 5])?;
        let state = serde_json::to_value(e.state())?;
        let model = e.stored_document()?.model_snapshot();
        let context = e.selection_context()?;
        let p = move_params(&e, 0., 0.);
        assert!(!e.dispatch("selection.translate", p)?.changed);
        assert_eq!(serde_json::to_value(e.state())?, state);
        assert!(Arc::ptr_eq(&model, &e.stored_document()?.model_snapshot()));
        assert!(context.is_current(&e));
        assert!(matches!(e.stored_document()?.to_cdxml()?, Cow::Borrowed(_)));
        for _ in 0..steps {
            e.dispatch("history.redo", json!({}))?;
        }
        assert_eq!(e.content(), &final_content);
        exact(&e, after)?;
        for _ in 0..steps {
            e.dispatch("history.undo", json!({}))?;
        }
        assert_eq!(e.content(), &original);
    }
    Ok(())
}

#[test]
fn current_c2_collinear_hidden_target_and_foreign_authority_guards_refuse_atomically() -> TestResult
{
    let mut methanol = engine("methanol")?;
    exact(&methanol, "methanol")?;
    failed(
        &mut methanol,
        "atom.set_authored_label",
        json!({"expected_revision":0,"page_id":1,"fragment_id":1,"atom_id":2,"element":6,"explicit_hydrogens":3}),
    )?;
    let mut bad = methanol.stored_document()?.clone();
    *bad.model_mut() = engine("methanol_bad_c2")?
        .stored_document()?
        .model()
        .clone();
    assert!(!bad.has_display_projection());
    assert!(bad.to_cdxml().is_err());
    let mut e = engine("amino_alcohol")?;
    e.set_ui_file_host(true);
    select(&mut e, &[3])?;
    failed(
        &mut e,
        "atom.set_authored_label",
        json!({"expected_revision":0,"page_id":1,"fragment_id":1,"atom_id":3,"element":6,"explicit_hydrogens":2}),
    )?;
    let before_form = serde_json::to_value(e.state())?;
    let original = e.content().clone();
    let model = e.stored_document()?.model_snapshot();
    assert!(
        !e.dispatch(
            "ui.atom_label",
            json!({"action":"open","expected_revision":0,"page_id":1,"fragment_id":1,"atom_id":3})
        )?
        .changed
    );
    let request = e
        .ui_atom_label_request()
        .ok_or("command49 creation draft")?;
    assert_eq!(
        serde_json::to_value(&request)?["creation_style"],
        json!({"font_family":"Roboto","size_pt":12.0,"face":0,"rgb":[0.0,0.0,0.0]})
    );
    assert_eq!(e.content(), &original);
    assert!(Arc::ptr_eq(&model, &e.stored_document()?.model_snapshot()));
    e.dispatch(
        "ui.atom_label",
        json!({"action":"cancel","expected_revision":0,"request_id":request.id}),
    )?;
    assert_eq!(serde_json::to_value(e.state())?, before_form);
    let bad = literal_move(e.stored_document()?, &[3], 0., 48.)?;
    assert_eq!(
        bad.model(),
        engine("mandatory_collinear")?.stored_document()?.model()
    );
    assert!(!bad.has_display_projection());
    assert!(bad.to_cdxml().is_err());
    let state = serde_json::to_value(e.state())?;
    let model = e.stored_document()?.model_snapshot();
    assert!(
        e.preview_selection_translation(&SelectionTranslate {
            expected_revision: 0,
            expected_selection_epoch: e.selection().epoch,
            delta_x_pt: 0.,
            delta_y_pt: 48.
        })
        .is_err()
    );
    assert_eq!(serde_json::to_value(e.state())?, state);
    assert!(Arc::ptr_eq(&model, &e.stored_document()?.model_snapshot()));
    let p = move_params(&e, 0., 48.);
    failed(&mut e, "selection.translate", p)?;
    select(&mut e, &[1, 2, 3, 4, 5])?;
    let offpaper = literal_move(e.stored_document()?, &[1, 2, 3, 4, 5], 600., 0.)?;
    assert!(offpaper.has_display_projection());
    assert!(
        offpaper.to_cdxml().is_ok(),
        "source-valid offpaper must reach the final page certificate"
    );
    assert!(
        e.preview_selection_translation(&SelectionTranslate {
            expected_revision: 0,
            expected_selection_epoch: e.selection().epoch,
            delta_x_pt: 600.,
            delta_y_pt: 0.
        })
        .is_err()
    );
    let p = move_params(&e, 600., 0.);
    failed(&mut e, "selection.translate", p)?;
    for (name, p) in [
        (
            "atom.set_label",
            json!({"expected_revision":0,"page_id":1,"fragment_id":1,"atom_id":3,"element":6,"explicit_hydrogens":2}),
        ),
        ("ui.ring_tool", json!({"size":6})),
        ("ui.chain_tool", json!({"action":"select","kind":"acyclic"})),
        (
            "bond.create",
            json!({"page_id":1,"begin_x_pt":320,"begin_y_pt":320,"end_x_pt":368,"end_y_pt":320}),
        ),
        (
            "page.set_size",
            json!({"page_id":1,"width_pt":512,"height_pt":512}),
        ),
        (
            "document.set_document_settings",
            json!({"expected_revision":0,"bond_length_pt":48,"chain_angle_degrees":null,"line_width_pt":1.25,"bond_spacing_percent":18}),
        ),
    ] {
        failed(&mut e, name, p)?;
    }
    let raw = f::source("amino_alcohol");
    for opaque in [
        raw.replace("<CDXML ", "<CDXML Future='retain' "),
        raw.replace(
            " ShowNonTerminalCarbonLabels='no'/>",
            " ShowNonTerminalCarbonLabels='no'><t future='retain'/></n>",
        ),
        raw.replace("Element='6' Charge='0'", "Element='6' Charge='1'"),
    ] {
        let mut e = Engine::from_document(StoredDocument::from_cdxml(
            opaque.as_bytes().into(),
            PageSize::A4,
        )?);
        assert!(!e.selection_available());
        failed(
            &mut e,
            "ui.selection",
            json!({"action":"all","expected_revision":0,"page_id":1}),
        )?;
        failed(
            &mut e,
            "atom.set_authored_label",
            json!({"expected_revision":0,"page_id":1,"fragment_id":1,"atom_id":1,"element":8,"explicit_hydrogens":1}),
        )?;
        assert_eq!(encoded(&e)?, opaque.as_bytes());
    }
    Ok(())
}

#[test]
fn mixed_captured_save_and_hidden_modal_weak_owners_never_acknowledge_newer_source() -> TestResult {
    let dir = files::FileTestDirectory::new()?;
    let input = dir.path("mixed-source-λ.cdxml");
    let output = dir.path("mixed-captured-保存.cdxml");
    let current = dir.path("mixed-current-移動.cdxml");
    std::fs::write(&input, f::source("amino_alcohol"))?;
    let mut e = Engine::default();
    e.dispatch("document.open_cdxml", json!({"path":input}))?;
    e.set_ui_file_host(true);
    let pending = e.prepare(Command::parse(
        "document.save_cdxml",
        json!({"path":output}),
    )?)?;
    select(&mut e, &[3])?;
    move_atoms(&mut e, 12.5, -7.25)?;
    exact(&e, "hidden_middle_p12_5_m7_25")?;
    let model = e.stored_document()?.model_snapshot();
    let finished = e.finish(pending.complete()?)?;
    assert!(!finished.changed && finished.file.dirty && finished.source.ok_or("source")?.modified);
    assert_eq!(finished.revision, 2);
    assert!(Arc::ptr_eq(&model, &e.stored_document()?.model_snapshot()));
    assert_eq!(
        std::fs::read(&output)?,
        f::source("amino_alcohol").as_bytes()
    );
    assert_eq!(
        f::read(&std::fs::read(&output)?)?,
        f::facts("amino_alcohol")
    );
    let proof = e.selection_context()?;
    let before = e.content().clone();
    let opened = e.dispatch(
        "ui.atom_label",
        json!({"action":"open","expected_revision":2,"page_id":1,"fragment_id":1,"atom_id":1}),
    )?;
    let id = opened.ui_atom_label_operation.ok_or("request ID")?;
    assert!(!proof.is_current(&e));
    e.dispatch(
        "ui.atom_label",
        json!({"action":"cancel","expected_revision":2,"request_id":id}),
    )?;
    assert!(!proof.is_current(&e));
    assert_eq!(e.content(), &before);
    assert!(e.selection_context()?.is_current(&e));
    assert!(Arc::ptr_eq(&model, &e.stored_document()?.model_snapshot()));
    e.dispatch("document.save_cdxml", json!({"path":current}))?;
    assert!(!e.state().file.dirty);
    assert!(e.state().source.ok_or("source")?.modified);
    assert_eq!(
        std::fs::read(&current)?,
        f::source("hidden_middle_p12_5_m7_25").as_bytes()
    );
    e.dispatch("document.open_cdxml", json!({"path":current}))?;
    exact(&e, "hidden_middle_p12_5_m7_25")?;
    assert!(!e.state().file.dirty);
    assert!(!e.state().source.ok_or("source")?.modified);
    assert_eq!(std::fs::read(input)?, f::source("amino_alcohol").as_bytes());
    Ok(())
}
