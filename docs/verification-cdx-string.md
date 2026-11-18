# Raw CDXString verification

`bondline_cdx::Document::string(&Record)` borrows the literal text remainder
and exposes exact-size iteration over five raw UINT16 words per style run.
The query checks an owned property handle and bounds the run block to that
property. The [contract](research/cdx-string.md) records the public sources,
the raw data boundaries and the unresolved display semantics.

The [original fixture set](../crates/cdx/tests/fixtures/raw-strings/SOURCES.md)
reproduced all 32 inputs and independent facts byte-for-byte. It covers 25
successful raw queries and seven invalid prefixes across five framing profiles.
Both maximum-count cases check all 65,535 entries. A separate long-text case holds
65,553 bytes. An invalid prefix followed by sufficient bytes in the next property
proves that the query respects the selected property's end.

The four integration tests were frozen before the producer. The initial library
check passed. The new integration target then failed to compile at missing
`StringError`, `StyleRun` and `Document::string()`. This was a new-API compile
probe. After implementation, the same four tests passed at runtime. They
compare complete independent facts, borrowed source pointers, foreign and
nonproperty record refusals, and actual bytes from the existing property writer.
The first aggregate stopped in clippy at three test-only `unwrap_err` assertions.
The follow-up uses explicit error comparisons and keeps the same expected errors.
The qualified `cargo xtask ci` passed 2,206 workspace tests, zero failures and
one ignored across 324 reports, plus three AccessKit tests. Formatting,
warnings-denied clippy, layer, license, asset and corpus checks all passed.
The standalone crate still has zero runtime dependencies. Its manifest and
lockfile stayed unchanged.

The standalone suite passed all 51 tests. With Rust 1.94.1,
`cargo check -p bondline-cdx --lib --target <target> --offline --locked`
passed for x86_64 Windows MSVC, x86_64 Linux GNU and wasm32-unknown-unknown.
These are library build checks. Application execution on those platforms
remains open.

AddressSanitizer ran [32 pinned original inputs](../fuzz/README.md#raw-cdxstring-records)
through 10,000 iterations with nightly-2026-04-01 on macOS. The run took 2 seconds
and finished at coverage 766, feature count 1508 and RSS 392 MiB. The initial
largest seed was 655,466 bytes, the final libFuzzer limit was 655,463 bytes and
the largest recorded NEW/REDUCE input was 623,179 bytes. The configured ceiling
was 1,048,576 bytes. Both immutable sources and the original corpus copies
passed the manifest length and hash checks afterward.

The 22,974-byte log at
`target/cdx-resources/next-text/raw-query.HoJhSI/asan.log` has SHA-256
`94920c6f7046e0ac13b36e5480a49c62889b306f9a962319a5e9785932c28fc8`.
The API and fuzzer inspect raw property data. Direct-Page caption preview and
native acceptance are the next application workflow.

The successful 277,886-byte local aggregate log at
`target/cdx-resources/next-text/raw-query.HoJhSI/ci-qualified.log` has SHA-256
`ec944792e8066b22831402913c5d18fbd36f82b030e8f0f1422fc4d4f9f7b1d4`.
The pre-producer compiler probe, frozen test hashes, focused run and
first lint diagnostics are saved in the same directory.
