use super::super::{Failure, FileError, Stage, Temporary};
use std::{
    fs::{self, File, Metadata, OpenOptions},
    io,
    os::windows::fs::{MetadataExt, OpenOptionsExt},
    path::Path,
};
use windows_permissions::{
    constants::{SeObjectType, SecurityInformation},
    wrappers,
};

pub(in crate::files) fn make_private(_: &Path, file: &File) -> io::Result<()> {
    // The empty new file has no confidential bytes. Protect it before the first write;
    // ReplaceFile later merges the existing target's original DACL without ignore flags.
    let descriptor =
        wrappers::ConvertStringSecurityDescriptorToSecurityDescriptor("D:P(A;;FA;;;OW)")?;
    let acl = wrappers::GetSecurityDescriptorDacl(&descriptor)?
        .ok_or_else(|| io::Error::other("private temporary descriptor has no DACL"))?;
    let mut handle = file.try_clone()?;
    wrappers::SetSecurityInfo(
        &mut handle,
        SeObjectType::SE_FILE_OBJECT,
        SecurityInformation::Dacl | SecurityInformation::ProtectedDacl,
        None,
        None,
        Some(acl),
        None,
    )
}

pub(in crate::files) fn is_reparse_point(metadata: &Metadata) -> bool {
    metadata.file_attributes() & 0x400 != 0 // FILE_ATTRIBUTE_REPARSE_POINT
}

pub(in crate::files) fn open_regular(path: &Path) -> io::Result<File> {
    OpenOptions::new()
        .read(true)
        .custom_flags(0x0020_0000)
        .open(path) // FILE_FLAG_OPEN_REPARSE_POINT
}

pub(in crate::files) fn preserve_metadata(
    original_path: &Path,
    _: &Metadata,
    _: &Path,
    temporary: &File,
) -> io::Result<()> {
    let original = open_regular(original_path)?;
    let requested = SecurityInformation::Owner | SecurityInformation::Group;
    let old = wrappers::GetSecurityInfo(&original, SeObjectType::SE_FILE_OBJECT, requested)?;
    let new = wrappers::GetSecurityInfo(temporary, SeObjectType::SE_FILE_OBJECT, requested)?;
    let old_owner = wrappers::GetSecurityDescriptorOwner(&old)?;
    let new_owner = wrappers::GetSecurityDescriptorOwner(&new)?;
    let old_group = wrappers::GetSecurityDescriptorGroup(&old)?;
    let new_group = wrappers::GetSecurityDescriptorGroup(&new)?;
    let owners_match = old_owner
        .zip(new_owner)
        .is_some_and(|(old, new)| wrappers::EqualSid(old, new));
    let groups_match = old_group
        .zip(new_group)
        .is_some_and(|(old, new)| wrappers::EqualSid(old, new));
    if !owners_match || !groups_match {
        // ReplaceFile explicitly promises DACL preservation, not owner/group SID
        // preservation. Refuse rather than reinterpret an owner-rights ACE or request
        // ownership-changing privileges that ordinary application saves do not need.
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "cannot preserve a different Windows owner or group",
        ));
    }
    Ok(())
}

pub(in crate::files) fn identity(path: &Path) -> io::Result<super::Identity> {
    let flags = winsafe::co::FILE_SHARE::READ
        | winsafe::co::FILE_SHARE::WRITE
        | winsafe::co::FILE_SHARE::DELETE;
    let (file, _) = winsafe::HFILE::CreateFile(
        win_path(path)?,
        winsafe::co::GENERIC::default(),
        Some(flags),
        None,
        winsafe::co::DISPOSITION::OPEN_EXISTING,
        winsafe::co::FILE_ATTRIBUTE::NORMAL,
        Some(winsafe::co::FILE_FLAG::OPEN_REPARSE_POINT),
        None,
        None,
    )
    .map_err(|error| io::Error::from_raw_os_error(error.raw() as i32))?;
    let information = file
        .GetFileInformationByHandle()
        .map_err(|error| io::Error::from_raw_os_error(error.raw() as i32))?;
    Ok(super::Identity(
        u64::from(information.dwVolumeSerialNumber),
        information.nFileIndex(),
    ))
}

fn win_path(path: &Path) -> io::Result<&str> {
    path.to_str().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "Windows replacement requires a valid Unicode path",
        )
    })
}

struct BackupDirectory(std::path::PathBuf);

impl BackupDirectory {
    fn create(parent: &Path) -> io::Result<Self> {
        for _ in 0..128 {
            let mut reservation = Temporary::create(parent)?;
            reservation.remove()?;
            match fs::create_dir(&reservation.path) {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            }
            let directory = Self(reservation.path.clone());
            let handle = OpenOptions::new()
                .read(true)
                // WRITE_DAC | READ_CONTROL | FILE_READ_ATTRIBUTES and
                // FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT.
                .access_mode(0x0006_0080)
                .custom_flags(0x0220_0000)
                .open(&directory.0)?;
            let metadata = handle.metadata()?;
            if !metadata.is_dir() || is_reparse_point(&metadata) {
                return Err(io::Error::other(
                    "backup directory changed before protection",
                ));
            }
            make_private(&directory.0, &handle)?;
            // The protected directory denies new access by other principals. As with
            // inherited macOS ACLs, tightening cannot revoke an already-open handle.
            // Never overwrite an entry left before protection was applied.
            match fs::symlink_metadata(directory.original()) {
                Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(directory),
                Err(error) => return Err(error),
                Ok(_) => return Err(io::ErrorKind::AlreadyExists.into()),
            }
        }
        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "backup directory collision limit",
        ))
    }

    fn original(&self) -> std::path::PathBuf {
        self.0.join("original.cdxml")
    }
}

impl Drop for BackupDirectory {
    fn drop(&mut self) {
        // Never recursively remove this directory: an unresolved replacement may have
        // placed the only recoverable original inside it.
        let _ = fs::remove_dir(&self.0);
    }
}

pub(in crate::files) fn replace(from: &Path, to: &Path) -> Result<(), FileError> {
    let parent = to
        .parent()
        .ok_or_else(|| FileError::new(to, Stage::Validate, Failure::InvalidPath))?;
    let directory = BackupDirectory::create(parent)
        .map_err(|error| FileError::io(to, Stage::CreateTemporary, error))?;
    let backup = directory.original();
    let call = || -> io::Result<()> {
        winsafe::ReplaceFile(
            win_path(to)?,
            win_path(from)?,
            Some(win_path(&backup)?),
            winsafe::co::REPLACEFILE::default(),
        )
        .map_err(|error| io::Error::from_raw_os_error(error.raw() as i32))
    };
    match call() {
        Ok(()) => {
            // The new target is already visible. A cleanup failure must carry that fact.
            fs::remove_file(&backup).map_err(|error| {
                let mut failure = super::super::after_publication(to, Stage::Cleanup, error);
                failure.recovery_path = Some(backup.clone());
                failure
            })?;
            fs::remove_dir(&directory.0)
                .map_err(|error| super::super::after_publication(to, Stage::Cleanup, error))
        }
        Err(error) if error.raw_os_error() == Some(1177) => {
            // ERROR_UNABLE_TO_MOVE_REPLACEMENT_2: Windows moved the original to the
            // backup and left the new bytes at `from`. Restore the old file instead of
            // treating a failed ReplaceFile as an ordinary unpublished failure.
            Err(super::super::restore_original(to, &backup, error))
        }
        Err(error) => {
            let mut error = FileError::io(to, Stage::Publish, error);
            if backup.exists() {
                error.recovery_path = Some(backup);
            }
            Err(error)
        }
    }
}
