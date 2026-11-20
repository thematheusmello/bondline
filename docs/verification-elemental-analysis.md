# Elemental analysis verification

The native Analysis window includes a selectable text table for the current drawing or selection. Columns identify each element, its contribution to relative molecular weight, and its percent by weight. The table scrolls independently when the drawing contains many elements. An unavailable denominator replaces the table with the precise missing-data reason. [Calculation and sources](research/elemental-analysis.md)

| Scope | Rows in atomic-number order, percent by weight |
|---|---|
| Ethanol drawing | H 13.13%, C 52.14%, O 34.73% |
| CH₃O end selected | H 9.74%, C 38.70%, O 51.55% |
| Water containing one deuterium | H 15.89%, O 84.11% |
| Ammonium ion | H 22.35%, N 77.65% |

The ethanol and selected-CH₂ CLI results agree with a separate RDKit process that adds physical H nodes, groups atomic masses by element and computes fractions with Python Decimal. Fixed reference cases independently cover natural H combined with deuterium, carbon-free ions and unavailable weight/isotope data. A synthetic drawing reaches all 118 element rows once, in order, while retaining carbon-13 in the carbon row. The Rust tests require unrounded row totals to agree with the molecular weight and percentage totals to agree with 100 within 1e-9.

The old binary returned `invalid_params` for `include_elemental_analysis`. The command test failed on this new option before implementation. The old panel failed the new visible table assertion. Four real CLI tests, initialized MCP discovery and calls, three elemental chemistry tests, the bounded read-only command workflow, existing formula/mass regressions and four Analysis App-frame tests passed. Omitted and false options retain identical complete wire results. Numeric labels, stale requests, missing/empty scopes, Undo, scan limits and the existing source/selection guards remain in effect. No new runtime package or command was added. [Wire contract](control-protocol.md)

## Native macOS

An isolated background-only macOS process opened original mixed-CDX ethanol. The authenticated controller opened its Analysis window and macOS Accessibility saw the complete selectable table. Selecting atoms 2 and 3 changed the table to the CH₃O contribution with the selection-specific heading. The app saved to `elemental-β.cdx`, reopened the byte-identical copy, and displayed the original whole-drawing table. The inspection before and after each read was identical. The process closed normally. Windows, Linux and browser visual behavior remain unverified for this increment.

Receipt SHA-256 `c0ef59b14cfb31210b21faba3f61b84d5514c3533f8f3f86ccd78bbd6d331db8`; unsigned executable `b9997e30388fb3f3e27290916b8d0583924b2438d6ccfcb5aaa7935c95529604`. The local receipt directory is `target/elemental-check/proof-qws32v0x` in the elemental-analysis checkout.

## Replay

Use the project’s pinned RDKit interpreter as `BONDLINE_PYTHON`.

```sh
cargo test -p bondline-chem --test elemental_analysis --test molecular_masses
cargo test -p bondline-commands --test elemental_analysis --test analysis_masses --test analysis_formula
cargo test -p bondline-cli --test analysis_cdx
cargo test -p bondline-cli --test mcp stdio_analysis_handles_show_selection_and_stale_requests
cargo test -p bondline-ui --lib analysis_
cargo run -p bondline-cli -- run \
  --cmd '{"command":"document.open_cdx","params":{"path":"crates/io/tests/fixtures/cdx-mixed-skeleton/ethanol.cdx"}}' \
  --cmd '{"command":"analysis.properties","params":{"expected_revision":1,"expected_selection_epoch":0,"include_elemental_analysis":true}}'
```

## V2000 and V3000 MOL

The combined source-aware hydrogen workflow also passes elemental analysis for six
independently generated CTfiles. Each format covers ethanol, isotope-labelled ethanol
and ammonium. A separate RDKit full graph supplies atoms and attached H. Tests compare
every element identity, weight and percentage for the complete structures and both
ethanol selections, then save each exact source to a Unicode path and repeat on reopen.
Before the source-aware MOL increment the same V2000 ethanol received
`unresolved_hydrogens` on atom 1. The combined test is
`cargo test -p bondline-cli --test elemental_mol`.

The isolated macOS app also opened both ethanol MOL editions, selected their CH₃O
end, saved and reopened exact copies and displayed the whole and partial tables through
its own accessibility tree. The integrated receipt SHA-256 is
`85bf0521879d1674906a93e4fb5172c2667018a4d3a9ebfea6a6d1e14174ce73`.

The App-frame test `elemental_table_copies_the_displayed_rows_without_changing_the_drawing`
drags from the table heading to its last percentage, delivers a normal Copy event and
checks that the outgoing clipboard text exactly equals the displayed table. The document,
selection and history inspection before and after the gesture is identical. Native
Accessibility establishes visibility separately from this clipboard-output test.
