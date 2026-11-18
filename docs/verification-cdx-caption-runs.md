# Explicit CDX caption run verification

The [run contract](research/cdx-caption-runs.md) extends the read-only
[ordered caption workflow](verification-cdx-caption-order.md) with literal
styles whose starts and glyph joins are unambiguous. Stored point sizes,
colours, text and run boundaries survive Open, capture and exact CDX Save.

## Runtime RED and GREEN

The [25 original cases](../crates/io/tests/fixtures/cdx-caption/runs/cases.json)
were pinned in `e6123fe`. The intact pre-run CLI refused every case. Three
new IO tests failed at runtime, while the new refusal test and eight earlier
preview tests passed. The old `two_runs.cdx` gains a complete positive model,
an exact pixel comparison with its one-run counterpart and exact source
checks. The other 62 original refusals, 13 ordering refusals and 20 new
refusals preserve their earlier code and detail.

FontTools initially disproved a positive test's assumed zero space–T
adjustment. The frozen witnesses use a zero space–H join and keep space–T
as a negative. This happened before production. The first producer run
exposed JSON integer versus float size representation in the new manifest.
The final manifest expresses the same numbers as the public model's floats.
Original binaries, raw field facts and the independent numeric expectations
remain exact. Both IO targets then passed all 12 tests.

## Adapter and visual evidence

The real CLI verified CPU and Metal equality and exact Unicode Save/reopen.
The three-run witness has red 24 pt `AV`, a 16 pt blank that advances the pen,
and blue 32 pt `Hg`. Independent supersampled Pillow/FreeType using the
already frozen font metrics measured colour intersection-over-union of
0.9716312057 and 0.9871904355. The shared 1024 × 1024 RGBA SHA-256 is
`c3a2e4aa7237e836908f3c2133d1c9c1565b5fcbdf320e26097015482a52293e`.
The actual saved file's independent struct check compared the same tuples
after JSON array conversion. The 169 bytes were equal before this verifier
representation correction.

Initialized stdio MCP opened the same source and captured those pixels.
It refused `text.set_run` with exact unchanged state, then refused
capture and Save after a detached page resize. Undo restored the exact
model, pixels and bytes. Unicode Save and reopen repeated all checks.
The actual CLI command list remains byte-identical at 49 records.

## Native macOS review

The signed independent review app had executable SHA-256
`13f472315efa765df105765a19a1c6f42e86f364af43c02822ffefbde8da2d94`.
It opened after the hardware was idle. Cyclooctane was selected before Open.
The actual canvas showed both colours and sizes in one read-only caption.
Option-click on the text and a paper drag left the complete inspected state
byte-identical.

Known Save advanced the controlled mtime to `1795036708263086500` ns and
retained every byte. Save As wrote `styled-再開-λ.cdx`. The complete model
also displayed the `λΩé` tail from the independent Unicode example. The
ambiguous source gave the old error and Cancel recovered the current document.
The saved copy reopened with the three original runs, clean state, empty
history and the same Metal pixels. The two native files match SHA-256
`7566ff6b374b51dc19c26d55426be04da1320a75a80bbb5f6c68d6c40604def3`.
Independent parsing confirms their raw starts and styles. The owned app
exited. CJK filename glyphs still show placeholders in some UI labels.

All local receipts are under `target/cdx-resources/next-text/caption-runs.aTZunB`.
`native/native-evidence/verified.json` has SHA-256
`2e6d84dbfa98163d4e12615cf163de6ec5a8614eff932c68d08230d5070ea67f`.
Physical density variation and Windows, Linux and browser interaction remain
unverified.

## Sanitizer and platform checks

ASan passed 10,000 runs in two seconds. The harness compares recovered text
and every run start with the original property data, in addition to the
existing detached-edit and Save invariants. All 117 immutable originals
verify by hash. The largest seed was 45,177 bytes. The largest recorded
mutation was 346 bytes of the configured 65,536-byte cap.
`fuzz.log` has SHA-256
`0b7614d2cb4428d5ad39cd5f2e134f37e7fa5d1f000216377ea48fd9186eab3e`.

The IO library type-checks for Windows MSVC, Linux GNU and WebAssembly with
the pinned Rust toolchain.

## Final local gate

`cargo xtask ci` passed **2,223/0/1** workspace tests in 327 reports and
**3/0** AccessKit tests. Formatting, warnings-denied clippy, layers, licenses,
assets and the pinned corpus passed. The 279,927-byte `ci.log` has SHA-256
`98ad22a4585d22af970970ca49a1a0c53a5a34aa331dc90cc51d801dbea82f98`.
The executed tracked diff from `e6123fe` has SHA-256
`0d18482f2642951ba7de042c43d570c5e7025673da9fc45206f833482f6c571b`
with the new IO test separately checked by `new-tests.sha256`. The final
product and test files match before commit. The historical MCP listener
failure remains unexplained.
