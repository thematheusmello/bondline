# Create-only SMILES to native CDXML conversion

2026-11-14. Original IO-only slice in `bondline-rxn`, supplied branch
`feat/smiles-conversion-io`, supplied base `c2b739a`. The scope is one filesystem API;
there is no new registry command, CLI/MCP/UI adapter, SMILES Open/Save association or
active Engine mutation. Root owns the later adapter integration and Git.

## Delivered contract

`bondline_io::files::convert_smiles_to_cdxml_file(input: &Path, output: &Path,
bond_length_pt: f64, origin: Point2, page: PageSize, cancel: &Cancellation)` returns
`Result<SmilesConversionReceipt, FileError>`. The receipt derives Debug/Clone/PartialEq/Eq
and contains `input_path: PathBuf`, `input_byte_length: usize`, `atom_count: usize`,
`bond_count: usize` and `output: SaveReceipt`. Paths and counts are metadata; no raw
notation/IR is printed or returned. `input_path` is the actual canonical read path.

The whole ordinary input file is read and parsed as ASCII one-notation syntax with
the existing optional single final LF/CRLF. Blank input, names, extra records/lines,
non-ASCII and malformed syntax do not become partial successes. The complete original
source/IR remains owned through layout and publication, then is dropped. The source
file remains unchanged; no SMILES backing/baseline is attached to the new drawing.

The existing [carbon-path layout](verification-smiles-layout.md) supplies fresh local
document/fragment IDs, fixed literal H and positioned neutral nonisotopic carbon/Single
paths of 1–12 atoms. Its topology/geometry/glyph/mask/stroke/page guards stay unchanged.
All parameters are explicit; 48 pt/origin (72,72) pt/A4 is the separately verified starting
policy, not a newly implicit default. Actual branches of degree 3, cycles, disconnected
graphs, aromatic markers/orders, heteroatoms/physical H, isotope/charge/class/stereo,
Double/Triple, ambiguous bracket valence and unsupported layout refuse. Equivalent
source syntax such as `C(C)C` or cross-dot `C1.C1` is decided by actual path topology.
The 1024 square reference-scene validation does not certify every native viewport/DPI.

The generated source-free `StoredDocument` uses the existing native CDXML encoder and
`save_cdxml_file(output, drawing, false, cancel)`. The caller cannot request overwrite.
This is a new editable artifact, not a receipt for saving/replacing an active document.
It reopens fully mapped; no foreign source projection is regenerated or silently lost.

## Read, errors and publication

The shared native reader now accepts a private closed Native/Smiles budget, preserving
the original 64 MiB behavior/messages for existing readers. SMILES is checked at 64 KiB
against both metadata inspections before a heap buffer is allocated and against every
read before growth. Requested amortized buffer capacity is clamped to that budget;
this is not an allocator/RSS guarantee. A partial-read test actually crosses the cap
after reading starts. Oversize input gives `file_too_large` at stage `read` and says
64 KiB; it never borrows the native64 MiB label. Exactly 65,536 bytes reach the decoder,
whose independent atom/token/profile limits remain active.

Typed SmilesError/LayoutError causes are retained inside FileError. Their existing
codes remain `invalid_smiles`, `unsupported_content`, `resource_limit`, or, for invalid
L, `invalid_params`; the path is canonical input and stage is`decode`. Existing path,
ordinary-file, cancellation and platform-open behavior stays shared. Read symlinks
resolve to their actual target; output symlinks are refused. The converter does not
change the platform security/metadata or atomic-publication implementation.

All save errors are forwarded as the original FileError, retaining actual code/path/
stage/`published`/`recovery_path`. A preexisting output, the source itself or a hard-link
alias refuses, as do a competing regular creator/hard link/symlink at publication.
Cancellation before publication leaves no output. Cancellation after publication
reports the completed receipt rather than pretending rollback. Real injected cleanup/
directory-sync failures retain the new native bytes and report `published=true`.
The recovery-metadata pass-through test is explicitly synthetic: it proves forwarding,
not actual Windows recovery or the existence of a synthetic output file.

## Independent actual-file acceptance

Six integration tests were written before the production API. For each1–12carbon path,
the converter reads a real Unicode-named input, writes a real new CDXML file and checks
receipt metadata. An external Python hashlib/os.stat snapshot verifies identical input
SHA256, byte count, mode and mtime_ns before/after. Atime/ctime are not claimed unchanged.
Actual output bytes then face the unchanged Expat/Decimal `cdxml_graph.py`, unchanged
`smiles_layout_oracle.py` with 40-digit independent analytic geometry/external RDKit,
and unchanged `native_bond_rdkit.py` for C2. No alternate synthetic molecule is passed
to those consumers. The separate same-JSON-boundary structural comparison retains the
previously documented 1 ULP transport qualification; raw Decimal/XML coordinates and
strict full Document/reopen equality are not weakened or rounded.

Each real output reopens as fully mapped native CDXML, equals its entire positioned
Document exactly and preserves complete bytes on unchanged export. A real C3 output
is then edited through the existing model replacement, saved to a separate native file
and reopened exactly, while the original conversion and source remain unchanged. This
is headless library editability, not a native gesture or general cleanup claim.

Error cases check both a new output and an existing unrelated target, exact input bytes/
independent snapshot and absence of temporary artifacts. They cover malformed/extra-EOF/
name/empty/oversize input, profile/topology/metadata/aromatic refusals, 13 atoms, invalid L,
page/label/origin clearance, missing/directory input, existing/same-source/hard-link output,
initial cancellation and Unix read/output symlinks. Private tests use the real shared
publisher hook for create/write/metadata/sync/publish/cleanup/directory-sync failures,
late creator/link races and cancellation on either side of publication. No substitute
publisher or new runtime dependency is shipped.

## RED/GREEN and actual environment

All Cargo calls use Rust 1.94.1, the absolute own RXN target, CARGO_INCREMENTAL=0,
CARGO_PROFILE_DEV_DEBUG=0, CARGO_PROFILE_TEST_DEBUG=0, and BONDLINE_PYTHON pointing to
the existing pinned Root `.cache/oracles/format-python/bin/python`. Consumers are
required, not skipped when missing; their scripts/pins were not changed.

- Missing API RED: `cargo test --locked -p bondline-io --test smiles_conversion`,
  session 21710 exit 101, E0432 missing converter/receipt. The obsolete unused test import
  warning was removed; it is not an independent behavioral failure. Log:
  `target/smiles-conversion-io-api-red.log`.
- Runtime read-budget RED: the same command, session 58602 exit 101, 5 pass/1 fail. A 65,537 byte
  input reached the decoder as `resource_limit`, rather than the required preallocation
  `file_too_large` read error. Log:`target/smiles-conversion-io-limit-red.log`.
- An incidental follow-up compile failure, session 70474 exit 101, required Debug on the
  new private ReadLimit. Its preserved, unfortunately named
  `target/smiles-conversion-io-first-green.log` is **not** counted as GREEN or TDD evidence.
- Actual focused GREEN: session 12733 exit 0, 6 pass/0 failed/ignored,
  `target/smiles-conversion-io-focused-green.log`.
- Private boundary tests: session 94339 exit 0, 5 pass/0 failed/ignored,
  `target/smiles-conversion-io-unit.log`.
- Full scoped IO: `cargo test --locked -p bondline-io`, session 67416 exit 0,
  230 pass/0 failed/ignored plus 0 doctests; `target/smiles-conversion-io-full-green.log`.
- All-target warnings-denied IO clippy: `cargo clippy --locked -p bondline-io
  --all-targets -- -D warnings`, session 91487 exit 0;
  `target/smiles-conversion-io-clippy.log`.
- Workspace rustfmt check: `cargo fmt --all -- --check`, exit 0;
  `target/smiles-conversion-io-fmt.log`.

All own jobs are terminal. The ignored frozen handoff records exact source/log/protected
file sizes/hashes. No dependency/lock/layout/normalizer/model/render/oracle/corpus/golden
change, Git operation, installation, fuzz run or full-workspace gate was performed here.
The user was actively using the desktop: no GUI/input/activation/native test ran. New
Windows/Linux execution is unverified. GitHub Actions remains disabled by user policy;
no hosted workflow was enabled, dispatched, rerun or awaited.

## Provenance

No new third-party implementation or asset was copied. The same pinned OpenSMILES
v1.0 source and IUPAC2008 recommendations recorded in
[sources.md](clean-room/sources.md#restricted-carbon-path-layout-2026-11-14) support the
existing parser/fixed-H/layout behavior, unchanged here. Whole-file conversion,64 KiB
read-budget selection, metadata-only receipts and create-only artifact semantics are
original Bondline policies. The earlier native IO PHOTO-ATOMIC/PHOTO-METADATA research
remains provenance for the reused publisher, not transplanted code or broader guarantees.
