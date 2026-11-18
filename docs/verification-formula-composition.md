# Resolved-model composition receiving record

## Main-worktree integration

Root received worker commit41ab8161d70b438779ab7fb898c00402ad757c61 onto base6d9ce09
without changing its implementation or tests. Independent source review finds no material
issue across19 pinned inputs:9351B/SHA256
`a0ab2ba5ddf61d19882a6a74a485eb80f711c6bc3ba00f64e7adc34b333569cf`.
Fragment's private Serialize-only fields and checked constructor support the bounded-input
assumption; no second validation or speculative document wrapper is needed. Root separately
verifies the old neutral-authoring implementation after MAX_AUTHORING_ATOMS is byte-exact.

Fresh root `cargo xtask ci`6948 exits0: **1831 passed/0 failed/1 ignored across272 result
lines**, plus fmt, warnings-denied workspace/vendor checks, layers, seven assets/notices,
strict155/166/221 runtime license closures and eight pinned files/five corpus bundles.
Log target/formula-integration/receiving/full-ci.log231502B/SHA256
`248396a5c627c94bbb3fe6c74f905dded45715de022c4a5f789b6df3db8c5134`.
Root uses its own target, CARGO_BUILD_JOBS=2 and the same literal venv/fixed seed below.
Actual CLI46 bytes remain43338B/SHA256
`02bb76b9f909a3691fddbb5a1f3c2cc794a72f759f43194791b880213558a12c`.
Both locks, workflow and protected MCP control test retain their preceding hashes.

This natural full-gate pass does not explain the preceding Text increment's MCP
replacement-bind AddrInUse. Its failed log and component-only receiving stay preserved;
no listener policy/test/timing or exhausted diagnostic context is changed. New code
adds neither parser nor editing command, so no new parser fuzz/UI/golden claim follows.
All worker and Root execution jobs are terminal. User apps/drawings stay untouched;
Actions OFF, no push/publication. Corpus-wide chemistry comparisons, inferred H, formula
formatting/mass and real source-aware analysis workflows remain required.

## Isolated implementation evidence

2026-11-17; isolated worktree `bondline-formula`, branch
`feat/formula-composition`, base `8e030bf`. Read the
[preimplementation contract](research/formula-composition.md) and
[implementation](../crates/chem/src/composition.rs).

The additive `bondline_chem::analyze_fragment` now counts explicitly resolved
elemental/isotopic model composition and sums atom formal charges. It refuses
every unresolved H count, preserves unspecified versus numbered isotope bins,
uses checked `u64` counts/`i64` charge and supports the full model resource bounds.
Input graphs and the old neutral-authoring implementation remain unchanged.
It provides numeric data only: no valence inference, nuclide-existence or mass
tables, formula formatting, source admission, document aggregation, registry/UI
binding or M3 completion is claimed.

## Test-first evidence

All paths below are in this worktree's ignored `target/formula-composition/`.

1. Research/API policy and an original 13C/H4 literal test were written before
   production. First source: 1,325 B, SHA256
   `a2e08cf2f42879510a6cda34b420f6135946f62e78b1ed37d128541574f13f72`,
   archived as `first-declaration.rs`. The first Cargo run exited101 at intended
   E0432 missing `ElementCount`, `IsotopeCount`, `analyze_fragment`.
   `first-api-red.log`: 1,189 B,
   `2f09306e7db55477b16a6d6501adfaa6871c97f133e79d3d7b5b3e179134af2e`.
2. Eight analytic cases and the external 13-molecule reference case were frozen
   before production (`expanded-before-production.json`). Expanded API receiving
   exited101 only on missing composition exports, including `CompositionError`;
   no unrelated setup failure was observed. `expanded-api-red.log`: 1,019 B,
   `32bbce1d7f897362e1906436d41356239f2656c847cf3945be942080769169a2`.
3. Before any analysis output, the independent Python/RDKit checker accepted six
   manually declared positives and rejected seven deliberate H/charge/isotope/
   ordering/type/schema errors with their exact expected exit statuses. Inputs
   are in `reader-controls.json`; `reader-qualification.json`: 6,474 B,
   `419efadb980f4de3807d2e2c6430f08b725e5b155f69a3c13428b41d91406525`.
   The installed consumer was RDKit2026.09.2 under Python3.13.0. A durable subset
   (two positives/seven negatives) was subsequently added to the committed test
   suite; that addition follows the first implementation and passes first run.
4. The first complete candidate passed all eight analytic cases plus the
   13-reference external case without changing expectations: 9/0/0.
   `first-green.log`: 1,337 B,
   `11bc963d958589cdcdab4b2d26c37b8746732d2d8207946c17ef77ec950a02f1`.
   No product runtime defect/RED is invented: the product REDs here were honest
   missing-API compiler failures. Rustfmt and the durable reader controls do not
   change any product behavior or the thirteen reference inputs.

The analytic cases distinguish physical H from attached H, preserve natural/1H/
2H/3H and natural/12C/13C bins, reject unknown H in authored array order, and keep
all sixteen bond orders, radicals, positions, flags and sparse IDs untouched.
They cover empty/isomeric/permuted inputs, all 118 elements with literal maximum
isotope numbers, cancellation and the full 100,000-atom/100,000-bond model bound.
The extreme literal control produces 6,553,600,000 H and charge totals +12,700,000
and −12,800,000. Those controls establish arithmetic, not chemical plausibility.
The overflow branch is defensive; a valid bounded Fragment cannot overflow the
chosen integer widths, and no reachable-overflow runtime claim is made.

The external references include both methane H representations, ethanol/ether,
carbonyl/Triple/aromatic graphs, ions, a salt, a zwitterion and isotope-labeled
ethanol/methanol. Literal model graphs are independent of the SMILES parser and
writer; RDKit parses separately authored bracket-explicit references with H
removal disabled and asserts no implicit H. Its formula descriptors, isotope
counters and formal-charge sum check actual analysis output. It is an external
test program, never a linked runtime dependency.

## Commands, results and environment

Every Cargo build used the absolute worktree target and:

```text
CARGO_TARGET_DIR=/Users/sarenaberg/Workspace/doozle/open-source-world/bondline-formula/target
CARGO_INCREMENTAL=0
CARGO_BUILD_JOBS=1
CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0
BONDLINE_PYTHON=/Users/sarenaberg/Workspace/doozle/open-source-world/bondline/.cache/oracles/format-python/bin/python
PROPTEST_RNG_SEED=671308429
```

The Python path was used literally, without resolving it into a different
interpreter. Host: Darwin27.2.0 arm64; rustc1.94.1
`e408947bf (2026-03-25)`, Cargo1.94.1 `29ea6fb6a (2026-03-24)`.

| Scoped command | Result |
|---|---|
| `cargo test -p bondline-chem --test composition --locked -- --nocapture` before API | intended E0432, exit101 |
| `cargo test -p bondline-chem --test composition --test composition_rdkit --locked -- --nocapture` before API | intended missing exports, exit101 |
| same two targets after implementation | 9 passed/0 failed/0 ignored |
| final `cargo test -p bondline-chem -p bondline-model --locked` | 57 passed/0 failed/0 ignored,21 result lines, exit0 |
| `cargo clippy -p bondline-chem -p bondline-model --all-targets --locked -- -D warnings` | exit0 |
| `cargo fmt -p bondline-chem -p bondline-model -- --check` | exit0 |

Final logs: `final-chem-model-tests.log` 9,010 B /
`6bd140054712e050ef21d5211961c45eff5175f3e0f0d8de16a44f489b53f726`;
`final-clippy.log`178 B /
`74e060f55521b534e98d6d3300d2a0f978b23cf909645966ce413dbc501a932c`;
`final-fmt-check.log` is empty with exit0. This is targeted chem/model evidence,
not a full-workspace/xtask gate. No GUI/GPU/native, fuzz, Windows/Linux, Actions,
push or publication was run. All worker execution jobs are terminal.

## Pins, preservation and limits

Final product module: 4,443 B, SHA256
`dd8161a269b28a190ead038667c963d5fe2913ea1b3ac996b858d58b4ab9deb7`.
Analytic test:13,663 B /
`243aaab3a761943c848f5577103349fcb3637f04ef03cb95869a84db5eb30b7d`;
RDKit test:9,720 B /
`631b0a54540e372bf7283ca8ccde834bdbd1b845c6b9a842f1bc7cb0ac9752b5`;
external checker:2,478 B /
`6c2dd2a8395618edcf0e37666061dd50021546a1dcf48457a3903b542b151c3c`.
`before-production.json`, `first-green-receipt.json`, `final-receipt.json` and
`protected-inputs.json` retain input/source/log identities. The existing chem
normalizer body, its tests, model, manifests/lock and all unowned tracked files
remain exact. Only the additive module/reexports, new tests and these two notes
are included in the local feature commit; no main-worktree files were edited.

Cached primary OpenSMILES source:86,816 B /
`00f0423da612de913770133aa13e456d0cebc562dd50166534a8a7f9a834bd21`;
SDK H page:15,143 B /
`a89a2f54d3c3daa2de3d02a084c6db4586f85adbd5319306b5cc9835c78d516a`.
Their URLs/version are in the research note and `research/sources.json`.
Initial raw HTTP retrieval returned406; the browser-header retry obtained those
two pins. RDKit's primary API page was read through the web tool but its direct
cache request returned406, so no raw-page hash is invented. Gold Book access
failures are retained as research limitations. Cached source material is ignored
and not shipped.

Future command integration must prove complete source/selected scope and resolve
hydrogens explicitly before invoking this API. Composition alone proves neither
topology/identity, source faithfulness, valence, isotope existence, abundance nor
mass. Document-level aggregation and user-facing formula/analysis remain later
work. Root owns central STATUS/provenance/integration and any broader gate.
