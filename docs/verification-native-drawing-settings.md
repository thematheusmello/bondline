# Native Document Settings receiving record

2026-11-17, branch `feat/multiple-bonds`, base `b6e1b18`. The accompanying atomic local
commit contains the implementation, tests and this record. Full M0–M5 stays active;
Actions stays OFF. No push, publication or proprietary runtime inspection occurs.

## Delivered scope

File > Document Settings… opens an original Drawing section with two stored optional
creation preferences, Bond length (pt) and Chain angle (degrees). Blank clears that
explicit root property. Missing values stay absent; existing geometry stays unchanged.
Finite length >0 through100000pt and angle1..179° use the existing model validator.
This is not complete settings/style parity or application of those values to tools.

One appended `ui.drawing_settings` has closed Open/Cancel parameter objects. The host's
Apply revalidates source/opaque association/File/close/view/effective-tool identity and
calls `document.set_drawing_settings` under the same Engine guard. Stale drafts remain
copyable with Apply disabled. Only exact explicit Cancel marks its fresh, serde-skipped
cancellation token, so remote Cancel dismisses its form without confusing detachment or
source/context loss with cancellation. No FilePlan, worker, Save receipt or close ACK is
created. Both fields reuse the384-byte TextBuffer and the mechanically extracted Atom
input-prefix router. The old Atom limits remain2/5 bytes and its17 cases remain unchanged.

The actual CLI has42 entries. Its first41 records, descriptions, schemas and order equal
the prior captured CLI output exactly; only the new final entry is appended. The new
`file.document_settings` control brings native/retained-SMILES/SDF/raw-SMILES counts to
32/21/15/11. All124 catalogue roles/140 entries stay12 delivered/128 disabled.

## Test first and qualifications

Root owns execution, tracked edits and integration. Independent workers supply engine,
actual-App, wire and inventory proposals under ignored `target/`; they run no builds or
tests. Their later expanded declarations are post-producer acceptance, not pre-producer
RED. Logs below are in `target/native-drawing-settings/`, disposable evidence rather than
required tracked inputs.

| Execution | Observation |
|---|---|
|45343 →89124| Existing JSON interface fails `UnknownCommand("ui.drawing_settings")`0/1 before producer; first integration then passes1/0. |
|59518 →42245| Actual-App File menu case fails missing actionable Document Settings0/1; qualified producer passes1/0. Intervening10381 is E0583 from a copied test-module declaration, removed without changing assertions; it runs no test. |
|42738 →63410| All17 old Atom cases pass before/after the input-router extraction. |
|15041| Wrong outer `--test mcp` target selects zero tests; setup only. |
|28546 →20906| Correct actual authenticated-loopback target receives Open then times out at the unchanged10s repaint wait. Adding one successful-operation notification condition gives3/0 across actual CLI, initialized stdio and authenticated socket. A redundant import warning is removed without assertion changes. |
|96168 →5872| Expanded App first3/2: one unsuitable fixed-scene consumer, one real remote-Cancel retained-window defect. Source qualification replaces only the new helper's wrong fixed-coordinate consumer with the unchanged general named-ring reader, retaining literal root/graph/H/coordinates/CC/C2H6 assertions. Exact per-request cancellation fixes the product defect. Qualified App5/0. |
|43334 →98252| Private checked-ID/cancellation test1/0; the common filter accidentally selects zero integration cases. Correct no-filter integration run passes initial1+independent6=7/0. |
|14206 →30846 →84512| First full CI passes fmt/workspace lint, then fails167/1 at a new metadata expectation: a headless foreign-source control correctly reports `ui_unavailable`, not `unsupported_content`. Add the host-unavailable premise and qualify only that added expectation; exact affected case1/0. A justified fresh full gate then passes1629/0/1. No old source, geometry, history, timing or socket assertion is weakened. |

Sixteen new cases are unique: initial JSON1, independent engine6, private counter/token1,
actual-App5 and actual-adapter3. They cover strict schemas and host/source precedence,
checked IDs, context/association ABA, normal/no-op/history/Save baselines,384/385-byte and
nonfinite inputs, action-prefix/suffix/modal/close fences, remote cancellation, copyable
stale drafts, foreign content, synthetic2× layout and independent Unicode file consumers.
Repeated old Atom tests and zero-selected groups are not additional passing cases.

The final source reviewer finds no material issue across45 inputs and14 completed logs,
including exact Atom extraction and the one-condition MCP delta. Its sealing predates
the metadata-expectation correction, final gate and native execution, and supplies no
evidence for them. Existing socket policy, workers, cancellation/publication ordering,
protected listener test, corpus, fonts, dependencies and human-approved golden are unchanged.
A later nine-input delta review finds no material issue in the new headless-premise
correction, terminal aggregate logs, current documentation and saved native-file receipt.
It does not independently establish CUA pixels or process observations. Its sealed
`final-source-review/DELTA-REVIEW.md` is3290B/SHA256
`d529cd754cf8c645dcfbb2a2479085cfbc616afef3c2c16932f6033b0d0549d1`.

## Full local checks and build

`cargo xtask ci`84512 exits0: **1629 passed,0 failed,1 ignored across242 result lines**.
It includes rustfmt, warnings-denied workspace clippy, workspace tests, existing vendor
tests/lint, layer checks, seven bundled assets, strict runtime license closures155/166/221
and the pinned8-file/5-bundle corpus. The existing offscreen application-frame artifact
case remains ignored. Corpus output still identifies missing general CDX/application and
chemistry coverage. Historical intermittent MCP listener replacement-bind failures remain
unattributed; this pass is not a causal repair or an exhausted-probe retry.

| Evidence | Bytes | SHA-256 |
|---|---:|---|
|First failed `ci.log`|22539|`5718e2e8463d0280ac44843a4bd2bd9972e1b0b95392aa41e57e30ed4e77156f`|
|Qualified passing `ci-qualified.log`|203657|`c5e046fd7ee237e3065a9c379f615cfdd480d185e85cbc57d25e57e476bf6c82`|
|Prior41-command CLI JSON|34619|`fd857d3f9643a6cb8f7957ee89419f2655ea56acb5e7f1c8d82d2992aff8879a`|
|Current `commands.json`|35478|`a5d7baaea88cdeb9340871eaad1929b8b27094a540f80bae6a29e4a932330334`|

Native/CLI `cargo build -p bondline-ui -p bondline-cli --bins --locked`10841 exits0 after
the gate. Actual native binary23927872B SHA256
`3b40734bb7f2465d86a7318d47908bfecd17c1240f8783d1b06d547fe4be2a72`;
CLI21691632B SHA256 `656689fed9f12d423a9d5f7684b17e38dbd76c7aac4db6f7dd3eea0645dbd9e4`.
No source change follows that build; remaining changes are documentation.
Final doc-only checks pass rustfmt, `git diff --check`, changed-text whitespace and all
local Markdown targets in the14 changed Markdown files; the full gate is not repeated
for prose-only changes.

## Actual native macOS workflow

The separate disposable `org.bondline.settings-review` bundle contains the exact native
binary above, with no control listener. On macOS27.2/AppleM3Max, CUA observes the actual
executable, not a mockup or source-only input test. From an empty document, at Actual100%:

1. Drag one80pt C–C bond; the actual inspector shows2 atoms/1 bond/1 fragment and CH3—CH3.
   Save As `before-原.cdxml` in the isolated review directory; header becomes clean.
2. Open File > Document Settings…. Both values start blank. Set length64.25 and angle137.5;
   inspect the rendered form for visible fields/buttons, Apply and observe dirty state.
3. Native Cmd+Z restores clean, Cmd+Shift+Z restores dirty. Reopen the form and observe
   both exact values. Clear only angle, Apply, then Save As `settings-保存-設定.cdxml`.
4. Native Open reopens that actual file clean, retaining CH3—CH3 and2/1/1. Reopen settings:
   length64.25, angle blank. Apply unchanged; form closes and document remains clean.
5. Close this disposable app through File > Close Window. CUA listApps confirms it gone;
   a subsequent narrow process check finds no review process. The two protected user
   bundles remain running with unchanged executable hashes; their drawings are not inspected.

The initial AX View > Actual attempt does not change rendered78.79%; subsequent CUA app
activation and native F5 reach100%. No causal focus/first-click fix is claimed. AX diffs
can lag the rendered form. An AX click also fails to focus the angle field for keyboard
clearing; a documented coordinate click followed by Cmd+A/Backspace succeeds. Native
screenshots qualify the observed state. These automation qualifications are not product
fixes. No new screenshot, chemical golden or golden approval is committed.

Exact Unicode filenames reach the filesystem and AX names, but the bundled Roboto face
shows missing glyphs for the reviewed CJK characters in the title. Correct persistence
does not establish complete Unicode UI/IME support. Native2×/DPI, hardware-mouse operation,
other platforms, full settings/default/style parity and native chains remain unverified.

## Independent saved-file evidence

Pinned external Python/RDKit2026.09.2 and independent ElementTree/Expat/Decimal readers
consume the actual native files. Before root is only `CreationProgram="Bondline 0.1.0"`;
after root adds only `BondLength="64.25"`, with no ChainAngle. Ordered non-root XML content
is equal; positions remain(260,327) and(340,327)pt,80pt apart. Each atom remains C/H3/
charge0; RDKit returns2 atoms/1 bond, `CC`, `C2H6`. Thus the saved64.25 preference does
not rescale the existing80pt geometry.

| Actual artifact, isolated native-review directory | Bytes | SHA-256 |
|---|---:|---|
|`before-原.cdxml`|436|`b4e8d180b5eae0bbca68e794744f245127bcc8befddf7fbf2d12a5e0cce0bae2`|
|`settings-保存-設定.cdxml`|455|`bc9e5e5a65740b971d54650e1aac9e7d997a07e75192e183a228c05d72e69d01`|
|`independent-native-files.json`|690|`bcf69e012c0b370620d250cc9d55d748cdf44ca170fc11104b30479dd553b794`|

The independent root oracle remains `crates/io/tests/drawing_settings_root.py`; actual
App fixtures use the unchanged general `crates/commands/tests/ring_create/named_ring_oracle.py`
with literal expected graph/H/positions, separate from the fixed-scene oracle's setup limits.

## Reproduction and continuation

Use the pinned development environment described in [development](development.md), with
the existing external format-oracle venv, Rust1.94.1 and exact lockfile. Root used
`CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`,
`PROPTEST_RNG_SEED=671308429` and `BONDLINE_PYTHON` pointing to the existing
`.cache/oracles/format-python/bin/python` without resolving its venv symlink.

```bash
cargo test -p bondline-commands --test drawing_settings_ui --test ui_drawing_settings --locked
cargo test -p bondline-commands --lib request_ids_are_checked_and_only_exact_explicit_cancel_marks_its_token --locked
cargo test -p bondline-ui --lib native_drawing_settings --locked
cargo test -p bondline-cli --test sdf_capture_adapters settings_ui_wire --locked
cargo xtask ci
cargo build -p bondline-ui -p bondline-cli --bins --locked
```

Read [the settings contract](research/native-drawing-settings.md),
[native-chain dependencies](research/native-chain-plan.md) and
[public source register](clean-room/sources.md). Next lock original complete chain
pose/count/snapping/backtrack/reversal/repeat policies before independent App tests, then
implement the full controller, Add Chain, repeat, Snaking and preference application
through the existing candidates/registry. Preserve literal H, full-ink/source/currentness/
modal/retirement guards and inspect actual native workflows before enabling grey tools.
Do not replace that work with another headless-only prerequisite or claim full M0–M5.
All owned test/build/review jobs are terminal; the disposable native review app is closed.
