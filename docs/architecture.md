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

`bondline-commands` owns the registry and history. It validates a candidate document before
replacing an Arc<io::StoredDocument> snapshot. That owned snapshot holds the model and, for
CDXML/CDX/MOL, the original Arc<[u8]> plus its baseline model. MOL also shares three headers
and its captured coordinate frame. Undo/redo move whole Arc references
in O(1); copy-on-write authoring edits share immutable fragments and source bytes. Equality
includes opaque source content, so equal projections cannot erase source-only changes.
Returned model snapshots remain immutable across later edits. Authored state excludes
revision/history. Exact undo assertions compare the whole stored document and original bytes.
CDX backing is always incomplete: its elemental preview never authorizes regeneration or
lossy conversion. Registered CDX Open/Save and the native chooser reuse the same atomic
adapter, source authority and save/receipt/close barriers. Native acceptance is separate
from headless/worker evidence; see verification-cdx-application.md.
History is limited to 128 prior snapshots and a 128 MiB accounted-storage threshold for
the current document plus history. Accounting deduplicates source allocations, models and
immutable fragments, includes source baselines and reserved vector capacity, and saturates
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
The first bond.create operation appends an independently authored C-C fragment with three
explicit hydrogens per carbon. Engine identity floors never move backward on Undo/Redo;
a new branch cannot reuse deleted session IDs. Floors update only after a successful commit.
This is a closed creation case, not a general valence or connected-bond editing engine.

The MCP adapter owns a shared `Arc<Mutex<Engine>>`; command handlers do not await while
holding it. Stdio is a separate process-local session. Only server/stdio SDK features are
enabled. `Engine::prepare` completes short edits or returns an immutable capture/file plan;
adapters release the lock before long work. Completed work applies through Engine::finish;
open rejects stale revisions/generations, and old saves cannot relabel a later document.
A cloneable CommandExecutor shares UI/loopback workers; stdio has its own session. Each bounds workers
to two, reserves deferred permits under the engine mutex before releasing a plan, keeps
permits with the actual worker lifetime and cancels cooperatively when an
awaiting task is dropped. Captures report their input revision and never apply results to
authored state. File saves publish immutable input and report its revision, keeping later
edits dirty. This is a bounded capture/file executor, not a general background-job scheduler.

Close intent is transient engine state, outside StoredDocument/history. The typed
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
strokes and bounded typed atom/bond hit regions. It supports positioned neutral carbon,
Single bonds and literal authored H; missing H is not inferred. PageViewport shares paper
layout and authoring-pointer conversion. Unsupported metadata/source remains guarded.
CPU and GPU paths also face independent pixel/geometry/font oracles and one human-approved
native-document golden. Native transient gesture state stays outside authored snapshots;
ordered pointer events preview, commit one registry edit per drag, or cancel. This handles
an entire drag between frames without losing its released press origin. See rendering.md
for bounds, native versus headless raster semantics and the substantial remaining work.
Transient hover never changes the captured Scene, selection, snapshots or file state.
Its private freshness flag remains clear after revision/Escape/focus/context invalidation
until a new ordered PointerMoved/PointerButton. Context tracks logical canvas origin/size,
page/revision, public native viewport origin and effective pixels-per-point; no OS polling.
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
