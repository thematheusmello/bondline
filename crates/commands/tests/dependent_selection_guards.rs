//! Independent current-API source/history/physical acceptance, declared before producers.
use bondline_commands::{Command, Engine, SelectionTranslate};
use bondline_io::StoredDocument;
use bondline_model::{Fragment, PageSize, Point2};
use serde_json::{Value, json};
use std::{borrow::Cow, sync::Arc};
#[path = "../../../xtask/oracles/dependent_label_support.rs"]
#[allow(dead_code, reason = "shared original source/file/App vectors")]
mod f;
type TestResult = f::Result;
fn engine(name: &str) -> f::Result<Engine> {
    Ok(Engine::from_document(StoredDocument::from_cdxml(
        f::source(name).as_bytes().into(),
        PageSize::A4,
    )?))
}
fn select(e: &mut Engine, ids: &[u32]) -> TestResult {
    let old = e.content().clone();
    let stamp = e.capture_stamp();
    let result = e.dispatch(
        "ui.selection",
        json!({"action":"set","expected_revision":e.state().revision,"page_id":1,"atom_ids":ids}),
    )?;
    assert!(!result.changed);
    assert_eq!(e.content(), &old);
    assert_eq!(e.capture_stamp(), stamp);
    Ok(())
}
fn movement(e: &Engine, x: f64, y: f64) -> Value {
    f::params(e.state().revision, e.selection().epoch, x, y)
}
fn execute(e: &mut Engine, x: f64, y: f64) -> TestResult {
    let p = movement(e, x, y);
    assert!(e.dispatch("selection.translate", p)?.changed);
    Ok(())
}
fn same(
    e: &Engine,
    state: &Value,
    content: &bondline_commands::StoredContent,
    model: &Arc<bondline_model::Document>,
) -> TestResult {
    assert_eq!(serde_json::to_value(e.state())?, *state);
    assert_eq!(e.content(), content);
    assert!(Arc::ptr_eq(model, &e.stored_document()?.model_snapshot()));
    Ok(())
}
fn failed(e: &mut Engine, name: &str, p: Value, code: Option<&str>) -> TestResult {
    let state = serde_json::to_value(e.state())?;
    let content = e.content().clone();
    let model = e.stored_document()?.model_snapshot();
    let stamp = e.capture_stamp();
    let file = e.ui_file_context_stamp();
    let view = e.view();
    let tool = e.bond_tool();
    let generation = e.session_generation();
    let error = e
        .dispatch(name, p)
        .err()
        .ok_or_else(|| format!("{name} unexpectedly succeeded"))?;
    if let Some(code) = code {
        assert_eq!(error.info().code, code, "{error}");
    }
    same(e, &state, &content, &model)?;
    assert_eq!(e.capture_stamp(), stamp);
    assert!(e.ui_file_context_stamp_matches(&file));
    assert_eq!(e.view(), view);
    assert_eq!(e.bond_tool(), tool);
    assert_eq!(e.session_generation(), generation);
    Ok(())
}
fn literal_candidate(
    base: &StoredDocument,
    ids: &[u32],
    dx: f64,
    dy: f64,
) -> f::Result<StoredDocument> {
    let page = base.model().pages().first().ok_or("page")?;
    let mut replacement = Vec::new();
    for fragment in page.fragments() {
        let mut atoms = fragment.atoms().to_vec();
        for atom in &mut atoms {
            if ids.contains(&atom.id.get()) {
                let p = atom.position.ok_or("point")?;
                atom.position = Some(Point2::new(p.x_pt() + dx, p.y_pt() + dy)?);
                let label = atom.authored_label.as_ref().ok_or("label")?;
                let mut box_ = label.bounding_box().ok_or("box")?;
                for (i, v) in box_.iter_mut().enumerate() {
                    *v += if i % 2 == 0 { dx } else { dy };
                }
                atom.authored_label = Some(Arc::new(label.with_bounding_box(Some(box_))?));
            }
        }
        replacement.push(Fragment::new(
            fragment.id(),
            atoms,
            fragment.bonds().to_vec(),
        )?);
    }
    let mut result = base.clone();
    result
        .model_mut()
        .replace_fragments(page.id(), replacement)?;
    Ok(result)
}
#[test]
fn subset_all_and_tolerated_current_boxes_move_with_exact_source_and_history() -> TestResult {
    let metadata: Vec<Value> = serde_json::from_str(f::METADATA48)?;
    assert_eq!(
        serde_json::to_value(bondline_commands::registry())?,
        json!(metadata)
    );
    for (base, after, ids, dx, dy) in [
        ("boxed", "boxed_sub_p12_5_m7_25", vec![1], 12.5, -7.25),
        (
            "boxed_tolerated_raw",
            "boxed_tolerated_sub_p12_5_m7_25",
            vec![1],
            12.5,
            -7.25,
        ),
        (
            "boxed_pair",
            "boxed_pair_all_p24_m12",
            vec![1, 2],
            24.,
            -12.,
        ),
        (
            "connected",
            "connected_sub_p12_5_m7_25",
            vec![1],
            12.5,
            -7.25,
        ),
        (
            "connected",
            "connected_all_p24_m12",
            vec![1, 2, 3],
            24.,
            -12.,
        ),
    ] {
        let mut e = engine(base)?;
        let original = e.content().clone();
        let doc = serde_json::to_value(e.state().document.clone())?;
        let labels = e.stored_document()?.model().pages()[0]
            .fragments()
            .iter()
            .flat_map(|fragment| fragment.atoms())
            .map(|atom| {
                Ok((
                    atom.id.get(),
                    Arc::clone(atom.authored_label.as_ref().ok_or("label")?),
                ))
            })
            .collect::<f::Result<Vec<_>>>()?;
        select(&mut e, &ids)?;
        let old_epoch = e.selection().epoch;
        let context = e.selection_context()?;
        let p = SelectionTranslate {
            expected_revision: 0,
            expected_selection_epoch: old_epoch,
            delta_x_pt: dx,
            delta_y_pt: dy,
        };
        let candidate = e.preview_selection_translation(&p)?;
        assert_eq!(e.content(), &original);
        assert!(context.is_current(&e));
        assert_eq!(
            serde_json::to_value(candidate.model())?,
            f::model_after(&doc, &ids, dx, dy)?
        );
        assert_eq!(candidate.to_cdxml()?.as_ref(), f::source(after).as_bytes());
        for (id, old) in &labels {
            let atom = candidate.model().pages()[0]
                .fragments()
                .iter()
                .flat_map(|fragment| fragment.atoms())
                .find(|atom| atom.id.get() == *id)
                .ok_or("candidate atom")?;
            assert_eq!(
                Arc::ptr_eq(old, atom.authored_label.as_ref().ok_or("candidate label")?),
                !ids.contains(id),
                "only actually moved labels gain owned data"
            );
        }
        execute(&mut e, dx, dy)?;
        assert_eq!(
            (
                e.state().revision,
                e.state().undo_depth,
                e.state().redo_depth
            ),
            (1, 1, 0)
        );
        assert_eq!(e.selection().atom_ids, ids);
        assert_eq!(e.selection().epoch, old_epoch + 1);
        assert!(!context.is_current(&e));
        assert!(!e.stored_document()?.is_fully_mapped());
        assert!(e.stored_document()?.has_display_projection());
        assert_eq!(
            e.stored_document()?.to_cdxml()?.as_ref(),
            f::source(after).as_bytes()
        );
        assert_eq!(
            f::read(e.stored_document()?.to_cdxml()?.as_ref())?,
            f::facts(after)
        );
        let edited = e.content().clone();
        e.dispatch("history.undo", json!({}))?;
        assert_eq!(e.content(), &original);
        assert_eq!(
            e.stored_document()?.to_cdxml()?.as_ref(),
            f::source(base).as_bytes()
        );
        e.dispatch("history.redo", json!({}))?;
        assert_eq!(e.content(), &edited);
        assert_eq!(
            e.stored_document()?.to_cdxml()?.as_ref(),
            f::source(after).as_bytes()
        );
    }
    Ok(())
}
#[test]
fn compound_label_edit_then_translation_composes_five_owned_fields_and_preserves_every_other_byte()
-> TestResult {
    let mut e = engine("connected")?;
    let original = e.content().clone();
    e.dispatch("atom.set_authored_label",json!({"expected_revision":0,"page_id":1,"fragment_id":1,"atom_id":1,"element":7,"explicit_hydrogens":1}))?;
    assert_eq!(
        e.stored_document()?.to_cdxml()?.as_ref(),
        f::source("connected_nh").as_bytes()
    );
    select(&mut e, &[1])?;
    execute(&mut e, 12.5, -7.25)?;
    assert_eq!(
        e.stored_document()?.to_cdxml()?.as_ref(),
        f::source("connected_nh_sub_p12_5_m7_25").as_bytes()
    );
    assert_eq!(
        f::read(e.stored_document()?.to_cdxml()?.as_ref())?,
        f::facts("connected_nh_sub_p12_5_m7_25")
    );
    e.dispatch("history.undo", json!({}))?;
    assert_eq!(
        e.stored_document()?.to_cdxml()?.as_ref(),
        f::source("connected_nh").as_bytes()
    );
    e.dispatch("history.undo", json!({}))?;
    assert_eq!(e.content(), &original);
    e.dispatch("history.redo", json!({}))?;
    e.dispatch("history.redo", json!({}))?;
    assert_eq!(
        e.stored_document()?.to_cdxml()?.as_ref(),
        f::source("connected_nh_sub_p12_5_m7_25").as_bytes()
    );
    Ok(())
}
#[test]
fn rounded_point_identity_and_zero_noops_keep_label_arcs_redo_and_weak_contexts() -> TestResult {
    let mut e = engine("boxed_rounding_boundary")?;
    select(&mut e, &[1])?;
    execute(&mut e, 24., -12.)?;
    e.dispatch("history.undo", json!({}))?;
    select(&mut e, &[1])?;
    let state = serde_json::to_value(e.state())?;
    let content = e.content().clone();
    let model = e.stored_document()?.model_snapshot();
    let stamp = e.capture_stamp();
    let context = e.selection_context()?;
    assert_eq!(e.state().redo_depth, 1);
    let atom = &model.pages()[0].fragments()[0].atoms()[0];
    let label = atom.authored_label.as_ref().ok_or("label")?;
    assert_eq!(128.0 + 1e-14, 128.0);
    let left = label.bounding_box().ok_or("box")?[0];
    assert_ne!(left + 1e-14, left);
    for (dx, dy) in [(0., 0.), (1e-14, 0.)] {
        let p = movement(&e, dx, dy);
        assert!(!e.dispatch("selection.translate", p)?.changed);
        same(&e, &state, &content, &model)?;
        assert_eq!(e.capture_stamp(), stamp);
        assert!(context.is_current(&e));
        assert!(matches!(e.stored_document()?.to_cdxml()?, Cow::Borrowed(_)));
        assert_eq!(
            e.stored_document()?.to_cdxml()?.as_ref(),
            f::source("boxed_rounding_boundary").as_bytes()
        );
    }
    e.dispatch("ui.view", json!({"action":"set_zoom","percent":200}))?;
    e.dispatch("ui.view", json!({"action":"set_zoom","percent":100}))?;
    assert!(!context.is_current(&e));
    assert!(e.selection_context()?.is_current(&e));
    Ok(())
}
#[test]
fn changed_label_preview_certifies_physical_page_before_returning_and_crossing_geometry_cannot_keep_authority()
-> TestResult {
    for (base, dx, dy) in [
        ("boxed_pair", 115.8779296875, 104.265625),
        ("boxed", -200., 0.),
    ] {
        let mut e = engine(base)?;
        select(&mut e, &[1])?;
        let literal = literal_candidate(e.stored_document()?, &[1], dx, dy)?;
        assert!(
            literal.has_boxed_atom_labels(),
            "complete source is deliberately distinct from physical page admission"
        );
        assert!(
            literal.to_cdxml().is_ok(),
            "this negative must reach the final page certificate"
        );
        let state = serde_json::to_value(e.state())?;
        let content = e.content().clone();
        let model = e.stored_document()?.model_snapshot();
        let p = SelectionTranslate {
            expected_revision: e.state().revision,
            expected_selection_epoch: e.selection().epoch,
            delta_x_pt: dx,
            delta_y_pt: dy,
        };
        assert!(
            e.preview_selection_translation(&p).is_err(),
            "unrenderable preview must not return a modelArc"
        );
        same(&e, &state, &content, &model)?;
        let request = movement(&e, dx, dy);
        failed(&mut e, "selection.translate", request, None)?;
    }
    let mut e = engine("connected_crossing_base")?;
    select(&mut e, &[3])?;
    let cross = literal_candidate(e.stored_document()?, &[3], 0., 60.)?;
    assert!(
        !cross.has_connected_atom_labels(),
        "CURRENT nonincident crossing must revoke the capability"
    );
    assert!(cross.to_cdxml().is_err());
    let request = movement(&e, 0., 60.);
    failed(&mut e, "selection.translate", request, None)?;
    let request = movement(&e, 1e308, 0.);
    failed(&mut e, "selection.translate", request, None)?;
    Ok(())
}
#[test]
fn new_selection_authority_never_leaks_to_ring_chain_or_other_writers_and_opaque_noops_stay_refused()
-> TestResult {
    for base in ["boxed", "connected"] {
        let mut e = engine(base)?;
        select(&mut e, &[1])?;
        for c in e.controls() {
            if c.id.starts_with("tool.ring.") || c.id.starts_with("chain.") {
                assert!(!c.enabled, "{} leaked authoring", c.id);
            }
        }
        let h = if base == "boxed" { 1 } else { 0 };
        for (name, p) in [
            ("ui.ring_tool", json!({"size":6})),
            ("ui.ring_tool", json!({"kind":"cyclohexane_chair_1"})),
            ("ui.chain_tool", json!({"action":"select","kind":"acyclic"})),
            (
                "atom.set_label",
                json!({"expected_revision":0,"page_id":1,"fragment_id":1,"atom_id":1,"element":8,"explicit_hydrogens":h}),
            ),
            (
                "bond.create",
                json!({"page_id":1,"begin_x_pt":300,"begin_y_pt":300,"end_x_pt":348,"end_y_pt":300}),
            ),
            (
                "page.set_size",
                json!({"page_id":1,"width_pt":512,"height_pt":512}),
            ),
            (
                "document.set_drawing_settings",
                json!({"expected_revision":0,"bond_length_pt":null,"chain_angle_degrees":null}),
            ),
            (
                "text.set_run",
                json!({"expected_revision":0,"page_id":1,"text_id":1,"run_index":0,"text":"O"}),
            ),
        ] {
            failed(&mut e, name, p, Some("unsupported_content"))?;
        }
        let request = f::params(99, e.selection().epoch, 0., 0.);
        failed(
            &mut e,
            "selection.translate",
            request,
            Some("stale_revision"),
        )?;
        let request = f::params(0, 0, 0., 0.);
        failed(
            &mut e,
            "selection.translate",
            request,
            Some("stale_selection"),
        )?;
        for raw in [
            f::source(base).replace("<CDXML ", "<CDXML Future='retain' "),
            f::source(base)
                .replace("face=\"0\"", "face=\"96\"")
                .replace("face='0'", "face='96'"),
        ] {
            let mut opaque = Engine::from_document(StoredDocument::from_cdxml(
                raw.as_bytes().into(),
                PageSize::A4,
            )?);
            assert!(!opaque.selection_available());
            failed(
                &mut opaque,
                "ui.selection",
                json!({"action":"set","expected_revision":0,"page_id":1,"atom_ids":[1]}),
                Some("unsupported_content"),
            )?;
            failed(
                &mut opaque,
                "selection.translate",
                f::params(0, 0, 0., 0.),
                Some("unsupported_content"),
            )?;
            assert_eq!(
                opaque.stored_document()?.to_cdxml()?.as_ref(),
                raw.as_bytes()
            );
        }
    }
    Ok(())
}
#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "existing isolated actual file directory")]
mod files;
#[test]
fn captured_save_of_unmoved_owner_never_acknowledges_newer_point_and_box_snapshot() -> TestResult {
    let dir = files::FileTestDirectory::new()?;
    let input = dir.path("dependent-original-λ.cdxml");
    let output = dir.path("dependent-captured-保存.cdxml");
    std::fs::write(&input, f::source("connected"))?;
    let mut e = Engine::default();
    e.dispatch("document.open_cdxml", json!({"path":input}))?;
    let pending = e.prepare(Command::parse(
        "document.save_cdxml",
        json!({"path":output}),
    )?)?;
    select(&mut e, &[1])?;
    execute(&mut e, 12.5, -7.25)?;
    let moved = e.stored_document()?.model_snapshot();
    let completed = pending.complete()?;
    assert_eq!(std::fs::read(&output)?, f::source("connected").as_bytes());
    assert_eq!(f::read(&std::fs::read(&output)?)?, f::facts("connected"));
    let finished = e.finish(completed)?;
    assert!(!finished.changed && finished.file.dirty && finished.source.ok_or("source")?.modified);
    assert_eq!(finished.revision, 2);
    assert!(Arc::ptr_eq(&moved, &e.stored_document()?.model_snapshot()));
    assert_eq!(
        e.stored_document()?.to_cdxml()?.as_ref(),
        f::source("connected_sub_p12_5_m7_25").as_bytes()
    );
    execute(&mut e, -12.5, 7.25)?;
    assert!(!e.state().file.dirty);
    assert_eq!(
        e.stored_document()?.to_cdxml()?.as_ref(),
        f::source("connected").as_bytes()
    );
    assert_eq!(std::fs::read(input)?, f::source("connected").as_bytes());
    Ok(())
}
