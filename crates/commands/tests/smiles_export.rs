use bondline_commands::Engine;
use bondline_io::StoredDocument;
use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Element, Fragment, FragmentId, PageId, Point2,
};
use serde_json::{Value, json};
use std::{fs, num::NonZeroU16};

#[path = "../../../xtask/oracles/files_test_support.rs"]
pub mod support;
type TestResult = Result<(), Box<dyn std::error::Error>>;
const NAME: &str = "file.export_smiles_fragment";

fn owner() -> Result<StoredDocument, Box<dyn std::error::Error>> {
    let mut methane = Atom::new(
        AtomId::new(40).ok_or("atom")?,
        Element::new(6).ok_or("element")?,
        Some(Point2::new(10.0, 20.0)?),
    );
    methane.explicit_hydrogens = Some(4);
    let mut carbon = Atom::new(
        AtomId::new(7).ok_or("atom")?,
        Element::new(6).ok_or("element")?,
        Some(Point2::new(330.0, 410.0)?),
    );
    carbon.explicit_hydrogens = Some(3);
    carbon.isotope = NonZeroU16::new(13);
    let mut oxygen = Atom::new(
        AtomId::new(80).ok_or("atom")?,
        Element::new(8).ok_or("element")?,
        Some(Point2::new(-10.0, 900.0)?),
    );
    oxygen.explicit_hydrogens = Some(1);
    let first = Fragment::new(FragmentId::new(3).ok_or("fragment")?, vec![methane], vec![])?;
    let selected = Fragment::new(
        FragmentId::new(9).ok_or("fragment")?,
        vec![carbon, oxygen],
        vec![Bond {
            double_position: bondline_model::DoublePosition::Unspecified,
            id: BondId::new(5).ok_or("bond")?,
            begin: AtomId::new(80).ok_or("atom")?,
            end: AtomId::new(7).ok_or("atom")?,
            order: BondOrder::Single,
        }],
    )?;
    let mut stored = StoredDocument::default();
    stored
        .model_mut()
        .append_fragments(PageId::new(1).ok_or("page")?, vec![first, selected])?;
    Ok(stored)
}

fn params(revision: u64, fragment: u32, output: &std::path::Path) -> Value {
    json!({"expected_revision":revision,"page_id":1,"fragment_id":fragment,"output_path":output})
}

fn consume(path: &std::path::Path) -> Result<Value, Box<dyn std::error::Error>> {
    use std::io::Write as _;
    use std::process::{Command, Stdio};
    let file = fs::read(path)?;
    let notation = file.strip_suffix(b"\n").ok_or("required single LF")?;
    assert!(notation.is_ascii() && !notation.is_empty());
    assert!(
        !notation.iter().any(u8::is_ascii_whitespace),
        "one complete unnamed ASCII record only"
    );
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child = Command::new(python)
        .arg("-B")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../io/tests/smiles_write_oracle.py"
        ))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child.stdin.take().ok_or("stdin")?.write_all(notation)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(
        result["notation"]["source_ascii"],
        std::str::from_utf8(notation)?
    );
    assert_eq!(result["notation"]["source_bytes"], notation.len());
    assert_eq!(result["notation"]["rdkit"], "2026.09.2");
    Ok(result)
}

#[test]
fn export_selects_the_exact_nonfirst_fragment_and_preserves_complete_session() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let output = directory.path("selected-λ.sdf");
    let mut engine = Engine::from_document(owner()?);
    let before = serde_json::to_value(engine.state())?;
    let content = engine.content().clone();
    let result = engine.dispatch(
        NAME,
        json!({"expected_revision":0,"page_id":1,"fragment_id":9,"output_path":output}),
    )?;
    assert_eq!(engine.content(), &content);
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert_eq!(fs::read(&output)?, b"[13CH3][OH1]\n");
    let value = serde_json::to_value(result)?;
    assert_eq!(value["changed"], false);
    assert!(value.get("file_operation").is_none());
    assert!(value.get("conversion").is_none());
    let receipt = &value["smiles_export"];
    assert_eq!(receipt["profile"], "neutral_explicit_h_fragment_v1");
    assert_eq!(receipt["scope"], "selected_fragment_chemical_graph");
    assert_eq!(
        receipt["not_encoded"],
        json!(["geometry", "document_context"])
    );
    assert_eq!(receipt["input_revision"], 0);
    assert_eq!(receipt["session_generation"], 0);
    assert_eq!(receipt["page_id"], 1);
    assert_eq!(receipt["fragment_id"], 9);
    assert_eq!(receipt["atom_count"], 2);
    assert_eq!(receipt["bond_count"], 1);
    assert_eq!(receipt["output"]["path"], json!(fs::canonicalize(&output)?));
    assert_eq!(receipt["output"]["byte_length"], b"[13CH3][OH1]\n".len());
    assert!(receipt["output"]["directory_synced"].is_boolean());
    assert_eq!(value["file"], before["file"]);
    assert!(engine.state().source.is_none());
    let external = consume(&output)?;
    assert_eq!(external["notation"]["bonds"], json!([[0, 1, "SINGLE"]]));
    assert_eq!(external["notation"]["atoms"][0]["isotope"], 13);
    assert_eq!(external["notation"]["atoms"][1]["element"], 8);
    assert_eq!(external["meaning"]["smiles"], "[13CH3]O");
    assert_eq!(external["meaning"]["conformers"], 0);
    Ok(())
}

#[test]
fn export_schema_selection_and_source_guards_are_closed_and_precede_io() -> TestResult {
    use bondline_commands::Command;
    let registry = bondline_commands::registry();
    assert_eq!(registry.len(), 51);
    let info = registry
        .iter()
        .find(|i| i.name == NAME)
        .ok_or("export registry")?;
    assert!(!info.read_only);
    let schema = info.input_schema.as_value();
    assert_eq!(schema["additionalProperties"], false);
    assert_eq!(schema["required"].as_array().ok_or("required")?.len(), 4);
    assert_eq!(schema["properties"]["page_id"]["minimum"], 1);
    assert_eq!(schema["properties"]["fragment_id"]["minimum"], 1);
    let directory = support::FileTestDirectory::new()?;
    let path = directory.path("absent.smi");
    let valid = params(0, 9, &path);
    for key in ["expected_revision", "page_id", "fragment_id", "output_path"] {
        let mut invalid = valid.clone();
        invalid.as_object_mut().ok_or("params")?.remove(key);
        assert_eq!(
            Command::parse(NAME, invalid)
                .err()
                .ok_or("missing required")?
                .info()
                .code,
            "invalid_params"
        );
    }
    for key in [
        "overwrite",
        "hydrogen_policy",
        "canonical",
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
    let mut engine = Engine::from_document(owner()?);
    let before = serde_json::to_value(engine.state())?;
    for (key, value, code) in [
        ("expected_revision", 1, "stale_revision"),
        ("page_id", 0, "invalid_params"),
        ("fragment_id", 0, "invalid_params"),
        ("page_id", 2, "not_found"),
        ("fragment_id", 99, "not_found"),
    ] {
        let mut invalid = valid.clone();
        invalid[key] = json!(value);
        assert_eq!(
            engine
                .prepare(Command::parse(NAME, invalid)?)
                .err()
                .ok_or("bad selection")?
                .info()
                .code,
            code
        );
        assert_eq!(serde_json::to_value(engine.state())?, before);
        assert!(!path.try_exists()?);
    }
    let source = directory.path("collection.sdf");
    fs::write(
        &source,
        include_bytes!("../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf"),
    )?;
    engine.dispatch("document.open_sdf", json!({"path":source}))?;
    let before = serde_json::to_value(engine.state())?;
    assert_eq!(
        engine
            .prepare(Command::parse(NAME, params(1, 1, &path))?)
            .err()
            .ok_or("SDF projection export")?
            .info()
            .code,
        "unsupported_content"
    );
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert!(!path.try_exists()?);
    Ok(())
}

#[test]
fn full_writer_profile_is_not_restricted_to_carbon_layout_or_twelve_atoms() -> TestResult {
    let atom = |id, element, h| -> Result<Atom, Box<dyn std::error::Error>> {
        let mut atom = Atom::new(
            AtomId::new(id).ok_or("atom")?,
            Element::new(element).ok_or("element")?,
            None,
        );
        atom.explicit_hydrogens = Some(h);
        Ok(atom)
    };
    let mut atoms = vec![];
    for (id, element, h) in [
        (1, 6, 1),
        (2, 6, 3),
        (3, 6, 3),
        (4, 6, 3),
        (10, 6, 2),
        (11, 6, 2),
        (20, 6, 1),
        (21, 6, 1),
        (30, 6, 2),
        (31, 6, 2),
        (32, 6, 2),
        (40, 1, 0),
        (41, 1, 0),
        (43, 7, 3),
        (44, 8, 2),
        (45, 9, 1),
        (46, 17, 1),
        (47, 35, 1),
        (48, 53, 1),
    ] {
        atoms.push(atom(id, element, h)?);
    }
    atoms[0].isotope = NonZeroU16::new(13);
    atoms[11].isotope = NonZeroU16::new(2);
    let bonds = [
        (1, 2, BondOrder::Single),
        (1, 3, BondOrder::Single),
        (1, 4, BondOrder::Single),
        (10, 11, BondOrder::Double),
        (20, 21, BondOrder::Triple),
        (30, 31, BondOrder::Single),
        (31, 32, BondOrder::Single),
        (32, 30, BondOrder::Single),
        (40, 41, BondOrder::Single),
    ]
    .into_iter()
    .enumerate()
    .map(|(i, (a, b, order))| {
        Ok(Bond {
            double_position: bondline_model::DoublePosition::Unspecified,
            id: BondId::new(i as u32 + 1).ok_or("bond")?,
            begin: AtomId::new(a).ok_or("atom")?,
            end: AtomId::new(b).ok_or("atom")?,
            order,
        })
    })
    .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
    let mut stored = StoredDocument::default();
    stored.model_mut().append_fragments(
        PageId::new(1).ok_or("page")?,
        vec![Fragment::new(
            FragmentId::new(1).ok_or("fragment")?,
            atoms,
            bonds,
        )?],
    )?;
    let mut engine = Engine::from_document(stored);
    let content = engine.content().clone();
    let directory = support::FileTestDirectory::new()?;
    let path = directory.path("wide-profile.smi");
    let result = engine.dispatch(NAME, params(0, 1, &path))?;
    assert_eq!(fs::read(&path)?,b"[13CH1]([CH3])([CH3])[CH3].[CH2]=[CH2].[CH1]#[CH1].[CH2]0[CH2][CH2]0.[2H][H].[NH3].[OH2].[FH1].[ClH1].[BrH1].[IH1]\n");
    assert_eq!(engine.content(), &content);
    assert_eq!(
        serde_json::to_value(result)?["smiles_export"]["atom_count"],
        19
    );
    let external = consume(&path)?;
    let atoms = external["notation"]["atoms"].as_array().ok_or("atoms")?;
    assert_eq!(atoms.len(), 19);
    for (actual, (element, h, isotope)) in atoms.iter().zip([
        (6, 1, 13),
        (6, 3, 0),
        (6, 3, 0),
        (6, 3, 0),
        (6, 2, 0),
        (6, 2, 0),
        (6, 1, 0),
        (6, 1, 0),
        (6, 2, 0),
        (6, 2, 0),
        (6, 2, 0),
        (1, 0, 2),
        (1, 0, 0),
        (7, 3, 0),
        (8, 2, 0),
        (9, 1, 0),
        (17, 1, 0),
        (35, 1, 0),
        (53, 1, 0),
    ]) {
        assert_eq!(actual["element"], element);
        assert_eq!(actual["explicit_h"], h);
        assert_eq!(actual["isotope"], isotope);
    }
    assert_eq!(
        external["notation"]["bonds"],
        json!([
            [0, 1, "SINGLE"],
            [0, 2, "SINGLE"],
            [0, 3, "SINGLE"],
            [4, 5, "DOUBLE"],
            [6, 7, "TRIPLE"],
            [8, 9, "SINGLE"],
            [8, 10, "SINGLE"],
            [9, 10, "SINGLE"],
            [11, 12, "SINGLE"]
        ])
    );
    assert_eq!(external["meaning"]["radicals"], json!(vec![0; 19]));
    assert_eq!(external["meaning"]["conformers"], 0);
    Ok(())
}

#[test]
fn captured_export_keeps_its_whole_owner_lease_and_identity_across_edits_and_new_open() -> TestResult
{
    use bondline_commands::{Cancellation, Command};
    let directory = support::FileTestDirectory::new()?;
    let path = directory.path("captured.smi");
    let input = directory.path("new.cdxml");
    fs::write(
        &input,
        include_bytes!("../../io/tests/fixtures/bondline-v0.cdxml"),
    )?;
    let mut engine = Engine::from_document(owner()?);
    engine.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":700.0,"height_pt":900.0}),
    )?;
    engine.dispatch("history.undo", json!({}))?;
    engine.dispatch(
        "document.close",
        json!({"expected_revision":2,"action":"request"}),
    )?;
    let before = serde_json::to_value(engine.state())?;
    let prepared = engine.prepare(Command::parse(NAME, params(2, 9, &path))?)?;
    assert!(prepared.is_deferred());
    assert_eq!(serde_json::to_value(engine.state())?, before);
    for command in [
        Command::parse(NAME, params(2, 9, &directory.path("second.smi")))?,
        Command::parse(
            "document.save_cdxml",
            json!({"path":directory.path("native.cdxml")}),
        )?,
        Command::parse(
            "file.convert_smiles_to_cdxml",
            json!({"expected_revision":2,"input_path":"not-read.smi","output_path":directory.path("not-created.cdxml"),"bond_length_pt":48.0,"origin_x_pt":72.0,"origin_y_pt":72.0,"page_width_pt":612.0,"page_height_pt":792.0}),
        )?,
    ] {
        assert_eq!(
            engine
                .prepare(command)
                .err()
                .ok_or("export reservation bypassed")?
                .info()
                .code,
            "busy"
        );
    }
    engine.dispatch("document.open_cdxml", json!({"path":input}))?;
    let before = serde_json::to_value(engine.state())?;
    let content = engine.content().clone();
    let cancel = Cancellation::default();
    let completed = prepared.complete_with_cancel(&cancel)?;
    assert_eq!(fs::read(&path)?, b"[13CH3][OH1]\n");
    assert_eq!(
        engine
            .prepare(Command::parse(
                "document.save_cdxml",
                json!({"path":directory.path("after-publication.cdxml")})
            )?)
            .err()
            .ok_or("early lease release")?
            .info()
            .code,
        "busy"
    );
    cancel.cancel();
    let mut result = serde_json::to_value(engine.finish(completed)?)?;
    let receipt = result
        .as_object_mut()
        .ok_or("result")?
        .remove("smiles_export")
        .ok_or("receipt")?;
    assert_eq!(result, before);
    assert_eq!(engine.content(), &content);
    assert_eq!(receipt["input_revision"], 2);
    assert_eq!(receipt["session_generation"], 0);
    assert_eq!(receipt["fragment_id"], 9);
    assert_eq!(serde_json::to_value(engine.state())?, before);
    consume(&path)?;
    drop(engine.prepare(Command::parse(
        "document.save_cdxml",
        json!({"path":directory.path("released.cdxml")}),
    )?)?);
    Ok(())
}

#[test]
fn incomplete_sources_and_unsupported_writer_content_refuse_without_loss_or_output() -> TestResult {
    use bondline_commands::Command;
    use bondline_model::{PageSize, Radical};
    let directory = support::FileTestDirectory::new()?;
    let xml = StoredDocument::from_cdxml(
        include_bytes!("../../cdxml/tests/fixtures/unknown.cdxml")
            .as_slice()
            .into(),
        PageSize::A4,
    )?;
    assert!(!xml.is_fully_mapped());
    let mut engine = Engine::from_document(xml);
    let before = engine.content().clone();
    let state = serde_json::to_value(engine.state())?;
    let path = directory.path("opaque.smi");
    assert_eq!(
        engine
            .prepare(Command::parse(NAME, params(0, 1, &path))?)
            .err()
            .ok_or("opaque projection export")?
            .info()
            .code,
        "unsupported_content"
    );
    assert_eq!(engine.content(), &before);
    assert_eq!(serde_json::to_value(engine.state())?, state);
    assert!(!path.try_exists()?);
    let binary = StoredDocument::from_cdx(
        include_bytes!("../../cdx/tests/fixtures/original-ethane-explicit.cdx")
            .as_slice()
            .into(),
        PageSize::A4,
    )?;
    let mut cdx = Engine::from_document(binary);
    let before = cdx.content().clone();
    assert_eq!(
        cdx.prepare(Command::parse(
            NAME,
            params(0, 1, &directory.path("binary.smi"))
        )?)
        .err()
        .ok_or("CDX projection export")?
        .info()
        .code,
        "unsupported_content"
    );
    assert_eq!(cdx.content(), &before);
    assert!(!directory.path("binary.smi").try_exists()?);
    for case in 0..5 {
        let mut stored = owner()?;
        let mut atoms = stored.model().pages()[0].fragments()[1].atoms().to_vec();
        match case {
            0 => atoms[0].explicit_hydrogens = None,
            1 => atoms[0].charge = 1,
            2 => atoms[0].radical = Radical::Doublet,
            3 => atoms[0].explicit_hydrogens = Some(2),
            _ => atoms[0].element = Element::new(5).ok_or("boron")?,
        }
        let bonds = stored.model().pages()[0].fragments()[1].bonds().to_vec();
        stored.model_mut().replace_fragment(
            PageId::new(1).ok_or("page")?,
            Fragment::new(FragmentId::new(9).ok_or("fragment")?, atoms, bonds)?,
        )?;
        let mut engine = Engine::from_document(stored);
        let before = engine.content().clone();
        let state = serde_json::to_value(engine.state())?;
        let path = directory.path(&format!("unsupported-{case}.smi"));
        assert_eq!(
            engine
                .dispatch(NAME, params(0, 9, &path))
                .err()
                .ok_or("unsupported writer content")?
                .info()
                .code,
            "unsupported_content"
        );
        assert!(!path.try_exists()?);
        assert_eq!(engine.content(), &before);
        assert_eq!(serde_json::to_value(engine.state())?, state);
        drop(engine.prepare(Command::parse(
            "document.save_cdxml",
            json!({"path":directory.path("retry.cdxml")}),
        )?)?);
    }
    Ok(())
}

#[test]
fn export_cancel_drop_native_binding_and_late_close_never_change_the_session() -> TestResult {
    use bondline_commands::{Cancellation, Command};
    let directory = support::FileTestDirectory::new()?;
    for case in 0..4 {
        let path = directory.path(&format!("lifecycle-{case}.smi"));
        let mut engine = Engine::from_document(owner()?);
        engine.dispatch(
            "document.close",
            json!({"expected_revision":0,"action":"request"}),
        )?;
        let before = serde_json::to_value(engine.state())?;
        let prepared = engine.prepare(Command::parse(NAME, params(0, 9, &path))?)?;
        match case {
            0 => {
                drop(prepared);
                assert!(!path.try_exists()?);
            }
            1 => {
                let cancel = Cancellation::default();
                cancel.cancel();
                assert_eq!(
                    prepared
                        .complete_with_cancel(&cancel)
                        .err()
                        .ok_or("cancel accepted")?
                        .info()
                        .code,
                    "cancelled"
                );
                assert!(!path.try_exists()?);
            }
            2 => {
                let completed = prepared.complete()?;
                assert_eq!(fs::read(&path)?, b"[13CH3][OH1]\n");
                drop(completed);
            }
            _ => {
                assert!(engine.complete_close());
                let state = serde_json::to_value(engine.state())?;
                let result = serde_json::to_value(engine.finish(prepared.complete()?)?)?;
                assert_eq!(result["close"], "committed");
                assert_eq!(serde_json::to_value(engine.state())?, state);
                assert_eq!(result["smiles_export"]["input_revision"], 0);
                continue;
            }
        }
        assert_eq!(serde_json::to_value(engine.state())?, before);
        drop(engine.prepare(Command::parse(
            NAME,
            params(0, 9, &directory.path(&format!("free-{case}.smi"))),
        )?)?);
    }
    let mut engine = Engine::from_document(owner()?);
    engine.set_ui_file_host(true);
    let request = engine.dispatch("ui.file", json!({"expected_revision":0,"action":"save_as"}))?;
    let id = request.ui_file_operation.ok_or("id")?;
    engine.claim_ui_file(id)?;
    let before = serde_json::to_value(engine.state())?;
    let path = directory.path("not-bound.smi");
    assert_eq!(
        engine
            .prepare_ui_file(id, Command::parse(NAME, params(0, 9, &path))?)
            .err()
            .ok_or("bound export")?
            .info()
            .code,
        "invalid_params"
    );
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert!(!path.try_exists()?);
    fs::write(&path, b"existing bytes")?;
    assert_eq!(
        engine
            .dispatch(NAME, params(0, 9, &path))
            .err()
            .ok_or("overwritten output")?
            .info()
            .code,
        "already_exists"
    );
    assert_eq!(fs::read(&path)?, b"existing bytes");
    assert_eq!(serde_json::to_value(engine.state())?, before);
    Ok(())
}
