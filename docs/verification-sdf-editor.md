# Actual-App SDF occurrence editor acceptance

2026-11-15. Independent test-only work in `bondline-sdf-edit-ui`, declared base
`778ea47`. Root owns product, Git and native execution. Actions stays disabled.
This is an original offline editor over `sdf.field.set_value`; public references do
not establish that exact proprietary desktop field-editing screen. See
[the qualified workspace plan](research/sdf-workspace-plan.md).

The first test uses the existing real-App `view_tests::Fixture` at 1160×800 logical
units and DPI2. Its only bridge appends a child-module declaration to the exact
48,088-byte parent source; no old input/assertion, fixture, consumer, font or golden
is changed. The test calls only current Fixture/App/SDK APIs, so its intended baseline
failure must be a missing Edit control rather than a missing future Rust API.

It opens the unchanged777-byte mixed SDF (SHA-256
`165e6f5d5cff37dae6315a07843d84733aa42dbbc497b2920da1702bad9526a0`),
expands Records → Inspect record1 fields → Field3:DUP, and verifies literal `second`.
The explicit Edit record1 field3 button must start an editable draft labelled
`Record 1 field 3 draft`. The original selectable field value remains read-only.
Typing `reviewed\nline two` changes only transient UI state until Apply field value.

After Apply, the complete source must be exactly
`source[..378] + b"reviewed\nline two\n" + source[385..]`:788 bytes, SHA-256
`e63cb9adf5cbc62b5b0be94992246c0d10b94cda405e49b9514993e7d9f9c361`.
This expectation is a predeclared literal byte splice, not a Bondline writer round trip.
First DUP, all other fields/headers/DT digits/padding/MOL data and the complete other
record remain exact. One Apply creates one history action and dirty state; it does not
silently save. Source metadata still describes the original777-byte backing.

Save uses Cmd+S through the actual FileWorkflow, then drains the existing job/reply
barrier. The actual Unicode-named file must contain the literal788 bytes. Unchanged
external pinned V3000/MOL consumers check each contained molecule; the raw splice does
the metadata-retention job that a supplier property map cannot do. Actual UI Undo
restores777 bytes/dirty/original-unmodified; Redo restores788 bytes/clean/source-modified
against the saved baseline. Reopening that actual file keeps all788 bytes.

The declared first source is7,908 bytes, SHA-256
`8c65b6f8f590ec60761aa0d82a51bec63a6a2a24367c32320ed6688e38053166`.
Baseline runtime execution is pending at this checkpoint. No editor success, native
execution, draft navigation or same-packet Apply/Save result is claimed. Those cases
are additive after interface agreement; no fabricated API/loader or direct setter
substitutes for the editor interaction.

## First runtime RED

Owner session93439 successfully compiled the actual existing app in this worktree's unique
absolute target, then exited101 with one failed test: `current actionable Edit record 1
field 3 field/control missing`. It reached the real Records/Inspect/third-DUP flow first.
This is the intended runtime UI RED, not a missing future API, compiler setup failure or
headless substitute. The unchanged first test remains7,908 bytes at the hash above.
The48,137-byte parent contains only the appended49-byte bridge after its exact48,088-byte
prefix (parent SHA-256 `0eb47419341d442adacc9d930e2931bb6e32db1a104459be1a3845235a8f82d8`).
The log is `target/sdf-editor-first-runtime-red.log`.

Before any editor product port,280 existing relevant source/test/fixture/oracle/manifest
inputs were hashed in `target/sdf-editor-protected-baseline.json`. The expected788-byte
literal output was independently checked with stdlib SHA-256, archived under target,
and never written by a Bondline encoder to generate this expectation. All worker jobs
are terminal at this checkpoint. Root may now implement the real editor; first GREEN
and native evidence remain pending. Additive navigation/Apply-Save tests await the actual
presentation contract, and cannot change this frozen first positive.

## Additive declared transaction cases

Root confirmed the original presentation contract before these tests were written:
changed-draft record navigation opens a Window titled `Unapplied field value`, with
`Apply`, `Discard`, `Cancel`. Cancel retains the same record/draft/source; Discard
navigates without committing; Apply makes one history commit before navigating.
The main editor retains `Apply field value` and `Cancel field edit` labels. A separate
editor Window is permitted; the original immutable inspector remains unchanged.

Two cases are appended after the byte-exact first7,908-byte positive. The first uses
fresh actual Apps to test each navigation decision independently, retaining the same
literal second-DUP input and source preservation. The second presses the actual Apply
button, then releases it and sends Cmd+S in the same RawInput packet. It requires the
actual FileWorkflow to publish the applied788-byte whole source, with one history edit,
clean saved baseline and real workers retired. It cannot pass by silently saving the
prior777-byte snapshot. An Apply error must not be turned into a successful old Save;
error/close/external-source races retain their separate required follow-on coverage.

The declared three-case file is14,341 bytes, SHA-256
`dc4a769a578d3f50e002ed78759823b2f1a51760ad456c0abd4681e3774286d8`.
It uses only existing Fixture/App/SDK/registry access and adds no product API or stub.
These two appended cases are unexecuted before the real backend port; the first intended
missing-control RED above remains the implementation prerequisite. No extra redundant
missing-button failure is advertised as new behavior evidence.

## First real backend and broader bounded cases

Root ported four actual product files, recorded in
`target/sdf-editor-first-backend-port.json`; each received hash was checked before
execution. No stub or future API was used. Worker session6880 ran the original three
cases unchanged:3 passed,0 failed. The first7,908-byte positive and complete14,341-byte
three-case prefix remain exact.

Four bounded cases were appended after that prefix:

- Remote Open of the original690-byte source and a distinct Open of identical777 bytes
  both detach the old changed draft. Its literal text stays visible/copyable, the actual
  current Apply button is disabled, and explicit navigation Discard leaves the replacement
  source/state and both files exact. Identical bytes still have a different CaptureStamp.
- Actual-App Cmd+W and a remote `document.close` request on a clean Engine with an
  unapplied UI draft cannot emit a final close/acknowledge committed. Cancel retains it;
  Discard then permits shared close without a source edit. A separate Apply route creates
  one dirty revision, obtains a fresh shared close decision, and saves788 bytes through
  the real FileWorkflow before committed close. These are synthetic App/remote routes,
  not native OS input or a window-close observation.
- An independently CRLF-framed823-byte input gets same-value Apply. Full state/history,
  CaptureStamp, complete StoredContent and source/index pointer sharing stay exact;
  original CRLF bytes remain in memory and on disk.
- A201-byte line is held as a local draft but refused by the shared command on Apply.
  Apply release followed by Save in one packet keeps the draft, exact prior state/source
  and an explicit error. A fixed old filesystem modification time also remains exact,
  detecting even byte-identical publication of the wrong old source.

Root source review then identified a precise Apply→Previous navigation risk. A final
case uses the unchanged first282 MOL bytes and65 original15-byte data fields. Input is
1262 bytes; the final `old65\n` value at1250..1256 becomes `updated65\n`, giving1266
bytes. The test performs real Next→Field65→Edit→Previous→Apply. It first requires the
literal candidate, one commit/dirty state and untouched disk source, then requires the
returned first page after real query retirement. The existing unchanged raw SDF/RDKit
consumer independently accepts both predeclared vectors with65 ordered fields and CC;
that literal self-check is separate from actual-App execution.

The eight-case source is31,003 bytes, SHA-256
`c41cea900464d5e3cfdef7a6b5baee457bef679d43b13320c44c4112cd4f3d12`.
Worker session16274 exits101:7 passed,1 failed exactly at the final Previous-page
assertion, after all preceding commit/source assertions pass. It is the intended
runtime regression, not an invalid test setup. Log:
`target/sdf-editor-eight-first.log`. No expectation/input/threshold has been changed to
hide the failure. Root owns its narrow production correction. Final all-eight GREEN,
UI clippy/rustfmt and native editor verification remain pending at this checkpoint.

## Final scoped worker verification

Root supplied the real Previous-refresh correction and separate committed-before-cancel
notice handling, plus its two private lifecycle gates. Final received product identities
are recorded in `target/sdf-editor-final-backend-port.json`; none was authored or edited
by this worker. The approved buffer delta corrects only a Root test fixture range.

Worker session16725 then ran all eight frozen acceptance cases unchanged:8 passed,
0 failed,0 ignored. This includes the exact Previous regression after its narrow fix.
The following clippy step stopped on a new Root-owned lifecycle test's duplicate shared
helper module declaration, so formatting was not reached in that pipeline. That lint
failure is retained separately; it is not another editor behavior RED.

Root supplied only the narrow cfg(test) helper annotation, with no production/body change.
Its final lifecycle test module is10,788 bytes, SHA-256
`86bc383a9a4b68dff38bf0c141500969ac88122e0f70e5f261156ea7fef7e611`.
Worker session69482 then passed all-target UI warnings-denied clippy and workspace fmt
check. The eight runtime tests were not redundantly repeated for an annotation-only edit.

Final logs:

- `target/sdf-editor-final-eight-green.log` — unchanged8-case runtime GREEN.
- `target/sdf-editor-ui-clippy-first-failure.log` — retained shared-helper declaration lint.
- `target/sdf-editor-ui-clippy-final.log` — warnings-denied UI clippy GREEN.
- `target/sdf-editor-fmt-check.log` — workspace formatting GREEN.

The final owned test remains31,003 bytes at SHA-256
`c41cea900464d5e3cfdef7a6b5baee457bef679d43b13320c44c4112cd4f3d12`.
Its original7,908-byte positive and14,341-byte three-case prefixes remain byte-for-byte
identical. The original48,088-byte parent test source also remains exact before its
approved49-byte child bridge. The final ignored handoff records the two owned files,
initial/final Root backend replacements, protected baseline comparison, source/consumer
snapshots and every actual RED/GREEN log. No old fixture, oracle, font, golden, dependency
or runtime source was changed by this worker.

Root's full-workspace/private lifecycle gates remain separate integration evidence.
This worker ran no native UI or Actions. Actual native field-editing/close execution,
Windows/Linux execution, broad collection operations and complete M1 remain unverified
here. The full product goal stays active. All worker processes are terminal.

## Additive external Requested-Save acceptance

One final loop case was appended after the exact31,003-byte eight-case prefix. It uses
only real `ui.file` and actual App/FileWorkflow operations, never private editor hooks.
With the changed local draft open, `ui.file` Save at revision1 creates request1/epoch1.
Before any decision, the request must remain Requested: no FileWorkflow worker,
publication, authored change or old-file mtime change is permitted.

The three independent decisions have different authority outcomes:

- Cancel retains the draft/777-byte source, consumes the actual file.cancel route and
  reaches request:null/epoch2. That epoch advance distinguishes cancellation from simply
  fabricating trusted completion of request1. No file publication occurs.
- Discard preserves the original current request1/epoch1, observes its actual Running
  FileWorkflow job and saves the authored777-byte source without a history edit.
- Apply commits once, invalidates the old request, observes fresh request2/epoch2 at
  revision2 Running, and saves the literal788-byte source. Saved state is clean with
  one history entry and original-source modified metadata intact.

The test inspects an already Running ticket before advancing another frame, because
polling could otherwise retire the tiny file job before the harness observes it. This
was settled before execution, not by relaxing a failed assertion. The first actual run
owner74194 is GREEN unchanged; no new production fix or artificial behavioral RED is
claimed for this already-delivered adapter behavior.

The final nine-case file is36,674 bytes, SHA-256
`72ac66b03bf74cd91e4c24ad7304ace75898a39f9eaac94d4b672d9963eb9843`.
Worker session51922 then passed9 cases,0 failures,0 ignored; UI all-target warnings-denied
clippy and workspace formatting also passed. Logs:
`target/sdf-editor-external-save-first.log`, `target/sdf-editor-final-nine-green.log`,
`target/sdf-editor-nine-ui-clippy.log`, `target/sdf-editor-nine-fmt-check.log`.

The earlier eight-case handoff remains unchanged at
`target/sdf-editor-handoff.json` (SHA-256
`a4ad84c75056e1c170c0121cb4f762f1910bc99a921bafb6defa460023776080`).
An additive v2 handoff records the new source/doc suffixes and logs, rechecks the same
protected inputs/backend and preserves all earlier prefixes/evidence. No runtime,
fixture, oracle, dependency or native behavior was changed here. Native field editing
and full product parity remain separate unverified work; all worker jobs are terminal.
