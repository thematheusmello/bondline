# Status — 2026-11-16

## Current native editable SMILES checkpoint

The complete M0–M5 goal remains **active**. Branch `feat/multiple-bonds`; this native
feature builds on recoverable checkpoint `c85beb1`, engine `12c07c7`, chemistry `2d12d71`
and renderer control `f65a333`. No push or publication occurred. Actions remains disabled
at both repository and workflow level; do not enable or dispatch hosted work. M0, M1
and M2–M5 remain incomplete.

**Verified on macOS:** Edit → Paste Special → SMILES opens the real form. Ordinary OS
clipboard paste, explicit geometry, unsupported-charge refusal with retained draft,
benzene retry, one-step Undo/Redo, existing-bond editing and Unicode Save As/reopen work
within the declared neutral chemistry/layout bounds. Independent XML/RDKit consumers
verify the actual saved 8-atom/7-bond/2-component file, C8H14, unchanged original fragment
bytes, fixed geometry and unchanged input. This is an explicit form, not automatic
clipboard capture or complete reference-dialog parity.

Read [native evidence](docs/verification-native-smiles-paste.md),
[engine evidence](docs/verification-smiles-paste-engine.md) and
[chemistry evidence](docs/verification-smiles-authoring-chem.md). The central presentation
uses `ui.file`; Apply uses the existing `structure.paste_smiles`. There are 31 commands
and 19 default Drawing/13 SDF controls. The native worker retains the two-worker bound,
accepted intent, complete source owner, cancellation and close barriers. Opaque context
identity closes real Save A→B→A races both before commit and during result presentation;
the Rust-only result proof grants no Save acknowledgment or file-publication authority.
Ten native-worker and six actual-App cases pass without weakening old assertions.

**Full local gate 69758: 1245 passed / 0 failed / 1 ignored**, 180 result lines. Rustfmt,
warnings-denied workspace/vendor clippy/tests, layers, licenses and corpus pass. Log
`target/aromatic-authoring/ui-final-ci.log` is 151705 bytes, SHA256
`ae8fc643726caab92a5fa41f81cf2348809f11162907252bffc41fdb7a51a365`.
Native build 36938 exits 0. The prior catalogue and ABA RED failures/fixes are recorded
in the native evidence. Runtime closures remain 155/166/221 packages plus two assets;
corpus remains eight files/five bundles with broader chemistry/CDX coverage missing.
The earlier MCP `AddrInUse` failure remains unexplained; this pass is not a causal fix.
No parser fuzzing, dependency/lock, SDK, font, old fixture/oracle, corpus or golden change.

The only live Bondline preview is PID 39430, bundle
`target/smiles-ring-native/Bondline.app`, identifier `org.bondline.neutral-simple-ring-review`.
It is Saved on `native-edited-λ.cdxml`, 8/7/2 at 100%, Single selected; no control port was
requested. Binary 20988368 bytes, SHA256
`cdecd57c69b81d91ae63ce70702094478147eb2fc8bb2c764ce2b2a5e1b96c14`.
Old Bondline previews are closed. No Bondline tabs remain in the inspected Chrome windows;
unrelated tabs/apps were preserved. Codex/terminal tabs were not inspected. All compiler,
test, oracle and native build jobs are terminal, including 28908, 69758 and 36938; do not
poll old handles. All agent handoffs are returned. Root owns continuation.

**Remaining:** exact reference dialog/defaults and automatic clipboard completion;
native Windows/Linux and race/cancel/poison checks; measured DPI/2×, Fit/readability and
broader chemistry/layout. The Windows shortcut is wired, not executed here. Neither the
old approved golden nor synthetic App tests establish those workflows.

**Next executable task:** follow Gate B of the
[source-qualified SMILES plan](docs/research/smiles-source-import-plan.md). First declare
independent selected/all-record preservation and writer expectations, then implement the
complete retained source/current mapping and notation-only record transaction with real
Undo/Redo and independently checked Save/reopen. Do not grant edit authority to a partial
projection or silently discard opaque/unselected bytes. Continue the M2 atom, selection,
ring and style roadmap after that boundary; no new converter-only detour is required.

The sections below preserve historical checkpoints. Their process IDs, live-job statements
and next-task proposals describe those earlier revisions, not the current state above.

## Previous editable SMILES engine checkpoint

The complete M0–M5 goal remains **active**. Branch feat/multiple-bonds. Recoverable
feature12c07c7 delivers the actual one-step Drawing paste command, supported layout and
CLI/stdio/authenticated-loopback workflows. Chemistry2d12d71 and renderer controlf65a333
are separate atomic commits. No push/publication/hosted run occurred; Actions stays off.
M0/M1 and M2–M5 remain incomplete.

Read [engine evidence](docs/verification-smiles-paste-engine.md),
[chemistry evidence](docs/verification-smiles-authoring-chem.md) and the
[source contract](docs/research/aromatic-authoring-contract.md). Actual neutral fixed-H/
Kekulé matching supports the declared C/N/O aromatic subset without inventing perception
or a4n+2 gate. The new layout accepts supported integer S/D small rings and all supported
components; old normalizer and four converter outputs/refusals stay separate and exact.

`structure.paste_smiles` is the31st command, with six closed required fields. Off-lock
work retains the complete native/source-free Drawing owner, checks all page geometry/
literal ink, remaps fresh global IDs and commits once. Strict cancel/source/revision/
generation/close guards and branch identity floors prevent stale or partial edits.
It publishes no file and has no Save lease/acknowledgment authority. Unknown/opaque or
SDF/SMILES-source sessions refuse rather than replacing their complete source.

Independent evidence: core9+IO6 and323 IO regression passes; layout6+command6+actualwire3
pass first. Real CLI/initialized stdio/authenticated socket append into nonempty Drawing,
Undo/Redo, edit and Unicode Save/reopen pass. Independent Expat/RDKit checks actual files
and complete graph/H/directions/IDs, including C6H6→C6H8. Source-only reviews report no
material finding, separately from execution. New test setup corrections remain recorded.

**Full local receiving71552 passes1229/0/1**,179 result lines, with rustfmt, warnings-
denied workspace/vendor clippy/tests, layers, licenses and corpus. Log149898B SHA256
`6181effc288002b4eadced066f640932053b01fdba50de23332ec5e77aab54cb`.
Earlier62848(83/1) and62137(182/1) expose omitted new-name expectations in old catalogue
tests; all eight exact lists now add only the new name and retain every old assertion.
Runtime closures155/166/221 plus2 assets pass; corpus remains8 files/5 bundles with its
broader chemistry/CDX omissions. No external version/font/fixture/oracle/SDK/golden change.

The old MCP AddrInUse failures remain unexplained; this passing unchanged test does not
establish a causal repair. Native Fit/readability, effective-DPI/2×, Windows/Linux
interaction and broader chemical/model features remain open. The existing saved native
preview is still the only Bondline window; it has not been refreshed to this command yet.

Next executable integration is the actual native Paste Special form and ordinary OS
TextEdit paste, using the existing typed host presentation and registered two-worker
budget. The independent five-App-test source is declared, uncommitted as the next feature;
its bridge is added and missing-control run28908 is live at this checkpoint. No UI product
implementation is claimed yet. The native-worker tests are being declared under ignored
target/aromatic-authoring/ui-work-tests/; do not copy them into the crate until receiving.
All engine/IO/CLI/CI jobs above are terminal; do not poll71552/62848/62137/94719/45008.
Root owns compilation; agents have no Cargo/build permission. After actual missing-control
RED, implement the scoped dialog/worker guards, run native acceptance and update this record.

## Previous native density checkpoint

The complete M0–M5 goal remains **active**. Branch feat/multiple-bonds; feature7da33ee
commits the native-density implementation and qualified receiving evidence.5bc8deb records
its contract;02e81f0 records the next SMILES authoring/source-import study. The current
full aggregate is NOT green. Feature6e6110b and checkpoint6ecc4b1 deliver bounded neutral-
component conversion and its earlier1188/0/1 gate;6ac876d records that source contract.
Preserve the distinct current/historical scopes. All intentional project work is committed.
Feature11101ce delivers canonical CDX FileEnd;691af6f records that checkpoint.9d8ad75
preserves bounded MCP shutdown observations. Earlier4d1736f/9050ab1 deliver simple-ring
conversion and its renderer prerequisite. Continue existing work; do not restart.
No push, merge or publication occurred.

M0's desktop/GPU license blocker is resolved; **full M0/M1 and M2–M5 remain incomplete**.
Actions stays disabled: repository Actions off and workflow417397204 manually disabled.
Do not enable, dispatch or rerun hosted work. New Windows/Linux interaction is unverified.

## Current native Drawing density increment

Read [the receiving record](docs/verification-native-drawing-density.md) and
[contract](docs/research/native-drawing-density-plan.md). The separate exact-DPI/fractional-
origin display API shares existing capture geometry; native physical painting, source-
qualified scenes, input/hover/drag mapping and logical text bounds are wired. Existing1×
pixels/refusals and2048 readback limits remain; vector geometry retains100000 bounds.
No new registry command, source/edit/Save authority, font or asset is introduced.

Independent existing-API Metal control90091 passes1/0 first. Actual-App37559 gives1/1,
expanded20715 gives2/2, and fractional44447 gives0/1 after valid phase[.25,.75] setup.
Missing API gives one then ten E0599 only. Before production, all original declarations/
prefixes and298 protected inputs remain exact except the approved73-byte test bridge.
Root93429's first
candidate passes5 renderer+5 App cases unchanged. Source-only Engine/UI review finds no
material finding; this is separate from execution. Three product pins are in ignored
target/drawing-density/root-first-product.json.

**Full local gate78964 FAILED767/1**,115 result lines, at old unchanged MCP control
shutdown_without_clients_releases_the_listener_address: replacement_bind127.0.0.1:54691,
AddrInUse/Os48. Log94,414B SHA256
58502c60022de553eecda0fea87f21c44d47034ea80a5249fb5a5316f2a68615. Fmt and warnings-denied
workspace clippy pass before that failure. One delayed lsof/netstat snapshot is empty;
there is no failure-time owner/cause evidence. Do not rerun the exhausted old diagnostics,
failed test or aggregate simply to make it green; do not change retries/assertions/reuse/
deadlines/concurrency. This is the same previously documented unexplained failure class.

Root21912 is terminal0 and runs ONLY stages the failed gate did not reach: remaining MCP
23/0; model/render/UI/compat/xtask403/0/1; remaining docs1/0; native vendor3/0. These430/0/1
and vendor clippy/layers/licenses/corpus pass; they are not an aggregate CI pass. Runtime
closures154/165/220 with2 assets and corpus8 files/5 bundles remain. Native build passes.
Rust1.94.1, CARGO_INCREMENTAL0, BUILD_JOBS2, DEV/TEST_DEBUG0 and literal pinned Python used.

Actual new Mac binary20,531,936B SHA256
`a290421488b83805d48194acff6ef9c0d7fc626ad6c5eb1fd5dcbca672949f9f` reuses only the existing
review bundle, exact-path PID39216/no socket. Original component file opens Saved9/7 at
Fit78.79. Two coordinate clicks with Double selected do not mutate; cause is unknown.
Key2 at the first bond commits Double/H2 and Fit still refuses/clears the canvas. CmdZ
restores SavedSingle; ShiftCmdZ restores dirtyDouble; F5Actual100renders. Unicode Save As
and explicit reopen leave Saved density-edited-λ.cdxml,9/7/1,100%, Double selected.

The actual1,407-byte output equals the earlier edited file exactly, SHA256
`d7c9074698b737d37ff15addf8858232992468a5536739bcfe5959c324af3cb2`; unchanged raw/Decimal/
Expat/RDKit verifies9 atoms/7 bonds/3 components and C8H18O. All original inputs/sentinel
stay exact. Native receipt5,409B SHA256
`917f151a7d66c22269f4b04dbabb891b46047003bb822070b4b1994d47fd5dcf` is under ignored
`target/drawing-density/native/receiving-receipt.json`. No native Fit fix, first-click,
measured effective-DPI/canvas, actual2×, full overview/pan or other-platform claim follows.
The5+5 App/reference evidence remains distinct. Legacy fractional-origin1× is unchanged;
proper antialiased native readability and measured-DPI acceptance require further work.
No new golden/fuzz/dependency/SDK/parser/old-fixture/oracle/lock/asset change.

Final protection passes: six product/test pins exact,294/298 old inputs unchanged; only
three intended product files and the73-byte test bridge differ. All582 parity IDs/order/
missing clipboard rows, the original MCP control source and349 local links pass. Final
rustfmt/Git whitespace checks pass. Protection receipt763B SHA256
`1bb74548dc5bbfc59f2d2b0b3ef2919e92c0e74b40878b475144110969bb1a8f` remains under ignored
target/drawing-density/. Only orientation prose changed afterward, correcting Double-
selected click provenance and the current renderer call map; product/tests stay exact.

All compiler/test/oracle/build jobs are terminal;78964/21912/93429 are not live handles.
All handoffs are returned and agents stopped; Root owns continuation. The current saved
preview is the only Bondline process. Old previews remain closed; unrelated apps/tabs
are untouched. Feature7da33ee preserves the coherent increment with its known failure/
limits. No push/publication/hosted action occurred. Disk was1.6GiB free during the gate;
avoid duplicate targets/heavy builds without checking current space.
Next executable authoring gate is the source-backed [SMILES plan](docs/research/smiles-source-import-plan.md):
real benzene/aromatic representation and independent graph/H/depiction expectations,
then one undoable selected-string Paste Special append and registered Save/reopen.
It is research only; no new paste/aromatic command exists. Full file record rewriting,
broader chemistry, complete reference flows and M0–M5 remain required. Investigate current
native Fit/coordinate-click evidence from actual input/render facts; do not infer a cause
or silently substitute a different camera/density to hide a refusal.

## Previously verified bounded neutral SMILES components

The current native-density section supersedes this historical process/view/task state.

Read [the contract](docs/research/smiles-components-plan.md) and
[receiving evidence](docs/verification-smiles-components-integration.md).
The additive fourth neutral_components_v1 choice retains one complete
normalized Fragment, every global ID/directed edge/fixed H and the whole notation.
Actual-edge BFS components each use the existing supported tree or3–8 Single-cycle pose,
with64 total atoms. Original source-order center-box shelves use gapL, fixed poses/no
scaling, divided search quotas and whole-scene certification. One-component output stays
exact. This is not general source-qualified drawing import or a reference packing claim.

Root64926 first observes missing-profile runtime RED0/1. Final independent eight-case
21,683B declaration fails exactly8E0599 before product GO; its original1,867-byte prefix
remains. The unchanged13,909B new Expat/Decimal/RDKit consumer is first qualified on seven
manual positives/20 negatives. First UI selector runs zero tests, then corrected unique
filter reaches actual missing-radio RED. New App projected()->Pos2 '?' and unused-import
setup corrections, and workspace-edition2024 formatting, retain all inputs/assertions and
first511B UI/760B command prefixes. These are not product defects or successful checks.

Root90518 first IO receiving passes8/0. Root5986 passes command3/schema1/wire2/App3.
Complete-source preservation, create-only publication, explicit Open/edit/Undo/Redo/
Unicode Save/reopen and actual CLI/stdio/authenticated loopback pass. Mixed9/7 CPU/GPU
capture matches exactly. Product11-file pins remain exact; old successful paths/defaults,
30 commands, eight required scalars, leases/worker/close/history and noautoOpen/SaveACK
remain. Source-only independent reviews report no material finding; they are not execution.

Root59627 is **terminal0**: full cargo xtask ci **1188/0/1**,171 result lines, then matching
native binary build. Rustfmt, warnings-denied all-target workspace/vendor clippy/tests,
layers, licenses and corpus pass. Log target/smiles-components/root-full-ci.log144,934B,
SHA256fc0778bdce5498e14f113c6f8ca31a7f4ea8cd8544bb7e4794b71929f0b2e7d3.
Runtime closures154/165/220 packages and2 assets; corpus8 files/5 bundles. No new parser-
fuzz run, dependency/lock/SDK/font/asset/old-oracle/fixture/corpus/golden change. The earlier
MCP AddrInUse failure remains unexplained; this passing run is not a causal fix.

Actual Mac native workflow passes legacy-profile refusal, occupied-output refusal, Unicode
retry retaining the known own dirty7/7 rotated ring, explicit Open Cancel then Discard/Open,
Double first C-C at100%, CmdZ SavedSingle, ShiftCmdZ UnsavedDouble, Unicode Save As and
explicit reopen Saved9/7. Both actual1,407B files pass the unchanged independent whole-
graph consumer:9 atoms/7 bonds/3 components, C8H20O→C8H18O. Original output SHA256
96b96531004420eafd30ace781a12077b0efe1f8df7e03ec0611d69a3d56e025; edited SHA256
d7c9074698b737d37ff15addf8858232992468a5536739bcfe5959c324af3cb2. Input15B, sentinel28B
and prior saved ring1,254B remain exact. Native receipt10,272B SHA256
019d9b1e80a97bff42d2da3b981478eebe4d26cfb398c9b1bf77d653aabfcf6f.

**Display limit reproduced:** edited-scene Fit78.79% refuses distinct multiple-bond pixels
and clears the canvas with its error.100% is restored and renders; a top ring H label is
partly clipped. AX slider SetValue and coordinate drag produced no change; no full overview
or native pan success is claimed. Investigate the logical/physical render boundary with
an independent fixed-phase/DPI case before changing the guard. Do not weaken the existing
CPU/GPU distinct-component/gap negative tests. Native races/cancel/close/DPI and other
platform input remain unverified. No new golden approval.

Only the existing target/smiles-ring-native/Bondline.app bundle was refreshed, identifier
org.bondline.neutral-simple-ring-review, exact-path PID21951, no socket. Binary23,010,584B
SHA256dc19eb622f8ccd9202c66373ad3667c255be994d0ec10a5c2b3f3b100ca7b696. It is Saved at
edited-components-化学.cdxml,9/7/1,100%, Double selected. All older previews remain closed;
never query a stopped app binding to verify closure, since getApp/getAXState can relaunch it.
Chrome has no visible Bondline tab; terminal UI access is restricted. Unrelated apps/tabs
are untouched. The preceding cleanup turn made verified progress through app/tab inventory.

Final protection passes:11 exact product pins, independent IO/oracle identities and first
declaration prefixes, append-only old bridges, fourth enum expectation only, all582 parity
IDs/order/clipboard-missing rows and381 local links.950/975 baseline files remain exact;
25 expected differences include the already-committed6ac876d question note. The first
checker omitted that note, then verified its exact committed bytes before correcting only
the checker. Final rustfmt/Git whitespace checks pass. The receiving note preserves details.

All Cargo/test/oracle/fuzz/build jobs are terminal; do not poll59627 or duplicate a gate.
All handoffs are returned; Root owns continuation. No external publication or hosted job
is live. Source-only review finds the distinct-pixel guard correct on its declared logical
lattice; an unexecuted848×728 A4 candidate analytically matches the78.79% failure. Native
canvas/DPI were not measured. Next write an independent fixed-phase1x-refusal/2x-visible
test using the existing physical-raster API, then establish actual-App native DPI routing
before changing shared rendering. Preserve old negative phases/widths and logical camera/
hit/source/history semantics. In parallel define source-qualified SMILES import
(selected/all-record ownership, complete opaque
backing/baseline/IDs, stale jobs/history/dirty decisions and explicit Save/loss authority).
Broader chemistry/rings/packing/clipboard, SDF creation/reordering and M0–M5 remain required.

## Previously verified canonical CDX file terminators

The component checkpoint above supersedes this historical process/view/task state.

The preceding cleanup turn made verified progress: all obsolete Bondline previews were
closed, without touching unrelated tabs. This receiving turn resumes the actual CDX
slice from [the public framing review](docs/research/cdx-file-end-plan.md).
Every legacy success retains priority; only TrailingData plus a canonical28-byte zero-
reserved header may select implicit/explicit DocumentEnd + exactly one FileEnd + EOF.
The old22-byte HeaderDocument rule remains. Canonical legacy+one end deliberately evolves
acceptance, and removing that final word can still be a valid legacy document. This is
not padding tolerance, producer/property heuristics or a proof that all truncations fail.

The only production file changed is crates/cdx/src/lib.rs,15134B/SHA256
c3f27f4645372fc70e75f93fa9168f4f6d4bf74c1bcdf1e631500213e79f81cd.
The shared exhaustive root classifier also prevents fuzz reconstruction from dropping
the first0805 property. The original258-byte external Indigo source, generators, licenses
and qualified new raw/RDKit oracle are retained. Writer, old fixtures/oracles, locks,
dependencies, corpus pins and the approved golden remain unchanged.

Independent runtime RED48668/Root59054 reproduces TrailingData. Expanded syntax first
fails compilation for missing variants; a separate new-test-only Arc::ptr_eq setup error
is preserved and corrected with content equality plus existing capture stamps. Root69681
passes8 syntax cases;76198 passes IO1/Engine3/actual-wire2, including CLI, stdio MCP and
authenticated loopback. Eighteen independent oracle controls pass. Ten distinct original
external files now parse and pass through exactly; this is syntax evidence, not general
application mapping. See [integration](docs/verification-cdx-file-end-integration.md).

The exact fixed seed first fails fuzz property conservation11vs12; this is a harness
classification defect, not an ASan memory error. Corrected fixed-input86996 and actual
10000-mutation ASan45946 pass. The ten declared seeds/old fixtures stay immutable;
mutations live under .cache/fuzz/cdx-file-end-20261115. Largest seed65750B, not64MiB.

Earlier full local gate97729 **fails754/1** at the unchanged old MCP shutdown case:
replacement_bind127.0.0.1:51670, AddrInUse/Os48. Formatting and warnings-denied workspace
clippy pass before that test failure; later aggregate stages did not execute. Preserve
target/cdx-file-end/root-final-ci.log,91080B/SHA256
78153a7443ff803009751be506278666a9b200a823b07f755cf8d40edc1e4a88.
The fixed20 default-control contexts64662 pass500/0; their bound is exhausted. The
single98404 controlled probe contradicts the proposed held-outbound collision: replacement
std listener binding also succeeds. Neither identifies the historical owner/cause.
No retry, deadline, reuse option, assertion or default parallelism has changed.

Exactly one final declared Root receiving gate81324 **passes1172/0/1**,169 result lines.
Formatting, warnings-denied all-target workspace/vendor clippy/tests, layers, licenses and
corpus pass; original MCP source and all three declared source pins remain exact.
Log target/cdx-file-end/root-receiving-ci.log is141627B/SHA256
a59098b56b4293de7e7da3f81b26ff1f2e61c6137d6ceea256cc6509b2e8ded6.
This is receiving evidence, not a causal fix for97729. Keep that failure explicit. Runtime
closures remain154/165/220 packages with2 assets; corpus8 files/5 bundles. See
[bounded diagnostic evidence](docs/verification-loopback-shutdown-observations.md).

Native Mac receiving passes actual Open, Unicode Save As and explicit reopen. The
actual roundtrip-λ.cdx is258B/source-exact SHA256
856482269dbd85640b87c11f608104dba72ebd308a686f1333a893965ed560ba and passes the independent
raw/RDKit consumer. The UI shows Saved/CDX, Original source preserved and faithful-preview
refusal; its2/1 counts are IO/consumer evidence only. Chemical edits/cross-format/native
races and other platforms were not exercised. Receipt target/cdx-file-end/native/
receiving-receipt.json is2737B/SHA256
2718e89bfb8a6ea3d7ebad2dce2539dce38c691d214d95d06b9f2f79bced5c8b.

One current bundle remains: target/smiles-ring-native/Bondline.app,
org.bondline.neutral-simple-ring-review, exact-path PID69285, no socket. Refreshed binary
22936488B/SHA256c052d758e209e1c0e53d016b6c02d93da427bc2b077b7272de04803a87285ad9.
Previous Saved edited-ring-λ.cdxml7/7 is restored at Fit78.79%, complete ring visible;
the1254-byte file remains exact8c95f0a244ee817d2e9be0ead26aa6be2c71b4dcd28c24a3b0189d6d5a175056.
Old preview apps remain closed. Do not revisit their stopped bindings to verify closure:
getApp/getAXState can relaunch them. ChemDraw/unrelated apps and tabs remain untouched.

Final protection/link checks pass:21 old syntax inputs and940 unchanged baseline files
remain exact, with only16 expected existing-file differences. CLI parent has only65
appended bytes; first1667-byte syntax/1685-byte Engine declarations remain exact prefixes.
All582 parity IDs/order and missing ME-015/025 rows remain;378 local links across16
documents pass. Source-only fuzz rustfmt and Git diff checks pass. Only docs changed
after81324. No SDK, shipped dependency, old fixture/oracle/font/asset/corpus/golden change.

All compiler/fuzz/test/investigation jobs are terminal, including native build79022,
receiving81324 and diagnostic64662/98404. All CDX/documentation handoffs are received.
Root retains50 sealed diagnostic source/log/receipt files under ignored
target/cdx-file-end/independent-listener-investigation; executables are excluded.
The later component section above supersedes this checkpoint's agent/task state.
Root owns STATUS/Git and other edits; consult actual handoffs before overlapping files.
No hosted work or publication is live.

## Previously verified simple-ring increment

The existing create-only converter offers Carbon chains, Acyclic neutral structures and
additive Neutral ring structures / neutral_simple_ring_v1. The third profile accepts one
actual connected3..8-atom Single cycle with supported attached trees,≤64 neutral
nonisotopic H/C/N/O/F/Cl/Br/I atoms, fixed-H and supported Single/Double/Triple branches.
Source/IR/IDs/direction/H, exact length and paper remain. Origin is minimum atom-center
bounds. Full geometry/intersection/stroke/glyph/paper and literal/resolved-scene
certificates remain. Larger/polycyclic/component/aromatic/charged/isotopic and unsupported
double-bond cases refuse. It is neither a ring drawing tool nor general SMILES import.

One30-command registry retains eight required scalars and two old profiles/default/
serialization/receipt behavior. The third optional closed value shares the full notation
owner, create-only publisher, Save lease, two-worker lifecycle, source/file/close freshness,
ordered radio actions and truthful historical results. Current/saved source, history,
association and native Save/ACK are unchanged; generated CDXML requires explicit Open.

Renderer prerequisites reproduce literal degree-two H occluding/picking a bond and a
separate density-specific four-candidate H-placement refusal. Authored-H degree≥2 uses
the existing safe path; four compact above/below offsets are appended after old choices.
Font/em/count/anchor/masks and all collision/readability guards remain. Unknown H stays
unknown. The high-density rotated-star control passes first and is not called a defect.
Old private constructors and old renderer tests remain exact.

Read [integrated evidence](docs/verification-smiles-ring-integration.md) and its independent
layout/renderer records. Nine IO, four command, two actual-wire, three App and four renderer
cases pass; producer default IO308/0. A ninth IO case was appended after implementation
to exercise cycle-boundary tree keys and is honestly passing-first coverage.
Independent connected43-file review finds no material finding; it is source review only.

## Previously verified simple-ring gate and corrected failures

Root38859 full gate passed fmt/clippy then failed an old listener test with Os48/AddrInUse
after740 passes. Its original phase/port/owner/cause remain unknown. Declared unchanged
diagnostics5 exact+3 default contexts and300 lifecycle/2048 churn attempts all pass.
A diagnostic-only one-function test patch adds phase/local-address error context while
preserving operations, assertions, deadlines, ErrorKind and exclusive-bind policy.
No production MCP or socket-security change; this is not a causal fix.

Root95551 passes that listener test but reaches179/2/1 UI: two unchanged old Carbon/
Acyclic receipt assertions see a success heading before clipped counts. The third profile
prose exposes egui's prior-size anchored-window clipping. The shared form now reads its
previous size before show and requests re-layout only for successful measured height
growth. No extra test wait/fixture/readiness/SDK change. Root26008 passes all13 conversion
App cases unchanged. The earlier new-test-only frame-readiness correction was insufficient
to preserve legacy presentation; both stages remain in the evidence.

Final full local gate71585 **passes1158/0/1**, using Rust1.94.1, literal
.cache/oracles/format-python/bin/python, CARGO_INCREMENTAL=0 and CARGO_BUILD_JOBS=2.
All166 result lines, formatting, warnings-denied workspace/vendor clippy/tests, layers,
licenses and corpus pass; the ignored test is the explicit visual-capture utility.
Log target/smiles-ring/root-final-ci.log is140,000 bytes/SHA256
5a71e81792427bee67bea5793d38ae115d5cde1f7ad394dc3e7af217b8306a57.
Earlier failure logs remain. No compiler/fuzz/debugger or worker job is live. All handoffs
are received. Runtime license closures stay154/165/220 with2 assets; corpus8 files/5 bundles.

Snapshotv2: target/smiles-ring/root-final-product-snapshot-v2.json,2339 bytes/SHA256
aa694a6023ebc787b8341de6c4d0bb815f06fc4d900bfff5f13e55fe5f103be8.
Only the shared UI sizing fix differs from the earlier native-tested twelve-file snapshot.
All11 other product files and frozen old/new tests/SDK sources remain exact.
Final rustfmt/diff checks,389 local link targets across23 changed documents and all12
committed product-source pins pass. Only documentation changed after the final gate.
No dependency/lock, font/asset, corpus, old external consumer or approved golden change.
The sole approved golden remains1b650231527f47db5dc7c368644939907d2b038d32f25e77578318f483521bbc.

## Previously verified simple-ring native and reference evidence

The first ring bundle native build46792 passes. Actual Mac workflow passes: retained
unsaved C–C; two legacy topology refusals; ring existing-output refusal; Unicode retry
without replacing current work; explicit Open Cancel then Discard/Open; external-bond
Double edit; CmdZ Saved Single; CmdShiftZ dirty Double; separate Unicode Save As/reopen.
The100% initial view clips a top label until native scrollbar pan exposes the full ring.
AX numeric SetValue made no immediate visible change; no provider cause is inferred.

Both actual1254-byte outputs pass unchanged independent Expat/Decimal/RDKit consumers
against frozen original vectors: CC1CCCCC1/C7H14→C=C1CCCCC1/C7H12. Their SHA256:
51124ce519403559e49a6f576492db55e89fbb0b9cc034131f599e5c08b6cd4b and
8c95f0a244ee817d2e9be0ead26aa6be2c71b4dcd28c24a3b0189d6d5a175056.
Input/sentinel and first output remain exact. Later native build37830 and receiving
acceptance pass: complete visible ring receipt, separate new output consumed independently
and byte-identical to the original, explicit edited-file Open and visible pan. Initial
inactive-window menu attempts produce no form; native title click/F5 and subsequent
ordinary menu interaction work. This receiving pass does not repeat every prior history/
refusal check or establish native first-pass timing.

At that checkpoint, bundle target/smiles-ring-native/Bondline.app, identifier
org.bondline.neutral-simple-ring-review, had exact-path PID10202, no socket, and
binary22,936,248 bytes/SHA2c2ea9fb5a2a64bbbc4b0c96a0aab29f378a6e9f6f0be7e22d41b9c2f35ad427.
The earlier native-tested binary is retained under ignored target, not misattributed.
It was Saved at edited-ring-λ.cdxml,7/7,100%, vertical center300. The current CDX receiving
bundle/process/view above supersedes that historical state. On the user's cleanup instruction, CUA closes
the three older Carbon/New/Acyclic review apps after checking saved state; final inventory
confirms all three not running, current Ring and ChemDraw running. Do not reopen obsolete
bundles to check their state: getAXState on a stopped app can relaunch it.
Unrelated project apps/browser tabs remain untouched. Browser host is unavailable; native
Chrome's visible tabs show no scoped Bondline reference tab.

ChemDraw About is rechecked26.1.0.6801/Product Signals, not Professional entitlement.
The renewed CUA review opens Layout/Drawing/Atom Labels/Reaction Display settings and
cancels without edits; [LIVE-012–018](docs/research/live-ui-review-2026-11-15.md) records
functional controls and selected-document values, not defaults or completed setting flows.
The blank reference document remains open. REA MCP runtime tools are absent; CUA is the
actual producer. No static binary/resource inspection or vendor asset copying.

## Remaining scope and next exact work

1. Receive/review the component-packing source plan from reference_map. Freeze independent
   source/graph/geometry and refusal/history/wire expectations before implementation.
   Reuse the existing eight-scalar create-only converter/profile/worker lifecycle, preserve
   every component/input ID/direction/H and exact bond length, and validate the full scene.
   Original packing policy must be distinguished from documented reference behavior.
   General source-qualified import and full parity remain the target, not just utilities.
2. Continue components and general source-qualified SMILES drawing import, preserving
   full-source authority. SDF creation/reordering/headers and wider CDXML/MOL/CDX editing
   remain. Ring authoring/selection/labels/styles, chemistry/stereo/reactions/text,
   publishing/clipboard, naming/templates/biopolymers/WASM/installers remain required.
3. M0 still needs full modern inventory beyond576 anchors, dialog/accessibility and
   interactive Windows/Linux. Reference palette/tool/flyout/clipboard flows and styled
   multi-window New remain gaps. Geometry typing/browsing/races, large/bulky native,
   hardware DPI, other platforms and further fuzzing are not newly verified.
4. Native CDX receiving and bounded ASan are complete; do not repeat them without a new
   source change or concrete concern. The earlier intermittent failure's cause remains
   unresolved despite the passing final aggregate. Keep only the current Bondline preview
   open. Inspect the agent/live handles above.
   Use the literal pinned Python without resolving its symlink. TDD and source
   decisions accompany each feature. Local atomic commits only; no push/publication or
   Actions enablement. Inspect live handles and frozen worker handoffs before resuming.
