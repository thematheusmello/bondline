# Owned source-preserving SMILES file container

2026-11-14. Original codec-only slice in supplied `bondline-rxn` branch
`feat/smiles-file-codec`, base `df34c66`. This adds complete source ownership and a compact
record index. It does not add filesystem adapters, commands, a Drawing projection,
chemical preview, editing, source-backed application Save or format association.

## Delivered API and ownership

`smiles::file` is public; `SmilesFileDocument` and `SmilesFileError` are also re-exported
from `smiles`. The owned API is:

- `parse(Arc<[u8]>)` and `parse_with_cancel(Arc<[u8]>, &Cancellation)` validate the whole
  file before returning a snapshot. Every meaningful notation uses the unchanged parser;
  malformed, unsupported or over-limit later records reject the whole file.
- `as_bytes()` borrows the exact complete source, including all ignored lines, opaque
  data, separators and endings. No regeneration or new multi-record writer is required.
- `records()` and `record(RecordId)` expose immutable metadata. RecordId is a positive
  u32 ordinal local to one immutable source, not a chemical or cross-source identity.
- `read_record(id)` reparses only the selected bounded notation into a separate temporary
  `SmilesDocument`. Its IR/token offsets are **notation-local**. RecordInfo ranges and
  the wrapper error offset are **whole-file-relative**. Missing ordinals give not_found,
  never a first/empty graph.
- `retained_bytes(iterator)` counts distinct inline snapshots and deduplicates source/
  index allocations. It saturates and excludes Arc/allocator headers, temporary reads
  and external copies, following the existing SdfDocument ownership pattern.

The document owns only `Arc<[u8]>` and `Arc<[RecordInfo]>`. No parsed IR, metadata String,
name map, copied per-record source or first-molecule projection is retained. Clones share
both allocations. Equality includes every source byte and metadata; Debug exposes only
length/count metadata. RecordInfo's private fields have getters for ID, 1-based physical
line, full/notation/separator/application-data/line-ending ranges and literal atom/bond
counts. Ranges are returned by value. Counts are not perceived formula or components.

## Exact file profile

The primary pinned [OpenSMILES v1.0 source](https://github.com/opensmiles/OpenSMILES/blob/9240a2887158699503605bc838e1de9a9f7883f4/opensmiles.asciidoc)
was rehashed and read: 86,816 bytes, SHA256
`00f0423da612de913770133aa13e456d0cebc562dd50166534a8a7f9a834bd21`.
Its grammar whitespace definition (line 154), string termination (1144–1152) and
SMILES Files section (1372–1383) are distinct evidence.

- LF alone splits physical lines. Only CR immediately paired with a terminal LF is
  removed from the borrowed body view; the original bytes always remain exact.
- Empty bodies, or bodies whose **first byte** is SP0x20/TAB0x09/CR0x0d, are ignored
  chemically and retained fully. This explicit set is an original interpretation of
  the primary grammar/file wording, not a full generic-whitespace conformance claim.
  Rust's whitespace predicate, VT/FF, Unicode space, BOM and comment/header guesses are
  not used. Meaningful VT/FF/BOM/# input reaches the unchanged notation refusal.
- The first SP/TAB separates notation. Its complete contiguous SP/TAB run has a separate
  span; all following application bytes are opaque. Empty data is Some(empty range),
  distinct from no separator/data. Non-UTF8, NUL, tabs and bare CR in data are retained;
  there is no whole-file ASCII/UTF8 check, field interpretation or whitespace trim.
- Final EOF is an explicit original compatibility extension. An unterminated final line
  has an empty ending range and is not modified. A terminal LF creates no phantom line.
  Bare CR is not another delimiter: leading CR is ignored by the prefix policy, CR in
  separated data is opaque and internal notation CR still fails.
- Empty and ignored-only sources are valid zero-record files, without a synthetic graph.

The independent public-contract review, ignored Root
`target/smiles-file-contract-review.md` (17,714 bytes, SHA256
`44cffe74855fb86ce65ad0fe08fbbcd140aaf0388ad636ce83b3e031bc88d70e`), records a bounded
peer probe of the existing pinned RDKit supplier. It accepts final EOF but normalizes
CR, leading data and comments and yields partial None records on bad input. Those
supplier behaviors are **not** metadata/framing/all-or-nothing oracles and were not
copied. The formal LF/CRLF requirement and observed EOF extension remain separate.
The generic ignored-whitespace ambiguity remains in clean-room/open-questions.md.

## Quotas, errors and cancellation

Original file quotas are 64 MiB complete source, 1,000,000 physical lines including
ignored lines, and 65,536 meaningful records. Index growth is checked before adding
another record. Metadata and physical lines have no separate 64 KiB cap. Each notation
is checked against the existing 65,536-byte limit **before copying it into an Arc**;
all other existing parser limits and the bounded partial notation language are unchanged.

File LF/CRLF framing is outside that notation limit. The original valid 65,536-byte
leading-zero isotope vector is accepted with EOF, LF (65,537 file bytes) and CRLF
(65,538). The old single-notation parser still refuses the full over-cap framed bytes.
This is a source/framing boundary test, not a graph at the writer's maximum output.

`SmilesFileError::Notation` retains the original typed SmilesError and its code, adding
the exact physical line, meaningful record ordinal and absolute file offset. Getter
context is optional only where no real line/record exists (whole-input bounds, missing
ordinal, internal invalid span). Resource errors use resource_limit, missing records
not_found and cooperative cancellation cancelled. No source/application bytes are echoed.

The indexing function borrows source and polls the real Cancellation through a small
private callback. It checks entry, every physical line (including ignored lines), before
and after each bounded notation parser call, and before returning. Cancellation does not
interrupt one parser call or a single physical-line scan. Private deterministic checkpoint
tests exercise both sides of parsing, later records and ignored lines; the public API test
also uses an actually cancelled token. No partial owned document escapes on failure.

## Independent acceptance

Eight integration tests preceded implementation. The original 34-byte mixed-ending binary
source has literal expected whole/notation/separator/empty-data/EOL spans for all three
records. A separate original 52-byte six-line source from the public-contract review has
SHA256 `df5888fbca54c70581fa9942319996ea3e098137da111201eab3404d58ba2a08` and separately
fixed spans/atoms/orders. New test-only `smiles_file_raw_oracle.py` scans the actual raw
bytes independently with Python standard-library byte operations, not Rust spans or a
molecular supplier. CR normalization, lost ignored bytes, changed nonUTF8 data, dropped
later record and collapsed separator controls change its full metadata expectations.

Each exact independently checked notation slice also faces the **unchanged** pinned
external `smiles_oracle.py`; element counts and every endpoint/order must match fixed
original expectations. A real existing Unicode-named filesystem export is then parsed
as one complete exact record and its selected notation is consumed by the unchanged
external writer/RDKit oracle. No consumer script, parser pin or chemistry assertion was
weakened. This verifies library interoperability, not an application/native flow.

Tests cover every agreed ignored prefix, opaque binary/empty data, terminal EOF/LF/CRLF,
leading/bare/internal CR distinctions, meaningful VT/FF/BOM/Unicode/NUL/# refusals, later
malformed/unsupported records, aromatic notation counts, 65,536/65,537 notation bytes,
metadata over 64 KiB, 64 MiB/one-over source, 1m/one-over physical lines, 65,536/65,537
records, full-byte equality, clone/source/index sharing, Debug redaction and retained
accounting across same/clone/reparsed/independent owners. These bounds are not an RSS cap.

## RED/GREEN and actual environment

All Cargo commands use Rust 1.94.1, the own absolute RXN target, incremental0, dev/test
debug0 and Root's existing pinned interpreter via BONDLINE_PYTHON. No install or shared
target was used. All old notation/parser/normalizer/writer/layout and oracle bytes remain
unchanged; removing only the new module/re-export lines reconstructs the original source.

- Missing API RED: `cargo test --locked -p bondline-io --test smiles_file`, session 62097
  exits 101, E0432 missing SmilesFileDocument/file; `target/smiles-file-api-red.log`.
- Initial execution session 58453 exits 101, 24 library passed and 7/8 file tests passed.
  My literal `C(` expectation incorrectly used the branch-open position. The unchanged
  parser first reports the missing atom at local EOF2, giving absolute5, not4. Only the
  new expectation was corrected after reading that existing priority; added `C(C` and
  `C1` cases retain independent opening-position checks. The unfortunately named
  `target/smiles-file-first-green.log` is **not** GREEN or a claimed production defect.
- Focused GREEN: session 63090 exits 0, 8 file tests passed, zero failed/ignored;
  `target/smiles-file-focused-green.log`. The deterministic cancellation unit had already
  passed in session 58453 and is included again in the final scoped IO gate.
- Full scoped IO: `cargo test --locked -p bondline-io`, session 50783 exits 0,
  249 passed, zero failed/ignored plus zero doctests; `target/smiles-file-io-final.log`.
- First clippy session 62549 exits 101 for two unnecessary to_vec calls in new raw-control
  test setup; `target/smiles-file-clippy.log`. Only those copies changed to iter().copied();
  production code/expectations stayed exact. Final affected-file run session 99076 exits 0,
  all 8 tests passed; `target/smiles-file-refactor-green.log`.
- Final warnings-denied all-target IO clippy exits 0:
  `cargo clippy --locked -p bondline-io --all-targets -- -D warnings`;
  `target/smiles-file-clippy-final.log`. Workspace fmt checks both exit 0;
  `target/smiles-file-fmt.log` and `target/smiles-file-fmt-final.log`.

Exact hashes and protected source identities are recorded in the ignored frozen handoff.
No Git, native filesystem implementation, command, GUI/input/
activation, native application, new Windows/Linux execution, hosted workflow, full-workspace
gate or container fuzzing ran here. Root owns integration and remaining verification.
M0/M1 and complete SMILES/OpenSMILES/chemistry/source-backed application parity remain incomplete.

## Root receiving and bounded container fuzzing

Root received the exact seven-file codec source3745908 as ca11b2f after independently
rehashing7 intentional files,9 logs,3 baseline/diff records,133 protected files and2
research records. Removing only the new module/re-export lines reconstructs the entire
old notation source exactly. Root's review found no blocking issue; its ignored receipt
is target/smiles-file-codec-root-review.json. A trailing blank EOF line in the new
open-question note was removed during receiving; no codec/expectation bytes changed.

The new isolated smiles_file_parse target uses the existing cargo-fuzz0.13.2,
libFuzzer0.4.13 and nightly-2026-04-01 pins with AddressSanitizer. Root35818 exits0:
10,000 runs, fixed seed143812957, max_len65536, timeout5. Eight initial seeds are the
six unchanged original notation files plus two original inline whole-file vectors.
Runtime mutations stay in a fresh ignored corpus; committed fixtures remain immutable.
Actual largest input52B, final231entries/4481B, cov671/features1977/RSS87MiB,1second.
The ceiling does not establish large-limit coverage; the independent unit boundaries
remain separate. Accepted mutations check complete source/index sharing, alias accounting,
every meaningful physical line and all spans/ordinals, record rereads/counts and absent
ordinal refusal. No external chemistry consumer runs on every mutation.

The final isolated target clippy and whole-fuzz rustfmt checks both exit0 after formatting.
No dependency/lock/parser/normalizer/writer/layout/old oracle/golden change or native/hosted
execution was involved. Reproduction is in fuzz/README.md; receipt and seed/log hashes
are under target/smiles-file-fuzz-receipt.json. ASan log is 49782B/SHA256
5bacacf536c4d0f2236a7191469132b66c4f1ba09876af8d0f41cd299a7af971. This is a bounded smoke check, not a security audit.
The full receiving workspace gate and application/file adapters are separately recorded
when integrated; neither is established by this parser run. Actions stays disabled.
