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

## Corrected native acceptance, 2026-11-14

Completed00:04:57UTC on macOS27.2 arm64/Apple M3 Max/Metal, source
1c533127e57825368e323b438b93e4db022ce0c8, complete Rust/Cargo manifest digest
3f07aa401d6defa7ea2524d1afb305da6774362b099d5227d64792e9abf5c824,
binary59ca30c81a16b8e5eccb9564ffd338328e70fd50a167cb71d2f3fdddad9ed8b0.
The earlier6add… partial run remains separate and was not substituted.

Actual native actions passed isolated C–C authoring→explicitV3/full-precision save→reopen;
the two-choice Open family and three Save As encodings/defaultV3; original344-byte Unicode
copy/known Save/headers; misleading.cdxml decoder refusal then explicit family override;
malformed retention/Cancel; V3→V2 Replace/refusal/retry preserving source and destination;
V2000 known Save/default and explicit V3 promotion; empty V3→first fragment with all three
headers; dirty CmdQ Cancel then Save/exit and fresh reopen. The successful native close
uses the new current-completion acknowledgment. Nine actual files face unchanged external
RDKit and separate exact-byte/header/Decimal checks; authored raw coordinates agree within
1e-12 MOL units and reopened model positions agree exactly in this scene.

Saved native330-byte file SHA168a36fa864a496ad575e227b830b6e18b72a4daa91f2fbc0e54b81273b3b255;
promoted313-byte file SHA2636354e0e85e7347b8e4d3839185560e9ac887bbf0de37b835c64cb80fe8eac;
edited empty381-byte file SHAc49b978911ceb9a332cd3b06d9e9b8a7d159965925132cd0cb8c190a2a96a335.
Fresh actual concurrent2048 CPU/Metal GPU captures remained pending while Close wasApproved;
both complete PNG replies then arrived (SHA516e56cc92ee13849eaaa19fb72ed3e95c22867063925327554b34d95309a3f4)
before exit0. A prior4096 probe was correctly invalid_params and is excluded from evidence.
The deterministic save/close ABA races remain core/UI test evidence, not native timing claims.

All three native processes91954/3402/8626 and controllers34220/1456/92948 exited0; CUA completed
exactly once. Credentials remained memory-only. Root target/native-v3000-corrected-handoff.md
and sibling target/native-v3000-corrected-ui/verification.json record all hashes, seven
untouched window captures and cleanup. No Rust/fixture/oracle/lock/font/golden changed.
Hosted application run43531603511 at1c53312 is running; interactive Windows/Linux and general
MOL graph editing/rendering, queries/stereo/chemistry, arbitrary lossy export and full M1–M5
remain missing or unverified. This is native verification of the stated supported subset.
