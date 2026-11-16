# Roadmap

The complete target is an offline desktop and WebAssembly chemical editor with familiar
documented workflows. The initial reference baseline is Professional 26.1; older public
guides are versioned evidence, not proof of complete modern parity.

The shared `ring.create` command now implements explicit3–8 carbon Single polygons,
including shared-atom and shared-Single-edge attachments. Independent geometry,
cyclohexane/decalin/spiro/terminal chemistry, source/history/ID guards, Unicode files,
CPU/GPU and actual CLI/MCP/socket checks pass. Its rounded-precision counterexamples
are fixed without a blanket minimum length. Registry37, native controls unchanged;
see [receiving](docs/verification-ring-authoring.md). Native ring tools/preview/gestures
are the next required integration, followed by aromatic/chair/sprout/styles and broader
chemistry. This is a checked engine increment, not delivered native ring parity or M2.

Bounded native Mac atom labels now support fresh Alt-click target/fields/Apply/Cancel,
OH/NH2 editing, history, draft-close protection and Unicode editable CDXML Save/reopen.
Independent consumers verify the actual files; see
[atom-editor evidence](docs/verification-native-atom-labels.md). This original form is not
full inline Text/alias/charge/isotope/radical parity. The selection checkpoint's registry36 adds
persistent selection and literal translation; native Drawing controls23, retained-SMILES
Drawing20, SDF14 and raw SMILES10. Independent command/context/wire and actual-App input
checks now exercise selection/movement/history/Unicode persistence. Native selection
execution remains unverified; see [receiving](docs/verification-selection-authoring.md).
The atom checkpoint's full gate fails848/1 at the old MCP replacement bind; remaining480/0/1 and
xtask stages pass separately. The current selection gate passes1361/0/1 with no socket
policy change; that pass does not explain the historical failure. The [controlled inheritance experiment](docs/research/macos-listener-lifetime.md)
does not attribute that natural failure. Full M0–M5 remains incomplete. Continue source-
preserving M2 authoring and independent native checks; keep broader format/chemistry,
publishing and M5 workflows visible rather than treating this slice as completion.
The [locked selection/translation contract](docs/research/selection-translation-plan.md)
now has a source-qualified implementation, reviewed local gates and an inspected
offscreen App frame. Finish actual native acceptance, then continue ring/chain/style
and broader label workflows. Full
selection/transform/26.1 parity remains required beyond this bounded atom-set slice.
The [ring-authoring contract](docs/research/ring-authoring-plan.md) locks the delivered
command and independent cyclohexane/decalin/spiro persistence. Lock its remaining native
tool/gesture/default policies before App REDs; it does not resolve aromatic/chair/sprout
or style workflows.

Native source-backed SMILES records now have bounded Mac selected/all import, refusal/
retry, registered click/key order editing, history and Unicode whole-file Save/reopen/
reimport, with independent actual-file consumers. See
[native receiving](docs/verification-native-smiles-records.md) and the earlier
headless1270/0/1 record checkpoint. The later atom-label command gate passes1306/0/1;
the earlier intermittent MCP listener-release AddrInUse failure remains unexplained.
Fit78.79% still refuses the edited multiple-bond scene, Actual100% recovers, pan
attempts did not change the view, and native races/DPI/other platforms remain unverified.
Full M0–M5 remains open. Continue M2 atom/selection/ring/style authoring after preserving
these complete-source/worker boundaries; broader record and chemistry parity remains.

The earlier native editable SMILES checkpoint passes the full local gate1245/0/1, with bounded
Mac OS clipboard/refusal/retry/history/edit/Unicode Save/reopen and independent file checks.
The earlier density pipeline78964's MCP `AddrInUse` failure remains unexplained; this pass
is not a causal fix. Automatic clipboard capture, exact reference-dialog behavior and
native2×/DPI remain unverified. See
[paste evidence](docs/verification-native-smiles-paste.md) and
[density evidence](docs/verification-native-drawing-density.md).

## M0: foundation — in progress

- [x] Resolve declared runtime/asset license closure for native egui/wgpu on all three platforms.
      Actual Mac Metal drawing/readback verified; hosted Mac/Windows window smoke passes.
      Linux native launch passes after the runtime-package correction below.
- [x] Cargo workspace, pinned toolchain, lints, build-time layer and license gates.
- [ ] Pure document model, named typed JSON commands, transactions and undo/redo tests.
      Page controls/history are delivered; headless elemental graphs now have structural/
      identity/budget tests. Independent C-C creation and transactional drag/history now work;
      connected editing, broader models and a general transaction/job engine remain.
      Conditional close intent now joins the shared registry, with epoch-bound Save and
      native worker/reply shutdown barriers. Shared delivered File/Edit/Page bindings and guarded
      file-presentation requests now exist; broader dialog discovery remains incomplete;
      see docs/research/m0-command-ui-audit.md for the remaining distinction.
- [x] CLI, stdio MCP and opt-in loopback control using that registry.
- [ ] Native UI and CPU/GPU reference canvas with actual launch/render evidence.
      Native Mac drawing/history/recovery and one human-approved golden now pass.
      All three hosted window smokes pass; interactive Windows/Linux workflows and broader rendering
      remain unverified.
- [x] macOS/Windows/Linux CI, licensed corpus tooling, parser fuzz infrastructure.
      Corrected hosted run 43451706668 passes aggregate checks and actual window launches
      on all three platforms plus Linux ASan. Corpus now checks eight pinned files from
      five bundles: CDXML, MOLV2000/V3000, ordered SDF, one original CDX scene and three
      original SMILES notations. Foreign/general CDX, broader SMILES/InChI and chemistry
      corpus coverage remain missing.
- [ ] Complete public-document menu/tool/shortcut inventory; continuation instructions.
      582 stable-ID source-cited control rows now exist, including three provisional
      unversioned Help roles; modern menus, unnamed glyphs and remaining
      context menus still need evidence. Six versioned screen/flow groups and pinned public
      research now guide implementation; the download center requires login and observed
      Signals demo states are not desktop evidence. Do not infer completeness from row counts.

## M1: files — in progress

Fresh document sessions now have `document.new`, File → New Document and primaryN,
guarded dirty/draft decisions and Save-before-New through actual worker retirement.
Independent tests reproduce and close both old-surface pointer races. The local aggregate
passes1090/0/1; actual Mac SDF→New, Cancel/Discard/Save, Unicode output, editable reopening
and Undo pass. See docs/verification-new-document.md. Styles/templates and multiple
windows remain missing. This closes the source-file→fresh-drawing dead end, not M0/M1.
The later user-authorized live26.1.0.6801 pass actually observes New creating a second
styled window while retaining the first; see docs/research/live-ui-review-2026-11-15.md.
That independent-window/style workflow remains required for parity.

The original native restricted SMILES→CDXML workflow is now delivered through the same
registry and registered-worker lifecycle. Input/output browsing, explicit geometry,
existing-destination refusal, successful Unicode retry without replacing unsaved work,
guarded explicit Open, Double edit/Undo and separate Save As/reopen pass on macOS.
Independent consumers accept both actual433-byte outputs. The final local aggregate
passes1116/0/1; native race/close/cancellation and other platforms remain unverified.
See docs/verification-native-smiles-conversion.md. This is a1–12 neutral-carbon Single
path utility, not general SMILES import or Copy As/Paste Special parity.

The additive neutral-acyclic conversion profile now delivers connected trees up to64
H/C/N/O/F/Cl/Br/I atoms, branches and Single/Double/Triple edges under the existing fixed-H
policy and bounded geometry. Original default/profile/receipt contracts remain exact.
Eight IO, two renderer, five command, two actual-wire and three App checks add20 passes;
the full local gate passes1136/0/1. Actual Mac profile/refusal/Unicode retry, retained unsaved
work, guarded Open, branch Double edit/Undo/Redo and Unicode Save As/reopen pass. Both actual
766-byte files agree with independent raw/RDKit expectations. Source/H, stereo, geometry
and platform limits are in docs/verification-smiles-acyclic-integration.md.

The additive `neutral_simple_ring_v1` profile now implements one actual connected3–8
Single cycle plus supported attached trees, at most64 neutral H/C/N/O/F/Cl/Br/I atoms.
It retains fixed H/source/IDs, exact supplied length, minimum atom-center origin and full
geometry/ink guards without scaling. The ordered third native choice uses the same
create-only command/worker/lease and grants no active-session Save or source-file editing
authority. Focused9 IO/4 command/2 wire/3 App/4 renderer checks pass, as do the producer's
308 default IO tests. Bounded native Mac profile/refusal/Unicode retry, retained unsaved
work, explicit Open Cancel/Discard, external methyl-bond Double edit/Undo/Redo and separate
Unicode Save As/reopen pass. Independent readers accept both actual1254-byte outputs,
C7H14→C7H12; source and existing-output sentinel remain exact. Geometry entry/browsing,
native races, large/bulky cases and other platforms remain unverified. The final local aggregate passes1158/0/1, including unchanged legacy receipt checks
after the shared dialog sizing fix. See [the integration record](docs/verification-smiles-ring-integration.md),
[independent layout checks](docs/verification-smiles-ring-layout.md), and the separate
[ring attachment](docs/verification-literal-ring-attachment.md) and
[compact-H extension](docs/verification-literal-compact-star-attachment.md) evidence.

The additive `neutral_components_v1` / **Neutral components** fourth choice now derives
all supported actual components into one complete Fragment, up to 64 total atoms. It
normalizes once, retains global arrays/IDs/directed edges/fixed H and changes positions
only. Existing tree or 3–8 Single-cycle poses use source-order center-box shelves with
gap L, no rotation/rescaling, divided search quotas and one whole geometry/ink certificate.
One-component output remains byte-exact with the corresponding earlier profile.

Root90518 passes eight independent IO cases. Root5986 passes three command, one schema,
two actual-wire and three App cases, including the complete 9-atom/7-bond/3-component
C8H20O drawing, identical CPU/GPU capture, explicit Open, first-edge Double edit to C8H18O,
Undo/Redo and Unicode Save/reopen through independent consumers. The fourth profile's
component checkpoint's full local gate passed1188/0/1, including fmt, workspace/vendor warnings-denied clippy,
tests, layers, licenses and corpus. Bounded Mac legacy/existing-output refusals, Unicode
retry with dirty work retained, explicit Open Cancel/Discard, Double edit/Undo/Redo and
Unicode Save As/reopen now pass. Independent consumers verify both actual 1407-byte
9/7/3 outputs as C8H20O/C8H18O, with only the intended first-edge order/H change; input,
sentinel and prior ring remain exact. Authoring worked at100%; edited Fit78.79% refuses
the distinct-pixel bond guard and an initial top label was clipped. Full native overview/
pan and other platforms remain unverified. Earlier counts stay historical checkpoints;
no new parser-fuzz or dependency change.
See [component integration](docs/verification-smiles-components-integration.md) and
[independent layout evidence](docs/verification-smiles-components-layout.md).

Drawing's additive physical-display path now shares the existing pixel transform with
exact effective DPI/fractional device origin, fresh glyph generation and inverse paint/hit
mapping. Logical zoom, source authority and the old1×/capture routes remain unchanged.
Five renderer and five actual-App cases pass, including honest1× refusal,2× Metal strip/
gap checks, fractional phase and logical-readability limits. This is headless verification,
not a native2× measurement or a Fit fix. The latest Mac build still refuses the edited
component scene at Fit78.79%; hotspot2/Undo/Redo/Actual100% and exact Unicode Save/reopen
pass, while two clicks with Double selected produced no mutation of unknown cause. Full overview/
pan remains unverified. [Receiving evidence](docs/verification-native-drawing-density.md)
records the failed aggregate and separately passing remaining checks.

The external CDX framing counterexample now has an additive canonical 28-byte implicit/
explicit Document + FileEnd path: eight syntax and six IO/engine/wire cases pass, retaining
all legacy successes, source bytes and limited application authority. The new oracle's
18 controls and a 10,000-run ASan mutation smoke pass. Bounded Mac Open, Unicode Save As
and explicit reopen preserve all 258 original bytes, with faithful preview still refused.
The final local receiving gate81324 passes1172/0/1, including the original MCP test.
Earlier97729's754/1 replacement-bind AddrInUse remains unexplained; twenty passing
diagnostic contexts and a contradicted outbound-owner hypothesis establish no causal fix.
See [integration](docs/verification-cdx-file-end-integration.md) and
[independent declarations](docs/verification-cdx-file-end-tests.md). No corpus admission
or broad CDX compatibility follows. That checkpoint proposed source-qualified SMILES drawing
import: selected/all-record behavior, complete opaque source/baseline and ID mapping,
stale-job/history/unsaved decisions and explicit Save/loss authority. Broader packing,
larger/polycyclic/multiple-bond rings and chemistry remain; retain unsupported source
rather than silently projecting it. The new utility still does not grant an opened SMILES source
session molecular editing authority or complete clipboard parity.
SDF creation/reordering and wider format editing stay in the backlog. The earlier proposed
SDF reordering priority was deferred after the sourced File New audit, not implemented.
The [SMILES authoring/import study](docs/research/smiles-source-import-plan.md) recommends
editable Paste Special with real benzene support, then source-preserving record editing
and whole-file Save. Neutral fixed-H/Kekulé chemistry, bounded layout and the one-step
Drawing append now pass independent engine/CLI/stdio/socket and actual saved-file checks;
the explicit native Paste Special form now adds actual OS clipboard/refusal/retry,
one Undo/Redo, native bond edit and Unicode Save/reopen. The full gate passes1245/0/1;
see docs/verification-native-smiles-paste.md. The later bounded record engine now adds
selected/all retained Drawing import, synchronized order edits and whole-file Save through
all delivered headless adapters. The later [native record gate](docs/verification-native-smiles-records.md)
adds bounded Mac explicit import/order/history/whole-file Save/reopen. Wider navigation, automatic
clipboard capture and broader chemistry/import remain. Finish native record acceptance
from the same plan, then the missing M2 atom/selection/ring/style workflows below.

Whole-record SDF removal now has a source-preserving codec, the shared typed command,
CLI/stdio/authenticated-loopback routes and original Records confirmation UI. Independent
literal raw/per-MOL consumers and actual-App checks cover exact surviving bytes,
Undo/Redo, draft decisions and Unicode Save/reopen. A real pre-spawn file-context race
was reproduced, fixed and retested without changing its regression. Root's final local
gate passes1060/0/1; native evidence is scoped in
docs/verification-sdf-record-removal.md. Record creation/reordering, broader format
editing and complete M1 remain required. This does not complete M0 or any later milestone.

The original SDF occurrence inspector and existing-value editor are delivered. Nine
independent actual-App editor cases cover exact duplicate-field edits, history, real Save,
Apply→Save input ordering, navigation decisions, external requests/source replacement,
no-op CRLF retention and refusal without publication. Two finite registered-worker gates
protect real commit/cancellation/retirement. Native Mac multiline Apply, Unicode Save As,
Undo/Redo and reopen preserve the predeclared788-byte output and both molecular records.
The editor's994-test local gate passes. See docs/verification-sdf-editor-integration.md and the
earlier inspector evidence. Native draft/close races, record operations and full M1 remain
required next work; the later view acceptance is below.
The subsequent [SDF view integration](docs/verification-sdf-view-integration.md) now delivers
Actual/Zoom/pan through the shared camera and source-preserving current-view captures.
Seven command cases, four actual-App cases, two native-adapter gates and eight renderer
cases pass; the full local gate is1016/0/1. Native Mac F5/Actual and both position controls
visibly display the ordinary CCO/CN records without changing source/history. Native wheel,
physical-DPI measurement and the combined edit/preview/Save workflow remain unverified.
The [original proposal and receiving decision](docs/research/sdf-view-plan.md) retain why
success-only prepared bounds replaced the proposed error-side-effect bootstrap.

The subsequent Fit readout correction labels SDF Fit textually before/after extent
preparation and retains explicit percentage entry, without changing legacy images or
camera routing. Actual-App cases and native read-only AX/painting are recorded in
docs/verification-fit-readout.md. Native percentage typing remains separate; high-DPI
relative steps still use the declared logical Fit basis rather than a measured overview.

A restricted all-record SDF capture route now connects source-preserving mixed-version
projection, separate display-H provenance and faithful resolved-H labels through the
existing command/CLI/MCP interfaces. Ordinary VAL-absent CCO/CN, both source orientations,
full CPU/GPU image comparisons and exact Save pass headlessly. The native read-only workspace
preview now has registered background work, source/view/DPI/
close-epoch invalidation and independent actual-frame texture/paint/retirement tests. Actual
default Fit can still refuse its em guard; the bounded native Actual/pan recovery above
now works. Ordinary source-backed MOL
absent-H depiction now uses the same closed display facts in its actual Page/IDs/frame,
with complete deferred ownership, independent CPU/GPU/source checks and headless canvas
acceptance. Authored None and source bytes remain exact; source-free/CDXML stay literal.
See docs/verification-mol-display-tests.md and docs/verification-mol-display-ui.md.
Centered Double/Triple depiction now passes independent analytic pixel/geometry, actual
CPU/GPU and source-preserving MOL/SDF checks. It uses an original3w profile, without
mapping inherited styles or context-sensitive sidedness. See
docs/verification-multiple-render.md and docs/verification-multiple-commands.md.
Broader formats, source metadata, SDF record operations and full M1 interoperability remain unfinished.
This is partial M1, with neutral Single/Double/Triple geometry and explicit whole-capture
refusals; general layout and reference packing
remain open. See docs/verification-sdf-capture-command.md and the linked independent evidence.

The complete SDF session/Open/Save slice now retains every mixed-version record and
ordered data byte through current/saved/history and all shared adapters. Native Mac mixed-
version Open/Save/Unicode Save As/reopen and failed-import retention now pass onbb6d5b9;
the690-byte and additional refusal checks are controller-only. Root's590 gate and matching
three-platform hosted checks pass. Bounded field inspection/existing-value commands now pass engine/CLI/MCP tests.
Broader native chemical preview acceptance, record operations and
broader SDF interoperability remain missing; see docs/verification-sdf-session-files.md and
docs/verification-native-sdf-and-extension.md. That earlier native build used a static summary;
the new read-only preview is wired/headlessly tested, with its native workflow unverified.
This is not M1 completion.

The V3000 application slice now retains versioned source/header/frame/history and exposes
both versions through the existing two shared MOL commands. Auto Save keeps its associated
version; Save As chooses V2000/V3000 explicitly; Open validates the actual family member.
Independent IO/engine/CLI/stdio/loopback and UI tests plus the corrected local372+3 gate pass.
Actual Mac author/save/reopen/refusal/close and CPU/GPU reply draining now pass at1c53312.
This feature's own hosted checks pass in43531603511 on all three platforms; see
docs/verification-v3000-application.md. Conservative conversion/layout/source-loss refusals
remain. Broader SDF/native editing workflows and mappings still follow; this is not M1 completion.

Current user-directed priority: finish M1, then continue M2–M5. MOLV2000 now has a source-
preserving codec, owned backing/history, atomic filesystem I/O and two shared commands;
engine/CLI/MCP tests use published BIOVIA2026 fields and external RDKit consumers. The native
format choice/Save/reopen/close workflow is verified on Mac; all three hosted aggregates and
native window smokes pass at c879f4c. One CTAB/fragment and layout/metadata guards
are partial interoperability, not completed M1. SDF V2000 now has a source-preserving
container library, ordered duplicate/empty/DT data, a wrapper writer and independent
raw-field/molecular consumers. A fetched original SDF corpus now checks two records and
every ordered field without a lossy property map. Broader SDF editing/display and wider mappings remain next work. Standalone V3000 parsing,
writing and the version-aware MOL application slice are now integrated; mixed-version SDF
container and reviewed legacy RXN container are integrated.
These libraries retain complete raw molecular/data/role payloads; SDF whole-session files
are now wired, while reaction models/commands/native workflows remain separate missing work.
The original bounded SMILES notation/source library now has literal graph/token/limit
tests and qualified pinned external-parser evidence. Literal aromatic atom/bond markers
are preserved without perception. Bare implicit H, bracket H0 and
physical H remain distinct. Named opt-in normalization materializes only a closed neutral
fixed-H subset; a separate Fragment exporter generates noncanonical notation. Both have
literal/external-consumer/boundary tests and combined ASAN evidence. General drawing
import/layout, aromaticity/stereo/perception and canonicalization remain missing. Complete
source-file ownership is a distinct path below. See docs/verification-smiles.md.
Full M0
inventory/dialog/addressability gaps remain tracked while this format work proceeds.

The restricted carbon-path layout now also has a create-only filesystem converter into
new editable native CDXML. Its 64 KiB input, source preservation, publication failure/race
guards and independently consumed actual output pass the local 670-test aggregate.
The shared conversion command now passes engine/CLI/stdio/authenticated-loopback workflows
through explicit CDXML Open/edit/history/Save/reopen, plus the local 686-test gate. It
preserves the active session and uses the same save lease through finish/drop. Native
conversion controls/flows and general cleanup remain missing.

The explicit-H writer now also creates real single-record SMILES files through the shared
atomic publisher, preserving its isotope/physical-H/branch/ring/disconnected/Single/Double/
Triple scope. Exact LF framing, actual-file consumers and publication guards pass the local
704-test gate. The shared explicit-selection command/adapters now pass the local715 gate,
including actual nonfirst native-source export, full graph/metadata/order comparisons and
semantic negative controls. It retains the complete owner/save lease through finish/drop,
reports omitted geometry/document context and preserves the active session. Native export
controls and broader chemistry remain missing.

Complete accepted SMILES files now retain all records, ignored lines, opaque data and
LF/CRLF/final-EOF bytes through source ownership, bounded native I/O and the shared
document.open_smiles/save_smiles commands. Any unsupported/malformed later notation
rejects the whole Open. Independent raw and unchanged per-record consumers verify the
codec/file adapters. Commands/CLI/stdio/authenticated-loopback retain complete snapshots,
save leases and generation/close rules; summary state has no dummy drawing or automatic
record data. A separate ASan target passes a bounded10,000-run smoke check. Native chooser
and static summary are wired/headlessly tested, with actual native acceptance pending
user availability. All-record chemical preview/editing, record operations, broader syntax
and native conversion/export remain missing. See [file/application evidence](docs/verification-smiles-file-app.md)
and [UI evidence](docs/verification-smiles-file-ui.md). M0/M1 remain incomplete and Actions remains disabled.

The next [original record-inspector plan](docs/research/smiles-native-workflow-plan.md)
now has a delivered typed native query bridge: three literal/background/guard/authority
tests and the full Root1024/0/1 local gate pass. See docs/verification-smiles-ui-work.md.
The actual UI component is now delivered with five independent App cases and two finite
worker/close gates. Root's full local gate passes1031/0/1. Native Mac AX/painting shows
ordinal/Inspect/navigation and no automatic notation disclosure, with an exact native-
process query/source check. Native input/clipboard/dialog acceptance, chemical preview
and complete M1 remain unverified/missing. See docs/verification-smiles-record-integration.md.


Standalone CDXML source preservation and checked existing-attribute replacement have
original-fixture/Expat and ASan smoke evidence plus one pinned unchanged RDKit corpus
document. A source-backed direct-fragment node geometry view now uses validated model
points; missing coordinates remain absent. Model atom/bond/fragment storage is validated.
Read-only elemental mapping now assigns fresh application IDs, resolves fragment-local
references and retains source backing; an external Expat/Decimal oracle checks the corpus.
Owned application snapshots/history retain that backing; unsynchronized edits cannot
export stale XML. New native elemental CDXML encoding now preserves modeled metadata,
connectivity and explicit Poster canvas extents; an external RDKit consumer opens the
actual single-bond command's output. Shared registry/native CDXML Open/Save/Save As and
Mac safe-close/reopen/edit workflows now have independent file-consumer evidence. Unknown
source passthrough/refusal, bounded atomic saves and background/stale-result guards pass.
The original single-document chooser is partial File coverage; broader imported editing remains.
Native independent C-C/three-H rendering is delivered, with explicit terminal labels.
Source/history now has shared-allocation accounting and oldest-first byte eviction.
A proven generated-file profile now reopens/edits/renders headlessly, retaining source and
exact history; extended/noncanonical files remain guarded. Next: general source-preserving
structural edits, broader native file parity and retention/recovery scenarios;
broader query/label/style mapping, application CDX mapping/editing, MOL/SDF V2000
and V3000, RXN, SMILES and explicit data-loss diagnostics. Every fixture needs a pinned
source, license and SHA-256. Never silently treat unsupported formats as passing.

The first standalone CDX slice now indexes bounded little-endian records, preserves unknown
source bytes and checks opaque property replacements. Original vectors face independent
Python struct and external RDKit consumers. Implicit/explicit roots have specific probe
evidence. A bounded new-tree syntax writer now independently reproduces an original chemical
scene and passes literal lengths/resource checks and external RDKit. General file compatibility
and editable encoding/faithful preview remain missing. Guarded shared/native Mac CDX
Open/Save preserves accepted source exactly; it does not regenerate changed projections.
One original explicit chemical scene is fetched from its real public commit,
SHA-verified and checked against independent struct/RDKit consumers. Foreign/general CDX
corpus and reference/chemical mapping remain M1 work.

## M2: drawing — in progress

Current ring command37 verifies explicit3–8/free/fused/spiro construction through the
engine/adapters/files/renderers; native ring controls and gestures are next. Selection
and movement are wired with12 actual-App passes, but native desktop acceptance remains
pending. See [ring receiving](docs/verification-ring-authoring.md) and
[selection receiving](docs/verification-selection-authoring.md) for exact scope.

The [elemental authoring contract](docs/research/atom-label-authoring-contract.md) now
has a bounded headless `atom.set_label` implementation. Nine core, four actual-wire and
one independent MOL-reference CPU/GPU case pass, including complete history and actual
saved-file consumers; see [receiving evidence](docs/verification-atom-label-authoring.md).
It changes only explicit element/literal-H fields in existing neutral atoms in source-free/
native CDXML. The original `ui.atom_label` presentation now adds fresh atom/label targets,
explicit fields, Apply/Cancel and stale/modal/Save/close guards. Actual Mac OH/NH2,
history, close protection and Unicode Save/reopen pass; see
[native evidence](docs/verification-native-atom-labels.md). No general valence inference.
Modern label-entry and repeat-label evidence stay separate; full inline labels, aliases,
charges/isotopes/radicals, full native selection/transforms/rings/chains and styles remain required.

The shared118-symbol catalog, positioned neutral-element labels and literal H have
independent font/CPU/GPU/file evidence. MOL/SDF display H stays separate from authored
None. Centered Single/Double/Triple scenes use logical bond IDs, actual component hits,
attachment/mask/visibility guards and the unchanged approved Single golden. Other metadata,
context-sensitive multiple placement and source-faithful styles remain missing.

`bond.create` accepts an explicit Single/Double/Triple order (defaultSingle) and authors
literal H=`4-order_weight`. `bond.extend` preserves identities/source/history while
extending a consistent neutral-carbon terminal within its explicit H budget; source-free
or byte-proven native CDXML only. Native Single C3/C4 authoring/history/CDXML reopening
has Mac evidence. The new multiple workflows are wired and pass independent actual-frame,
file/chemistry/image tests. Bounded native multiple creation now has Mac evidence at the
recorded readable scale; native multiple terminal extension remains unverified.

`ui.bond_tool` supplies three procedural accessible palette controls and a checked
transient epoch, outside authored history/dirty/source state. Ordered activations cancel
older gestures; held previews and hover use the same actual component geometry. Existing
Single/attachment regressions remain. See docs/verification-multiple-ui.md and
docs/verification-multiple-authoring.md. This does not finish M2 or establish26.1 styles.

The original refreshed workspace has a compact rail and contextual document inspector,
with source/count/file state and stable paper geometry. Native macOS visual review,
Single/Double/Triple selection, pointer authoring, existing-edge edits, history and Unicode
CDXML Save As now have finite actual-window evidence. The later scene is native-authored;
its420pt-square page was explicitly registry-prepared. Broader native attachment, source,
platform and view workflows remain unverified. See docs/verification-bond-order-native.md;
the earlier registry-authored captures retain their original scope.

Existing-bond editing now has a sourced public-guide contract and one typed
`bond.set_order` command with explicit order/revision/IDs and a required literal neutral-
carbon C4 H policy. Exact no-op/history/source/overflow/error guards, real CLI/stdio/socket,
independent Expat/Decimal/RDKit saved files and fixed-seed edit properties pass. The UI
wires absolute1/2/3, source-supported Single conversion clicks and Single-tool endpoint
Single→Double / Double→Triple; unsupported alignment/click cases report their gap.
Ordered commands reacquire full backing, graph and page geometry. See
verification-bond-order-{command,ui,native}.md under docs/ for exact evidence and limits.
Actual native Mac drawing/conversion/hotkeys/history/Unicode Save As pass for the recorded
readable-scale scene; no universal26.1 click matrix or native multiple extension is claimed.

Usable transient view navigation now has one typed command, exact legacy Fit, uniform
logical zoom, numeric entry, Actual/Fit/Magnify/Reduce, two-axis controls and wheel pan.
Independent renderer/command/adapters/actual-App tests and native Mac unchanged-A4
Single/Double/Triple creation and150% editing/persistence pass. The fresh-focus SDK
filter and rapid Undo→Save prepass bugs were independently reproduced and corrected.
The aggregate961-test gate passes; source/scale/label guards remain strict. No physical
DPI calibration, SDF navigation, saved view preferences or complete reference parity.
See docs/verification-view-{render,commands,ui,native}.md and
docs/verification-ordered-file-actions.md.

Next: native ring tools/gestures and outstanding desktop selection acceptance, then
aromatic/chair/sprout/chain construction, snapping, transforms, broader labels and styles.
M1 source-faithful interoperability and M0 addressability/inventory/platform evidence
remain open. Keep imported-source/mapping gaps visible; these bounded workflows do not
complete a milestone.

Bonds, rings, chains, labels/abbreviations, charges, isotopes, radicals, wedge/hash bonds,
selection, transforms, hotkeys, undo/redo and style sheets from public journal guidelines.

## M3: chemistry — planned

Valence, aromaticity, stereo/CIP, checking, cleanup, formula/mass, analysis, reactions,
curved arrows, brackets and chemically formatted captions. Compare against independently
run RDKit, reference InChI and OPSIN; document conventions and tolerances before testing.

## M4: publishing — planned

SVG/PDF/PNG, physical-scale output, documented clipboard/Office workflows, atomic saving,
recovery and independent export consumers. Editable embedding needs public evidence.

## M5: depth and distribution — planned

Independent name/structure conversion, original templates, biopolymers/HELM, WebAssembly,
packaging and signed installers. Keep documented advanced capabilities (including naming,
predictions, annotations and file types) visible in the parity backlog until supported.

No milestone claims full commercial parity. Account/cloud/licensing services are not required
for Bondline; unsupported reference integrations remain identified rather than disappearing.

The [May2026 capability audit](docs/research/capability-chart-audit.md) refines existing
milestones with explicit placement-aware paste, dark drawing styles, axial/M/P stereo,
chirality-flag interpretation, hydrogen-bond relationships, named descriptors, additional
format directions and monomer cases. All remain missing with source/platform qualifications.
Optional scientific-service connectors must not become offline creation dependencies.
The [CDX candidate review](docs/research/cdx-corpus-candidate.md) adds a concrete unsupported
framing/provenance question; it does not add a passing corpus file or justify relaxed parsing.

The [installed desktop map](docs/research/installed-feature-map.md) and
[implementation tickets](docs/research/installed-implementation-tickets.md) add exact-build
control/state/flow evidence without replacing the complete M0–M5 target. First-pass503 menu
entries and a successful original local-file flow do not complete inventory or parity.
Native CDX framing precedes safe CDX projection/application I/O; collection ownership and
record editing precede application SDF; general graph scenes/hit testing precede connected
gestures. Account/cloud routes stay optional to the offline authoring product.

The header-embedded CDX framing slice now preserves the22-byte prefix/root/separate-file-end
profile while keeping successful legacy parses and all existing strict negative tests.
An unchanged original native1196-byte save and original public-field vectors face independent
record/byte/consumer checks. Receiving root417+3 and normal-graph ASAN pass; the new source
needs hosted checks. This parser feature precedes model mapping and application CDX files.

The first CDX elemental view and owned complete-source snapshot now have literal/raw/
consumer/identity/bounds/error/history-storage evidence. Unchanged export is exact; edited/
cross-format/model-generated export is refused. Native metadata and application CDX I/O
remain next, alongside source-aware safe rendering. Receiving438+3 and normal adapter
ASAN pass; this new source requires hosted checks. Do not count it as completed M1.

The CDX filesystem library now reuses bounded reads and security-preserving atomic saves
for unchanged complete snapshots. Both original framing variants' actual output has strict
independent consumer checks; receiving443+3 and all aggregate gates pass. Registry/CLI/MCP/
native chooser, broader native metadata and editing remain separate next features. This
library-only slice needs its own hosted checks and does not complete application CDX or M1.


Guarded CDX application files now have exact-build Mac native chooser/Known Save/Unicode
Save As/reopen/retry/dirty-close refusal and Undo recovery, plus independent actual-output
consumers. Four original variants and one ignored own-drawing source retain every byte.
Source450962c passes its own three-platform aggregates/window smokes and six ASAN targets.
The16-command terminal-extension integration501a315 separately passes523/521/513 hosted
tests/windows and six ASAN targets. Actual native attachment acceptance remains next; SDF
collections remain M1 library-to-application work. No milestone is completed by these slices.


The SDF whole-source ownership prerequisite now passes9 new literal/consumer/identity/
resource/accounting cases, receiving549 aggregate checks and a normal10000-run ASAN smoke.
It retains all records/fields/versions in shared immutable snapshots; Engine/file/native
collection integration remains next. Field edits are being implemented as a separate
immutable candidate operation, not yet exposed as an application command. Installed SDF
observation confirms both drawings in one document; an initial record browser is unproven.


Existing SDF field-value replacement now has10 independent literal/no-op/invalid/boundary/
consumer cases, receiving559 aggregate checks and normal10000-run ASAN. It preserves every
other source byte and leaves original snapshots immutable. Whole-collection Engine/current/
saved/history/file commands and native data editing remain the next M1 integration; this
library is not advertised as an application editor or complete format parity.

## Explicit SMILES record addressability

The source-file slice now has smiles.record.inspect: one explicit revision/ordinal, exact
notation and global ranges/counts, with opaque data retained privately. It uses the same
historical deferred-result/cancellation/close machinery, no write lease or mutation, and
a128KiB serialized-object budget. Core and independent raw/graph/wire evidence are linked
from docs/verification-smiles-record.md; native selection/preview/editing remain missing.
The newer bounded source-backed record engine now delivers selected/all import, supported
order edits and complete-source Save/history headlessly. Finish its native worker/control
gate next. Wider CDX typed metadata/Text/Double/style mapping,
faithful rendering and source-backed editing remain required; these source/automation
foundations do not complete format or editor parity.
