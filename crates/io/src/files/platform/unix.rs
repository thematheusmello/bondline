use rustix::{
    fs::{self as sys, Mode, OFlags, XattrFlags},
    io::Errno,
};
#[cfg(target_os = "macos")]
use std::fs;
use std::{
    ffi::CStr,
    fs::{File, Metadata},
    io,
    os::unix::fs::MetadataExt,
    path::Path,
};

const MAX_ATTRIBUTE_NAMES: usize = 1024 * 1024;
const MAX_ATTRIBUTE_VALUE: usize = 16 * 1024 * 1024;
const MAX_ATTRIBUTES_TOTAL: usize = 32 * 1024 * 1024;

pub(in crate::files) fn open_regular(path: &Path) -> io::Result<File> {
    let descriptor = sys::open(
        path,
        OFlags::RDONLY | OFlags::NONBLOCK | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )?;
    Ok(File::from(descriptor))
}

pub(in crate::files) fn identity(path: &Path) -> io::Result<super::Identity> {
    let metadata = open_regular(path)?.metadata()?;
    Ok(super::Identity(metadata.dev(), metadata.ino()))
}

pub(in crate::files) fn security_unchanged(before: &Metadata, after: &Metadata) -> bool {
    before.mode() == after.mode()
        && before.uid() == after.uid()
        && before.gid() == after.gid()
        && before.ctime() == after.ctime()
        && before.ctime_nsec() == after.ctime_nsec()
}

pub(in crate::files) fn sync_file(file: &File) -> io::Result<()> {
    match file.sync_all() {
        Ok(()) => Ok(()),
        // Some macOS network filesystems cannot perform F_FULLFSYNC. A plain fsync is
        // still a real flush; other errors must propagate instead of claiming durability.
        #[cfg(target_os = "macos")]
        Err(error) if matches!(error.raw_os_error(), Some(code) if code == Errno::NOTSUP.raw_os_error() || code == Errno::NOTTY.raw_os_error()) => {
            sys::fsync(file).map_err(Into::into)
        }
        Err(error) => Err(error),
    }
}

pub(in crate::files) fn sync_directory(parent: &Path) -> io::Result<bool> {
    let directory = File::open(parent)?;
    sys::fsync(&directory)?;
    Ok(true)
}

pub(in crate::files) fn make_private(path: &Path, file: &File) -> io::Result<()> {
    // Clear inherited extended ACLs before writing any document bytes. The creation mode
    // already excludes group/other access; an inherited macOS allow ACE can override it.
    #[cfg(target_os = "macos")]
    exacl::setfacl(&[path], &[], None)?;
    #[cfg(target_os = "linux")]
    remove_attribute(file, c"system.posix_acl_access")?;
    let _ = path;
    sys::fchmod(file, Mode::from_raw_mode(0o600))?;
    Ok(())
}

pub(in crate::files) fn preserve_metadata(
    original_path: &Path,
    expected: &Metadata,
    temporary_path: &Path,
    temporary: &File,
) -> io::Result<()> {
    let original = open_regular(original_path)?;
    let actual = original.metadata()?;
    if actual.dev() != expected.dev() || actual.ino() != expected.ino() {
        return Err(io::Error::other(
            "destination changed before metadata preservation",
        ));
    }
    let new_metadata = temporary.metadata()?;
    // Owner and group are part of the access policy. Never apply somebody else's mode
    // bits to a temporary that belongs to a different owner/group.
    if new_metadata.uid() != actual.uid() || new_metadata.gid() != actual.gid() {
        sys::fchown(
            temporary,
            Some(rustix::fs::Uid::from_raw(actual.uid())),
            Some(rustix::fs::Gid::from_raw(actual.gid())),
        )?;
    }
    copy_attributes(&original, temporary)?;
    #[cfg(target_os = "macos")]
    {
        use std::os::macos::fs::FileTimesExt;
        temporary.set_times(fs::FileTimes::new().set_created(actual.created()?))?;
        let acl = exacl::getfacl(original_path, None)?;
        exacl::setfacl(&[temporary_path], &acl, None)?;
    }
    #[cfg(target_os = "linux")]
    copy_access_acl(&original, temporary)?;
    let _ = temporary_path;
    // chown clears set-id bits and ACL writes may adjust modes: restore mode last.
    temporary.set_permissions(actual.permissions())?;
    Ok(())
}

fn names(file: &File) -> io::Result<Vec<u8>> {
    let length = match sys::flistxattr_query(file) {
        Ok(length) => length,
        Err(Errno::NOTSUP) => return Ok(Vec::new()),
        Err(error) => return Err(error.into()),
    };
    if length > MAX_ATTRIBUTE_NAMES {
        return Err(io::Error::other("extended attribute names exceed 1 MiB"));
    }
    if length == 0 {
        return Ok(Vec::new());
    }
    let mut bytes = vec![0; length];
    let length = sys::flistxattr(file, bytes.as_mut_slice())?;
    bytes.truncate(length);
    Ok(bytes)
}

fn attribute(file: &File, name: &CStr) -> io::Result<Vec<u8>> {
    let length = sys::fgetxattr_query(file, name)?;
    if length > MAX_ATTRIBUTE_VALUE {
        return Err(io::Error::other("extended attribute value exceeds 16 MiB"));
    }
    let mut bytes = vec![0; length];
    if length != 0 {
        let length = sys::fgetxattr(file, name, bytes.as_mut_slice())?;
        bytes.truncate(length);
    }
    Ok(bytes)
}

fn copy_attributes(original: &File, temporary: &File) -> io::Result<()> {
    let names = names(original)?;
    let mut total = 0_usize;
    for raw in names.split_inclusive(|byte| *byte == 0) {
        let name = CStr::from_bytes_with_nul(raw).map_err(io::Error::other)?;
        let value = attribute(original, name)?;
        total = total.saturating_add(value.len());
        if total > MAX_ATTRIBUTES_TOTAL {
            return Err(io::Error::other("extended attributes exceed 32 MiB"));
        }
        // Fail closed when security/quarantine/resource metadata cannot be preserved.
        // No best-effort ACL or plain-rename fallback may silently widen access.
        sys::fsetxattr(temporary, name, &value, XattrFlags::empty())?;
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn absent_attribute(error: Errno) -> bool {
    matches!(error, Errno::NODATA | Errno::NOTSUP)
}

#[cfg(target_os = "linux")]
fn remove_attribute(file: &File, name: &CStr) -> io::Result<()> {
    match sys::fremovexattr(file, name) {
        Ok(()) => Ok(()),
        Err(error) if absent_attribute(error) => Ok(()),
        Err(error) => Err(error.into()),
    }
}

#[cfg(target_os = "linux")]
fn copy_access_acl(original: &File, temporary: &File) -> io::Result<()> {
    // Linux listxattr need not expose this name. Read the kernel ACL bytes directly;
    // using exacl on Linux would link LGPL libacl into the shipped application.
    let name = c"system.posix_acl_access";
    match attribute(original, name) {
        Ok(bytes) => {
            sys::fsetxattr(temporary, name, &bytes, XattrFlags::empty()).map_err(Into::into)
        }
        Err(error)
            if error
                .raw_os_error()
                .is_some_and(|raw| absent_attribute(Errno::from_raw_os_error(raw))) =>
        {
            remove_attribute(temporary, name)
        }
        Err(error) => Err(error),
    }
}
