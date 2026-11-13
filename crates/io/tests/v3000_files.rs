use bondline_io::{
    StoredDocument,
    files::{read_mol_file, save_mol_file, save_mol_file_as},
    mol::{MolTransform, Version, v3000},
};
use bondline_model::{PageId, PageSize, Point2};
use bondline_render::Cancellation;
use std::{
    fs,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};
type TestResult = Result<(), Box<dyn std::error::Error>>;
const V3: &[u8] = include_bytes!("fixtures/v3000/noncontiguous-cc.mol");
const V2: &[u8] = include_bytes!("fixtures/mol/rdkit-CH3.mol");
struct Directory(PathBuf);
impl Directory {
    fn new() -> Result<Self, std::io::Error> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "bondline-v3000-files-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path)?;
        Ok(Self(path))
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn frame() -> Result<MolTransform, bondline_io::mol::MolError> {
    MolTransform::new(24.0, Point2::new(72.0, 72.0)?)
}

#[test]
fn bounded_auto_version_read_and_default_save_keep_v3000_source_exact_despite_misleading_extension()
-> TestResult {
    let directory = Directory::new()?;
    let source = directory.0.join("source.v2000");
    let destination = directory.0.join("saved.v2000");
    fs::write(&source, V3)?;
    let loaded = read_mol_file(&source, frame()?, &Cancellation::default())?;
    assert_eq!(loaded.document.mol_version(), Some(Version::V3000));
    assert_eq!(loaded.document.export_mol()?, V3);
    let receipt = save_mol_file(
        &destination,
        &loaded.document,
        false,
        frame()?,
        &Cancellation::default(),
    )?;
    assert_eq!(receipt.path, destination.canonicalize()?);
    assert_eq!(receipt.byte_length, V3.len());
    assert_eq!(fs::read(&destination)?, V3);
    assert_eq!(
        save_mol_file(
            &destination,
            &loaded.document,
            false,
            frame()?,
            &Cancellation::default()
        )
        .err()
        .ok_or("clobbered")?
        .code(),
        "already_exists"
    );
    assert_eq!(fs::read(&destination)?, V3);
    Ok(())
}

#[test]
fn explicit_version_publication_uses_the_selected_codec_and_refuses_downgrade_before_temporary_creation()
-> TestResult {
    let directory = Directory::new()?;
    let path = directory.0.join("keep.mol");
    fs::write(&path, b"previous bytes")?;
    let v3 = StoredDocument::from_mol(Arc::from(V3), PageSize::A4, frame()?)?;
    let before = v3.clone();
    let error = save_mol_file_as(
        &path,
        &v3,
        true,
        Version::V2000,
        frame()?,
        &Cancellation::default(),
    )
    .err()
    .ok_or("downgrade published")?;
    assert_eq!(error.code(), "unsupported_content");
    assert_eq!(error.stage(), "encode");
    assert!(!error.published());
    assert_eq!(fs::read(&path)?, b"previous bytes");
    assert_eq!(fs::read_dir(&directory.0)?.count(), 1);
    assert_eq!(v3, before);
    let v2 = StoredDocument::from_mol(Arc::from(V2), PageSize::A4, frame()?)?;
    save_mol_file_as(
        &path,
        &v2,
        true,
        Version::V3000,
        frame()?,
        &Cancellation::default(),
    )?;
    assert!(v3000::MolFile::parse(&fs::read(&path)?).is_ok());
    assert_eq!(v2.export_mol()?, V2);
    let native = directory.0.join("native.mol");
    save_mol_file_as(
        &native,
        &StoredDocument::default(),
        false,
        Version::V3000,
        frame()?,
        &Cancellation::default(),
    )?;
    assert!(v3000::MolFile::parse(&fs::read(&native)?).is_ok());
    Ok(())
}

#[test]
fn invalid_reads_and_format_specific_failures_preserve_current_document_and_destination()
-> TestResult {
    let directory = Directory::new()?;
    let path = directory.0.join("keep.mol");
    fs::write(&path, b"keep")?;
    let mut document = StoredDocument::from_mol(Arc::from(V3), PageSize::A4, frame()?)?;
    document
        .model_mut()
        .set_page_size(PageId::new(1).ok_or("page")?, PageSize::new(612.0, 792.0)?)?;
    let before = document.clone();
    let error = save_mol_file(&path, &document, true, frame()?, &Cancellation::default())
        .err()
        .ok_or("layout was lost")?;
    assert_eq!(error.code(), "unsupported_content");
    assert!(!error.published());
    assert_eq!(error.stage(), "encode");
    assert_eq!(fs::read(&path)?, b"keep");
    assert_eq!(document, before);
    for (bytes, code) in [
        (b"not a mol".to_vec(), "invalid_mol"),
        (
            std::str::from_utf8(V3)?
                .replace("COUNTS 2 1", "COUNTS 3 1")
                .into_bytes(),
            "invalid_mol",
        ),
        (
            std::str::from_utf8(V3)?
                .replace("VAL=4", "VAL=4 X=\"original\"")
                .into_bytes(),
            "unsupported_content",
        ),
    ] {
        let source = directory.0.join("bad.mol");
        fs::write(&source, bytes)?;
        let error = read_mol_file(&source, frame()?, &Cancellation::default())
            .err()
            .ok_or("bad input accepted")?;
        assert_eq!(error.code(), code);
        assert_eq!(error.stage(), "decode");
        assert!(!error.published());
        assert_eq!(document, before);
        assert_eq!(fs::read(&path)?, b"keep");
    }
    let cancel = Cancellation::default();
    cancel.cancel();
    assert_eq!(
        save_mol_file_as(&path, &before, true, Version::V3000, frame()?, &cancel)
            .err()
            .ok_or("cancel ignored")?
            .code(),
        "cancelled"
    );
    assert_eq!(fs::read(&path)?, b"keep");
    Ok(())
}
