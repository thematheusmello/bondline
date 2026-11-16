use bondline_io::{
    DocumentError, StoredDocument,
    mol::{MolFile, MolTransform},
};
use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Document, Element, Fragment, FragmentId, PageId,
    PageSize, Point2,
};
use std::{borrow::Cow, mem::size_of, sync::Arc};
use std::{
    io::Write,
    process::{Command, Stdio},
};

type TestResult = Result<(), Box<dyn std::error::Error>>;
const CH3: &[u8] = include_bytes!("fixtures/mol/rdkit-CH3.mol");
const NATIVE: &[u8] = include_bytes!("fixtures/bondline-v0.cdxml");
const OPAQUE: &[u8] = include_bytes!("../../cdxml/tests/fixtures/unknown.cdxml");

fn frame() -> Result<MolTransform, DocumentError> {
    MolTransform::new(
        24.0,
        Point2::new(100.0, 200.0).map_err(bondline_io::mol::MolError::Model)?,
    )
    .map_err(DocumentError::Mol)
}

fn external(bytes: &[u8]) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child = Command::new(python)
        .arg("-B")
        .arg(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/mol_oracle.py"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child.stdin.take().ok_or("oracle stdin")?.write_all(bytes)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(serde_json::from_slice(&output.stdout)?)
}

fn append_cc(document: &mut Document, base: u32) -> TestResult {
    let c = Element::new(6).ok_or("C")?;
    let first = AtomId::new(base).ok_or("first")?;
    let second = AtomId::new(base + 1).ok_or("second")?;
    let mut a = Atom::new(first, c, Some(Point2::new(100.0, 100.0)?));
    let mut b = Atom::new(second, c, Some(Point2::new(136.0, 100.0)?));
    a.explicit_hydrogens = Some(3);
    b.explicit_hydrogens = Some(3);
    document.append_fragments(
        PageId::new(1).ok_or("page")?,
        vec![Fragment::new(
            FragmentId::new(base).ok_or("fragment")?,
            vec![a, b],
            vec![Bond {
                double_position: bondline_model::DoublePosition::Unspecified,
                id: BondId::new(base).ok_or("bond")?,
                begin: first,
                end: second,
                order: BondOrder::Single,
            }],
        )?],
    )?;
    Ok(())
}

#[test]
fn owned_mol_backing_keeps_exact_bytes_headers_frame_and_original_model_without_debug_leak()
-> TestResult {
    let source: Arc<[u8]> = CH3.into();
    let pointer = source.as_ptr();
    let stored = StoredDocument::from_mol(source, PageSize::A4, frame()?)?;
    assert_eq!(stored.export_mol()?, CH3);
    assert_eq!(stored.export_mol()?.as_ptr(), pointer);
    assert_eq!(stored.mol_transform(), Some(frame()?));
    assert!(matches!(stored.to_mol(frame()?)?, Cow::Borrowed(_)));
    assert!(stored.is_fully_mapped());
    let info = serde_json::to_value(stored.source_info().ok_or("source")?)?;
    assert_eq!(
        info,
        serde_json::json!({"format":"mol","byte_length":170,"modified":false})
    );
    assert!(!format!("{stored:?}").contains("Mrv0541"));
    assert!(
        stored.to_cdxml().is_err(),
        "MOL headers/source must not silently vanish in a CDXML save"
    );
    let fragment = stored
        .model()
        .pages()
        .first()
        .ok_or("page")?
        .fragments()
        .first()
        .ok_or("fragment")?;
    assert_eq!(
        fragment.atoms().first().ok_or("atom")?.explicit_hydrogens,
        Some(3)
    );
    Ok(())
}

#[test]
fn source_free_native_cc_can_write_mol_but_metadata_or_fragment_layout_loss_is_refused()
-> TestResult {
    let mut native = StoredDocument::default();
    append_cc(native.model_mut(), 1)?;
    let encoded = native.to_mol(frame()?)?;
    let mapped = MolFile::parse(&encoded)?.to_fragment(frame()?)?;
    assert_eq!(mapped.atoms().len(), 2);
    assert_eq!(mapped.bonds().len(), 1);
    assert!(native.source_info().is_none());
    let before = native.clone();
    append_cc(native.model_mut(), 10)?;
    assert!(
        native.to_mol(frame()?).is_err(),
        "must not silently flatten authored fragment partition"
    );
    assert_ne!(native, before);
    let mut custom = before.clone();
    custom
        .model_mut()
        .set_page_size(PageId::new(1).ok_or("page")?, PageSize::new(612.0, 792.0)?)?;
    assert!(
        custom.to_mol(frame()?).is_err(),
        "MOL cannot preserve a custom authored page size"
    );
    assert!(
        StoredDocument::from_cdxml(Arc::from(OPAQUE), PageSize::A4)?
            .to_mol(frame()?)
            .is_err(),
        "unknown XML must stay guarded"
    );
    let mut known = StoredDocument::from_cdxml(Arc::from(NATIVE), PageSize::A4)?;
    assert!(known.is_fully_mapped());
    known
        .model_mut()
        .set_page_size(PageId::new(1).ok_or("page")?, PageSize::A4)?;
    assert!(
        known.to_mol(frame()?).is_ok(),
        "proven canonical native CDXML may export its completely mapped molecule"
    );
    Ok(())
}

#[test]
fn restored_mol_source_borrows_exact_original_and_page_edits_cannot_publish_stale_mol() -> TestResult
{
    let mut stored =
        StoredDocument::from_mol(Arc::from(CH3), PageSize::new(612.0, 792.0)?, frame()?)?;
    let original = stored.clone();
    let id = PageId::new(1).ok_or("page")?;
    stored.model_mut().set_page_size(id, PageSize::A4)?;
    assert!(stored.source_info().ok_or("source")?.modified);
    assert!(stored.export_mol().is_err());
    assert!(
        stored.to_mol(frame()?).is_err(),
        "changed preview page metadata cannot be dropped on same-format save"
    );
    stored
        .model_mut()
        .set_page_size(id, PageSize::new(612.0, 792.0)?)?;
    assert_eq!(stored, original);
    assert_eq!(stored.export_mol()?, CH3);
    Ok(())
}

#[test]
fn header_regeneration_limits_do_not_discard_source_or_advertise_an_editable_profile() -> TestResult
{
    let mut source = vec![b'x'; 81];
    source.extend_from_slice(b"\n");
    let text = std::str::from_utf8(CH3)?;
    let rest = text.split_once('\n').ok_or("header")?.1;
    source.extend_from_slice(rest.as_bytes());
    let stored = StoredDocument::from_mol(Arc::from(source.as_slice()), PageSize::A4, frame()?)?;
    assert!(
        !stored.is_fully_mapped(),
        "an unregenerable header must not authorize edits"
    );
    assert_eq!(stored.export_mol()?, source);
    assert!(
        stored.to_mol(frame()?).is_ok(),
        "untouched source still passes through exactly"
    );
    Ok(())
}

#[test]
fn source_header_storage_is_shared_by_clones_and_counted_in_retained_history() -> TestResult {
    let original = StoredDocument::from_mol(Arc::from(CH3), PageSize::A4, frame()?)?;
    let clone = original.clone();
    let once = StoredDocument::retained_bytes([&original]);
    assert_eq!(StoredDocument::retained_bytes([&original, &original]), once);
    assert_eq!(
        StoredDocument::retained_bytes([&original, &clone]) - once,
        size_of::<StoredDocument>()
    );
    let bytes: Arc<[u8]> = CH3.into();
    let shared = StoredDocument::from_mol(Arc::clone(&bytes), PageSize::A4, frame()?)?;
    let another = StoredDocument::from_mol(bytes, PageSize::A4, frame()?)?;
    let distinct = StoredDocument::from_mol(Arc::from(CH3), PageSize::A4, frame()?)?;
    assert_eq!(
        StoredDocument::retained_bytes([&shared, &distinct])
            - StoredDocument::retained_bytes([&shared, &another]),
        CH3.len()
    );
    Ok(())
}

#[test]
fn edited_mol_graph_keeps_all_headers_and_uses_its_original_frame_before_exact_restoration()
-> TestResult {
    let mut stored = StoredDocument::from_mol(Arc::from(CH3), PageSize::A4, frame()?)?;
    let original = stored.clone();
    let fragment = stored
        .model()
        .pages()
        .first()
        .ok_or("page")?
        .fragments()
        .first()
        .ok_or("fragment")?;
    let mut atoms = fragment.atoms().to_vec();
    atoms.first_mut().ok_or("atom")?.position = Some(Point2::new(124.0, 152.0)?);
    let changed = Fragment::new(fragment.id(), atoms, fragment.bonds().to_vec())?;
    let mut replacement = Document::default();
    replacement.append_fragments(PageId::new(1).ok_or("page")?, vec![changed])?;
    *stored.model_mut() = replacement;
    let different_native_frame = MolTransform::new(1.0, Point2::new(0.0, 0.0)?)?;
    let encoded = stored.to_mol(different_native_frame)?;
    assert!(matches!(encoded, Cow::Owned(_)));
    let output = MolFile::parse(&encoded)?;
    assert_eq!(output.header(), MolFile::parse(CH3)?.header());
    assert_eq!(
        (
            output.atoms().first().ok_or("atom")?.x,
            output.atoms().first().ok_or("atom")?.y
        ),
        (1.0, 2.0)
    );
    let independent = external(&encoded)?;
    assert_eq!(independent["smiles"], "[CH3]");
    assert_eq!(
        independent["atoms"][0]["position"],
        serde_json::json!([1.0, 2.0, 0.0])
    );
    assert!(stored.source_info().ok_or("source")?.modified);
    assert_eq!(original.export_mol()?, CH3);
    *stored.model_mut() = original.model().clone();
    assert_eq!(stored, original);
    assert_eq!(stored.export_mol()?, CH3);
    Ok(())
}

#[test]
fn empty_ctab_starts_as_an_empty_document_and_first_drawing_retains_source_headers() -> TestResult {
    let source=b"empty original\n  original program\noriginal comment\n  0  0  0  0  0  0            999 V2000\nM  END\n";
    let mut stored =
        StoredDocument::from_mol(Arc::from(source.as_slice()), PageSize::A4, frame()?)?;
    assert!(
        stored
            .model()
            .pages()
            .first()
            .ok_or("page")?
            .fragments()
            .is_empty()
    );
    assert_eq!(stored.export_mol()?, source);
    append_cc(stored.model_mut(), 1)?;
    let encoded = stored.to_mol(frame()?)?;
    assert_eq!(
        MolFile::parse(&encoded)?.header(),
        ["empty original", "  original program", "original comment"]
    );
    assert_eq!(external(&encoded)?["smiles"], "CC");
    Ok(())
}

#[test]
fn a_coordinate_frame_that_loses_original_positions_cannot_authorize_regeneration() -> TestResult {
    let tiny = MolTransform::new(1e-300, Point2::new(100.0, 200.0)?)?;
    let stored = StoredDocument::from_mol(Arc::from(CH3), PageSize::A4, tiny)?;
    assert!(
        !stored.is_fully_mapped(),
        "a representable writer alone does not prove it retained source geometry"
    );
    assert_eq!(stored.export_mol()?, CH3);
    let source = std::str::from_utf8(CH3)?.replace("   -1.2375", " -12345678");
    let scale = MolTransform::new(0.001, Point2::new(100.0, 200.0)?)?;
    let stored = StoredDocument::from_mol(Arc::from(source.as_bytes()), PageSize::A4, scale)?;
    assert!(
        !stored.is_fully_mapped(),
        "MOL field overflow must not be advertised as regenerable"
    );
    assert_eq!(stored.export_mol()?, source.as_bytes());
    Ok(())
}
