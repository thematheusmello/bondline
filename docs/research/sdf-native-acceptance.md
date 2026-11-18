# Bounded native SDF preview acceptance

2026-11-15, supplied source `9fc6944`. Preparation and four measured headless frames,
not native acceptance or a new feature test. No product defect was established and no
production/regression test was added. Root alone owns Git and native input; user
availability must be confirmed before desktop interaction. Actions remains disabled.

## Exact inputs and literal expectations

All inputs are original MIT OR Apache-2.0 bytes with their existing fixture notices.
Preserve hashes, line endings and meaningful spaces; do not regenerate a fixture.

| Existing input | Bytes / SHA-256 | Role |
|---|---|---|
|`crates/io/tests/fixtures/sdf-preview/valence-absent-mixed.sdf`|740 / `e5cbdc14329c59bad55148c5de37058de90f651af10966c30cf8b89b3b5efd39`|Primary ordinary source: V3000 ethanol then V2000 methylamine, unspecified authored H.|
|`crates/io/tests/fixtures/sdf/mixed-v2000-v3000.sdf`|777 / `165e6f5d5cff37dae6315a07843d84733aa42dbbc497b2920da1702bad9526a0`|Two ethane records, mixed molecular versions and ordered/duplicate/DT-only data.|
|`crates/io/tests/fixtures/sdf/ordered-fields-v2000.sdf`|690 / `f95838cbb55c107c649ce2eb99ac62e191f747ce6994a171e64019f8e6c53d1a`|Two V2000 ethane records, empty title, duplicate/empty/multiline/DT/spaces values.|

The740-byte primary input has record ranges `[0,410)` and `[410,740)`, MOL ranges
`[0,371)` and `[410,713)`, one NOTE occurrence each, and delimiters `[405,410)` and
`[735,740)`. NOTE values are respectively `ethanol`/`vertical O bond` and `methylamine`.
State must remain `document:null`, complete SDF metadata,2 records and740 bytes.
Record1 is V3000/410 bytes; record2 V2000/330 bytes. Source-local ordinals are not
authored drawing IDs or selectable atom/bond authority.

Literal sheet data from the existing independent vectors:

| Record | Sheet coordinates, pt | Bonds | Separate display H |
|---|---|---|---|
|1, ethanol|C(72,108), C(108,108), O(108,72)|1–2,2–3; Single|3,2,1|
|2, methylamine|C(108,216), N(72,180)|4–5; Single|3,2|

Fresh preview fragment IDs1/2 and atom IDs1..5 are presentation-only; bond IDs1..3
preserve local correspondence. All authored attached-H values remain None. The O has
one attached H and N has two; their chemical anchors and vertical/diagonal source
directions must remain, with no rotation, H omission or attachment to H instead of
the element. The protected horizontal preference places H to the left for these two
vectors. Carbon counts do not require printed carbon labels in skeletal depiction.

The unchanged pinned `mol_oracle.py` was executed on all six actual embedded MOL spans:
CCO/C2H6O/H[3,2,1], CN/CH5N/H[3,2], then four CC/C2H6 structures. The777-byte ranges
are `[0,462)`/`[462,777)`, MOL `[0,344)`/`[462,744)`;690-byte ranges `[0,424)`/
`[424,690)`, MOL `[0,282)`/`[424,685)`. This independently checks molecules, not ordered
property preservation. Use exact bytes and the existing raw-span oracle for that;
RDKit's property dictionary/full-supplier DT-only loss cannot be a preservation oracle.

The existing [display plan](sdf-preview-plan.md), [CTfile provenance](../../crates/io/tests/fixtures/sdf-preview/README.md),
[derived-data evidence](../verification-sdf-preview-model.md) and
[resolved-label evidence](../verification-sdf-display-render.md) distinguish published
format/attachment rules from original24pt/unit, y inversion, row packing and closed
neutral display-H policy. Historical reference observations in installed-sdf-import.md
are version/method qualified; this work inspected no proprietary program artifact.

## Measured default workspace, not a dimension estimate

One ignored source export copied838 current files and appended measurement-only code
after the original six preview tests. No shipped bridge or source was changed. The
real `BondlineApp::update_workspace`, registered capture, decoder and UI were used.
The actual Records header was clicked for expanded cases. Each settled to ready or a
keyed error, with complete source/authored state unchanged and no live preview job.

At1160×800 **logical client** size, origin(0,0), current chrome and default Records state:

| Records | Actual logical preview rect | DPI | Capture pixels | Actual main em, device/logical | Outcome |
|---|---|---|---|---|---|
|collapsed|(64,125)–(912,776),848×651|1|848×651|not returned|keyed glyph8..256 refusal|
|collapsed|same|2|1696×1302|17.638889 /8.819445|ready|
|expanded|(64,173)–(912,776),848×603|1|848×603|not returned|keyed glyph8..256 refusal|
|expanded|same|2|1696×1206|16.268095 /8.134048|ready|

The error is the existing capture glyph-size guard, not an invented blank preview.
Do not report an arithmetic12pt/A4 estimate as an actual main metric when capture
failed: smaller glyph/subscript guards can reject before a metric is returned. Main
logical em and subscript readability are distinct. DPI2 eligibility does not prove
every monitor or physical reading size. Both DPI1 results are honest current-profile
limits, not this plan's new RED or grounds to weaken the guard.

An earlier coarse footer-based estimate was corrected before measurement; these
settled coordinates and outcomes govern. For DPI1 native positive observation, use
the already-tested1600×1400 large-view control or record a separately measured larger
window. Do not claim default1160×800 positive on a1× display. View navigation applies
to Drawing; SDF zoom/pan is not delivered and the header's retained camera value is
not SDF magnification authority.

Session86065 exits0: one instrumented measurement, four cases,64s cold compile and
0.40s runtime. One unused initial output assignment warning belongs only to disposable
instrumentation; no new lint or feature-test pass is claimed. Every original file,
including the original six-test source SHA
`141e6f93ad1bc3d5b9e0d0b7b75ccb58a9651243c464598349ec2716bc713b78`, remains exact.
No full suite or old six-case rerun was performed.

## Native observation sequence for Root

1. Confirm user availability and the actual owned process/window. Record binary/source
   identity, platform/backend, client size/origin and effective DPI. Open an exact copy
   of740-byte input through the real SDF chooser. Bind the screenshot to that file's
   hash and actual state, not merely its filename. Never expose the loopback token.
2. Observe preparation followed by a real image at an eligible size, or the truthful
   current-size refusal. Require both record regions, O+H and N+H2, correct source
   directions, read-only role and absent drawing hits/gestures. A nonblank first
   structure alone fails. Do not mutate paper or omit H to fit.
3. Toggle Records, resize, and where actually available change effective DPI. Old
   pixels/errors must clear at the view change; only the matching view may install.
   Record both current rect and outcome rather than treating disappearance during
   recomputation as data loss. The measured table predicts this exact fixture only.
4. Save/Unicode Save As and reopen the entire source. Compare complete output bytes,
   ordered fields and per-MOL consumers. Use777/690-byte sources for DT/duplicate
   retention; exact hashing and raw order are decisive, not RDKit's last-value map.
   Preview must never authorize CDXML/model export or change SDF into a drawing.
5. Exercise available Undo/Redo after a real field command only if needed for lifecycle
   evidence. `sdf.field.set_value` is currently a command, **not a native field editor**.
   A metadata-only change can legitimately produce identical imagery; identical pixels
   alone cannot prove that source/generation-key invalidation occurred.
6. Record Cancel/Retry and close behavior only when the state was actually observable.
   Do not infer a cancelled worker from an image that finished before the click.
   A close request/cancel must clear old ready/error state and late publication must
   not revive it. Native close at ready state does not prove queued-decode retirement.

## Existing safety coverage and evidence limits

The [six existing actual-frame cases](../verification-sdf-native-preview-tests.md)
already cover full registered PNG→decoded texture→painted quad/UV/clip, exact opaque
RGBA against independent Pillow, both literal record regions/chords, no authoring
targets, source edits/history/identical Open, resize/origin/capped DPI, logical main-em
rejection, queued success/error after source replacement, and close epochs through
actual worker retirement. Controlled finite gates prove races that native clicking
may not reproducibly expose. They were not duplicated or changed here.

The worker is registered under the originating Engine lock before spawning, without
a Save lease. The UI keeps its pending/job barrier through decode and thread retirement.
The key includes full weak content/revision/generation, close epoch, logical rectangle,
physical capture and DPI. Root should retain this headless evidence separately when
recording native compositor observations. No field UI, persistent record selection,
general layouts/chemistry, SDF navigation or broader reference parity follows.

A lossless native PNG plus recorded client origin/DPI can establish the displayed
preview's two-record geometry, literal black chord neighborhoods, functional-label
regions, clipping and absence of stale painted content after a visibly different input.
Crop using measured native coordinates; an OS title bar is not part of logical client
origin. Use existing independent source-coordinate probes, not a second invocation of
our renderer as the only oracle. Missing-record/H/Y negative controls remain meaningful.

Native screenshot RGBA is not automatically byte-identical to an uploaded image:
fractional destination origin, sampling, compositor color handling and screenshot
encoding matter. JPEG is unsuitable for strict pixel equality. Record these transforms
or compare declared semantic regions/tolerances. Pixels cannot certify hidden source
retention, attachment data, history/close barriers, worker retirement or unknown-field
fidelity; pair them with exact saved bytes and state evidence. No new golden is approved.

Ignored evidence is under `target/sdf-native-analysis/`: original/export hashes, exact
four-case output, unchanged external molecular results and measurement log. The final
handoff records all inputs and before/after checks. No own Cargo/worker jobs remain;
there was no native input, runtime source editing, new dependency/install, Git or Actions.
