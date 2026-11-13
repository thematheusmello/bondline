use super::*;

struct Directory(PathBuf);
impl Directory {
    fn new() -> io::Result<Self> {
        let id = NEXT_TEMPORARY.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("bondline-atomic-tests-{}-{id}", std::process::id()));
        fs::create_dir(&path)?;
        Ok(Self(path))
    }
    fn target(&self) -> PathBuf {
        self.0.join("document.cdxml")
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn cancellation_at_the_publication_boundary_preserves_the_original()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = Directory::new()?;
    let path = directory.target();
    fs::write(&path, b"original")?;
    let cancel = Cancellation::default();
    let result = save_with_hook(
        &path,
        &StoredDocument::default(),
        true,
        &cancel,
        &mut |stage| {
            if stage == Stage::Publish {
                cancel.cancel();
            }
            Ok(())
        },
    );
    let error = result.err().ok_or("published after cancellation")?;
    assert_eq!(error.code(), "cancelled");
    assert!(!error.published());
    assert_eq!(fs::read(&path)?, b"original");
    assert_eq!(fs::read_dir(&directory.0)?.count(), 1);
    Ok(())
}

#[test]
fn filesystem_failures_before_publication_preserve_old_bytes_and_remove_temporaries()
-> Result<(), Box<dyn std::error::Error>> {
    for failure_stage in [
        Stage::CreateTemporary,
        Stage::Write,
        Stage::Metadata,
        Stage::SyncFile,
        Stage::Publish,
    ] {
        let directory = Directory::new()?;
        let path = directory.target();
        fs::write(&path, b"original")?;
        let error = save_with_hook(
            &path,
            &StoredDocument::default(),
            true,
            &Cancellation::default(),
            &mut |stage| {
                if stage == failure_stage {
                    Err(io::Error::other("injected filesystem failure"))
                } else {
                    Ok(())
                }
            },
        )
        .err()
        .ok_or("ignored filesystem failure")?;
        assert_eq!(error.stage(), failure_stage.name());
        assert!(!error.published());
        assert_eq!(fs::read(&path)?, b"original", "stage: {failure_stage:?}");
        assert_eq!(fs::read_dir(&directory.0)?.count(), 1);
    }
    Ok(())
}

#[test]
fn post_publication_failures_report_new_bytes_and_cancellation_cannot_claim_rollback()
-> Result<(), Box<dyn std::error::Error>> {
    for failure_stage in [Stage::Cleanup, Stage::SyncDirectory] {
        let directory = Directory::new()?;
        let path = directory.target();
        fs::write(&path, b"original")?;
        let document = StoredDocument::default();
        let expected = document.to_cdxml()?.into_owned();
        let error = save_with_hook(
            &path,
            &document,
            true,
            &Cancellation::default(),
            &mut |stage| {
                if stage == failure_stage {
                    Err(io::Error::other("injected durability failure"))
                } else {
                    Ok(())
                }
            },
        )
        .err()
        .ok_or("ignored durability failure")?;
        assert_eq!(error.stage(), failure_stage.name());
        assert!(error.published());
        assert_eq!(error.path(), fs::canonicalize(&path)?);
        assert_eq!(fs::read(&path)?, expected);
        assert_eq!(fs::read_dir(&directory.0)?.count(), 1);
    }
    let directory = Directory::new()?;
    let path = directory.target();
    let document = StoredDocument::default();
    let cancel = Cancellation::default();
    let receipt = save_with_hook(&path, &document, false, &cancel, &mut |stage| {
        if stage == Stage::Cleanup {
            cancel.cancel();
        }
        Ok(())
    })?;
    assert!(cancel.is_cancelled());
    assert_eq!(receipt.byte_length, document.to_cdxml()?.len());
    assert_eq!(fs::read(&path)?, document.to_cdxml()?.as_ref());
    Ok(())
}

#[test]
fn a_competing_creator_cannot_be_clobbered_by_default() -> Result<(), Box<dyn std::error::Error>> {
    let directory = Directory::new()?;
    let path = directory.target();
    let error = save_with_hook(
        &path,
        &StoredDocument::default(),
        false,
        &Cancellation::default(),
        &mut |stage| {
            if stage == Stage::Publish {
                fs::write(&path, b"competing creator")?;
            }
            Ok(())
        },
    )
    .err()
    .ok_or("clobbered competing creator")?;
    assert_eq!(error.code(), "already_exists");
    assert!(!error.published());
    assert_eq!(fs::read(&path)?, b"competing creator");
    assert_eq!(fs::read_dir(&directory.0)?.count(), 1);
    Ok(())
}

#[cfg(unix)]
#[test]
fn a_symlink_created_during_encoding_cannot_redirect_the_save()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = Directory::new()?;
    let path = directory.target();
    let other = directory.0.join("other.cdxml");
    fs::write(&path, b"original")?;
    fs::write(&other, b"unrelated")?;
    let error = save_with_hook(
        &path,
        &StoredDocument::default(),
        true,
        &Cancellation::default(),
        &mut |stage| {
            if stage == Stage::Publish {
                fs::remove_file(&path)?;
                std::os::unix::fs::symlink(&other, &path)?;
            }
            Ok(())
        },
    )
    .err()
    .ok_or("followed late symlink")?;
    assert_eq!(error.code(), "symlink_target");
    assert!(!error.published());
    assert_eq!(fs::read(&other)?, b"unrelated");
    assert_eq!(fs::read_dir(&directory.0)?.count(), 2);
    Ok(())
}

#[cfg(unix)]
#[test]
fn replacement_refuses_an_unrelated_file_created_during_encoding()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = Directory::new()?;
    let path = directory.target();
    let moved = directory.0.join("original.cdxml");
    fs::write(&path, b"original")?;
    let error = save_with_hook(
        &path,
        &StoredDocument::default(),
        true,
        &Cancellation::default(),
        &mut |stage| {
            if stage == Stage::Publish {
                fs::rename(&path, &moved)?;
                fs::write(&path, b"unrelated new file")?;
            }
            Ok(())
        },
    )
    .err()
    .ok_or("overwrote a different file using stale permissions")?;
    assert_eq!(error.code(), "file_changed");
    assert!(!error.published());
    assert_eq!(fs::read(&path)?, b"unrelated new file");
    assert_eq!(fs::read(&moved)?, b"original");
    assert_eq!(fs::read_dir(&directory.0)?.count(), 2);
    Ok(())
}

#[cfg(target_os = "macos")]
#[test]
fn overwrite_preserves_extended_acl_quarantine_and_arbitrary_attribute_bytes()
-> Result<(), Box<dyn std::error::Error>> {
    use rustix::fs::{XattrFlags, fgetxattr, fsetxattr};
    let directory = Directory::new()?;
    let path = directory.target();
    fs::write(&path, b"original")?;
    let acl = [exacl::AclEntry::allow_user("0", exacl::Perm::READ, None)];
    exacl::setfacl(&[&path], &acl, None)?;
    let expected_acl = exacl::getfacl(&path, None)?;
    let original = File::open(&path)?;
    let quarantine = b"0081;01234567;BondlineTest;";
    fsetxattr(
        &original,
        c"com.apple.quarantine",
        quarantine,
        XattrFlags::empty(),
    )?;
    fsetxattr(
        &original,
        c"org.bondline.test",
        b"\0\xff\x81",
        XattrFlags::empty(),
    )?;
    save_cdxml_file(
        &path,
        &StoredDocument::default(),
        true,
        &Cancellation::default(),
    )?;
    let saved = File::open(&path)?;
    assert_eq!(exacl::getfacl(&path, None)?, expected_acl);
    let mut bytes = [0_u8; 64];
    let count = fgetxattr(&saved, c"com.apple.quarantine", bytes.as_mut_slice())?;
    assert_eq!(bytes.get(..count), Some(quarantine.as_slice()));
    let count = fgetxattr(&saved, c"org.bondline.test", bytes.as_mut_slice())?;
    assert_eq!(bytes.get(..count), Some(b"\0\xff\x81".as_slice()));
    Ok(())
}

#[cfg(unix)]
#[test]
fn replacement_refuses_a_same_inode_security_change_before_publication()
-> Result<(), Box<dyn std::error::Error>> {
    use std::os::unix::fs::PermissionsExt;
    let directory = Directory::new()?;
    let path = directory.target();
    fs::write(&path, b"original")?;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644))?;
    let error = save_with_hook(
        &path,
        &StoredDocument::default(),
        true,
        &Cancellation::default(),
        &mut |stage| {
            if stage == Stage::Publish {
                fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
            }
            Ok(())
        },
    )
    .err()
    .ok_or("replaced a newly private file with stale wider permissions")?;
    assert_eq!(error.code(), "file_changed");
    assert!(!error.published());
    assert_eq!(fs::read(&path)?, b"original");
    assert_eq!(fs::metadata(&path)?.permissions().mode() & 0o777, 0o600);
    assert_eq!(fs::read_dir(&directory.0)?.count(), 1);
    Ok(())
}

#[cfg(target_os = "macos")]
#[test]
fn inherited_acl_is_removed_before_any_temporary_document_bytes_are_written()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = Directory::new()?;
    let acl = [exacl::AclEntry::allow_user(
        "0",
        exacl::Perm::READ,
        Some(exacl::Flag::FILE_INHERIT),
    )];
    exacl::setfacl(&[&directory.0], &acl, None)?;
    let mut inspected = false;
    save_with_hook(
        &directory.target(),
        &StoredDocument::default(),
        false,
        &Cancellation::default(),
        &mut |stage| {
            if stage == Stage::Write {
                let files = fs::read_dir(&directory.0)?.collect::<io::Result<Vec<_>>>()?;
                assert_eq!(files.len(), 1);
                let temporary = files
                    .first()
                    .ok_or_else(|| io::Error::other("no temporary"))?
                    .path();
                assert!(exacl::getfacl(&temporary, None)?.is_empty());
                inspected = true;
            }
            Ok(())
        },
    )?;
    assert!(inspected);
    Ok(())
}

#[cfg(target_os = "linux")]
#[test]
fn linux_preserves_posix_access_acl_without_linking_libacl()
-> Result<(), Box<dyn std::error::Error>> {
    use rustix::fs::{XattrFlags, fgetxattr, fsetxattr};
    let directory = Directory::new()?;
    let path = directory.target();
    fs::write(&path, b"original")?;
    // linux/posix_acl_xattr.h version2, tags USER_OBJ/USER/GROUP_OBJ/MASK/OTHER.
    // A named root read ACE is harmless and forces a nontrivial extended access ACL.
    let mut acl = 2_u32.to_le_bytes().to_vec();
    for (tag, permission, id) in [
        (1_u16, 6_u16, u32::MAX),
        (2, 4, 0),
        (4, 0, u32::MAX),
        (16, 4, u32::MAX),
        (32, 0, u32::MAX),
    ] {
        acl.extend_from_slice(&tag.to_le_bytes());
        acl.extend_from_slice(&permission.to_le_bytes());
        acl.extend_from_slice(&id.to_le_bytes());
    }
    let original = File::open(&path)?;
    fsetxattr(
        &original,
        c"system.posix_acl_access",
        &acl,
        XattrFlags::empty(),
    )?;
    fsetxattr(
        &original,
        c"user.bondline",
        b"opaque attribute",
        XattrFlags::empty(),
    )?;
    save_cdxml_file(
        &path,
        &StoredDocument::default(),
        true,
        &Cancellation::default(),
    )?;
    let saved = File::open(&path)?;
    let mut bytes = [0_u8; 1024];
    let count = fgetxattr(&saved, c"system.posix_acl_access", bytes.as_mut_slice())?;
    assert_eq!(bytes.get(..count), Some(acl.as_slice()));
    let count = fgetxattr(&saved, c"user.bondline", bytes.as_mut_slice())?;
    assert_eq!(bytes.get(..count), Some(b"opaque attribute".as_slice()));
    Ok(())
}

#[test]
fn interrupted_replacement_restores_the_original_without_overwriting_a_new_creator()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = Directory::new()?;
    let path = directory.target();
    let backup = directory.0.join("original-backup.cdxml");
    fs::write(&backup, b"original")?;
    let error = restore_original(&path, &backup, io::Error::other("injected ReplaceFile1177"));
    assert!(!error.published());
    assert_eq!(error.code(), "io_error");
    assert!(error.recovery_path().is_none());
    assert_eq!(fs::read(&path)?, b"original");
    assert!(!backup.exists());

    fs::write(&backup, b"original retained")?;
    fs::write(&path, b"concurrent creator")?;
    let error = restore_original(&path, &backup, io::Error::other("injected ReplaceFile1177"));
    assert_eq!(error.code(), "recovery_required");
    assert!(!error.published());
    assert_eq!(error.recovery_path(), Some(backup.as_path()));
    assert!(error.to_string().contains("injected ReplaceFile1177"));
    assert_eq!(fs::read(&backup)?, b"original retained");
    assert_eq!(fs::read(&path)?, b"concurrent creator");
    Ok(())
}

#[cfg(windows)]
fn dacl_sddl(path: &Path) -> io::Result<std::ffi::OsString> {
    use windows_permissions::{
        constants::{SeObjectType, SecurityInformation},
        wrappers,
    };
    let descriptor = wrappers::GetNamedSecurityInfo(
        path.as_os_str(),
        SeObjectType::SE_FILE_OBJECT,
        SecurityInformation::Dacl,
    )?;
    wrappers::ConvertSecurityDescriptorToStringSecurityDescriptor(
        &descriptor,
        SecurityInformation::Dacl,
    )
}

#[cfg(windows)]
#[test]
fn windows_temporary_is_owner_private_before_writing_and_remains_accessible_to_its_owner()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = Directory::new()?;
    let path = directory.target();
    let mut inspected = false;
    save_with_hook(
        &path,
        &StoredDocument::default(),
        false,
        &Cancellation::default(),
        &mut |stage| {
            if stage == Stage::Write {
                let temporary = fs::read_dir(&directory.0)?
                    .next()
                    .ok_or_else(|| io::Error::other("no temporary"))??
                    .path();
                assert_eq!(
                    dacl_sddl(&temporary)?,
                    std::ffi::OsStr::new("D:P(A;;FA;;;OW)")
                );
                assert!(
                    File::open(&temporary).is_err(),
                    "exclusive temporary must deny a competing data-read handle"
                );
                inspected = true;
            }
            Ok(())
        },
    )?;
    assert!(inspected);
    OpenOptions::new().read(true).write(true).open(&path)?;
    assert_eq!(dacl_sddl(&path)?, std::ffi::OsStr::new("D:P(A;;FA;;;OW)"));
    Ok(())
}

#[cfg(windows)]
#[test]
fn windows_replacement_preserves_original_dacl_and_zone_identifier_stream()
-> Result<(), Box<dyn std::error::Error>> {
    use std::os::windows::fs::OpenOptionsExt;
    use windows_permissions::{
        constants::{SeObjectType, SecurityInformation},
        wrappers,
    };
    let directory = Directory::new()?;
    let path = directory.target();
    let mut original = OpenOptions::new()
        .write(true)
        .create_new(true)
        .access_mode(0xc004_0000)
        .open(&path)?;
    original.write_all(b"original")?;
    let descriptor = wrappers::ConvertStringSecurityDescriptorToSecurityDescriptor(
        "D:P(A;;FA;;;OW)(A;;FR;;;SY)",
    )?;
    let acl = wrappers::GetSecurityDescriptorDacl(&descriptor)?.ok_or("no original DACL")?;
    wrappers::SetSecurityInfo(
        &mut original,
        SeObjectType::SE_FILE_OBJECT,
        SecurityInformation::Dacl | SecurityInformation::ProtectedDacl,
        None,
        None,
        Some(acl),
        None,
    )?;
    drop(original);
    let mut stream = path.as_os_str().to_os_string();
    stream.push(":Zone.Identifier");
    fs::write(&stream, b"[ZoneTransfer]\r\nZoneId=3\r\n")?;
    let expected = dacl_sddl(&path)?;
    save_cdxml_file(
        &path,
        &StoredDocument::default(),
        true,
        &Cancellation::default(),
    )?;
    assert_eq!(dacl_sddl(&path)?, expected);
    assert_eq!(fs::read(&stream)?, b"[ZoneTransfer]\r\nZoneId=3\r\n");
    assert_eq!(fs::read_dir(&directory.0)?.count(), 1);
    Ok(())
}
