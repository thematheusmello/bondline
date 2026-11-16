use bondline_commands::{Cancellation, Command, ConvertSmilesToCdxml, Engine};
use serde_json::{Value, json};
use std::{fs, path::Path};

#[path = "../../cli/tests/support/conversion.rs"]
mod conversion_support;
#[path = "../../../xtask/oracles/files_test_support.rs"]
mod support;

type TestResult = Result<(), Box<dyn std::error::Error>>;
const NAME: &str = "file.convert_smiles_to_cdxml";

fn params(revision: u64, input: &Path, output: &Path) -> Value {
    json!({"expected_revision":revision,"input_path":input,"output_path":output,
        "bond_length_pt":48.0,"origin_x_pt":72.0,"origin_y_pt":72.0,
        "page_width_pt":210.0*72.0/25.4,"page_height_pt":297.0*72.0/25.4})
}

#[test]
fn conversion_publishes_a_new_drawing_without_changing_the_session() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("input.smi");
    let output = directory.path("converted-化学.mol");
    fs::write(&input, b"CCC\r\n")?;
    let mut engine = Engine::default();
    let before = serde_json::to_value(engine.state())?;
    let result = engine.dispatch(NAME, params(0, &input, &output))?;
    assert_eq!(serde_json::to_value(engine.state())?, before);
    let result = serde_json::to_value(result)?;
    assert_eq!(result["changed"], false);
    assert_eq!(result["revision"], 0);
    assert!(result.get("file_operation").is_none());
    assert_eq!(result["conversion"]["profile"], "neutral_carbon_path_v1");
    assert_eq!(result["conversion"]["input_revision"], 0);
    assert_eq!(result["conversion"]["session_generation"], 0);
    assert_eq!(
        result["conversion"]["input_path"],
        json!(fs::canonicalize(&input)?)
    );
    assert_eq!(result["conversion"]["input_byte_length"], 5);
    assert_eq!(result["conversion"]["atom_count"], 3);
    assert_eq!(result["conversion"]["bond_count"], 2);
    assert_eq!(
        result["conversion"]["output"]["path"],
        json!(fs::canonicalize(&output)?)
    );
    assert_eq!(
        result["conversion"]["output"]["byte_length"],
        fs::read(&output)?.len()
    );
    assert!(result["conversion"]["output"]["directory_synced"].is_boolean());
    assert_eq!(fs::read(&input)?, b"CCC\r\n");
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let consumed = std::process::Command::new(python)
        .arg("-B")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../io/tests/smiles_layout_oracle.py"
        ))
        .arg(
            json!([
                ["72", "72"],
                ["113.56921938165306", "96"],
                ["155.1384387633061", "72"]
            ])
            .to_string(),
        )
        .stdin(fs::File::open(&output)?)
        .output()?;
    assert!(
        consumed.status.success(),
        "{}",
        String::from_utf8_lossy(&consumed.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&consumed.stdout)?,
        json!({"smiles":"CCC","formula":"C3H8","atoms":3,"bonds":2})
    );
    let custom_input = directory.path("custom-input.smi");
    let custom_output = directory.path("custom-output.cdxml");
    fs::write(&custom_input, b"CC\n")?;
    let mut custom = params(0, &custom_input, &custom_output);
    for (field, value) in [
        ("bond_length_pt", 40.0),
        ("origin_x_pt", 100.0),
        ("origin_y_pt", 200.0),
        ("page_width_pt", 612.0),
        ("page_height_pt", 792.0),
    ] {
        custom[field] = json!(value);
    }
    engine.dispatch(NAME, custom)?;
    support::independently_read_file(&custom_output, 1)?;
    assert_eq!(serde_json::to_value(engine.state())?, before);
    Ok(())
}

#[test]
fn conversion_has_one_closed_required_schema_and_validates_direct_typed_numbers() -> TestResult {
    let all = bondline_commands::registry();
    assert_eq!(all.len(), 32);
    let found = all
        .iter()
        .filter(|entry| entry.name == NAME)
        .collect::<Vec<_>>();
    assert_eq!(found.len(), 1);
    assert!(
        !found[0].read_only,
        "disk publication is not a read-only tool"
    );
    let schema = found[0].input_schema.as_value();
    assert_eq!(schema["additionalProperties"], false);
    assert_eq!(schema["required"].as_array().ok_or("required")?.len(), 8);
    assert_eq!(
        schema["properties"]["bond_length_pt"]["exclusiveMinimum"],
        0.0
    );
    assert_eq!(schema["properties"]["origin_x_pt"]["minimum"], -100000.0);
    assert_eq!(schema["properties"]["origin_y_pt"]["maximum"], 100000.0);
    assert_eq!(
        schema["properties"]["page_width_pt"]["exclusiveMinimum"],
        0.0
    );
    assert_eq!(schema["properties"]["page_height_pt"]["maximum"], 100000.0);
    let valid = params(0, Path::new("input.smi"), Path::new("new.cdxml"));
    for key in valid.as_object().ok_or("params")?.keys() {
        let mut missing = valid.clone();
        missing.as_object_mut().ok_or("params")?.remove(key);
        assert_eq!(
            Command::parse(NAME, missing)
                .err()
                .ok_or("missing required")?
                .info()
                .code,
            "invalid_params"
        );
    }
    for key in [
        "overwrite",
        "discard_unsaved",
        "hydrogen_policy",
        "ui_request_id",
        "unexpected",
    ] {
        let mut invalid = valid.clone();
        invalid[key] = json!(true);
        assert_eq!(
            Command::parse(NAME, invalid)
                .err()
                .ok_or("unknown field")?
                .info()
                .code,
            "invalid_params"
        );
    }
    for field in [
        "expected_revision",
        "input_path",
        "output_path",
        "bond_length_pt",
        "origin_x_pt",
        "origin_y_pt",
        "page_width_pt",
        "page_height_pt",
    ] {
        let mut invalid = valid.clone();
        invalid[field] = Value::Null;
        assert_eq!(
            Command::parse(NAME, invalid)
                .err()
                .ok_or("null required value")?
                .info()
                .code,
            "invalid_params"
        );
    }
    let mut engine = Engine::default();
    let before = serde_json::to_value(engine.state())?;
    for field in 0..5 {
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let mut typed: ConvertSmilesToCdxml = serde_json::from_value(valid.clone())?;
            match field {
                0 => typed.bond_length_pt = bad,
                1 => typed.origin_x_pt = bad,
                2 => typed.origin_y_pt = bad,
                3 => typed.page_width_pt = bad,
                _ => typed.page_height_pt = bad,
            }
            assert_eq!(
                engine
                    .prepare(Command::ConvertSmilesToCdxml(typed))
                    .err()
                    .ok_or("nonfinite parameter")?
                    .info()
                    .code,
                "invalid_params"
            );
            assert_eq!(serde_json::to_value(engine.state())?, before);
        }
    }
    for (field, bad) in [
        ("bond_length_pt", 0.0),
        ("bond_length_pt", -1.0),
        ("origin_x_pt", 100001.0),
        ("origin_y_pt", -100001.0),
        ("page_width_pt", 0.0),
        ("page_height_pt", 100001.0),
    ] {
        let mut invalid = valid.clone();
        invalid[field] = json!(bad);
        assert_eq!(
            engine
                .prepare(Command::parse(NAME, invalid)?)
                .err()
                .ok_or("invalid numeric parameter")?
                .info()
                .code,
            "invalid_params"
        );
        assert_eq!(serde_json::to_value(engine.state())?, before);
    }
    Ok(())
}

#[test]
fn conversion_retains_whole_drawing_and_sdf_sources_history_and_close_approval() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("original.smi");
    fs::write(&input, b"CC\n")?;
    for sdf in [false, true] {
        let source = directory.path(if sdf { "source.sdf" } else { "source.cdxml" });
        fs::write(
            &source,
            if sdf {
                include_bytes!("../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf").as_slice()
            } else {
                include_bytes!("../../io/tests/fixtures/bondline-v0.cdxml").as_slice()
            },
        )?;
        let mut engine = Engine::default();
        engine.dispatch(
            if sdf {
                "document.open_sdf"
            } else {
                "document.open_cdxml"
            },
            json!({"path":source}),
        )?;
        if !sdf {
            engine.dispatch(
                "page.set_size",
                json!({"page_id":1,"width_pt":700.0,"height_pt":900.0}),
            )?;
            engine.dispatch("history.undo", json!({}))?;
            assert_eq!(engine.state().redo_depth, 1);
        }
        let revision = engine.state().revision;
        engine.dispatch(
            "document.close",
            json!({"expected_revision":revision,"action":"request"}),
        )?;
        let before = serde_json::to_value(engine.state())?;
        let content = engine.content().clone();
        let output = directory.path(if sdf {
            "sdf-session-output.cdxml"
        } else {
            "drawing-session-output.cdxml"
        });
        let result = engine.dispatch(NAME, params(revision, &input, &output))?;
        assert_eq!(serde_json::to_value(engine.state())?, before);
        assert_eq!(engine.content(), &content);
        assert_eq!(
            serde_json::to_value(result)?["conversion"]["session_generation"],
            1
        );
        assert_eq!(
            fs::read(&source)?,
            if sdf {
                include_bytes!("../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf").as_slice()
            } else {
                include_bytes!("../../io/tests/fixtures/bondline-v0.cdxml").as_slice()
            }
        );
        conversion_support::independent(&output, 2)?;
    }
    Ok(())
}

#[test]
fn conversion_and_native_saves_share_the_lease_until_finish_or_drop() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("input.smi");
    fs::write(&input, b"CC\n")?;
    for native_first in [false, true] {
        for published in [false, true] {
            let output = directory.path(&format!("convert-{native_first}-{published}.cdxml"));
            let native = directory.path(&format!("native-{native_first}-{published}.cdxml"));
            let mut engine = Engine::default();
            let before = serde_json::to_value(engine.state())?;
            let first = if native_first {
                Command::parse("document.save_cdxml", json!({"path":native}))?
            } else {
                Command::parse(NAME, params(0, &input, &output))?
            };
            let prepared = engine.prepare(first)?;
            assert!(prepared.is_deferred());
            let completed = if published {
                Some(prepared.complete()?)
            } else {
                let second = if native_first {
                    Command::parse(NAME, params(0, &input, &output))?
                } else {
                    Command::parse("document.save_cdxml", json!({"path":native}))?
                };
                assert_eq!(
                    engine
                        .prepare(second)
                        .err()
                        .ok_or("save lease escaped")?
                        .info()
                        .code,
                    "busy"
                );
                assert_eq!(serde_json::to_value(engine.state())?, before);
                drop(prepared);
                None
            };
            if let Some(completed) = completed {
                let second = if native_first {
                    Command::parse(NAME, params(0, &input, &output))?
                } else {
                    Command::parse("document.save_cdxml", json!({"path":native}))?
                };
                assert_eq!(
                    engine
                        .prepare(second)
                        .err()
                        .ok_or("published lease escaped")?
                        .info()
                        .code,
                    "busy"
                );
                assert_eq!(serde_json::to_value(engine.state())?, before);
                if native_first {
                    assert!(!output.try_exists()?);
                } else {
                    assert!(!native.try_exists()?);
                }
                drop(completed);
            }
            assert_eq!(serde_json::to_value(engine.state())?, before);
            let free = engine.prepare(Command::parse(
                NAME,
                params(
                    0,
                    &input,
                    &directory.path(&format!("free-{native_first}-{published}.cdxml")),
                ),
            )?)?;
            drop(free);
        }
    }
    let mut engine = Engine::default();
    let output = directory.path("finished.cdxml");
    let completed = engine
        .prepare(Command::parse(NAME, params(0, &input, &output))?)?
        .complete()?;
    engine.finish(completed)?;
    drop(engine.prepare(Command::parse(
        "document.save_cdxml",
        json!({"path":directory.path("after-finish.cdxml")}),
    )?)?);
    Ok(())
}

#[test]
fn stale_cancelled_and_refused_conversions_leave_state_and_destination_untouched() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("input.smi");
    let output = directory.path("output.cdxml");
    fs::write(&input, b"CC\n")?;
    let mut engine = Engine::default();
    let before = serde_json::to_value(engine.state())?;
    assert_eq!(
        engine
            .prepare(Command::parse(NAME, params(1, &input, &output))?)
            .err()
            .ok_or("stale accepted")?
            .info()
            .code,
        "stale_revision"
    );
    assert!(!output.try_exists()?);
    let prepared = engine.prepare(Command::parse(NAME, params(0, &input, &output))?)?;
    let cancel = Cancellation::default();
    cancel.cancel();
    assert_eq!(
        prepared
            .complete_with_cancel(&cancel)
            .err()
            .ok_or("cancelled conversion accepted")?
            .info()
            .code,
        "cancelled"
    );
    assert!(!output.try_exists()?);
    for (source, code) in [
        (b"CO\n".as_slice(), "unsupported_content"),
        (b"c1ccccc1\n".as_slice(), "unsupported_content"),
        (b"C:C\n".as_slice(), "unsupported_content"),
        (b"C(\n".as_slice(), "invalid_smiles"),
        (b"CC\nCCC\n".as_slice(), "invalid_smiles"),
    ] {
        fs::write(&input, source)?;
        assert_eq!(
            engine
                .dispatch(NAME, params(0, &input, &output))
                .err()
                .ok_or("unsupported conversion accepted")?
                .info()
                .code,
            code
        );
        assert_eq!(serde_json::to_value(engine.state())?, before);
        assert!(!output.try_exists()?);
        assert_eq!(fs::read(&input)?, source);
    }
    fs::write(&input, b"CC\n")?;
    fs::write(&output, b"unrelated existing bytes")?;
    assert_eq!(
        engine
            .dispatch(NAME, params(0, &input, &output))
            .err()
            .ok_or("clobbered output")?
            .info()
            .code,
        "already_exists"
    );
    assert_eq!(fs::read(&output)?, b"unrelated existing bytes");
    assert_eq!(serde_json::to_value(engine.state())?, before);
    drop(engine.prepare(Command::parse(
        "document.save_cdxml",
        json!({"path":directory.path("lease-released.cdxml")}),
    )?)?);
    Ok(())
}

#[test]
fn published_conversion_receipts_report_captured_identity_and_latest_state_after_open_or_close()
-> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("input.smi");
    fs::write(&input, b"CC\n")?;
    let native = directory.path("native.cdxml");
    fs::write(
        &native,
        include_bytes!("../../io/tests/fixtures/bondline-v0.cdxml"),
    )?;
    for close in [false, true] {
        let output = directory.path(if close {
            "late-close.cdxml"
        } else {
            "late-open.cdxml"
        });
        let mut engine = Engine::default();
        let prepared = engine.prepare(Command::parse(NAME, params(0, &input, &output))?)?;
        if close {
            engine.dispatch(
                "document.close",
                json!({"expected_revision":0,"action":"request"}),
            )?;
            assert!(engine.complete_close());
        } else {
            engine.dispatch("document.open_cdxml", json!({"path":native}))?;
        }
        let before = serde_json::to_value(engine.state())?;
        let content = engine.content().clone();
        let cancel = Cancellation::default();
        let completed = prepared.complete_with_cancel(&cancel)?;
        cancel.cancel(); // Publication already happened: return the real receipt.
        let mut result = serde_json::to_value(engine.finish(completed)?)?;
        let receipt = result
            .as_object_mut()
            .ok_or("result")?
            .remove("conversion")
            .ok_or("conversion")?;
        assert_eq!(result, before);
        assert_eq!(engine.content(), &content);
        assert_eq!(serde_json::to_value(engine.state())?, before);
        assert_eq!(receipt["input_revision"], 0);
        assert_eq!(receipt["session_generation"], 0);
        conversion_support::independent(&output, 2)?;
        if close {
            assert_eq!(
                engine
                    .prepare(Command::parse(
                        NAME,
                        params(0, &input, &directory.path("after-close.cdxml"))
                    )?)
                    .err()
                    .ok_or("committed close accepted")?
                    .info()
                    .code,
                "close_committed"
            );
        }
    }
    Ok(())
}

#[test]
fn accepted_native_file_requests_cannot_bind_conversion_or_be_completed_by_it() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("input.smi");
    let output = directory.path("output.cdxml");
    fs::write(&input, b"CC\n")?;
    let mut engine = Engine::default();
    engine.set_ui_file_host(true);
    let requested =
        engine.dispatch("ui.file", json!({"expected_revision":0,"action":"save_as"}))?;
    let id = requested.ui_file_operation.ok_or("request")?;
    engine.claim_ui_file(id)?;
    let before = serde_json::to_value(engine.state())?;
    assert_eq!(
        engine
            .prepare_ui_file(id, Command::parse(NAME, params(0, &input, &output))?)
            .err()
            .ok_or("bound conversion to native Save As")?
            .info()
            .code,
        "invalid_params"
    );
    assert_eq!(serde_json::to_value(engine.state())?, before);
    let result = engine.dispatch(NAME, params(0, &input, &output))?;
    assert!(serde_json::to_value(result)?.get("conversion").is_some());
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert!(!engine.finish_ui_file(id, false));
    Ok(())
}

#[test]
fn converted_native_output_opens_extends_undoes_redoes_saves_and_reopens() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("workflow.smi");
    let output = directory.path("workflow-λ.cdxml");
    fs::write(&input, b"CC\n")?;
    let mut engine = Engine::default();
    let converted = engine.dispatch(NAME, conversion_support::params(0, &input, &output))?;
    conversion_support::assert_conversion(&serde_json::to_value(converted)?, &input, &output)?;
    conversion_support::independent(&output, 2)?;
    let original = fs::read(&output)?;
    for (name, params, count, revision, dirty) in [
        ("document.open_cdxml", json!({"path":output}), 2, 1, false),
        ("bond.extend", conversion_support::extend(1), 3, 2, true),
        ("history.undo", json!({}), 2, 3, false),
        ("history.redo", json!({}), 3, 4, true),
        (
            "document.save_cdxml",
            json!({"path":output,"overwrite":true}),
            3,
            4,
            false,
        ),
        ("document.open_cdxml", json!({"path":output}), 3, 5, false),
    ] {
        let result = serde_json::to_value(engine.dispatch(name, params)?)?;
        conversion_support::assert_chain(&result, count)?;
        assert!(result.get("conversion").is_none());
        assert_eq!(result["revision"], revision);
        assert_eq!(result["file"]["dirty"], dirty);
        if name == "history.undo" {
            assert_eq!(engine.stored_document()?.to_cdxml()?.as_ref(), original);
            assert_eq!(result["source"]["modified"], false);
        }
        if name == "document.save_cdxml" {
            assert_eq!(result["source"]["modified"], true);
            conversion_support::independent(&output, 3)?;
        }
    }
    assert!(!engine.state().source.ok_or("native source")?.modified);
    assert_eq!(engine.state().undo_depth, 0);
    assert!(engine.stored_document()?.is_fully_mapped());
    assert_eq!(fs::read(&input)?, b"CC\n");
    Ok(())
}

#[test]
fn conversion_neither_clears_a_current_trusted_save_receipt_nor_renews_one_after_undo_aba()
-> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("receipt-source.smi");
    fs::write(&input, b"CC\n")?;
    for obsolete in [false, true] {
        let mut engine = Engine::default();
        engine.set_ui_file_host(true);
        let request =
            engine.dispatch("ui.file", json!({"expected_revision":0,"action":"save_as"}))?;
        let id = request.ui_file_operation.ok_or("request")?;
        engine.claim_ui_file(id)?;
        let native = directory.path(&format!("trusted-native-{obsolete}.cdxml"));
        let save = engine.prepare_ui_file(
            id,
            Command::parse("document.save_cdxml", json!({"path":native}))?,
        )?;
        engine.finish(save.complete()?)?;
        if obsolete {
            let original = engine.content().clone();
            engine.dispatch(
                "page.set_size",
                json!({"page_id":1,"width_pt":700.0,"height_pt":900.0}),
            )?;
            engine.dispatch("history.undo", json!({}))?;
            assert_eq!(engine.content(), &original);
        }
        let revision = engine.state().revision;
        let before = serde_json::to_value(engine.state())?;
        let output = directory.path(&format!("receipt-conversion-{obsolete}.cdxml"));
        engine.dispatch(NAME, params(revision, &input, &output))?;
        assert_eq!(serde_json::to_value(engine.state())?, before);
        assert_eq!(engine.finish_ui_file(id, false), !obsolete);
        conversion_support::independent(&output, 2)?;
    }
    Ok(())
}

#[test]
fn independent_workflow_oracle_rejects_changed_coordinates_hydrogens_bonds_and_malformed_bytes()
-> TestResult {
    use std::io::Write as _;
    use std::process::{Command as Process, Stdio};
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("oracle-source.smi");
    let output = directory.path("oracle-source.cdxml");
    fs::write(&input, b"CC\n")?;
    Engine::default().dispatch(NAME, params(0, &input, &output))?;
    conversion_support::independent(&output, 2)?;
    let original = fs::read_to_string(&output)?;
    for (old, new) in [
        ("p=\"120 72\"", "p=\"121 72\""),
        ("NumHydrogens=\"3\"", "NumHydrogens=\"2\""),
        ("Order=\"1\"", "Order=\"2\""),
    ] {
        assert!(
            original.contains(old),
            "literal mutation must change actual XML"
        );
        let mutated = original.replace(old, new);
        let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
            if cfg!(windows) {
                "python".into()
            } else {
                "python3".into()
            }
        });
        let mut child = Process::new(python)
            .arg("-B")
            .arg(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/smiles_conversion_oracle.py"
            ))
            .arg("2")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        child
            .stdin
            .take()
            .ok_or("oracle stdin")?
            .write_all(mutated.as_bytes())?;
        assert!(
            !child.wait_with_output()?.status.success(),
            "oracle accepted {old} -> {new}"
        );
    }
    let mut malformed = original.into_bytes();
    malformed.push(0);
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child = Process::new(python)
        .arg("-B")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/smiles_conversion_oracle.py"
        ))
        .arg("2")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("oracle stdin")?
        .write_all(&malformed)?;
    assert!(
        !child.wait_with_output()?.status.success(),
        "oracle accepted appended NUL bytes"
    );
    conversion_support::independent(&output, 2)?;
    Ok(())
}
