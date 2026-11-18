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

## Sanitizer verification

The [resource fuzz recipe](../fuzz/README.md#cdx-font-and-colour-resources) ran 10,000
AddressSanitizer cases in 2 seconds on macOS with nightly-2026-04-01. It finished at
coverage 1639, feature count 3419 and RSS 373 MiB. The run started with 85 independently
checked immutable sources. The maximum source, final fuzzer limit and largest
recorded NEW/REDUCE input were each 393,481 bytes under the configured 524,288-byte
cap. All original sources and their copies passed length and SHA-256 checks after
the run.

Seed manifest SHA-256 is
`2698d4cf1150cb2190c3de590778380459d3792a999c42438e872d6ee3adf899`.
The 26,298-byte ASan log in `target/cdx-resources/resume.lQ1sz7/asan.log` has SHA-256
`6bccf47698deb8e19b7f8e50cb020e1f499117efce9e7c780eeecb4076dfc34f`.
The new assertions verify source records, resource queries and exact raw byte
preservation. They do not test font rendering or caption interpretation.

## Workspace acceptance

The first resumed run reached the existing external CDX checks and failed five
tests because `BONDLINE_PYTHON` was unset. The next run used the absolute pinned
interpreter with RDKit 2026.09.2, Pillow 12.3.0 and NumPy 2.5.4. It passed
`cargo xtask ci` with 2,202 workspace tests passed, zero failed and one ignored
across 323 reports, plus three macOS AccessKit tests. Formatting, warnings-denied
workspace and AccessKit clippy, layer checks, runtime license checks, seven bundled
assets and the eight-file corpus passed. The fuzz workspace formatting check also
passed. No test assertions or listener behavior changed between these runs.

The successful 275,882-byte log at
`target/cdx-resources/resume.lQ1sz7/ci-qualified.log` has SHA-256
`47d75c8e8044e6829b8bf2859de1176404c573d19e3904baf163be839aac897d`.
The pre-pause aggregate was interrupted and does not contribute to this result.
Native resource display and new Windows, Linux and WebAssembly runs remain unverified.
