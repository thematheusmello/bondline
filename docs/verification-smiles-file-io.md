# Whole SMILES file filesystem I/O

2026-11-14. This owner works only in the supplied isolated
`bondline-mol-v3000` checkout, initial base `df34c66`. Root owns Git and integration.
No GUI/input/activation, Actions, installs, dependencies, locks, notation parser, writer,
model, renderer, commands, fixtures or old oracle changes are authorized here.

The delivered APIs are `files::read_smiles_file(path, cancel)` returning a complete
`LoadedFile<SmilesFileDocument>` and `save_smiles_file(path, document, overwrite, cancel)`
returning the existing `SaveReceipt`. They use the existing native 64 MiB reader
and atomic publisher. They do not normalize notation, add LF, infer chemical metadata,
project the first record, select a drawing or create an application file association.

The separate pure file codec owns every source byte and validates every meaningful
notation with the unchanged bounded parser. File records may include LF/CRLF and optional
space/tab-separated opaque data. Leading SP/TAB/CR lines and blank lines are retained
but ignored, not trimmed into notation. Final EOF without LF is a documented compatibility
extension. Generic-whitespace conformance is not claimed. Per-notation 65,536-byte and
whole-file 64 MiB limits have separate jobs; multiple records and metadata do not shrink
the native whole-source budget to 64 KiB.

## Primary evidence and independent expectations

The cached immutable OpenSMILES publisher source at commit
`9240a2887158699503605bc838e1de9a9f7883f4` was rehashed/read: 86,816 bytes,
SHA-256 `00f0423da612de913770133aa13e456d0cebc562dd50166534a8a7f9a834bd21`.
Its [SMILES Files section](https://github.com/opensmiles/OpenSMILES/blob/9240a2887158699503605bc838e1de9a9f7883f4/opensmiles.asciidoc#L1371-L1383)
provides the multiple-line record, optional application data, ignored-line and LF/CRLF
facts. File boundaries/opaque interpretation and the final-EOF accommodation are qualified
in the separate file-codec record. The existing native publisher/security/error policies
remain original Bondline decisions; no upstream source or help prose is transplanted.

The original inline 123-byte mixed vector has three meaningful records on physical lines
3, 4 and 6. Its independent literal expectations distinguish complete line, notation,
separator, application data and line-ending spans. It includes LF, CRLF, final EOF, ignored
invalid-looking lines, isotope/H notation, aromatic notation and physical deuterium/H.
Complete copied bytes, metadata and order must remain exact under Unicode paths and
misleading suffixes. Every meaningful notation faces the unchanged pinned external
`smiles_oracle.py`, with literal element/isotope/H and endpoint/order expectations; the
full-file supplier/name parser is not used as a metadata oracle.

Further cases cover a whole source exceeding 64 KiB through opaque data,
empty/ignored-only sources, old ownership after input removal, malformed or unsupported
later records, the independent native 64 MiB bound, cancellation, create-only/overwrite,
read/write symlink distinction and actual full-profile writer-file reopening. That original
21-atom output spans branches, a ring, Single/Double/Triple, isotopes/physical H, multiple
components and every currently supported writer element, beyond the converter's 12-atom
profile. The original writer/consumer files and all old fixtures remain unchanged.

## Actual TDD evidence

The isolated target and pinned Root `BONDLINE_PYTHON` are used with incremental 0 and
dev/test debug 0. Initial command:

```bash
cargo test --locked -p bondline-io --test smiles_files
```

The actual missing-API RED (terminal tool chunk `dbfeed`) exits 101 with E0432 for both
`read_smiles_file` and `save_smiles_file`. It does not fail on syntax, linkage or a
runtime assertion; no success expectation was reached. The real-file success test was
written first. Log: `target/smiles-file-io-red.log`.

At the initial checkpoint the separate real codec was still being implemented. This
owner held the draft, added no stub/copied uncommitted backend and counted no draft GREEN.
The real codec was subsequently received at Root-supplied `ca11b2f` (codec source
`3745908`); its API was inspected before wrapper implementation. Those identifiers were
supplied by Root, without worker Git operations.


## Implemented composition and executed checks

The only production changes are the two public wrappers/reexports and one transparent
`Failure::SmilesFile` cause with direct code forwarding. Read calls the existing native
`read_source_bytes` (64 MiB, never the separate notation converter's 64 KiB reader), then
real `SmilesFileDocument::parse_with_cancel`. Its failure keeps the canonical path, stage
`decode` and original code/context. Save borrows `document.as_bytes()` directly into
`save_encoded_with_hook`; it returns the original publisher result. Neither operation
changes the existing exporter/converter bodies or creates an application content kind.

The first actual-backend focused run (terminal chunk `9a24f1`) exits 0: all nine prepared
integration tests pass. Their source remains byte-identical to pre-codec draft v2; no
expectation or implementation correction was needed after the first actual compile.
`target/smiles-file-io-focused.log` records the result. Before this, the independent vector
self-check (`target/smiles-file-io-vector-check.log`) confirmed four literal graph vectors
through unchanged pinned `smiles_oracle.py`; that was vector evidence, not wrapper GREEN.

One focused private regression also injects Write, Cleanup and SyncDirectory failures,
with both create-only and overwrite. Before-publication failures retain an existing
destination or leave it absent; later failures report `published=true` with the complete
new bytes present. Canonical path/stage/code, empty recovery path, owned-source equality
and temporary cleanup are checked. This regression follows the initial implementation;
it is not claimed as a separate newly observed RED. Existing locally enabled reader,
publication, cancellation, race, security and recovery regressions run unchanged.

Final owner commands, using the same isolated target/interpreter:

```bash
cargo test --locked -p bondline-io
cargo clippy --locked -p bondline-io --all-targets -- -D warnings
cargo fmt --all -- --check
```

Session `44576` is terminal, exit 0. Full I/O: **259 passed, 0 failed, 0 ignored** across
37 suite result lines, including empty doctests. The received codec-integrated baseline
has 249 tests; this slice adds nine integration and one private passthrough test. All-target
I/O warnings-denied clippy and workspace formatting both exit 0. Logs are
`target/smiles-file-io-tests.log`, `target/smiles-file-io-clippy.log` and
`target/smiles-file-io-fmt.log`. No unrelated test run was repeated after these passed.

`docs/native-file-io.md` records the delivered public API. The ignored final handoff
records the exact three source/test and three documentation files, log hashes, received
codec identities and unchanged protected files. Old notation reconstruction and received
codec evidence remain distinct from this wrapper's execution. The whole-file API accepts
aromatic/otherwise supported notation directly; it does not narrow files to the exporter's
neutral explicit-H profile or the converter's restricted carbon-path profile.

No full-workspace test/xtask/fuzz gate, new Windows/Linux or native UI workflow was run
by this owner. Root's receiving gates are separate. Registry count stays 22: application
SMILES content snapshots/Open/Save, chooser and editing remain missing. Source-backed
drawing authority, general chemistry/perception, normalization, canonicalization and M1
completion are not established. All own jobs are terminal; no GUI/Git/Actions/install or
dependency change occurred.
