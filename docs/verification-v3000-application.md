# V3000 application verification

2026-11-13. This feature follows standalone codec5caf6ce in the persistent
bondline-mol-v3000 checkout, with its own absolute Cargo target. The public source is
CTFILE-2026, printed7–13/32 (PDF indices10–16/35), as pinned in clean-room/sources.md.
Version selection, conservative conversion refusal and drawing placement are original
Bondline policies. No proprietary application/binary/assets were used.

## Actual test-first evidence

- IO missing-API RED: target/v3000-document-red.log. A finite but poorly conditioned
  frame also lost raw0.0001 coordinates when upgrading V2000: the old four-decimal
  proof allowed it. target/v3000-conversion-precision-red.log records that failure.
  A distinct strict V3000 raw-coordinate proof now refuses it. Conversion-message RED
  ensures an unchanged-source refusal does not falsely claim the model was edited.
- Core actual RED: target/mol-versions-command-red.log, unknown version selector and
  missing version/schema/actual-V3000 decode. Four independent behavior/schema/receipt
  tests become GREEN in target/mol-versions-command-first-green.log; the full core
  suite passes target/mol-versions-core-green.log and warnings-denied clippy.
- Adapter schema RED: target/v3000-adapter-schema-red.log. Actual CLI9, stdio5 and
  authenticated loopback15 tests pass. The new wire tests initially expected an unknown
  V4000 on-disk marker to be unsupported_content; the existing InvalidField contract
  correctly exposes invalid_mol. Only that new expectation was corrected. This is not
  a production fix or the substantive RED evidence; all state/history/byte assertions stay.
- UI missing-version/helper RED: target/native-v3000-ui-red.log. All43 UI library tests
  and all-target clippy/fmt pass. A first test expectation used an uncanonical temporary
  path; it was corrected against fs::canonicalize without changing product behavior.
  Egui/input tests alone are not native execution evidence.

All107 IO tests pass, including independent external8dp/1000-atom read/edited write,
exact original bytes, empty-to-first-fragment header retention, conversion/error
messages and every seven-stage injected atomic-publication failure. Frozen production
concat SHA (mol/mod.rs,document.rs,files.rs):
a8505a8b6382ca6eaf5fea11d61904ac900b083f1b54a046bb2eb55030f43c31.

The actual CLI/stdio/socket files face the unchanged external RDKit2026.09.2 consumer.
Native-created V3000 has authored C/H3/single connectivity and raw coordinates within
1e-14 of the explicit frame conversion; reopened drawing points agree within1e-10.
The original sparse-ID344-byte source auto-saves exactly (SHA
1867a5904e742030431b523a31167951ff998007a89b9c3d94ec88cf7d4c4380).
Rejected downgrade/malformed/version/selector paths retain complete inspect state,
redo and old destination bytes. Receipt-only saves request repaint without an authored
edit. Same-path version changes invalidate a pending picker; a known-Save request
cannot change its accepted version. Native saves do not poison later CDXML export.

## Full local gate

The first aggregate stopped during compilation when the local disk filled, before
workspace tests completed. target/mol-versions-ci-disk-full.log retains it. Only the
two Bondline target/debug/incremental caches were removed; sources, fixtures, logs and
worktrees remained. The same gate then passed with no source/test change:

```sh
CARGO_TARGET_DIR=/absolute/path/bondline-mol-v3000/target \
BONDLINE_PYTHON=/absolute/path/format-python/bin/python cargo xtask ci
```

target/mol-versions-ci-integrated.log prints the sibling root and passes364 workspace
plus three native bridge tests, rustfmt, all-target clippy, layers, native license/asset
checks and that branch's inherited two-file CDXML/MOL corpus. The newer root SDF/CDX
corpus is a separate branch and is not claimed in this result. Runtime closures remain
153/164/219 on Mac/Windows/Linux, with two bundled assets. No dependency/lock/codec,
renderer/shader/font/golden or filesystem publication algorithm changed.

## Native and hosted status

Partial native acceptance completed23:24:48UTC on macOS27.2/Apple M3 Max/Metal. Exact
binary6addbfb7603e4e40b174a80d672279a1352d99a7989d0a0411fa11c051376a79;
Rust/Cargo source manifestf9a42db06b4543314d83a85d9e7c67269a1864085ba848c67247e1c64f4407ab.
Actual Open family, three Save As choices/defaultV3, Unicode exact copy/known Save,
misleading .cdxml explicit-family decoding and malformed-source retention/Cancel passed.
The344-byte copies match1867a590…4c4380 and an independently run RDKit reads C/H3/single
and all original raw positions. Native/controller processes exited0 and CUA completed.
Root target/native-v3000-ui-handoff.md records the raw captures, exact artifact hashes and
bounded sequence; no golden was changed. This is explicitly partial, not final acceptance.

Independent review confirmed a pre-existing overlapping-save bug: publish oldV2, publish
and apply newerV3, then apply old receipt. The clean association becomesV2 over V3 disk;
the next Auto save rounds coordinates. The target-only compiled probe and independent
file observations are in root target/v3000-integration-review.md. Acceptance stopped and
all processes closed before product fixes. Root is adding a shared per-Engine reservation
with an actual failing regression on fix/save-completion-order. This feature stays local
until that correction and final native/hosted verification are integrated.

Not yet run natively: authored full-precision V3/reopen, downgrade refusal, empty-to-first-
fragment header retention, V2 regression and dirty-close Save/Cancel. Hosted application execution
requires this feature's own source publication. Standalone codec43519338382 is GREEN
on all jobs, but is not application evidence. Interactive Windows/Linux, general imported
graph editing/rendering, SDF V3000 dispatch, broader fields/queries/stereo and complete
M0/M1 remain unverified or missing. One-fragment/page and source/layout guards still apply.

## Save and close correction integration

Root correctiona689b0d is integrated here without changing the version-aware codecs or
frame/metadata guards. The new actual V2/V3 race test first fails in
target/v3000-save-order-red.log, then passes in target/v3000-save-order-green.log. A
published-but-unapplied V2 receipt blocks overlapping V3 before I/O and retains whole state/
bytes. Finishing that receipt releases the save; explicit V3 independently preserves raw
x28/24 within1e-14, and a following Auto Save keeps exact V3 bytes and association.

The root correction also has six lifecycle and one cross-executor regressions. A second
actual UI RED caught historical close results approving after later format/baseline ABA;
private current-completion acknowledgment fixes that post-finish/pre-poll gap. Wire receipt
fields stay historical. All existing path/format/revision/epoch/clean guards remain.
See verification.md for root's356+3 gate and exact RED/GREEN logs. This checkout's combined
source needs its own full gate and a new native executable; the partial earlier native
binary is not substituted. Source/tests/locks are frozen during those acceptance checks.

The corrected actual-worktree gate now passes372 workspace plus3 bridge tests, rustfmt,
warnings-denied all-target clippy, layers, unchanged153/164/219 native closures/two assets
and this checkout's inherited two-file corpus. CARGO_INCREMENTAL=0 was explicit to avoid
recreating disposable caches; target/v3000-completion-ci-integrated.log identifies the
invoked sibling root. Session3849 is terminal0. Newer root corpus coverage is not inferred.
The exact corrected binary's native walkthrough and its own hosted run remain next.
