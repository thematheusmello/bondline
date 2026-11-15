# Architecture

All formats share a per-Engine save reservation acquired by command preparation. The
owned Arc token moves from FilePlan into Completed and lives through Engine::finish.
A second outstanding Save fails busy before I/O, including the publication-to-application
gap; other commands and other Engines remain available. This prevents out-of-order save
receipts from changing association or baseline after newer publication. Error/cancel/drop
releases the token. This is a save-order safeguard, not a general job scheduler or a lock
against independent applications editing the same file. See control-protocol.md.

Trusted file completion has a separate private currentness bit. Its own matching finish
sets it after baseline changes; subsequent context invalidation clears it permanently.
UI receipt consumption checks and retires this state before a close continuation. Returned
JSON receipt fields remain historical snapshots; path/format/revision values returning
to old values cannot turn them into fresh close authorization.

Engine first. Authored documents are pure data with stable IDs and point-based coordinates.
Runtime selection, cameras, jobs and revision counters are separate from authored state.
Command handlers validate before committing an edit. Snapshot history restores the entire
document; failed commands do not mutate it. All delivered interfaces use the same registry.

The authored document contains pages and immutable elemental-graph fragments. Page IDs
are nonzero u32 values; dimensions are finite
f64 points with validated construction/deserialization and a documented 100,000-point bound.
One point is 1/72 inch; native display scale is a view concern. A new document has one A4 page
with ID 1. Point2 provides finite signed coordinates within ±100000 pt for io's source-backed
CDXML node geometry view. Zero and negative coordinates are valid; absent positions remain
None. AtomId, BondId and FragmentId are separate nonzero-u32 application identity domains,
unique document-wide within each kind. They do not reuse container-scoped source identities.
See model.md for structural validation, atom metadata, bond direction and resource limits.
Text, query nodes, reaction models and chemical perception remain missing.

`bondline-commands` owns the registry and history. It validates candidates before replacing
an Arc<StoredContent> snapshot. Its Drawing variant contains io::StoredDocument with the model and, for
CDXML/CDX/MOL, the original Arc<[u8]> plus its baseline model. MOL also shares three headers
and its captured coordinate frame. Undo/redo move whole Arc references
in O(1); copy-on-write authoring edits share immutable fragments and source bytes. Equality
includes opaque source content, so equal projections cannot erase source-only changes.
The Sdf variant instead contains complete immutable original/current SdfDocument values,
including every ordered record/field and mixed molecular version. It never substitutes an
active-record graph or empty drawing. Returned model snapshots remain immutable across
later edits; SDF results have document:null plus count/record metadata. Authored state
excludes revision/history. Exact undo assertions compare complete content and original bytes.
CDX backing is always incomplete: its elemental preview never authorizes regeneration or
lossy conversion. Registered CDX Open/Save and the native chooser reuse the same atomic
adapter, source authority and save/receipt/close barriers. Native acceptance is separate
from headless/worker evidence; see verification-cdx-application.md.
History is limited to 128 prior snapshots and a 128 MiB accounted-storage threshold for
current/saved content plus history. Accounting counts each outer StoredContent inline once,
deduplicates drawing source/model/fragment and SDF source/index allocations independently,
includes original/current/saved baselines and reserved vector capacity, and saturates
on overflow. A changed commit clears redo first and drops the oldest Undo prefix needed
to retain a recent suffix. Undo/Redo move that existing pool without new payload allocations.
The current document is always retained, even if it alone exceeds the threshold. This is
approximate retained payload/container accounting: Arc/allocator headers, temporary work,
caches and externally held snapshots are excluded, so it is not a process-memory guarantee.
The cold-session constructor creates a clean unassociated baseline. Native open/save are
registry commands with independent file-consumer and failed-import-retention evidence.
Every snapshot retains its saved baseline; essential current+saved data may exceed the
history threshold. Source.modified and file.dirty describe separate original/saved baselines.
See control-protocol.md for the actual registered operations and explicit limits.
The default bond.create operation appends an independently authored C-C fragment with three
explicit hydrogens per carbon. Optional Double/Triple order instead authors literal
H=`4-order_weight` at both ends. Engine identity floors never move backward on Undo/Redo;
a new branch cannot reuse deleted session IDs. Floors update only after a successful commit.
This is a closed creation case, not a general valence or connected-bond editing engine.
A separate closed bond.extend command replaces one same-ID fragment atomically. A neutral
carbon terminal with one Single/Double/Triple bond must have the consistent literal
H=`4-existing_weight`; the requested weight must fit that budget. It subtracts that weight
and adds a new C/H=`4-requested_weight`, preserving all old identities.
Required expected revision/policy, complete-source authority and global atom/bond floors
are checked before one history commit. Only source-free/byte-proven native CDXML is
allowed; imported MOL/CDX refuses. Native attachment is now wired through that same
registry, with fresh per-press graph hits, raw click threshold versus exact atom anchor,
per-event foreground/paper release guards and ordered graph/revision reacquisition.
Actual native acceptance remains separate from deterministic input tests.

`bond.set_order` replaces one existing edge in a same-ID fragment and corrects only its
supported neutral-carbon endpoints under the required explicit C4 H policy. Every incident
Single/Double/Triple contributes to the literal budget; absent/inconsistent H and invalid
source authority refuse before mutation. Positionless graph edits and unrelated hetero data
are retained exactly. Validated equal order is a true complete-state/Arc/history no-op; no
identity floors are consumed. See [the closed contract](control-protocol.md#existing-bond-absolute-order-setter).

`ui.bond_tool` owns a checked transient order/epoch, outside StoredContent/history/dirty
state. Real changes advance that epoch; no-op selection does not. Normal command snapshots
omit it; explicit operation replies and three central bindings expose it. Selection
persists across history/Open, and direct authoring defaults remain Single. Actual UI
activations are ordered by their input-event indices; each tool state is replayed at its
event boundary so later activation cannot relabel an earlier press. The gesture stores
that order/epoch and explicitly passes its order into the existing authoring commands.
Loopback publication notifies real tool changes even though authored changed=false.


`ui.view` keeps zoom, optional point center and checked epoch outside stored content.
Default Fit uses the exact old PageViewport; explicit PageView uses a uniform logical
scale, generates whole-source geometry and then clips it with signed integer translation.
IO resolves its existing MOL display-H facts into that same camera without rewriting
authored None or source bytes. Engine::page_view and drawing_view_scene serve both
initial layout and later same-frame command reacquisition. Physical raster density stays
separate from the logical camera; fresh glyph geometry is described below.

All view controls use the same tagged command. Window actions are resolved at their raw
event boundary, so the other pan coordinate comes from the camera already in effect.
Explicit numeric accessibility requests retain A→B→A epochs even when the final widget
value is unchanged. Real camera changes retire gestures/hotspots; no-ops preserve them.
Wheel math uses the exact f64 scale and floored logical canvas dimensions. A retained
center beyond a newly opened page is clamped for projection and new navigation without
changing stored state during Open. Menu/numeric recovery remains available when a camera
exceeds renderer bounds. Default whole-page captures remain independent of the window
camera; the optional current-view capture below opts into it.

The MCP adapter owns a shared `Arc<Mutex<Engine>>`; command handlers do not await while
holding it. Stdio is a separate process-local session. Only server/stdio SDK features are
enabled. `Engine::prepare` completes short edits or returns an immutable capture/file/query/field-edit plan;
adapters release the lock before long work. Completed work applies through Engine::finish;
open rejects stale revisions/generations, and old saves cannot relabel a later document.
A cloneable CommandExecutor shares UI/loopback workers; stdio has its own session. Each bounds workers
to two, reserves deferred permits under the engine mutex before releasing a plan, keeps
permits with the actual worker lifetime and cancels cooperatively when an
awaiting task is dropped. Captures report their input revision and never apply results to
authored state. File saves publish immutable input and report its revision, keeping later
edits dirty. This is a bounded executor for delivered deferred operations, not a general background-job scheduler.

Close intent is transient engine state, outside StoredContent/history. The typed
document.close command handles Request/Discard/Cancel with an authored-revision guard;
its monotonic epoch identifies continuations across Cancel/new Request at the same revision.
Actual authored/baseline/path/format changes permanently invalidate approval. A close-specific Save
renews Request only after its matching current receipt/epoch; an unrelated save cannot do so.
The native adapter acknowledges valid approval only while holding the engine mutex, with
no registered workers/replies, UI file jobs or pending file action. That acknowledgement
commits shutdown; new commands except Inspect refuse. The retained document/history stays
intact until the host is destroyed. A late Open cannot resurrect it; a late published Save
still reports its receipt without relabeling the committed session.

The executor counts loopback reply publication as activity, registered before a short
command can approve close and released after output write/flush. A callback alone is
insufficient: another UI frame could poll before it fires. This barrier and the shared
engine mutex prevent that race without a second command registry. Current lifecycle metadata
is refreshed at finish, while an older operation's authored snapshot remains immutable.
See control-protocol.md for state/output semantics and native-file-ui.md for actual evidence.

Delivered File/Edit/Page bindings are typed engine metadata whose invocations contain the
same registered Command enum. Native menus/shortcuts and ui.controls read those bindings.
The ui.file command stores one bounded native presentation intent outside authored history;
headless engines refuse it without a host. IDs, revision/path/baseline/format and close epochs guard
picker continuations. The native adapter reserves work and prepares a selected file command
under the same engine lock, then sends an opaque engine-bound plan to a worker. A host
acknowledgement is distinct from visible pixels or a successful file receipt. See the
control contract for phases, retries, cancellation and still-private dialog fields.

The native app and opt-in loopback server now share this engine. A dedicated adapter thread
owns the Tokio listener/tasks. Input is bounded before JSON decoding; no engine guard crosses
an await. The server notifies the UI after committed changes and cancels all connection tasks
on shutdown. Headless stdio remains independent. See control-protocol.md for actual limits.
The first bounded frame authenticates each loopback connection with an OS-random session
token before the engine lock or command dispatch is reachable. Startup stdout is the only
native credential-delivery channel; the UI displays the address only. Tokens last for the
server lifetime and authorize that connection's current document commands.

Format crates own their syntax and source-preserving representations. They do not depend
on the model. The io layer translates supported data and retains original backing for
unknown content; unsupported edits must fail explicitly rather than discard data.
CDXML's borrowed tree remains read-only. Its attribute replacement returns revalidated
source bytes with untouched spans retained. A separate bounded standalone Writer builds new
syntax without model/native dependencies; it never regenerates an imported tree. Io maps
new elemental model documents to it with unique source IDs, metadata and Poster canvas
extents. Native filesystem and registry adapters retain complete StoredDocument backing;
UI delivery is tracked separately and general imported structural editing remains missing.
Io's Geometry adapter maps direct-fragment positions and writes only an existing p value.
The read-only ElementalGraph adapter maps supported elemental properties and connectivity,
assigning separate application identities while retaining the original syntax. Its local
reference maps handle scoped source IDs and zero's unassigned meaning. Group hierarchy,
styles, captions and other unmodelled data stay in source backing, not in its flat projection.
StoredDocument owns that backing beyond parser lifetimes. It accepts zero or one direct
page, reads complete explicit Poster extents and otherwise uses caller preview dimensions;
it does not derive print paper from graphical bounds or decode printer records. Raw XML is
neither serialized in command results nor included in StoredDocument Debug.
Unchanged source-backed models export the exact original bytes. The constructor classifies a
complete native elemental profile only when re-encoding reproduces every input byte; the
cached decision is not based on a producer name. Only that profile can safely encode edits
and participate in headless rendering/bond creation after reopen. Other unsynchronized edits
fail explicitly, and exact undo restores passthrough. to_cdxml borrows unchanged source or
returns owned native/profile bytes. Save callers must use that full StoredDocument entry point, not regenerate
stored.model(), which lacks unknown data. See file-formats.md; general imported structural
editing remains missing. Native file UI dispatches those commands in background workers,
checks receipt revision/path before closing and protects dirty Open/Save/Close. Directory
listing is a separate bounded worker; transient dialogs/jobs stay outside authored history.
Capture refuses unmapped or unsupported chemical
content instead of returning a misleading blank preview; the native C-C creation case works.

The native canvas and registered capture consume one GraphScene with ordered rectangles/
strokes and bounded typed atom/bond hit regions. It supports positioned neutral elements
with a shared pure-model symbol catalog, Single/centered Double/Triple bonds and literal authored H; the literal
renderer does not infer missing H. A source-aware IO scene entry supplies separate closed-
profile display counts only for actual MOL backing, preserving current Page/IDs/frame and
authored None. Source-free/CDXML keep the literal path. PageViewport shares paper
layout and authoring-pointer conversion. Unsupported metadata/source remains guarded.
CPU and GPU paths also face independent pixel/geometry/font oracles and one human-approved
native-document golden. Native transient gesture state stays outside authored snapshots;
ordered pointer events preview, commit one registry edit per drag, or cancel. This handles
an entire drag between frames without losing its released press origin. See rendering.md
for bounds, native versus headless raster semantics and the substantial remaining work.
`centered_bond_strokes` is the shared geometry constructor for graph production and held
preview; `GraphScene::bond_strokes` supplies the actual components for hover. Masks,
clipping, pixel/gap visibility and congestion stay scene-level checks. The original3w
profile does not model imported spacing/position/style inheritance or general joins.
Transient hover never changes the captured Scene, selection, snapshots or file state.
The thin workspace uses a48-unit header,64-unit procedural tool rail,248-unit read-only
document inspector and fixed24-unit page/job footer. Tool policy and errors live in the
inspector, keeping paper placement stable while retaining label-readability guards.
Explicit two-button rows avoid egui Grid's first-pass discard. These are original
presentation choices; the shell does not add authoring or file authority.
Its private freshness flag remains clear after revision/Escape/focus/context invalidation
until a new ordered PointerMoved/PointerButton. Context tracks logical canvas origin/size,
page/revision, tool epoch/order, public native viewport origin and effective pixels-per-point; no OS polling.
Synthetic regressions cover multiple idle frames and ordered input, while actual native
acceptance is recorded separately in verification-carbon-graph.md.

Layer 0: model, cdxml, cdx. Layer 1: chem, render. Layer 2: io. Layer 3: commands.
Layer 4: mcp/loopback adapters. Layer 5: cli, ui. Tooling lives in xtask. Core is GUI-free.
Original dynamic-loader and Win32 clipboard boundaries now isolate OS FFI needed to retain
the strict graphics license policy. Their exact pinned-consumer APIs, unsafe surface and
platform evidence are documented in dependency-policy.md; native Windows/Linux tests remain.

`cargo xtask layers` reads Cargo's complete all-features dependency resolution. It rejects
unclassified Bondline packages, incorrect layer declarations, equal/upward workspace edges,
workspace dependencies in standalone parsers, and GUI dependencies reachable from any
headless product crate. Compatibility OS shims and build tooling are named exceptions to
the product-layer map; this is not permission to add GUI dependencies to core crates.

The graphics foundation adopted PhotoCraft's reviewed, pinned fork approach. Do not
copy unrelated product features or claim that PhotoCraft meets Bondline's license policy.


The whole-SDF owner validates/retains all bytes plus shared compact ordinal record metadata.
StoredContent now retains that owner throughout current/saved/history and immutable file
plans. Registry Open validates all records outside the engine lock and applies once;
Save borrows complete current source through the existing bounded atomic publisher. No
wrapper writer, first-record projection or field-name dictionary reconstructs an import.
Drawing accessors return typed unsupported_content errors for collections. Static UI
metadata clears prior gesture/hit/paper state; the no-executor compatibility helper has no preview;
all-record chemical display uses the registered path below; record operations remain missing. The existing-value
editor below now delivers the separate field-editing UI. See
verification-sdf-session-files.md for exact source/consumer/history/lifecycle evidence.


SDF explicit field inspection captures immutable source/revision/generation, measures bounded
ordered JSON off-lock and returns a historical result without applying authored state.
Existing-field value edits similarly prepare an immutable complete-content candidate,
then recheck cancellation, close, revision/generation and current Arc before one commit.
No-op retains source/index identity, redo and transient close state. Input/output budgets,
explicit versus automatic data disclosure, source/current/saved meanings and existing
worker/reply/save barriers are defined in control-protocol.md. No extra pool or registry.

The native field adapter registers those two existing operations through
`CommandExecutor::prepare_ui_sdf` before spawning. It binds an opaque work value to the
originating Engine, complete CaptureStamp, Open close epoch and existing shared worker
reservation. Core completion/finish and accepted-earlier close semantics stay unchanged.
See [the adapter checks](verification-ui-sdf-work.md).

`SdfFieldsWorkflow` now adds explicit read-only native occurrence inspection. One pending
intent, one bounded channel/job and one field page sit beside the existing preview.
Source/close/record/offset/request keys reject stale installation; native close includes
pending work through actual thread retirement. Query/formatting runs off-lock, then the
inspector displays ordered occurrences and follows returned pagination offsets. It never
parses SDF in the frame, installs historical Engine state or grants drawing/Save authority.
Immutable text buffers retain selection/copy; explicit AccessKit ReadOnly and cleared text
undo storage prevent edit claims and retained payload history. Finished cancelled work
can retire without an Engine guard. This is original presentation, not reference-field
UI parity or a mutating editor. Draft/Apply/Save/close policy remains separate next work;
see [the independent UI and native evidence](verification-sdf-inspector.md).

SMILES retains a distinct complete notation/source IR, including literal aromatic atom/bond
markers. These are not perceived aromaticity; every aromatic marker/bond refuses fixed-H
model mapping. Named opt-in normalization returns
a new local-ID Fragment with fixed neutral-profile H and absent positions; it changes no
source token or model None semantics. The separate restricted explicit-H writer emits
noncanonical notation from a Fragment. Neither gives a projected graph source-backed Save
authority, native layout or document-global IDs. Future import must preserve source,
remap local identities and define layout/conversion/unsupported policies explicitly.


The separate named carbon-path layout creates a new pure drawing from a narrow normalized
profile, checking actual topology, analytic geometry and reference-scene ink/page bounds.
It preserves the original source/IR, adds positions without changing normalized metadata,
and grants no application SMILES association or source-backed Save authority. Explicit file
conversion now has a create-only IO API using the shared bounded reader and native publisher;
it retains the complete input owner through publication, returns metadata, and makes no
session association. `file.convert_smiles_to_cdxml` exposes it through the single registry,
existing deferred worker/close barriers and save reservation through completion finish/drop.
Its receipt identifies historical preparation while returning the latest unchanged session;
it never grants a trusted native Save acknowledgment. Native input workflows remain separate;
see verification-smiles-layout.md and verification-smiles-conversion-io.md.

`file.export_smiles_fragment` takes an explicit page/fragment and captures the complete
immutable content owner only after its existing source-completeness guard. The worker
uses that same owner, existing full-profile writer and create-only publisher. Owner and
save lease remain in Completion through finish/drop. Finish returns current state plus
historical `smiles_export` metadata, never a Save baseline or association change. Scope
is the selected chemical graph; geometry, application IDs/order, unselected content and
source/document metadata are outside the output. Unsupported chemical data still refuses.
One LF frames the notation; this is distinct from source-backed SMILES Open/Save or
native export controls. See verification-smiles-export-command.md.

The separate complete-file `SmilesFileDocument` retains an Arc byte source and compact
Arc record index, never every parsed IR or a first-record drawing. Each meaningful
notation is validated transiently through the unchanged bounded parser. `StoredContent`
adds one immutable SmilesFile owner; no redundant original/current pair is needed without
a file edit operation. Current/saved/captured plans retain the whole variant, with shared
source/index accounting. A new Open replaces the complete session only after common
revision/generation/dirty/close checks. Save borrows exact whole bytes through the same
publisher and reservation through finish/drop. No drawing encoder is used.

Automatic state uses an O(1) smiles_file summary (format/byte length/record count) and
document:null. It never materializes a recordsVec or discloses notation/application data.
Drawing/SDF operations refuse the content kind. The UI clears stale gestures/hover and
shows only count/size plus unavailable-preview status. Its existing file worker routes
SMILES Open/Save/Save As and clean-close draining; native execution remains unverified.
Explicit conversion/export remain distinct commands with their old limits and association
rules. See verification-smiles-file-app.md and verification-smiles-file-ui.md.

`smiles.record.inspect` captures a whole immutable StoredContent owner, then reparses only
one explicit source-local record off-lock. Exact notation and compact file-relative
ranges/counts are returned under a128KiB serialized-object cap using the existing bounded
JSON helper. Application data stays opaque in source. The completed reply contains no
extra whole-source owner; Debug and automatic state omit notation. Existing historical
read-only result/close/UI refresh, deferred cancellation and worker/reply barriers apply.
No save reservation, native file binding/ACK, mutation, selection or repaint is added.
Independent raw/graph/wire evidence is linked from verification-smiles-record.md.

## Derived all-record SDF capture

The existing read-only capture plan now retains an Arc of the complete SDF content, then
borrows each indexed actual-version MOL off-lock. A bounded IO producer keeps authored
None/literal H exact and emits a separate display-count/provenance map with globally unique
preview IDs and per-record local correspondence. One shared coordinate transform and one
translation per record preserve molecular directions. All records succeed or the whole
preview refuses. No derived Page is placed in Engine content or saved baselines.

A separate renderer entry consumes exact resolved counts and chooses label attachment;
it performs no valence perception. Actual glyph/mask/stroke ink must fit the paper. The
legacy literal path, font and approved golden stay exact. The same scene feeds CPU/GPU
and the existing PNG response. Historical/current lifecycle, cancellation and registered
worker/reply barriers stay shared; capture acquires no Save reservation.

The native SDF adapter now uses a narrow UiCaptureWork reserved before spawning, plus one
UI job retained through PNG/decode/publication/thread retirement. Its key includes opaque
Weak content/revision/generation, close/view epochs, logical rectangle, actual physical dimensions
and DPI. Texture/error/hit state is cleared on invalidation; only matching Open-context
results install. The PNG bridge validates the pinned own-output envelope before the existing
native decoder; actual main-label em is Rust-only and omitted from wire JSON. UI display
checks logical readability with actual image ratios. Registry execution still shares its
existing two-worker budget; the single bounded UI decode/publication stage stays tracked
separately for close after command completion. There is no new general job pool.

Actual app-frame/texture/paint instructions are headlessly tested; Mac Actual/pan now has
finite native acceptance. Wheel/physical-DPI and combined edit-preview native behavior
remain unverified. Cancel/Retry/Records are transient controls whose full
public addressability remains part of M0. See verification-sdf-native-preview.md and its
independent acceptance record.

## Source-aware Drawing display

Every deferred Drawing capture now retains the complete Arc<StoredContent>, including
source/baseline, until work completes or drops. Current native vector painting calls
Engine::drawing_page_scene at initial acquisition and after an attempted edit; the UI has
no direct IO dependency. Both paths use io::display::drawing_page_scene, which validates
source coverage and chooses policy from original MOL backing rather than file association.
It uses the actual current Page, identity and retained coordinate frame without SDF packing.
An existing4096 hit preflight bounds work before derived topology allocation.

The common private resolver retains source/model None and all literal counts. Drawing
literal physical H0 remains compatible; SDF keeps its separate stricter physical-H rule,
record errors and exact packing. Resolved labels still require readable, safe paper ink.
Unknown chemistry refuses as a whole. Display facts create no edit/Save authority or
history step. Independent command/CPU/GPU/adapter and real-frame checks are recorded in
verification-mol-display-tests.md and verification-mol-display-ui.md; actual native window
acceptance and broader chemistry/orders remain unverified/missing.

## Transactional SDF field drafts

The occurrence inspector owns one `SdfEditor`, separate from its read-only query page.
A draft retains the complete source stamp, revision, record/field ordinal, original text
and one byte-bounded temporary buffer. It creates no authored snapshot or saved baseline.
The buffer bounds UTF-8 bytes before copying, checks character indices and rolls back an
entire rejected widget transaction; only that draft's cursor/limited undo state is reset.
Existing immutable inspector values retain their read-only/text-storage protections.

Apply prepares the existing registered setter through `UiSdfWork` before spawning and
shares its two-worker budget. The job remains owned through publication and actual thread
retirement. The worker obtains a post-result stamp only when the originating Engine still
has the result revision; the UI checks that full stamp before refresh or continuation.
A later source change keeps recovery text and the historical success, with no replay.
Successful mutation is never dropped as if it were a cancelled read-only response.

Both the drawing input loop and collection fallback replay editor activations at their
raw event indices. A later File intent waits for the actual Apply result/retirement.
The resolver retains a semantic target plus source/path/format/file/close context, then
resolves fresh bindings after Apply. Discard can adopt a still-current external request;
changed Apply invalidates it and uses a fresh request. Cancel issues real cancellation.
Field-page navigation waits for the refreshed query instead of using a cleared old page.

Native and remote close requests pass through the same draft resolver; changed drafts,
decisions, continuations and pending/live Apply work join the existing shutdown barriers.
Native Mac edit/Save As/history/reopen and finite independent App/worker tests have
separate scopes in [the integration record](verification-sdf-editor-integration.md).

## Current-view capture and SDF pan bounds

`render.capture.view` optionally captures current camera state with closed logical
dimensions and an expected view epoch. Physical output dimensions are independent. The
absent/null path retains exact legacy whole-page pixels. Opted-in work owns complete
source, derives every SDF record/real extent off-lock and rasterizes the shared `PageView`
at the actual output density. Drawing captures use the existing source-aware IO entry.
No second chemistry/layout implementation or UI parser is introduced.

Only a fully successful render/encode can prepare SDF bounds. Finish checks cancellation,
Weak source/revision/generation identity, camera epoch and exact Open/close context before
installing only PageSize. Errors install nothing. Historical successful images can still
be returned, but cannot install in a newer source or foreign Engine. The cache is valid by
source, so ordinary pan/zoom keeps it; history/Open/field edits invalidate it through the
full stamp. It owns no Page, source lease, edit/Save authority or history snapshot.

Actual/Fit/SetZoom need no bounds and can recover an unreadable SDF overview. Pan and
relative-from-Fit need the prepared size; explicit relative zoom uses its known percentage.
The same accessor serves controls, view inspection and pan limits. No recent-drawing focus
is inferred for SDF. Default native Fit retains the old physical-margin request; explicit
zoom opts into logical camera capture. Opt-in headless logical Fit is deliberately distinct.

Native capture preparation also checks the pending close epoch before reserving/spawning.
First current extent installation sets a Rust-only notification flag, absent from JSON;
repeat captures do not fake a camera change. UI keys include view epochs, preventing
A→B→A texture reuse. Collection input is replayed before painting/scheduling; its wheel
uses the same math as Drawing and preserves focus/modal/foreground/paper/context guards.
The existing job/decode/retirement and reply barriers remain. Exact evidence and limitations
are in [the view integration record](verification-sdf-view-integration.md).
