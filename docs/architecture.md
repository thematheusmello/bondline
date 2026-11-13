# Architecture

Engine first. Authored documents are pure data with stable IDs and point-based coordinates.
Runtime selection, cameras, jobs and revision counters are separate from authored state.
Command handlers validate before committing an edit. Snapshot history restores the entire
document; failed commands do not mutate it. All delivered interfaces use the same registry.

Format crates own their syntax and source-preserving representations. They do not depend
on the model. The io layer translates supported data and retains original backing for
unknown content; unsupported edits must fail explicitly rather than discard data.

Layer 0: model, cdxml, cdx. Layer 1: chem, render. Layer 2: io. Layer 3: commands.
Layer 4: mcp/loopback adapters. Layer 5: cli, ui. Tooling lives in xtask. Core is GUI-free.
An independently implemented dynamic-library OS boundary may be necessary to keep the
graphics runtime within the strict license policy. Its exact API and unsafe surface must
be isolated, documented and tested against platform APIs before it is used.

The graphics foundation will adopt PhotoCraft's reviewed, pinned fork approach. Do not
copy unrelated product features or claim that PhotoCraft meets Bondline's license policy.
