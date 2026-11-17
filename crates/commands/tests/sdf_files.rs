use bondline_commands::{Command, Engine, registry};
use serde_json::json;
use std::fs;

#[path = "../../../xtask/oracles/sdf_test_support.rs"]
mod support;

type TestResult = Result<(), Box<dyn std::error::Error>>;
const ORDERED: &[u8] = include_bytes!("../../io/tests/fixtures/sdf/ordered-fields-v2000.sdf");
const MIXED: &[u8] = include_bytes!("../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf");

#[test]
fn sdf_registry_parameters_are_typed_closed_and_keep_every_existing_command() -> TestResult {
    let names = registry()
        .into_iter()
        .map(|row| row.name)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        names,
        [
            "atom.set_label",
            "chain.create",
            "chain.create_path",
            "document.inspect",
            "document.import_smiles_records",
            "document.new",
            "document.close",
            "ui.file",
            "ui.controls",
            "ui.atom_label",
            "ui.selection",
            "selection.translate",
            "ui.bond_tool",
            "ui.ring_tool",
            "ui.view",
            "document.open_cdxml",
            "document.save_cdxml",
            "document.open_cdx",
            "document.save_cdx",
            "document.open_mol",
            "document.save_mol",
            "document.open_sdf",
            "document.open_smiles",
            "document.save_sdf",
            "document.save_smiles",
            "document.set_drawing_settings",
            "file.convert_smiles_to_cdxml",
            "file.export_smiles_fragment",
            "page.set_size",
            "bond.create",
            "bond.extend",
            "bond.set_order",
            "history.undo",
            "history.redo",
            "render.capture",
            "ring.create",
            "sdf.field.set_value",
            "sdf.record.remove",
            "smiles.record.inspect",
            "structure.paste_smiles",
            "sdf.record.inspect"
        ]
        .into_iter()
        .collect()
    );
    for name in ["document.open_sdf", "document.save_sdf"] {
        assert!(Command::parse(name, json!({"path":"collection.sdf"})).is_ok());
        for params in [
            json!({}),
            json!({"path":3}),
            json!({"path":"x","unexpected":true}),
            json!({"path":"x","version":"v2000"}),
            json!({"path":"x","points_per_unit":24}),
        ] {
            let error = Command::parse(name, params)
                .err()
                .ok_or("closed SDF params")?;
            assert_eq!(error.info().code, "invalid_params");
        }
        let info = registry()
            .into_iter()
            .find(|row| row.name == name)
            .ok_or("SDF schema")?;
        assert!(!info.read_only);
        assert_eq!(info.input_schema.as_value()["additionalProperties"], false);
        assert_eq!(info.input_schema.as_value()["required"], json!(["path"]));
    }
    Ok(())
}

#[test]
fn complete_two_record_sdf_sessions_inspect_save_and_reopen_without_a_dummy_drawing() -> TestResult
{
    let directory = support::SdfTestDirectory::new()?;
    for (source, mixed) in [(ORDERED, false), (MIXED, true)] {
        let input = directory.path(if mixed {
            "mixed-input.sdf"
        } else {
            "ordered-input.sd"
        });
        let output = directory.path(if mixed {
            "mixed-saved.data"
        } else {
            "ordered-saved.data"
        });
        fs::write(&input, source)?;
        let mut engine = Engine::default();
        let drawing = serde_json::to_value(engine.state())?;
        assert!(drawing["document"].is_object());
        assert!(drawing.get("collection").is_none());
        let result = engine.dispatch("document.open_sdf", json!({"path":input}))?;
        let wire = serde_json::to_value(&result)?;
        assert_eq!(wire["document"], json!(null));
        assert_eq!(wire["collection"], support::expected_summary(mixed));
        assert_eq!(
            wire["source"],
            json!({"format":"sdf","byte_length":source.len(),"modified":false})
        );
        assert_eq!(
            wire["file"],
            json!({"format":"sdf","path":fs::canonicalize(&input)?,"dirty":false})
        );
        assert_eq!(
            (result.revision, result.undo_depth, result.redo_depth),
            (1, 0, 0)
        );
        assert!(result.changed);
        let inspect = engine.dispatch("document.inspect", json!({}))?;
        assert!(!inspect.changed);
        let inspect = serde_json::to_value(inspect)?;
        assert_eq!(inspect["collection"], wire["collection"]);
        assert_eq!(inspect["document"], json!(null));
        let text = inspect.to_string();
        for private in [
            "numbered",
            "alpha",
            "beta",
            "second record",
            "<DUP>",
            "M  END",
        ] {
            assert!(
                !text.contains(private),
                "automatic state disclosed {private}"
            );
        }
        let saved = engine.dispatch("document.save_sdf", json!({"path":output}))?;
        assert!(!saved.changed);
        assert_eq!(saved.revision, 1);
        assert_eq!(
            serde_json::to_value(&saved)?["file_operation"]["byte_length"],
            source.len()
        );
        support::independently_read_sdf(&output, source, mixed)?;
        engine.dispatch("document.open_sdf", json!({"path":output}))?;
        assert_eq!(
            serde_json::to_value(engine.state())?["collection"],
            support::expected_summary(mixed)
        );
        assert_eq!(fs::read(&input)?, source);
    }
    Ok(())
}

#[test]
fn malformed_later_records_and_encoding_keep_the_prior_whole_drawing_or_collection() -> TestResult {
    let directory = support::SdfTestDirectory::new()?;
    let input = directory.path("good.sdf");
    let bad = directory.path("bad.sdf");
    fs::write(&input, ORDERED)?;
    let native = bondline_io::StoredDocument::from_cdxml(
        include_bytes!("../../io/tests/fixtures/bondline-v0.cdxml")
            .as_slice()
            .into(),
        bondline_model::PageSize::A4,
    )?;
    let mut engine = Engine::from_document(native);
    engine.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":640,"height_pt":792}),
    )?;
    for collection in [false, true] {
        if collection {
            engine.dispatch(
                "document.open_sdf",
                json!({"path":input,"discard_unsaved":true}),
            )?;
        }
        let before = engine.content().clone();
        let state = serde_json::to_value(engine.state())?;
        for (source, code) in [
            (
                [&ORDERED[..424], b"broken second record\n"].concat(),
                "invalid_sdf",
            ),
            ([&ORDERED[..424], b"\xff\n"].concat(), "unsupported_content"),
            ([&ORDERED[..424], b"\0\n"].concat(), "invalid_sdf"),
        ] {
            fs::write(&bad, source)?;
            assert_eq!(
                engine
                    .dispatch(
                        "document.open_sdf",
                        json!({"path":bad,"discard_unsaved":true})
                    )
                    .err()
                    .ok_or("bad later record")?
                    .info()
                    .code,
                code
            );
            assert_eq!(engine.content(), &before);
            assert_eq!(serde_json::to_value(engine.state())?, state);
        }
    }
    Ok(())
}

#[test]
fn valid_but_unprojectable_mol_records_open_and_save_as_complete_raw_sdf() -> TestResult {
    let directory = support::SdfTestDirectory::new()?;
    let input = directory.path("opaque-source.sdf");
    let output = directory.path("opaque-saved.sdf");
    let source = std::str::from_utf8(ORDERED)?.replacen("M  END", "M  XYZ  1   1   7\nM  END", 1);
    fs::write(&input, source.as_bytes())?;
    let mut engine = Engine::default();
    let opened = engine.dispatch("document.open_sdf", json!({"path":input}))?;
    assert!(opened.document.is_none());
    assert_eq!(opened.collection.ok_or("collection")?.record_count, 2);
    let view = engine.content().sdf()?.read()?;
    let transform =
        bondline_io::mol::MolTransform::new(24.0, bondline_model::Point2::new(0.0, 0.0)?)?;
    assert!(
        bondline_io::mol::MolFile::parse(view.records()[0].mol_bytes())?
            .to_fragment(transform)
            .is_err()
    );
    assert_eq!(view.records().len(), 2);
    engine.dispatch("document.save_sdf", json!({"path":output}))?;
    assert_eq!(fs::read(&output)?, source.as_bytes());
    let independent = support::describe_source("sdf_oracle.py", &fs::read(&output)?)?;
    assert_eq!(
        independent["records"]
            .as_array()
            .ok_or("raw records")?
            .len(),
        2
    );
    assert_eq!(independent["records"][0]["rdkit"]["smiles"], "CC");
    assert_eq!(independent["records"][1]["rdkit"]["smiles"], "CC");
    assert_eq!(independent["records"][0]["fields"][3]["dt_number"], "0009");
    Ok(())
}

#[test]
fn a_prepared_sdf_open_cannot_replace_newer_authoring_or_a_committed_close() -> TestResult {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf"
    );
    let mut engine = Engine::default();
    let plan = engine.prepare(Command::parse("document.open_sdf", json!({"path":path}))?)?;
    engine.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":612,"height_pt":792}),
    )?;
    let before = serde_json::to_value(engine.state())?;
    let complete = plan.complete()?;
    assert_eq!(
        engine
            .finish(complete)
            .err()
            .ok_or("stale Open")?
            .info()
            .code,
        "stale_revision"
    );
    assert_eq!(serde_json::to_value(engine.state())?, before);
    let plan = engine.prepare(Command::parse(
        "document.open_sdf",
        json!({"path":path,"discard_unsaved":true}),
    )?)?;
    engine.dispatch(
        "document.close",
        json!({"expected_revision":1,"action":"discard"}),
    )?;
    assert!(engine.complete_close());
    let before = serde_json::to_value(engine.state())?;
    assert_eq!(
        engine
            .finish(plan.complete()?)
            .err()
            .ok_or("committed close")?
            .info()
            .code,
        "close_committed"
    );
    assert_eq!(serde_json::to_value(engine.state())?, before);
    Ok(())
}
