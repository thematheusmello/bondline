//! Original native XML and existing JSON Engine API, independent of proposed Rust fields.
use bondline_commands::Engine;
use serde_json::{Value, json};
use std::fs;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse the existing isolated test directory only")]
mod support;

const ALL_HIDDEN:&[u8]=br#"<?xml version="1.0" encoding="UTF-8"?><CDXML CreationProgram="Bondline 0.1.0"><page id="1" DrawingSpace="Poster" Width="420" Height="280"><fragment id="2"><n id="3" NodeType="Element" Element="6" Charge="0" Radical="None" p="144 144" NumHydrogens="3" ShowTerminalCarbonLabels="no" ShowNonTerminalCarbonLabels="no"/><n id="4" NodeType="Element" Element="6" Charge="0" Radical="None" p="208 144" NumHydrogens="2" ShowTerminalCarbonLabels="no" ShowNonTerminalCarbonLabels="no"/><n id="5" NodeType="Element" Element="6" Charge="0" Radical="None" p="240 96" NumHydrogens="3" ShowTerminalCarbonLabels="no" ShowNonTerminalCarbonLabels="no"/><b id="6" B="3" E="4" Order="1"/><b id="7" B="4" E="5" Order="1"/></fragment></page></CDXML>"#;

#[test]
fn existing_same_count_label_restores_both_false_data_on_a_terminal_atom_as_one_edit() -> TestResult
{
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("terminal-hidden-炭-λ.cdxml");
    fs::write(&input, ALL_HIDDEN)?;
    let mut engine = Engine::default();
    let opened = engine.dispatch("document.open_cdxml", json!({"path":input}))?;
    assert_eq!(opened.revision, 1);
    assert_eq!(opened.undo_depth, 0);
    assert!(!opened.file.dirty);
    assert!(engine.stored_document()?.is_fully_mapped());
    assert_eq!(engine.stored_document()?.export_cdxml()?, ALL_HIDDEN);
    let original = engine.content().clone();
    let before = serde_json::to_value(engine.document()?)?;
    let mut expected = before.clone();
    let atoms = expected["pages"][0]["fragments"][0]["atoms"]
        .as_array_mut()
        .ok_or("atoms")?;
    assert_eq!(
        atoms
            .iter()
            .map(|a| a["explicit_hydrogens"].clone())
            .collect::<Vec<_>>(),
        vec![json!(3), json!(2), json!(3)]
    );
    for a in atoms.iter() {
        assert_eq!(a["show_terminal_carbon_labels"], false);
        assert_eq!(a["show_non_terminal_carbon_labels"], false);
    }
    atoms[0]["show_terminal_carbon_labels"] = json!(true);
    atoms[0]["show_non_terminal_carbon_labels"] = json!(true);
    let result = engine.dispatch(
        "atom.set_label",
        json!({"expected_revision":1,"page_id":1,
        "fragment_id":1,"atom_id":1,"element":6,"explicit_hydrogens":3}),
    )?;
    assert!(result.changed);
    assert_eq!(
        (result.revision, result.undo_depth, result.redo_depth),
        (2, 1, 0)
    );
    assert!(result.file.dirty && result.source.ok_or("source")?.modified);
    assert_eq!(serde_json::to_value(engine.document()?)?, expected);
    assert_ne!(expected, before);
    assert_eq!(fs::read(&input)?, ALL_HIDDEN);
    engine.dispatch("history.undo", json!({}))?;
    assert_eq!(engine.content(), &original);
    assert!(!engine.state().file.dirty);
    Ok(())
}

#[path = "../../../xtask/oracles/terminal_label_test_support.rs"]
#[allow(
    dead_code,
    reason = "share only original fixed expectations and unchanged chemical consumer"
)]
mod literal;
use bondline_io::StoredDocument;
use bondline_model::PageSize;
use std::sync::Arc;

#[test]
fn each_false_flag_restores_independently_of_degree_and_none_true_noops_preserve_redo_source_and_owner()
-> TestResult {
    assert_eq!(ALL_HIDDEN, literal::HIDDEN);
    assert_eq!(bondline_commands::registry().len(), 47);
    for target in [0_usize, 1] {
        for t in [None, Some(false), Some(true)] {
            for n in [None, Some(false), Some(true)] {
                let mut flags = literal::ALL_FALSE;
                flags[target] = (t, n);
                let raw = literal::source(&flags)?;
                let stored = StoredDocument::from_cdxml(raw.clone().into(), PageSize::A4)?;
                assert!(stored.is_fully_mapped());
                let mut engine = Engine::from_document(stored);
                // Existing unrelated edit/Undo establishes a real redo owner for every no-op cell.
                engine.dispatch(
                    "atom.set_label",
                    json!({"expected_revision":0,"page_id":1,"fragment_id":1,
            "atom_id":3,"element":8,"explicit_hydrogens":1}),
                )?;
                engine.dispatch("history.undo", json!({}))?;
                let before: Value = serde_json::to_value(engine.state())?;
                let content = engine.content().clone();
                let owner = engine.stored_document()?.model_snapshot();
                let stamp = engine.capture_stamp();
                let h = if target == 0 { 3 } else { 2 };
                let params = json!({"expected_revision":engine.state().revision,"page_id":1,
            "fragment_id":1,"atom_id":target+1,"element":6,"explicit_hydrogens":h});
                let mut extra = params.clone();
                extra["show_terminal_carbon_labels"] = json!(false);
                assert_eq!(
                    engine
                        .dispatch("atom.set_label", extra)
                        .err()
                        .ok_or("new field became command parameter")?
                        .info()
                        .code,
                    "invalid_params"
                );
                assert_eq!(serde_json::to_value(engine.state())?, before);
                let result = engine.dispatch("atom.set_label", params)?;
                let changed = t == Some(false) || n == Some(false);
                assert_eq!(result.changed, changed);
                let mut expected = flags;
                if t == Some(false) {
                    expected[target].0 = Some(true);
                }
                if n == Some(false) {
                    expected[target].1 = Some(true);
                }
                assert_eq!(
                    serde_json::to_value(engine.document()?)?["pages"][0]["fragments"],
                    json!([literal::fragment(&expected)])
                );
                if changed {
                    assert_eq!(
                        (result.revision, result.undo_depth, result.redo_depth),
                        (3, 1, 0)
                    );
                    assert!(result.file.dirty && result.source.ok_or("source")?.modified);
                    let restored = engine.content().clone();
                    engine.dispatch("history.undo", json!({}))?;
                    assert_eq!(engine.content(), &content);
                    assert_eq!(engine.stored_document()?.export_cdxml()?, raw);
                    engine.dispatch("history.redo", json!({}))?;
                    assert_eq!(engine.content(), &restored);
                } else {
                    assert_eq!(serde_json::to_value(engine.state())?, before);
                    assert_eq!(engine.content(), &content);
                    assert!(Arc::ptr_eq(
                        &owner,
                        &engine.stored_document()?.model_snapshot()
                    ));
                    assert!(engine.capture_stamp_matches(&stamp));
                    assert_eq!(result.redo_depth, 1);
                    assert_eq!(engine.stored_document()?.export_cdxml()?, raw);
                }
            }
        }
    }
    let directory = support::FileTestDirectory::new()?;
    let output = directory.path("terminal-both-restored-化学-λ.cdxml");
    let mut engine =
        Engine::from_document(StoredDocument::from_cdxml(ALL_HIDDEN.into(), PageSize::A4)?);
    engine.dispatch("atom.set_label",json!({"expected_revision":0,"page_id":1,"fragment_id":1,"atom_id":1,"element":6,"explicit_hydrogens":3}))?;
    engine.dispatch("document.save_cdxml", json!({"path":output}))?;
    let mut restored = literal::ALL_FALSE;
    restored[0] = (Some(true), Some(true));
    literal::consume(&output, &restored)?;
    engine.dispatch("document.open_cdxml", json!({"path":output}))?;
    assert_eq!(
        serde_json::to_value(engine.document()?)?["pages"][0]["fragments"],
        json!([literal::fragment(&restored)])
    );
    assert!(!engine.state().file.dirty);
    assert_eq!(engine.state().undo_depth, 0);
    Ok(())
}
