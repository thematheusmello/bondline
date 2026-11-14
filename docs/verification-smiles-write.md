# Explicit-H SMILES structure export

2026-11-14. Original Rust library-only writer, supplied branch
`feat/smiles-explicit-writer`, supplied base `5df1b28`, in the isolated
`bondline-mol-v3000` checkout. No Git operation was run. The earlier syntax parser
and its independent tests remain unchanged; this is a separate generated-output
operation, not a source-backed document conversion or application command.

## Contract and original policy

`smiles::write_fragment(&Fragment) -> Result<Vec<u8>, WriteError>` exports one
explicitly authored supported structure as valid **noncanonical** SMILES. All atoms
must have `Some(literal_H)`, neutral charge, Radical::None and element H/C/N/O/F/Cl/
Br/I. Supported positive isotopes1..65535 retain their exact numeric value without
testing physical isotope existence. Every bond must be Single, Double or Triple.
Whole-fragment metadata and finite valence checks happen before output construction.

The required identity is bond-order sum plus literal H equals H1/C4/N3/O2/halogen1.
Physical H neighbors contribute their bonds to the sum; they never decrement,
replace or get added into the authored literal H count. A physical H atom requires
literal H0 and exactly one Single bond; its token is `[H]` or `[massH]`, with no
forbidden attached-H suffix. Other atoms always receive an explicit `Hn`, including
H0/H1. Missing H, mismatched counts, unsupported elements/orders/charge/radical state
and empty fragments refuse. No periodic-table implicit-H or radical inference runs.
The model has no stereo field; this API makes no stereo export claim and does not
reinterpret positions as stereochemistry.

IDs choose deterministic traversal, independent of atom/bond vector order or endpoint
direction. Roots and neighbors use ascending AtomId; the writer iteratively plans a
DFS forest, emits all but the last child in parentheses and uses the last as chain
continuation. A dot separates actual connected components. Cycle annotations precede
branches, as required by the declared grammar. Closing labels are released before
new openings at the same atom; the lowest free label0..99 is reused. Labels below10
use one digit, others use `%` plus two digits. No total-cycle limit of100 is invented.
This ordering is not graph-isomorphism-invariant canonicalization.

Limits are4096 atoms,8192 bonds,1024 actual output components,128 open parentheses,
100 simultaneous open labels and65536 output bytes. Graph-size limits precede graph
allocation. Iterative planning/emission uses heap stacks bounded by the accepted
graph, not recursive calls. Branch and label budgets apply to this chosen encoding;
refusal does not prove no alternate traversal could fit. Each byte append checks the
complete candidate length before changing it, and no error returns partial output.
`WriteError` reports model atom/bond IDs, finite-valence facts or a resource category;
it invents no parser byte offset for model input. Input is borrowed and stays exact on
both success and failure. These resource policies are not a hard allocator/RSS cap.

Positions and application IDs are intentionally not encoded as SMILES data. This is
explicit structure export, not a whole drawing Save. It does not authorize flattening
unknown CDXML/CDX/MOL/SDF backing, mark a drawing clean, associate a filename or alter
a saved/source baseline. No model, StoredDocument/history/accounting, Engine, filesystem,
UI, import, layout or clipboard code changes here. The existing symbol catalog is
reused by one authorized crate-private visibility change; no symbol data is copied.

## Sources and independent expectations

The same pinned [OpenSMILES v1.0 publisher source](https://github.com/opensmiles/OpenSMILES/blob/9240a2887158699503605bc838e1de9a9f7883f4/opensmiles.asciidoc)
provides bracket, H-count, isotope, bond, branch, ring and normalization facts. Exact
86816-byte SHA256`00f0423da612de913770133aa13e456d0cebc562dd50166534a8a7f9a834bd21`
remains cached only in ignored target/. Its optional normalization guidance is not a
canonicalization algorithm. No GFDL prose/grammar/example corpus, upstream parser
implementation or depiction asset is copied. Existing IUPAC-ELEMENTS facts supply
symbol names. The finite neutral valence subset, H policy, ID order and budgets are
explicit original Bondline restrictions, not complete chemistry/OpenSMILES conformance.

Original tests declare literal ethane/propane/branched/cyclic strings, Single/Double/
Triple tree and closure bonds, all supported elements, zero-H atoms, isotope65535,
H-H and mixed isotope/physical-H output. Shuffled storage/endpoints give the same
literal output. Wrong/missing/huge H, physical-H misuse, charge, radical, unsupported
element/order, empty graph and failed resource cases retain the complete model.

Actual4096-atom chains pass without parentheses or recursion;4097 refuses. Original
regular small graph components reach8192 bonds on4096 carbon/H0 atoms;8193 refuses
before a later valence error. These are graph-budget cases, not evidence of physical
realizability. Actual comb trees reach128 open parentheses and refuse129. Actual1024
components pass and1025 refuses. Original100-ring windows pass,101 refuses, and1001
closed cycles reuse labels rather than exhausting a total-label counter. `%73` and
`%74` appear in separate paired closures. A focused private byte-append test reaches
65536 and refuses the next byte without changing the candidate; this is a guard-level
boundary test, not a claim of an independently constructed whole graph at that size.

## Actual RED/GREEN and external consumers

All Cargo commands selected this checkout's absolute target, CARGO_INCREMENTAL=0,
CARGO_PROFILE_DEV_DEBUG=0, CARGO_PROFILE_TEST_DEBUG=0, and the existing absolute Root
BONDLINE_PYTHON. No new tool, package, dependency, lock or interpreter was installed.

- `target/smiles-write-red.log`: initial literal ethane/propane test failed E0432
  because write_fragment did not exist, session48819 exit101, before production edits.
- `target/smiles-write-contract-red.log`: the expanded independent output/error/
  deterministic/resource/consumer contract failed for missing WriteError/write_fragment,
  session3397 exit101, also before production implementation.
- `target/smiles-write-first-green.log`: first implementation passes all7 focused
  integration tests, session71073 exit0. Additional literal Double/Triple closure
  assertions then passed in the final IO gate, without a production behavior change.
- `target/smiles-write-io-green.log`: exact final `cargo test -p bondline-io --locked`,
  session56467 exit0:194 tests,0 failures/ignored, plus0 doctests. This includes7 new
  integration tests, the new byte-boundary unit and all186 prior tests/assertions.
- `target/smiles-write-clippy.log`: all-target IO clippy with warnings denied passes,
  session20448 exit0. Workspace rustfmt check passes in
  `target/smiles-write-fmt-check.log`.

New original external-only `smiles_write_oracle.py` imports the unchanged notation
consumer rather than duplicating its raw graph reader. Actual generated bytes face
pinned RDKit2026.09.2 with sanitize/removeHs/parseName/allowCXSMILES false. This retains
the known Python skipCleanup/default-cleanup and token-normalization qualifications
from [syntax verification](verification-smiles.md); no old assertion is weakened.
Unique original isotope values provide an independent atom-to-model mapping for a
201-atom/300-bond100-ring output: every externally consumed edge and literal H count
must match the authored graph, independently of writer traversal. Physical isotope-H
nodes stay separate, and carbon's authored literal H2 remains2 beside two H nodes.

A separate external molecule instance parses the same bytes with sanitization enabled,
H removal/name/CX parsing still disabled. Its canonical SMILES, formula and calculated
radical counts independently check supported ethane/propane/ethene/ethyne meaning;
no coordinates are generated. Those calculations are external expectations, not a
Bondline chemical engine or canonicalizer. The unsanitized raw graph result is captured
separately. Missing/wrong external versions fail, never skip. The release parser source
remains pinned to
[f86b7dc9b31e2531200bbd1f54589bcbd968bb05](https://github.com/rdkit/rdkit/blob/f86b7dc9b31e2531200bbd1f54589bcbd968bb05/Code/GraphMol/SmilesParse/SmilesParse.h).

## Frozen source and continuation

| File | Bytes | SHA256 |
| --- | --- | --- |
| `crates/io/src/smiles/write.rs` |16220|`77e0fc89ea6cdbf66cffbd3d271cbb411cc076c9e69f8a6f238b54fb27758be2`|
| `crates/io/tests/smiles_write.rs` |18401|`76b8fae0523803cf4ee17c0788a4a837b8900c4c85793485718ddbbba316742d`|
| `crates/io/tests/smiles_write_oracle.py` |1368|`fae51a488badd0d9cc6eb7bbe4cf85a8f176220d3b35218ac44924afbcce5f2b`|

Removing only the new parent mod/reexport reconstructs the exact original parser hash
`c9d8e2c810e9b9a1d1c7ca1c26ecdb441a214d63531ba67efaa4d6e6d15f9ff3`.
Undoing only symbol()'s visibility line reconstructs the existing catalog hash
`a6006d4bf23623bcec38b3e2ced050b427b4ad874b4feb97011788dc5787f0a0`.
Original parser tests/oracle remain exactly7401a294…3810 / a13b0de3…6982. No old
fixture, golden, dependency, lock or model definition changed. All own jobs are terminal.
Independent peer review and Root receive/fuzz/full/hosted gates remain separate; Root
owns commits and current global status. No Git, GUI, install, native or full-workspace
execution was performed for this writer. Import projection, implicit-H/radical policy,
source-backed edited regeneration, commands/clipboard/layout/native workflows and
canonicalization remain missing; no complete M1 claim follows.
