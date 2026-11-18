# SDF current-view command acceptance

This independent seven-case suite verifies the success-only SDF camera/capture contract.
It uses the shared registry and public prepare/complete/finish APIs. It is CPU/headless
command evidence, not native navigation, physical-monitor calibration, 2x DPI, GPU or full
Professional screen/flow parity. The original camera/arrangement policies are recorded in
[the bounded view plan](research/sdf-view-plan.md); no vendor zoom/pan defaults are inferred.

## Delivered contract exercised

`render.capture` accepts an optional closed `view` object containing logical width/height
and the expected current view epoch. Logical dimensions and physical output dimensions
remain distinct. Omission or null keeps legacy whole-page capture; null serializes as
omission. The nested schema is closed, bounds logical dimensions to 1..100000 and retains
the old required capture fields. Epoch mismatch refuses during preparation as `stale_view`.

SDF Actual, Fit and numeric zoom work without a prepared extent. Explicit relative zoom
uses its actual percentage; relative zoom from Fit and bounded Pan need the real prepared
source extent. Pan while in Fit retains its existing refusal. Existing close/picker and
Drawing/SMILES/source-completeness guards remain effective.

Only a successful opted-in capture can install the weak-source-qualified SDF PageSize.
Errors and early/late cancellation install nothing; a failed render also cannot overwrite
an existing valid extent. Installation requires the current complete source, camera epoch,
Open close state and matching close epoch. Accepted historical images remain historical
replies and do not install anything in another/current session. The cache grants no
Drawing, chemistry-edit, Save, file-lease or trusted native acknowledgement authority.
The earlier error-bootstrap idea was rejected; an image failure has no extent side effect.

## Independent expectations and seven cases

The unchanged original ordinary fixture is 740 bytes, SHA-256
`e5cbdc14329c59bad55148c5de37058de90f651af10966c30cf8b89b3b5efd39`.
Its actual source contains two mixed-version records, five atoms and three bonds with
VAL/HCOUNT absent. Original source coordinates and authored missing-H remain distinct
from derived display facts. At 848 by 651 logical/physical pixels, 100% and center
(100,144)pt, the declared logical anchors are (396,290), (432,290), (432,254),
(432,398), (396,362). Pinned Pillow verifies actual generated PNG framing/RGBA dimensions,
three source-direction chords, expected O/H/N/subscript ink regions and separated rows.
The captured actual label metric is 12px. These boxes verify expected ink placement;
they do not establish a full glyph-shape or whole-image golden comparison.

1. Actual enables a real 1x current capture, then Pan reaches the independent anchors.
   Complete source/content/stamp, model=None, revision/history/dirty and actual whole-SDF
   Save bytes remain unchanged.
2. Unprepared camera controls preserve authored state and no-op epochs; Pan and relative
   Fit cannot invent bounds. Close and pending native-picker guards still refuse changes.
3. Unreadable Fit and tiny physical output report `unsupported_scale`. Pre-cancel and
   cancellation after real completion/before finish refuse without an installed extent
   or state change. A later error preserves an already valid extent.
4. Camera A-to-B-to-A, metadata edit/Undo, identical-byte Open, close Request/Cancel,
   pending close and a foreign Engine with matching bytes/revision/view all return only
   the accepted historical image. No stale/foreign cache, camera or source is installed;
   close metadata is refreshed by existing finish semantics.
5. Six repetitions of the unchanged complete pair create 12 records and 4440 source
   bytes. Original packing gives a literal 1368pt height: 72 + 12 * (36 + 72), with A4
   minimum width. First, middle and final source rows are reached at 1x. Unchanged external
   RDKit MOL consumers inspect actual file spans [0,371) and [4110,4413), independently
   checking formula, original coordinates and H. Replacing only the final N symbol with
   neutral P is independently consumed as a real P molecule, but the closed preview
   correctly refuses record 12 even when the retained camera points at record 1.
6. Closed/invalid view values and stale epochs fail without state changes; legacy/null
   capture bytes remain identical after camera changes. Registry schema required keys,
   nested closure and bounds are checked explicitly.
7. Prepared extent cannot authorize carbon creation, page edits, lossy CDXML/MOL Save,
   release another Save's reservation, or bind a capture as native file completion.
   Metadata-only Apply invalidates the old source cache despite unchanged geometry.
   SMILES and opaque/noncanonical Drawing backing stay guarded and preserved.

## Observed test-first evidence and oracle correction

The first 4532-byte positive was frozen before execution at SHA-256
`7cabf828b6a396024e939c78dc0216e574cff33cc122c66a127c9a2018b03090`.
Baseline session 11305 exited 101 at the real registry `ui.view` Actual call:
`WrongContent { required: drawing, actual: sdf }`. Compilation took 20.50s and the test
0.00s. The 4795-byte first log has SHA-256
`1f00e237c411611646e6e4ff3e24da845b847f79c0b5c27b7b0f897e3215bbb6`.
Root separately observed Actual pass followed by the not-yet-implemented capture `view`
parameter refusal. The six suffix cases compiled against baseline with existing APIs
before any suffix execution. A pre-execution last-N mutation-offset typo was corrected
from raw original byte inspection, with an explicit N assertion; its freeze records it.

Root's first complete seven-case run passed six and exposed one new-oracle setup error:
this suite assumed OH's H was stacked above O. The unchanged `resolved_label` policy tries
clear horizontal placement before stacked candidates. For the literal vertical O bond,
horizontal-left H is valid. Existing independent pinned OpenType vectors give H ink at
x420.3135..426.8701, y249.7344..258.2656, not the invented above location.

The pinned Pillow/FreeType 4x+BASIC+BOX reference, with its unchanged 128 threshold,
independently has 23 black reference pixels in (420,249,428,260) and zero in the old
(428,237,437,247) box. The actual parent-produced 1x PNG (13044 bytes, SHA-256
`818dcfe40006c7b4b65611ada1410343f2a6b6cd13424652d283de90e2828b28`) has 22
black pixels in the corrected box and zero above; O/N/H/subscript counts are 20/24/20/5.
The correction changes only that one H box and adds the old above box as an empty negative
control. No other box, threshold, chord, source input, assertion or production code changed.
Original 30871-byte source/SHA-256
`452b329088bb94c562f04c5b2c377376cecd8f425cd7a1e09ccd308355e46f55`, exact delta,
font/actual-PNG proofs and old RED evidence remain in ignored receipts. This is not a
renderer-defect RED/GREEN or a new human-approved golden.

## Final scoped checks and preservation

Root supplied the real seven renderer/commands files in
`target/sdf-view-root-backend-port.json`; this worker verified every byte length/hash
before running. Received production is classified separately, not claimed unchanged.
The final test source is 30951 bytes, SHA-256
`3b86d6af1b05fc9962a7f8fd0a419a3caf6b6e990f3fec3bf9eeb2cb3d831779`.

With Rust 1.94.1, absolute per-worktree target, incremental=0, dev/test debug=0 and the
existing pinned Python interpreter:

```sh
cargo +1.94.1 test --locked -p bondline-commands --test sdf_view
cargo +1.94.1 clippy --locked -p bondline-commands --all-targets -- -D warnings
cargo +1.94.1 fmt --package bondline-commands -- --check
```

Independent receiving session 62216 exited 0: **7 passed,0 failed**, 3.02s compilation
and 0.83s tests. Commands all-target clippy session 73925 exited 0 (7.19s). Package format
and separate non-mutating new-file rustfmt checks exited 0. This is not an aggregate
workspace or all commands runtime claim; Root owns integration, the intentional old
SDF-Actual requirement upgrade and mechanical optional-field constructor migrations.

Of the 24 protected baseline inputs, 18 remain exact and six are explicit received
production files; the separately received renderer page file is also pinned by its port
receipt. No old tests, parser/writer/oracle, fixture, lockfile, font, approved golden,
dependency, native input, Actions or Git state was changed by this worker. Final
`target/sdf-view-command-tests/final-handoff.json` records source/log/input identities,
classified ports and terminal owned jobs. Native recovery/pan/Save/reopen, 2x DPI and
interactive Windows/Linux remain separate required evidence.
