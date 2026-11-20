# Printed labels on hidden CDX atoms

The existing label-creation command changes a hidden carbon into a printed heteroatom on a qualified skeletal CDX page. The source must carry a usable font and colour table. An atom with an existing Text or either explicit carbon-display flag cannot receive a second label through this command. An unqualified page stays source-preserved. The standalone CDX edit writes the shortest header for a new property and preserves each existing property’s length-header width where its new value fits.

Four independently generated binaries check the terminal and interior cases, the first printed label on an all-carbon page, and the last hidden atom on a mixed page. The CLI saves each edit to a Unicode filename. Undo restores the original bytes and Redo restores the expected file. Reopening verifies composition against RDKit and the rendered page against the independent binary. A subsequent edit and failed creation requests preserve history and original data. [Fixtures, generator, values and public sources](../crates/io/tests/fixtures/cdx-skeletal-label-creation/SOURCES.md) · [Native verification](verification-cdx-skeletal-creation-native.md).

## MCP and browser

An initialized stdio MCP client opens the all-carbon propane drawing, creates the first printed label at its middle atom, saves a Unicode-named CDX file and reopens it. Its bytes match the independently constructed dimethyl ether fixture, and its reopened formula is C2H6O. Run `cargo test -p bondline-cli --test sdf_capture_adapters stdio_creates_the_first_printed_atom` to replay it.

In real headless Chromium, the WebAssembly codec created the blue 14 pt nitrogen label on the last hidden carbon of the methanol fixture. Reopening matched the expected complete model, bytes and CPU paper. The resulting Wasm module was 1,751,838 bytes, SHA-256 `82bf17a187b1178fc34758da7f4795371d6e4657b5a274a2415be6a42c7c2226`. Run `python3 xtask/oracles/cdx_caption_browser.py` with wasm-bindgen-cli 0.2.130 and a Chromium browser. Pointer-driven authoring in the browser needs a separate check.

## Parser and edit sanitizer

AddressSanitizer ran 10,000 inputs using the pinned cargo-fuzz 0.13.2 and Rust nightly 2026-04-01. All 360 original seed SHA-256 hashes were identical before and after. The reached input limit was 45,173 bytes, and the run finished in two seconds with zero crash artifacts. The harness verifies every retained record outside the two editable Node scalar properties and the inserted child. Each successful edit reparses into the exact same model with an editable labelled atom. A fixed eligible source also must produce the full expected binary. The log SHA-256 is `06604d22bd86d99c2c0342dc83f4803cdb6aef1d391f376d85d84921e22e1876`.

```sh
cargo test --locked -p bondline-cdx
cargo test --locked -p bondline-commands --test cdx_skeletal_move
cargo test --locked -p bondline-cli --test cdx_skeletal_creation
```

Set `BONDLINE_PYTHON` to the pinned RDKit interpreter. The full local integration gate and native Windows/Linux interaction require their own evidence.
