# First caption on an empty CDX page

Base `c1bdbdd`, 2026-11-19. The public contract keeps the existing
closed caption-only layout, with explicit Poster dimensions and the
same white paper and UTF-8 Roboto resources as the writer. Its first
caption uses explicit signed depth zero. [Contract](research/cdx-empty-caption.md).

Independent Python replay matches 37 original CDX binaries and their
manifest. The six positive triples exercise all five framings and font
selector 1 with a white palette after Page. They pin the complete model,
the matched Page end, the exact 112-byte insertion and unchanged source.
The 19 controls retain their earlier Open outcomes and standalone raw
bytes. [Witnesses and replay](../crates/io/tests/fixtures/cdx-caption-empty/SOURCES.md).

The baseline CLI opened the six candidate files at caller A4 and
refused `text.create`. It also confirmed the 18 data-only and one refused
control Open outcomes. The receiving IO test failed to certify the first
empty Poster as a caption page. Its unsupported-source test passed. Run
receipts are under `target/cdx-empty-caption-preparation`. The IO RED
SHA-256 is
`10b33b29dad0026717646ddddcd63007cc5b3f345cbfb64c0fc7cb0c20da6942`.

## Receiving checks, 2026-11-20

The pre-change command refused the first creation with
`UnsupportedCaptionCreationSource`. The pre-change App showed A4 where the fixture
requires 512.25 × 384.5 pt. The first draft of the independent JSON expectation
used an explicit empty text list. The existing model omits the key. Correcting the
manifest left all 37 CDX binaries identical and the size assertion still failed
before the product change. The generator now reproduces the corrected manifest
with SHA-256 `be9dd2b68fee0967ed40f2107eeca041beef9ff2c0fe2bb03d43d6eaaed33629`.

The focused suites passed 30 IO, seven command and 14 App tests. The IO test
compares the *entire* model and file for both possible first creations in
each framing. A blank page starts at live ID 1. Creation after Undo uses live
ID 2, raw object ID 91 and the same explicit Z zero. The old signed maximum
plus one rule still applies to populated pages. Reopening the new file keeps
the exact ID. The App test uses the chooser, paper click, draft, Apply, Undo,
Redo, branch, Unicode Save As, reopen and a click on the saved glyphs.
[IO tests](../crates/io/tests/cdx_empty_caption.rs), [command tests](../crates/commands/tests/cdx_caption_creation.rs), [App tests](../crates/ui/src/native_caption_creation_tests.rs).

The real CLI saved six exact branched files, 199–222 bytes. Stdio MCP and the
production authenticated loopback server each exercised all six sources,
compared both saved outputs and edited the reopened caption. All 24 outputs
matched the independent fixture bytes. At 512 × 384, CPU and Apple M3 Max
Metal produced the same 5015-byte PNG, SHA-256
`d82d0e66a0c9fec71e9b52d56da4adcae312fe47c075f81f8fb105abd5a12317`.
The pixel difference from blank paper has half-open bounds (164,205)–(215,214).
The local adapter replay and results are in `adapter-nov20` within the receipt
directory above. All 51 registry entries match the integrated
caption-creation branch at `c1bdbdd`.

On macOS, the visible isolated app opened the 87-byte Unicode-named drawing at
its authored size. A blank-paper click with Text opened the draft at its actual
mouse baseline. That process ended before a physical Apply was recorded. A
fresh run of the signed desktop executable stayed hidden while the computer
was in use. Through its native host, Select, Open New, Apply, Undo, Redo, Undo
and Apply produced the expected branched state. Saving to `First Caption β.cdx`
and reopening preserved all 199 bytes and the source drawing. Opening live ID
2 for editing returned the original body. Cancel kept the file clean. The
host's captures matched the PNG above and Close returned approved. The native
receipt is `native-nov20/run-fku_f9l5/verified.json`. Physical keystrokes and
Save As for this empty-page variant have App-frame coverage.

The first ASan run stopped on an assertion in the fuzz oracle. The blank
fixture can pass both the raw elemental scan and the exact caption preview
proof. The revised invariant checks the empty model and fragments and the
absence of atom-label authority. Labelled and data-only invariants still run.
The failing input matched the immutable 87-byte source exactly. The qualified
run passed 10,000 cases in two seconds with seed 721533879 and all 103 input
hashes unchanged. The largest reported new/reduced input was 812 bytes.
The commands crate type-checked for Windows MSVC, Linux GNU and wasm32. Wasm
emitted eight warnings. Desktop execution on Windows and Linux and browser
input remain unverified.

To replay the committed checks from the repository root, run

```sh
python3 -B crates/io/tests/fixtures/cdx-caption-empty/generate.py
cargo test --locked -p bondline-io --test cdx_empty_caption
cargo test --locked -p bondline-commands --test cdx_caption_creation
cargo test --locked -p bondline-ui empty_cdx_paper_creates_its_first_caption
BONDLINE_PYTHON="$PWD/.cache/oracles/format-python/bin/python" cargo xtask ci
```


The local `cargo xtask ci` run passed 2,324 workspace tests with one ignored,
plus all three AccessKit tests. Formatting, warnings-denied Clippy, layers,
licenses, assets and the pinned corpus passed. After adding explicit A4,
empty-model, display and conversion checks for the 18 fallback files, the
affected IO suite and warnings-denied Clippy also passed separately. The full
log SHA-256 is `5b51fdf255d4d42d25f49accb5337de110b6d220d1f9c7105825fcbe50ce877f`.
GitHub Actions stayed disabled.
