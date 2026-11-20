# Analysis of MOL structures

The old CLI reported unresolved H at atom 1 on the original V3000 ethanol
file. The independent RDKit reader returned `C2H6O`. The existing Marquee
then rejected selection on the same molecule. Replaying both operations
first exposed each missing behavior.

The differential oracle opens 37 V2000/V3000 inputs in the application
registry through the actual CLI. It compares each formula with RDKit
2026.09.2, invokes Save As to a Unicode path, compares the complete source
bytes and reopens the result. It verifies the number of omitted H counts
in the model and the absence of history or dirty state after querying.
Inputs include the pinned methyl radical, the noncontiguous-ID V3000
ethane, both individual MOL blocks in the ordered SDF fixture, isotope-
labelled ethanol and water, multiple bonds, rings, ions, sulfur, explicit
zero, an unmarked doublet and ethanol with coincident coordinates. Carbon-free
borane uses the documented Hill/RDKit ordering qualification. In a separate
ethanol plus iron file, the whole report identifies the ambiguous atom
and the selected oxygen gives `HO`. [Reproducible oracle](../xtask/oracles/mol_formula.py) · [Source and scope](research/mol-analysis.md).

```sh
cargo test --locked -p bondline-cli --test analysis_mol
cargo test --locked -p bondline-cli --test mcp stdio_mol_analysis
cargo test --locked -p bondline-commands --test selection_authoring
cargo test --locked -p bondline-ui --lib complete_foreign_and_collection_sources
```

Point `BONDLINE_PYTHON` at the project's pinned RDKit interpreter. Initialized
MCP read selected `CH3O`, refused movement, preserved the source and wrote the
exact file. Actual egui frames selected an atom by its painted position, dragged it, then selected both atoms and exercised
the arrow key. The original positions and
source survived. A background macOS process opened and reopened three MOL
files and exposed `C2H6O`, `CH5N`, `CH3` and selected `CH3O` in the
native accessibility tree. The window distinguished recorded and calculated
hydrogen counts. Windows/Linux desktop interaction and browser MOL file
handling still need independent receiving runs.

The macOS run closed its own process through the registry. Its final receipt
SHA-256 is `20b14f7d2b5b35a1d5f6b9fe5799a90417441dc78117c4d5b755ec87475e3591`.

With the shared mass query integrated, the same 37 sources and exact reopened
copies also pass `analysis.properties` against RDKit `MolWt` and
`CalcExactMolWt`. Both queries must return identical formula and scope data.
Absolute tolerances fixed before comparison are 0.010 for molecular weight
and 0.000001 Da for monoisotopic mass on these small molecules. Conventional
weight tables have different editions. The separate chemistry table audit
covers the underlying values and their stated uncertainty.
[Mass validation](verification-analysis-masses.md).

The integrated macOS window displayed V3000 ethanol at 46.069 and
46.041865 Da and the selected carbon–oxygen contribution at 31.034 and
31.018390 Da. Methylamine and methyl radical also survived an exact Save
and native reopen with both mass rows. The accessibility and file receipt
SHA-256 is `0907d53cb8f7018157284cbca388c374e3566a6fec82e05a57ab6f3cf1330949`.

The integrated local `cargo xtask ci` passed on `7c73558`: 2,406 workspace
tests passed, zero failed and one was ignored. Three additional macOS
AccessKit tests passed. The complete log SHA-256 is
`0c007e693cb537a88950b0317194a8e307976c3109846c9500f85a15e2635f53`.
An earlier run exposed a drawing-settings test that still expected 23
controls for MOL. Its replacement verifies all 26 and explicitly checks
Selection, Select All and Deselect All on MOL and their absence on the
unsupported CDX, SDF and SMILES sources in the same test. Existing form
guards and request-ID assertions still pass.
