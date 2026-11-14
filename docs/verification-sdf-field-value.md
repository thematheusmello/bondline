# Immutable SDF field-value replacement

2026-11-14. Library feature on the isolated `bondline-mol-v3000` checkout, based on
owned-source commit0ae10e5. It adds only replacement of one existing field's value.
No Engine/history, application command, filesystem, native UI, record add/delete/reorder,
molecular edit, field-header edit, dependency or schema is added. Original Rust remains
MIT OR Apache-2.0.

## Contract and source preservation

`SdfDocument::with_field_value(RecordId, zero_based_field_index, &[&str])` returns a new
validated immutable snapshot. Addressing is by record identity and ordered occurrence,
including duplicates and DT-only fields; it never rebuilds a name/property map. Missing
positive record IDs or field occurrences return typed MissingRecord/MissingField errors
with code `not_found`. Every prior parser/writer error and code stays unchanged.

Identical logical value lines return a shared clone of the original source and index,
preserving original CRLF exactly. For an actual change, each new nonempty value line
uses LF. Only the old value range is replaced; the exact header/name/textual DT metadata,
first blank terminator, extra padding, other fields, every MOL byte/version, other records,
delimiter suffix and final-EOL state remain untouched. Record count/order/IDs stay exact.
An empty slice means an empty value; spaces and tabs remain data. Empty strings, embedded
CR/LF, other controls, non-ASCII, lines over200 bytes or a column-zero dollar prefix refuse.
Header-looking lines and a space before dollar signs remain ordinary supported data.

Whole-source line/byte budgets are preflighted before allocating the output, including a
final unterminated delimiter line. The completed candidate is fully revalidated with
the existing parser. The original remains immutable on success and failure. New snapshots
retain their complete changed source and metadata; the existing allocation-sensitive
accounting still includes both when a caller retains original and candidate. No application
baseline is invented: retaining originals for Undo/saved/original-source state is still
the next Engine integration.

The old writer's ASCII/control/empty/injection/length validation body is factored into
one private helper and called unchanged. No imported source is passed through the wrapper
writer. Borrowed parsing/span code, original tests/fixtures/oracles and all format limits
remain exact.

## Actual TDD and independent vectors

Before production changes, the literal second-DUP edit test failed because the method
did not exist (E0599 plus consequent inference errors), session19729 exit101,
`target/sdf-field-edit-red.log`. Expanded error/ownership/quota/consumer tests also failed
before implementation in `target/sdf-field-edit-contract-red.log`, session23515 exit101.
These are missing-API REDs, not invented runtime/parser failures.

A separate stdlib splice before implementation declared these exact expectations in
`target/sdf-field-edit-literal-vectors.json`:

| Original | Selected zero-based occurrence / old value bytes | Expected new source |
|---|---|---|
|690-byte V2000 vector|record1,field5,[383,387) → `revised\nmultiline\n`|704 bytes, SHA256`a38749176c76066c2326853f55974f4d450c34bb03fa4f25c9adbf1b2477358b`|
|777-byte mixed vector|record1,field2,[378,385) → same literal|788 bytes, SHA256`aae285d9b6f66717b9dc7da7da0cfbb9bc8edf65d94f3ea32f95a0c81fd4e5b1`|

Tests compare the entire output with those literal prefix/replacement/suffix bytes,
all other-record/MOL/header/DT/value content, unchanged originals and stable IDs/versions.
They also edit record2 and a DT-only occurrence, assert same-sized source-only equality
changes, and verify exact empty/space/tab/multiline/injection-boundary behavior.

The CRLF source is730 bytes: only [408,413) changes, to LF-authored values; the original
CRLF blank terminator remains, producing743 bytes. Extra EMPTY padding and the other
record remain exact. A separate no-final-LF source retains that final state. Logical
no-ops on CRLF, empty fields, mixed encodings and the second record share both Arcs.

Quota tests reach exactly1,000,000 logical lines and refuse the next line for both final-
LF and unterminated inputs. Another test provides a replacement below64MiB whose complete
candidate exceeds64MiB, requiring ByteLimit. Length199/200 succeeds and201 refuses. Missing
record/MAX/field, malformed lines and quota failures leave the original snapshot exact.
A syntax-valid opaque MOL record remains source-authoritative despite model projection
refusal. Allocation accounting distinguishes genuinely new source/index payloads from
no-op shared clones.

Minimal implementation first passed all9 then-present tests, session54641 exit0,
`target/sdf-field-edit-first-green.log`. The final second-record/DT and EOF-boundary
coverage brings the focused suite to10. All178 IO tests pass,0 failed/ignored, session20390
exit0, `target/sdf-field-edit-io-tests.log`; warnings-denied all-target IO clippy passes,
session78985 exit0, `target/sdf-field-edit-io-clippy.log`. Rustfmt passes. A subsequent
method-doc sentence clarifies the zero-based index/empty-slice contract; final aggregate
checks include that exact source.

## Independent consumers and limits

Actual edited owned LF/CRLF output goes to unchanged `sdf_oracle.py`: independent raw
span scanning, ordered duplicate/DT values and separate pinned RDKit2026.09.2 consumption
of both molecular blocks. The LF output must match the predeclared704-byte hash; the
original first DUP and DT0009 remain exact. The known DT-only full-supplier data/record
loss stays asserted as a diagnostic, never a preservation oracle.

Actual edited mixed output is checked by the complete literal byte splice; its unchanged
V3000/V2000 MOL slices go to the unchanged individual molecular consumers. The V2000-only
raw scanner is not relabeled as a mixed scanner, and its printable-ASCII subset is not
broadened for tab tests. Tabs have independent literal source expectations. All original
scripts and pins remain exact; missing consumers fail rather than skip. External CC/C2H6
results are not Bondline chemistry calculations.

`target/sdf-field-edit-preservation.json` records exact parent equality for borrowed
parser/span code and exact old-writer/shared-helper validation bodies after removing
indentation. Removing only the new owned method/imports reconstructs the prior module
SHA256`e2c45b1d980554eaf98fb12f2a63b8fae3ef2325a3748a200e585b206d7a715b`.

## Reproduce and continue

```sh
export CARGO_TARGET_DIR=/Users/sarenaberg/Workspace/doozle/open-source-world/bondline-mol-v3000/target
export CARGO_INCREMENTAL=0
export CARGO_PROFILE_DEV_DEBUG=0
export CARGO_PROFILE_TEST_DEBUG=0
export BONDLINE_PYTHON=/Users/sarenaberg/Workspace/doozle/open-source-world/bondline/.cache/oracles/format-python/bin/python
cargo test -p bondline-io --locked
cargo clippy -p bondline-io --all-targets --locked -- -D warnings
cargo fmt --all -- --check
cargo xtask ci
```

The final own `cargo xtask ci` passes556 workspace plus3 native bridge tests,559 total,
0 failures/ignored, session64629 exit0. Rustfmt, warnings-denied workspace all-target
clippy, layers, unchanged154/165/220 runtime license closures/two assets and the existing
five pinned files/four source bundles with independent corpus checks all pass.
`target/sdf-field-edit-ci.log` prints the correct absolute `bondline-mol-v3000` root and
ends successfully,69242 bytes, SHA256
`907741752aac99ed87a9ecc9b626f7361a7d225195303f0360fa8a09a9b13794`.

Frozen source: `sdf.rs`24456 bytes/SHA256
`fbdb2b3e0b96c9849778c6f23a0dbad9975a74df5d60127915cfeac6055d9b2c`;
`sdf/document.rs`8024 bytes/SHA256
`5f19e520afa8bfd7860d29289c80efc7019a80f8a772b817ddf656a32c3baf44`;
new focused tests18404 bytes/SHA256
`1eda93076acc71f11cf48c3f008380bafccd922195156ae4a3a59f0bba5da2df`.
Every old test/oracle/fixture, both locks, native-v0, UI attachment source and approved
golden remain exact. No owned job remains; no GUI or Git action was performed by this worker.

This is not application SDF editing, native parity or M1 completion. Whole-content current/saved/history ownership,
revision-checked shared commands, atomic filesystem publication, field disclosure limits
and native presentation/acceptance remain separate. The installed reference observation
does not establish existing-field retention/editing or export behavior; LF authoring and
this source-safe replacement contract are original Bondline policies.
Receiving review/integration and this source's own hosted checks remain pending.
