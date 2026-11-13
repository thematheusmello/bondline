# Native CDXML filesystem contract

`bondline_io::files` is a library adapter. It reads/writes complete `StoredDocument` values;
it never regenerates a foreign elemental projection or exposes an extension without decoding.
Registry/UI delivery is recorded separately in control-protocol.md and STATUS.md.

- `read_cdxml_file(&Path, &Cancellation)` returns a canonical path and owned complete source.
- `canonical_save_path(&Path)` resolves the parent without following/replacing the final leaf.
- `save_cdxml_file(&Path, &StoredDocument, overwrite, &Cancellation)` returns the canonical
  destination, encoded byte length and whether the parent directory was synced.
- `FileError` exposes stable `code`, `path`, `stage`, `published` and optional `recovery_path`.

Read input is bounded to 64 MiB, including bytes read after metadata inspection. Only ordinary
files are accepted. Selected read symlinks resolve explicitly; Unix opens use nonblocking,
no-follow flags and post-open validation, preventing FIFOs from blocking the worker. Invalid
XML/model input returns an error without an application-state change. Cancellation is checked
between 64 KiB chunks and around decoding; it cannot interrupt an OS syscall or parser call.

Save encodes through `StoredDocument::to_cdxml`: unchanged unknown data passes through exactly,
supported native/profile edits encode completely, and unsupported modified opaque data fails.
It prepares a same-directory temporary, writes in chunks, preserves supported security/metadata,
syncs the file, then publishes. Default no-clobber uses an atomic hard-link create, so a racing
creator wins without being overwritten. Filesystems lacking hard links fail explicitly.
Overwrite is opt-in and refuses symlink/reparse, read-only and special leaves. File identity
and Unix owner/group/mode/ctime changes detected before the final check cause refusal.
These checks are not atomic compare-and-swap against external changes after that check.

Failures before publication retain the destination; injected tests cover creation, write,
metadata, file sync and publish stages on real files. Cleanup/directory-sync failures after
publication carry `published=true`: the new bytes exist and the old file was not restored.
Cancellation after publication finishes reporting cleanup/durability; it does not claim rollback.
Unix syncs the parent directory. Windows currently reports `directory_synced=false`.

## Platform access and recovery

Unix fresh files use mode 0600. Owner/group, bounded xattrs, macOS ACL/creation time and Linux
raw POSIX access ACL are preserved; failure aborts before publication. Xattr limits are 1 MiB
of names, 16 MiB per value and 32 MiB total values. Linux uses kernel xattr bytes through
rustix, with no linked libacl. macOS exacl is target-specific and calls OS ACL APIs.
Inherited ACLs are cleared before our writes. A different process's handle opened during
the macOS create-to-ACL-clear interval cannot be revoked; universal temporary confidentiality
in a directory with inherited allow ACEs is not established.

Windows temporaries request WRITE_DAC and deny competing opens while live, then apply a
protected owner-rights DACL before writing. Overwrite uses zero-flag ReplaceFileW, preserving
documented DACL/security streams; original/temporary owner or group SID differences refuse
instead of changing privileges. A protected same-volume backup directory retains the original
for recovery. Protection cannot revoke an inherited directory handle opened before tightening.
The 1177 partial-replacement case restores the original by no-clobber hard link. If a different
creator occupies the destination, the retained backup is reported instead of overwriting it.
No recursive cleanup deletes unknown files. Native Windows verification is still pending.

## Evidence and provenance

TDD logs are ignored under target/: file-io-red, file-io-cancel-red, file-io-metadata-red,
file-io-recovery-red and file-io-security-red. Native Mac passes 44 io tests, including 18
new file cases. Linux arm64 Docker passes nine file unit and eight integration tests on its
native filesystem, including non-UTF8 filenames, FIFO and raw access ACL. Linux is not
simulated on APFS. Three-platform source clippy and declared-license gates pass; Windows
filesystem behavior is not inferred from compilation. Detailed evidence is in verification.md.

PhotoCraft's pinned atomic-save and metadata adapters informed the design; see source IDs
PHOTO-ATOMIC and PHOTO-METADATA. Bondline uses independently written smaller code and explicit
failure semantics; upstream best-effort security downgrades are not adopted.

The dependency pins below add only target-specific OS adapters. Existing rustix 1.1.6 is reused.
All archives were compared with their published crate-index checksums:

| Package | Published archive SHA-256 | Upstream VCS revision | Permission reviewed |
|---|---|---|---|
| exacl 0.13.0, macOS only | `8de9e551ced9d700a31dacb8b168129640240fcd7a279a3fa4d59608b84ebcae` | `fa36df1745a41e50ab0730546c04d86a8503e46b` | MIT; original upstream notice remains in its package |
| winsafe 0.0.29, Windows kernel only | `9ef0ffc427f045c0cc9ebffd6f4f91153dcd2a1547b5c3c29ee3f541e16e95c6` | `71ed88c2a0d18b03ee452f6d4261f4c22f443483` | MIT; original upstream notice remains in its package |
| windows-permissions 0.2.4, Windows only | `9e2ccdc3c6bf4d4a094e031b63fadd08d8e42abd259940eb8aa5fdc09d4bf9be` | `8740e4efbd88dd01046ad9c169894f3a52eb6e2c` | Published Cargo.toml declares MIT; package and reviewed upstream tree contain no LICENSE file |

windows-permissions names Daniel Dulaney <dan@dulaney.xyz> as author. Its exact declaration,
author, source revision and archive identity are retained here; no absent notice is invented.
This satisfies the existing declared-license gate, not a completed release notice/source audit.
The missing upstream notice is an explicit release-audit question in clean-room/open-questions.md.
