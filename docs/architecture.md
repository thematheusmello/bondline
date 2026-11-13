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

Format crates own their syntax and source-preserving representations. They do not depend
on the model. The io layer translates supported data and retains original backing for
unknown content; unsupported edits must fail explicitly rather than discard data.

Layer 0: model, cdxml, cdx. Layer 1: chem, render. Layer 2: io. Layer 3: commands.
Layer 4: mcp/loopback adapters. Layer 5: cli, ui. Tooling lives in xtask. Core is GUI-free.
An independently implemented dynamic-library OS boundary may be necessary to keep the
graphics runtime within the strict license policy. Its exact API and unsafe surface must
be isolated, documented and tested against platform APIs before it is used.

`cargo xtask layers` reads Cargo's complete all-features dependency resolution. It rejects
unclassified Bondline packages, incorrect layer declarations, equal/upward workspace edges,
workspace dependencies in standalone parsers, and GUI dependencies reachable from any
headless product crate. Compatibility OS shims and build tooling are named exceptions to
the product-layer map; this is not permission to add GUI dependencies to core crates.

The graphics foundation will adopt PhotoCraft's reviewed, pinned fork approach. Do not
copy unrelated product features or claim that PhotoCraft meets Bondline's license policy.
