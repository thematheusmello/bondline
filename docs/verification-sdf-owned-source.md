# Owned SDF source: library verification

2026-11-14. Isolated `bondline-mol-v3000`, branch `feat/sdf-owned-source`, basef0910ca.
This is the immutable whole-source prerequisite for M1 SDF application work. It adds
no document layout, active-record browser, native filesystem, Engine/history migration,
registry command, structural/data edit or UI. Original code is MIT OR Apache-2.0.

## Owned contract

`io::sdf::SdfDocument::parse(Arc<[u8]>)` validates every record with the existing bounded
SdfFile parser before returning a snapshot. The whole source and compact RecordInfo
slice are separate immutable Arcs. Clone shares both; equality includes the complete
source bytes. Debug prints byte/record counts only, not titles, molecular data or fields.
`as_bytes()` borrows the complete unchanged source; it never calls the wrapper writer.

`records()` and checked `record(RecordId)` expose ordinal IDs1..record_count, complete
record/MOL/delimiter ranges, actual molecular encoding and field count. IDs are positive
and stable for these immutable snapshots/clones. They are neither molecular IDs nor
persisted SDF fields, and do not promise identity across a different source/reopen or
future editing. No record is selected, flattened, projected or silently omitted.

`read()` returns the existing borrowed SdfFile view on demand, including ordered fields,
duplicate names, textual DT digits and raw/header/value spans. It reparses under the
same limits rather than storing a self-referential parser. Syntax-valid opaque MOL data
remains usable even when elemental model projection refuses. Invalid later molecular
syntax, unsupported encoding, incomplete fields and exceeded limits reject the whole
constructor. No parser/writer behavior or profile limit changed.

`retained_bytes()` deduplicates repeated snapshot references and shared source/index
allocations, counts each distinct inline snapshot plus exact Arc slice payloads, and
saturates arithmetic. Distinct equal byte allocations still count separately. Temporary
borrowed views, allocator/Arc headers, external copies and caches are excluded. This
is a component for later Engine accounting, not an Engine integration or hard RSS cap.

## Independent expectations and actual RED/GREEN

The original fixtures remain byte-identical:

| Source | Bytes / SHA256 | Literal expected records |
|---|---|---|
| ordered-fields-v2000.sdf |690 /`f95838cbb55c107c649ce2eb99ac62e191f747ce6994a171e64019f8e6c53d1a`|[0,424),[424,690), both V2000;7/0 fields|
| mixed-v2000-v3000.sdf |777 /`165e6f5d5cff37dae6315a07843d84733aa42dbbc497b2920da1702bad9526a0`|[0,462),[462,777), V3000/V2000;6/1 fields|

Their existing README records independent original construction, license and public
format sources. A separate stdlib line-offset inspection confirmed all literal spans
before implementation. No vendor/sample bytes or new fixture were introduced.

- `target/sdf-owned-source-red.log`: first literal whole-source/record test failed
  compilation because SdfDocument/RecordId were missing (E0432 plus consequent inference
  errors), session43261 exit101. This is the missing-API RED, not a runtime parser failure.
- `target/sdf-owned-source-contract-red.log`: the expanded ownership/accounting/error/
  consumer contract likewise failed before production edits because the API was absent.
- `target/sdf-owned-source-first-green.log`: minimal implementation passed all9 tests,
  session54710 exit0. Literal per-field ranges and complete independent raw-span comparisons
  were then added without changing the implementation.
- `target/sdf-owned-source-io-tests.log`: all168 IO tests pass,0 failed/ignored,
  session78324 exit0, including all9 new ownership tests and every existing test.
- `target/sdf-owned-source-io-clippy.log`: warnings-denied IO all-target clippy passes,
  session22768 exit0. `cargo fmt --all -- --check` also passes.

Coverage includes caller-buffer lifetime and retained pointer identity; clone source/index
sharing; allocation-sensitive storage accounting; equal metadata with unequal source-only
field values; count-only Debug; every ID through1024 and refusal at1025; absent/MAX lookup;
malformed later V2000/V3000 and data fields; ASCII/control refusal; byte/record/field/line/
physical-line/value budgets; exact CRLF, tab/space values, blank title, delimiter suffix and
post-item padding; and a valid opaque-MOL record that retains every byte while model
projection fails. Original parser tests/negatives and molecular encodings are unchanged.

## External consumer distinction

Actual owned690-byte and CRLF source bytes go to the unchanged `sdf_oracle.py` independent
raw scanner plus per-record pinned RDKit2026.09.2 consumer. Every record/MOL/delimiter and
field/header/value span, header identity, textual DT identifier and ordered value is
compared, including both DUP occurrences. The original DT-only full-supplier loss of
later data/record remains an asserted diagnostic; a last-value dictionary never defines
preservation.

The actual owned777-byte source has separate literal full-span/field/version checks;
its unchanged molecular slices go to `v3000_oracle.py` and `mol_oracle.py`. Both molecules
are consumed independently; V2000 is CC/C2H6. The V2000-only raw scanner is not relabeled
as a mixed-version scanner. All consumer scripts/pins are unchanged, external-only and
mandatory; missing consumers fail instead of skipping. This does not implement Bondline
formula, valence, canonical SMILES or other chemistry perception.

## Reproduce and continue

Use the documented isolated interpreter and this checkout's own target:

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

The stable own `cargo xtask ci` passes546 workspace plus3 native bridge tests,549 total,
0 failures/ignored, session45670 exit0. It includes rustfmt, warnings-denied workspace
all-target clippy, layers, unchanged154/165/220 declared runtime closures/two assets,
and the existing five pinned files/four source bundles and independent corpus checks.
`target/sdf-owned-source-ci.log` prints the correct absolute `bondline-mol-v3000` root
and ends successfully,68636 bytes, SHA256
`5ece882b18f38fe4a405e325dc7150f274d01390908f9e558508fd77fa24eb0d`.

Frozen source module is5121 bytes/SHA256
`e2c45b1d980554eaf98fb12f2a63b8fae3ef2325a3748a200e585b206d7a715b`;
focused tests19642 bytes/SHA256
`9dd667dd4ac576ea71bde3588bb72e39a8764660ad3a3166cdc6c78ba2c6e374`.
The existing borrowed parser/writer source reconstructs its prior SHA256
`f38031d5b4258fe1217a71effb9723e00abee140b6788975990e4493204dc378` after removing
only the new module/reexport lines. Both locks, all old tests/fixtures/oracles, native-v0,
UI attachment and approved golden remain unchanged.

Next work must retain this whole collection in current/saved/history snapshots before
application SDF Open/Save or edits are exposed. The qualified installed26.1 import
observation shows both drawings in one document, not an initial record browser; grouping,
fields, layout and export remain unknown. This library chooses none of those policies
and does not establish application SDF compatibility, native verification or completed M1.
Receiving review/integration and this source's own hosted checks remain pending. No
Cargo/native/GUI job belongs to this frozen handoff.
