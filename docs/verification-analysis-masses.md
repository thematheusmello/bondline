# Analysis window mass verification

This increment adds relative molecular weight and calculated monoisotopic mass to the native Analysis panel. With a partial selection both are explicitly labelled contributions. Unavailable nuclide data preserve the formula. The registry accepts the same request through the CLI, loopback and MCP. [Independent data and ion conventions](research/molecular-masses.md)

| Receiving path | Observed result |
|---|---|
| Chemistry | 25 tests passed, including four mass tests. RDKit 2026.09.2 independently selected the same predominant nuclide for all 84 naturally weighted elements. Literal reference cases include H₂O, heavy water, an isotope-labelled carbanion, an iron dication, chloride and Tc-99. |
| Command | Two new tests and all eight earlier formula cases passed. The earlier CLI and two new command tests rejected the unknown command before implementation. The combined query retains selection epochs, source coverage, incomplete H, fragment completeness and history. |
| Real CLI and initialized stdio MCP | Three CDX CLI tests and the separate live MCP discovery/selection test passed. Canonical complete hashes protect all 53 existing tool records in both transports. Record 54, `analysis.properties`, is read-only. The existing `analysis.formula` response stays exact. |
| App frames | Four Analysis tests passed. The existing panel failed the new weight assertion before the change. Receipt shows formula, amount, units, scope changes, missing mass data, Undo, and suppression of a gesture through the window with Bond selected. |
| Native macOS Apple Silicon | An isolated, background-only app opened ethanol and propane CDX and selected the middle carbon of propane. The authenticated host returned full-precision results with unchanged inspected state. macOS Accessibility read the panel without foregrounding it. The app approved Close and exited. Windows, Linux and browser visual checks remain open for this increment. |

| Native scope | Formula | Molecular weight | Exact mass shown |
|---|---|---:|---:|
| Whole mixed CDX | C₂H₆O | 46.069 | 46.041865 Da |
| Whole skeletal CDX | C₃H₈ | 44.097 | 44.062600 Da |
| Selected middle carbon | CH₂ | 14.027 | 14.015650 Da |

Local receipt SHA-256 `6f85b86a7c3334ad3c24761e27e9735490c2045c232c99b12d3928c14608ba8b`; unsigned app executable `438b8d2eae520673eed0c250c7cd3e5658d10392ece3cf85fd35af219fcc2717`; CLI `5bb69dbb74f62cd947d633667efda0024ba742dbd2459ea6b6404d1fb7d41eb8`. The focused Mac receipt is in the feature worktree’s ignored `target/mass-check/proof-b615xalu/`. The panel currently shows three decimals for molecular weight and six for exact mass. Configurable decimals, isotope envelopes, m/z, elemental percentages, bracket mass overrides and live captions remain open.

## Replay

Set `BONDLINE_PYTHON` to the absolute path to the project’s RDKit environment. Run the following from the checkout.

```sh
cargo test -p bondline-chem
cargo test -p bondline-commands --test analysis_masses --test analysis_formula
cargo test -p bondline-cli --test analysis_cdx
cargo test -p bondline-cli --test mcp stdio_analysis_handles_show_selection_and_stale_requests
cargo test -p bondline-ui --lib analysis_
cargo xtask licenses
cargo run -p bondline-cli -- run \
  --cmd '{"command":"document.open_cdx","params":{"path":"crates/io/tests/fixtures/cdx-mixed-skeleton/ethanol.cdx"}}' \
  --cmd '{"command":"analysis.properties","params":{"expected_revision":1,"expected_selection_epoch":0}}'
```

The declared native runtime license gate passed for macOS (157 packages), Windows (168) and Linux (223), along with the seven existing asset checks. Relative to the pre-feature checkout, the UI’s normal non-procedural-macro closure adds only MIT elements_rs and MIT strum on each of those platforms and wasm32-unknown-unknown. The existing browser release and license qualifications still apply. [Dependency records](research/molecular-masses.md)

## Combined workspace gate

The complete local `cargo xtask ci` run on `523e0c4`, including PR 28, passed
2,403 workspace tests with zero failures and one ignored test. The separate
macOS AccessKit suite passed all three. Formatting, both Clippy checks,
architecture, licenses, assets and the pinned corpus completed successfully.
The warm checkout receipt `target/analysis-ci/masses-523e0c4.log` has SHA-256
`271045175bb8fbae4585c3b83195a566c4d45fc1285be1823ea44306a9376ef6`.
An earlier run found a stale CLI inventory expected-name list. The fresh gate
includes the complete corrected lists and exact old-wire checks. The merged
movement profile also allows the CH₃O end of ethanol to be selected. Its
molecular weight contribution is 31.034 and its exact-mass contribution
is 31.018390 Da, with the source valence default on one carbon.

A separate exhaustive public Rust API sweep tried mass numbers 1 through 350
on every element (41,300 calls). It returned both metrics for 3,414 tabulated
nuclides and an exact typed missing-isotope result for the remaining 37,886.
The actual sorted positive output SHA-256 is
`9be5a97fca150cca599ea0522536e2869efa85225561f0fb3b04d2b8521c61f8`.
Comparison of these API values with NIST reproduces the published table audit.
