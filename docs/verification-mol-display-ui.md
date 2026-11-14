# Source-backed MOL canvas acceptance

Verified headlessly on 2026-11-14: the three app-frame tests pass on Root's corrected
source-aware route, and final UI package tests pass all 110 with warnings-denied
clippy and rustfmt. The expected baseline runtime failures, initial port compiler
failure and test-setup lint failure remain recorded. No native window/input,
compositor, new GPU or approved-golden evidence is claimed.

## Independent expectations

The tests run `BondlineApp::update_workspace` through actual headless egui frames.
They inspect emitted canvas rectangle/closed-polygon primitives and actual
`DrawingState` hits, rather than merely calling a renderer or checking a label's
text. The expected `Page` is independently authored from literal source facts and
uses the protected literal `graph_page_scene` route; it does not call the new
source-aware producer or reconstruct expectations from its output. Equality covers
the ordered glyph masks/ink and stroked geometry at the canvas's actual dimensions
and origin. This is a vector-instruction comparison, not an independent font
rasterizer or an OS screenshot. Existing font/golden evidence remains separate.

Source: the unchanged original 740-byte
`crates/io/tests/fixtures/sdf-preview/valence-absent-mixed.sdf`, SHA-256
`e5cbdc14329c59bad55148c5de37058de90f651af10966c30cf8b89b3b5efd39`.
The [fixture provenance](../crates/io/tests/fixtures/sdf-preview/README.md) and
[MOL display research](research/mol-display-plan.md) record the public CTfile and
depiction sources. No product assets or proprietary sample bytes were copied.

| Actual raw MOL range | Expected default individual-MOL coordinates, pt | Display H |
|---|---|---|
| 0..371, V3000 CCO | (42,60), (78,60), (78,24) | 3,2,1 |
| 410..713, V2000 CN | (108,108), (72,72) | 3,2 |

These coordinates retain the vertical O bond and original C-to-N direction. They
are not SDF-packed coordinates. Every mapped authored H stays `None`; IDs are the
existing dense local MOL IDs, not V3000 source IDs 10/40/90. The test's unchanged
external `mol_oracle.py` must separately read actual saved bytes and report exact
CCO/C2H6O and CN/CH5N, per-atom H and original raw coordinate triples. Its sanitized
ordinary-molecule evidence does not establish a general chemistry implementation.

Before backend receipt, one bounded direct run of that unchanged pinned consumer
verified both original raw spans, H arrays and coordinate triples. This is input-
vector evidence, not passing UI/Save evidence. Individual raw SHA-256 values are
`1c58439ace843b2b84baeb03b9f8540f0f2a576be0df5d6bf967ba303e23ec96`
and `4d608cd176eee3e546d1a429028cbc585d36be156ae08afade6ea1e9570c70ec`;
the ignored output is `target/mol-display-ui-raw-oracle.log`.

[IUPAC GR-2.1.1](https://iupac.qmul.ac.uk/drawing/drawing.html#GR211) supplies the
heteroatom/H depiction requirement; [GR-2.1.5–7](https://iupac.qmul.ac.uk/drawing/drawing.html#GR215)
supplies chemical-anchor/orientation context. The reused closed neutral display-H
policy is an explicit existing Bondline policy, separate from authored data and
generic CDXML absent-H interpretation.

## Three acceptance workflows

1. Open each exact raw span through `document.open_mol`, check complete fully
   mapped backing and literal model coordinates/unknown H, then inspect real app
   canvas primitives. A fresh pointer event over expected H ink, beyond the legacy
   bare atom's hit region, must select the proper O/N atom ID. Frame/hover must
   preserve complete engine state. Save/reopen must keep exact bytes, unknown H,
   source directions, frame geometry and zero authored history.
2. Drag from the actual terminal carbon anchor to empty paper, reaching the
   existing MOL `bond.extend` refusal. In that same frame, press the H label.
   Both repainted vectors and the new gesture's target must use source-aware
   post-command graph acquisition. Engine state must remain unchanged. This test
   intentionally does not invent an empty-paper `bond.create` refusal: the current
   fully mapped MOL guard can allow creation, with later Save limitations.
3. Source-free and native-CDXML pages retain the protected literal policies for
   `None` and authored C/H0 + N/H12, including Save/open and unchanged frame state.
   These controls prevent the new default-H policy from leaking into non-MOL
   backing or repairing unusual literal counts.

The baseline literal-policy control already passes. Missing-H canvas geometry is
an explicit negative control; H-hit points must also fail the old bare-label hit
test at the existing five-unit tolerance.

## Observed RED and freeze

The baseline's initial and post-command graph acquisitions both still call the
literal renderer. Initial runtime test 48765 exited 101: two MOL test functions failed
with 30 actual versus 160 expected primitives, first difference index 5; the
source-free/native-CDXML control passed. After adding explicit external H/coordinate
checks and actual Save/reopen frames, frozen runtime 76764 repeated the same
two failures and one pass. Both failures stop at the first ethanol iteration;
methylamine's source-backed path is exercised by the later GREEN runs. No
production implementation was changed to obtain RED.

Pre-lint frozen test: `crates/ui/src/mol_display_tests.rs`, 17,991 bytes, SHA-256
`44b367efa8154eeb8b6cd31a497d9954f1b20172184e18e512f11fc5a3194185`.
Final RED log: `target/mol-display-ui-frozen-red.log`, 1,812 bytes, SHA-256
`2b32df68287a4fa5d049444c6cb276cfd59b4cecad68bafd3745da98e976007a`.
The initial compile/runtime log is retained separately. The first unexecuted
empty-paper-refusal draft was corrected after source review to use the actual
existing extension guard; no delivered guard or old test was altered.

Root owns the sole `#[cfg(test)] mod mol_display_tests;` bridge. Package rustfmt
moved that module ahead of `mod preview`; reversing only that module reorder
reproduces the precheck library hash exactly. The bridge is an explicit Root-owned
input, not a claimed producer-owned production edit.

Root's first real port uses IO commit
`fd51ca17f36caba16c412d6ef8165a3175e48490` plus pending command/two-site UI routing.
Its three supplied production hashes were verified before execution. Run 6051
exited 101 at compilation: the UI called `bondline_io::display::drawing_page_scene`
without a declared `bondline_io` dependency. The primary errors are E0433, with
subsequent inference/borrowing diagnostics. Tests never ran, and this is not an
image/runtime failure. No producer correction or expectation change was made.
The failed log retains its provisional name
`target/mol-display-ui-focused-green.log`: 4,332 bytes, SHA-256
`4ec5035c8f94b157d7e44961b1566867653af2abc51bb37fc7f8600cab244431`.
The initial receipt and received IO/doc identities are preserved separately.

Root corrected the route through a thin `Engine::drawing_page_scene` IO delegate
and both actual UI graph acquisitions. No manifest, dependency or lock changed.
All three v2 production hashes were verified before execution, and the nine
received IO/test/doc inputs remained exact. The first actual runtime GREEN is
23205, after this compiler-port correction: all three tests pass with the exact
pre-lint frozen source and no expected-value adjustment. Full UI run 46858 then
passes all 110.

Clippy run 28078 exited 101 on three new-test setup lints: the shared directory
helper is also loaded privately by sibling preview tests, the facts tuple needed
a type alias, and the case loop could enumerate the same two `CASES`. Only this
new test was corrected: `LiteralFacts` names the identical tuple; enumeration
retains the same ranges/order; a targeted `duplicate_mod` allow documents reuse
without altering old-test visibility or copying the directory implementation.
The existing helper's atomic directory creation/retry keeps those instances
isolated. No assertions, literals, thresholds or production code changed.

The pre-lint source and exact 1,355-byte setup diff are preserved in ignored
`target/`; diff SHA-256
`36b1ccc407b77e9d99204f5b237a933bf88ad4f6522d7ac3eda2afa33a2ea5e8`.
The failed clippy log is 8,165 bytes, SHA-256
`687ff86183876918d74c61064d4c8af14d0b4cb1bf5574e36923492895a6979e`.
Final test source is 18,150 bytes, SHA-256
`b9df4c1949330bcd8e62c6caa749b14c6746cc319ccf8fe87f0339d655a0c0ac`.

| Check | Actual result | Ignored log |
|---|---|---|
| Frozen three app-frame tests after corrected backend | 3 pass, exit 0; 23205 | `mol-display-ui-focused-v2.log` |
| `cargo test -p bondline-ui --locked` on final source | 110 pass, exit 0; 7745 | `mol-display-ui-final-tests.log` |
| `cargo clippy -p bondline-ui --all-targets --locked -- -D warnings` | Exit 0; 79149 | `mol-display-ui-clippy-green.log` |
| `cargo fmt -p bondline-ui -- --check` | Exit 0 | `mol-display-ui-final-fmt.log` |

Final package counts are 63 library, one binary and 46 integration tests, with
zero failures or ignored tests. Final test log: 11,701 bytes, SHA-256
`8a98a054fc0ce8291020434f02051cb5b3ef274b13bd5400bd9f45df23c22a90`.
Final clippy log: 181 bytes, SHA-256
`8f9c7640c244e90f5b6c0a4125b6e68a30d391c34c0ee9e6539c3b6515888038`.
The Root-owned UI body is independently checked: reversing only its three routing/
snapshot blocks exactly recovers the frozen bridge file, proving that its old unit
test body did not change. Backend and protected-file identities are separate in
the ignored final handoff.

These tests open/save through the actual shared commands, rather than native OS
file pickers. Actual native file/canvas acceptance and Root's receiving workspace
gate remain separate from this headless app-frame evidence.

Runs use this worktree's absolute target, incremental off, dev/test debug zero and
the existing pinned `BONDLINE_PYTHON`. No dependency, old fixture, oracle, font or
approved golden changes. No Git or Actions operation was performed. All producer
jobs are terminal. Actual native acceptance still waits for user availability.
