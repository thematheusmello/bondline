# Verification of mixed skeletal CDX drawings

The pre-change CLI at `98f3edd` refused a CPU capture of all 12 inputs. The positive methanol Rust case initially failed during Open at direct child Text record 21. In the same checkout the existing CDXML workflow rendered methanol, ethanol and 3-aminopropan-1-ol at 100% and 200%. These frames were frozen before changing the display code. [Fixture generator and chemicals](../crates/io/tests/fixtures/cdx-mixed-skeleton/SOURCES.md)

Four original binary drawings now Open with the authored paper, hidden carbon vertices and printed chemical labels. The independent RDKit reader obtained CH₄O, C₂H₆O and C₃H₉NO and their expected connectivity. Literal and omitted carbon H have different binary encodings and the two methanol files produce identical paper. The model preserves each absence, each local dormant flag, every direct label and every bond. The existing data-only reader retains the unsupported label Oη and its source bytes. The control suite also exercises a conflicting box, touching geometry, missing printed H and unsupported visibility. [IO acceptance](../crates/io/tests/cdx_mixed_skeleton.rs)

The Rust test compares every CPU pixel with its independently authored CDXML control at both scales. It checks all atom anchors and bond midpoints, the measured blue glyph ink and black rail, and the incident strokes against independent FontTools curve envelopes. At the methanol OH end the independent cut is 5.84557648–5.84557987 points. A detached page loses display and exact Save permission. The separate all-carbon movement path refuses this mixed paper. [Pinned intervals](../crates/io/tests/fixtures/cdx-mixed-skeleton/clearance.json)

An actual Apple Silicon CLI, initialized stdio MCP and an authenticated background macOS process opened the binary inputs, saved to Unicode filenames and reopened the exact 314, 316, 370 and 555 source bytes. Each adapter produced eight CPU/Metal PNG pairs across the four source files and two magnifications. Every pair matched the frozen CDXML frame. The MCP and native runs refused Selection, caption creation and printed-label edits. The macOS application approved Close and its process exited. An initial verification script expected a disabled Selection record where the registry omits the unavailable control. The corrected host test checks command refusal and compares the result with the unchanged document. The egui App test exercises retained Selection, canvas drags, Alt clicks and native Save As using input events delivered to its harness. The background process received no physical pointer input. [App regression](../crates/ui/src/skeletal_display_tests.rs)

The drawing grant is read-only, bound to the exact model and CDX source. These checks cover the stated corpus and this macOS host. No new human-approved golden or Windows/Linux desktop execution is claimed.

## Replay

```sh
cargo test --locked -p bondline-io --test cdx_mixed_skeleton
cargo test --locked -p bondline-ui --lib mixed_skeletal_binary
python3 -B xtask/oracles/cdx_caption_browser.py
```

The browser example reuses the same filesystem-independent IO and CPU rendering APIs. The CDX parser fuzz oracle checks literal child ownership, styled text and boxes, local implied flags, retained H and exact source bytes for admitted mixed papers. [Browser check](../crates/io/examples/cdx_caption_browser/skeletal.rs) · [Fuzz target](../fuzz/fuzz_targets/cdx_parse.rs)

On the isolated macOS checkout the offline Chromium run returned `main=0` with wasm-bindgen 0.2.130. The 1,705,630-byte module had SHA256 `e09b95d837bc927ade4ebeb33f9632d284f31228a963e627105aa60e894e621d`. The mixed and previous CDX sources supplied 103 distinct SHA256-pinned seed files. The AddressSanitizer run completed 10,000 iterations with seed 20261123 and an input bound of 65,536 bytes. The corpus directory used disposable copies. This run produced no crash artifacts.

## Integrated checkout

Starting from the main tree at `d68390c`, the previous CLI returned
`unsupported_content` for all four positive files. With the viewer integrated,
each adapter reopened the saved file and reproduced the frozen rendering.
The background-only macOS host exited through the registry after 53 successful
requests. Its receipt SHA-256 is
`9af244ae345260c91cfb4f3159944168949f3d8102e9efbf1930c859d1fc3971`.
The automated CLI regression also compares the reopened ethanol capture with
the XML control. [CLI test](../crates/cli/tests/support/cdx_bond_wire.rs)

The combined WebAssembly module ran offline in Chromium with main returning
zero (1,725,062 bytes, SHA-256
`53d6eeb3687bb38a51e9f824c79232a20630ae5f72d90a9cc6a83625e84ad587`).
The integrated AddressSanitizer run preserved all 257 input hashes, completed
10,000 iterations with seed 20261124 and produced zero crash artifacts. The
primary checkout retains these receipts under `target/mixed-skeleton-integration`.

The final `cargo xtask ci` run at `a0269d2` passed 2,373 workspace tests
with one ignored, all three macOS AccessKit tests, and the formatting, Clippy,
layer, license, asset and corpus checks.
