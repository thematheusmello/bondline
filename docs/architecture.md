# Architecture

Engine first. Authored documents are pure data with stable IDs and point-based coordinates.
Runtime selection, cameras, jobs and revision counters are separate from authored state.
Command handlers validate before committing an edit. Snapshot history restores the entire
document; failed commands do not mutate it. All delivered interfaces use the same registry.

The first model slice contains pages only. IDs are nonzero u32 values; dimensions are finite
f64 points with validated construction/deserialization and a documented 100,000-point bound.
One point is 1/72 inch; native display scale is a view concern. A new document has one A4 page
with ID 1. Atoms, bonds, text and reactions will be added with their first verified behaviors.

`bondline-commands` owns the registry and history. It validates a candidate document before
replacing an Arc snapshot. Undo/redo move Arc references in O(1); authoring edits currently
clone the small document. Returned snapshots remain immutable across later edits. Authored
state excludes revision/history, so exact undo assertions compare authored bytes separately.
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

Format crates own their syntax and source-preserving representations. They do not depend
on the model. The io layer translates supported data and retains original backing for
unknown content; unsupported edits must fail explicitly rather than discard data.
CDXML's borrowed tree remains read-only. Its attribute replacement returns revalidated
source bytes with untouched spans retained; no model mapping or application file adapter
exists yet. See file-formats.md for the exact supported edit boundary.

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
