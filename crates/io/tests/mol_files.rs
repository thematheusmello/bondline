use bondline_io::{
    StoredDocument,
    files::{read_mol_file, save_cdxml_file, save_mol_file},
    mol::MolTransform,
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
const CH3: &[u8] = include_bytes!("fixtures/mol/rdkit-CH3.mol");

struct Directory(PathBuf);
impl Directory {
    fn new() -> Result<Self, std::io::Error> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "bondline-mol-files-{}-{}",
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
    MolTransform::new(24.0, Point2::new(100.0, 200.0)?)
}

#[test]
fn native_mol_read_and_same_format_save_preserve_the_entire_pinned_file() -> TestResult {
    let directory = Directory::new()?;
    let source = directory.0.join("source.mol");
    let destination = directory.0.join("saved.mol");
    fs::write(&source, CH3)?;
    let loaded = read_mol_file(&source, frame()?, &Cancellation::default())?;
    assert_eq!(loaded.path, source.canonicalize()?);
    assert_eq!(loaded.document.export_mol()?, CH3);
    let receipt = save_mol_file(
        &destination,
        &loaded.document,
        false,
        frame()?,
        &Cancellation::default(),
    )?;
    assert_eq!(receipt.path, destination.canonicalize()?);
    assert_eq!(receipt.byte_length, 170);
    assert_eq!(fs::read(&destination)?, CH3);
    assert_eq!(
        save_mol_file(
            &destination,
            &loaded.document,
            false,
            frame()?,
            &Cancellation::default()
        )
        .err()
        .ok_or("clobbered existing destination")?
        .code(),
        "already_exists"
    );
    assert_eq!(fs::read(&destination)?, CH3);
    let cross = directory.0.join("lossy.cdxml");
    let error = save_cdxml_file(&cross, &loaded.document, false, &Cancellation::default())
        .err()
        .ok_or("lost MOL metadata")?;
    assert_eq!(error.code(), "unsupported_content");
    assert!(!error.published());
    assert!(!cross.exists());
    Ok(())
}

#[test]
fn unsupported_edits_and_failed_imports_leave_full_source_and_old_destination_untouched()
-> TestResult {
    let directory = Directory::new()?;
    let path = directory.0.join("keep.mol");
    fs::write(&path, b"old destination")?;
    let mut document = StoredDocument::from_mol(Arc::from(CH3), PageSize::A4, frame()?)?;
    let original = document.clone();
    document
        .model_mut()
        .set_page_size(PageId::new(1).ok_or("page")?, PageSize::new(612.0, 792.0)?)?;
    let before = document.clone();
    let error = save_mol_file(&path, &document, true, frame()?, &Cancellation::default())
        .err()
        .ok_or("dropped page edit")?;
    assert_eq!(error.code(), "unsupported_content");
    assert!(!error.published());
    assert_eq!(error.stage(), "encode");
    assert_eq!(document, before);
    assert_eq!(fs::read(&path)?, b"old destination");
    assert_eq!(fs::read_dir(&directory.0)?.count(), 1);
    assert_eq!(original.export_mol()?, CH3);
    for (source, code) in [
        (b"not a MOL".to_vec(), "invalid_mol"),
        (
            std::str::from_utf8(CH3)?
                .replace("M  END", "M  XYZ  1   1   7\nM  END")
                .into_bytes(),
            "unsupported_content",
        ),
    ] {
        let malformed = directory.0.join("bad.mol");
        fs::write(&malformed, source)?;
        let error = read_mol_file(&malformed, frame()?, &Cancellation::default())
            .err()
            .ok_or("bad input accepted")?;
        assert_eq!(error.code(), code);
        assert!(!error.published());
        assert_eq!(error.stage(), "decode");
        assert_eq!(document, before);
        assert_eq!(original.export_mol()?, CH3);
    }
    Ok(())
}

#[test]
fn mol_files_share_native_bounds_ordinary_file_checks_and_prepublication_cancellation() -> TestResult
{
    let directory = Directory::new()?;
    let oversized = directory.0.join("large.mol");
    fs::File::create(&oversized)?.set_len(64 * 1024 * 1024 + 1)?;
    assert_eq!(
        read_mol_file(&oversized, frame()?, &Cancellation::default())
            .err()
            .ok_or("oversized file")?
            .code(),
        "file_too_large"
    );
    assert_eq!(
        read_mol_file(&directory.0, frame()?, &Cancellation::default())
            .err()
            .ok_or("directory read")?
            .code(),
        "not_regular_file"
    );
    let source = directory.0.join("source.mol");
    fs::write(&source, CH3)?;
    let destination = directory.0.join("cancelled.mol");
    fs::write(&destination, b"keep me")?;
    let document = StoredDocument::from_mol(Arc::from(CH3), PageSize::A4, frame()?)?;
    let cancel = Cancellation::default();
    cancel.cancel();
    let error = save_mol_file(&destination, &document, true, frame()?, &cancel)
        .err()
        .ok_or("cancelled save ran")?;
    assert_eq!(error.code(), "cancelled");
    assert!(!error.published());
    assert_eq!(fs::read(&destination)?, b"keep me");
    assert_eq!(
        read_mol_file(&source, frame()?, &cancel)
            .err()
            .ok_or("cancelled read ran")?
            .code(),
        "cancelled"
    );
    Ok(())
}
