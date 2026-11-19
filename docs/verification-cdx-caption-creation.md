# CDX caption creation verification

Base `492b6ae`, 2026-11-19. The [source contract](research/cdx-caption-creation.md)
specifies the direct Page Text, explicit font resources and persistent ObjectTag.
The [fixture register](../crates/io/tests/fixtures/cdx-caption-create/SOURCES.md)
pins six independently checked binary files.

## Persistent ID receiving

The identity runtime RED failed four valid marked inputs at the existing
unsupported Text record. Both legacy and invalid-marker controls passed.
With the identity reader, all six pass. The same run passed 17 existing
caption preview, styled-run and edit tests.

The tests independently check raw-order legacy IDs and sparse IDs with
explicit and implicit persistence. They move a marked Text ahead of the
old raw ID and still recover the original live IDs. Both old and marked
run edits match exact byte splices, including Unicode Save and reopen.
Malformed identities still refuse application Open. The standalone CDX
reader retains their exact bytes, and the Open attempt leaves the file
intact.

Primary worktree receipts are under `target/cdx-caption-creation`.
The identity RED log SHA-256 is
`32a166b73416f59cdca10e4f0b73a6ea5e7d500f584374e9cebd2e1e9c341dd2`.
The passing log SHA-256 is
`af254ec993e56ca3561598e5e84838e02cb5ffc52c1188a2d18d80cbb80e46fc`.
