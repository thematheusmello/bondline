# Native chain workflow: dependency contract

2026-11-16, base a433c3c. Previous turn is progress: the explicit fixed chain engine,
independent files/adapters, full local1563/0/1 gate and native/CLI build are committed.
Full M0–M5 remains active. This plan keeps the complete native workflow required;
an engine prerequisite does not enable either grey chain tool.

## Public evidence and complete target

[GUIDE-25 public PDF](https://chemistry.beloit.edu/classes/programs/ChemDraw_25_manual.pdf),
printed104–105, documents drag/count, atom attachment, click Add Chain, Alt repeat and
direction changes while drawing through Snaking or Control. One fixed pair of alternating
edge vectors cannot represent that changing path in one drag. Repeated authoring commands
would expose partial edits/history and defeat gesture cancellation. A literal-path
candidate/commit is therefore the first engine prerequisite below.

Printed40–42 describes active-document drawing settings and style sheets, including
fixed bond length and interior Chain Angle1..179°. Document settings must not be silently
implemented as unsaved UI preferences. The
[public CDXML DTD](https://static.chemistry.revvitycloud.com/cdxml/CDXML.dtd) declares root BondLength and
ChainAngle. Their normative numeric encoding/defaults, preservation/loss boundaries and
inheritance need a separately locked model/format increment before the full settings UI.
Do not reinterpret an unknown source setting or grant geometry authority to opaque input.

Printed105 separately qualifies Fixed Lengths,15° Fixed Angles and opposite-drag or
Control+Alt reversal; these are conditional rules, not one universal modifier toggle.
Release-mouse-before-Control belongs to that documented sequence. Printed74 Shift+x
is older version evidence, not a verified26.1/Mac shortcut. Shared-anchor count wording,
pointer thresholds/ties/sampling/backtracking, initial pose/count/length/side, repeat
lifetime/error recovery and modern/platform transitions remain unknown. Declare original
policies before tests; do not guess from memory or an installed proprietary application.

[IUPAC2008 GR3.2/4.1](https://iupac.qmul.ac.uk/drawing/drawing.html#GR32) recommends
overall horizontal chains, ordinary120° Single-chain interiors and outward branches,
with180° at Single+Triple/two-Double centers. Automatic attachment must distinguish these
cases and preserve old coordinates/styles/stereo information; a literal H budget alone
does not supply a recommended pose. The headless explicit path does not infer stereo.

The pinned [egui0.29.1 PointerMoved event](https://github.com/emilk/egui/blob/fe368bacc4a15697e347607e73a56c0299b3d42d/crates/egui/src/data/input.rs)
has no modifiers;
[egui-winit0.29.1](https://github.com/emilk/egui/blob/fe368bacc4a15697e347607e73a56c0299b3d42d/crates/egui-winit/src/lib.rs)
ModifiersChanged updates RawInput.modifiers without an ordered event. Final-frame
modifiers cannot reconstruct Control transitions between earlier pointer moves. Full
native Control behavior needs preserved ordered modifier evidence at the input boundary,
with native execution; press-latching supports only its declared narrower sequence.
Never claim complete chronological behavior from a frame-global guess.

Primary PDF/text/IUPAC hashes remain those in chain-authoring-plan.md. No proprietary
program, artwork, template, sample or binary is downloaded/inspected/executed. All future
icons are independently drawn. Actions stays OFF and both user windows remain protected.

New reviewed inputs: public DTD28821B SHA256
5311978e514ffe154108540c3634c314dc66031a4f3c877a681fc6dca8e128c2;
local egui input41217B SHA256
993cfd72386989f5155aac10f8cbd48091e9564457a7b37c3b03402a373bd3c5;
registry egui-winit source73260B SHA256
1a0ad59f3a3dd7e679a2fc16456a0cc55f985e1c70a8ffac74def9b02729a4cc.
Both SDK source URLs use crate VCS commitfe368bacc4a15697e347607e73a56c0299b3d42d;
no vendor code is changed by this prerequisite.

## Delivery order and acceptance gates

1. Deliver the literal positioned-path command below, using the existing common H,
   immutable candidate, history, ID and source rules. Preserve the fixed API exactly.
   Add same-builder read-only previews needed by native complete-candidate validation.
2. Lock and implement drawing-style model/format preservation and explicit loss guards;
   settings changes are authored commands with history and editable persistence. Separate
   them from transient tool, repeat and form state. Journal/style workflows remain M2.
3. Preserve ordered modifier transitions at the native input boundary. Test raw event
   chronology independently and verify actual platform behavior; no invented key events.
4. Lock original pose/drag count/snapping/backtracking/reversal policies against the
   public facts, then implement one effective Acyclic/Snaking mode, weak source/tool/view/
   file/close/association context, numeric Add Chain, drag counter, complete candidate
   preview, press/release commit/cancel and Alt repeat. A successful repeat context must
   capture the post-commit revision under the same Engine lock, not reuse its stale
   pre-edit stamp or rely on generation alone. No new worker pool is needed.
5. Wire menu/tool/shortcut/form actions through the registry. Respect existing File/
   Atom/SDF foreground owners, actual canvas/DPI/layer qualification, physical/logical
   real-ink and paper guards, pointer chronology, busy retirement and close/Save leases.
   One gesture creates one edit; cancel creates none. Never silently detach or merge.
6. Exercise independent App inputs at1×/2×/fractional origins and stale/ABA/modal cases,
   literal files/external consumers, actual CLI/MCP/loopback, then actual native flows:
   free/attached drawing, Add Chain, repeat, turning, settings, undo/redo, Unicode Save/
   reopen and recovery. Inspect real screenshots; chemical goldens need human approval.
   Keep exact26.1, measured DPI and unexecuted platforms distinct from evidence.

These are dependencies of the full requested flow, not substitutes for it. Both tools
remain disabled until their corresponding native workflows are implemented and checked.

## Locked first increment: chain.create_path

Add one new closed registry command with public ChainCreatePath:
expected_revision:u64, positive page_id:u32, required
hydrogen_policy:"rectify_neutral_carbon", placement:ChainPathPlacement.
Keep ChainCreate, ChainPlacement, ChainSide, Command::ChainCreate, their wire/schema
and all existing field/default semantics unchanged. Registry39→40; native controls and
catalogue12 delivered/128 planned remain unchanged.

Placement is exactly one closed tagged object:

- free: mode:"free", points:2..64 point objects, in path order;
- atom: mode:"atom", positive fragment_id/atom_id, new_points:1..63 point objects.
  The start is exactly the stored target position and is not repeated in new_points.

Each point is public ChainPathPoint{x_pt:f64,y_pt:f64}, finite and within±100000 pt.
JSON points are closed objects, not positional arrays. The request type supplies the
command schema without adding a schemars dependency or changing serde behavior in L0
model Point2. Convert through its existing validator. Required root/branch fields reject
null, unknown and irrelevant selectors; policy remains string-only for this new request.
Typed Rust coordinates also validate before publication. No optional angle/count/side,
inferred pose, reordering, snapping, rescaling, closure or chemical normalization.

Resolved count includes the anchor. Literal paths may have unequal edge lengths,
mixed turn signs, collinear180° interiors, tiny represented edges and crossings. They
must not inherit the fixed command's equal-edge/1..179° certificate. Preserve every
supplied finite coordinate exactly. All resolved vertices must be distinct; consecutive
edge hypot must be finite/nonzero. Check every new center against all old same-page
centers; no joining an occupied endpoint. Crossing/ink/paper/renderability are later
native candidate guards, not a new headless geometry restriction.

Source-free Drawing or completely mapped native CDXML alone has membership authority.
Use the same target-local positioned neutral/natural/nonradical carbon proof, weighted
Single/Double/Triple incidence and sufficient exact literal H as chain.create. Consume
one anchor H; append only the new atoms and directed Singles. Preserve all old fields,
arrays, IDs, coordinates, directions/styles and unrelated owners. New C has H3 at free
termini/last point, H2 internally and both false carbon-label flags. Isolated C2 still
has mandatory labels; metadata does not authorize lossy MOL/retained-source export.

Error precedence: ordinary parse/committed-close/stale-revision guards; then source,
placement count/policy, page, attachment/H/position, point validation, duplicate/edge
geometry, old-center occupancy, checked IDs/global model validation, normal commit.
Use existing InvalidChain/UnsupportedChain and model/target/ID errors. One local maxima
copy is published only after success; attachment spends no fragment ID. Errors preserve
source/current/saved/history/selection/file/context/floors. Undo/Redo restore exact
whole content and floors remain monotone. Pending close/file contexts follow existing
invalidation, not a new global mutation ban. Immediate prepare wins before late cancel.

Engine::preview_chain_create(&ChainCreate) and preview_chain_path(&ChainCreatePath)
return the exact same builders' StoredDocument candidates without changing source
owners, epochs, view, selection, history, floors or Save authority. They require current
revision and reject committed close, but do not require a native tool/host. Later native
context qualification is additional. Preview is not a second editing implementation.

First independent existing-JSON test must fail UnknownCommand before producer. Literal
five-vertex mixed-turn witness [(144,144),(192,144),(192,192),(240,192),(240,144)]
expects C5H12/H[3,2,2,2,3], preserved coordinates, directed path edges and one edit.
Add source/metadata/history/floors, malformed/bounded/precision/occupancy, preview and
actual Unicode adapter/file checks. Raw exact flags/orders/styles precede unchanged
external chemistry. Keep old API/control consumers and all previous geometry assertions.

The first path/preview prerequisite now has [receiving evidence](../verification-chain-paths.md).
This does not complete steps2–6 or enable either native chain tool.

Current dependency checkpoint2026-11-17: step2 now has authored data/format preservation
and the atomic settings command, with the native form/default application still missing.
Step3 has [ordered transport evidence](../verification-ordered-native-modifiers.md):
six synthetic SDK cases and bounded actual Mac OS input, preserving old gesture semantics.
This does not implement a chain consumer. Continue steps4–6 and the settings form/application;
both chain controls stay grey until their corresponding complete workflows are checked.
