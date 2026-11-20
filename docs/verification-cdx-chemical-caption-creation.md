# Adding a caption to an imported chemical drawing

Verified on 2026-11-20. The two original CDX inputs contain two fragments
with independently boxed atom labels. One page has only chemistry. The other
also has two direct captions, including one with the same raw ID as both atom
labels. Before this change, the CLI and initialized MCP server could open both
sources and refused `text.create`. [Source contract](research/cdx-chemical-caption-creation.md),
[original fixtures and generator](../crates/io/tests/fixtures/cdx-chemical-caption-create/SOURCES.md).

| Starting drawing | Original bytes | Bytes after adding “Phase β & q” | Paint depth | Live ID, first / after Undo |
| --- | ---: | ---: | ---: | --- |
| Chemical only | 320 | 432 | 1 | 1 / 2 |
| Chemistry and two captions | 487 | 600 | 8 | 20 / 21 |

Both edits place direct Page Text raw ID 903 at (144.25, 216.75) pt. The
independent reader checks the signed coordinates, font, style, parent, paint
depth and persistent ID. Removing the inserted subtree from any of the four
outputs recovers every byte of its original. Earlier caption-run and atom
label edits survive an append. Insufficient IDs, exhausted signed depth,
detached models and contradictory ink bounds refuse the edit.
[IO tests](../crates/io/tests/cdx_caption_creation.rs),
[command and Undo tests](../crates/commands/tests/cdx_caption_creation.rs).

The native App tests select Text and click blank paper, apply the draft,
Undo, create again, save to a Unicode path, reopen, and pick the new caption.
A click on a painted existing caption edits its run. A chemical hit keeps the
document and history intact. The real macOS executable separately exercised
the same registry over its authenticated loopback interface, including dirty
Close/Cancel, Unicode Save As, reopen, and an approved clean exit. It ran in
the background. Physical mouse and keyboard input on macOS were outside this
run. [App-frame tests](../crates/ui/src/native_caption_creation_tests.rs),
[real CLI regression](../crates/cli/tests/support/cdx_caption_wire.rs).

The CLI, initialized stdio MCP and native host agreed with renders of the
independent expected files made by the pre-change renderer. CPU and Apple
M3 Max Metal PNGs matched at 100% and 200% for both drawings. A second
placement of “Note β” at (300.25, 400.75) pt produced the independently
predicted two 595-byte files with live IDs 20 and 21. The native receipt
is `target/cdx-chemical-caption-creation/native/host-e7qm3bsp/verified.json`,
SHA-256 `de4bb339d56875bde27b0bd77381a861adacaeeb2f17e529bdd061d2ae36a524`.

AddressSanitizer completed 10,000 trials from 142 unchanged binary seeds.
The oracle checked exact ownership of the insertion, the full model, the
new raw ID and depth, and exact original bytes outside the new subtree.
The final reported RSS was 316 MiB. The log SHA-256 is
`7a64522b328e00f9182a7be5f20cfeda8421980313dfd9ecea49e0a1e18c3ca0`.
[Reproduction](../fuzz/README.md#creating-captions-above-chemical-structures).

## Browser execution

The Rust IO code ran in Chrome 157 on macOS using the release
`wasm32-unknown-unknown` build and wasm-bindgen 0.2.130. The reproducible
example opens, annotates, exports and reopens both drawings, compares the
four complete files, checks the old graph and caption IDs, and checks the
specific refusal on raw-ID exhaustion. The browser returned `main=0`.
A build with an intentionally incorrect expected file returned `main=1`.
The 1,189,667-byte Wasm module had SHA-256
`e2d3289c5e8709c372df40d93be39f5a33b084678ce96d86c9e3383f356a7879`.
[Rust example](../crates/io/examples/cdx_caption_browser.rs),
[offline runner](../xtask/oracles/cdx_caption_browser.py).

A separate offline browser invocation read the four actual output buffers
from Wasm memory and compared them with the pinned files on disk. The
browser example supplies sparse IDs directly. Native command tests exercise
Undo and Redo. Browser UI, file pickers, browser GPU, and Windows/Linux
desktop execution remain open. Windows MSVC, Linux GNU and wasm32 all
passed the commands-crate type check.

The final local `cargo xtask ci` passed all 2,343 workspace tests with one
ignored, all three AccessKit tests and the formatting, warnings-denied clippy,
layer, license, asset and corpus checks. Log SHA-256
`2aa4caac27397b03f73026e766f743501f65aef4e8f068cf55408ddbd2c8897b`.
An earlier run exposed an obsolete Text-selection refusal on a chemical
page. The updated test verifies opening and cancelling a new draft on that
page, preserves the missing-run error, and checks the full previous refusal
on a contradictory chemical drawing. Another run reached the intermittent
MCP replacement-bind failure on port 64219. The unchanged listener test
passed on the complete rerun. The port's owner at failure remains unknown.

```sh
python3 -B crates/io/tests/fixtures/cdx-chemical-caption-create/generate.py
cargo test --locked -p bondline-io --test cdx_caption_creation
cargo test --locked -p bondline-commands --test cdx_caption_creation
cargo test --locked -p bondline-ui --lib native_caption_tests
cargo test --locked -p bondline-cli --test sdf_capture_adapters cli_adds_a_caption_above_the_chemical_graph
python3 xtask/oracles/cdx_caption_browser.py
```
