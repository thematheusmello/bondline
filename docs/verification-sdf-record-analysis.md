# Analyzing one molecule in an SDF collection

Open the Analysis window on an SDF, then choose **Next molecule**, **Previous molecule** or enter a record number. Each result belongs to the indicated record. The window starts at the first record after opening or replacing a collection. A changed source invalidates the cached result. Queries leave the collection, its data fields and edit history intact.

The registry command `analysis.properties` accepts an optional positive `sdf_record_id`, with the same revision and selection-epoch checks as the existing analysis query. The result names the record, e.g. `{"scope":"sdf_record","record_id":2,...}`. A collection query with no record ID reports `unsupported_source`. Each calculation reads only the selected record’s existing MOL byte range. A synchronous query accepts up to one MiB of MOL bytes and 4,096 drawn atoms. A limit result on one record leaves its neighbors available. The selector resets when the source changes because record ordinals can change after removal.

The independent test uses RDKit 2026.09.2 to read five molecules with both MOL editions from one SDfile. Bondline and RDKit agree on formula, average weight, exact mass and elemental percentages, including isotope-labelled methane and ammonium. A deliberately incorrect formula in a data field has no effect. All five Unicode Save As outputs match the original bytes and reproduce the same answer when reopened. A separate case checks a large valid connection table and its small neighbor. [Executable oracle](../xtask/oracles/sdf_record_analysis.py).

| Record in the original mixed fixture | Formula | Molecular weight | Exact mass (Da) |
| --- | --- | ---: | ---: |
| V3000 ethanol | C₂H₆O | 46.069 | 46.041865 |
| V2000 methylamine | CH₅N | 31.058 | 31.042199 |

The CLI tests check whole-collection refusal, invalid and missing IDs, stale revisions, and the new ordinal after removal. App-frame pointer input opened **View → Show Analysis Window**, navigated between records and restored record one after reopening. In a background macOS process, the registry menu binding opened the window and genuine AX button presses navigated 1 → 2 → 1 → 2. Both sets of values, the hydrogen-default messages and percent rows appeared in the accessibility tree. A Unicode Save As and reopen preserved the 740-byte file (SHA-256 `e5cbdc14329c59bad55148c5de37058de90f651af10966c30cf8b89b3b5efd39`). The observed native receipt was `target/record-analysis/native-biyfb9qg/receipt.json` (SHA-256 `16b425d62564058500c3aafd4dc7f12875459f525196014ac347b6ed90d88223`). That build verified the previous/next buttons. Numeric-entry interaction and Windows/Linux native execution await separate checks.

A direct macOS AX press on the menu entry closed the popup without opening the panel in two attempts. The panel opened through the menu’s registered command. The actual AX menu discrepancy remains under investigation.

```sh
cargo test -p bondline-cli --test sdf_record_analysis
cargo test -p bondline-ui --lib analysis_navigates_individual_sdf
```

Set `BONDLINE_PYTHON` to the pinned RDKit environment for the external test.
