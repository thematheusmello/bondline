# Read-only SDF occurrence inspector acceptance

Current receiving Root: full local gate975/0/1, five independent real-App cases and two
Root lifecycle cases pass. Bounded native Mac occurrence/preservation checks pass below;
successful native chemical preview and mutating field UI remain unverified/missing.

2026-11-15. Independent test owner in `bondline-sdf-inspector-acceptance`, created from
`5f8b076`. This increment is a read-only UI over existing `sdf.record.inspect`; a mutating
field editor, draft handling and Apply/Save workflow remain separate required work.
Root owns production, Git, integration and native execution. Actions remains disabled.

`crates/ui/src/sdf_inspector_tests.rs` is a child of the existing real-App view test module.
The approved two-line bridge only appends a declaration; the original 47960-byte parent
file is preserved exactly before any later mechanical App-field initialization. No public
test hook, replacement loader, invented result or alternate command implementation is used.

## Fixed inputs and UI contract

The first case uses 1160×800 logical units and native pixels-per-point2 through the actual
Fixture input. Window size, DPI and font/readability policies are not adjusted for success.
It opens the complete original777-byte mixed SDF, SHA-256
`165e6f5d5cff37dae6315a07843d84733aa42dbbc497b2920da1702bad9526a0`.
The unchanged fixture is `crates/io/tests/fixtures/sdf/mixed-v2000-v3000.sdf`.
Its first record is V3000 with six ordered fields; the second is V2000 with one field.

Existing Records disclosure reveals small command-backed inspection buttons. Inspecting
record1 must show `Record 1 fields` and these collapsed occurrences in source order:
EMPTY, DUP, DUP, MASS, DT0012, SPACES. The duplicate occurrence is identified by its
position, not a property-map name; DT digits keep their leading zeros.

Expanding `Field 3: DUP` exposes the exact header `> <DUP>`, metadata
`Value: 7 bytes, 1 line`, and a read-only selectable value labelled
`Record 1 field 3 value`, with literal `second`. Legitimate TextInput and MultilineTextInput
roles are accepted. Values are read from actual reachable InlineTextBox descendants,
not an assumed parent-node value or a cached unreachable tooltip.

The first case focuses that real value, selects/copies `second`, and sends a text event
which must not replace it. It checks whole Engine metadata/source/history equality,
complete StoredContent equality, exact in-memory/on-disk source bytes, no automatic
`sdf_record` disclosure and metadata-only Debug. The actual SDF preview must still paint a
non-font image texture after Records expansion and inspection, with its real job retired.
This is a real registered preview path, not a new image oracle or native screenshot.

## RED checkpoint

The first draft failed compilation because its child module omitted the existing
`eframe::egui` alias: owner session30645 exited101 with22 related import/inference errors.
Only that import was added. Removing it exactly reconstructs the6344-byte first draft;
no dependency was added. That setup failure is not missing-UI evidence.

The corrected6362-byte positive, SHA-256
`1245902bb658050d37faeb4effa84a35c18aa0bb00b2e5efba1c00b8ddcc33d4`,
then compiled and ran in owner session9401. It exited101 with one failed test:
`current actionable Inspect record 1 fields field/control missing`, after the actual
Records click. This is the intended baseline UI RED, without missing new adapter types.
Logs are retained separately as `target/sdf-inspector-missing-ui-red.log` (setup) and
`target/sdf-inspector-runtime-red.log` (runtime).

Root separately reports its first real implementation passes that exact positive in
session97294. This is producer evidence; this worker has not received/run the backend
at this checkpoint. Future mechanical `SdfFieldsWorkflow::default()` additions to real
App fixtures are setup ports, not permission to change test bodies or expectations.

## Additive declared case

A second case is appended after the exact6362-byte positive prefix. It declares:

- EMPTY has the exact raw header and0-byte/0-line metadata.
- First DUP contains `first`, separately from the first case's third occurrence `second`.
- SPACES contains three literal spaces and4 retained bytes including its LF.
- Record2 replaces the current page with ORDINARY=`second record`,14 bytes/one line;
  old DUP occurrences must not remain merged into the new page.
- Opening the unchanged690-byte fixture (whose second record has no fields) clears the
  old record2 value in the next actual frame. Automatic state stays private; complete
  new source and both original on-disk files remain exact, and the preview can complete.

This extended source is10077 bytes, SHA-256
`fe065e597d3b5fc4504f85b1749bcc933b2b7913b007dd8fb9a9cdaea74a85f4`.
It is declared before the worker's real-backend GREEN and has not been run at this
checkpoint; no redundant missing-button run is claimed as another behavioral RED.

Reproducible owner command, with the worktree's unique absolute target and pinned Python:

```sh
CARGO_TARGET_DIR=/absolute/path/bondline-sdf-inspector-acceptance/target \
CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
BONDLINE_PYTHON=/absolute/path/bondline/.cache/oracles/format-python/bin/python \
cargo test --locked -p bondline-ui --lib view_tests::sdf_inspector:: -- --nocapture
```

The next gate requires Root's real product/bridge port, the unchanged cases, then relevant
UI checks/clippy/rustfmt. These two small sources do not cover oversized-value omissions,
budget-driven pagination, historical-result races/close drain or actual native inspection.
Existing core/adapter and preview tests retain their separate evidence. No mutating field
UI, new golden, full M1/M0 completion or reference-product screen parity is claimed.
No worker Cargo job remains at this checkpoint; the full M0–M5 goal stays active.

## Real-backend receiving checkpoint and focused regressions

Root supplied the real registered-work bridge, `SdfFieldsWorkflow` and mechanical App
fixture defaults in ten explicit files, recorded by
`target/sdf-inspector-backend-port.json` (SHA-256
`f38a6c3dd198f8af113d52f6e2591b644c6d94a9a98515dcd62fcfad4a444a1d`).
All ten byte/hash entries were checked against this worktree. Removing the single
`sdf_fields` default from the parent Fixture and the approved child bridge reconstructs
its original47960-byte source exactly. No old assertion/input was changed by this worker.

The worker then ran the two unchanged cases against that actual backend:
`target/sdf-inspector-two-case-green.log` reports2 passed,0 failed. This supersedes only
the earlier checkpoint's unexecuted status; it is synthetic real-App execution, not a
native-window observation.

Two additional small tests expose separate concrete gaps before their production fixes:

- Owner29342 exits101 at the actual third-DUP value's `node.is_read_only()` assertion.
  The value already refuses text edits; the missing accessibility ReadOnly flag is a
  different contract. The test also requires Focus and no SetValue support. Existing
  selection/copy proof remains in the unchanged first case.
- Owner94692 exits101 after focusing that same real value and loading its actual
  `egui::TextEditState` by the context's focused widget Id. The undoer retains a state
  different from an empty string. The test requires an empty/non-flux undoer both while
  focused and after Close fields, avoiding persistent copies of disclosed read-only data.

Logs remain `target/sdf-inspector-readonly-red.log` and
`target/sdf-inspector-text-history-red.log`; neither failure is a missing API or a
fabricated loader. Root owns the minimal accessibility/state fixes and the later real
source port. No native bridge execution is inferred from these node/state assertions.

## Budget-driven pagination and explicit omission

A further original test source reuses only the unchanged690-byte fixture's first282 MOL
bytes. It appends37 explicitly framed named fields: each has322 lines of200 ASCII bytes
and one line of160 bytes. The independently counted logical vector serializes to65530
JSON bytes, while its LF source value occupies64883 bytes. Both individual disclosure
limits pass. Sixteen such vectors alone use1048480 bytes, leaving only96 bytes of the
1 MiB page budget for all occurrence/page metadata; the real page must contain fifteen.
This is not37 raw65530-byte values, which would instead exceed the separate JSON limit.

The actual frame case follows Next from Field1 to16 to31, Previous to16 then1, and checks
that the final page has no Next. A separate38th occurrence has328 lines of200 `z` bytes
(raw65928 bytes), so its value is explicitly omitted. The UI must display its byte/line
counts and omission message and must not invent an empty text input. The complete source,
automatic metadata/state and actual registered preview remain exact. First execution
owner6726 is GREEN unchanged:1 passed,0 failed, logged in
`target/sdf-inspector-paging-first.log`. This tests UI navigation of real returned offsets;
existing core page/JSON-boundary tests remain separate evidence.

The five-case source was then rustfmt-formatted without changing tokens/expectations.
Its original first6362 and first10077 bytes remain exact. Declared pre-format snapshots
and both intended regression logs remain retained. ReadOnly/undo-state GREEN is still
pending the actual Root fix at this checkpoint. No worker Cargo process is live here.

## Final worker verification

Root then supplied the actual final field module: 17,216 bytes, SHA-256
`e20fd23f5f91af73dd15951c12a6edfb561fc7edebbf59a7af5bf2007d69e2b4`,
recorded by `target/sdf-inspector-final-backend-port.json`. Its declared scope is the
ReadOnly accessibility flag, clearing/storing read-only TextEdit undo state while keeping
cursor selection, and retirement of cancelled finished jobs without an Engine. The last
item has Root's own focused regression; it is not an additional independent worker test.
No shipped egui/SDK source, parser, command schema, fixture, oracle or golden was changed
by this worker.

Worker session68700 ran the five frozen cases unchanged against that final module:
5 passed, 0 failed, 0 ignored. The same sequential pipeline then passed all-target UI
clippy with warnings denied and workspace rustfmt check. Logs are:

- `target/sdf-inspector-five-green.log`
- `target/sdf-inspector-ui-clippy.log`
- `target/sdf-inspector-fmt-check.log`

The accepted test file is 17,584 bytes, SHA-256
`46838485d690ff2c3ac8575976972bfe076a882be892b131bb3fdececd7fc86c`.
Its first 10,077-byte two-case prefix remains byte-for-byte identical, as does the first
6,362-byte positive. Rustfmt changed only wrapping in the appended paging/undo-state
cases; original pre-format declared snapshots and intended runtime failures remain
retained. The first case's selection/copy/non-editability assertions still pass after
both accessibility/state fixes.

The final ignored handoff records both owned files, log identities, explicit received
backend exceptions, protected input checks and parent Fixture reconstruction. Root's
reported additional UI/thread-retirement suites are separate producer evidence; the
worker did not rerun the full workspace or operate a native app. Reference-product field
browser layout, mutating field editing, unsaved drafts and Apply/Save/close policy remain
required follow-on work. This slice verifies read-only real-App occurrence inspection,
privacy, navigation and preservation only; it does not complete M1 or the product goal.
All worker Cargo processes are terminal.

## Root integration and review corrections

Root49212 reproduces the intended missing Inspect-button runtime RED on the corrected
first6362-byte test, before product work. The first real implementation97294 passes that
test unchanged; receiving96375 then passes both initial cases. Root separately reproduces
the missing ReadOnly flag at `selected_field_accessibility` before setting it. The real
65554 run has9 passed/2 failed: focused text undo retention and finished-job deactivation.
After public TextEdit state cleanup and engine-free cancelled retirement,67209 passes
all11 selected SDF cases, including the five independent cases, both Root lifecycle cases
and four existing regressions. No assertion, source fixture or limit was relaxed.

The private Root gate holds a published result behind an unfinished thread: fields must
remain busy/uninstalled, then a source replacement cancels and drains without installing
old data. A separate finished-job case calls deactivate without any Engine guard. These
are controlled private lifecycle tests, not claims of observing a native scheduling race.

Setup-only corrections are retained separately: the first child lacked its `eframe::egui`
import; Root's first port check expected a different module name; wrong filter attempts
selected zero tests; Root's private helper path and Drop-type struct-update syntax were
corrected before their actual runtime checks. They are not behavior REDs. The final
17584-byte source equals the standard rustfmt transformation of its declared17020-byte
snapshot; the first10077-byte prefix remains exact. Six existing App fixtures gained only
one default field initializer; their original assertions are unchanged.

Independent review found the full-value TextEdit undo retention and bounded poisoned-
Engine finished-job retention; the final source recheck closes both and the known
ReadOnly flag gap. Follow-up SHA-256:
`f12ddc18e13118a390db0be3f819ce1cd73037f576ce169bcb1dff5e64d8c6c4`.
It is source review, not another runtime pass. Ordinary GUI caches/allocator overhead are
not a hard RSS promise, and clearing undo storage is not a secure-memory-erasure claim.

Root68521 `CARGO_INCREMENTAL=0 cargo xtask ci` exits0: **975 passed,0 failed,1 ignored**,
147 result lines including3 macOS AccessKit bridge tests. Rustfmt, all-target warnings-
denied clippy, layers, runtime/asset licenses and the pinned corpus all pass. Log
`target/sdf-inspector/root-ci.log`,116983B/SHA-256
`1b10b66302d5bb8bf4cfb3c77369f4590fd7d41461fa6d3e3f47043490efa444`.
Runtime declarations remain154/165/220 and2 assets. No dependency/lock/SDK/font/original
fixture/oracle/approved golden changed. The registry remains28 commands. Actions is off.

## Bounded actual macOS field workflow

Root55229 builds the actual native executable21,387,016B/SHA-256
`8410ed26523e3795d98744ffc4664cf78e7217f44c1ed9ffba15d63cf824adc9`.
Its new isolated ignored bundle is`target/sdf-inspector-native/Bondline.app`, PID26649.
Build receipts bind the exact UI source`c0b46295352508bb74ee35293216dd3d179ad69bf0cfe9930511ec3580f6a7dc`
and final field module`e20fd23f5f91af73dd15951c12a6edfb561fc7edebbf59a7af5bf2007d69e2b4`.
The user requested the desktop preview; no proprietary app or asset was inspected.

Actual chooser Open of the original777-byte input reveals both Records inspection
buttons. Record1 shows the ordered six occurrences; third-DUP is`second`/7 bytes/1 line
and second-DUP is separately`first`/6 bytes/1 line. Native select-all plus typing into
the third-DUP leaves its value unchanged. EMPTY reports0 bytes/0 lines; SPACES reports
4 bytes/1 line and retains its space-only value. Selecting record2 replaces the page
with ORDINARY=`second record`/14 bytes/1 line and removes duplicate rows.

Unicode Save As writes`Bondline fields verified λ.sdf`: exact777 bytes/SHA-256
`165e6f5d5cff37dae6315a07843d84733aa42dbbc497b2920da1702bad9526a0`.
Both unchanged stdin MOL consumers pass actual saved spans `[0,344)`/`[462,744)`:
CC/C2H6/H[3,3] for each. Full bytes prove ordered fields/duplicates/DT preservation;
RDKit's property map does not. Native same-source reopen clears the explicit panel.
The saved952-byte drawing is then opened in this newest executable and keyboard numeric
entry sets150%; final read-only inspection confirms revision3, Undo0/Redo0, clean, view
epoch1. The latest drawing workspace is left visible; older previews are untouched.

The generic native tool tree still labels text entries “settable”; that annotation is
not permission proof. Actual-App nodes prove ReadOnly/no SetValue, and native typing
proves the observed value did not change. Native copy, SetValue permission, pagination,
large-value omission, cancellation/decode/close races and effective DPI were not tested.
An initial tool SetValue attempt on Zoom made no observed change; keyboard entry worked.
No cause or passing native accessibility setter is inferred from that attempt.

Default native chemical preview still refuses its existing em guard. No successful native
chemical image, enlarged view, new screenshot asset/golden or human approval is claimed.
The computer-use trace retains actual observations; ignored native receipt7112B/SHA-256
`9e4efcbeca8f94e4aae344d66bf24164a755a1233a88f93f72b29b6f87df199a`
binds saved bytes, external consumers and final state without the private session token.
The complete M0–M5 goal remains active.
