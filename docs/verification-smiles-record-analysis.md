# Inspect one molecule in an open SMILES file

Open a multi-record `.smi` file and choose **View → Show Analysis Window**. The record number selects a molecule for formula, molecular weight, exact mass and elemental percentages. The panel also reports its drawn atoms and actual connected components. **Previous molecule** and **Next molecule** move one record at a time. Opening another file resets the selector to its first molecule.

| Pinned source | Formula | Drawn atoms | Components | Molecular weight | Exact mass (Da) |
| --- | --- | ---: | ---: | ---: | ---: |
| Isotopic chain | C[13C]H6O | 3 | 1 | 47.061 | 47.045220 |
| Two separate carbon rings | C7H14 | 7 | 2 | 98.189 | 98.109550 |
| Branched sodium salt | C4H9NaO | 6 | 2 | 96.105 | 96.055109 |

A query returns no formula for a complete raw collection, an aromatic atom, an aromatic bond or an explicit isotope zero. A raw-record query on a drawing returns `unsupported_content`, including when its file association still points to SMILES. Numbered isotopes, brackets, signed charge, normal valences and ring closures follow the [pinned OpenSMILES rules](research/smiles-record-analysis.md).

## Reproduce

Set `BONDLINE_PYTHON` to the interpreter installed by the [corpus setup](corpus/README.md). The CLI test writes a ten-record file with CRLF and opaque per-record metadata, reopens its Unicode Save As, and checks both copies using RDKit 2026.09.2. Set `include_elemental_analysis` on `analysis.properties` to read percent-by-weight rows at full precision. The CLI test compares seven molecular graphs, three unresolved records and six error conditions. It also checks the raw-file restriction after a successful drawing import.

```sh
cargo test -p bondline-chem --test opensmiles_valence
cargo test -p bondline-io --test smiles_record_analysis
cargo test -p bondline-cli --test smiles_record_analysis
cargo test -p bondline-ui --lib analysis_
cargo xtask corpus --offline
```

The corpus gate verified five direct formula/atom-count results across all raw collections, two SDF records and three SMILES records. Each pinned SMILES record must return a complete match. Source-preserving Open and drawing-import diagnostics run separately. The CLI compares isotope-resolved elemental contributions within 1e-8 and percentages within 1e-7 percentage points. The pinned RDKit and CIAAW Na/Cl weights differ, and the [oracle](../xtask/oracles/smiles_record_analysis.py) checks the two conventions explicitly.

## Actual macOS and MCP

An isolated background macOS host opened four SMILES records, showed the Analysis panel through its registered View binding, and received targeted native accessibility actions on its buttons and number stepper. With both panels open, the Analysis stepper and raw notation inspector originally shared one label. The host found two matching incrementors. A failing App-frame test reproduced this ambiguity. The distinct **Analysis record number** label now identifies the Analysis stepper while both panels are visible. Increment and decrement each advanced one record, and the guarded App-frame test found two independent controls.

On the retained native host, the accessibility tree showed all three formulas, masses, default-count messages and percentage rows. The sodium salt displayed Na 23.92% by weight. Moving to record four showed the unavailable state and removed the previous molecular formula. Unicode Save As and reopen kept all 134 bytes (SHA-256 `3a730641f9ecc1b9bc6c572df97866158dbd09c87559a90a3a76dbbfefa1bc43`), the clean file state and empty history. The reopened panel selected record one and the process exited with approved Close. These observations used no physical keyboard or pointer input. Windows and Linux native execution remains unverified.

The local macOS receiver receipt is `target/record-analysis/native-0s10dpox/receipt.json`, SHA-256 `4fa132858b7a99b8b0d319107e7bcf5be633d694b98d163e94ef3602f6ef8207`. A separately initialized stdio MCP session checked all four records before and after Unicode Save/reopen, whole-collection refusal, selector bounds, contradictory selectors and stale revision. Its receipt is `target/record-analysis/mcp-final.json`, SHA-256 `916c537ed98d4dcc0d95be859044fffc0edf7a9d9f2e011a551ba5de18e2df27`.
