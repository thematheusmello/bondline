# Skeletal CDX drawing verification

On 2026-11-20, the macOS app opened a three-carbon chain at its declared
512 × 512 pt Poster size. Both 80 pt single bonds appeared. Saving to a Unicode
filename and reopening preserved the complete input. The previous executable
opened the ordinary 283-byte file on A4 and refused to capture it. It refused
to open the variant with a dormant visibility flag. The earlier app-frame test
also reproduced an enabled page rotation control on the new read-only view.
The integrated UI disables that control. [Binary sources and scope](research/cdx-skeletal.md).

| Input | Original bytes | Authored H on the three atoms | Native Save As and reopen |
| --- | ---: | --- | --- |
| Explicit H | 283 | 3, 2, 3 | Byte-identical |
| Middle omitted | 277 | 3, absent, 3 | Byte-identical |
| All omitted | 265 | absent, absent, absent | Byte-identical |
| Terminal flag on middle atom | 287 | 3, 2, 3 | Byte-identical, flag retained |

The real CLI, initialized stdio MCP, and background macOS app each performed
the four file journeys. Their CPU and Apple M3 Max Metal captures matched the
independent CDXML drawing at 100% and 200%, before and after reopening. A
separate comparison included the dormant nonterminal flag on an endpoint and
matched all 12 CPU/Metal frame pairs at 100% and 125%. Unit checks exercised all
three atom hit regions, both bond hit regions, four interior black pixels, and
the distance of every one of the 638 black paper pixels from the bond segments.

Independent review added a four-carbon ring with sparse raw IDs and no Element
or H fields. Both the ordinary and dormant-flag sources produced exactly 1,776
black pixels at the independently calculated paper locations. Atomic Save As to
a Unicode path and reopening preserved every byte, all four bonds and all
omitted H fields. RDKit read the saved ring as C4H8 and C1CCC1. An active
flag and a conflicting H count kept their prior error or data-only behavior.
[Cycle fixtures](../crates/io/tests/fixtures/cdx-skeletal-cycle/SOURCES.md).

The native app-frame regression exercised pointer input, the view inspector,
file dialogs and retained drawing tools. The actual native loopback also
verified editing refusals and the earlier direct page-resize history behavior.
A direct resize detaches the model, preventing render and CDX Save until Undo.
The owned background process exited through Close. The desktop catalogue still
has 17 bound entries and 123 planned entries. These results cover automated
macOS input and rendering. New human-approved goldens and Windows/Linux
interactive checks remain open.

Offline Chrome 157 executed the Rust reader and CPU rasterizer in WebAssembly.
All five supported input variants matched every RGBA pixel of the optical
control, kept literal H and preserved source bytes. A straightened chain
with a mandatory label refused display. The pre-existing browser caption and
atom-label checks ran in the same executable. The browser editor, browser file
dialogs and WebGPU rendering require their own verification.

## Reproduction

The fixture generator compares 24 original binary files and one optical CDXML
file to their expected bytes. External RDKit 2026.09.2 checks eight binary
graphs, their hydrogen counts, formula and SMILES. [Inputs and derivation](../crates/io/tests/fixtures/cdx-skeletal/SOURCES.md).

```sh
python3 -B crates/io/tests/fixtures/cdx-skeletal/generate.py
.cache/oracles/format-python/bin/python -B crates/io/tests/fixtures/cdx-skeletal/check_rdkit.py
export BONDLINE_PYTHON="$PWD/.cache/oracles/format-python/bin/python"
cargo test --locked -p bondline-io --test cdx_skeletal --test cdx_skeletal_display --test cdx_skeletal_cycle
cargo test --locked -p bondline-render --test hidden_carbon_atoms
cargo test --locked -p bondline-cli --test sdf_capture_adapters skeletal_cdx_cli
cargo test --locked -p bondline-ui --lib skeletal_cdx_opens
python3 -B xtask/oracles/cdx_caption_browser.py
```

The browser runner takes the Chrome and matching wasm-bindgen executables from
its documented environment settings. Its executable returned zero with
wasm-bindgen 0.2.130. The CDX parser ran 10,000 AddressSanitizer iterations with
236 SHA-256-checked seeds, including the 24 chain and four ring cases. The corpus, local native
receipts and workspace log are under `target/cdx-skeletal-verification`.
The complete desktop binary type-checked for x86-64 MSVC Windows and GNU Linux.
The commands crate also type-checked for those targets and WebAssembly.
The WebAssembly commands check emitted eight existing warnings. A separate
`cargo check --locked --offline -p bondline-ui --lib --target wasm32-unknown-unknown`
failed in mio 1.2.4. The UI pulls in the native MCP transport, which enables
Tokio networking. A browser host needs separation from the native transport
and a browser execution/file path. GitHub Actions stayed off.

The complete local gate on the production and test tree through `00ec56f`
passed 2,363 workspace tests, with one ignored, and all three separate AccessKit
tests. Formatting, warnings-denied Clippy, layering, dependency licenses,
asset verification and the pinned corpus passed.
