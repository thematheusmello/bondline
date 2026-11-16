# Skeletal carbon display and explicit-label restoration

2026-11-16; locked contractfb17db9 follows data caa8d60. The
[public-source contract](research/skeletal-carbon-display.md) separates SDK/IUPAC facts
from original compatibility, numerical and authoring policies. This is bounded shared
display/restore support, not new ring tools, general chemistry or full M0–M5 parity.

## Delivered behavior

The private render/skeletal helper collects actual incidence once per flagged fragment.
None and dormant noncarbon/degree0–1 flags retain the old label path. Active true prints
the known literal/resolved C/H label. Active false requires supplied H and checked actual
Single/Double/Triple weight+h=4 before suppression. All two-Double centers and degree2
identical rays at the inclusive authored-f64 threshold retain full labels. Unknown H and
invalid proofs refuse; no inference or chemical/model/source mutation occurs.

Hidden labels contribute neither glyph pixels nor white masks; stable atom anchors/hits
remain. Existing real component/gap/junction/ink/paper/scale/resource/cancel guards run
unchanged with actual remaining masks. label_em_px=None means no labels were drawn;
native no-font handling and the separate SDF-preview requirement are unchanged. Complete
source authority, unknown root/Text refusal, MOL/retained-SMILES loss guards and legacy
native-v0 bytes stay intact. No runtime dependency, asset, font, shader or parser changes.

The existing atom.set_label flips only an existing false to true along with explicit
element/H. Same-count false restoration is one normal commit/history/revision; None/true
same-count calls retain validated sharing/no-op/redo/close behavior. It can explicitly
repair unknown/inconsistent H without a prior render/C4 proof. Changing to O retains true
as dormant data. The same registry, parameters and existing native editor perform this;
no new command/form/control or source authority. Registry38/control counts29/20/14/10 stay.

## Actual test-first and qualification record

Root executes all checks; workers independently declare tests and review sources only.
The pre-render suite preserves its initial2,529-byte hexagon case. The command expansion
preserves the corrected1,988-byte first case; App expansions preserve3,027/15,290 prefixes.
Only the old DATA blanket-refusal expectation is positively upgraded, with its same
collinear/terminal inputs, stronger actual label/CPU/hit assertions and exact old legacy/
cancel suffix. Other old assertions/fixtures/consumers/goldens remain unchanged.

| Stage | Actual evidence |
|---|---|
| Renderer69592 | Initial original regular Single hexagon runtime RED0/1, UnsupportedChemicalContent. |
| Command75539 | Setup failure: manual native input lacks the exact XML prolog. No producer change. |
| Command69870 | Prolog-only correction, all assertions/graph unchanged; intended same-count no-op RED0/1. |
| Initial App | First filter selects0, not behavior evidence. Correct full module executes1 and fails false versus true after actual Apply. |
| App22424 | Three pre-product actual-App cases RED0/3: form no-op/missing strokes/hidden target unavailable. |
| Glyphless square | First filter selects0; corrected exact test executes1 and finds0 real bonds versus4. |
| Core5474 | Pre-render expansion RED3/4; old no-op/source/metadata controls pass first. |
| Wire11450 | Actual CLI/initialized stdio/authenticated-loopback RED0/3. |
| Renderer83598 / DATA94794 | Pre-render five-case RED0/5; DATA legacy/cancel pass first, positive scene RED2/1. |
| Core53027 /44003 | Initial restoration GREEN1, then all7 core cases including actual-file/capture/history GREEN. |
| Renderer47879 | Five pre-render cases plus3 DATA GREEN; full CPU/Metal-GPU Kekulé frame agrees. |
| Renderer32425 | Post-first-draft supplement:9/1, with sole failure in its exact-dot fixture precondition before scene construction. |
| Boundary2585 | Corrected mathematical fixture passes exact Rust precondition, visible full label and hit; no product change. |
| App59672 / wire32123 | Four actual-App and three real transport cases GREEN, including Unicode native FileWorkflow Save/reopen and unchanged external graph/RDKit consumers. |

The supplement adds169/171/180/mirrored/reversed/camera/density cases, exact threshold,
straight/bent two-Double, physical-H and existing refusal/cancel controls. These5 cases
are post-first-renderer-draft acceptance, not preimplementation REDs. Total new cases24:
render10, core7, transport3 and App4; old DATA count stays3.

The original boundary Y normalized to one floating-point step beyond the intended cosine.
Root derives Y=sqrt(1−K²) using90-digit Decimal on the exact represented locked K, then
rounds to0.17364817766693058. This single literal correction preserves the threshold,
strict precondition, all assertions and the complete original source on inverse replace.
It is fixture qualification, not a demonstrated renderer defect or predicate relaxation.
Raw corrected source SHA2569b63e07e6cf65610767972bd28cc066a5d31b80971b23cecc2e97c421fb7e989;
independent source-review receipt3052B SHA2563f6343d5cb3aedf91f0429287c5346b126743d4ad85c86b80ac0594fbf7db867.

The first aggregate51082 stops in clippy on one new-test assert_eq!(bool,false).
Replacing only that comparison with assert!(!bool) preserves its exact meaning; no lint
allowance or assertion removal. The new raw declaration and inverse proof are retained.
Corrected aggregate42092 exits0:1494 passed/0 failed/1 ignored across212 result lines,
including fmt, warnings-denied workspace/vendor clippy/tests, layers/licenses/corpus.
Log184162B SHA25606a217eafd4b03a53d3733798c1663b616dfe2053975e4ef29f6af3c876f97e4.
Native/CLI build63733 exits0; native binary SHA256
a00f57bd446ea26b451481228d047a50956ae7179c89f82ec26f10a0dbc7e4e7; CLI SHA256
ac419139d597d14077ce49750f89874fa73a3cfef76fa729762120ebb515cd22.
No natural MCP retry or socket/timing policy change; historical AddrInUse remains
unattributed. All named Cargo handles and reviewer jobs are terminal.

The bounded independent source review finds no material product issue. Report5921B SHA256
e5dbac5a844d22073702f6cc72b17947616fb2ba1a66131acc800a864162314a;
source receipt11678B SHA256eaa774f45876f13231662bd05652fae4f07ccf3f63f9b0d3eff067257e47f8ae.
It predates later test-only numeric/lint corrections and does not claim their executions.
Ignored target/skeletal-display/root-receiving records exact declarations, formatting-only
ports, correction proofs, product/log identities and the actual root/toolchain environment.

## Reproduction and limits

```sh
export BONDLINE_PYTHON="$PWD/.cache/oracles/format-python/bin/python"
export CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0
cargo test -p bondline-render --test skeletal_carbon --test carbon_label_data --locked
cargo test -p bondline-commands --test skeletal_label --locked
cargo test -p bondline-cli --test sdf_capture_adapters --locked skeletal_label_wire
cargo test -p bondline-ui --lib --locked view_tests::native_atom_label::skeletal_display::
cargo xtask ci
```

No parser fuzz is repeated for this unchanged-parser increment. App tests inspect actual
egui output at density2, forms, target picking, event/history/source fences and actual
file-worker publication; they do not establish native OS input or measured hardware DPI.
The user's desktop remains on bundle desktop-review-caa8d60 with its unsaved drawing
untouched; it is not hot-reloaded. No new approved golden, hosted run, push or publication.
Actions stays off. Defaults/terminal-H preferences, false-setting authoring controls,
weighted unsaturated ring tools, imported style/Text and cross-platform parity remain.
