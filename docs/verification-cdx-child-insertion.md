# CDX child insertion verification

Base `67cf050`, 2026-11-19. [The contract](research/cdx-child-insertion.md)
defines the binary splice and its matched-end ownership. The
[fixture register](../crates/cdx/tests/fixtures/child-insertion/SOURCES.md)
pins twelve original CDX files and their Python generator.

## Independent tests

The Python reader checks containment and framing independently of the Rust
implementation. The five before and after pairs add one 18-byte object to
Page 55 while preserving the exact enclosing bytes. The two controls show
that a parsable output can still place the child under the wrong owner or
duplicate a direct sibling ID. Each fixture reproduced byte-for-byte.

The predeclared Rust suite failed to compile with 18 missing-API diagnostics.
The first producer build needed the depth-limit constant imported. With the
test declarations unchanged, all eight cases pass. They cover the five
framings, direct and nested IDs, unassigned IDs, exact ownership across two
parses of identical bytes, empty owners, caller property order, the extended
length threshold, and exact depth, record and output byte limits.

The standalone CDX suite passes **67 tests** across 14 reports, with
warnings-denied CDX Clippy and rustfmt. The first suite attempt from the
isolated worktree lacked the required `BONDLINE_PYTHON` setting. It passed
with the pinned external interpreter and the same source and assertions.
Local receipts are under `target/child-insertion` in the isolated worktree.

## Sanitizer

The existing `cdx_parse` target now appends one unassigned child to an
available exact ObjectEnd, checks a literal encoding and byte retention,
and reparses to check framing and ownership. AddressSanitizer completed
**10,000 runs in one second** using 18 frozen inputs and an input limit
of 65,536 bytes. All seed hashes matched afterward. The largest recorded
new or reduced mutation was 363 bytes. The large-input and length
thresholds also have dedicated deterministic tests.

The sanitizer log SHA-256 is
`99235067fb877fd845208d9ddae60bbe6f29ab403c89abfc955385d2a7699a3b`.
The passing standalone CDX log SHA-256 is
`5e01826b71b18390cec7fab03cb1257df9219df024df92e2a8c7c25f9af0f24a`.

## Scope

The splice adds one property-only child to a matched encoded owner.
Callers own semantic admission, references, resources, style and
caption identity. The next layer is a qualified CDX Page Text creation
adapter with its own source, history, rendering and native checks.
