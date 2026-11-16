//! Independent complete-source SMILES drawing-import and notation-edit commands.

use bondline_commands::Engine;
use serde_json::json;
use std::fs;

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse the unchanged isolated test directory")]
mod support;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn import_of_already_open_smiles_records_creates_a_clean_source_backed_drawing() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("record-source-λ.smi");
    let source = b"CC\tKEEP\0\xff\r\n";
    fs::write(&input, source)?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_smiles", json!({"path":input}))?;
    let result = engine.dispatch(
        "document.import_smiles_records",
        json!({
            "expected_revision":1,"selection":{"mode":"all"},"bond_length_pt":64.0,
            "origin_x_pt":96.0,"origin_y_pt":96.0,"page_width_pt":612.0,"page_height_pt":792.0
        }),
    )?;
    let value = serde_json::to_value(&result)?;
    assert!(value["document"].is_object());
    assert_eq!(value["source"]["format"], "smiles");
    assert_eq!(value["source"]["modified"], false);
    assert_eq!(value["file"]["format"], "smiles");
    assert_eq!(value["file"]["dirty"], false);
    assert_eq!(result.revision, 2);
    assert_eq!((result.undo_depth, result.redo_depth), (0, 0));
    assert!(result.file_operation.is_none());
    assert_eq!(fs::read(input)?, source);
    Ok(())
}

use bondline_commands::{Cancellation, Command, StoredContent};
use bondline_model::{PageSize, Point2, Radical};
use serde_json::Value;
use std::{
    io::Write,
    path::Path,
    process::{Command as Process, Stdio},
};

const IMPORT: &str = "document.import_smiles_records";
const SOURCE: &[u8] = b"\r\nCC\tFIRST\0\xff\r\nC1.N.C1 \tKEEP\0\xff\nCO\tLAST";
const DOUBLE: &[u8] = b"\r\nCC\tFIRST\0\xff\r\n[CH2]=[CH2].[NH3] \tKEEP\0\xff\nCO\tLAST";
const SINGLE: &[u8] = b"\r\nCC\tFIRST\0\xff\r\n[CH3][CH3].[NH3] \tKEEP\0\xff\nCO\tLAST";

fn import_params(revision: u64, selection: Value) -> Value {
    json!({"expected_revision":revision,"selection":selection,"bond_length_pt":64.0,
        "origin_x_pt":96.0,"origin_y_pt":96.0,"page_width_pt":612.0,"page_height_pt":792.0})
}

fn order_params(revision: u64, fragment: u32, bond: u32, order: &str) -> Value {
    json!({"expected_revision":revision,"page_id":1,"fragment_id":fragment,"bond_id":bond,
        "order":order,"hydrogen_policy":"rectify_neutral_carbon"})
}

fn open(path: &Path, bytes: &[u8]) -> TestResult<Engine> {
    fs::write(path, bytes)?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_smiles", json!({"path":path}))?;
    Ok(engine)
}

fn state(engine: &Engine) -> TestResult<Value> {
    Ok(serde_json::to_value(engine.state())?)
}

fn graph(engine: &Engine) -> TestResult<Value> {
    let page = &engine.document()?.pages()[0];
    assert_eq!(page.size(), PageSize::new(612.0, 792.0)?);
    Ok(json!(
        page.fragments()
            .iter()
            .map(|fragment| {
                let atoms = fragment
                    .atoms()
                    .iter()
                    .map(|atom| {
                        assert!(atom.position.is_some());
                        assert!(atom.isotope.is_none());
                        assert_eq!(atom.charge, 0);
                        assert_eq!(atom.radical, Radical::None);
                        json!([atom.id.get(), atom.element.get(), atom.explicit_hydrogens])
                    })
                    .collect::<Vec<_>>();
                let bonds = fragment
                    .bonds()
                    .iter()
                    .map(|bond| {
                        json!([bond.id.get(), bond.begin.get(), bond.end.get(), bond.order])
                    })
                    .collect::<Vec<_>>();
                json!([fragment.id().get(), atoms, bonds])
            })
            .collect::<Vec<_>>()
    ))
}

fn positions(engine: &Engine) -> TestResult<Vec<(u32, Option<Point2>)>> {
    Ok(engine.document()?.pages()[0]
        .fragments()
        .iter()
        .flat_map(|fragment| {
            fragment
                .atoms()
                .iter()
                .map(|atom| (atom.id.get(), atom.position))
        })
        .collect())
}

fn consume(script: &str, bytes: &[u8]) -> TestResult<Value> {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child = Process::new(python)
        .arg("-B")
        .arg(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../io/tests")
                .join(script),
        )
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("consumer stdin")?
        .write_all(bytes)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(serde_json::from_slice(&output.stdout)?)
}

fn consume_saved(
    path: &Path,
    expected: &[u8],
    ranges: [[usize; 2]; 3],
    middle: Value,
) -> TestResult {
    let bytes = fs::read(path)?;
    assert_eq!(bytes, expected);
    let raw = consume("smiles_file_raw_oracle.py", &bytes)?;
    assert_eq!(raw["byte_length"], expected.len());
    assert_eq!(raw["physical_lines"], 4);
    let records = raw["records"].as_array().ok_or("raw records")?;
    assert_eq!(records.len(), 3);
    assert_eq!(records[0]["full_range"], json!([2, 14]));
    assert_eq!(records[1]["full_range"], json!([14, ranges[2][0]]));
    assert_eq!(
        records[2]["full_range"],
        json!([ranges[2][0], expected.len()])
    );
    assert_eq!(records[0]["application_data_hex"], "464952535400ff");
    assert_eq!(records[1]["application_data_hex"], "4b45455000ff");
    assert_eq!(records[2]["application_data_hex"], "4c415354");
    assert_eq!(records[0]["line_ending_range"], json!([12, 14]));
    assert_eq!(
        records[2]["line_ending_range"],
        json!([expected.len(), expected.len()])
    );
    let expected_graphs = [
        json!({"atoms":[[6,0],[6,0]],"bonds":[[0,1,"SINGLE"]]}),
        middle,
        json!({"atoms":[[6,0],[8,0]],"bonds":[[0,1,"SINGLE"]]}),
    ];
    for (index, (record, range)) in records.iter().zip(ranges).enumerate() {
        assert_eq!(record["line_number"], index + 2);
        assert_eq!(record["notation_range"], json!(range));
        let start = usize::try_from(
            record["notation_range"][0]
                .as_u64()
                .ok_or("notation start")?,
        )?;
        let end = usize::try_from(record["notation_range"][1].as_u64().ok_or("notation end")?)?;
        let notation = bytes.get(start..end).ok_or("notation slice")?;
        let chemical = consume("smiles_oracle.py", notation)?;
        assert_eq!(chemical["accepted"], true);
        assert_eq!(
            chemical["source_ascii"],
            std::str::from_utf8(
                expected
                    .get(range[0]..range[1])
                    .ok_or("expected notation")?
            )?
        );
        let atoms = chemical["atoms"].as_array().ok_or("consumer atoms")?;
        assert!(
            atoms
                .iter()
                .all(|atom| atom["isotope"] == 0 && atom["charge"] == 0)
        );
        let actual = json!({"atoms":atoms.iter().map(|atom| json!([atom["element"],atom["explicit_h"]])).collect::<Vec<_>>(),
            "bonds":chemical["bonds"]});
        assert_eq!(actual, expected_graphs[index]);
    }
    Ok(())
}

fn refuse(engine: &mut Engine, name: &str, params: Value, code: &str) -> TestResult {
    let before = state(engine)?;
    let owner = engine.content().clone();
    let stamp = engine.capture_stamp();
    let error = engine
        .dispatch(name, params)
        .err()
        .ok_or("refused operation succeeded")?;
    assert_eq!(error.info().code, code, "{name}: {error}");
    assert_eq!(state(engine)?, before);
    assert_eq!(engine.content(), &owner);
    assert!(engine.capture_stamp_matches(&stamp));
    Ok(())
}

#[test]
fn all_import_uses_captured_complete_source_and_a_fresh_clean_namespace() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("complete-λ.smi");
    let mut engine = Engine::default();
    engine.dispatch("ui.view", json!({"action":"set_zoom","percent":150.0}))?;
    fs::write(&input, SOURCE)?;
    engine.dispatch("document.open_smiles", json!({"path":input}))?;
    assert_eq!(
        state(&engine)?["smiles_file"],
        json!({"format":"smiles","byte_length":37,"record_count":3})
    );
    assert!(state(&engine)?["document"].is_null());
    let raw = engine.content().clone();
    let old_generation = engine.session_generation();
    let old_view_epoch = engine.view().epoch;
    let before = state(&engine)?;
    let prepared = engine.prepare(Command::parse(
        IMPORT,
        import_params(1, json!({"mode":"all"})),
    )?)?;
    assert!(prepared.is_deferred());
    assert_eq!(state(&engine)?, before);
    // Import owns the already-open source; a later disk replacement is not its input.
    fs::write(&input, b"N DIFFERENT DISK CONTENT\n")?;
    let completed = prepared.complete()?;
    assert_eq!(state(&engine)?, before);
    let result = engine.finish(completed)?;
    assert!(result.changed);
    assert_eq!(result.revision, 2);
    assert_eq!((result.undo_depth, result.redo_depth), (0, 0));
    assert_eq!(engine.session_generation(), old_generation + 1);
    assert_eq!(engine.view().zoom_percent, None);
    assert_eq!(engine.view().center, None);
    assert_eq!(engine.view().epoch, old_view_epoch + 1);
    assert_eq!(
        state(&engine)?["smiles_file"],
        json!({"format":"smiles","byte_length":37,"record_count":3,"projected_records":[1,2,3]})
    );
    assert_eq!(
        state(&engine)?["source"],
        json!({"format":"smiles","byte_length":37,"modified":false})
    );
    assert_eq!(
        serde_json::to_value(&result)?["file"]["path"],
        before["file"]["path"]
    );
    assert!(!result.file.dirty);
    assert!(result.file_operation.is_none());
    assert!(result.conversion.is_none());
    assert_eq!(
        graph(&engine)?,
        json!([
            [1, [[1, 6, 3], [2, 6, 3]], [[1, 1, 2, "single"]]],
            [2, [[3, 6, 3], [4, 7, 3], [5, 6, 3]], [[2, 3, 5, "single"]]],
            [3, [[6, 6, 3], [7, 8, 1]], [[3, 6, 7, "single"]]]
        ])
    );
    let drawing = engine.content().clone();
    assert_eq!(
        StoredContent::retained_bytes([&drawing, &raw]),
        StoredContent::retained_bytes([&drawing]) + size_of::<StoredContent>()
    );
    let output = directory.path("captured-whole-化学.smiles");
    engine.dispatch("document.save_smiles", json!({"path":output}))?;
    consume_saved(
        &output,
        SOURCE,
        [[2, 4], [14, 21], [30, 32]],
        json!({"atoms":[[6,0],[7,0],[6,0]],"bonds":[[0,2,"SINGLE"]]}),
    )?;
    assert_eq!(fs::read(input)?, b"N DIFFERENT DISK CONTENT\n");
    Ok(())
}

#[test]
fn notation_edit_tracks_writer_dfs_without_reordering_model_ids_and_keeps_saved_history_separate()
-> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let mut engine = open(&directory.path("mapping.smi"), SOURCE)?;
    engine.dispatch(IMPORT, import_params(1, json!({"mode":"all"})))?;
    let geometry = positions(&engine)?;
    let original_model = engine.document()?.clone();
    let owner = engine.content().clone();
    let stamp = engine.capture_stamp();
    let no_op = engine.prepare(Command::parse(
        "bond.set_order",
        order_params(2, 2, 2, "single"),
    )?)?;
    assert!(no_op.is_deferred());
    let result = engine.finish(no_op.complete()?)?;
    assert!(!result.changed);
    assert_eq!(
        (result.revision, result.undo_depth, result.redo_depth),
        (2, 0, 0)
    );
    assert_eq!(engine.content(), &owner);
    assert!(engine.capture_stamp_matches(&stamp));
    let unchanged = directory.path("no-op.smi");
    engine.dispatch("document.save_smiles", json!({"path":unchanged}))?;
    assert_eq!(fs::read(unchanged)?, SOURCE);
    let before = state(&engine)?;
    let prepared = engine.prepare(Command::parse(
        "bond.set_order",
        order_params(2, 2, 2, "double"),
    )?)?;
    assert!(prepared.is_deferred());
    assert_eq!(state(&engine)?, before);
    let completed = prepared.complete()?;
    assert_eq!(state(&engine)?, before);
    let result = engine.finish(completed)?;
    assert_eq!(
        (result.revision, result.undo_depth, result.redo_depth),
        (3, 1, 0)
    );
    assert!(result.changed && result.file.dirty);
    assert!(result.file_operation.is_none());
    assert_eq!(positions(&engine)?, geometry);
    assert_eq!(
        graph(&engine)?[1],
        json!([2, [[3, 6, 2], [4, 7, 3], [5, 6, 2]], [[2, 3, 5, "double"]]])
    );
    assert_eq!(
        state(&engine)?["source"],
        json!({"format":"smiles","byte_length":37,"modified":true})
    );
    assert_eq!(state(&engine)?["smiles_file"]["byte_length"], 47);
    let doubled = directory.path("double-λ.smiles");
    let saved = engine.dispatch("document.save_smiles", json!({"path":doubled}))?;
    assert!(!saved.file.dirty);
    let edited = engine.content().clone();
    consume_saved(
        &doubled,
        DOUBLE,
        [[2, 4], [14, 31], [40, 42]],
        json!({"atoms":[[6,2],[6,2],[7,3]],"bonds":[[0,1,"DOUBLE"]]}),
    )?;
    engine.dispatch("history.undo", json!({}))?;
    assert_eq!(engine.document()?, &original_model);
    assert_eq!(state(&engine)?["source"]["modified"], false);
    assert!(engine.state().file.dirty);
    engine.dispatch("history.redo", json!({}))?;
    assert_eq!(engine.content(), &edited);
    assert!(!engine.state().file.dirty);
    assert_eq!(state(&engine)?["source"]["modified"], true);
    // The rewritten token order is C,C,N, while the model stays C,N,C.
    engine.dispatch("bond.set_order", order_params(5, 2, 2, "single"))?;
    assert_eq!(engine.state().revision, 6);
    assert_eq!(positions(&engine)?, geometry);
    assert_eq!(engine.document()?, &original_model);
    assert_eq!(state(&engine)?["source"]["modified"], true);
    let singled = directory.path("single-化学.smi");
    engine.dispatch("document.save_smiles", json!({"path":singled}))?;
    consume_saved(
        &singled,
        SINGLE,
        [[2, 4], [14, 30], [39, 41]],
        json!({"atoms":[[6,3],[6,3],[7,3]],"bonds":[[0,1,"SINGLE"]]}),
    )?;
    engine.dispatch("document.open_smiles", json!({"path":singled}))?;
    assert_eq!(engine.state().revision, 7);
    assert!(state(&engine)?["document"].is_null());
    assert_eq!(
        state(&engine)?["smiles_file"],
        json!({"format":"smiles","byte_length":46,"record_count":3})
    );
    engine.dispatch(IMPORT, import_params(7, json!({"mode":"all"})))?;
    assert_eq!(engine.state().revision, 8);
    assert_eq!(
        (engine.state().undo_depth, engine.state().redo_depth),
        (0, 0)
    );
    assert_eq!(
        graph(&engine)?[1],
        json!([2, [[3, 6, 3], [4, 6, 3], [5, 7, 3]], [[2, 3, 4, "single"]]])
    );
    assert_eq!(state(&engine)?["source"]["modified"], false);
    assert!(!engine.state().file.dirty);
    Ok(())
}

#[test]
fn selected_record_uses_a_new_dense_namespace_and_preserves_unprojected_records() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let mut selected = open(&directory.path("selected.smi"), SOURCE)?;
    selected.dispatch(
        IMPORT,
        import_params(1, json!({"mode":"record","record_id":2})),
    )?;
    assert_eq!(
        graph(&selected)?,
        json!([[1, [[1, 6, 3], [2, 7, 3], [3, 6, 3]], [[1, 1, 3, "single"]]]])
    );
    assert_eq!(
        state(&selected)?["smiles_file"],
        json!({"format":"smiles","byte_length":37,"record_count":3,"projected_records":[2]})
    );
    let original = b"CC supported\r\n[13CH4]\tUNPROJECTED\0\xff\n";
    let mut engine = open(&directory.path("unprojected-isotope.smi"), original)?;
    refuse(
        &mut engine,
        IMPORT,
        import_params(1, json!({"mode":"all"})),
        "unsupported_content",
    )?;
    engine.dispatch(
        IMPORT,
        import_params(1, json!({"mode":"record","record_id":1})),
    )?;
    assert_eq!(
        graph(&engine)?,
        json!([[1, [[1, 6, 3], [2, 6, 3]], [[1, 1, 2, "single"]]]])
    );
    assert_eq!(
        state(&engine)?["smiles_file"]["projected_records"],
        json!([1])
    );
    engine.dispatch("bond.set_order", order_params(2, 1, 1, "double"))?;
    let output = directory.path("selected-output-λ.smiles");
    engine.dispatch("document.save_smiles", json!({"path":output}))?;
    let expected = b"[CH2]=[CH2] supported\r\n[13CH4]\tUNPROJECTED\0\xff\n";
    let actual = fs::read(&output)?;
    assert_eq!(actual, expected);
    let raw = consume("smiles_file_raw_oracle.py", &actual)?;
    let records = raw["records"].as_array().ok_or("raw records")?;
    assert_eq!(records.len(), 2);
    assert_eq!(records[0]["notation_range"], json!([0, 11]));
    assert_eq!(records[1]["notation_range"], json!([23, 30]));
    let second = consume(
        "smiles_oracle.py",
        actual.get(23..30).ok_or("unprojected notation")?,
    )?;
    assert_eq!(second["accepted"], true);
    assert_eq!(second["atoms"][0]["isotope"], 13);
    assert_eq!(second["atoms"][0]["explicit_h"], 4);
    assert_eq!(
        fs::read(directory.path("unprojected-isotope.smi"))?,
        original
    );
    Ok(())
}

#[test]
fn import_schema_targets_numeric_limits_and_backing_authority_are_closed() -> TestResult {
    let registry = bondline_commands::registry();
    assert_eq!(registry.len(), 38);
    let entry = registry
        .iter()
        .find(|entry| entry.name == IMPORT)
        .ok_or("import registry")?;
    assert!(!entry.read_only);
    let schema = entry.input_schema.as_value();
    assert_eq!(schema["type"], "object");
    assert_eq!(schema["additionalProperties"], false);
    let mut required = schema["required"]
        .as_array()
        .ok_or("required")?
        .iter()
        .map(|item| item.as_str().ok_or("required field"))
        .collect::<Result<Vec<_>, _>>()?;
    required.sort_unstable();
    assert_eq!(
        required,
        [
            "bond_length_pt",
            "expected_revision",
            "origin_x_pt",
            "origin_y_pt",
            "page_height_pt",
            "page_width_pt",
            "selection"
        ]
    );
    for selection in [
        json!({"mode":"unknown"}),
        json!({"mode":"record"}),
        json!({"mode":"all","record_id":1}),
        json!({"mode":"all","extra":false}),
        json!({"mode":"record","record_id":1,"extra":false}),
    ] {
        assert_eq!(
            Command::parse(IMPORT, import_params(1, selection))
                .err()
                .ok_or("open selection")?
                .info()
                .code,
            "invalid_params"
        );
    }
    for key in [
        "expected_revision",
        "selection",
        "bond_length_pt",
        "origin_x_pt",
        "origin_y_pt",
        "page_width_pt",
        "page_height_pt",
    ] {
        let mut params = import_params(1, json!({"mode":"all"}));
        params.as_object_mut().ok_or("params object")?.remove(key);
        assert_eq!(
            Command::parse(IMPORT, params)
                .err()
                .ok_or("missing field")?
                .info()
                .code,
            "invalid_params"
        );
    }
    for key in ["path", "profile", "notation"] {
        let mut params = import_params(1, json!({"mode":"all"}));
        params[key] = json!("not declared");
        assert_eq!(
            Command::parse(IMPORT, params)
                .err()
                .ok_or("extra field")?
                .info()
                .code,
            "invalid_params"
        );
    }
    let directory = support::FileTestDirectory::new()?;
    let mut engine = open(&directory.path("guards.smi"), SOURCE)?;
    refuse(
        &mut engine,
        IMPORT,
        import_params(1, json!({"mode":"record","record_id":0})),
        "invalid_params",
    )?;
    refuse(
        &mut engine,
        IMPORT,
        import_params(1, json!({"mode":"record","record_id":4})),
        "not_found",
    )?;
    for (key, value, code) in [
        ("bond_length_pt", 0.0, "invalid_params"),
        ("bond_length_pt", -1.0, "invalid_params"),
        ("page_width_pt", 0.0, "invalid_params"),
        ("page_height_pt", 100001.0, "invalid_params"),
        ("origin_x_pt", -1.0, "unsupported_content"),
        ("origin_y_pt", 800.0, "unsupported_content"),
    ] {
        let mut params = import_params(1, json!({"mode":"all"}));
        params[key] = json!(value);
        refuse(&mut engine, IMPORT, params, code)?;
    }
    let mut empty = open(&directory.path("empty.smi"), b"\r\n \tignored\n")?;
    refuse(
        &mut empty,
        IMPORT,
        import_params(1, json!({"mode":"all"})),
        "unsupported_content",
    )?;
    let mut drawing = Engine::default();
    refuse(
        &mut drawing,
        IMPORT,
        import_params(0, json!({"mode":"all"})),
        "unsupported_content",
    )?;
    let sdf_path = directory.path("source.sdf");
    fs::write(
        &sdf_path,
        include_bytes!("../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf"),
    )?;
    let mut sdf = Engine::default();
    sdf.dispatch("document.open_sdf", json!({"path":sdf_path}))?;
    refuse(
        &mut sdf,
        IMPORT,
        import_params(1, json!({"mode":"all"})),
        "unsupported_content",
    )?;
    engine.dispatch(IMPORT, import_params(1, json!({"mode":"all"})))?;
    for (name, params) in [
        (
            "bond.create",
            json!({"page_id":1,"begin_x_pt":320,"begin_y_pt":480,"end_x_pt":384,"end_y_pt":480}),
        ),
        (
            "bond.extend",
            json!({"expected_revision":2,"page_id":1,"fragment_id":1,"begin_atom_id":1,"end_x_pt":320,"end_y_pt":480,"hydrogen_policy":"rectify_terminal_carbon"}),
        ),
        (
            "structure.paste_smiles",
            json!({"expected_revision":2,"page_id":1,"notation":"CC","bond_length_pt":64,"origin_x_pt":320,"origin_y_pt":480}),
        ),
        (
            "page.set_size",
            json!({"page_id":1,"width_pt":600,"height_pt":800}),
        ),
    ] {
        refuse(&mut engine, name, params, "unsupported_content")?;
    }
    let cross_format = directory.path("loss.cdxml");
    refuse(
        &mut engine,
        "document.save_cdxml",
        json!({"path":cross_format}),
        "unsupported_content",
    )?;
    assert!(!cross_format.exists());
    Ok(())
}

#[test]
fn deferred_import_and_mapped_edits_reject_cancellation_foreign_source_and_file_close_aba()
-> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("context-A.smi");
    let mut engine = open(&input, SOURCE)?;
    let original = state(&engine)?;
    for cancel_after_complete in [false, true] {
        let cancel = Cancellation::default();
        let prepared = engine.prepare(Command::parse(
            IMPORT,
            import_params(1, json!({"mode":"all"})),
        )?)?;
        let error = if cancel_after_complete {
            let completed = prepared.complete_with_cancel(&cancel)?;
            cancel.cancel();
            engine.finish(completed).err().ok_or("late cancellation")?
        } else {
            cancel.cancel();
            prepared
                .complete_with_cancel(&cancel)
                .err()
                .ok_or("early cancellation")?
        };
        assert_eq!(error.info().code, "cancelled");
        assert_eq!(state(&engine)?, original);
    }
    let completed = engine
        .prepare(Command::parse(
            IMPORT,
            import_params(1, json!({"mode":"all"})),
        )?)?
        .complete()?;
    let mut foreign = open(&input, SOURCE)?;
    let foreign_state = state(&foreign)?;
    assert_eq!(
        foreign
            .finish(completed)
            .err()
            .ok_or("foreign source")?
            .info()
            .code,
        "stale_revision"
    );
    assert_eq!(state(&foreign)?, foreign_state);
    let completed = engine
        .prepare(Command::parse(
            IMPORT,
            import_params(1, json!({"mode":"all"})),
        )?)?
        .complete()?;
    engine.dispatch(
        "document.close",
        json!({"expected_revision":1,"action":"request"}),
    )?;
    engine.dispatch(
        "document.close",
        json!({"expected_revision":1,"action":"cancel"}),
    )?;
    let after_close = state(&engine)?;
    assert_eq!(
        engine
            .finish(completed)
            .err()
            .ok_or("close ABA")?
            .info()
            .code,
        "ui_stale_request"
    );
    assert_eq!(state(&engine)?, after_close);
    let completed = engine
        .prepare(Command::parse(
            IMPORT,
            import_params(1, json!({"mode":"all"})),
        )?)?
        .complete()?;
    let stamp = engine.capture_stamp();
    let other = directory.path("context-B.smiles");
    engine.dispatch("document.save_smiles", json!({"path":other}))?;
    engine.dispatch(
        "document.save_smiles",
        json!({"path":input,"overwrite":true}),
    )?;
    assert!(engine.capture_stamp_matches(&stamp));
    let after_save = state(&engine)?;
    assert_eq!(
        engine
            .finish(completed)
            .err()
            .ok_or("association ABA")?
            .info()
            .code,
        "ui_stale_request"
    );
    assert_eq!(state(&engine)?, after_save);
    assert_eq!(fs::read(&other)?, SOURCE);
    assert_eq!(fs::read(&input)?, SOURCE);
    let completed = engine
        .prepare(Command::parse(
            IMPORT,
            import_params(1, json!({"mode":"all"})),
        )?)?
        .complete()?;
    engine.dispatch("document.new", json!({"expected_revision":1}))?;
    let replacement = state(&engine)?;
    assert_eq!(
        engine
            .finish(completed)
            .err()
            .ok_or("new namespace")?
            .info()
            .code,
        "stale_revision"
    );
    assert_eq!(state(&engine)?, replacement);
    let mut committed = open(&input, SOURCE)?;
    let completed = committed
        .prepare(Command::parse(
            IMPORT,
            import_params(1, json!({"mode":"all"})),
        )?)?
        .complete()?;
    committed.dispatch(
        "document.close",
        json!({"expected_revision":1,"action":"request"}),
    )?;
    assert!(committed.complete_close());
    let closed = state(&committed)?;
    assert_eq!(
        committed
            .finish(completed)
            .err()
            .ok_or("committed close")?
            .info()
            .code,
        "close_committed"
    );
    assert_eq!(state(&committed)?, closed);
    let mut mapped = open(&input, SOURCE)?;
    mapped.dispatch(IMPORT, import_params(1, json!({"mode":"all"})))?;
    let before_edit = state(&mapped)?;
    let cancel = Cancellation::default();
    let completed = mapped
        .prepare(Command::parse(
            "bond.set_order",
            order_params(2, 2, 2, "double"),
        )?)?
        .complete_with_cancel(&cancel)?;
    cancel.cancel();
    assert_eq!(
        mapped
            .finish(completed)
            .err()
            .ok_or("mapped edit cancellation")?
            .info()
            .code,
        "cancelled"
    );
    assert_eq!(state(&mapped)?, before_edit);
    let completed = mapped
        .prepare(Command::parse(
            "bond.set_order",
            order_params(2, 2, 2, "double"),
        )?)?
        .complete()?;
    mapped.dispatch(
        "document.save_smiles",
        json!({"path":other,"overwrite":true}),
    )?;
    mapped.dispatch(
        "document.save_smiles",
        json!({"path":input,"overwrite":true}),
    )?;
    let before_refusal = state(&mapped)?;
    assert_eq!(
        mapped
            .finish(completed)
            .err()
            .ok_or("mapped edit file ABA")?
            .info()
            .code,
        "ui_stale_request"
    );
    assert_eq!(state(&mapped)?, before_refusal);
    Ok(())
}

#[test]
fn a_published_smiles_save_keeps_its_lease_and_truthful_receipt_after_new_without_reassociation()
-> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let mut engine = open(&directory.path("published-source.smi"), SOURCE)?;
    engine.dispatch(IMPORT, import_params(1, json!({"mode":"all"})))?;
    engine.dispatch("bond.set_order", order_params(2, 2, 2, "double"))?;
    let output = directory.path("published-old-λ.smiles");
    let completed = engine
        .prepare(Command::parse(
            "document.save_smiles",
            json!({"path":output}),
        )?)?
        .complete()?;
    assert_eq!(fs::read(&output)?, DOUBLE);
    engine.dispatch(
        "document.new",
        json!({"expected_revision":3,"discard_unsaved":true}),
    )?;
    let replacement = state(&engine)?;
    let owner = engine.content().clone();
    let denied = directory.path("busy.cdxml");
    refuse(
        &mut engine,
        "document.save_cdxml",
        json!({"path":denied}),
        "busy",
    )?;
    assert!(!denied.exists());
    let result = engine.finish(completed)?;
    let receipt = serde_json::to_value(result.file_operation.ok_or("historical Save receipt")?)?;
    assert_eq!(receipt["operation"], "save");
    assert_eq!(receipt["format"], "smiles");
    assert_eq!(receipt["input_revision"], 3);
    assert_eq!(receipt["byte_length"], 47);
    assert!(receipt["ui_request_id"].is_null());
    assert!(receipt["ui_request_current"].is_null());
    assert_eq!(state(&engine)?, replacement);
    assert_eq!(engine.content(), &owner);
    assert!(engine.state().file.path.is_none());
    assert!(!engine.state().file.dirty);
    assert!(!engine.finish_ui_file(1, false));
    consume_saved(
        &output,
        DOUBLE,
        [[2, 4], [14, 31], [40, 42]],
        json!({"atoms":[[6,2],[6,2],[7,3]],"bonds":[[0,1,"DOUBLE"]]}),
    )?;
    Ok(())
}

use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig {
        cases:16,
        rng_seed:proptest::test_runner::RngSeed::Fixed(0xb0_0d_5a_17),
        failure_persistence:None,
        ..ProptestConfig::default()
    })]

    #[test]
    fn generated_order_edits_restore_complete_smiles_snapshots_and_maps(
        orders in proptest::collection::vec(0usize..3,1..24)
    ) {
        let directory = support::FileTestDirectory::new()?;
        let input = directory.path("generated.smi");
        let source = b"CC\tFIRST\0\xff\r\nN\tOPAQUE\0\xff";
        let mut engine = open(&input,source).map_err(|error|TestCaseError::fail(error.to_string()))?;
        engine.dispatch(IMPORT,import_params(1,json!({"mode":"record","record_id":1})))?;
        let mut snapshots = vec![engine.content().clone()];
        let geometry = positions(&engine).map_err(|error|TestCaseError::fail(error.to_string()))?;
        for index in orders {
            let (name,hydrogens) = [("single",3),("double",2),("triple",1)][index];
            let revision = engine.state().revision;
            let result = engine.dispatch("bond.set_order",order_params(revision,1,1,name))?;
            let actual = graph(&engine).map_err(|error|TestCaseError::fail(error.to_string()))?;
            prop_assert_eq!(actual,json!([[1,[[1,6,hydrogens],[2,6,hydrogens]],[[1,1,2,name]]]]));
            prop_assert_eq!(positions(&engine).map_err(|error|TestCaseError::fail(error.to_string()))?,geometry.clone());
            if result.changed { snapshots.push(engine.content().clone()); }
        }
        for expected in snapshots.iter().rev().skip(1) {
            engine.dispatch("history.undo",json!({}))?;
            prop_assert_eq!(engine.content(),expected);
        }
        prop_assert_eq!(engine.content(),&snapshots[0]);
        for expected in snapshots.iter().skip(1) {
            engine.dispatch("history.redo",json!({}))?;
            prop_assert_eq!(engine.content(),expected);
        }
        prop_assert_eq!(engine.content(),snapshots.last().ok_or_else(||TestCaseError::fail("snapshot"))?);
        prop_assert_eq!(fs::read(input)?,source);
    }
}
