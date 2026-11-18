//! Original binary, complete model and persistence witnesses for owned labels.
use bondline_io::{
    DocumentError, StoredDocument,
    cdx::ElementalGraph,
    display::{DisplayError, drawing_page_scene},
    files::{read_cdx_file, save_cdx_file},
    mol::MolTransform,
};
use bondline_model::{Fragment, PageSize, Point2};
use bondline_render::Cancellation;
use serde_json::Value;
use std::{borrow::Cow, collections::BTreeMap, fs, path::Path, sync::Arc};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse the existing isolated file directory")]
mod files;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const WITNESSES: &str = include_str!("fixtures/cdx-node-labels/cases.json");
const LABELED: &[u8] = include_bytes!("fixtures/cdx-node-labels/literal_conflict.cdx");
const REORDERED: &[u8] = include_bytes!("fixtures/cdx-node-labels/reordered_conflict.cdx");

fn source(name: &str) -> Result<Arc<[u8]>> {
    Ok(Arc::from(fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/cdx-node-labels")
            .join(format!("{name}.cdx")),
    )?))
}

#[test]
fn explicit_child_text_preserves_all_owned_fields_independently_of_the_numeric_graph() -> Result {
    let witnesses: BTreeMap<String, Value> = serde_json::from_str(WITNESSES)?;
    let mut failures = Vec::new();
    let mut admitted = 0;
    for (name, case) in witnesses {
        if case["model"].is_null() {
            continue;
        }
        let source = source(&name)?;
        let graph = ElementalGraph::parse(&source);
        let stored = StoredDocument::from_cdx(Arc::clone(&source), PageSize::A4);
        let (graph, stored) = match (graph, stored) {
            (Ok(graph), Ok(stored)) => (graph, stored),
            (graph, stored) => {
                failures.push(format!("{name}: {:?} / {:?}", graph.err(), stored.err()));
                continue;
            }
        };
        assert_eq!(
            serde_json::to_value(stored.model())?,
            case["model"],
            "{name}"
        );
        assert_eq!(
            serde_json::to_value(graph.fragments())?,
            case["model"]["pages"][0]["fragments"],
            "{name}"
        );
        assert_eq!(graph.as_bytes().as_ptr(), source.as_ptr(), "{name}");
        assert_eq!(graph.to_bytes(), source.as_ref(), "{name}");
        assert_eq!(stored.export_cdx()?.as_ptr(), source.as_ptr(), "{name}");
        assert!(matches!(stored.to_cdx()?, Cow::Borrowed(bytes) if bytes == source.as_ref()));
        assert!(stored.model().has_authored_atom_labels(), "{name}");
        assert!(!stored.is_fully_mapped(), "{name}");
        assert!(!stored.has_display_projection(), "{name}");
        assert!(!stored.has_cdx_caption_preview(), "{name}");
        admitted += 1;
    }
    assert!(
        failures.is_empty(),
        "explicit label rejections: {failures:?}"
    );
    assert_eq!(admitted, 7);
    Ok(())
}

#[test]
fn unproved_child_shapes_preserve_raw_bytes_and_legacy_diagnostics() -> Result {
    let witnesses: BTreeMap<String, Value> = serde_json::from_str(WITNESSES)?;
    let mut refused = 0;
    for (name, case) in witnesses {
        if !case["model"].is_null() {
            continue;
        }
        let source = source(&name)?;
        assert_eq!(
            bondline_cdx::Document::parse(&source)?.to_bytes(),
            source.as_ref()
        );
        let graph_error = ElementalGraph::parse(&source)
            .err()
            .ok_or_else(|| format!("unproved graph opened: {name}"))?;
        let stored_error = StoredDocument::from_cdx(source, PageSize::A4)
            .err()
            .ok_or_else(|| format!("unproved document opened: {name}"))?;
        for (code, detail) in [
            (graph_error.code(), graph_error.to_string()),
            (stored_error.code(), stored_error.to_string()),
        ] {
            assert_eq!(code, case["legacy"]["code"], "{name}");
            assert_eq!(detail, case["legacy"]["detail"], "{name}");
        }
        refused += 1;
    }
    assert_eq!(refused, 27);
    Ok(())
}

#[test]
fn exact_file_backing_survives_unicode_save_and_reopen_but_refuses_label_drift() -> Result {
    let directory = files::FileTestDirectory::new()?;
    let input = directory.path("atom-原-λ.cdx");
    let output = directory.path("atom-再開-β.cdx");
    fs::write(&input, LABELED)?;
    let cancel = Cancellation::default();
    let mut loaded = read_cdx_file(&input, &cancel)?.document;
    let before = loaded.clone();
    let alternate = StoredDocument::from_cdx(Arc::from(REORDERED), PageSize::A4)?;
    assert_eq!(loaded.model(), alternate.model());
    assert_ne!(loaded, alternate);
    assert_eq!(alternate.export_cdx()?, REORDERED);
    let page = loaded.model().pages().first().ok_or("missing page")?;
    let page_id = page.id();
    assert!(matches!(
        drawing_page_scene(1024, 1024, &loaded, page_id, &cancel),
        Err(DisplayError::IncompleteSource)
    ));
    assert!(matches!(
        loaded.to_cdxml(),
        Err(DocumentError::CrossFormatSource {
            from: "cdx",
            to: "cdxml"
        })
    ));
    let frame = MolTransform::new(24.0, Point2::new(72.0, 72.0)?)?;
    assert!(matches!(
        loaded.to_mol(frame),
        Err(DocumentError::UnsupportedMolLayout(
            "authored atom text cannot be represented"
        ))
    ));
    let original = page
        .fragments()
        .first()
        .ok_or("missing fragment")?
        .as_ref()
        .clone();
    let mut atoms = original.atoms().to_vec();
    let atom = atoms.first_mut().ok_or("missing atom")?;
    let label = atom.authored_label.as_deref().ok_or("missing label")?;
    atom.authored_label = Some(Arc::new(label.with_run_text(0, "Xe".into())?));
    loaded.model_mut().replace_fragment(
        page_id,
        Fragment::new(original.id(), atoms, original.bonds().to_vec())?,
    )?;
    assert!(loaded.source_info().ok_or("missing backing")?.modified);
    assert!(matches!(
        loaded.to_cdx(),
        Err(DocumentError::UnsupportedCdxEdit)
    ));
    assert!(save_cdx_file(&output, &loaded, false, &cancel).is_err());
    assert!(!output.exists());
    assert_eq!(fs::read(&input)?, LABELED);

    loaded.model_mut().replace_fragment(page_id, original)?;
    assert_eq!(loaded, before);
    save_cdx_file(&input, &loaded, true, &cancel)?;
    save_cdx_file(&output, &loaded, false, &cancel)?;
    assert_eq!(fs::read(&input)?, LABELED);
    assert_eq!(fs::read(&output)?, LABELED);
    assert_eq!(read_cdx_file(&output, &cancel)?.document, before);
    Ok(())
}
