//! Explicitly selected native files. Parsing and source preservation stay in `StoredDocument`.

mod platform;
mod smiles;
pub use smiles::{
    SmilesConversionReceipt, convert_smiles_acyclic_to_cdxml_file,
    convert_smiles_components_to_cdxml_file, convert_smiles_simple_ring_to_cdxml_file,
    convert_smiles_to_cdxml_file, export_smiles_fragment_file, read_smiles_file, save_smiles_file,
};
#[cfg(test)]
mod tests;

use crate::mol::{MolTransform, Version};
use crate::sdf::{SdfDocument, SdfError};
use crate::smiles::{LayoutError, SmilesError, SmilesFileError, WriteError};
use crate::{DocumentError, StoredDocument};
use bondline_model::PageSize;
use bondline_render::Cancellation;
use std::{
    borrow::Cow,
    fs::{self, File, Metadata, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const MAX_BYTES: usize = bondline_cdxml::MAX_INPUT_BYTES;
const CHUNK: usize = 64 * 1024;
const MAX_PATH_BYTES: usize = 64 * 1024;
#[derive(Debug, Clone, Copy)]
enum ReadLimit {
    Native,
    Smiles,
}
impl ReadLimit {
    fn bytes(self) -> usize {
        match self {
            Self::Native => MAX_BYTES,
            Self::Smiles => crate::smiles::MAX_SMILES_BYTES,
        }
    }
}
impl std::fmt::Display for ReadLimit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Native => "64 MiB",
            Self::Smiles => "64 KiB SMILES",
        })
    }
}
static NEXT_TEMPORARY: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub struct LoadedFile<T = StoredDocument> {
    pub path: PathBuf,
    pub document: T,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveReceipt {
    pub path: PathBuf,
    pub byte_length: usize,
    /// False on platforms that cannot flush a parent directory with this implementation.
    pub directory_synced: bool,
}

/// An error after publication does not mean the old file was restored. A recovery path
/// identifies an original retained after a failed operating-system replacement/recovery.
#[derive(Debug, thiserror::Error)]
#[error("{failure} during {stage:?} for {path:?}")]
pub struct FileError {
    #[source]
    failure: Box<Failure>,
    stage: Stage,
    path: PathBuf,
    published: bool,
    recovery_path: Option<PathBuf>,
}

impl FileError {
    pub fn code(&self) -> &'static str {
        match self.failure.as_ref() {
            Failure::InvalidPath => "invalid_path",
            Failure::TooLarge(_) => "file_too_large",
            Failure::NotRegular => "not_regular_file",
            Failure::Symlink => "symlink_target",
            Failure::Cancelled => "cancelled",
            Failure::Changed => "file_changed",
            Failure::Document(error) => error.code(),
            Failure::Sdf(error) => error.code(),
            Failure::Smiles(error) => error.code(),
            Failure::SmilesFile(error) => error.code(),
            Failure::Layout(error) => error.code(),
            Failure::SmilesWrite(error) => error.code(),
            Failure::Io(error) => {
                #[cfg(unix)]
                if error.raw_os_error() == Some(rustix::io::Errno::ILSEQ.raw_os_error()) {
                    return "invalid_path";
                }
                match error.kind() {
                    io::ErrorKind::NotFound => "not_found",
                    io::ErrorKind::AlreadyExists => "already_exists",
                    io::ErrorKind::PermissionDenied | io::ErrorKind::ReadOnlyFilesystem => {
                        "permission_denied"
                    }
                    io::ErrorKind::InvalidInput | io::ErrorKind::InvalidFilename => "invalid_path",
                    io::ErrorKind::Unsupported => "unsupported_filesystem",
                    _ => "io_error",
                }
            }
            #[cfg(any(windows, test))]
            Failure::Recovery { .. } => "recovery_required",
        }
    }

    pub fn published(&self) -> bool {
        self.published
    }

    pub fn recovery_path(&self) -> Option<&Path> {
        self.recovery_path.as_deref()
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn stage(&self) -> &'static str {
        self.stage.name()
    }

    fn new(path: &Path, stage: Stage, failure: Failure) -> Self {
        Self {
            failure: Box::new(failure),
            stage,
            path: path.to_owned(),
            published: false,
            recovery_path: None,
        }
    }

    fn io(path: &Path, stage: Stage, source: io::Error) -> Self {
        Self::new(path, stage, Failure::Io(source))
    }
}

#[derive(Debug, thiserror::Error)]
enum Failure {
    #[error("invalid or overlong file path")]
    InvalidPath,
    #[error("file exceeds the {0} input limit")]
    TooLarge(ReadLimit),
    #[error("only ordinary files are supported")]
    NotRegular,
    #[error("saving through a symbolic link is not supported; choose its target explicitly")]
    Symlink,
    #[error("operation cancelled before publication")]
    Cancelled,
    #[error("destination changed while preparing the save; choose the destination again")]
    Changed,
    #[error(transparent)]
    Document(#[from] DocumentError),
    #[error(transparent)]
    Sdf(#[from] SdfError),
    #[error(transparent)]
    Smiles(#[from] SmilesError),
    #[error(transparent)]
    SmilesFile(#[from] SmilesFileError),
    #[error(transparent)]
    Layout(#[from] LayoutError),
    #[error(transparent)]
    SmilesWrite(#[from] WriteError),
    #[error(transparent)]
    Io(io::Error),
    #[cfg(any(windows, test))]
    #[error(
        "replacement failed ({publication}); cannot restore the retained original ({restoration})"
    )]
    Recovery {
        publication: io::Error,
        restoration: io::Error,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    Validate,
    Read,
    Decode,
    Encode,
    CreateTemporary,
    Write,
    Metadata,
    SyncFile,
    Publish,
    Cleanup,
    SyncDirectory,
    #[cfg(any(windows, test))]
    Recovery,
}

impl Stage {
    fn name(self) -> &'static str {
        match self {
            Self::Validate => "validate",
            Self::Read => "read",
            Self::Decode => "decode",
            Self::Encode => "encode",
            Self::CreateTemporary => "create_temporary",
            Self::Write => "write",
            Self::Metadata => "metadata",
            Self::SyncFile => "sync_file",
            Self::Publish => "publish",
            Self::Cleanup => "cleanup",
            Self::SyncDirectory => "sync_directory",
            #[cfg(any(windows, test))]
            Self::Recovery => "recovery",
        }
    }
}

fn check_cancel(path: &Path, stage: Stage, cancel: &Cancellation) -> Result<(), FileError> {
    if cancel.is_cancelled() {
        Err(FileError::new(path, stage, Failure::Cancelled))
    } else {
        Ok(())
    }
}

fn validate_path(path: &Path) -> Result<(), FileError> {
    let bytes = path.as_os_str().as_encoded_bytes();
    if bytes.is_empty() || bytes.len() > MAX_PATH_BYTES || bytes.contains(&0) {
        Err(FileError::new(path, Stage::Validate, Failure::InvalidPath))
    } else {
        Ok(())
    }
}

/// Read a canonical ordinary file into an owned complete source snapshot. Symlinks chosen
/// for reading are resolved; nonblocking/no-follow opens guard special files and leaf races.
pub fn read_cdxml_file(path: &Path, cancel: &Cancellation) -> Result<LoadedFile, FileError> {
    let (path, bytes) = read_source_bytes(path, cancel)?;
    let document = StoredDocument::from_cdxml(bytes.into(), PageSize::A4)
        .map_err(|error| FileError::new(&path, Stage::Decode, error.into()))?;
    check_cancel(&path, Stage::Decode, cancel)?;
    Ok(LoadedFile { path, document })
}

/// Read the supported elemental CDX view while retaining the complete original binary.
/// Unsupported projections fail before a host can replace its current document.
pub fn read_cdx_file(path: &Path, cancel: &Cancellation) -> Result<LoadedFile, FileError> {
    let (path, bytes) = read_source_bytes(path, cancel)?;
    let document = StoredDocument::from_cdx(bytes.into(), PageSize::A4)
        .map_err(|error| FileError::new(&path, Stage::Decode, error.into()))?;
    check_cancel(&path, Stage::Decode, cancel)?;
    Ok(LoadedFile { path, document })
}

/// Read a validated V2000/V3000 molecule through the same bounded native path.
/// The drawing frame is supplied by the host, never inferred from chemical coordinates.
pub fn read_mol_file(
    path: &Path,
    frame: MolTransform,
    cancel: &Cancellation,
) -> Result<LoadedFile, FileError> {
    let (path, bytes) = read_source_bytes(path, cancel)?;
    let document = StoredDocument::from_mol(bytes.into(), PageSize::A4, frame)
        .map_err(|error| FileError::new(&path, Stage::Decode, error.into()))?;
    check_cancel(&path, Stage::Decode, cancel)?;
    Ok(LoadedFile { path, document })
}

/// Validate the complete mixed-version SDfile without requiring molecular projection.
pub fn read_sdf_file(
    path: &Path,
    cancel: &Cancellation,
) -> Result<LoadedFile<SdfDocument>, FileError> {
    let (path, bytes) = read_source_bytes(path, cancel)?;
    let document = SdfDocument::parse(bytes.into())
        .map_err(|error| FileError::new(&path, Stage::Decode, error.into()))?;
    check_cancel(&path, Stage::Decode, cancel)?;
    Ok(LoadedFile { path, document })
}

fn read_source_bytes(path: &Path, cancel: &Cancellation) -> Result<(PathBuf, Vec<u8>), FileError> {
    read_source_bytes_with_limit(path, cancel, ReadLimit::Native)
}

fn read_source_bytes_with_limit(
    path: &Path,
    cancel: &Cancellation,
    limit: ReadLimit,
) -> Result<(PathBuf, Vec<u8>), FileError> {
    check_cancel(path, Stage::Validate, cancel)?;
    validate_path(path)?;
    let path = fs::canonicalize(path).map_err(|error| FileError::io(path, Stage::Read, error))?;
    let metadata =
        fs::symlink_metadata(&path).map_err(|error| FileError::io(&path, Stage::Read, error))?;
    validate_regular(&path, &metadata)?;
    if metadata.len() > limit.bytes() as u64 {
        return Err(FileError::new(&path, Stage::Read, Failure::TooLarge(limit)));
    }
    let mut file =
        platform::open_regular(&path).map_err(|error| FileError::io(&path, Stage::Read, error))?;
    let metadata = file
        .metadata()
        .map_err(|error| FileError::io(&path, Stage::Read, error))?;
    validate_regular(&path, &metadata)?;
    if metadata.len() > limit.bytes() as u64 {
        return Err(FileError::new(&path, Stage::Read, Failure::TooLarge(limit)));
    }
    let bytes = read_bounded_bytes(&path, &mut file, cancel, limit)?;
    check_cancel(&path, Stage::Decode, cancel)?;
    Ok((path, bytes))
}

fn read_bounded_bytes(
    path: &Path,
    reader: &mut impl Read,
    cancel: &Cancellation,
    limit: ReadLimit,
) -> Result<Vec<u8>, FileError> {
    let mut bytes = Vec::new();
    let mut chunk = [0_u8; CHUNK];
    loop {
        check_cancel(path, Stage::Read, cancel)?;
        let count = reader
            .read(&mut chunk)
            .map_err(|error| FileError::io(path, Stage::Read, error))?;
        if count == 0 {
            break;
        }
        let needed = bytes
            .len()
            .checked_add(count)
            .filter(|needed| *needed <= limit.bytes())
            .ok_or_else(|| FileError::new(path, Stage::Read, Failure::TooLarge(limit)))?;
        if needed > bytes.capacity() {
            // Retain amortized growth without requesting capacity beyond the active source budget.
            let capacity = bytes
                .capacity()
                .saturating_mul(2)
                .max(needed)
                .min(limit.bytes());
            bytes
                .try_reserve_exact(capacity.saturating_sub(bytes.len()))
                .map_err(|error| FileError::io(path, Stage::Read, io::Error::other(error)))?;
        }
        bytes.extend_from_slice(chunk.get(..count).ok_or_else(|| {
            FileError::io(path, Stage::Read, io::Error::other("invalid read length"))
        })?);
    }
    Ok(bytes)
}

fn validate_regular(path: &Path, metadata: &Metadata) -> Result<(), FileError> {
    if !metadata.is_file() || platform::is_reparse_point(metadata) {
        Err(FileError::new(path, Stage::Validate, Failure::NotRegular))
    } else {
        Ok(())
    }
}

/// Encode the complete stored document, sync a same-directory temporary, then atomically
/// publish. The default no-clobber path uses a hard-link create, never an existence check
/// followed by an overwriting rename. An unsupported filesystem fails without publication.
pub fn save_cdxml_file(
    path: &Path,
    document: &StoredDocument,
    overwrite: bool,
    cancel: &Cancellation,
) -> Result<SaveReceipt, FileError> {
    save_with_hook(path, document, overwrite, cancel, &mut |_| Ok(()))
}

/// Publish only exact unchanged CDX backing through the shared atomic save path.
pub fn save_cdx_file(
    path: &Path,
    document: &StoredDocument,
    overwrite: bool,
    cancel: &Cancellation,
) -> Result<SaveReceipt, FileError> {
    save_format_with_hook(
        path,
        document,
        overwrite,
        cancel,
        Encoding::Cdx,
        &mut |_| Ok(()),
    )
}

/// Publish every current record and data byte through the shared atomic save path.
pub fn save_sdf_file(
    path: &Path,
    document: &SdfDocument,
    overwrite: bool,
    cancel: &Cancellation,
) -> Result<SaveReceipt, FileError> {
    save_encoded_with_hook(
        path,
        overwrite,
        cancel,
        || Ok(Cow::Borrowed(document.as_bytes())),
        &mut |_| Ok(()),
    )
}

/// Save complete MOL backing or a guarded supported graph using the established
/// atomic/security-preserving publication path. Imported MOL keeps its captured frame.
pub fn save_mol_file(
    path: &Path,
    document: &StoredDocument,
    overwrite: bool,
    native_frame: MolTransform,
    cancel: &Cancellation,
) -> Result<SaveReceipt, FileError> {
    save_format_with_hook(
        path,
        document,
        overwrite,
        cancel,
        Encoding::Mol(native_frame),
        &mut |_| Ok(()),
    )
}

/// Explicit MOL encoding through the same atomic/security-preserving path.
pub fn save_mol_file_as(
    path: &Path,
    document: &StoredDocument,
    overwrite: bool,
    version: Version,
    native_frame: MolTransform,
    cancel: &Cancellation,
) -> Result<SaveReceipt, FileError> {
    save_format_with_hook(
        path,
        document,
        overwrite,
        cancel,
        Encoding::MolAs(version, native_frame),
        &mut |_| Ok(()),
    )
}

#[derive(Clone, Copy)]
enum Encoding {
    Cdxml,
    Cdx,
    Mol(MolTransform),
    MolAs(Version, MolTransform),
}

/// Resolve only the chosen parent, retaining the final leaf for no-follow/no-clobber
/// validation. Adapters may validate their path encoding before any file is published.
pub fn canonical_save_path(chosen: &Path) -> Result<PathBuf, FileError> {
    validate_path(chosen)?;
    let filename = chosen
        .file_name()
        .ok_or_else(|| FileError::new(chosen, Stage::Validate, Failure::InvalidPath))?;
    let parent = chosen
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let parent =
        fs::canonicalize(parent).map_err(|error| FileError::io(chosen, Stage::Validate, error))?;
    let path = parent.join(filename);
    validate_path(&path)?;
    Ok(path)
}

fn save_with_hook(
    chosen: &Path,
    document: &StoredDocument,
    overwrite: bool,
    cancel: &Cancellation,
    hook: &mut impl FnMut(Stage) -> io::Result<()>,
) -> Result<SaveReceipt, FileError> {
    save_format_with_hook(chosen, document, overwrite, cancel, Encoding::Cdxml, hook)
}

fn save_format_with_hook(
    chosen: &Path,
    document: &StoredDocument,
    overwrite: bool,
    cancel: &Cancellation,
    encoding: Encoding,
    hook: &mut impl FnMut(Stage) -> io::Result<()>,
) -> Result<SaveReceipt, FileError> {
    save_encoded_with_hook(
        chosen,
        overwrite,
        cancel,
        || {
            match encoding {
                Encoding::Cdxml => document.to_cdxml(),
                Encoding::Cdx => document.to_cdx(),
                Encoding::Mol(frame) => document.to_mol(frame),
                Encoding::MolAs(version, frame) => document.to_mol_as(version, frame),
            }
            .map_err(Failure::from)
        },
        hook,
    )
}

fn save_encoded_with_hook<'a>(
    chosen: &Path,
    overwrite: bool,
    cancel: &Cancellation,
    encode: impl FnOnce() -> Result<Cow<'a, [u8]>, Failure>,
    hook: &mut impl FnMut(Stage) -> io::Result<()>,
) -> Result<SaveReceipt, FileError> {
    check_cancel(chosen, Stage::Validate, cancel)?;
    let path = canonical_save_path(chosen)?;
    let parent = path
        .parent()
        .ok_or_else(|| FileError::new(chosen, Stage::Validate, Failure::InvalidPath))?;
    let original = destination_metadata(&path, overwrite)?;
    let identity = original
        .as_ref()
        .map(|_| platform::identity(&path))
        .transpose()
        .map_err(|error| FileError::io(&path, Stage::Validate, error))?;
    let bytes = encode().map_err(|error| FileError::new(&path, Stage::Encode, error))?;
    if bytes.len() > MAX_BYTES {
        return Err(FileError::new(
            &path,
            Stage::Encode,
            Failure::TooLarge(ReadLimit::Native),
        ));
    }
    check_cancel(&path, Stage::CreateTemporary, cancel)?;
    hook(Stage::CreateTemporary)
        .map_err(|error| FileError::io(&path, Stage::CreateTemporary, error))?;
    let mut temporary = Temporary::create(parent)
        .map_err(|error| FileError::io(&path, Stage::CreateTemporary, error))?;
    platform::make_private(&temporary.path, temporary.file()?)
        .map_err(|error| FileError::io(&path, Stage::Metadata, error))?;
    for chunk in bytes.chunks(CHUNK) {
        check_cancel(&path, Stage::Write, cancel)?;
        hook(Stage::Write).map_err(|error| FileError::io(&path, Stage::Write, error))?;
        temporary
            .file()?
            .write_all(chunk)
            .map_err(|error| FileError::io(&path, Stage::Write, error))?;
    }
    hook(Stage::Metadata).map_err(|error| FileError::io(&path, Stage::Metadata, error))?;
    if let Some(metadata) = &original {
        platform::preserve_metadata(&path, metadata, &temporary.path, temporary.file()?)
            .map_err(|error| FileError::io(&path, Stage::Metadata, error))?;
    }
    hook(Stage::SyncFile).map_err(|error| FileError::io(&path, Stage::SyncFile, error))?;
    platform::sync_file(temporary.file()?)
        .map_err(|error| FileError::io(&path, Stage::SyncFile, error))?;
    temporary.close();
    check_cancel(&path, Stage::Publish, cancel)?;
    hook(Stage::Publish).map_err(|error| FileError::io(&path, Stage::Publish, error))?;
    // Refuse a leaf that changed to a symlink/read-only/special file while encoding.
    // The identity check is not compare-and-swap: another process can still replace the
    // destination after the check. Only no-clobber publication arbitrates that race.
    // No-clobber publication needs no second existence check: hard_link is the arbiter.
    if overwrite {
        let current = destination_metadata(&path, true)?;
        if let Some(expected) = identity {
            let actual = current
                .as_ref()
                .map(|_| platform::identity(&path))
                .transpose()
                .map_err(|error| FileError::io(&path, Stage::Publish, error))?;
            if actual != Some(expected)
                || original
                    .as_ref()
                    .zip(current.as_ref())
                    .is_some_and(|(before, after)| !platform::security_unchanged(before, after))
            {
                return Err(FileError::new(&path, Stage::Publish, Failure::Changed));
            }
        }
    }
    check_cancel(&path, Stage::Publish, cancel)?;
    if overwrite && original.is_some() {
        platform::replace(&temporary.path, &path)?;
        temporary.published();
    } else {
        fs::hard_link(&temporary.path, &path)
            .map_err(|error| FileError::io(&path, Stage::Publish, error))?;
    }
    // Cancellation past this point cannot undo publication. Finish cleanup and durability.
    if let Err(error) = hook(Stage::Cleanup).and_then(|()| temporary.remove()) {
        return Err(after_publication(&path, Stage::Cleanup, error));
    }
    hook(Stage::SyncDirectory)
        .map_err(|error| after_publication(&path, Stage::SyncDirectory, error))?;
    let directory_synced = platform::sync_directory(parent)
        .map_err(|error| after_publication(&path, Stage::SyncDirectory, error))?;
    Ok(SaveReceipt {
        path,
        byte_length: bytes.len(),
        directory_synced,
    })
}

fn after_publication(path: &Path, stage: Stage, error: io::Error) -> FileError {
    let mut error = FileError::io(path, stage, error);
    error.published = true;
    error
}

#[cfg(any(windows, test))]
fn restore_original(path: &Path, backup: &Path, publication: io::Error) -> FileError {
    // The operating system has moved the original to backup. A no-clobber link restores
    // its complete inode/streams/ACLs without replacing a concurrent creator at the target.
    if let Err(restoration) = fs::hard_link(backup, path) {
        let mut error = FileError::new(
            path,
            Stage::Recovery,
            Failure::Recovery {
                publication,
                restoration,
            },
        );
        error.recovery_path = Some(backup.to_owned());
        return error;
    }
    if let Err(cleanup) = fs::remove_file(backup) {
        let mut error = FileError::io(path, Stage::Cleanup, cleanup);
        error.recovery_path = Some(backup.to_owned());
        return error;
    }
    FileError::io(path, Stage::Publish, publication)
}

fn destination_metadata(path: &Path, overwrite: bool) -> Result<Option<Metadata>, FileError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || platform::is_reparse_point(&metadata) {
                return Err(FileError::new(path, Stage::Validate, Failure::Symlink));
            }
            validate_regular(path, &metadata)?;
            if !overwrite {
                return Err(FileError::io(
                    path,
                    Stage::Validate,
                    io::ErrorKind::AlreadyExists.into(),
                ));
            }
            if metadata.permissions().readonly() {
                return Err(FileError::io(
                    path,
                    Stage::Validate,
                    io::ErrorKind::PermissionDenied.into(),
                ));
            }
            Ok(Some(metadata))
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(FileError::io(path, Stage::Validate, error)),
    }
}

struct Temporary {
    path: PathBuf,
    file: Option<File>,
    owned: bool,
}

impl Temporary {
    fn create(parent: &Path) -> io::Result<Self> {
        for _ in 0..128 {
            let id = NEXT_TEMPORARY
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                    value.checked_add(1)
                })
                .map_err(|_| io::Error::other("temporary identity exhausted"))?;
            let path = parent.join(format!("bondline-{}-{id}.tmp", std::process::id()));
            let mut options = OpenOptions::new();
            options.read(true).write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            #[cfg(windows)]
            {
                use std::os::windows::fs::OpenOptionsExt;
                // GENERIC_READ | GENERIC_WRITE | WRITE_DAC: the security wrapper needs
                // WRITE_DAC on the existing handle before any confidential bytes land.
                options.access_mode(0xc004_0000).share_mode(0);
            }
            match options.open(&path) {
                Ok(file) => {
                    return Ok(Self {
                        path,
                        file: Some(file),
                        owned: true,
                    });
                }
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                Err(error) => return Err(error),
            }
        }
        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "temporary name collision limit",
        ))
    }

    fn file(&self) -> Result<&File, FileError> {
        self.file.as_ref().ok_or_else(|| {
            FileError::io(
                &self.path,
                Stage::Write,
                io::Error::other("temporary is closed"),
            )
        })
    }

    fn close(&mut self) {
        self.file.take();
    }
    fn published(&mut self) {
        self.owned = false;
    }
    fn remove(&mut self) -> io::Result<()> {
        self.close();
        if self.owned {
            fs::remove_file(&self.path)?;
            self.owned = false;
        }
        Ok(())
    }
}

impl Drop for Temporary {
    fn drop(&mut self) {
        let _ = self.remove();
    }
}
