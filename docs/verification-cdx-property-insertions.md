# Combined CDX property insertion and replacement

The standalone edit uses the same checked splices and canonical lengths as
the [replacement batch](verification-cdx-batch-edits.md). A new tag belongs
to the exact containing object of its property anchor. The writer checks
both output record count and total bytes before emitting it. The
[source contract](research/cdx-implicit-element.md) records the format rules.

The compiler RED reported the missing insertion type and batch operation.
The four new cases and all previous standalone tests then passed **59/0/0**
with pinned Python. Warnings-denied all-targets Clippy also passed. The
independent binary constructions cover all five framings, an insertion
and replacement at the same anchor, same-anchor tag order, reused scoped
IDs, exact unchanged extended prefixes and trailers, conflicts, foreign
handles, invalid tags and exact record and 64 MiB output boundaries.
The fuzz harness compares generated insertion output with independently
assembled tag, length and unchanged source spans.

Receipts are under `target/cdx-resources/next-text/implicit-element`.
