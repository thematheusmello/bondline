# Bondline agent instructions

## Development Actions policy — user instruction, 2026-11-14

GitHub Actions is disabled during development to avoid hosted-run charges. Repository
Actions is off and the Rust foundation workflow is disabled manually. Do not enable,
dispatch or rerun workflows, including for a passing feature, milestone, branch or docs
checkpoint. Re-enable only when the full project is complete or the user explicitly
changes this instruction. Preserve the workflow definitions for that later verification.

Continue local rustfmt, warnings-denied clippy, workspace tests, xtask gates and bounded
fuzzing. Local passing gates govern development integration; do not wait for or manufacture
hosted evidence while Actions is disabled. Mark new Windows/Linux execution unverified
when it has not actually run. This policy supersedes older hosted-before-main instructions
and historical pending-hosted notes in project records. Do not restore automatic runs to
make an old CI badge green.

## Goal and orientation

Implement an independent offline Rust chemical structure and reaction editor, native on
macOS, Windows and Linux, eventually also WebAssembly. Original code is MIT OR Apache-2.0.
The reference is Professional26.1 behavior from public documentation and explicitly
authorized visible runtime observation, not proprietary code.
Read STATUS.md, ROADMAP.md, docs/architecture.md, docs/parity-checklist.md and
docs/clean-room/sources.md before selecting work. Continue existing work; do not restart.
The persistent project goal is active. M0's native license blocker is resolved; this does
not complete M0. See docs/control-protocol.md for the delivered command/adapter contract and
docs/verification.md for exact RED/GREEN and platform evidence.

## Research, provenance and implementation

- Target one-to-one Professional desktop screens, control roles, icon roles, workflows and
  functionality, with Bondline branding and the existing offline/open-source commitments.
  Use public documentation, screenshots, videos, open standards and licensed source references.
  The user's latest2026-11-15 instruction explicitly authorizes opening installed ChemDraw
  to inspect its visible screens and workflows; this supersedes the earlier no-execution
  boundary for that UI observation. Keep public and local runtime provenance distinct.
- Use CUA for the authorized installed-app UI interaction. Never download or statically
  inspect, decompile or disassemble ChemDraw program binaries in this workflow.
  Never copy proprietary templates, sample libraries, icons, fonts or help text.
- Use ordinary source tools for open-source repositories.
  Record exact source/build/version/platform, URLs or artifact hashes, video timestamps,
  observations, inferences and unknowns. A screenshot or static string alone proves no flow.
- Obtain documentation and open-source research artifacts from public sources. Record
  actual access blockers and continue independent public research. Keep reference downloads,
  screenshots, installers and REA reports in ignored .cache/ or target/, never shipped by default.
- Implement original Rust logic and original redistributable assets matching documented roles
  and appearance. Do not transplant proprietary source, decompiled code, fonts, templates,
  sample libraries, help prose or icon files into Bondline without applicable redistribution
  rights and the asset/dependency policy. Reference caching is distinct from product reuse.
- Never use proprietary product marks in Bondline's UI or executable names. README may
  identify compatibility and must state non-affiliation. Parity/source records may name references.
- Shipped dependencies and bundled assets must permit use under MIT, Apache-2.0 or BSD.
  Do not broaden this allowlist or change license labels to pass a check. Preserve notices.
  Build-only tools are distinct from shipped runtime dependencies. GPL/LGPL/AGPL software
  may be run only as external test programs, never linked or copied into the shipped application.
- Record sources and behavior decisions in docs/clean-room/sources.md (historical path).
  Unresolved behavior goes in docs/clean-room/open-questions.md; do not guess from memory.
  Keep historical evidence attributed to its actual method; new work follows this boundary.

## Architecture

The intended crate graph is model/cdxml/cdx -> chem/render -> io -> commands -> mcp -> cli/ui.
Packages use bondline- prefixes. cdxml and cdx have no workspace dependencies; formats
preserve unknown data and must never silently drop it. Native I/O stays out of model/parsers.
The UI, CLI, loopback channel and MCP call one typed JSON command registry.
StoredContent distinguishes Drawing(StoredDocument), complete Sdf{original,current},
and immutable complete SmilesFile(SmilesFileDocument);
Engine current/saved/history/file plans retain that whole variant. Drawing JSON is unchanged;
SDF has document:null plus collection metadata; SMILES files have a compact count/size
summary. Neither substitutes a first/empty drawing. document() and
stored_document() return typed Results; preserve wrong-kind refusals and clear stale UI data.
StoredDocument owns drawing source/model/baseline. Keep that backing
when wiring import; a cloned elemental projection alone loses unknown data. Require failed-
import/source-retention tests before exposing imports. Retained payload/container accounting
now enforces 128 prior snapshots and a 128 MiB threshold, deduplicating shared source/model/
fragment allocations and including baselines/reserved vectors. Keep current data even when
it alone exceeds the threshold; this excludes allocator headers, caches and external copies.
Update accounting/tests whenever a new model field owns heap data; do not claim a hard RSS cap.
Unchanged-source export is guarded; never write stale XML after a model edit. StoredDocument::
to_cdxml borrows unchanged source or encodes new native elemental data. Do not bypass it by
regenerating stored.model(): that projection omits unknown data. A generated profile is fully
mapped only if re-encoding reproduces every input byte. Its cached classification now permits
headless editing/rendering; unknown/extended/noncanonical files remain guarded and preserved.
Keep the pinned v0 fixture editable when changing the encoder/version; add migrations instead
of refreshing it. Native file wiring now uses those complete snapshots. Poster Width/Height
are drawing extents, not inferred paper/printer settings.
xtask checks layers and licenses, fetches a pinned SHA-256 corpus, and orchestrates CI.
No GUI dependencies below ui; wgpu belongs in render. Add crates when useful behavior lands,
not empty successful-looking API stubs. Document any isolated OS FFI exception explicitly.
Loopback connections require the fresh session token emitted only in opt-in startup stdout.
Never commit or expose actual tokens in logs, screenshots or verification records. Stdio MCP
is separate and opens no listener. Keep file access bounded and explicitly authorized.

Current map: model (L0, pages, Point2 and validated elemental atom/bond/fragment graphs);
cdxml (L0, source-preserving syntax, integral/source-ID decoding, checked attribute
replacement and a bounded new-syntax writer);
cdx (L0, bounded flat binary syntax, exact source passthrough and checked opaque property
replacement and new-tree Writer; model mapping and application commands live above this crate);
render (L1, shared neutral-element Single/centered Double/Triple GraphScene, bounded hit regions, literal authored-H
labels plus a separate resolved-H entry with attachment/paper-ink guards, CPU/GPU
rectangles/strokes and unchanged approved golden); io (L2,
source-backed CDXML/MOL and limited CDX projections, elemental encoding, owned snapshots, PNG encoding,
bounded native CDXML/CDX/MOL/SDF/SMILES filesystem libraries and source-preserving
SDF/RXN/SMILES file records;
bounded SMILES notation/source ownership including literal aromatic markers/bonds,
opt-in fixed-H normalization and
restricted noncanonical Fragment export and validated1–12carbon/Single path layout into
a new pure Document, separate from complete SMILES source-file sessions or general chemistry;
see docs/native-file-io.md for publication/access limits);
commands (L3, registry, complete stored-snapshot Arc history and transient close policy); mcp (L4, stdio/native loopback, registered work and reply publication barriers); cli/ui (L5,
headless and native adapters). Other planned crates do not exist yet. `compat/libloading`
and `compat/clipboard-win` opt into reviewed OS unsafe; `compat/hexf-parse` forbids
unsafe. These three original compatibility packages retain upstream package names solely
for Cargo patch routing. The eframe/egui/winit source forks and Apache font retain upstream notices.
The pinned egui0.29.1 current-focus filter guard fix retains its exact published version
and MIT/Apache notices; see vendor/egui/BONDLINE-PATCH.md and SDK/UI regressions.
The pinned winit macOS application-quit callback is the other reviewed OS-boundary exception:
two standard main-thread AppKit visibility actions, with exact source/pin/native evidence in
vendor/winit/BONDLINE-PATCH.md. No unsafe enters Bondline's UI/model/command/format crates.
The pinned accesskit_macos string-value bridge is another reviewed native boundary; its
exact source/notices, nullable NSString checks, bounds and tests are in that vendor's
BONDLINE-PATCH.md. cargo xtask ci includes its native tests/clippy on macOS from the root lock.

`ui.view` is transient camera state with a checked epoch, shared by the delivered adapters.
Default Fit retains legacy pixels; explicit100% is one logical unit/authored point, not
physical monitor calibration. Use Engine::page_view/drawing_view_scene for rendering and
picking together, retaining source-qualified MOL display-H. Camera never changes paper,
source, revision/history/dirty or CaptureStamp. Open retains camera; projection and later
navigation use a bounded displayed center. Whole-source render guards remain mandatory.
Optional current-view capture and read-only SDF Actual/Zoom/pan are delivered below.
Saved preferences and complete reference behavior remain missing; see docs/control-protocol.md.

SDF Fit's native readout is textual `Fit`, independent of cached bounds or control
enablement; it has no fabricated numeric AX value. Preserve explicit100 Enter/blur
commits even when a numeric widget's internal seed was100. Known Drawing Fit stays
numeric. This presentation fix does not reconcile legacy physical-margin Fit with the
declared logical basis of relative steps; see docs/verification-fit-readout.md.

## Test first, then implement

For each behavior: write one test using independent expectations, observe the intended
RED failure, implement minimally, observe GREEN, then refactor. Record commands/results
in docs/verification.md. A round trip through only our own reader and writer is insufficient.
Use independent parsers, published reference vectors, externally run chemistry tools,
analytic cases, approved goldens and property tests as appropriate.
Documentation/configuration-only work needs relevant checks, not artificial tests.
Keep invalid input, resource bounds, error handling, accessibility and data preservation.
No production panic/unwrap/expect/todo/unimplemented and no unsafe outside an explicitly
reviewed OS boundary. Borrow where practical; use typed Result errors in libraries.

To add a command: first add its independent behavioral/error/history tests, register its
typed closed parameter object once in command_registry!, implement it below the UI, wire
delivered adapters, update CLI/MCP wire tests as needed, then update provenance and parity.
For UI changes, launch the actual native app and inspect the workflow. Eguis's input tests
and source cross-checks do not substitute for native platform execution.

Every native save must use the shared FilePlan/Completed reservation. One outstanding Save
per Engine is allowed from prepare through finish/drop, including after publication. Never
release it at the worker/publication boundary or replace it with an adapter-only guard:
reversed completions reproduced a clean wrong encoding and a later Auto precision downgrade.
Preserve busy-before-I/O, release on errors/cancellation/drop, independent Engines and
truthful old-generation/committed receipts. Successful receipts must be finished, not discarded.

MOLV2000/V3000 now have source-preserving codecs, owned snapshots, atomic I/O and shared commands.
Its transform is explicit, headers/raw source stay authoritative, and unsupported projection
must fail rather than erase data. Associated file.format is independent of source.format and
filename extension. Open defaults24pt/unit and origin(72,72)pt; source-backed Save keeps its
captured frame/headers. Multiple fragments, changed page layout and source metadata loss
remain refused. Do not call this complete M1 support. Pinned MOL fixtures have -text Git
attributes: preserve exact hashes on Windows, never normalize them in an oracle/parser.
Declared valence yields exact authored H by the published finite arithmetic; do not infer
periodic-table valence. Preserve the documented normative M ISO/RDKit discrepancy, and the
original/independent test vectors. The corpus now actually checks one MOL input, not only
its hash. An original explicit-root CDX scene has fetched byte/struct/RDKit checks;
foreign/general/application CDX and Bondline chemistry calculations remain missing.

Open selects the actual MOL header version and fully validates that codec; no suffix-based
version fallback. Save's optional version is auto/v2000/v3000, captured during prepare.
Auto keeps the associated MOL version, else original MOL version, else V2000. Source info
still says mol; file state/receipts say mol_v2000/mol_v3000. Original V3000→V2000 refuses;
V2000→V3000 also requires strict raw-coordinate preservation, never the V2000 rounding
allowance. The chooser has one MOL Open family and two concrete Save encodings. Preserve
same-path version-change invalidation, exact failed retry and trusted close-Save receipts.
Native/hosted evidence must be separate from engine/egui tests; see
docs/verification-v3000-application.md. No runtime dependency or codec source changed here.

SDF is a mixed V2000/V3000 source container and complete application-content variant.
Shared Open/Save retain every record. sdf.record.inspect exposes bounded ordered fields;
sdf.field.set_value edits one existing occurrence through immutable deferred candidates.
The restricted render.capture SDF virtual page1 now uses all-record derived display data;
the native read-only preview has bounded Mac Actual/pan acceptance; native wheel/DPI/
edit-preview workflows and record insertion/reordering remain missing. Whole-record removal
has the later bounded native evidence below. The existing-value editor uses the same
registered setter: temporary drafts never change source/dirty/history before Apply. Retain
Apply/Discard/Cancel, event-indexed Apply→Save ordering, stale-source recovery, guarded
semantic continuations and actual worker retirement. A committed edit is never discarded
as a cancelled query; report its true outcome and retain detached recovery text. The
bounded TextBuffer rolls back rejected widget transactions and resets only its own bounded
text undo state. Native Mac edit/save/history/reopen has finite evidence; OS draft/close
races and interactive Windows/Linux remain unverified. See
docs/verification-sdf-editor-integration.md. Preserve off-lock
work, exact no-op sharing, stale revision/generation/Arc checks, saved/original distinctions
and the same worker/reply/close barriers. Explicit data results are separate from automatic
metadata-only state/Debug; see control-protocol.md for input/output budgets.
Its generic writer preserves actual molecular versions; write_v2000 must refuse V3000. Preserve exact
record/MOL/item spans, duplicate order, raw headers/DT digits, empty/multiline values and
opaque embedded MOL bytes. Never rebuild these from RDKit's last-value property map: its
full supplier loses records/data at the original valid DT-only vector. Independent raw
checks and per-record molecular consumers have different jobs. The writer must refuse
reserved delimiter prefixes in embedded MOL blocks before output, not normalize the source.
Strict EOF/header/ASCII bounds and the documented post-item empty-padding accommodation
must stay explicit. See file-formats.md. Corpus now has five mandatory pinned source bundles, including original SMILES vectors;
validate all licenses/notices/global cache names before cache/network I/O. The original
SDF fixture is fetched from its actual public source commit; never invent a future pin or
rewrite existing RDKit metadata. Cache ancestry must be confined before directory creation.

RXN is a standalone legacy container. Keep wrapper/MOL versions distinct, ordered
reactant/product/reagent counts, complete molecular payloads/maps/unknown data and full EOF.
Do not flatten it into an elemental fragment or call the library a reaction editor.
Mixed molecular versions and omitted zero-reagent counts are observed consumer extensions;
V3000 wrapper AGENT versus published REAGENT remains an explicit unresolved conflict.

The current30-command registry includes document.new, complete SDF/SMILES-file Open/Save and bond.create
for independent neutral-carbon fragments with optional Single(default)/Double/Triple order
and literal H=`4-order_weight`, plus CDXML/CDX/MOL open/save. bond.extend requires the
explicit H policy/revision and one incident supported bond with consistent literal
H=`4-existing_weight`; the requested weight must fit that budget. Only source-free or
byte-proven native CDXML is allowed. Actual MOL/CDX backing
refuses. Native attachment is wired with ordered fresh hit/anchor/release/context tests;
actual native verification is separate. Document::replace_fragment atomically retains
fragment/old IDs and unrelated Arcs; global validation excludes only the old fragment. Save complete StoredDocument data; file.dirty uses the saved snapshot, while
source.modified describes original backing. Prepare/complete/finish separates I/O from
revision-checked apply. Count current+saved snapshots and never call a publication error rollback.
Identity floors must survive Undo branches and advance only after successful commits.
Native gestures consume ordered events; press_origin alone loses a drag completed between
frames. File menu/shortcuts also dispatch at their raw event, resolving the current central
binding after preceding edits/history. Never restore an eager File shortcut prepass: it
saved an un-undone Double after rapid Undo→Save. Preserve same-packet actual-file and
separated-input controls; pending intents must block later authoring while existing
Running/worker/receipt policies remain exact. Canvas interaction must not advance layout beyond visible error text.
Use Engine::controls() / ui.controls as the authoritative delivered File/Edit/Page bindings;
do not duplicate labels, shortcut tables, preset parameters or enabled rules in the UI.
ui.file is presentation intent, distinct from actual document.open_cdxml/save_cdxml or MOL I/O.
Trusted file plans bind ui_request_id; ui_request_current is captured before their own save
changes state. This returned marker is historical. Native close-Save also requires the
current Engine finish_ui_file success acknowledgment; later format/baseline changes clear
it permanently even after values return. Never acknowledge uncompleted work or release
close from an old worker result. Keep both receipt markers and format/revision/path/epoch
checks. A direct save never fabricates trusted completion state.
Headless engines have no host. Keep request IDs and context guards through native selection,
worker preparation and retry. Reserve/prepare under one engine lock before spawning; only
the matching current request may restore a failed picker. A host acknowledgement does not
prove visible pixels or saved bytes. Keep unaddressed dialog fields/job controls explicit.
Close intent/epoch never belongs in authored history or serialized CDXML. Request/Discard/
Cancel must use document.close; successful actions advance close_epoch without changing
authored revision. A close-specific Save must retain its captured epoch/revision and matching
current receipt; Cancel/new Request must not reuse it. Native complete_close is a trusted
host acknowledgement under the shared Engine mutex, after executor activity, UI file jobs
and pending file actions are absent. Register deferred work/replies under that same mutex;
release response activity after write/flush. Never restore UI-private close approval or let
an accepted file action start after Committed. Headless close intent does not kill stdio.
The user approved crates/render/tests/goldens/single-bond.png for its exact scene/hash only.
Its JSON and README record provenance; external Pillow compares every CPU/GPU RGBA byte.
Do not regenerate that golden without renewed human approval. Other images remain unapproved.
requirements-dev.txt now pins external RDKit/NumPy as well as Pillow. Select an isolated
interpreter with BONDLINE_PYTHON as described in docs/development.md; missing consumers fail
instead of skipping. The RDKit test verifies engine-created CDXML interoperability for
ethane only, not an implemented chemistry engine or complete differential corpus coverage.

## Pick and finish the next task

Take the highest-priority unblocked item in ROADMAP.md and the parity checklist. Read its
source, callers, tests, fixtures, config and exports. Implement the smallest complete slice
through the shared registry and delivered adapters. Check meaningful success/error cases.
Track missing, wired and verified separately; wired is not evidence of correct behavior.
Never call all M0 complete while desktop/GPU, cross-platform or inventory evidence is missing.
Do not spawn subagents unless the user or other applicable instructions explicitly authorize it.

## Commands and commits

When the workspace exists: cargo fmt --all -- --check;
cargo clippy --workspace --all-targets --locked -- -D warnings;
cargo test --workspace --locked; cargo xtask layers; cargo xtask licenses;
cargo xtask corpus; cargo xtask ci. Until implemented, a documented future command is not a pass.
Keep Cargo target directories separate for each worktree and staged-tree export. Shared artifacts caused a real
wrong-checkout CI run and stale metadata; runtime workspace discovery does not make shared
binary outputs safe. xtask locates the invoked workspace through Cargo, and ci prints its
root. Verify that root before counting results. Use local Git commits. Small conventional
commits: one command or format feature, with its
tests, provenance and relevant docs. Keep main green; record red evidence without committing
failing main. Commit all intentional project files, never caches, secrets or build outputs.
Do not push or publish without an explicit request. Update STATUS.md with verified state,
actual failures, remaining work, next executable steps and live job handles before stopping.
Currently implemented xtask gates: layers, licenses, corpus and ci. Corpus checks the delivered
CDXML syntax/elemental projection, MOL, ordered SDF and one original explicit CDX scene,
plus pinned file integrity; its output identifies missing foreign/general/application CDX,
chemistry and broader format coverage. New source needs its own hosted evidence; see STATUS.md.
Hosted native-window smoke passes on all three platforms, plus Linux ASan, after the
Linux runtime-package correction. These are window/API checks, not interactive Windows/
Linux chemical-workflow evidence; see docs/ci.md.
Development bundles
and REA reports under target/ are disposable build outputs and must not be committed.
Parser fuzzing is isolated under fuzz/; read its README for pinned toolchain/tool versions,
seed setup and bounded runs. Do not mutate committed fixture directories as fuzz corpora.

## New CDX syntax

CDX Writer creates only new canonical explicit-root documents with ID0 and zero reserved
header bytes. Its raw property payloads and object sequence do not validate chemistry,
property types, references/unique IDs or object containment. Never use it to regenerate an
imported source projection; Document::to_bytes and checked property replacement retain their
separate exact-source contract. Include pending terminators in byte/record/depth budgets
before mutation. Independent original bytes/RDKit, literal boundary tests and fuzz construction
cover this builder; they do not deliver application CDX files.

## CDX header-embedded framing

Default parsing preserves successful28-byte legacy interpretations. Only legacy TrailingData
and an8000 tag at byte22 may select HeaderDocument's22-byte prefix, encoded root and exactly
one distinct FileEnd. Do not turn this into arbitrary trailing-zero acceptance, scanning or
normalization. FileEnd counts toward the record budget and the encoded root toward depth.
header() is22 bytes for that profile; property offsets/ownership use its actual length.
Writer's canonical28-byte profile and all old fixtures/pins stay unchanged. Full-byte source
retention/property edits and original independent/native probes are separate from model/UI
compatibility. A complete legacy tree is inherently indistinguishable from a native file
missing only its file end; keep this ambiguity explicit. Fuzz construction omits root and
FileEnd intentionally to build new canonical syntax, never to reserialize an import.

## Limited CDX model backing

IO's ElementalGraph retains complete borrowed source while mapping only its closed initial
Element/default-C, Single/default-order, fixed16.16-point subset. Its fresh IDs are not
container-scoped source IDs. Selected Z/bounds/CIP0U-or1N/opaque AtomID/unstyled bond-warning
fields now remain source-only with strict widths/owners. FragmentZ is a qualified public
compatibility case. AtomID is never a reference/app-ID alias; its normative uniqueness is
unknown. Other standard metadata refuses; no stereo/H/charge perception or general compatibility.
StoredDocument::from_cdx owns raw Arc plus baseline; is_fully_mapped is always false here.
Export only exact unchanged backing. Changed models, source-free CDX encoding and cross-
format metadata loss must fail; do not regenerate a projection with syntax Writer.
Snapshot equality/accounting includes opaque bytes and baseline. Preview paper is caller
policy, not inferred from BoundingBox. CDX filesystem functions reuse the existing bounded
read/atomic save path. document.open_cdx/save_cdx and the native chooser are now wired
through the shared registry; see verification-cdx-application.md for exact local versus
native evidence. Never bypass shared FilePlan/Completed reservations in those adapters.
Normal CDX fuzzing now exercises projection/snapshot guards; both lock edges add only the
existing internal CDX crate. New native declared closures154/165/220 include that crate.

## Neutral graph display and transient hover

The literal graph_page_scene entry and source-aware captures share a bounded GraphScene.
Positioned neutral, nonisotopic/RadicalNone elements and Single/centered Double/Triple
bonds are supported under the original3w profile. Other orders remain refused. In the literal
entry, missing H is never inferred:
connected unknown-H carbon may be skeletal, isolated unknown-H carbon refuses, and other
elements retain literal symbols without inventing H. Authored0/1/N H renders literally;
physical H cannot carry positive attached H. Unsupported metadata/orders/geometry refuse.
Element::from_symbol/symbol owns the verbatim IUPAC catalog; IO delegates without new
heap fields. Source completeness/export guards remain independent and strict.
Atom/label priority, visible clipped ink, five logical canvas units and stable-ID ties are
original policies, not full selection or reference-product default claims. Hover state is
UI-only and excluded from Scene/history/source: revision/Escape/focus/context invalidation
stays clear until a fresh ordered positional event. Context includes available origin,
public native origin and effective DPI. Keep multi-idle-frame regressions and native checks;
do not rearm from cached hover_pos, key release or focus gain alone. No OS pointer polling.

For multiples, preserve one logical bond ID/hit/neighbour vote while checking all actual
components and gaps. `centered_bond_strokes` is the sole geometry constructor for graphs
and held previews; `GraphScene::bond_strokes` supplies hover components. Keep exact all-
Single pixels, fundamental stroke coverage arithmetic, approved golden and independent
visibility/exclusion/cap/attachment/budget checks. The original short/corridor/junction
policies are not a vendor default. Literal multiple scenes use exact authored counts via
component-aware attachment; resolved-only paper-ink policy remains distinct. Unknown file
styles stay preserved and do not gain edit or faithful-render authority.

`ui.bond_tool` owns a transient checked order/epoch outside document/source/history/dirty
state. Direct bond commands do not inherit it. Gestures/hover include both order and epoch;
A→B→A cannot revive an earlier gesture. Preserve raw event indices before shortcut
consumption, order tool activations and replay their state at the correct event boundary.
Never relabel a press retroactively or choose activation order by palette iteration.
Keep H-adjustment guidance visible before commitment, including error frames. Loopback
must notify real transient changes despite outer changed=false; no-op selection does not
notify. See verification-multiple-authoring.md, verification-multiple-ui.md and the shared
control contract. New multiple native drawing acceptance remains unverified. The refreshed
workspace's native visual review/Double selection is separate evidence; see STATUS.md and
docs/verification-workspace-presentation.md before any further desktop interaction.

Neutral-label verification includes independent metric vectors and one uniform4xBOX
FreeType reference at the same96em/0.90 threshold. Keep all80 positives/controls, failed
native96 measurements and bounded8x consistency qualifications. Do not adjust production,
font or the old carbon oracle/golden to fit a new test. See verification-neutral-element-labels.md;
native/newWindows/Linux label workflows remain unverified.


## Owned whole-SDF prerequisite

io::sdf::SdfDocument owns validated complete source and compact immutable record metadata.
Clone shares bytes/index, equality includes every byte and Debug omits content. IDs are
local positive ordinals, not persisted/source chemical IDs or guaranteed identities across
other sources/future edits. read() reparses under existing limits; retained_bytes excludes
borrowed views/allocator headers. StoredContent supplies the separate Engine integration. Preserve every
record/field/MOL/version and never pass a first-record model to a file Save. The normal
SDF fuzz target now checks owned retention/metadata/sharing/accounting as well as old spans.


SdfDocument::with_field_value returns an immutable candidate for an existing ordinal
record/zero-based field occurrence; never find duplicate fields by a property map/name.
Logical no-op shares old bytes/index; authored changed lines use LF, preserving original
terminator/header/DT/padding/other records/MOL bytes. Typed missing targets use not_found;
existing parser/writer error codes and quotas remain. It is not an Engine edit command;
Open/Save already retain whole current/saved/history source. Future record operations
must not make ordinal IDs promises across record deletion/reorder without an identity design.


## SMILES source markers

Aromatic atom/bond markers are notation, not aromaticity or valence perception. The named
fixed-H normalizer must reject any aromatic atom OR bond before integer counts; uppercase
C:C has false atom markers and an explicit Aromatic bond. Keep the writer/model guard intact.
Match the notation BondOrder exhaustively in corpus/fuzz. Ring explicit tokens live in
RingClosure.open_bond_range/close_bond_range even when Bond.bond_range is None; endpoint
aromatic defaults apply only when every applicable explicit token is absent. Original
explicit-ring regression seeds protect this distinction. Fetched corpus remains three
original nonaromatic vectors; six committed fuzz seeds are a different dataset. Never
repin old manifests or drop Root's additive seed/consumer tests when receiving codec work.


The named carbon-path layout takes explicit bond length/origin/page, preserves the input
notation/IR and local normalized identities/H, and returns a new pure drawing. Its1024square
reference-scene ink/geometry checks do not prove every viewport or native UI. Keep source-
backed SMILES Open/Save separate from explicit conversion into a new native CDXML file.
Do not attach an original SMILES path to a CDXML association, overwrite the input or grant
StoredDocument source authority to a raw projection. See verification-smiles-layout.md.

file.convert_smiles_to_cdxml is create-only, not an active-session Save. Its eight required
closed parameters select the neutral_carbon_path_v1 fixed-H profile and explicit geometry.
Keep the same save reservation through prepare/publication/finish/drop and the existing
worker/reply/close barriers. Finish returns current state plus historical conversion metadata;
do not alter current/saved/original/history/association/close/native intent or acknowledgments.
MCP read_only is false because it writes disk. Conversion/nonfile work cannot bind native
Open/Save requests. Complete-file SMILES Open/Save is a separate immutable content path.
The later specialized native form does not change those generic binder refusals; see
the native conversion continuation below and verification-native-smiles-conversion.md.
See verification-smiles-conversion-command.md for scope, RED/GREEN and independent consumers.

file.export_smiles_fragment requires expected_revision/page_id/fragment_id/output_path.
Require complete Drawing source and the exact explicit page/fragment before I/O; never
choose the first fragment, infer H or let an omission receipt bypass source authority.
Keep the whole StoredContent owner and shared save lease through Completed/finish/drop.
It exports the unchanged writer's full supported graph, including isotopes/physical H,
Single/Double/Triple, branches/rings/dots, without renderer or carbon-path restrictions.
One LF frames the file record; notation remains capped65536, framed bound65537, and raw
parser/converter input remains capped65536 (maximum re-import limitation is explicit).
smiles_export is a historical metadata receipt, omitted from ordinary state; geometry
and document_context categories describe exclusions, never permission to lose chemistry.
Current/saved/source/history/association/close/nativeACK stay unchanged, with read_only=false
and existing worker/reply barriers. No native control or SMILES backing follows. See
verification-smiles-export-command.md; Root's isotope-keyed actual-file test must retain
all five semantic controls, including equal-count/formula endpoint/order decoys.

## Complete SMILES files

`smiles::SmilesFileDocument` owns complete source plus a compact shared record index; every
meaningful notation is validated before success. Its separate file budgets and exact
LF/CRLF/opaque-data/prefix/final-EOF policies are in verification-smiles-file.md. Do not
weaken the unchanged single-notation parser/converter limits, normalize names, discard
ignored lines or substitute the first/empty graph. Ordinals are source-local, not stable
chemical identities. Keep the independent raw-byte and unchanged per-record consumers.

`StoredContent::SmilesFile` stores one immutable owner because no file edit API exists.
Current/saved/plans retain the complete variant; retained accounting deduplicates source
and index. `document.open_smiles` and `document.save_smiles` reuse the common guarded
Open/Save path, lease, generation, publication and native request/close rules. Automatic
state has document:null and compact smiles_file {format,byte_length,record_count}, without
notations/data or a recordsVec. Graph/SDF edits and cross-format Save refuse this kind;
document.new explicitly starts a distinct source-free session. The .smi/.smiles chooser,
original inspector and Unicode Save/reopen have bounded native evidence. No chemical
preview/editing, metadata editor or source-to-drawing authority follows. See
verification-smiles-file-app.md and verification-smiles-file-ui.md. Run the separate
smiles_file_parse fuzz target locally as in fuzz/README.md; Actions stays disabled.

## Explicit SMILES record inspection

`smiles.record.inspect` requires current expected_revision and an explicit positive
source-local record_id. It captures the complete immutable StoredContent Arc; selected-only
parsing/JSON measurement runs off-lock. The12-field smiles_record object returns exact
notation plus file-relative ranges/counts, with null distinct from existing empty data.
Opaque application bytes are never copied/decoded/disclosed. Automatic state omits the
result, and Debug omits notation. The128KiB cap applies to the serialized object, not
parent/wire envelopes; keep the real bounded serializer guard and old notation limits.

Use existing SDF historical snapshot/close/UI refresh semantics. Both Prepared deferred
flags must include this plan so cancellation survives finish. A newer Open is not applied
backwards; input_revision/session_generation identify the source. Retain the whole owner
through work, then release the extra source from the self-contained completed reply.
Queries reserve no write slot, bind no native file request and acknowledge no Save/close.
Keep the independent raw/graph/maximum notation tests and actual adapter/no-repaint checks.
This is original automation policy, not a native record browser/editor. See
verification-smiles-record.md and its linked receiving records; Actions stays disabled.

Native SMILES queries now prepare through typed `CommandExecutor::prepare_ui_smiles`
and opaque `UiSmilesWork`, with Open/full-stamp/exact-close guards and the existing
shared two-worker reservation before spawn. Do not bypass it by parsing in-frame or
spawning before validation. Completion can be historical; the host still owns stale
display rejection and actual worker retirement. See docs/verification-smiles-ui-work.md.

`SmilesRecordsWorkflow` now uses that bridge in the actual App. Preserve its one weak
source/close/revision/ordinal/sequence request, pending intent, actual thread and bounded
reply; clear text/focus on source/close/hide and never install stale/historical results.
Parsing stays off-frame; file/close gates include publication through real retirement,
including poisoned-engine cleanup. Immutable notation stays selectable/copyable with
ReadOnly AX and cleared local undo. No opaque appdata or inferred chemistry is displayed.
Five independent App cases and two finite lifecycle gates pass. Bounded native ordinal
typing/Inspect/Previous, readonly typing refusal and Unicode Save/reopen now preserve all49
bytes; native OS clipboard, lifecycle races and other platforms remain unverified.
See docs/verification-smiles-record-integration.md; do not duplicate completed work.

## Derived SDF display continuation

`io::sdf::preview` returns an ephemeral Page, exact preview-to-local ID maps and separate
display-H/provenance maps. Authored Some/None and all source bytes stay unchanged. Unknown
H derives only in the documented neutral C/N/O/halogen profile; literal counts are retained.
The new render entry requires complete counts, validates actual paper ink and chooses
faithful H placement without moving chemical anchors. The literal entry/golden is unchanged.
The existing render.capture command uses complete deferred source ownership and virtual
page1; preview never becomes a drawing, history entry or Save/export authority. All-record
projection/scene failures refuse the whole capture. See verification-sdf-preview-model.md,
verification-sdf-display-render.md and verification-sdf-capture-command.md under docs/.

Native preview now reuses the registered capture through UiCaptureWork, reserved under the
Engine mutex before spawning. The UI retains a single job through off-thread bounded PNG/
ColorImage decode, channel publication and actual thread retirement. CaptureStamp has opaque
Weak-content/revision/generation identity, never Serialize or a source lease. UI keys also
include close_epoch, view epoch, actual logical rectangle, physical dimensions and effective DPI; changed
keys clear old textures/errors/hits immediately. The renderer supplies main-label em from
actual labels; it is Rust-only metadata omitted from PngCapture JSON. Logical readability
must use actual destination/capture ratios, including a uniform2048 cap, not DPI alone.

Keep pending intent in close busy checks before registration, and keep the real job after
command completion through decode/publication/retirement. Do not replace this with an
execute_blocking call first invoked inside a new thread. Join only after is_finished; normal
close drains, while forced host Drop cancels cooperatively. The legacy public no-executor
workspace helper still displays metadata; independent UI tests use the actual app frame.
Native SDF Actual/pan is verified in a finite Mac workflow; wheel/physical-DPI and the new
combined edit-preview workflow remain unverified. Cancel/Retry/Records are transient
presentation controls; full addressability remains M0 work. Check current user availability
before further desktop input. Actions stays off.

Ordinary VAL-absent source-backed MOL now uses io::display::drawing_page_scene through
registered capture and Engine::drawing_page_scene for both native vector acquisitions.
Deferred capture retains complete Arc<StoredContent>, not only Page. Original MOL backing
selects closed display facts; associated filename/encoding never does. Preserve actual
current Page/IDs/frame, authored None/literal zero/unusual counts and source bytes; never
reuse SDF packing here. Common topology/H lives below UI, with the existing4096 hit
preflight before derived allocation. Drawing literal physicalH0 remains supported; SDF's
stricter physicalH/error/packing policy stays exact. The renderer chooses no valence.

Independent actual MOL CPU/GPU/consumer/Save/history/cancel and real-frame vector/hit tests
pass; native-window acceptance remains unverified while the user is active. Read
docs/verification-mol-display-model.md, verification-mol-display-tests.md and
verification-mol-display-ui.md. Root's initial direct-IO UI port failed to compile; the
thin Engine delegate fixes it without a dependency/lock change. Do not repeat that edge.
Source-free/CDXML remain literal; no general H/stereo/aromaticity/edit authority follows.
Centered Double/Triple geometry/hits/attachment and closed authoring now have independent
headless evidence; see the multiple-bond verification records and STATUS.md for exact gates.
The existing-bond order-edit command and UI/adapter/history/H-budget checks are delivered
below. Styles, sidedness, general
construction, inventory/platform/native evidence and full M0–M5 remain unfinished.

`bond.set_order` requires an explicit order, current revision/target IDs and
`rectify_neutral_carbon` policy. Validate both neutral nonisotopic/nonradical C endpoints'
literal H plus incident1/2/3 weights equals4 before changing either or returning a no-op.
The setter does not need positions and preserves unrelated metadata/None/geometry, IDs,
direction and fragments. Source-free/byte-proven native CDXML authority is mandatory;
filenames do not confer authority. No IDs are consumed. Reuse complete-snapshot commit and
preserve same-order Arc/history/redo/dirty/close state. Read
docs/verification-bond-order-command.md and docs/research/bond-order-edit-plan.md.


Existing-bond canvas routes use `bond.set_order`; numeric1/2/3 are absolute and do not
change the selected tool. Preserve a stationary logical hotspot only across our own
setter. Fresh positions re-hit; history/context/source/epoch/focus invalidation must not
revive a cached target. Keep Edit shortcuts and Edit/Page menu commands ordered with raw
canvas events and resolve current control availability at dispatch. Reacquire complete
backing, page dimensions, viewport and graph after history, including the non-drawing SDF
fallback. Unsupported Double alignment remains a visible capability gap. The final tests
and finite Mac evidence are in docs/verification-bond-order-{ui,native}.md and
docs/verification-bond-history-order.md; no wider26.1 matrix is established.

## Read-only native SDF fields

`SdfFieldsWorkflow` uses the existing `sdf.record.inspect` through registered `UiSdfWork`,
never a frame parser or another executor. One source/close/record/offset/request-qualified
pending intent, bounded channel/job and field page stay outside authored state. Keep
pending/publication/actual thread retirement in native close busy checks; join only after
is_finished, including engine-free cancelled cleanup on a poisoned lock. Stale queries
never replace Engine state. Use actual next_field_offset and occurrence indices, not
name maps or a fixed64-page increment; omitted null differs from an existing empty value.

Readonly values use immutable TextEdit buffers, AccessKit ReadOnly and cleared/stored
TextEdit undoers while preserving current selection/copy. The pinned SDK otherwise stores
full readonly values when focused. Preserve the five independent real-App tests and two
Root lifecycle regressions; six older App fixture changes are default initialization only.
Native Mac finite inspection/typing/Unicode Save As/reopen passes; Actual/pan now recovers
the preview after default-window em refusal. Native tool
“settable” annotations and a failed Zoom SetValue attempt prove no permission or cause.

Read docs/verification-sdf-inspector.md, verification-sdf-editor-integration.md and the
source-backed workspace plan for the delivered editor. Retain Apply/Discard/Cancel across
navigation/Open/close, ordered Apply→Save and the predeclared788-byte second-DUP workflow. The
registry now has30 commands including document.new; full UI addressability and Professional field-screen evidence are
still incomplete. GitHub Actions remains disabled. See STATUS.md for current artifacts.

## Source-qualified SDF camera

Default `render.capture` stays whole-page and independent of the camera. Its optional
closed `view` context captures logical dimensions and expected view epoch; physical output
dimensions remain separate. `PageView::at_pixel_size` generates fresh physical geometry,
preserves logical percentage/phase and rejects a second conversion. Keep exact1x identity,
whole-source/ink/resource guards and actual logical readability; never multiply DPI into
the percentage, upscale masks or skip off-camera invalid records.

SDF Actual/Fit/SetZoom require no fabricated bounds. Successful opted-in capture alone can
install Weak CaptureStamp + PageSize after source/view/Open-close/cancellation checks.
Pan and relative-from-Fit require those current bounds; errors install nothing. Historical
successful replies cannot prepare another source/engine. The cache grants no Drawing,
authoring/history/Save/export/native-ACK authority. Default native Fit retains the legacy
whole-page path; explicit zoom opts into the logical camera. Do not equate legacy physical
Fit and opt-in logical Fit at2x.

Native preparation uses `prepare_ui_capture_for_close` before spawning to bind the pending
close epoch as well as source/view. A Rust-only result flag wakes the host only when its
current extent is newly installed; it never enters JSON. Keep actual job/decode/retirement
and reply barriers. Both collection and drawing input paths resolve current camera actions
at their raw events. Repeated wheel uses shared math and respects foreground/paper/text/
modal/DPI/source guards. See docs/verification-sdf-view-integration.md; no full M0/M1 claim.

## Whole SDF record removal

The29th command is `sdf.record.remove {expected_revision,record_id}`. Keep the exact whole
source through the shared deferred candidate/commit and Arc history. The codec removes
only the validated complete record range, reparses/reindexes local ordinals and refuses
the last compound per CTFILE-2026 printed97. Validate missing target before Empty. A
survivor stays SDF/document:null; Save remains the same complete-source operation.

Native removal uses the existing SdfEditor Field/Remove pending/job owner, not a second
pool. Resolve changed drafts before fresh nonce-qualified confirmation. Clear old record/
field/query ordinals, preserve truthful committed success after late cancellation, and
hold File/close through actual worker retirement, including poisoned-lock draining.
Same-packet Confirm→Save must wait for the mutation result and retirement.

Retain opaque UiSdfRemoveContext from replay through pending. Release the frame Engine
mutex before prepare_ui_sdf_remove; that method atomically checks source/Open/close,
path/format/file-request epoch and absence of a request before permit/preparation.
Generic prepare_ui_sdf is query/field only. The initial implementation lost file context
after frame unlock; an independent actual-worker test reproduced it. Never remove this
regression or replace the locked check with another pre-call UI check.

Read verification-sdf-record-removal.md, the independent command/codec/App records and
research/sdf-record-removal-review.md. Root's final local gate passes1060/0/1, all other
xtask gates included. Native Mac Keep/Confirm/Undo/Redo/Unicode Save/reopen/retained NOTE
passes on ordinary740→330 bytes; native draft/race/close and Windows/Linux remain
unverified. The current native bundle/process and next work are in STATUS.md. Actions
stays disabled; no broader reference-screen/chemical parity follows from this command.

## Fresh sessions and native New

The30th command is `document.new {expected_revision, discard_unsaved:false}`. It creates
a distinct source-free A4 session even from empty, advances checked revision/generation/
view epoch before mutation and clears association/history/IDs/extent/focus. Retain the
existing Engine, save lease, host, selected tool and canvas dimensions. Do not replace the
Engine with default: old saves still own their lease and may publish truthfully, while an
old generation cannot associate or clean the new session. Dirty, stale and close guards
must preserve state; document.new cannot acknowledge a Save or close.

`file.new` uses `ui.file` actionnew and Accepted-New authority through create_ui_document.
Resolve changed SDF drafts first. New Save resumes only after the current trusted actual
Save receipt and JoinHandle retirement; cancellation/failure/staleness/close/poison disarm
continuation. An ordinary retry must never silently restart New. Reuse FileWorkflow and
the existing worker budget. No new pool, save lease or receipt authority is needed.

Existing DrawingState tracks the last actually presented namespace. New/Open between
frames must clear stale gestures/hotspot and reject old-surface raw canvas input until
the new canvas is painted. Readonly surfaces tag their layout; invalid/unsupported Drawing
returns do not tag an unseen surface. The Rust-only session_generation getter is local
to the same Engine, not globally unique or command authority. Do not replace full
CaptureStamp job guards or this persistent fence with a poll-only flag: two independent
actual-App regressions reproduced unintended C–C bonds on the old surface.

Read verification-new-document.md and linked command/App/worker/review records. The
final local gate passes1090/0/1. Native Mac SDF→New, dirty Cancel/Discard/Save, Unicode
Save-before-New, editable reopen and Undo pass; native races and other platforms remain
unverified. Next is the existing restricted SMILES converter's honest native workflow,
then broader drawing import/layout. SDF reordering is deferred, not delivered. Full M0/M1
and M2–M5 remain incomplete; Actions stays disabled.

## Native file conversion and stable picker controls

`file.convert_smiles` binds existing ui.file actionconvert_smiles; no shortcut or new
registry command. Specialized prepare_ui_smiles_conversion requires exact Accepted intent,
reserves under origin mutex and consumes presentation only on successful existing command
prepare. It cannot bind/finish a trusted Open/Save or close ACK. Keep the existing save lease,
worker budget and actual handle through retirement. Conversion never opens its output or
marks the active document saved. Source defaults/profile and all eight actual parameters
remain explicit. Broader import and reference clipboard roles remain separate missing work.

Unstarted obsolete forms/focus dismiss on source/close transition. Running/publication
outcomes remain owned/reportable. FileWorkflow Drop cancels conversion work too: an
independent gated test reproduced output after owner destruction. Explicit valid retry
clears its prior global error; never clear later unrelated errors indiscriminately at finish.

Asynchronous chooser rows must not change the filename/footer's parent allocation or
geometry. Pinned egui child automatic IDs depend on that allocation; a reproduced click
loss occurred even with the old point inside both rectangles. Keep the fixed browser
region and separately always-allocated two-row error region. Errors must stay visible
after the main list scrolls; reset their scroll only for new message/navigation via one
fixed temporary key. No per-message cache or forced continuous scroll. Normal browser
correction does not establish confirmation-path stability. Keep both frozen component
regressions and all old chooser assertions. Actual prior timeout causality remains unknown.

Read verification-native-smiles-conversion.md, its command/App/worker/review links and
verification-file-picker-stability.md. Native and final aggregate scope live in STATUS.md;
do not infer either from headless App tests. Actions stays disabled, no new dependency,
parser/corpus/oracle/font/golden change. Next M1 work is broader SMILES drawing import/layout.
