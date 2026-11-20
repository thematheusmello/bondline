# Skeletal CDX atom movement

## Reproduce

The three test groups use the existing registry, source-qualified candidate and actual App frame harness. Run with the repository's pinned external Python interpreter.

```sh
python3 -B crates/io/tests/fixtures/cdx-skeletal-move/generate.py
cargo test --locked -p bondline-io --test cdx_skeletal_translation
cargo test --locked -p bondline-commands --test cdx_skeletal_move
cargo test --locked -p bondline-ui --lib skeletal_selection_drag
cargo test --locked -p bondline-cli --test sdf_capture_adapters skeletal_cdx_cli
```

## Results

The preceding release could open all seven initial independently generated source and result files but refused Selection with `unsupported_content`. On the new workflow, five single-fragment inputs retain their literal H values, missing H values and dormant flags. Their actual serialized output matches the independent position-only changes at every byte. One two-fragment input retains repeated child source IDs and long-form length encodings. Two supplemental fixed-point cases place selected atoms on opposite grid parities to test ties-to-even displacement as a rigid translation. [Fixture construction and original inputs](../crates/io/tests/fixtures/cdx-skeletal-move/SOURCES.md)

The App-frame harness clicks Marquee, selects a hidden atom with omitted H, drags it and inspects **both painted bond quadrilaterals during the uncommitted preview**. The two midpoints move while revision, document and history remain unchanged. Release creates one edit. The right arrow adds one point. Undo, Redo and Save As produce the expected bytes, and opening the saved Unicode path returns a clean document.

The real CLI, initialized stdio MCP and an authenticated background macOS host produced six exact files. After reopening them, the three surfaces rendered matching CPU and Apple M3 Max Metal PNGs at 100% and 200%. The pre-change reader rendered each independently predicted output at both zooms. For five single-fragment files, the same frames also match a separately constructed, moved CDXML control. Independent RDKit 2026.09.2 kept the atomic numbers, explicit and total hydrogen counts, directed Single bonds, formula C3H8 and SMILES CCC for all seven fragments. The macOS host accepted a clean Close and exited. Its loopback checks exercised the registry; pointer press, drag, release, and key input were exercised in App frames. Windows and Linux execution and human golden approval remain pending.

The bounded checks also cover an unchanged source snapshot and live redo after valid half-unit no-ops, an exact byte inverse, stale revision and selection epochs, source detachment, coordinate overflow, off-paper placement, and a center moved into a straight line that would require visible carbon ink. Unsupported requests leave chemical data and source bytes intact. [Source and policy](research/cdx-skeletal-movement.md)

## Browser and sanitizer

Offline Chrome executed the Rust writer, reader and CPU renderer. It moved
an atom in an all-H-omitted source and a pair of atoms with repeated raw IDs
in separate fragments, then compared the output to independently constructed
files. It verified byte-exact reversal, retained original source ownership,
and matching CDXML paper at 100% and 200%. Run
`python3 -B xtask/oracles/cdx_caption_browser.py` with the documented matching
wasm-bindgen executable. Version 0.2.130 returned `main=0`.

The CDX parser fuzz target exercised 10,000 AddressSanitizer iterations with
245 immutable SHA-256-verified seeds. For each accepted skeletal source, its
movement oracle writes the expected signed coordinate words independently,
tests alternate selected Node owners, compares a fresh re-import, then checks
both a half-unit no-op and exact reversal.

The first full lint run caught six redundant conversions in the movement test.
Commit `e1a1cd3` compares the same expected bytes directly. The complete
desktop binary type-checked on x86-64 MSVC Windows and GNU Linux.

The full local gate at `e1a1cd3` passed 2,369 workspace tests with one
ignored, then passed the three macOS AccessKit tests, both Clippy checks,
formatting, layer, license, asset and corpus checks. The complete run is in
`target/cdx-move-verification/final-ci.log` in the primary checkout.
