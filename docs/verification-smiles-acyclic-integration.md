# Neutral acyclic conversion integration

2026-11-15, branch `feat/multiple-bonds`, following plan checkpoint `ad68dc9` and native
converter `c71d5fc`. This completes one bounded M1 increment. Full M0/M1 and M2–M5 remain
incomplete; the persistent goal stays active. Actions remains disabled, with no push,
publication, dispatch or new hosted execution.

## Delivered contract

The existing30-command registry gains no command. Its create-only
`file.convert_smiles_to_cdxml` retains all eight required scalar parameters and adds a
closed optional `profile`. Omission/default serialization, legacy geometry/refusals and
old receipt identity remain compatible. `neutral_carbon_path_v1` is the original1–12
carbon/Single path; `neutral_acyclic_v1` accepts a connected tree of at most64 neutral,
nonisotopic H/C/N/O/F/Cl/Br/I atoms with branches and Single/Double/Triple edges, using the
existing closed fixed-H normalizer. Physical H nodes, graph direction/IDs, source and IR
remain intact. It neither perceives chemistry nor selects a source-file record implicitly.

The [public-source review](research/smiles-general-layout-sources.md) and
[agreed plan](research/smiles-acyclic-layout-plan.md) distinguish OpenSMILES notation,
IUPAC120°/180°/contextual degree-four depiction and original policy. The new origin is
minimum atom-center x/y, while the old profile retains its endpoint anchor. There is no
automatic scaling. Actual connected-tree edges, nonincident intersections/stroke separation,
bounded deterministic alternatives and full literal ink/paper limits at1024² must pass.
Literal and same-authored-count resolved-H Scenes must agree exactly.64 is a resource
ceiling, not a guarantee of fit. Cancellation, unsupported graphs and budget exhaustion
return typed errors without destroying input.

The shared renderer uses its existing attachment-safe label path for authored-H degree≥3
as well as multiple bonds. An independently frozen star reproduces a real H occlusion/
wrong-Atom-hit defect, then verifies Bond1 picking, analytic pixel corridors, source
retention and actual CPU/GPU agreement. A separate stacked-H overlap control already
passed under the old union-mask guard: that suggested defect was disproved, and collision
policy was not changed. [Exact renderer evidence](verification-literal-branch-attachment.md)
keeps these findings separate.

UI radio choices replay as ordered form actions; a later choice cannot relabel an earlier
Convert. Pending/busy/browsing/stale/success states prevent changes. Profile and receipt
stay captured in the existing FilePlan/worker. Whole-notation read, create-only publication,
save lease, real worker retirement, stale-origin guards and no native Save/close ACK remain.
Success keeps the current document and directs explicit ordinary Open. Complete SMILES
source-file sessions do not acquire molecular editing authority from a projection.

## Test-first evidence and corrections

- Root73477 observes actual unknown-profile runtime RED before implementation. Independent
  layout76197 and expanded75886 are missing-API compiler stages, not eight runtime failures.
- Root77912 and expanded24008 observe missing native profile controls. Intermediate18207
  instead exposed a new-test `StoredContent`/Arc type assumption. The correction retains
  complete owner equality and the original exact CaptureStamp checks; it changes no product.
- Independent command/wire48122/10135/30646 record actual missing-profile refusals. New-test
  bond.create setup and `finish_ui_file(false)` dismissal ordering corrections preserve
  every behavioral assertion. Rustfmt changes are retained separately from product REDs.
- Producer16397 passes3/8 and exposes the real literal/resolved branch-label prerequisite.
  After the exact shared renderer correction,50864 passes7/8; its remaining failure is the
  new external consumer's blanket `STEREONONE` assertion for actual `CC=CC`.

Pinned primary [RDKit Bond.h](https://github.com/rdkit/rdkit/blob/f86b7dc9b31e2531200bbd1f54589bcbd968bb05/Code/GraphMol/Bond.h#L89-L99)
and [Chirality.cpp](https://github.com/rdkit/rdkit/blob/f86b7dc9b31e2531200bbd1f54589bcbd968bb05/Code/GraphMol/Chirality.cpp#L201-L238)
define ANY as intentionally unknown and map the exact linear treatment to it.
[IUPAC GR4.1](https://iupac.qmul.ac.uk/drawing/drawing.html#GR41) supplies the depiction
basis. Exact-file horizontal/vertical/diagonal/reversed probes remain canonical `CC=CC`,
without E/Z or assigned atom chirality. The initial message “no invented E/Z” did not
establish a product stereo defect.

The reviewed correction affects only the new oracle/assertion output: preserve all raw,
graph/H/formula/canonical checks; require actual antiparallel substituents at both ends
of a source-unassigned degree-two internal Double; permit ANY only in that context.
Reject assigned stereo, wrong context and near-linear geometry even if the reader says
ANY. Always report actual `bond_stereo`, truthful `stereo_none` and `no_assigned_stereo`.
Qualification21061 passes16 actual-file controls plus10 separately identified in-memory
guard controls. No product stereo algorithm changed. Potential degree-three internal
Double remains unsupported. [Independent layout/oracle evidence](verification-smiles-acyclic-layout.md)
retains original declarations, failed log/output and exact correction inverses.

## Receiving and full local checks

The20 additive tests comprise eight IO, two renderer, five command, two actual-wire and
three actual-App cases. Root receives all frozen declarations. Final IO90162 passes8/0;
full renderer70598 passes86/0; initial Root command28931, wire91979 and App92091 pass5/2/3.
Independent producer15170 full default IO passes299/0, with warnings-denied all-target IO
clippy/fmt. Independent backend88997/81769 passes5+2 and25193 clippy/fmt pass. See
[backend source/transport evidence](verification-smiles-acyclic-conversion-tests.md).

Root40759 runs with Rust1.94.1, literal pinned format-python venv and
`CARGO_INCREMENTAL=0 cargo xtask ci`: **1136 passed,0 failed,1 ignored**,161 result lines,
exit0. Rustfmt, warnings-denied all-target workspace/vendor clippy/tests, layers, licenses
and corpus pass. The old explicit visual-capture utility remains ignored. Full log
`target/smiles-acyclic/root-final-ci.log`:138074 bytes, SHA256
`a20d8e4ce5de417283b4dc5c421ae4510614d10ba56a0648531e7e955009fc33`.

Final product snapshot `target/smiles-acyclic/root-product-snapshot.json`:1815 bytes,
SHA256 `ea701d056ba66fd2372c42b863816fb71549031fbce47f59c303d46700262d40`.
Its ten source files remain exact through native acceptance. New App tests are8998 bytes,
SHA256 `2b85b5eb201328b69e3c3d4f886b71ae53d5e2635f5210e6455da9d6e759f812`;
new IO tests25490/`980bf95c6759d405957209d19f2e31a7783c460113deac50abb18bc13ff5a83d`,
new oracle10090/`190628332cb63f1287edd4debcdff1ebaeaf451fa885ac965f06229080789a0a`.
Only new test expected-output objects gain the qualified truthful stereo fields. Every
old oracle/fixture/dependency/lock/SDK/font/approved golden stays exact; old direct typed
parameter tests add only the explicit default profile and aggregate bridges are append-only.

Final independent handoffs are received under ignored `target/smiles-acyclic/`:
producer177203/`9d8533d46a1025a41381e2dc1b9984fa89fa415f06f79a5ad17b188ae49ab6fd`,
backend6792/`97d85696d1d7831f60f6dc022a95a89557409d666579aa0f5f1f1f64428f6517`,
layout/oracle39299/`6d559865c747747ece70075a5923952c866d89e486d68643bdb4cb7fcd51803f`
and renderer13047/`98d5f6debb8037ce63dcbc55a6b3dffb1705ee9b04d9c2dc3b23e8311f8cb546`.
All worker jobs are terminal. No protected old expectations were relaxed.

## Actual native Mac acceptance

Root87151 builds the final native binary, exit0. A fresh review bundle
`target/smiles-acyclic-native/Bondline.app`, identifier
`org.bondline.neutral-acyclic-review`, runs without a control channel. Binary22776584 bytes,
SHA256 `f6d0ce575e0e13f77ac9bf5cfe99648d460545948e378e17f93c8173932f723a`.
The exact-path process is PID70457 at final observation. Inputs/expected graphs were
declared before interaction in ignored `pre-native.json`; no product-generated geometry
was used to decide independent chemical correctness.

CUA performs actual menu/pointer/key actions and native AX path setters:

1. F5 gives100%; draw an independent unsaved C–C. File conversion shows two radio choices,
   default Carbon chains, explicit48pt length,72pt origin and A4 dimensions.
2. The default profile refuses `CC(C)O`. Select Acyclic neutral structures; existing output
   refuses. Retry to absent Unicode `branched-λ.cdxml` succeeds,4atoms/3bonds, with the
   correct layout receipt. Original Untitled*,2atoms/1bond and position remain visible;
   no automatic Open/current Save occurs, and the prior global error clears.
3. Close form and explicitly Open output. Cancel the dirty decision: original drawing
   remains. Repeat and Discard and Open the agent-created drawing. Actual branch/OH/
   attachment labels and three bonds are visible; source is Saved.
4. Select Double and click the first C–C. It becomes dirty C=C with corrected terminal/
   central H. Edit Undo restores Saved Single; Edit Redo restores dirty Double.
5. Save As a separate Unicode `edited-λ.cdxml`; explicitly Open it again. The saved
   Double, unchanged branch/OH and4atoms/3bonds remain visible.

Independent Expat/Decimal/RDKit consumer executions use the predeclared source notation,
literal H/elements/directed bonds/formula,48pt lengths, min-center72pt and A4 expectations:

| Actual native output | Bytes / SHA256 | Independent result |
|---|---|---|
| `branched-λ.cdxml` |766 / `d15d3c1a353f7afeb26d38640f7afd38f6383193dfe4b15da5ca495cf41abfae`| `CC(C)O`, H[3,1,3,1],3Single, C3H8O, no assigned stereo |
| `edited-λ.cdxml` |766 / `01b3fc207fb705977386e80fe97727076db5153ff8fc13b1a6c9697d45a27d76`| `C=C(C)O`, H[2,0,3,1], firstedgeDouble, C3H6O, no assigned stereo |

The7-byte input and52-byte protected sentinel remain exact. The original766-byte
conversion also stays exact after edit/Save As. Native receipt and independent results
live under ignored `target/smiles-acyclic-native/`. The separately prepared cycle vector
was not submitted through the native form; cycle refusal is headless evidence only.

Finally CUA raises installed `com.revvity.ChemDraw` as requested and observes the unchanged
blank `Untitled ACS Document 1996-2`. No reference file was edited/saved/closed, no binary/
resource was statically inspected and no icon/font/sample/template was copied. The new
Bondline bundle remains open at Saved edited output in the background. Actual inspection
images are not new human-approved goldens.

## Limits and next task

No native numeric geometry typing, OS clipboard, race/cancel/close timing, physical-DPI,
Windows/Linux or new fuzz/hosted execution is established. Runtime closures stay154/165/220
with2 assets; corpus stays8 files/5 bundles and still lacks general foreign CDX/chemistry.
The sole approved golden stays
`1b650231527f47db5dc7c368644939907d2b038d32f25e77578318f483521bbc`.
Historical UI wait/listener-rebind failures remain unexplained; this gate does not prove
their causes. Next specify/test simple rings with substituents, then component packing and
general source-qualified SMILES import. Do not repeat the delivered acyclic increment or
claim clipboard, reference-screen or complete M1 parity.
