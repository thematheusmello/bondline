# SDF camera — integrated evidence

2026-11-15, branch `feat/multiple-bonds`, built over `af7fab0`; renderer foundation
`06ab5a9` is an independent checkpoint. This delivers original Bondline read-only SDF
navigation and opt-in current-view capture, not complete Professional screen/flow parity.

## Contract and source preservation

SDF Actual/Fit/SetZoom need no invented page bounds. Pan and relative-from-Fit require
the exact extent derived from every record; explicit relative zoom uses its known percent.
`render.capture` gains only optional closed `view:{logical_width,logical_height,
expected_view_epoch}`. Absent/null retains exact legacy whole-page pixels. Opt-in work
captures camera/source/close context and generates fresh physical geometry through the
shared renderer. Default native Fit stays legacy; explicit native zoom opts in. Headless
logical Fit is not equated to legacy physical-margin Fit at higher density.

Only successful render/encode can install Weak CaptureStamp + PageSize, guarded by
source/revision/generation, view epoch, exact Open/close context and cancellation. Errors
install nothing. Historical successful images cannot prepare another source or Engine.
No Page/source lease/model/history/Save/ACK authority is created. Normal JSON snapshots
remain unchanged; explicit view inspection conditionally adds prepared_sdf_extent.

Native capture preparation also checks the pending close epoch before reservation. A
Rust-only first-installation flag wakes the native host without entering JSON; repeat
captures do not report a camera change. UI keys include view epochs, preserving freshness
through source/view/DPI A→B→A. Collection input runs before image painting/scheduling.
Drawing and SDF use one wheel calculation with paper/foreground/focus/modal/context guards.
Every molecular record remains validated even outside the current crop.

[Renderer](verification-view-raster.md), [seven command cases](verification-sdf-view-command.md),
[four actual-App cases](verification-sdf-view-ui.md) and
[read-only core review](research/sdf-view-core-review.md) retain their separate inputs,
hashes, declarations, RED/GREEN and limits. The original proposal plus receiving decision
is [here](research/sdf-view-plan.md); success-only bootstrap replaced its error-side-effect idea.

## Independent checks and corrections

- The frozen App positive first reaches a real label-scale refusal, then fails at the
  disabled Actual control. The command positive independently fails at WrongContent for
  SDF Actual. Minimal camera support then reaches a second Root RED: unknown capture
  `view` field. No future hook or fabricated success replaces these runtime failures.
- Root's first App integration attempt enabled the menu but omitted its selected action
  from early collection replay. The frozen test failed at null versus100%; adding that
  action to the existing ordered route yields GREEN. The later wheel case first fails
  at unchanged center100/144 versus declared124/168; shared math then passes repeated
  line/point events, zero/modifier/outside/text-focus and real picker gates unchanged.
- Seven command cases cover literal1x glyph/chord regions, successful bootstrap, exact
  twelve-record1368pt extent and first/middle/last views, late unsupported off-camera
  content, closed schema/null/legacy bytes, source/view/close/foreign/cancel guards and
  complete source/Save/export/native-ACK limits. Root's first run is6/1 because a new
  image assertion incorrectly assumed stacked H above O. Existing horizontal-first
  policy, independent pinned fontTools/FreeType/Pillow evidence and actual pixels agree
  on left H. Only that new box moves to the independently calculated region; the old
  above box becomes an additional empty negative. Every other assertion/threshold/input
  stays exact. Root59767 and independent62216 then pass7/0. This is an oracle setup
  correction, not a renderer behavior fix or relaxed image requirement.
- Four App cases require fresh real texture uploads across view100→150→100, DPI1→2→1,
  source740→literal749→740, and NOTE draft Apply→fresh preview→real Save/reopen. The last
  originally compares848×651 before opening Records with848×603 afterward. Existing
  metadata rows intentionally consume48 logical units. Only its new baseline capture
  moves after Records expansion; exact image equality and whole-file/consumer checks
  remain. The original failure/source are retained; final independent84986 passes4/0,
  UI all-target clippy and rustfmt. Both earlier6273/14105-byte prefixes remain exact.
- Root's new authenticated-loopback test reaches real PNG/extent success but observes
  zero host notifications instead of one. Adding the first-installation notification
  passes; repeat/legacy captures stay quiet and state/source remain exact. A second
  missing-API compiler RED precedes the minimal close-epoch preparation overload;
  Root22454 passes both native-adapter cases with real shared worker reservations.
- The prior source-guard test deliberately expected SDF Actual to refuse. Root46037
  records5/1 after the new requirement is implemented. That one SDF block is explicitly
  upgraded to accepted Actual plus exact complete state/content/stamp, no Drawing,
  unprepared-pan refusal and exact raw Save assertions. Other format/refusal/whole-page
  expectations are unchanged. Root97142 passes6/0. This is a capability-contract upgrade,
  not a hidden skip or weakened source guard. Three old Rust capture literals gain only
  `view:None`; old parent test modules gain only child declarations.
- Workspace clippy first flags the new routing tuple's type complexity. Named private
  ShortcutRouting preserves the same fields/callers; no suppression or assertion change.
  Root45116 passes corrected all-target clippy. Whole UI7759 passes122/0/1 before the
  two additive cases; final aggregate below includes all four.

One further positive Drawing case was added after implementation, without an invented
RED: ordinary source-backed MOL ethanol/methylamine retain their original, unpacked
coordinates and H=None through current-view PNGs at1x/2x, independent Pillow regions,
unchanged actual saved MOL bytes and existing external consumers. Root32887 passes first
run. Its3738-byte declaration SHA256 is
`263cec29e5131b67a06e4f628bbf02cb6d928d3bec20b9f95daf6fb30e36f11d`.
The original MOL parent remains exact apart from its appended child declaration.

## Root aggregate

```sh
BONDLINE_PYTHON="$PWD/.cache/oracles/format-python/bin/python" \
  CARGO_INCREMENTAL=0 cargo xtask ci
```

Final Root77879 exits0: **1016 passed,0 failed,1 ignored**,149 result lines including
three macOS AccessKit bridge tests. Rustfmt, warnings-denied all-target clippy, strict
layers, licenses and corpus pass. Log`target/sdf-view/root-ci-final.log`:121562B, SHA256
`46eae4838170cca5da3621624ba4ee422ab1a7d89d47a6be2ba0798ca0c69294`.
Earlier91855 passes1015/0/1 before the additive Drawing case. The visual-capture utility
remains ignored; no new golden approval, fuzz or hosted/interactive Windows/Linux run.
Runtime declarations remain154/165/220 and two assets; corpus remains eight files/five
bundles, with foreign/general CDX and general chemistry coverage explicitly missing.
Dependencies/locks/SDK/font/original fixtures/oracles/approved golden are unchanged.

An additional actual stdio-SDK check uses the built CLI binary SHA256
`04d67e747d799f63dac64330166ad68acbc7ecefe2c756a7de9931cd4be18388`.
Initialize→tools/list discovers28 tools and the new nested view schema; actual tools/call
Open→inspect→Actual→capture→pan100/144→capture succeeds, obsolete epoch returns structured
stale_view, and final normal state equals the pre-camera state. Its13044-byte PNG/SHA256
`818dcfe40006c7b4b65611ada1410343f2a6b6cd13424652d283de90e2828b28`
equals the independently checked CLI frame. EOF exits0. Ignored
`target/sdf-view/manual-stdio-receipt.json` records this bounded manual adapter execution;
it does not add an automated test or native-window claim.

## Finite native Mac acceptance

Ignored `target/sdf-view-native/Bondline.app`, identifier `org.bondline.sdf-view-review`,
PID51412, actual executable21770456B/SHA256
`938289e6371421cc61260758d2c7404e5b3d9e14612d359589622c1e0a494adf`.
Root32589 builds it; build receipt records exact product source hashes. Older previews
are untouched. Desktop opening follows the user's explicit instruction, not a fabricated
availability reply. Native keyboard/chooser opens an ignored740-byte original copy, SHA
`e5cbdc14329c59bad55148c5de37058de90f651af10966c30cf8b89b3b5efd39`.

The initial Fit visibly reports the existing em refusal. Native F5 sets Actual100 and
produces a real CPU-rendered chemical texture composed by the Metal app, plus pan controls.
This is not a native GPU chemical-rendering claim. Pointer clicks on the two actual
sliders set horizontal100 and vertical143; the native screenshot visibly contains both
records, ethanol with OH and methylamine with H2N, respecting original bond directions.
It is a viewport crop of unchanged source paper, not a repacked/resized authored document.
Read-only loopback inspection confirms revision1, clean740-byte source, Undo0/Redo0,
camera epoch3/center100,143 and prepared full A4 extent. It did not perform navigation.
Original input bytes remain exact. Actual monitor DPI was not measured; native100%
acceptance is not native1x/2x hardware proof.

Native slider SetValue with string100 makes no observed change; pointer clicks work.
The provider-versus-platform cause and general numeric accessibility remain unestablished.
CJK filename text appears as question-mark glyphs in the screenshot while AX/path bytes
remain correct; wider font coverage is not delivered. Initial unprepared Fit can still
show the old100% numeric fallback; F5 is the observed explicit state change, not that label.
Relative-from-Fit uses logical fit math, while the legacy native overview uses physical
margins; their scales can differ above1x. Native relative-step consistency is not verified.

A subsequent CUA scroll is aborted before dispatch; an immediate read finds no state
change. Native input stops there. Availability clarification is pending; no elapsed-time
approval is inferred. Native wheel, combined field-edit/preview/Save/reopen, close/cancel
races and physical-DPI measurement are unverified in this build. The combined749-byte
edit/preview/file workflow is real-App evidence only. Its literal first-NOTE replacement
`source[..380]+b"reviewed ethanol\n"+source[388..]` has SHA256
`54cdb6721c80ad19afb891e973fed5f18977ac11e2c5335faab856443837b61d`;
the unchanged per-MOL consumers check actual saved output.

No native/chemical/complete-M1 claim extends beyond these scopes. Source/control lifecycle
and other platforms still require work. GitHub Actions remains disabled and M0–M5 active.
