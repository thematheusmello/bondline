#![cfg(not(target_arch = "wasm32"))]

use bondline_io::{
    StoredDocument,
    files::{read_cdxml_file, save_cdxml_file},
};
use bondline_model::{PageId, PageSize};
use bondline_render::Cancellation;
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

const SOURCE: &[u8] = include_bytes!("fixtures/bondline-v0.cdxml");
static NEXT: AtomicU64 = AtomicU64::new(0);

struct Directory(PathBuf);

impl Directory {
    fn new() -> std::io::Result<Self> {
        let path = std::env::temp_dir().join(format!(
            "bondline-file-tests-{}-{}",
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
fn native_files_load_owned_complete_source_and_reject_invalid_or_oversized_inputs()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = Directory::new()?;
    let path = directory.0.join("sample.cdxml");
    fs::write(&path, SOURCE)?;
    let loaded = read_cdxml_file(&path, &Cancellation::default())?;
    assert_eq!(loaded.path, fs::canonicalize(&path)?);
    assert!(loaded.document.is_fully_mapped());
    assert_eq!(loaded.document.to_cdxml()?.as_ref(), SOURCE);
    fs::write(&path, b"<CDXML><broken></CDXML>")?;
    assert_eq!(
        read_cdxml_file(&path, &Cancellation::default())
            .err()
            .ok_or("accepted invalid XML")?
            .code(),
        "invalid_document"
    );
    let sparse = fs::File::create(&path)?;
    sparse.set_len(bondline_cdxml::MAX_INPUT_BYTES as u64 + 1)?;
    drop(sparse);
    assert_eq!(
        read_cdxml_file(&path, &Cancellation::default())
            .err()
            .ok_or("accepted oversized file")?
            .code(),
        "file_too_large"
    );
    assert_eq!(
        read_cdxml_file(&directory.0, &Cancellation::default())
            .err()
            .ok_or("accepted directory")?
            .code(),
        "not_regular_file"
    );
    assert_eq!(
        read_cdxml_file(&directory.0.join("missing.cdxml"), &Cancellation::default())
            .err()
            .ok_or("accepted missing file")?
            .code(),
        "not_found"
    );
    assert_eq!(
        loaded.document.to_cdxml()?.as_ref(),
        SOURCE,
        "failed reads cannot alter a previously loaded snapshot"
    );
    Ok(())
}

#[test]
fn fresh_save_preserves_complete_bytes_and_existing_targets_are_not_clobbered_by_default()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = Directory::new()?;
    let target = directory.0.join("copy.cdxml");
    let document = StoredDocument::from_cdxml(SOURCE.into(), PageSize::A4)?;
    let saved = save_cdxml_file(&target, &document, false, &Cancellation::default())?;
    assert_eq!(saved.path, fs::canonicalize(&target)?);
    assert_eq!(saved.byte_length, SOURCE.len());
    assert_eq!(saved.directory_synced, cfg!(unix));
    assert_eq!(fs::read(&target)?, SOURCE);
    fs::write(&target, b"an existing file")?;
    let error = save_cdxml_file(&target, &document, false, &Cancellation::default())
        .err()
        .ok_or("clobbered file")?;
    assert_eq!(error.code(), "already_exists");
    assert!(!error.published());
    assert_eq!(fs::read(&target)?, b"an existing file");
    assert_eq!(
        fs::read_dir(&directory.0)?.count(),
        1,
        "no temporary files should remain"
    );
    Ok(())
}

#[test]
fn replacement_contains_the_edited_model_and_preserves_private_permissions()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = Directory::new()?;
    let target = directory.0.join("replace.cdxml");
    fs::write(&target, b"old valid data")?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&target, fs::Permissions::from_mode(0o600))?;
    }
    let mut document = StoredDocument::from_cdxml(SOURCE.into(), PageSize::A4)?;
    document
        .model_mut()
        .set_page_size(PageId::new(1).ok_or("page")?, PageSize::new(400.0, 600.0)?)?;
    let expected = document.to_cdxml()?.into_owned();
    let saved = save_cdxml_file(&target, &document, true, &Cancellation::default())?;
    assert_eq!(saved.byte_length, expected.len());
    assert_eq!(fs::read(&target)?, expected);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(fs::metadata(&target)?.permissions().mode() & 0o777, 0o600);
    }
    assert_eq!(fs::read_dir(&directory.0)?.count(), 1);
    Ok(())
}

#[test]
fn cancelled_and_unrepresentable_saves_leave_destination_bytes_intact()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = Directory::new()?;
    let target = directory.0.join("unchanged.cdxml");
    fs::write(&target, b"old data")?;
    let document = StoredDocument::from_cdxml(SOURCE.into(), PageSize::A4)?;
    let cancel = Cancellation::default();
    cancel.cancel();
    let error = save_cdxml_file(&target, &document, true, &cancel)
        .err()
        .ok_or("ignored cancellation")?;
    assert_eq!(error.code(), "cancelled");
    assert!(!error.published());
    assert_eq!(
        read_cdxml_file(&target, &cancel)
            .err()
            .ok_or("ignored cancellation")?
            .code(),
        "cancelled"
    );
    let mut opaque = StoredDocument::from_cdxml(
        include_bytes!("../../cdxml/tests/fixtures/unknown.cdxml")
            .as_slice()
            .into(),
        PageSize::A4,
    )?;
    opaque
        .model_mut()
        .set_page_size(PageId::new(1).ok_or("page")?, PageSize::new(72.0, 144.0)?)?;
    assert_eq!(
        save_cdxml_file(&target, &opaque, true, &Cancellation::default())
            .err()
            .ok_or("discarded unknown source")?
            .code(),
        "unsupported_content"
    );
    assert_eq!(fs::read(&target)?, b"old data");
    assert_eq!(fs::read_dir(&directory.0)?.count(), 1);
    Ok(())
}

#[test]
fn read_only_targets_are_refused_before_writing() -> Result<(), Box<dyn std::error::Error>> {
    let directory = Directory::new()?;
    let target = directory.0.join("readonly.cdxml");
    fs::write(&target, b"old data")?;
    let original_permissions = fs::metadata(&target)?.permissions();
    let mut permissions = original_permissions.clone();
    permissions.set_readonly(true);
    fs::set_permissions(&target, permissions)?;
    let document = StoredDocument::from_cdxml(SOURCE.into(), PageSize::A4)?;
    let error = save_cdxml_file(&target, &document, true, &Cancellation::default())
        .err()
        .ok_or("replaced readonly target")?;
    assert_eq!(error.code(), "permission_denied");
    assert!(!error.published());
    assert_eq!(fs::read(&target)?, b"old data");
    #[cfg(windows)]
    {
        fs::set_permissions(&target, original_permissions)?;
    }
    Ok(())
}

#[cfg(unix)]
#[test]
fn reading_a_symlink_resolves_its_target_but_save_as_does_not_replace_the_link()
-> Result<(), Box<dyn std::error::Error>> {
    use std::os::unix::fs::symlink;
    let directory = Directory::new()?;
    let target = directory.0.join("target.cdxml");
    let link = directory.0.join("link.cdxml");
    fs::write(&target, SOURCE)?;
    symlink(&target, &link)?;
    let loaded = read_cdxml_file(&link, &Cancellation::default())?;
    assert_eq!(loaded.path, fs::canonicalize(&target)?);
    let error = save_cdxml_file(&link, &loaded.document, true, &Cancellation::default())
        .err()
        .ok_or("replaced symlink")?;
    assert_eq!(error.code(), "symlink_target");
    assert!(fs::symlink_metadata(&link)?.file_type().is_symlink());
    assert_eq!(fs::read(&target)?, SOURCE);
    Ok(())
}

#[test]
fn malformed_paths_fail_before_accessing_the_filesystem() -> Result<(), Box<dyn std::error::Error>>
{
    use std::path::Path;
    for path in [
        PathBuf::new(),
        PathBuf::from("embedded\0nul"),
        PathBuf::from("a".repeat(64 * 1024 + 1)),
    ] {
        assert_eq!(
            read_cdxml_file(&path, &Cancellation::default())
                .err()
                .ok_or("accepted invalid read path")?
                .code(),
            "invalid_path"
        );
        assert_eq!(
            save_cdxml_file(
                &path,
                &StoredDocument::default(),
                false,
                &Cancellation::default()
            )
            .err()
            .ok_or("accepted invalid save path")?
            .code(),
            "invalid_path"
        );
    }
    assert_eq!(
        save_cdxml_file(
            Path::new("/"),
            &StoredDocument::default(),
            false,
            &Cancellation::default()
        )
        .err()
        .ok_or("accepted directory destination")?
        .code(),
        "invalid_path"
    );
    Ok(())
}

#[cfg(unix)]
#[test]
fn native_paths_work_and_fifos_are_rejected_without_opening_a_stream()
-> Result<(), Box<dyn std::error::Error>> {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt};
    let directory = Directory::new()?;
    // APFS requires well-formed Unicode; Linux filesystems also support arbitrary bytes.
    let filename = if cfg!(target_os = "macos") {
        OsString::from("native-β.cdxml")
    } else {
        OsString::from_vec(b"native-\xff.cdxml".to_vec())
    };
    let target = directory.0.join(filename);
    let document = StoredDocument::from_cdxml(SOURCE.into(), PageSize::A4)?;
    save_cdxml_file(&target, &document, false, &Cancellation::default())?;
    assert_eq!(
        read_cdxml_file(&target, &Cancellation::default())?
            .document
            .to_cdxml()?
            .as_ref(),
        SOURCE
    );
    let fifo = directory.0.join("fifo");
    let status = std::process::Command::new("mkfifo").arg(&fifo).status()?;
    assert!(
        status.success(),
        "the POSIX mkfifo test fixture must be created"
    );
    assert_eq!(
        read_cdxml_file(&fifo, &Cancellation::default())
            .err()
            .ok_or("accepted named pipe")?
            .code(),
        "not_regular_file"
    );
    assert_eq!(
        save_cdxml_file(&fifo, &document, true, &Cancellation::default())
            .err()
            .ok_or("replaced named pipe")?
            .code(),
        "not_regular_file"
    );
    Ok(())
}
