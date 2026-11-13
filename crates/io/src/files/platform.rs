#[cfg(not(windows))]
use super::{FileError, Stage};
#[cfg(not(windows))]
use std::fs;
#[cfg(not(unix))]
use std::{fs::File, io};
use std::{fs::Metadata, path::Path};

#[cfg(unix)]
mod unix;
#[cfg(unix)]
pub(super) use unix::*;
#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub(super) use windows::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct Identity(pub u64, pub u64);

#[cfg(not(unix))]
pub(super) fn security_unchanged(_: &Metadata, _: &Metadata) -> bool {
    // ReplaceFile itself preserves the current Windows target's DACL/security attributes.
    true
}

#[cfg(not(windows))]
pub(super) fn is_reparse_point(_: &Metadata) -> bool {
    false
}

#[cfg(not(windows))]
pub(super) fn replace(from: &Path, to: &Path) -> Result<(), FileError> {
    fs::rename(from, to).map_err(|error| FileError::io(to, Stage::Publish, error))
}

#[cfg(not(any(unix, windows)))]
pub(super) fn open_regular(path: &Path) -> io::Result<File> {
    File::open(path)
}
#[cfg(not(any(unix, windows)))]
pub(super) fn identity(_: &Path) -> io::Result<Identity> {
    Err(io::ErrorKind::Unsupported.into())
}
#[cfg(not(unix))]
pub(super) fn sync_file(file: &File) -> io::Result<()> {
    file.sync_all()
}
#[cfg(not(unix))]
pub(super) fn sync_directory(_: &Path) -> io::Result<bool> {
    Ok(false)
}
#[cfg(not(any(unix, windows)))]
pub(super) fn make_private(_: &Path, _: &File) -> io::Result<()> {
    Ok(())
}
#[cfg(not(any(unix, windows)))]
pub(super) fn preserve_metadata(_: &Path, _: &Metadata, _: &Path, _: &File) -> io::Result<()> {
    Ok(())
}
