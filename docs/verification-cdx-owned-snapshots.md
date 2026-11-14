# Owned CDX source snapshots

Verified locally2026-11-14 in the receiving root after1512532's independent projector review.
This is library/model backing, not application CDX Open/Save, chemical perception or an
editable CDX profile. Source metadata remains authoritative and unchanged.

## Behavior

StoredDocument::from_cdx owns the complete Arc<[u8]> and the limited elemental model.
The private preview helper creates one caller-sized page and appends validated fragments;
no physical paper/print settings are inferred from a bounding box. The parser supports at
most one direct page; unknown source remains raw. All imported CDX is_fully_mapped=false.
The existing UI/capture/edit guards are not relaxed by a successful elemental view.

export_cdx/to_cdx borrow exact unchanged backing. There is no model encoder or conversion:
changed models refuse UnsupportedCdxEdit, source-free snapshots refuse NoCdxSource, and
CDX→CDXML/MOL refuses metadata loss. Equal detached models still export cleanly. Source
info reports familycdx/bytes/modified without raw data; Debug has counts/source info only.

Full snapshot equality includes raw bytes/baseline; opaque-only edits cannot disappear
behind equal projections. Shared raw/model/fragment storage and the original baseline join
the existing retained_bytes accounting, including modified snapshots. It remains approximate
payload/container accounting, not an allocator/RSS cap. No history/registry/file association,
file worker, close or save-reservation code changes here.

## Test-first and independent checks

Actual first RED target/cdx-owned-document-red.log exits101: from_cdx/export_cdx/to_cdx and
source-specific error variants are absent. The first six tests pass after implementation
in target/cdx-owned-document-first-green.log. Literal original legacy/header fixtures supply
signed points(10.25,-3.5)/(26.25,8.5), fresh IDs1/2 and authoredHNone; their existing independent
Python struct/RDKit checks are unchanged. Snapshots face exact pointer/byte equality, typed
source JSON, immutable clone restore, no-op detachment, opaque-source inequality, memory
sharing/baseline retention and stale/cross-format refusals. The seventh test checks real
syntax/unsupported-NumHydrogens/input-budget errors through the wrapper; it is additional
regression coverage, not a separately claimed RED.

All144 IO tests pass in target/cdx-owned-document-io-tests.log. Root's actual aggregate
passes438 workspace plus3 macOS accesskit bridge tests: rustfmt, warnings-denied all-target
clippy, locked tests, layers, native licenses/assets and five-file/four-bundle corpus. Log
 target/cdx-owned-document-ci.log, session56791 terminal0. Adding only the internal IO→CDX
edge raises native runtime package counts to154Mac/165Windows/220Linux; original CDX is
MIT OR Apache-2.0. Both root/fuzz lock diffs are only that edge; no registry version/checksum,
license policy, bundled asset, old fixture/pin or approved golden changes.

Independent read-only review target/cdx-owned-snapshot-review.md
SHA96c026c69f207d0a9a4ae140330389c1cac1aa8d94692c1de54c3a44abb9b7ad found no confirmed
source/accounting/export/privacy regression. Earlier projector review is separately recorded
in target/cdx-projection-integration-review.md, SHA04c63c05fcb03bddfeb4fbdcad4ac713a58ce425115454adfc10e73e068534e5.
Reviews read the source/evidence; their authors did not claim a second aggregate execution.

## Normal adapter fuzzing

The existing CDX target now checks successful elemental views, dense IDs/local endpoints
and exact source, then owns a snapshot, mutates preview size, requires stale/cross-format
refusal and restores the complete original. Original syntax/property/new-tree invariants
remain. The same target count remains six; no new fuzz package/dependency is introduced.

Normal full-graph warnings-denied clippy/fmt pass. Fresh four seeds are the three unchanged
original70/65750/65746-byte syntax fixtures plus a92-byte independently packed default-C/
default-single/no-position/no-H vector, SHA
d1d830c92ca3b6d56a4be418a3f7353369242ba869f084c52be1ef39fed535d0.
They live only in ignored .cache/fuzz/cdx-owned-source-20261114; no fixture directory is a
writable mutation corpus. Exact settings: pinnednightly2026-04-01/cargo-fuzz0.13.2/
libfuzzer0.4.13, ASAN10000, seed721533879/max_len131072/timeout5. Session70497 terminal0,
1second after compilation, coverage1019/features2161,81retained inputs/reported394KiB,
RSS303MiB, maximum exercised65750bytes. Log target/cdx-owned-source-fuzz-asan.log,
SHAe812d3ce32f4fdf5eb7c04bbbaf76f8b2ea8e4a27e8e0865db7dcc4bf107e719.
This is bounded smoke evidence, not exhaustive conformance or a security/RSS guarantee.

## Remaining scope

The actual native1196-byte output's Z/BoundingBox/CIP/AtomID/warning metadata is still outside
this strict graph whitelist. The standalone syntax parser preserves it; this model bridge
explicitly refuses unsupported content. Native research outputs remain ignored, not corpus.
MOL/CDXML workflows keep their original guards and bodies. CDX native filesystem/shared
commands/UI, broader metadata/bond orders, source-faithful rendering/editing and a model
encoder require further independent features. Integrated00e5b02 now passes hosted43550955963
on all three platforms and all six bounded sanitizer targets. Earlier5e6b644's Windows
socket assertion failed; the test-only portability correction and exact counts/adapters
remain in ci.md. No snapshot/data-preservation guard was weakened to pass that run.
