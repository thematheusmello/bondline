# Analyzing one molecule in an SDF collection

Open the Analysis window on an SDF, then choose **Next molecule**, **Previous molecule** or enter a record number. Each result belongs to the indicated record. The window starts at the first record after opening or replacing a collection. A changed source invalidates the cached result. Queries leave the collection, its data fields and edit history intact.

The registry command `analysis.properties` accepts an optional positive `sdf_record_id`, with the same revision and selection-epoch checks as the existing analysis query. The result names the record, e.g. `{"scope":"sdf_record","record_id":2,...}`. A collection query with no record ID reports `unsupported_source`. Each calculation reads only the selected record’s existing MOL byte range. A synchronous query accepts up to one MiB of MOL bytes and 4,096 drawn atoms. A limit result on one record leaves its neighbors available. The selector keeps its ordinal across a metadata edit or history step when the same session still has the same record count and exact molecular block at that position. A replacement session or changed record count restarts at record one. Every content change invalidates the chemistry result.

The independent test uses RDKit 2026.09.2 to read five molecules with both MOL editions from one SDfile. Bondline and RDKit agree on formula, average weight, exact mass and elemental percentages, including isotope-labelled methane and ammonium. A deliberately incorrect formula in a data field has no effect. All five Unicode Save As outputs match the original bytes and reproduce the same answer when reopened. A separate case checks a large valid connection table and its small neighbor. [Executable oracle](../xtask/oracles/sdf_record_analysis.py).

The offline pinned corpus also opens the 690-byte SDF with ordered and duplicate fields. It checks that each query identifies the requested record and compares its formula and drawn atom count with RDKit. Both records yielded C₂H₆ with two drawn atoms. The separate extracted-block check still passes for each record. [Corpus evidence](corpus/README.md#molecular-formula-evidence).

| Record in the original mixed fixture | Formula | Molecular weight | Exact mass (Da) |
| --- | --- | ---: | ---: |
| V3000 ethanol | C₂H₆O | 46.069 | 46.041865 |
| V2000 methylamine | CH₅N | 31.058 | 31.042199 |

The CLI tests check whole-collection refusal, invalid and missing IDs, stale revisions, and the new ordinal after removal. App-frame pointer input opened **View → Show Analysis Window**, navigated between records and restored record one after reopening. In a background macOS process, the registry menu binding opened the window and genuine AX button presses navigated 1 → 2 → 1 → 2. Both sets of values, the hydrogen-default messages and percent rows appeared in the accessibility tree. A Unicode Save As and reopen preserved the 740-byte file (SHA-256 `e5cbdc14329c59bad55148c5de37058de90f651af10966c30cf8b89b3b5efd39`). The observed native receipt was `target/record-analysis/native-biyfb9qg/receipt.json` (SHA-256 `16b425d62564058500c3aafd4dc7f12875459f525196014ac347b6ed90d88223`). That build verified the previous/next buttons. In a separate App-frame run, accessibility set the Record number control directly to 1 and 2. The panel displayed ethanol and methylamine respectively and the document and history stayed identical. Physical numeric entry on macOS and Windows/Linux native execution await separate checks.

On macOS, the Record number stepper initially accepted an accessibility increment and stayed on the first molecule. The integer widget's default 0.25 step rounded back to 1. A whole-record step and bounds check now keep every report within the collection, including a request past either end. A real macOS accessibility increment and decrement each moved exactly one record and repainted formula, mass and percentages. The same host saved, reopened and closed normally. Its receipt SHA-256 is `fa005f57dfc39c5002395f004d433477ea3e44a3f2b5800140a9ea293a3c2127` at `target/record-analysis/native-k_6q8_ax/receipt.json`. The App-frame test also clicked the numeric control, replaced its text and pressed Enter. Physical desktop typing was not exercised.

The metadata workflow keeps methylamine selected while editing the preceding ethanol record's note, through Undo and Redo, even as its byte offset changes. The App-frame test also removes the first record and checks the renumbered molecule and its restoration on Undo. The actual macOS run returned the original 740-byte file after Undo and Save and restarted at ethanol after reopening. Its receipt SHA-256 is `1ba6e68d0f9f98f5306be96598257ad1ed73344a5d9b4975ca461e16848d8d50` at `target/record-analysis/native-fjgql0vx/receipt.json`.

A direct macOS AX press on the menu entry closed the popup without opening the panel in two attempts. The panel opened through the menu’s registered command. The native menu correction and its separate macOS receipt are tracked in [PR 37](https://github.com/thematheusmello/bondline/pull/37).

```sh
cargo test -p bondline-cli --test sdf_record_analysis
cargo test -p bondline-ui --lib analysis_navigates_individual_sdf
```

Set `BONDLINE_PYTHON` to the pinned RDKit environment for the external test.
