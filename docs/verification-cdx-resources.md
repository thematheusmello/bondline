# Standalone CDX resource verification

The checked resource query reads Document-owned font and colour tables from CDX. It
keeps borrowed font-name bytes, declared IDs, raw unsigned charsets, raw RGB channels
and source record spans. Empty tables remain present, and palette lookups address the
last stored index, 65536. The [format contract](research/cdx-resources.md) records the
public format evidence and unresolved Text questions.

Each call scans owners and duplicate tables before decoding either body. It validates
the complete payloads before reserving descriptor vectors. A bounded font ID bitset
detects duplicates in source order, and a sorted index resolves declared-ID lookups.
The only new dependency edge is test-only access to the already pinned workspace
`serde_json`. The standalone library has no runtime dependencies.

The [original fixture set](../crates/cdx/tests/fixtures/resources/SOURCES.md) has 17
successful resource queries and 31 query errors across 48 valid CDX sources. The
independent Python generator and reader reproduce every source, ordered record span,
lookup fact and check receipt byte-for-byte. They check all 65,535 entries of each
maximum-size table and every byte of a 65,535-byte font name. Fixture formatting was
qualified by reproducing the same binary sources, manifest, facts and checks before
committing them. The final scripts write pinned JSON as bytes, and checkout attributes
preserve resource fixture bytes with `core.autocrlf` enabled. The portability change
reproduced all 48 files and all JSON pins. The
[resource manifest](../crates/cdx/tests/fixtures/resources/resource-vectors.json)
has SHA-256 `3919892595c4077623b320113252249dd54d0c93d80f7c8e41f26ad1b85c6317`.

The new API compile probe failed as expected at missing `ResourceError`, `Font` and
`Document::resources()`. An initial implementation of absence, empty tables and
ownership then ran all 12 tests. Five passed and seven failed on nonempty-table
requirements. With the complete parser the same 12 tests passed, including complete
frozen source and query facts, nonzero Document IDs, max tables, original record owners,
borrowed name slices and generic property replacement read by the independent oracle.
The full standalone suite passed all 47 tests. Its all-target clippy, rustfmt and normal
dependency-tree checks passed. Logs and the preimplementation test snapshot are in
`target/cdx-resources/receiving`.

Two IO tests and an actual CLI test also passed on the prior baseline. These exercise
guarded Unicode Open, unchanged Save and reopen even when a resource query would fail.
The same cases cover unchanged file bytes, refusal of unsupported capture and chemical
authoring, rejection of lossy cross-format saves and recovery from detached page edits.
After integration all 47 standalone tests, both IO tests and the CLI test passed again.
The CLI command inventory remains byte-for-byte identical at 49 entries. The resource
query remains an opt-in library operation.

The first patch application stopped when the filesystem ran out of space. Three affected
paths were restored from the preceding commit. Reclaiming only superseded Cargo test
executables and disabled incremental build cache made space for a clean apply. All five
imported paths matched the tested worker bytes before the integrated run.
