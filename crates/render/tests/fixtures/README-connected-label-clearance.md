# Independent connected-label clearance vectors

`connected-label-clearance.json` is original test data generated independently with
FontTools4.61.1 from the bundled Apache-2.0 Roboto font. It is not a reference-app export,
image or approved golden. The file SHA256 is
`ddc0cca206cc1c66175b2fcb71021d2b9dea6f599476b6ec703f6595e13780f2`;
the font SHA256 is recorded in its header and checked against the existing asset policy.

The 128 cases describe actual glyph placements and independent axial cut intervals for
the original Euclidean butt-strip policy in
[the contract](../../../../docs/research/connected-label-styles.md). The oracle uses bounded
quadratic-to-chord envelopes and analytic segment maxima; the Rust implementation uses
quadratic control hull subdivision. Both account separately for output float32 rounding.
The fixture includes authored and native derived placements; source-labelled authority
is tested separately. Public scene tests use independent raw boxes, not our box generator.

The [independent generator and checks](../../../../xtask/oracles/README-clearance.md)
reproduce the128 semantic vectors, fixed-seed dense controls and Pillow painted spans.
Only elapsed timing metadata differs during regeneration; never overwrite this frozen
fixture automatically. Original research receipts and rejected countermodels remain in
ignored `target/connected-labels/independent/`. Renderer numerical and public-scene tests
consume this frozen fixture without importing the generator.
No exact reference behavior for Roboto, other fonts or formula faces is established.
