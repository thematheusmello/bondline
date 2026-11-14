#![cfg(not(target_arch = "wasm32"))]

use bondline_io::{
    StoredDocument,
    files::{read_cdx_file, save_cdx_file, save_cdxml_file},
};
use bondline_model::PageSize;
use bondline_render::Cancellation;
use std::{
    fs,
    path::PathBuf,
    process::{Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
};

type TestResult = Result<(), Box<dyn std::error::Error>>;
const LEGACY: &[u8] = include_bytes!("../../cdx/tests/fixtures/original-ethane-explicit.cdx");
const HEADER: &[u8] =
    include_bytes!("../../cdx/tests/fixtures/original-ethane-header-document.cdx");

struct Directory(PathBuf);
impl Directory {
    fn new() -> std::io::Result<Self> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "bondline-cdx-files-{}-{}",
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

#[test]
fn both_framings_save_all_original_bytes_and_face_the_independent_consumer() -> TestResult {
    for (source, oracle, arguments) in [
        (LEGACY, "cdx_oracle.py", &["verify", "1", "explicit"][..]),
        (HEADER, "cdx_header_oracle.py", &["verify", "1"][..]),
    ] {
        let directory = Directory::new()?;
        let path = directory.0.join("original-λ.misleading");
        let destination = directory.0.join("saved-λ.cdx");
        fs::write(&path, source)?;
        let loaded = read_cdx_file(&path, &Cancellation::default())?;
        assert_eq!(loaded.path, path.canonicalize()?);
        assert_eq!(loaded.document.export_cdx()?, source);
        assert!(!loaded.document.is_fully_mapped());
        let saved = save_cdx_file(
            &destination,
            &loaded.document,
            false,
            &Cancellation::default(),
        )?;
        assert_eq!(saved.path, destination.canonicalize()?);
        assert_eq!(saved.byte_length, source.len());
        assert_eq!(saved.directory_synced, cfg!(unix));
        assert_eq!(fs::read(&destination)?, source);
        let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
            if cfg!(windows) {
                "python".into()
            } else {
                "python3".into()
            }
        });
        let output = Command::new(python)
            .arg("-B")
            .arg(
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../../xtask/oracles")
                    .join(oracle),
            )
            .args(arguments)
            .stdin(fs::File::open(&destination)?)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(fs::read(&path)?, source);
        assert_eq!(fs::read_dir(&directory.0)?.count(), 2);
    }
    Ok(())
}

#[test]
fn overwrite_requires_opt_in_and_unsupported_exports_create_no_temporary_or_data_loss() -> TestResult
{
    let directory = Directory::new()?;
    let path = directory.0.join("keep.cdx");
    fs::write(&path, b"old destination")?;
    let mut stored = StoredDocument::from_cdx(HEADER.into(), PageSize::A4)?;
    let original = stored.clone();
    let error = save_cdx_file(&path, &stored, false, &Cancellation::default())
        .err()
        .ok_or("clobbered destination")?;
    assert_eq!(error.code(), "already_exists");
    assert!(!error.published());
    assert_eq!(fs::read(&path)?, b"old destination");
    let page = stored.model().pages().first().ok_or("page")?.id();
    stored
        .model_mut()
        .set_page_size(page, PageSize::new(612.0, 792.0)?)?;
    let edited = stored.clone();
    for snapshot in [&stored, &StoredDocument::default()] {
        let error = save_cdx_file(&path, snapshot, true, &Cancellation::default())
            .err()
            .ok_or("saved unsupported CDX projection")?;
        assert_eq!(error.code(), "unsupported_content");
        assert_eq!(error.stage(), "encode");
        assert!(!error.published());
        assert_eq!(fs::read(&path)?, b"old destination");
        assert_eq!(fs::read_dir(&directory.0)?.count(), 1);
    }
    assert_eq!(stored, edited);
    let cross = directory.0.join("lossy.cdxml");
    let error = save_cdxml_file(&cross, &original, false, &Cancellation::default())
        .err()
        .ok_or("lost binary metadata")?;
    assert_eq!(error.code(), "unsupported_content");
    assert_eq!(error.stage(), "encode");
    assert!(!cross.exists());
    let receipt = save_cdx_file(&path, &original, true, &Cancellation::default())?;
    assert_eq!(receipt.byte_length, HEADER.len());
    assert_eq!(fs::read(&path)?, HEADER);
    Ok(())
}

#[test]
fn failed_decode_and_native_input_bounds_do_not_modify_the_existing_snapshot() -> TestResult {
    let directory = Directory::new()?;
    let path = directory.0.join("input.cdx");
    fs::write(&path, HEADER)?;
    let loaded = read_cdx_file(&path, &Cancellation::default())?;
    let before = loaded.document.clone();
    fs::write(&path, b"not binary CDX")?;
    let error = read_cdx_file(&path, &Cancellation::default())
        .err()
        .ok_or("decoded malformed input")?;
    assert_eq!(error.code(), "invalid_document");
    assert_eq!(error.stage(), "decode");
    assert!(!error.published());
    let file = fs::File::create(&path)?;
    file.set_len(64 * 1024 * 1024 + 1)?;
    drop(file);
    assert_eq!(
        read_cdx_file(&path, &Cancellation::default())
            .err()
            .ok_or("read oversized input")?
            .code(),
        "file_too_large"
    );
    assert_eq!(
        read_cdx_file(&directory.0, &Cancellation::default())
            .err()
            .ok_or("read directory")?
            .code(),
        "not_regular_file"
    );
    assert_eq!(loaded.document, before);
    assert_eq!(loaded.document.export_cdx()?, HEADER);
    Ok(())
}

#[test]
fn cancellation_precedes_file_access_and_keeps_the_old_destination() -> TestResult {
    let directory = Directory::new()?;
    let source = directory.0.join("missing.cdx");
    let destination = directory.0.join("existing.cdx");
    fs::write(&destination, b"keep")?;
    let document = StoredDocument::from_cdx(LEGACY.into(), PageSize::A4)?;
    let cancel = Cancellation::default();
    cancel.cancel();
    let error = read_cdx_file(&source, &cancel)
        .err()
        .ok_or("ignored read cancellation")?;
    assert_eq!(error.code(), "cancelled");
    assert_eq!(error.stage(), "validate");
    let error = save_cdx_file(&destination, &document, true, &cancel)
        .err()
        .ok_or("ignored save cancellation")?;
    assert_eq!(error.code(), "cancelled");
    assert!(!error.published());
    assert_eq!(fs::read(&destination)?, b"keep");
    assert_eq!(fs::read_dir(&directory.0)?.count(), 1);
    Ok(())
}
