use serde_json::{Value, json};
use std::process::Command;
#[path = "support/cdx.rs"]
mod cdx_support;
#[path = "support/chain.rs"]
mod chain_support;
#[path = "support/conversion.rs"]
mod conversion_support;
#[path = "../../../xtask/oracles/files_test_support.rs"]
mod file_support;
#[path = "support/mol.rs"]
mod mol_support;
#[path = "../../../xtask/oracles/sdf_field_test_support.rs"]
mod sdf_field_support;
#[path = "../../../xtask/oracles/sdf_record_test_support.rs"]
mod sdf_record_support;
#[path = "support/smiles_export.rs"]
mod smiles_export_support;
#[path = "support/smiles_file.rs"]
mod smiles_file_support;

#[path = "support/smiles_record.rs"]
mod smiles_record_support;

#[test]
fn executable_explicit_smiles_record_query_selects_nonfirst_notation_and_keeps_metadata_private()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = file_support::FileTestDirectory::new()?;
    let input = directory.path("cli-record-化学.smi");
    std::fs::write(&input, smiles_record_support::SOURCE)?;
    for stage in 0..3 {
        let output = directory.path(&format!("cli-record-copy-{stage}.data"));
        let mut process = Command::new(env!("CARGO_BIN_EXE_bondline-cli"));
        process.arg("run");
        let mut requests = vec![
            json!({"command":"document.open_smiles","params":{"path":input}}),
            json!({"command":"smiles.record.inspect","params":smiles_record_support::params(1,2)}),
        ];
        if stage > 0 {
            requests.push(json!({"command":"document.save_smiles","params":{"path":output}}));
        }
        if stage == 2 {
            requests.push(json!({"command":"document.open_smiles","params":{"path":output}}));
        }
        let revision = if stage == 2 { 2 } else { 1 };
        requests.push(json!({"command":"smiles.record.inspect","params":smiles_record_support::params(revision,2)}));
        for request in requests {
            process.arg("--cmd").arg(request.to_string());
        }
        let result = process.output()?;
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let value: Value = serde_json::from_slice(&result.stdout)?;
        smiles_record_support::assert_query(&value, revision, revision, 2)?;
        smiles_record_support::consume_notation(&value)?;
        if stage > 0 {
            assert_eq!(std::fs::read(&output)?, smiles_record_support::SOURCE);
        }
    }
    let result = Command::new(env!("CARGO_BIN_EXE_bondline-cli"))
        .arg("run")
        .arg("--cmd")
        .arg(json!({"command":"document.open_smiles","params":{"path":input}}).to_string())
        .arg("--cmd")
        .arg(
            json!({"command":"smiles.record.inspect","params":smiles_record_support::params(1,2)})
                .to_string(),
        )
        .arg("--cmd")
        .arg(json!({"command":"document.inspect","params":{}}).to_string())
        .output()?;
    assert!(result.status.success());
    let value: Value = serde_json::from_slice(&result.stdout)?;
    smiles_record_support::assert_summary(&value, 1);
    assert!(value.get("smiles_record").is_none());
    assert!(!value.to_string().contains("[13CH2]"));
    assert_eq!(std::fs::read(&input)?, smiles_record_support::SOURCE);
    Ok(())
}

#[test]
fn executable_smiles_record_invalid_requests_refuse_without_changing_saved_source()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = file_support::FileTestDirectory::new()?;
    let input = directory.path("cli-record-guards.smi");
    std::fs::write(&input, smiles_record_support::SOURCE)?;
    for (case, (params, code)) in [
        (
            json!({"expected_revision":0,"record_id":2}),
            "stale_revision",
        ),
        (json!({"expected_revision":1,"record_id":4}), "not_found"),
        (
            json!({"expected_revision":1,"record_id":0}),
            "invalid_params",
        ),
        (
            json!({"expected_revision":1,"record_id":2,"field_offset":0}),
            "invalid_params",
        ),
        (json!({"expected_revision":1}), "invalid_params"),
    ]
    .into_iter()
    .enumerate()
    {
        let output = directory.path(&format!("cli-record-guard-{case}.data"));
        let result = Command::new(env!("CARGO_BIN_EXE_bondline-cli"))
            .arg("run")
            .arg("--cmd")
            .arg(json!({"command":"document.open_smiles","params":{"path":input}}).to_string())
            .arg("--cmd")
            .arg(json!({"command":"document.save_smiles","params":{"path":output}}).to_string())
            .arg("--cmd")
            .arg(json!({"command":"smiles.record.inspect","params":params}).to_string())
            .output()?;
        assert!(!result.status.success());
        let value: Value = serde_json::from_slice(&result.stderr)?;
        assert_eq!(value["error"]["code"], code);
        smiles_record_support::assert_private(&value);
        assert_eq!(std::fs::read(&output)?, smiles_record_support::SOURCE);
        assert_eq!(std::fs::read(&input)?, smiles_record_support::SOURCE);
    }
    let result = Command::new(env!("CARGO_BIN_EXE_bondline-cli"))
        .arg("run")
        .arg("--cmd")
        .arg(
            json!({"command":"smiles.record.inspect","params":smiles_record_support::params(0,2)})
                .to_string(),
        )
        .output()?;
    assert!(!result.status.success());
    let value: Value = serde_json::from_slice(&result.stderr)?;
    assert_eq!(value["error"]["code"], "unsupported_content");
    Ok(())
}

#[test]
fn executable_smiles_file_open_save_reopen_retains_all_records_and_opaque_data()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = file_support::FileTestDirectory::new()?;
    let input = directory.path("cli-smiles-λ.smi");
    let output = directory.path("cli-smiles-copy.data");
    std::fs::write(&input, smiles_file_support::SOURCE)?;
    let mut process = Command::new(env!("CARGO_BIN_EXE_bondline-cli"));
    process.arg("run");
    for request in [
        json!({"command":"document.open_smiles","params":{"path":input}}),
        json!({"command":"document.save_smiles","params":{"path":output}}),
        json!({"command":"document.open_smiles","params":{"path":output}}),
    ] {
        process.arg("--cmd").arg(request.to_string());
    }
    let result = process.output()?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let value: Value = serde_json::from_slice(&result.stdout)?;
    smiles_file_support::assert_file(&value, 2);
    assert_eq!(std::fs::read(&output)?, smiles_file_support::SOURCE);
    assert_eq!(std::fs::read(&input)?, smiles_file_support::SOURCE);
    assert_eq!(
        value["file"]["path"],
        json!(std::fs::canonicalize(&output)?)
    );
    Ok(())
}

#[test]
fn executable_exports_only_the_selected_native_fragment_without_saving_the_document()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = file_support::FileTestDirectory::new()?;
    let input = directory.path("cli-export-native.cdxml");
    let output = directory.path("cli-export-λ.sdf");
    std::fs::write(&input, smiles_export_support::SOURCE)?;
    let result=Command::new(env!("CARGO_BIN_EXE_bondline-cli")).arg("run")
        .arg("--cmd").arg(json!({"command":"document.open_cdxml","params":{"path":input}}).to_string())
        .arg("--cmd").arg(json!({"command":"file.export_smiles_fragment","params":smiles_export_support::params(1,&output)}).to_string()).output()?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let value: Value = serde_json::from_slice(&result.stdout)?;
    smiles_export_support::assert_export(&value, &output)?;
    assert_eq!(value["file"]["path"], json!(std::fs::canonicalize(&input)?));
    assert_eq!(std::fs::read(&input)?, smiles_export_support::SOURCE);
    smiles_export_support::consume(&output)?;
    Ok(())
}

#[test]
fn executable_conversion_then_open_extend_history_save_and_reopen_use_the_same_registry()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = file_support::FileTestDirectory::new()?;
    let input = directory.path("cli-conversion.smi");
    std::fs::write(&input, b"CC\n")?;
    for workflow in [false, true] {
        let output = directory.path(if workflow {
            "cli-conversion-workflow-λ.cdxml"
        } else {
            "cli-conversion-only-λ.cdxml"
        });
        let mut process = Command::new(env!("CARGO_BIN_EXE_bondline-cli"));
        process.arg("run");
        let mut requests = vec![
            json!({"command":"file.convert_smiles_to_cdxml","params":conversion_support::params(0,&input,&output)}),
        ];
        if workflow {
            requests.extend([
                json!({"command":"document.open_cdxml","params":{"path":output}}),
                json!({"command":"bond.extend","params":conversion_support::extend(1)}),
                json!({"command":"history.undo","params":{}}),
                json!({"command":"history.redo","params":{}}),
                json!({"command":"document.save_cdxml","params":{"path":output,"overwrite":true}}),
                json!({"command":"document.open_cdxml","params":{"path":output}}),
            ]);
        }
        for request in requests {
            process.arg("--cmd").arg(request.to_string());
        }
        let result = process.output()?;
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let value: Value = serde_json::from_slice(&result.stdout)?;
        if workflow {
            conversion_support::assert_chain(&value, 3)?;
            assert_eq!(value["revision"], 5);
            assert_eq!(value["undo_depth"], 0);
            assert_eq!(value["redo_depth"], 0);
            assert_eq!(value["file"]["dirty"], false);
            assert_eq!(value["source"]["modified"], false);
            assert!(value.get("conversion").is_none());
        } else {
            conversion_support::assert_conversion(&value, &input, &output)?;
        }
        conversion_support::independent(&output, if workflow { 3 } else { 2 })?;
    }
    Ok(())
}

#[test]
fn executable_field_edit_save_undo_and_redo_preserve_complete_sdf_source_and_baselines()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = sdf_support::SdfTestDirectory::new()?;
    for mixed in [false, true] {
        let input = directory.path(if mixed {
            "cli-set-mixed.sdf"
        } else {
            "cli-set-ordered.sdf"
        });
        std::fs::write(&input, sdf_field_support::source(mixed))?;
        for stage in 0..3 {
            let output = directory.path(&format!("cli-set-{mixed}-{stage}-λ.data"));
            let mut process = Command::new(env!("CARGO_BIN_EXE_bondline-cli"));
            process.arg("run");
            let mut requests = vec![
                json!({"command":"document.open_sdf","params":{"path":input}}),
                json!({"command":"sdf.record.inspect","params":{"expected_revision":1,"record_id":1}}),
                json!({"command":"sdf.field.set_value","params":{"expected_revision":1,"record_id":1,"field_index":sdf_field_support::field_index(mixed),"value_lines":["revised","multiline"]}}),
                json!({"command":"sdf.record.inspect","params":{"expected_revision":2,"record_id":1}}),
                json!({"command":"document.save_sdf","params":{"path":output}}),
            ];
            if stage >= 1 {
                requests.push(json!({"command":"history.undo","params":{}}));
            }
            if stage >= 2 {
                requests.push(json!({"command":"history.redo","params":{}}));
            }
            requests.push(json!({"command":"sdf.record.inspect","params":{"expected_revision":2+stage,"record_id":1}}));
            for request in requests {
                process.arg("--cmd").arg(request.to_string());
            }
            let result = process.output()?;
            assert!(
                result.status.success(),
                "{}",
                String::from_utf8_lossy(&result.stderr)
            );
            let result: Value = serde_json::from_slice(&result.stdout)?;
            assert_eq!(result["revision"], 2 + stage);
            assert_eq!(result["file"]["dirty"], stage == 1);
            assert_eq!(
                result["source"]["byte_length"],
                sdf_field_support::source(mixed).len()
            );
            assert_eq!(result["source"]["modified"], stage != 1);
            assert_eq!(
                result["collection"]["byte_length"],
                if stage == 1 {
                    sdf_field_support::source(mixed).len()
                } else {
                    sdf_field_support::edited_bytes(mixed).len()
                }
            );
            let index = sdf_field_support::field_index(mixed);
            assert_eq!(
                result["sdf_record"]["fields"][index]["value_lines"],
                if stage == 1 {
                    if mixed {
                        json!(["second"])
                    } else {
                        json!(["two"])
                    }
                } else {
                    json!(["revised", "multiline"])
                }
            );
            assert_eq!(
                result["sdf_record"]["fields"][index]["omitted_reason"],
                Value::Null
            );
            sdf_field_support::assert_edited_file(&output, mixed)?;
        }
        assert_eq!(std::fs::read(&input)?, sdf_field_support::source(mixed));
    }
    Ok(())
}
#[path = "../../../xtask/oracles/sdf_test_support.rs"]
mod sdf_support;

#[test]
fn executable_explicit_sdf_record_queries_keep_duplicates_dt_text_and_automatic_state_private()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = sdf_support::SdfTestDirectory::new()?;
    for (source, mixed) in [
        (
            include_bytes!("../../io/tests/fixtures/sdf/ordered-fields-v2000.sdf").as_slice(),
            false,
        ),
        (
            include_bytes!("../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf").as_slice(),
            true,
        ),
    ] {
        let path = directory.path(if mixed {
            "query-mixed.sdf"
        } else {
            "query-ordered.sdf"
        });
        std::fs::write(&path, source)?;
        for record_id in [1, 2] {
            let output=Command::new(env!("CARGO_BIN_EXE_bondline-cli")).arg("run")
                .arg("--cmd").arg(json!({"command":"document.open_sdf","params":{"path":path}}).to_string())
                .arg("--cmd").arg(json!({"command":"sdf.record.inspect","params":{"expected_revision":1,"record_id":record_id}}).to_string()).output()?;
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            let response: Value = serde_json::from_slice(&output.stdout)?;
            assert_eq!(
                response["sdf_record"]["fields"],
                sdf_record_support::expected_fields(mixed, record_id)
            );
            assert_eq!(response["sdf_record"]["input_revision"], 1);
            assert_eq!(response["sdf_record"]["session_generation"], 1);
            assert_eq!(response["revision"], 1);
            assert_eq!(response["changed"], false);
            assert_eq!(response["file"]["dirty"], false);
            assert_eq!(response["undo_depth"], 0);
            assert_eq!(response["redo_depth"], 0);
            sdf_record_support::assert_json_budget(&response["sdf_record"], None)?;
        }
        let output=Command::new(env!("CARGO_BIN_EXE_bondline-cli")).arg("run")
            .arg("--cmd").arg(json!({"command":"document.open_sdf","params":{"path":path}}).to_string())
            .arg("--cmd").arg(json!({"command":"sdf.record.inspect","params":{"expected_revision":1,"record_id":1}}).to_string())
            .arg("--cmd").arg(json!({"command":"document.inspect","params":{}}).to_string()).output()?;
        assert!(output.status.success());
        let response: Value = serde_json::from_slice(&output.stdout)?;
        assert!(response.get("sdf_record").is_none());
        assert_eq!(response["collection"], sdf_support::expected_summary(mixed));
        sdf_support::independently_read_sdf(&path, source, mixed)?;
    }
    Ok(())
}

#[test]
fn executable_extends_saves_undoes_reopens_and_extends_the_same_native_chain()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = file_support::FileTestDirectory::new()?;
    let path = directory.path("cli-propane.cdxml");
    let mut process = Command::new(env!("CARGO_BIN_EXE_bondline-cli"));
    process.arg("run");
    for request in [
        json!({"command":"bond.create","params":chain_support::create()}),
        json!({"command":"bond.extend","params":chain_support::extend(1,2,180.0,240.0)}),
        json!({"command":"document.save_cdxml","params":{"path":path}}),
        json!({"command":"history.undo","params":{}}),
        json!({"command":"history.redo","params":{}}),
    ] {
        process.arg("--cmd").arg(request.to_string());
    }
    let output = process.output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(
        (result["revision"].as_u64(), result["undo_depth"].as_u64()),
        (Some(4), Some(2))
    );
    assert_eq!(result["file"]["dirty"], false);
    assert_eq!(
        result["document"]["pages"][0]["fragments"]
            .as_array()
            .ok_or("fragments")?
            .len(),
        1
    );
    chain_support::independently_read(&path, 3)?;
    let mut process = Command::new(env!("CARGO_BIN_EXE_bondline-cli"));
    process.arg("run");
    for request in [
        json!({"command":"document.open_cdxml","params":{"path":path}}),
        json!({"command":"bond.extend","params":chain_support::extend(1,3,220.0,280.0)}),
        json!({"command":"document.save_cdxml","params":{"path":path,"overwrite":true}}),
    ] {
        process.arg("--cmd").arg(request.to_string());
    }
    let output = process.output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(result["source"]["format"], "cdxml");
    assert_eq!(result["source"]["modified"], true);
    assert_eq!(result["file"]["dirty"], false);
    chain_support::independently_read(&path, 4)?;
    Ok(())
}

#[test]
fn executable_lists_schemas_and_runs_a_stateful_command_sequence()
-> Result<(), Box<dyn std::error::Error>> {
    let binary = env!("CARGO_BIN_EXE_bondline-cli");
    let list = Command::new(binary).arg("commands").output()?;
    assert!(list.status.success());
    let commands: Value = serde_json::from_slice(&list.stdout)?;
    let commands = commands.as_array().ok_or("expected command array")?;
    let mut names = commands
        .iter()
        .map(|info| info["name"].as_str().ok_or("command name"))
        .collect::<Result<Vec<_>, _>>()?;
    names.sort_unstable();
    assert_eq!(
        names,
        [
            "atom.create_authored_label",
            "atom.set_authored_label",
            "atom.set_label",
            "bond.create",
            "bond.extend",
            "bond.set_order",
            "chain.create",
            "chain.create_path",
            "document.close",
            "document.import_smiles_records",
            "document.inspect",
            "document.new",
            "document.open_cdx",
            "document.open_cdxml",
            "document.open_mol",
            "document.open_sdf",
            "document.open_smiles",
            "document.save_cdx",
            "document.save_cdxml",
            "document.save_mol",
            "document.save_sdf",
            "document.save_smiles",
            "document.set_document_settings",
            "document.set_drawing_settings",
            "file.convert_smiles_to_cdxml",
            "file.export_smiles_fragment",
            "history.redo",
            "history.undo",
            "page.set_size",
            "render.capture",
            "ring.create",
            "ring.sprout",
            "sdf.field.set_value",
            "sdf.record.inspect",
            "sdf.record.remove",
            "selection.translate",
            "smiles.record.inspect",
            "structure.paste_smiles",
            "text.set_run",
            "ui.atom_label",
            "ui.bond_tool",
            "ui.chain_tool",
            "ui.controls",
            "ui.drawing_settings",
            "ui.file",
            "ui.preferences",
            "ui.ring_tool",
            "ui.selection",
            "ui.view"
        ]
    );
    let query = commands
        .iter()
        .find(|info| info["name"] == "smiles.record.inspect")
        .ok_or("SMILES record query")?;
    assert_eq!(query["read_only"], true);
    assert_eq!(query["input_schema"]["additionalProperties"], false);
    assert_eq!(
        query["input_schema"]["required"],
        json!(["expected_revision", "record_id"])
    );
    let properties = query["input_schema"]["properties"]
        .as_object()
        .ok_or("query properties")?;
    assert_eq!(
        properties
            .keys()
            .map(String::as_str)
            .collect::<std::collections::BTreeSet<_>>(),
        std::collections::BTreeSet::from(["expected_revision", "record_id"])
    );
    assert_eq!(properties["record_id"]["minimum"], 1);
    for (name, permission) in [
        ("document.open_cdx", "discard_unsaved"),
        ("document.save_cdx", "overwrite"),
        ("document.open_sdf", "discard_unsaved"),
        ("document.save_sdf", "overwrite"),
        ("document.open_smiles", "discard_unsaved"),
        ("document.save_smiles", "overwrite"),
    ] {
        let info = commands
            .iter()
            .find(|info| info["name"] == name)
            .ok_or("CDX command")?;
        assert_eq!(info["read_only"], false);
        let schema = &info["input_schema"];
        assert_eq!(schema["additionalProperties"], false);
        assert_eq!(schema["required"], json!(["path"]));
        let properties = schema["properties"].as_object().ok_or("CDX properties")?;
        assert_eq!(
            properties
                .keys()
                .map(String::as_str)
                .collect::<std::collections::BTreeSet<_>>(),
            std::collections::BTreeSet::from(["path", permission])
        );
        assert_eq!(properties[permission]["type"], "boolean");
        assert_eq!(properties[permission]["default"], false);
    }
    assert!(
        commands
            .iter()
            .any(|command| command["name"] == "page.set_size"
                && command["input_schema"]["properties"]["width_pt"]["exclusiveMinimum"] == 0.0)
    );
    for name in ["document.open_mol", "document.save_mol"] {
        let command = commands
            .iter()
            .find(|info| info["name"] == name)
            .ok_or("MOL command")?;
        assert_eq!(command["read_only"], false);
        let schema = &command["input_schema"];
        assert_eq!(schema["additionalProperties"], false);
        assert_eq!(schema["required"], json!(["path"]));
        let properties = schema["properties"].as_object().ok_or("MOL properties")?;
        let keys: std::collections::BTreeSet<_> = properties.keys().map(String::as_str).collect();
        if name == "document.open_mol" {
            assert_eq!(
                keys,
                std::collections::BTreeSet::from([
                    "path",
                    "discard_unsaved",
                    "points_per_unit",
                    "origin_x_pt",
                    "origin_y_pt"
                ])
            );
            assert_eq!(properties["discard_unsaved"]["default"], false);
            for (key, default) in [
                ("points_per_unit", 24.0),
                ("origin_x_pt", 72.0),
                ("origin_y_pt", 72.0),
            ] {
                assert_eq!(properties[key]["type"], "number");
                assert_eq!(properties[key]["default"].as_f64(), Some(default));
            }
        } else {
            assert_eq!(
                keys,
                std::collections::BTreeSet::from(["path", "overwrite", "version"])
            );
            assert_eq!(properties["overwrite"]["default"], false);
            assert_eq!(properties["version"]["default"], "auto");
            assert_eq!(
                schema["$defs"]["MolVersionChoice"]["enum"],
                json!(["auto", "v2000", "v3000"])
            );
        }
    }
    let result = Command::new(binary)
        .args([
            "run",
            "--cmd",
            r#"{"command":"page.set_size","params":{"page_id":1,"width_pt":612,"height_pt":792}}"#,
            "--cmd",
            r#"{"command":"history.undo","params":{}}"#,
            "--cmd",
            r#"{"command":"history.redo","params":{}}"#,
        ])
        .output()?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let state: Value = serde_json::from_slice(&result.stdout)?;
    assert_eq!(state["revision"], 3);
    assert_eq!(
        state["file"],
        json!({"path":null,"dirty":true,"format":"cdxml"})
    );
    assert_eq!(
        state["document"]["pages"][0],
        json!({"id":1,"size":{"width_pt":612.0,"height_pt":792.0},"fragments":[]})
    );
    Ok(())
}

#[test]
fn executable_opens_inspects_saves_and_reopens_both_complete_sdf_encodings()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = sdf_support::SdfTestDirectory::new()?;
    for (source, mixed) in [
        (
            include_bytes!("../../io/tests/fixtures/sdf/ordered-fields-v2000.sdf").as_slice(),
            false,
        ),
        (
            include_bytes!("../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf").as_slice(),
            true,
        ),
    ] {
        let input = directory.path(if mixed {
            "cli-mixed.sdf"
        } else {
            "cli-ordered.sd"
        });
        let output = directory.path(if mixed {
            "cli-mixed-λ.data"
        } else {
            "cli-ordered-λ.data"
        });
        std::fs::write(&input, source)?;
        let mut process = Command::new(env!("CARGO_BIN_EXE_bondline-cli"));
        process.arg("run");
        for request in [
            json!({"command":"document.open_sdf","params":{"path":input}}),
            json!({"command":"document.inspect","params":{}}),
            json!({"command":"document.save_sdf","params":{"path":output}}),
            json!({"command":"document.open_sdf","params":{"path":output}}),
        ] {
            process.arg("--cmd").arg(request.to_string());
        }
        let result = process.output()?;
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let result: Value = serde_json::from_slice(&result.stdout)?;
        assert_eq!(result["document"], json!(null));
        assert_eq!(result["collection"], sdf_support::expected_summary(mixed));
        assert_eq!(
            result["source"],
            json!({"format":"sdf","byte_length":source.len(),"modified":false})
        );
        assert_eq!(result["file"]["dirty"], false);
        assert_eq!(result["file"]["format"], "sdf");
        assert_eq!(result["revision"], 2);
        assert_eq!(result["undo_depth"], 0);
        sdf_support::independently_read_sdf(&output, source, mixed)?;
        let bad=Command::new(env!("CARGO_BIN_EXE_bondline-cli")).arg("run")
            .arg("--cmd").arg(json!({"command":"document.open_sdf","params":{"path":input}}).to_string())
            .arg("--cmd").arg(json!({"command":"document.save_cdxml","params":{"path":directory.path("must-not-publish.cdxml")}}).to_string()).output()?;
        assert_eq!(bad.status.code(), Some(2));
        assert_eq!(
            serde_json::from_slice::<Value>(&bad.stderr)?["error"]["code"],
            "unsupported_content"
        );
        assert!(!directory.path("must-not-publish.cdxml").exists());
    }
    Ok(())
}

#[test]
fn executable_cdx_open_save_uses_complete_source_and_refuses_lossy_exports()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = file_support::FileTestDirectory::new()?;
    let input = directory.path("input-λ.mol");
    let copy = directory.path("copy.cdx");
    std::fs::write(&input, cdx_support::SOURCE)?;
    let open = json!({"command":"document.open_cdx","params":{"path":input}}).to_string();
    let save = json!({"command":"document.save_cdx","params":{"path":copy}}).to_string();
    let output = Command::new(env!("CARGO_BIN_EXE_bondline-cli"))
        .args(["run", "--cmd", &open, "--cmd", &save])
        .output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: Value = serde_json::from_slice(&output.stdout)?;
    cdx_support::assert_state(&result)?;
    assert_eq!(result["revision"], 1);
    assert_eq!(result["file_operation"]["format"], "cdx");
    assert_eq!(result["file_operation"]["operation"], "save");
    cdx_support::independently_read_saved(&copy)?;
    for (command, params, code) in [
        ("document.save_cdx", json!({"path":copy}), "already_exists"),
        (
            "document.save_cdxml",
            json!({"path":copy,"overwrite":true}),
            "unsupported_content",
        ),
        (
            "document.save_mol",
            json!({"path":copy,"overwrite":true}),
            "unsupported_content",
        ),
        (
            "document.open_cdx",
            json!({"path":input,"version":"auto"}),
            "invalid_params",
        ),
    ] {
        let request = json!({"command":command,"params":params}).to_string();
        let output = Command::new(env!("CARGO_BIN_EXE_bondline-cli"))
            .args(["run", "--cmd", &open, "--cmd", &request])
            .output()?;
        assert!(!output.status.success());
        let error: Value = serde_json::from_slice(&output.stderr)?;
        assert_eq!(error["error"]["code"], code);
        assert_eq!(std::fs::read(&copy)?, cdx_support::SOURCE);
    }
    Ok(())
}

#[test]
fn executable_returns_a_png_capture_from_the_same_authored_revision()
-> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_bondline-cli"))
        .args([
            "run", "--cmd",
            r#"{"command":"page.set_size","params":{"page_id":1,"width_pt":72,"height_pt":144}}"#,
            "--cmd",
            r#"{"command":"render.capture","params":{"page_id":1,"width_px":8,"height_px":8,"backend":"cpu"}}"#,
        ]).output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(result["revision"], 1);
    assert_eq!(result["changed"], false);
    assert_eq!(result["undo_depth"], 1);
    assert_eq!(result["capture"]["width_px"], 8);
    assert_eq!(result["capture"]["height_px"], 8);
    assert_eq!(result["capture"]["backend"], "cpu");
    assert_eq!(result["capture"]["adapter"], Value::Null);
    assert!(
        result["capture"]["png_base64"]
            .as_str()
            .ok_or("missing PNG")?
            .starts_with("iVBORw0KGgo")
    );
    Ok(())
}

#[test]
fn executable_reports_consistent_errors_without_a_success_payload()
-> Result<(), Box<dyn std::error::Error>> {
    let binary = env!("CARGO_BIN_EXE_bondline-cli");
    for (arguments, code) in [
        (
            vec!["run", "--cmd", r#"{"command":"history.undo","params":{}}"#],
            "empty_history",
        ),
        (
            vec![
                "run",
                "--cmd",
                r#"{"command":"page.set_size","params":{"page_id":1,"width_pt":-1,"height_pt":72}}"#,
            ],
            "invalid_params",
        ),
        (
            vec!["run", "--cmd", r#"{"command":"missing","params":{}}"#],
            "unknown_command",
        ),
        (
            vec![
                "run",
                "--cmd",
                r#"{"command":"document.open_mol","params":{"path":"unused.mol","unexpected":true}}"#,
            ],
            "invalid_params",
        ),
        (
            vec![
                "run",
                "--cmd",
                r#"{"command":"document.save_mol","params":{"path":"unused.mol","format":"v3000"}}"#,
            ],
            "invalid_params",
        ),
        (
            vec![
                "run",
                "--cmd",
                r#"{"command":"document.open_mol","params":{"path":"unused.mol","points_per_unit":0}}"#,
            ],
            "invalid_params",
        ),
        (
            vec![
                "run",
                "--cmd",
                r#"{"command":"document.save_mol","params":{"path":"unused.mol","version":"v4000"}}"#,
            ],
            "invalid_params",
        ),
        (
            vec![
                "run",
                "--cmd",
                r#"{"command":"document.save_mol","params":{"path":"unused.mol","version":null}}"#,
            ],
            "invalid_params",
        ),
        (vec!["commands", "extra"], "usage"),
        (vec!["run"], "usage"),
    ] {
        let result = Command::new(binary).args(arguments).output()?;
        assert_eq!(result.status.code(), Some(2));
        assert!(result.stdout.is_empty());
        let error: Value = serde_json::from_slice(&result.stderr)?;
        assert_eq!(error["error"]["code"], code);
    }
    assert!(
        Command::new(binary)
            .arg("--help")
            .output()?
            .status
            .success()
    );
    Ok(())
}

#[test]
fn executable_creates_and_captures_a_labeled_single_bond() -> Result<(), Box<dyn std::error::Error>>
{
    let result=Command::new(env!("CARGO_BIN_EXE_bondline-cli")).args([
        "run","--cmd",r#"{"command":"page.set_size","params":{"page_id":1,"width_pt":60,"height_pt":60}}"#,
        "--cmd",r#"{"command":"bond.create","params":{"page_id":1,"begin_x_pt":10,"begin_y_pt":20.5,"end_x_pt":50,"end_y_pt":20.5}}"#,
        "--cmd",r#"{"command":"render.capture","params":{"page_id":1,"width_px":80,"height_px":80,"backend":"cpu"}}"#,
    ]).output()?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let state: Value = serde_json::from_slice(&result.stdout)?;
    assert_eq!(state["revision"], 2);
    assert_eq!(state["undo_depth"], 2);
    let fragment = &state["document"]["pages"][0]["fragments"][0];
    assert_eq!(fragment["atoms"].as_array().ok_or("atoms")?.len(), 2);
    assert_eq!(fragment["atoms"][0]["explicit_hydrogens"], 3);
    assert_eq!(fragment["bonds"][0]["order"], "single");
    assert_eq!(state["capture"]["backend"], "cpu");
    assert!(
        state["capture"]["png_base64"]
            .as_str()
            .ok_or("png")?
            .starts_with("iVBORw0KGgo")
    );
    Ok(())
}

#[test]
fn executable_saves_reopens_and_edits_real_cdxml_across_processes()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = file_support::FileTestDirectory::new()?;
    let path = directory.path("drawing.cdxml");
    let binary = env!("CARGO_BIN_EXE_bondline-cli");
    let draw = json!({"command":"bond.create","params":{"page_id":1,"begin_x_pt":100.0,"begin_y_pt":200.0,"end_x_pt":140.0,"end_y_pt":200.0}}).to_string();
    let save = json!({"command":"document.save_cdxml","params":{"path":path}}).to_string();
    let output = Command::new(binary)
        .args(["run", "--cmd", &draw, "--cmd", &save])
        .output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let saved: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(saved["revision"], 1);
    assert_eq!(
        saved["file"],
        json!({"path":std::fs::canonicalize(&path)?,"dirty":false,"format":"cdxml"})
    );
    file_support::independently_read_file(&path, 1)?;
    let open = json!({"command":"document.open_cdxml","params":{"path":path}}).to_string();
    let draw_more = json!({"command":"bond.create","params":{"page_id":1,"begin_x_pt":100.0,"begin_y_pt":300.0,"end_x_pt":140.0,"end_y_pt":300.0}}).to_string();
    let overwrite =
        json!({"command":"document.save_cdxml","params":{"path":path,"overwrite":true}})
            .to_string();
    let output = Command::new(binary)
        .args([
            "run", "--cmd", &open, "--cmd", &draw_more, "--cmd", &overwrite,
        ])
        .output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let edited: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(edited["revision"], 2);
    assert_eq!(edited["undo_depth"], 1);
    assert_eq!(edited["file"]["dirty"], false);
    file_support::independently_read_file(&path, 2)?;
    let output = Command::new(binary)
        .args(["run", "--cmd", &open])
        .output()?;
    assert!(output.status.success());
    let reopened: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(reopened["document"], edited["document"]);
    assert_eq!(reopened["undo_depth"], 0);
    Ok(())
}

#[test]
fn executable_opens_and_saves_real_mol_with_source_and_headers_intact()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = file_support::FileTestDirectory::new()?;
    let source = directory.path("methyl-source.mol");
    let destination = directory.path("methyl-copy.mol");
    let bytes = mol_support::METHYL;
    std::fs::write(&source, bytes)?;
    let output = Command::new(env!("CARGO_BIN_EXE_bondline-cli"))
        .args([
            "run",
            "--cmd",
            &json!({"command":"document.open_mol","params":{"path":source}}).to_string(),
            "--cmd",
            &json!({"command":"document.save_mol","params":{"path":destination}}).to_string(),
        ])
        .output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(result["revision"], 1);
    assert_eq!(result["undo_depth"], 0);
    assert_eq!(result["redo_depth"], 0);
    assert_eq!(
        result["file"],
        json!({
            "path":std::fs::canonicalize(&destination)?,"dirty":false,"format":"mol_v2000"
        })
    );
    mol_support::assert_methyl_state(&result, 42.3, 20.3784)?;
    assert_eq!(result["file_operation"]["operation"], "save");
    assert_eq!(result["file_operation"]["input_revision"], 1);
    assert_eq!(std::fs::read(&destination)?, bytes);
    let external = mol_support::independently_read_mol(&destination)?;
    assert_eq!(
        external["source_sha256"],
        "afd2352697ca20c0d580c4981fe8bd24da36c4e1661e9d2aee501a40364fd42d"
    );
    assert_eq!(external["smiles"], "[CH3]");
    assert_eq!(external["formula"], "CH3");
    assert_eq!(external["atoms"][0]["hydrogens"], 3);
    assert_eq!(external["atoms"][0]["radical_electrons"], 1);
    Ok(())
}

#[test]
fn executable_exports_native_cc_to_mol_and_preserves_association_through_history()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = file_support::FileTestDirectory::new()?;
    let path = directory.path("native-ethane.mol");
    let output = Command::new(env!("CARGO_BIN_EXE_bondline-cli")).args([
        "run", "--cmd", r#"{"command":"bond.create","params":{"page_id":1,"begin_x_pt":72,"begin_y_pt":72,"end_x_pt":108,"end_y_pt":72}}"#,
        "--cmd", &json!({"command":"document.save_mol","params":{"path":path}}).to_string(),
        "--cmd", r#"{"command":"history.undo","params":{}}"#,
        "--cmd", r#"{"command":"history.redo","params":{}}"#,
    ]).output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(result["revision"], 3);
    assert_eq!(result["undo_depth"], 1);
    assert_eq!(result["redo_depth"], 0);
    assert_eq!(
        result["file"],
        json!({"path":std::fs::canonicalize(&path)?,"dirty":false,"format":"mol_v2000"})
    );
    assert!(result.get("source").is_none());
    let external = mol_support::independently_read_mol(&path)?;
    assert_eq!(external["smiles"], "CC");
    assert_eq!(external["formula"], "C2H6");
    assert_eq!(external["atoms"].as_array().ok_or("atoms")?.len(), 2);
    assert_eq!(external["bonds"], json!([{"begin":0,"end":1,"order":1.0}]));
    for (index, position) in [json!([0.0, 0.0, 0.0]), json!([1.5, 0.0, 0.0])]
        .into_iter()
        .enumerate()
    {
        assert_eq!(external["atoms"][index]["position"], position);
        assert_eq!(external["atoms"][index]["hydrogens"], 3);
        assert_eq!(external["atoms"][index]["radical_electrons"], 0);
    }
    let reopened = Command::new(env!("CARGO_BIN_EXE_bondline-cli"))
        .args([
            "run",
            "--cmd",
            &json!({"command":"document.open_mol","params":{"path":path}}).to_string(),
        ])
        .output()?;
    assert!(
        reopened.status.success(),
        "{}",
        String::from_utf8_lossy(&reopened.stderr)
    );
    let reopened: Value = serde_json::from_slice(&reopened.stdout)?;
    assert_eq!(reopened["document"], result["document"]);
    assert_eq!(reopened["file"], result["file"]);
    assert_eq!(reopened["undo_depth"], 0);
    Ok(())
}

#[test]
fn executable_exports_explicit_v3000_at_full_precision_and_auto_keeps_its_version()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = file_support::FileTestDirectory::new()?;
    let path = directory.path("explicit-v3.v2000");
    let copy = directory.path("auto-v3.copy");
    let output = Command::new(env!("CARGO_BIN_EXE_bondline-cli")).args([
        "run", "--cmd", r#"{"command":"bond.create","params":{"page_id":1,"begin_x_pt":100,"begin_y_pt":200,"end_x_pt":140,"end_y_pt":200}}"#,
        "--cmd", &json!({"command":"document.save_mol","params":{"path":path,"version":"v3000"}}).to_string(),
        "--cmd", &json!({"command":"document.save_mol","params":{"path":copy}}).to_string(),
    ]).output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(result["revision"], 1);
    assert_eq!(result["undo_depth"], 1);
    assert_eq!(result["changed"], false);
    assert_eq!(
        result["file"],
        json!({"path":std::fs::canonicalize(&copy)?,"dirty":false,"format":"mol_v3000"})
    );
    assert_eq!(result["file_operation"]["format"], "mol_v3000");
    assert_eq!(result["file_operation"]["input_revision"], 1);
    assert!(result.get("source").is_none());
    assert_eq!(std::fs::read(&path)?, std::fs::read(&copy)?);
    let external = mol_support::independently_read_v3000(&copy)?;
    assert_eq!(external["rdkit"], "2026.09.2");
    assert_eq!(external["atoms"].as_array().ok_or("atoms")?.len(), 2);
    assert_eq!(
        external["bonds"],
        json!([{"begin":0,"end":1,"order":1.0,"direction":"NONE","stereo":"STEREONONE"}])
    );
    for (index, x) in [28.0 / 24.0, 68.0 / 24.0].into_iter().enumerate() {
        let atom = &external["atoms"][index];
        assert_eq!(atom["element"], 6);
        assert_eq!(atom["explicit_h"], 3);
        assert_eq!(atom["radical_electrons"], 0);
        assert!((atom["position"][0].as_f64().ok_or("x")? - x).abs() < 1e-14);
        assert!((atom["position"][1].as_f64().ok_or("y")? + 128.0 / 24.0).abs() < 1e-14);
        assert_eq!(atom["position"][2], 0.0);
    }
    let reopened = Command::new(env!("CARGO_BIN_EXE_bondline-cli"))
        .args([
            "run",
            "--cmd",
            &json!({"command":"document.open_mol","params":{"path":copy}}).to_string(),
        ])
        .output()?;
    assert!(
        reopened.status.success(),
        "{}",
        String::from_utf8_lossy(&reopened.stderr)
    );
    let reopened: Value = serde_json::from_slice(&reopened.stdout)?;
    assert_eq!(reopened["file"], result["file"]);
    assert_eq!(reopened["undo_depth"], 0);
    for (atom, x) in reopened["document"]["pages"][0]["fragments"][0]["atoms"]
        .as_array()
        .ok_or("atoms")?
        .iter()
        .zip([100.0, 140.0])
    {
        assert!((atom["position"]["x_pt"].as_f64().ok_or("x")? - x).abs() < 1e-10);
        assert!((atom["position"]["y_pt"].as_f64().ok_or("y")? - 200.0).abs() < 1e-10);
        assert_eq!(atom["explicit_hydrogens"], 3);
    }
    Ok(())
}

#[test]
fn executable_opens_actual_v3000_and_default_save_preserves_all_original_source_bytes()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = file_support::FileTestDirectory::new()?;
    let source = directory.path("original-v3.v2000");
    let copy = directory.path("original-v3-copy.mol");
    std::fs::write(&source, mol_support::V3000_SOURCE)?;
    let output = Command::new(env!("CARGO_BIN_EXE_bondline-cli"))
        .args([
            "run",
            "--cmd",
            &json!({"command":"document.open_mol","params":{"path":source}}).to_string(),
            "--cmd",
            &json!({"command":"document.save_mol","params":{"path":copy}}).to_string(),
        ])
        .output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: Value = serde_json::from_slice(&output.stdout)?;
    mol_support::assert_v3000_state(&result)?;
    assert_eq!(result["revision"], 1);
    assert_eq!(result["undo_depth"], 0);
    assert_eq!(result["redo_depth"], 0);
    assert_eq!(result["changed"], false);
    assert_eq!(result["file"]["path"], json!(std::fs::canonicalize(&copy)?));
    assert_eq!(result["file_operation"]["format"], "mol_v3000");
    assert_eq!(result["file_operation"]["input_revision"], 1);
    assert_eq!(result["file_operation"]["byte_length"], 344);
    assert_eq!(std::fs::read(&copy)?, mol_support::V3000_SOURCE);
    let external = mol_support::independently_read_v3000(&copy)?;
    assert_eq!(
        external["source_sha256"],
        "1867a5904e742030431b523a31167951ff998007a89b9c3d94ec88cf7d4c4380"
    );
    assert_eq!(external["atoms"][0]["position"], json!([-1.25, 0.5, 0.0]));
    assert_eq!(external["atoms"][1]["position"], json!([0.25, 0.5, 0.0]));
    assert_eq!(external["atoms"][0]["explicit_h"], 3);
    assert_eq!(external["atoms"][1]["explicit_h"], 3);
    assert_eq!(external["bonds"][0]["order"], 1.0);
    Ok(())
}
