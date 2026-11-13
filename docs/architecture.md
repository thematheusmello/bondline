# Architecture

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
CDXML, the original Arc<[u8]> plus its baseline model. Undo/redo move whole Arc references
in O(1); copy-on-write authoring edits share immutable fragments and source bytes. Equality
includes opaque source content, so equal projections cannot erase source-only changes.
Returned model snapshots remain immutable across later edits. Authored state excludes
revision/history. Exact undo assertions compare the whole stored document and original bytes.
The 128-snapshot count limit is not a byte budget; add a source/history byte budget before
exposing repeated large imports. A cold-session constructor is delivered, not an import command.
See control-protocol.md for the actual registered operations and explicit limits.

The MCP adapter owns a shared `Arc<Mutex<Engine>>`; command handlers do not await while
holding it. Stdio is a separate process-local session. Only server/stdio SDK features are
enabled. `Engine::prepare` completes short edits or returns an immutable capture plan;
adapters release the lock before rendering/encoding it. Each server bounds capture workers
to two, keeps permits with the actual worker lifetime and cancels cooperatively when an
awaiting task is dropped. Captures report their input revision and never apply results to
authored state. This is a bounded query path, not a general authoring/background-job system.

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
source bytes with untouched spans retained; no complete application file adapter exists yet.
Io's Geometry adapter maps direct-fragment positions and writes only an existing p value.
The read-only ElementalGraph adapter maps supported elemental properties and connectivity,
assigning separate application identities while retaining the original syntax. Its local
reference maps handle scoped source IDs and zero's unassigned meaning. Group hierarchy,
styles, captions and other unmodelled data stay in source backing, not in its flat projection.
StoredDocument owns that backing beyond parser lifetimes. It accepts zero or one direct
page with explicitly chosen preview dimensions; it does not derive print paper from graphical
bounds. Raw XML is neither serialized in command results nor included in StoredDocument Debug.
Unchanged models can export the exact original bytes; unsynchronized edits fail explicitly,
and exact undo restores passthrough. See file-formats.md; this is not an editable codec or
an exposed file-open/save workflow. Capture refuses imported/chemical content until the
renderer can represent it rather than returning a misleading blank preview.

The native canvas now consumes the render crate's page scene instead of owning a parallel
page-layout algorithm. CPU and instanced GPU reference paths consume that same scene;
tests also compare against an independent expected pixel grid. See rendering.md for exact
primitive semantics, rounding, resource bounds and the remaining rendering requirements.

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

The graphics foundation will adopt PhotoCraft's reviewed, pinned fork approach. Do not
copy unrelated product features or claim that PhotoCraft meets Bondline's license policy.
