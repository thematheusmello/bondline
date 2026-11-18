# Actual-frame view acceptance

Follow-up checkpoint: a native-derived Undo→Save event-order defect is now reproduced
by an additive eleventh actual-frame test. The original ten and their GREEN below
remain exact. The new completed-file regression is RED and awaits a production fix;
do not interpret the earlier scoped pass as complete native workflow verification.

On the final verified100-file source port, all10 frozen actual-frame regressions
and both direct SDK focus cases pass independently. No settling frame or behavior
assertion was added to obtain GREEN. Formatting and relevant all-target warnings-denied
clippy pass; native verification remains separate.

Independent headless tests began at `2ac7739`,2026-11-15, using the real private
`BondlineApp::update_workspace` and pinned egui frames in a separate target directory.
They create no native window and send no OS input. Native workflow execution remains
unverified by this test owner; Root owns any native validation.

## Contract and independent expectations

The guide evidence and original camera policies are separated in
[verification-view-commands.md](verification-view-commands.md). The test matrix uses
the original A4 page,1160×800 logical workspace,100% exact uniform mapping and integer
camera translation rounded to nearest with half ties away from zero. It calculates
expected pointer positions directly from those declared numbers, without calling the
new production camera helpers. DPI2 changes physical rasterization, not authored
coordinates.

Current accessible controls are found through reachable AccessKit nodes, actionable
semantic roles and Focus support. Zoom may receive its name through `labelled_by`;
cached tooltip Labels cannot substitute for the current editable field.

The ten tests in `crates/ui/src/view_tests.rs` require:

1. Central View bindings and F5 on untouched A4, visibly clipped Actual paper,
   actual Double drawing, F7 own-last-drawn focus, editable Zoom text whose150%
   value applies only on Enter, F5 suppression during numeric editing, and actual
   Triple drawing with five vector components. Unicode Save is checked by unchanged
   external Expat/Decimal and pinned RDKit consumers. Undo/Redo/Open must preserve
   camera state and exact graph/source relationships.
2. An actual native-sized frame with a current visible focusable Zoom field and
   no authored change from layout/automatic viewport metrics.
3. At DPI2, an actual held Double preview, ordered Fit→Actual A→B→A cancellation
   before the old release, and a later fresh press in the same packet using the
   final camera. A true no-op Actual action must preserve a valid armed gesture.
4. Reachable horizontal/vertical position Sliders, actual accessible SetValue and
   Point-wheel pan at200%, with an independent negative-delta/scale expectation.
   Outside, modified, stale-focus and text-edit input must not pan. Fresh pointer
   input rearms it; real pan cancels an armed gesture and zero wheel does not.
5. Reverse-axis and mixed F5/axis actions resolve the other coordinate at the actual
   input event, never from a stale layout snapshot. Same-axis A→B→A retains both
   epoch changes. Point, Line and Page wheel witnesses at175.5% use exact f64 scale
   and multiplication, with original factors1,24 and0.8×floored logical viewport.
6. Open a smaller native source while retaining the original stored out-of-page
   camera. Fresh wheel and isolated horizontal/vertical controls must resolve from
   the bounded displayed center, preserving source, CaptureStamp and exact Save.
7. Pointer movement down the actual vertical control increases authored y; its
   accessibility bounds/value remain truthful. Real focused horizontal Left/Right
   keys each reach the shared command engine without authoring a canvas bond.
8. Numeric Zoom accessibility A→B→A advances both epochs and retires an earlier
   drawing gesture. Single-value and genuine no-op positive controls are included.
9. From a real large-page camera failure, actual header Zoom input recovers a valid
   camera and canvas without an authored edit. Existing F6 recovery also remains.
10. An interior vertical pan node advertises Increment and Decrement honestly;
    both actual actions change camera y in opposite directions while retaining
    native source bytes, complete state/history and CaptureStamp identity.

The initial main flow was frozen at15,398 bytes, SHA-256
`9aae97a5fa3a4742b95df4118af1791a3758ba73027fe5668a01ea4b399cf8e8`.
The second case retained that exact prefix:16,151 bytes, SHA-256
`20b779208bcedd9371d08e3a1e4c6f4d7eb00b0d73817e5f666274a709d8fbc6`.
The third retained both prefixes:19,165 bytes, SHA-256
`19a798a4611193ad8ae8f955b4e86d79064bd4617d9549b2bcd2c1cfef999808`.
The fourth is additive:24,991 bytes, SHA-256
`f42618b533438dfa57c4700f83fa7ad7895b2d8709b7522522c273c642e77427`.
Through the fifth freeze, all prior prefixes remained exact. Direct rustfmt checking
of all four new test files passed without a formatting correction.
The fifth adds no changes to those bodies:29,264 bytes, SHA-256
`2054528235c37b4af066eb5dc0853055273bc77458c311d2d51879cb531ddd15`.
It was frozen after independent review identified a concrete reversed-axis snapshot
counterexample and f32 wheel-scale roundoff in the new pan draft. Root approved
these original event/precision expectations before the fifth runtime gate. The
ignored source observations are bound by `target/view-pan-draft-observation.json`.

Root's warnings-denied clippy run73583 found the helper's nested reachability `if`
could be a let chain. This was the only change inside the old five: reversing that
syntax-only change reconstructs all pre-lint source exactly, with no changed assertion,
event or expectation. Six before lint was32,754 bytes/SHA-256
`d4485997f728b44eee96e6388b318eb10b2b0ea221a56b4d561aa4a9fa492431`;
six after was32,742 bytes/SHA-256
`8826010c790d21e0e19d2e92981bf1d08a355ea202baba4dcd201eba7962051d`.
The proof is `target/view-ui-six-freeze.json`; older prefix claims require this
explicit lint reversal rather than claiming current raw bytes are unchanged.

Eight retained the corrected six exactly:38,028 bytes/SHA-256
`782ea80d04af9d2346aa6d2488213159d3e776a90e80bef431d948622efcf0b4`.
The ninth initially used `?` on the existing Option-returning `PageId::new` constructor.
Root65885 therefore failed compilation; it was not a behavior RED. The only correction
was `.ok_or("page id")?`, retaining all eight and every input/assertion exactly.
Current nine is40,001 bytes/SHA-256
`16fa1701b98f49e104d92f31670c75a219c98e0230423d7a495c3d637f14f9e3`.
`target/view-ui-nine-page-id-fix.json` reconstructs the pre-correction39,984-byte
source exactly. All later success/failure observations must use this qualified source.
The tenth retained the corrected40,001-byte nine prefix exactly. Final source is
43,346 bytes/SHA-256
`34644dea8fdd873e49bab6feebb53a2f1a9a6e1d7b8d289ef727d906538266d2`.

Wheel panning and its safety guards are original Bondline policy. Pinned
[egui0.29.1 input.rs](https://github.com/emilk/egui/blob/0.29.1/crates/egui/src/data/input.rs),
lines500–519, defines positive wheel delta as content movement right/down;1147–1158
defines Point as logical UI units. Thus delta(20,-40) at200% changes camera center
from(240,300) to(230,320). Pinned egui-winit lines684–704 converts physical wheel
pixels by pixels-per-point. The test uses the actual Slider numeric SetValue path
implemented at slider.rs741–750. This is SDK evidence, not vendor navigation parity.
The exact three local SDK source hashes are in `target/view-wheel-sdk-inputs.json`.

One pre-freeze source-review correction moved Undo/Redo before Open, respecting
existing Open history reset. It preceded all official main-flow freezes and runtime
observations. The discarded drafting source remains ignored for audit; no failing
behavior assertion was weakened.

## RED observations and pending verification

Root first ran the exact initial main flow on its minimal real command backend:
session26362 exited101 because F5 left `zoom_percent:null`, where100 was required.
This was an actual frame/UI wiring failure, not a compile or missing-type failure.

This owner independently ran all three on unchanged old production plus Root's sole
test-module bridge:

```sh
cargo test -p bondline-ui --lib view_tests --locked
```

Session86509 exited101:0/3. The central Actual binding was absent; the actual current
Zoom field was absent; and the ordered DPI2 flow returned UnknownCommand(`ui.view`).
The log is `target/view-ui-three-red.log`. The Root-owned bridged UI source was
SHA-256 `b9f0dc135cc9b9c8f48d091458c31a78e9c790e4f2c83b8b8798c66cf32cd57a`.
The additive fourth targeted baseline92773 also exited101,0/1 with the intended
UnknownCommand; its log is `target/view-ui-fourth-red.log`.

No new goldens, mockup images, reference assets or dependencies are introduced.
Actual file consumers were not reached on the old-source RED.

After verifying Root's11 real partial-port inputs and all726 other original files,
session55184 ran the unchanged four-test source:3/4 passed. This independently
reached and passed original A4 Actual/150% drawing, all five Double/Triple components,
actual external saved-file consumers, Zoom accessibility, DPI2 event ordering,
view-epoch cancellation and no-op preservation. The fourth failed exactly at the
missing actionable Horizontal position Slider. This is partial RED, not a finished
pan workflow. See `target/view-ui-four-partial-port.log` and the source receipt linked
from [verification-view-commands.md](verification-view-commands.md).

At that partial checkpoint, final pan GREEN and final package checks remained
pending. Root performs the aggregate workspace gate and separate native validation.

Root's separate receiving tests additionally observed actual runtime RED for reversed
axis order93977 (Y300 versus400), then unchanged five GREEN29808; smaller-page wheel
53912 (X100 versus90), then unchanged six GREEN44704. On the exact eight it observed
89287: vertical drag reversed (top678, bottom163), and numeric Zoom A→B→A epoch1
versus3. Follow-up production corrections exposed horizontal Right and then Left
failures rather than relaxing the frozen directional assertions. These are receiving
records, not this owner's independent final GREEN. After the Option setup correction,
ninth receiving59590 reached the intended runtime failure (999% versus1%). Its
fallback-dispatch correction36759 reached8/9; horizontal Left focus remains the
single unresolved case at this checkpoint. No settling frame or behavioral assertion
was added to hide it. Root is investigating the pinned SDK focus behavior before
freezing the final implementation port.

The initially transposed printed-page numbers in one research mailbox message were
checked against the exact cached primary text and corrected by its owner:26 is the
appearance/range/General toolbar page;79 lists F5–F8;74 is the generic tool table.
The frozen research document and the citations here were already correct.

Root receiving logs have separate retained copies and hashes under
`target/root-view-receiving-log-retention.json`. A log filename containing “green”
is not treated as a pass: its actual result and source state determine the evidence.

## Independent final GREEN

Root's tenth receiving31538 reached the intended missing Increment capability RED;
its correction26524 passed10. Root's independent bare SDK probe had separately
reproduced the fresh-focus failure before a one-guard correction. The reviewed
[egui patch record](../vendor/egui/BONDLINE-PATCH.md) pins the public archive/VCS,
exact source delta and full notices. It allows the current focus owner to install its
orientation filter during its first focused pass. No version, feature, asset or license
allowlist changed. This is an audited local fix, not an upstream acceptance claim.

This owner then verified every input in the final receipt linked from
[verification-view-commands.md](verification-view-commands.md) and independently ran:

| Session | Command | Actual result | Ignored log |
|---|---|---|---|
|48988|`cargo test -p bondline-ui --lib view_tests --locked`|10/10; exit0|`target/view-final-ui.log`|
|60068|`cargo test -p bondline-ui --test slider_focus --locked`|2/2; exit0|`target/view-final-sdk.log`|

Workspace rustfmt and warnings-denied all-target clippy for commands/CLI/MCP/UI also
exited0; commands and logs are in the companion record. All jobs are terminal. The
final handoff binds every Root input, owned source, preserved baseline and log hash.

The direct SDK tests use real egui controls, the original Apache font, an immediate
arrow after fresh focus and a separate settled-focus control. Together with unchanged
actual application frames they distinguish the SDK cause from a product workaround.
All frozen event sequences, directions, epochs, source checks, external saved-file
consumers and precision tolerances pass. The documented helper lint, PageId setup and
separate adapter schema-oracle corrections are the complete test-correction record.

These are headless actual-frame checks. This owner did not launch a window, use OS
input, touch the user's preview process, run hosted Actions, install tools or perform
Git actions. Root owns aggregate CI, native interaction, integration and commits.

## Follow-up: ordered Undo and actual Save

Root's later native inspection found that a rapid primary-Z then primary-S batch
could show Saved while retaining and writing the edited Double. This owner did not
operate that native window. It appended one original test after the exact43,346-byte
ten prefix:47,960 bytes/SHA-256
`e7dfb9cbc2940c1b855546b20cb34d9b9de90e9aa5fa190f67560e679b911906`.

The test opens associated native source, clicks Single→Double through the actual App
at150%, then runs separated Undo/Save as a positive control before the same-packet
sequence. It waits at most ten seconds for actual FileWorkflow and executor retirement,
and checks the saved bytes through the unchanged external Expat/Decimal reader before
checking memory/history. It does not substitute direct Save dispatch for the UI action.

Independent session35353 exited101 at the intended behavior assertion: the separated
control passed; `batched=true` saved literal Double/H2 instead of Single/H3 and retained
revision3/undo1/redo0. The actual file worker had finished. No code or expectation was
changed to obtain this result. `target/view-undo-save-red-receipt.json` binds the source,
log and exact held100 production inputs. The prior final handoff remains historical;
the new regression's correction and subsequent native verification are pending.
