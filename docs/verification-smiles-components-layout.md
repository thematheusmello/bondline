# Independent neutral-component layout declarations

2026-11-15, base691af6f. This records test-first declarations and an independently
qualified external consumer. **The new layout API has not passed here.** Root owns
implementation, Cargo execution, receiving gates and native acceptance. Nothing in this
record completes M1 or establishes general SMILES import, chemistry or platform parity.

The contract is in [the component plan](research/smiles-components-plan.md),
13,210B/SHA256`c8d80fc86d3225e8d2bdf7279258a7dc860b60073136523c42fdb0e35c6c045c`.
Its additive `layout_neutral_components(length,origin,page,&Cancellation)` keeps one
complete Fragment and all global IDs/directed edges/fixed H. Actual edge connectivity
defines components. At most64 total neutral nonisotopic H/C/N/O/F/Cl/Br/I atoms may use
existing supported trees or3–8-member Single cycles with attached trees. Any unsupported
member refuses the whole input. Normalization occurs once; original source/IR stay exact.

Source-order shelf packing is **original Bondline policy**: rigid component poses,
atom-center boxes separated by exactly L, minimum overall center bounds at origin,
width-triggered wrapping by prior row height+L, no rescaling/deduplication/drop. The
complete actual geometry and literal/resolved-H paper/ink certificates remain mandatory.
The declared64-isolated-carbon positive tests a particular A4 shelf/budget, not universal
fit, performance or a hard memory limit. The per-component100000/N and64/N budgets are
the plan's bounded-search policy, not numerical accuracy or elapsed-time claims.

## RED and immutable declarations

The first source is1,867B/SHA256
`8f5d6df83c57ccdc414259864cef047f74717c460e8bb05ae86e6e38ade8b624`.
It independently requires `C.O\r\n` to give one Fragment, no bonds, literal C/H4 and
O/H2 at(144,144)/(208,144), retaining original source/IR allocations. Root observed
exactly one E0599: missing `layout_neutral_components`; terminal101. The retained log
`target/smiles-components/first-io-api-red.log` is596B/SHA256
`e11c554a43d03a36921e096efa3c89849baf7d35e88e5555a5f88e083df4828d`.
This is an intended missing-API compiler RED, not a runtime chemistry/geometry failure.
The pre-execution rustfmt-only1859→1867-byte source/diff is retained separately.

The eight-case Rust declaration is21,683B/SHA256
`80d3262e04d31ad12aace596d446b20b5e1bf26239ea17d8dc76f4155192a04b`.
Its entire first1,867-byte source remains an exact prefix. The earlier20,994B declaration
is SHA256`7b2ed840edf0f0183268764297b42818b0249b641b6c464cb448523fffa4f2e2`;
the sole approved subsequent delta adds analytic120° checks for the existing CC(C)O.F
central carbon. This strengthens the frozen template expectation before production;
it is neither a reproduced defect nor an input/assertion relaxation.

The declared cases cover:

- `C1.O.C1`: global arrays1/2/3, directed bond1→3, components{1,3}/{2}; three textual
  chain starts must not become three graph components. `C1.C1` is one connected tree.
- Exact Document/CDXML equality to existing tree/ring APIs for a single actual component,
  including branched CC(C)O, triple-bond CC#N and methylcyclohexane.
- Literal graph/H/metadata/ID comparisons for interleaved source IDs, CC.O.C1CCCCC1
  (9 atoms/7 bonds/C8H20O), physical H plus Double/Triple, a neutral branched alcohol/HF,
  unassigned internal Double plus water, and two distinct small-ring components.
- A360×650pt page where a64pt hexagon's128pt center-box height forces the ethane/water
  shelf to y336; isolated atoms retain zero-width boxes and exact64pt gaps.
- Actual generated CDXML consumed by independent Expat/Decimal/RDKit, then reopened
  fully mapped/model-equal/source-exact. No writer-reader-only compatibility claim.
-64 distinct CH4 components at L48/origin72/A4: eleven zero-width center boxes per row,
  every atom/label retained;65 atoms refuse before unsupported isotope normalization.
- Nonfinite/invalid length, typed cancellation, late isotope/charge/aromatic/large or
  multiple ring/ambiguous Double/physical-H failures, out-of-paper origin, tiny/overflow
  geometry and full-paper/ink refusal. Every attempted result leaves source/IR exact.

For accepted examples, actual1024² CPU scenes must contain ink and equal the same-count
attachment-safe/resolved-H scene. This is an offscreen reference certificate, not native
input, all viewport scales or human-approved golden evidence. No new GPU helper/dependency
was introduced; Root separately owns command capture CPU/GPU acceptance.

## New external consumer and independent controls

`crates/io/tests/smiles_components_oracle.py` is13,909B/SHA256
`2f1106057469c3f06c62342bafe597d21e557fd768a61c32b0948ae3a5954a5b`.
No older oracle, fixture or golden was edited. Invoke the existing pinned external
interpreter with `-B`, original notation and an optional closed JSON object:

```text
python -B crates/io/tests/smiles_components_oracle.py 'CC.O.C1CCCCC1' \
  '{"atoms":[[6,3],[6,3],[8,2],[6,2],[6,2],[6,2],[6,2],[6,2],[6,2]],"bonds":[[1,2,1],[4,5,1],[5,6,1],[6,7,1],[7,8,1],[8,9,1],[4,9,1]],"components":[[1,2],[3],[4,5,6,7,8,9]],"formula":"C8H20O"}' < actual.cdxml
```

`length`, `origin`, `page` may be supplied; otherwise they are48/72,72/A4. Other keys
refuse. The consumer bounds raw XML to1MiB, requires one complete XML Fragment, validates
literal nodes/H/neutral metadata and directed source-ordinal references independently,
and computes actual components from edges. RDKit supplies independent notation chemistry,
components/rings/formula and actual-file interpretation. Returned Mols are combined if
needed; the number of reader Mols is not the component-count oracle. The pinned reader
actually returned one disconnected Mol for the original interleaved manual control.

Raw Decimal checks require finite coordinates, exact lengths within relative1e-9,
minimum origin, paper centers, original shelf placement, distinct vertices, no nonincident
vertex-on-edge or edge intersection, regular convex3–8 rings and preferred ring edge
orientation. These center/graph checks do not replace production glyph/stroke/paper guards.
Assigned atom/bond stereo always refuses. STEREOANY is allowed only for source-unassigned
degree2/degree2 Double endpoints with raw antiparallel Singles at **both** ends, regardless
of whether the reader reports NONE or ANY. The JSON reports actual `bond_stereo`, truthful
`stereo_none`, `no_assigned_stereo`, physical-H count and actual `raw_graph` arrays alongside
atom/bond/component counts and formula. STEREOANY is not described as STEREONONE.

Before product output existed, an original generator wrote manual XML from fixed literal
graphs/coordinates, never calling Bondline's parser/normalizer/layout/writer. External
session52278 completed25/25 declared outcomes: seven positives (isolated/interleaved,
mixed ring, height-aware wrap, physical H+multiple, collinear unknown Double,64 duplicates)
and18 negatives (empty/bad options, false dot components, direction/H/length/gap/row,
coincidence/on-edge/nonfinite/joined/same-formula graph, distorted ring, noncollinear Double,
assigned source/style). A supplemental two actual-file negatives also pass:

- CO.C retains the target's C/O/C element order and C2H8O formula, but connectivity/H differ
  from CC.O. Independent RDKit confirms the formula and canonical C.CO before rejection.
- A bent manually authored CC=CC.O file independently produces
  `[STEREONONE,STEREOZ,STEREONONE]` and assigned canonical geometry; the new consumer refuses.
  This is actual-file evidence, not an in-memory enum injection.

All27 expected outcomes match, with no consumer correction after its13,909-byte freeze.
The first qualification log is4,701B/SHA256
`f20aaabece596f1be8a66f644bd322b2789196e02b9950e3519cd76d57ac8df7`;
the25-control receipt is5,875B/SHA256
`37e2f81165f368d21df1d06bc5c57fc270d2829ae106eef294c323655c14e8d9`.
Original XML, generator, stdout/stderr, supplemental measurements and source hashes live
under ignored `target/smiles-components/test-declarations/` and are not product assets.

The public basis remains pinned [OpenSMILES1.0](https://github.com/opensmiles/OpenSMILES/blob/9240a2887158699503605bc838e1de9a9f7883f4/opensmiles.asciidoc),
[IUPAC2008](https://iupac.qmul.ac.uk/drawing/drawing.html) and the archived public
[Fragment description](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Fragment.htm),
with exact bytes/qualifications in the source plan. Atom IDs/order, shelf spacing,
resource quotas and neutral supported-profile limits are explicit Bondline policies.
RDKit2026.09.2/NumPy/Pillow remain external pinned test programs, never shipped runtimes.

## Receiving work still required

Root must observe the expanded missing-API RED, implement the real API and run the eight
cases plus relevant IO/clippy/fmt checks. The intended focused command after integration is:

```sh
BONDLINE_PYTHON=/Users/sarenaberg/Workspace/doozle/open-source-world/bondline/.cache/oracles/format-python/bin/python \
  cargo test --locked -p bondline-io --test smiles_components_layout
```

This declaration worker ran direct rustfmt, Python AST and bounded external consumer
checks only; no Cargo, native app/input, hosted Actions or Git operation.130 protected
pre-existing source/lock/fixture/golden identities match at declaration freeze. No
production GREEN, file publisher/command/App/native workflow or Windows/Linux result is
claimed by this note. Root's integration record must append actual receiving evidence.
