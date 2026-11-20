# Moving skeletal CDX atoms with printed labels

Four source-preserving operations passed on independently written methanol, ethanol and amino-alcohol CDX drawings. Before the implementation, the CLI refused Selection on each input. The IO and command acceptance tests independently failed at the missing source grant. The starting CLI could already open the independently computed final CDX and CDXML controls, which produced identical reference frames at 100% and 200%. [Public source and coordinate policy](research/cdx-mixed-skeletal-movement.md) · [Fixture generator](../crates/io/tests/fixtures/cdx-mixed-skeletal-move/generate.py)

| Operation | Atoms moved | Displacement in points | Expected CDX SHA256 prefix |
| --- | --- | --- | --- |
| Hidden carbon in ethanol | Live 2 | +8.25, −5.125 | e9c37099e |
| Printed OH in methanol | Live 2 | −5.125, +3.25 | cf5b9e9d2 |
| Both printed ends and an interior carbon | Live 1, 3, 5 | +8.25, −5.125 | a2ca5c7e1 |
| Different Fragments reusing the same raw Node, Text and Bond IDs | Live 2, 3 | −5.125, +3.25 | c837e8406 |

The IO test checks each complete edited file, its inverse, the original snapshot identity and the complete CDXML paper. Conflicting or stale selection, coincident vertices, clipped label ink, signed-coordinate bounds and detached source/model ownership preserve the current document. The zero-grid edit keeps the redo stack and the capture owner. The labelled and hidden atoms retain the source H fields. The resource table's original Étude λ entry, raw run data, all original record headers and terminators survive. [IO tests](../crates/io/tests/cdx_mixed_skeletal_move.rs) · [Registry and history](../crates/commands/tests/cdx_skeletal_move.rs)

The real egui App-frame test selects the printed oxygen with Marquee, drags it and checks the moving blue glyph ink before the commit. Release creates one edit. The arrow key creates a separate nudge. Undo, redo and Unicode Save As restore exact expected bytes. Alt gestures and an active ring tool retain the source restrictions. [App regression](../crates/ui/src/skeletal_display_tests.rs) · [CLI file regression](../crates/cli/tests/support/cdx_bond_wire.rs)

On macOS, the CLI, initialized stdio MCP server and a background native application completed all four workflows with the same original and saved bytes. Each produced eight CPU and Metal frame pairs that matched the frozen XML references. RDKit 2026.09.2 opened all twelve resulting Unicode files. It recovered CH₄O, C₂H₆O, C₃H₉NO and the two separate CH₄O molecules in the repeated-ID source. The hidden application approved Close and its process exited. Event-level native interaction was exercised through the App harness. The background process received no physical mouse input.

The integrated WebAssembly codec ran offline in Chromium. The 1,733,719-byte module returned zero and matched the independent CDXML reference after a scoped mixed move (SHA-256 `edc3c4b5b804b51ac7b2ec342169e08c77a92c06cfa87dec0c199351770536d4`). The parser AddressSanitizer run completed 10,000 iterations in four seconds with 108 distinct seed files, a final 600-byte mutation limit reached, and zero crash artifacts. The raw oracle translates each selected point and its owned rectangle and compares the complete binary. [Browser check](../crates/io/examples/cdx_caption_browser/skeletal_move.rs) · [Fuzz oracle](../fuzz/fuzz_targets/cdx_parse/skeletal_move.rs)

Workspace Clippy with warnings denied passed on the implemented movement. Local focused tests cover IO, registry, egui and the CLI. The full workspace integration gate is tracked separately. Interactive Windows and Linux checks remain open.

## Replay

```sh
python3 -B crates/io/tests/fixtures/cdx-mixed-skeletal-move/generate.py
cargo test -p bondline-io --test cdx_mixed_skeletal_move
cargo test -p bondline-commands --test cdx_skeletal_move
cargo test -p bondline-ui --lib mixed_skeletal_binary_moves
cargo test -p bondline-cli --test sdf_capture_adapters skeletal_cdx_cli_moves_scoped
python3 -B xtask/oracles/cdx_caption_browser.py
```
