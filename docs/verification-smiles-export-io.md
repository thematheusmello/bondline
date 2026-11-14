# Create-only SMILES fragment file export

2026-11-14. Original IO-only slice in the supplied `bondline-rxn` branch
`feat/smiles-export-io`, base `a393fb9`. Root owns Git and the later registry/adapter
integration. No command/UI, source authority, format association or new writer policy
is implemented here. The user remains active on the desktop and Actions remains off.

## API and file framing

`bondline_io::files::export_smiles_fragment_file(output: &Path,
fragment: &bondline_model::Fragment, cancel: &Cancellation)` returns
`Result<SaveReceipt, FileError>`. It always creates a new output and provides no overwrite
option. The exact current `smiles::write_fragment` supplies notation; the IO encoder
appends exactly one literal LF, then delegates to the existing atomic publisher with
overwrite=false. No new publisher, encoder framework or runtime dependency is added.

The primary pinned [OpenSMILES v1.0 SMILES Files section](https://github.com/opensmiles/OpenSMILES/blob/9240a2887158699503605bc838e1de9a9f7883f4/opensmiles.asciidoc#L1371-L1383)
defines line-terminated file records. The cached source was independently rehashed and
read: 86,816 bytes, SHA256
`00f0423da612de913770133aa13e456d0cebc562dd50166534a8a7f9a834bd21`.
One record with LF, no leading whitespace/name/extra data, is an original output choice
within that public framing. No upstream prose, implementation or asset was copied.

The original draft expected notation without LF. Root's primary-source review corrected
that expectation **before production implementation**. Both earlier failures were missing
API/helper compile REDs; the superseded no-LF draft never demonstrated a production
behavior defect. Logs stay intact. The pure writer still returns the exact unframed
notation and was not changed to accommodate filesystem framing.

The writer's 65,536-byte notation cap remains exact. Adding LF gives a file upper bound
of 65,537 bytes; no extra notation truncation, rounding, name or platform CRLF conversion
runs. A whole graph at that maximum file size was not constructed in this slice. The
existing parser and converter enforce a **raw whole-input** 64 KiB limit, unchanged here,
and can refuse that maximum framed size. Their bounded notation/12-carbon conversion
profile is not general SMILES-file re-import. There is no export/import parity claim.

## Preserved chemistry and authority boundaries

All existing writer capabilities remain delegated intact: neutral H/C/N/O/F/Cl/Br/I,
exact supported positive isotopes, physical-H nodes, literal explicit H, Single/Double/
Triple bonds, branches, cycles and disconnected components. Deterministic authored-ID
traversal remains noncanonical. Missing H, charge/radical/unsupported element/order,
finite-valence mismatch, empty fragment and existing resource limits retain their typed
writer refusals; no chemical perception or periodic-table inference is added.

The borrowed Fragment stays exact on success and failure. IDs and geometry are not
encoded in notation. Positioned, absent, coincident or off-page coordinates do not add
drawing/layout restrictions to chemistry export. Pages, styles, source-only payloads and
other drawing context are outside this pure API. It cannot establish whether a projected
fragment has complete source authority. A future command must retain the complete source
owner, require an explicitly selected fragment and guard unsupported backing; this API
does not authorize dropping unknown CDXML/CDX/MOL/SDF data or marking a session saved.

The existing SMILES→new CDXML API, raw source limits, parser/normalizer/layout, symbol
catalog/routing, model, native-v0, writer, platform publisher, locks and assets remain
unchanged. The earlier converted-CDXML file tests still run in the full IO gate.

## Errors, cancellation and publication

The new transparent FileError cause holds the original WriteError. Codes are unchanged:
`invalid_smiles_export`, `unsupported_content` and `resource_limit`. Encode refusals
identify the canonical output path and stage `encode`; no temporary/output is created.
Destination validation still precedes encoding, matching the shared publisher contract.

The existing `save_encoded_with_hook` returns its original FileError/SaveReceipt without
rebuilding code/path/stage/`published`/`recovery_path`. Existing leaves and output symlinks
refuse. A racing creator at the actual publish stage wins without being overwritten.
Cancellation before publication leaves no output. Cancellation after publication returns
the completed receipt; real cleanup/directory-sync errors report `published=true` with
the new exact file present. No rollback or Windows replacement/recovery claim is made.
Create-only export does not enter the overwrite replacement/recovery route.

## Independent saved-file acceptance

Six integration tests were defined before the production API. A real Unicode/misleading-
suffix output must be exactly `[CH3][CH3]` followed by one LF, with the canonical receipt
path and byte length 11. The original positioned Fragment stays equal. Actual saved bytes
are read back, checked as ASCII and checked for one final LF and no preceding CR/LF.
Only that exact pre-LF byte slice goes to the **unchanged** pinned external
`smiles_write_oracle.py`; there is no whitespace trim, alternate reconstructed graph or
consumer mutation. The known unsanitized/cleanup qualifications from
[writer verification](verification-smiles-write.md) remain. Its separate sanitized molecule
checks meaning/formula/radicals; neither pass implements Bondline chemistry.

Real saved-file vectors exercise branches, rings, Single/Double/Triple, C/N/O and every
supported halogen, disconnected components, isotope 13, physical H/deuterium, and isotope
65,535. Literal output bytes, external atom/H/isotope/charge/order/connectivity and zero
conformers/radicals are checked. An actual 4,096-carbon file has 20,481 bytes including LF
and faces the same consumers, proving the filesystem API has no 12-atom or drawing-ink
restriction. Coincident off-page authored positions remain untouched throughout.

Independent refusal vectors cover empty/unknown-H/charge/radical/element/valence/physical-H/
aromatic-order errors and 4,097 atoms. Existing output bytes/mode/mtime remain, initial
cancellation publishes nothing, and Unix output symlinks preserve their targets. Two new
private tests use the real shared publisher hook for a write failure, cleanup/directory
sync after publication, a competing creator and cancellation on either side of publication.
They verify exact file bytes, canonical path, stage, publication flag, absent recovery path,
unchanged Fragment and no leftover temporary. Existing exhaustive platform/publication
tests are reused, not duplicated or weakened.

## Actual RED/GREEN and environment

All Cargo calls use Rust 1.94.1, this checkout's absolute target, CARGO_INCREMENTAL=0,
CARGO_PROFILE_DEV_DEBUG=0, CARGO_PROFILE_TEST_DEBUG=0, and Root's existing pinned Python
interpreter via BONDLINE_PYTHON. No installation or target sharing occurred.

- Initial real-file acceptance missing-API RED: session 63251 exits 101, E0432 missing
  `files::export_smiles_fragment_file`; `target/smiles-export-io-api-red.log`.
- Expanded contract missing-hook RED: session 81819 exits 101, E0425 missing private
  `export_smiles_with_hook`; `target/smiles-export-io-contract-red.log`. The framing
  correction above preceded the minimal implementation.
- Minimal GREEN: `cargo test --locked -p bondline-io --lib --test smiles_export_files`,
  session 98010 exits 0, 23 library + 6 integration tests, zero failed/ignored.
  Eight of these tests are new; `target/smiles-export-io-first-green.log`.
- Full scoped IO: `cargo test --locked -p bondline-io`, session 21228 exits 0,
  240 passed, zero failed/ignored plus zero doctests;
  `target/smiles-export-io-full-green.log`.
- All-target IO clippy with warnings denied: `cargo clippy --locked -p bondline-io
  --all-targets -- -D warnings`, session 14159 exits 0;
  `target/smiles-export-io-clippy.log`.
- Workspace rustfmt: `cargo fmt --all -- --check` exits 0;
  `target/smiles-export-io-fmt.log`.

Exact protected/source/log identities are recorded in the ignored frozen handoff. No broad
workspace, fuzz, native application, GUI/input/
activation, Windows/Linux execution or hosted workflow ran in this slice. Those checks
remain distinct from this library behavior; current M1 and M0 are incomplete.
