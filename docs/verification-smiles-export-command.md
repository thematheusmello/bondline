# Shared selected-fragment SMILES export

2026-11-14. Headless command/CLI/MCP slice in supplied isolated
`bondline-mol-v3000`, initial base `a393fb9`, received real IO exporter `5f308ac`.
Root owns Git/integration. No GUI/input/activation, installation, Actions, runtime
dependency/lock, model, writer, renderer, old oracle/fixture or golden changes.

The single typed `file.export_smiles_fragment` command has four required closed fields:
expected revision, positive page ID, positive fragment ID and output path. Its source
eligibility guard requires complete Drawing backing. Its explicit chemical-graph scope
excludes geometry and document context; it does not infer H, canonicalize or act as Save.
No first fragment, current UI selection, renderer limit or carbon-path conversion profile
is substituted. Every current writer capability/limit remains available.

`FilePlan::ExportSmiles` captures the entire immutable StoredContent owner, params,
generation and existing save lease. Worker selection uses that same whole source owner.
`Completed::SmilesExport` retains owner and lease beyond publication through finish/drop.
Finish adds only historical `smiles_export` metadata to the latest Engine state. No
current/saved/original/history/path/format/dirty/close/UI/trusted-ack change is made.
The generic binder explicitly refuses export. The existing file worker/reply/close
barriers remain unchanged; their preexisting tests run with the new file variant.

The separately reviewed native IO operation writes unchanged notation plus one LF,
create-only, retaining original publisher errors. The public OpenSMILES Files section
establishes record termination; the bare writer stays unchanged. Maximum notation is
65,536 bytes, maximum file 65,537; existing parser/converter raw-input 65,536 caps are
unchanged. No maximum-file reimport or source-backed SMILES association is claimed.

## Independent expectations and actual files

The first command test constructs a two-fragment original model with sparse application
IDs: fragment3 is methane; selected fragment9 has atom7 carbon13/H3 and atom80 oxygen/H1,
with reversed stored Single endpoint direction. It must produce exactly
`[13CH3][OH1]\n`, not the first fragment. The isotope, literal H, graph, full original
coordinates (including off-page data), application IDs and complete session remain exact;
the output intentionally contains no drawing geometry. Existing external unsanitized and
separate sanitized RDKit consumers check actual selected bytes/meaning/no conformer.

A separate 19-atom disconnected original fragment exceeds the converter's 12-atom limit
and includes branched C13, a cycle, Double/Triple, physical H/deuterium and N/O/F/Cl/Br/I.
Its complete literal file is independently specified; external atom element/isotope/H
and every indexed endpoint/order are checked, as are zero external radicals/conformers.
No renderer/normalizer/periodic-table inference is used. The existing writer tests and
oracles remain byte-for-byte unchanged; external canonical text is not Bondline output.

Other public tests cover closed schemas/zero/stale/missing targets before IO; SDF,
opaque XML and CDX refusal; None H, charge, radical, incorrect H and unsupported-element
refusal; whole content/source/history/redo/close retention; native-save/conversion/export
lease contention through publication; cancellation/drop/error release; actual old receipt
after a new Open generation and committed close; create-only sentinel retention and native
binding refusal. The private test uses a Weak owner handle to prove the complete captured
StoredContent survives a new Open through publication, releases only after finish, and
does not overwrite the newly opened session.

Actual CLI, stdio MCP and authenticated loopback Open an original canonical two-fragment
native XML input and export only its second isotope-methanol fragment. Literal LF/file
receipt, all state/source/association fields and unchanged original input are checked.
The socket callback stays unchanged by export; subsequent Inspect has no receipt cache.
The MCP tool is explicitly `readOnlyHint:false`. Output suffixes/Unicode do not select a
different format. Every actual file faces unchanged `smiles_write_oracle.py`/`smiles_oracle.py`
after asserting exact single-LF, complete ASCII record with no whitespace/name/trailing data.
Only that verified final LF is excluded for the unchanged raw-notation consumer; no trim,
partial record or consumer relaxation occurs.

## Actual RED/GREEN

Every Cargo command uses this checkout's absolute target, incremental 0, dev/test debug 0
and the existing pinned Root interpreter through BONDLINE_PYTHON. No shared artifacts.

```bash
cargo test --locked -p bondline-commands --test smiles_export
cargo test --locked -p bondline-commands -p bondline-mcp -p bondline-cli
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all -- --check
```

- Runtime RED 66251 exits 101: 0 passed / 1 failed,
  `UnknownCommand("file.export_smiles_fragment")`, before production/backend wiring.
  An unused test import warning was incidental, not its failure cause; that import is
  used by the final schema/lifecycle tests. Log `target/smiles-export-command-red.log`.
- The public file-record LF clarification arrived before implementation. Final tests
  expect LF-inclusive bytes/receipts; the initial runtime failure proved only the
  missing command and did not validate the pre-clarification byte expectation.
- First actual backend compile/focused GREEN 26785 exits 0: 6 passed / 0 failed / 0 ignored,
  `target/smiles-export-command-focused.log`. No production/test correction was needed.
- Full affected-package GREEN 15885 exits 0: 230 passed / 0 failed / 0 ignored across 30 suites,
  including the 10 new tests (6 public command cases, 1 private owner/binder case,
  CLI/stdio/authenticated-loopback). Log `target/smiles-export-command-adapters.log`,
 27,029 B / SHA-256 `5a3c500d9442e2b3b8549d973a50c715d7dd62086bbd912bb6dfec5dfea1ec24`.
- Workspace all-target warnings-denied clippy and fmt 47899 both exit 0; logs
  `target/smiles-export-command-clippy.log` and `target/smiles-export-command-fmt.log`.
  The unchanged thin UI compiles against the new optional receipt/command variant.

All 21 prior registry names and their old parameter assertions remain; exact discovery
fixtures add only the 22nd name/count. Ordinary state omits the receipt. Every source/test
file remains exact to the pre-IO draft; the real backend was ported by Root and no stub,
duplicate IO or production correction was used.

The ignored final handoff identifies exact source/test/doc/log/protected hashes. Root's
public-source contract/oracle review (`target/smiles-fragment-export-contract-review.md`,
18,920 B, SHA-256
`be751f5eb56d69679e7c1b5f94333b0b540bc39d89ac4695815cdced0e5772b4`)
is distinct from runtime acceptance and the receiving full gate. This owner did not run full-workspace tests/xtask/fuzz, new Windows/Linux or
native UI. No new native/clipboard/source-backed Save/general chemistry or completed M1
claim follows. Actions stays disabled; all own Cargo jobs are terminal.
