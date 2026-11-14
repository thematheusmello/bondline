# Multiple-bond tools: independent frame acceptance

2026-11-14. This record concerns original Single/Double/Triple tool roles and the
shared centered-bundle policy. It is not native-window execution, reference-product
pixel/layout parity, or approval of a new golden. All six frozen frame workflows
are now GREEN after the real renderer, preview and ordered-event integration.
The full UI package passes116tests; all-target UI clippy and formatting pass.
The first full-package RED caught a Single guidance regression; its production
correction passed the same unchanged assertion. No native input, window, desktop
inspection or hosted workflow ran.

## Fixed expectations and source

The six tests in `crates/ui/src/multiple_bond_tests.rs` run the actual
`BondlineApp::update_workspace` through egui frames. They discover actual buttons
through AccessKit names/bounds, click those bounds or supply egui's existing
AccessKit Default action, and inspect actual canvas polygons and stable hit IDs.
They do not call a new production-only test hook or recreate the renderer to
produce expected polygons. Normal state, authored model, history, source-backed
Save/reopen and explicit operation receipts are observed through the real Engine.

The research and its limitations are in
[the multiple-bond plan](research/multiple-bond-plan.md). [IUPAC GR1.6](https://iupac.qmul.ac.uk/drawing/drawing.html#GR16)
requires visibly associated parallel components; [GR1.10](https://iupac.qmul.ac.uk/drawing/drawing.html#GR110)
distinguishes centered and offset depictions. Neither establishes Bondline's
original spacing policy. The tests freeze `s=3w`: Double offsets `[-1.5w,+1.5w]`,
Triple `[-3w,0,+3w]`, with unchanged authored endpoints. Vector comparisons use
an independently declared `0.001` logical-unit tolerance. The original 128pt-square
test page makes every component separately testable outside the former axis-only
hit envelope. This does not assert a reference-product numeric default or imported
style interpretation.

Actual registered CPU PNGs are checked by an original Pillow strip/gap consumer,
independently of the UI vector paths. Saved native CDXML is passed to RDKit's
actual `MolsFromCDXML` consumer for bond orders and total H. These checks passed
in the first six-workflow GREEN. The pinned interpreter is selected with
`BONDLINE_PYTHON`; expected versions are Pillow12.3.0 and RDKit2026.09.2. No old
oracle, fixture, dependency, font or approved Single golden is edited.

## Acceptance cases

1. Central labels, accessible actual buttons, original one/two/three-line icons,
   selected state, monotonic mode epoch and same-mode no-op. Tool changes preserve
   ordinary state and authored history.
2. Actual Double/Triple drags create the selected orders and literal H; initial
   and refreshed canvas show every component; each component resolves to the
   same authored bond and receives its own outline. Save, independent CDXML
   consumption, Undo/Redo and reopen retain the graph and mode.
3. A previous-frame armed gesture is cancelled by activation; a later fresh
   press in that same frame still uses the newly selected order. A mode
   A→B→A round trip cannot revive the old gesture or cached hover.
4. A vertical Single followed by an actual horizontal Double/Triple extension
   uses the expected authored H budget, refreshes the whole bundle, and survives
   Save and Undo/Redo.
5. A held Double/Triple drag has two/three real preview polygons before release,
   preserves all authored state, and clears on Escape without a later commit.
6. Within one raw-event packet, a Single press before Double activation must be
   cancelled on the old release. Two activations Triple→Double must end Double
   with epoch2. The test executes both scenarios before reporting violations.

## Recorded RED and first GREEN

All commands below use the worktree's absolute `CARGO_TARGET_DIR`, incremental
compilation disabled, development/test debug info disabled, `--locked`, and the
existing pinned external interpreter. Root owns all production changes and Git.

| Frozen source / run | Actual outcome |
|---|---|
| First four, baseline `a279bced1fee31eebc163b5d1e02e3ade0fd05ff`; `cargo test -p bondline-ui --lib multiple_bond_tests --locked`; session82522 | Exit101, 0pass/4fail/63filtered. Missing central `tool.bond.single` binding or actual accessible Double button. Runtime behavior failures, not missing Rust APIs. |
| Exact first four after seven real tool-stage files; `cargo test -p bondline-ui --lib multiple_bond_tests::central_palette_controls_icons_and_actual_clicks_have_ephemeral_order_epochs --locked -- --exact`; session86404 | Exit0, 1pass/66filtered. No source or expectation correction. Renderer was not imported and the other three workflows were not run. |
| Append-only fifth test; exact held-preview test; session90282 | Exit101, 0pass/1fail/67filtered. Held Double had one actual preview polygon rather than two. Triple and later cancellation assertions were not reached. |
| Append-only sixth test; exact ordering test; session16213 | Exit101, 0pass/1fail/68filtered. Earlier Single press authored on its old release; Triple→Double ended Triple with epoch1. Both scenarios executed and failed. |
| Exact six after final 30-input production receipt; `cargo test -p bondline-ui --lib multiple_bond_tests --locked`; session14209 | Exit0, 6pass/0fail/63filtered. First complete local GREEN; no test correction. Actual vector/hit/preview/order, PNG, saved CDXML consumer and history/reopen checks all ran. Root's separate receiving run25282 also passed the same six; it is separate evidence. |
| First `cargo test -p bondline-ui --locked`; session44557 | Exit101. All69library+1main tests passed. Attachment16pass/1fail: unchanged `chemistry_guidance_is_visible_before_a_gesture_even_with_an_existing_error` requires visible `terminal CH3 to CH2`. Generic new guidance omitted that deliberate H adjustment. Subsequent integration targets did not run. Production and old test are Root-owned; this worker changed neither. |
| `cargo fmt -p bondline-ui -- --check` after final port | Exit0; formatting only, no mutation. |
| After exact guidance correction, `cargo test -p bondline-ui --locked`; session51499 | Exit0,116pass/0fail/0ignored:69library,1main,17attachment,11drawing,15hover,1SDF,2SMILES; doc-tests0. All six frozen workflows passed again. The unchanged old guidance assertion now passes. |
| `cargo clippy -p bondline-ui --all-targets --locked -- -D warnings`; session3317 | Exit0, no warnings; production, binary and integration-test targets checked. |
| `cargo fmt -p bondline-ui -- --check` after guidance correction | Exit0; no source changes. |

Ignored evidence in this worktree:

| Artifact | Bytes / SHA-256 |
|---|---|
| `target/multiple-ui-red.log` | 8703 / `52d09c1d4409a701666a31a4241c26544a40297c253d69be8a9a5809155e0425` |
| `target/multiple-ui-palette-first.log` | 735 / `4f0ec9938076242c9c9385054181b61a04fc16197d1967184e2db0cb12b6d1bf` |
| `target/multiple-ui-held-preview-red.log` | 1171 / `9ac825f950a9c41ea6611e26ab42aced26673eced56b172c632a97cc97d3487d` |
| `target/multiple-ui-ordering-red.log` | 1265 / `05e905bfa948ac05385385b3fee3ce97524bca31e47dcda6c08123366adf3c42` |
| `target/multiple-ui-six-first-green.log` | 1567 / `915e619498b5b25add7acc8003e2df211e5aaef55dd7b9936908294b90a69ad4` |
| `target/multiple-ui-package-green.log` (prospective filename; actual RED) | 9958 / `a1700a57292fce14ddd55832bd0690416f11d8127b8327cfd0877cba6e784be5` |
| `target/multiple-ui-package-final.log` | 12395 / `586db08c579bdd3fc38b3f94292b77cc192c28ca103be38e409b31b08fb35f81` |
| `target/multiple-ui-clippy-final.log` | 5991 / `c455a301594c5efbb32f9d8d8e934af55a5cf19317bbcea09dc23db1cfcad0e5` |
| `target/multiple-ui-guidance-fmt.log` | 0 / `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |

## Frozen lineage and current limitation

The first four tests are 20,567bytes, SHA-256
`d61f5258133e715b04f9f2a1f489510a2cd712db34d57502e918fb43b3fb3142`.
The fifth was appended without changing that prefix: 23,413bytes, SHA-256
`ffc7658f6664e14e5d367a496358664c3a71cc92233b688129b5ecaa35234adf`.
The sixth was appended without changing the first five: 26,234bytes, SHA-256
`0af46746fefa0afc9ddb0f34dcfc153eb3c1f9d13f6b3e992eaef9a44fb444fa`.
Copies, initial 698-file identities, the seven Root-owned tool-stage inputs and
append-only receipts remain under `target/multiple-ui-*`. The intermediate UIlib
layout drift reconstructs the original import exactly by reversing only rustfmt
line wrapping/if-let layout; its separate reconstruction receipt attributes no
semantic change. The final backend receipt
`target/multiple-final-backend-import.json` is 7647bytes, SHA-256
`a0a0213ab2778b561d9ddc443689df2a751b4ae3bb8c041b0f0abd0f3b5eac87`:
all30 exact imported production/test identities were checked before the first
six-workflow GREEN. The guidance correction has its own7968-byte receipt
`target/multiple-guidance-backend-import.json`, SHA-256
`564a8fa98caca531c5ceaae495f692e5d17c95896c9f900854a48b1f3589b19f`.
Only Root-owned UIlib changed to
`d57cc3bf6506c06bfdf90397c8dbc29c3db7c62f23e3c24d010c2c622a35e117`;
the other29 inputs were exact. The default Single wording was restored with
additional supported literal-H/order guidance. Root separately passed all17
unchanged attachment tests in session29866 before this worktree's full116pass.

Final identities preserve678 of the original698 files byte-for-byte. All20
changed original files are exact Root-owned backend inputs. All11 new source/doc
paths are either received Root inputs or this worker's two owned files. No
baseline file is missing; no unclassified new path exists. In particular, the
Cargo locks/manifests, existing UI tests, fixtures, external consumers, fonts,
asset notices and approved Single golden remain exact. This worker changed only
the new test file and this record, and never corrected a frozen test assertion.
The final handoff under `target/multiple-ui-handoff.json` lists source, protected,
import, prefix and log identities. All local jobs are terminal.

These results verify the bounded original workflows in real headless egui/App
frames and actual saved-file/registered-CPU consumers. They do not establish
native-window behavior, platform accessibility delivery, GPU equality (covered
by the separate renderer gate), style/default parity, offset/ring/join behavior,
or complete M0–M5. Native platform execution remains unverified. No new golden
approval is implied. Git integration and the complete workspace gate belong to
Root.
