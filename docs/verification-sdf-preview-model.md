# SDF preview data prerequisite

2026-11-14. This implements the data dependency from
[sdf-preview-plan.md](research/sdf-preview-plan.md), not application molecular display.
No native window, UI/input, REA runtime, GPU or hosted Actions check was run.
Actions remains disabled by user instruction.

## Delivered API and boundaries

`bondline_io::sdf::preview::prepare_sdf_preview(&SdfDocument, &Cancellation)`
returns a complete ephemeral `SdfPreview` or a typed `PreviewError`. It borrows
each indexed MOL range, selects its actual version and calls the unchanged strict
V2000/V3000 parser/mapper. A later unsupported/empty record refuses the whole
preview; no first-record fallback or partial success is returned. The source
owner and every original record/data byte remain unchanged.

`page()` returns a derived Page. `display_hydrogens()` and `hydrogen_origins()`
return BTreeMaps keyed by its remapped AtomId. `records()` retains source ordinal
RecordId and preview FragmentId. Each PreviewRecord's `atom_mapping()` and
`bond_mapping()` maps **preview ID to per-record local mapper ID**, not to a
V3000 raw source index. IDs are deterministic for the same source order, not
cross-source/future reorder identities. Errors expose `code()` and optional
`record_id()`; nested MOL errors retain their original typed cause/code.

The preview retains no raw-source allocation: the caller owns that separately.
It is not a StoredDocument, does not grant Save/export/edit authority and must
not replace authored content or advance Engine ID floors. Debug prints only
record/atom counts, not source text or molecular fields.

Authored Some(count), including zero, unusual declarations and neutral elements
outside C/N/O, stays exact. `Literal` includes counts decoded by existing strict
declared-valence arithmetic. Authored None stays None. A separate original
`NeutralDefaultV1` display profile derives only neutral C4/N3/O2/F1/Cl1/Br1/I1
using checked Single/Double/Triple bond-order sums; over-valence or unresolved
elements refuse. Physical H remains a node: it requires one Single neighbour
and no attached H, and contributes to that neighbour's bond sum. Unknown inline
H on that node resolves to zero with `PhysicalHydrogenV1` provenance; authored
zero retains `Literal`. Charge/isotope/radical annotations and strict-mapping
failures refuse, without deleting them from the retained source. Literal counts
are not rectified or certified chemically valid.

The original packing uses one shared 24pt/unit, x-right/y-down transform, then
translation only: left atom bound 72pt; first top atom bound 72pt; next top is
the prior atom-bound height plus 72pt below that row's top. PageSize starts at
A4 and grows to maximum atom coordinate plus 72pt, subject to existing model
limits. Checked remapping/budgets apply across every record. Empty records,
coincident bond endpoints, unprojectable coordinates and excessive page extents
are explicit failures. Point, page and 100000-item model limits stay unchanged.

This API does **not** prove glyph/stroke ink clearance, label attachment,
renderer support, readable viewport scale or actual pixels. Double/Triple
bonds are valid data for the derived sums; the current rendering profile still
needs its own support/refusal. The opposed C—N—C test preserves real directions
and derived NH1, without claiming the existing horizontal label masking can
display that case correctly. The renderer/command/UI slice must close these
limits using the real additive renderer entry; no stub was used here.

Cancellation is checked initially, between records and throughout new
topology/remapping/bounds loops, and before success. Existing bounded parser,
mapper and model final-validation calls remain non-interruptible internally;
this is not an immediate-cancellation or UI-latency claim.

## Independent inputs and sources

The new [original fixture](../crates/io/tests/fixtures/sdf-preview/README.md)
is MIT OR Apache-2.0: 740 bytes, SHA-256
`e5cbdc14329c59bad55148c5de37058de90f651af10966c30cf8b89b3b5efd39`.
Its V3000 ethanol has vertical O geometry; V2000 methylamine has a reversed
C-to-N direction. Literal expected display H is [3,2,1]/[3,2], while all
projected authored H remains None. Literal display positions are
[(72,108),(108,108),(108,72)] and [(108,216),(72,180)]pt. Expected raw MOL spans
are 0..371 and 410..713; neither our writer nor a generated expected graph is
used as its source of truth.

The unchanged `mol_oracle.py` consumes each actual raw MOL, independently
reporting CCO/C2H6O/H[3,2,1] and CN/CH5N/H[3,2] with the original coordinates.
The unchanged `sdf_mixed_oracle.py` separately checks the named-only whole file,
two records, source SHA and ordinary NOTE values. Existing DT-only/duplicate
full-supplier limitations remain explicit; the supplier is not a raw retention
oracle. Literal ranges/delimiters have separate assertions. No existing oracle
or fixture was changed. The old 690/777-byte fixtures separately verify local
ID remapping, H3 literals, source identity and translation to two display rows.

Primary sources, already pinned in the source register and committed plan:

- [BIOVIA Chemistry2026 CTfile Formats](https://www.3ds.com/assets/invest/2026-01/biovia_chemistry_2026_ctfile_formats.pdf),
  printed 9–13,64–70,97–99; PDF SHA-256
  `959f2bbf5487e00fa7535ea7ee36f7ce04f39ea72920844641952276de146c94`.
  It defines source coordinates, declared valence/query H and mixed framing,
  not this complete normal-valence display algorithm.
- [OpenSMILES pinned organic subset](https://github.com/opensmiles/OpenSMILES/blob/9240a2887158699503605bc838e1de9a9f7883f4/opensmiles.asciidoc#L275-L309),
  raw SHA-256 `00f0423da612de913770133aa13e456d0cebc562dd50166534a8a7f9a834bd21`,
  and [RDKit's valence documentation](https://www.rdkit.org/docs/RDKit_Book.html#valence-calculation-and-allowed-valences)
  support the deliberately closed default profile. SMILES rules and a consumer
  table are not universal MOL semantics; alternative valences remain deferred.
- [IUPAC GR2.1.1/2.1.5](https://iupac.qmul.ac.uk/drawing/drawing.html#GR211),
  cached HTML SHA-256
  `32d2c2d694059e78b6b6a3ed8c274770377e73058d11c174951e7b4e1ce4f71e`,
  explains why counts and faithful label attachment are distinct prerequisites.
  No reference prose, diagram, implementation or asset was copied.

## Actual RED/GREEN evidence

All Cargo commands used pinned Rust 1.94.1, the absolute worktree target,
`CARGO_INCREMENTAL=0`, `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0`,
and the existing pinned `BONDLINE_PYTHON` interpreter. No dependency/lock change.

```sh
cargo +1.94.1 test --locked -p bondline-io --test sdf_preview valence_absent_records_resolve_display_h_without_authoring_it
cargo +1.94.1 test --locked -p bondline-io --test sdf_preview
cargo +1.94.1 test --locked -p bondline-io
cargo +1.94.1 clippy --locked -p bondline-io --all-targets -- -D warnings
cargo +1.94.1 fmt --package bondline-io -- --check
```

1. Initial real-file test, session 20654: exit101, first E0432 missing
   `bondline_io::sdf::preview`; later inference errors cascade from the missing
   API. `target/sdf-preview-model-red.log` retains the complete RED output.
2. Before implementation, actual fixture bytes faced the unchanged external
   molecular/named-only consumers: expected ranges, CCO/CN and H counts passed.
   Receipt: `target/sdf-preview-model-fixture-oracles.json`.
3. First production GREEN, session 46787: exit0, all 11 initial focused tests
   pass. `target/sdf-preview-model-first-green.log`.
4. After the memory refactor and additional direction/source-lease regressions,
   session 20011: exit0, full IO suite **271 passed**, including **12 new preview
   tests**, zero failed/ignored. `target/sdf-preview-model-io-test.log`.
5. All-target IO clippy, session 47652: exit0, warnings denied.
   `target/sdf-preview-model-clippy.log`. IO rustfmt check: exit0,
   `target/sdf-preview-model-fmt.log`.

Tests cover the original ordinary/mixed raw input, old complete sources,
authored None/literals/unusual counts/explicit zero, all closed derived
elements, physical H, integer bond orders, branch/ring/opposed directions,
late unsupported records, empty records, metadata/3D/query refusal, over-valence,
zero-length bonds, page growth/limits, cross-record aggregate caps, cancellation,
source lifetime and metadata-only Debug. No test assertion or quota was weakened.

The protected-before manifest tracks 97 old source/oracle/fixture/config files.
Only `sdf.rs` changes, by the two additive module/reexport lines; removing those
reproduces its old SHA-256
`fbdb2b3e0b96c9849778c6f23a0dbad9975a74df5d60127915cfeac6055d9b2c`.
All other protected bytes remain exact. The ignored frozen handoff records
source/log hashes and terminal handles for Root's independent receive/review.

No command, UI/cache/job wiring or native SDF display is delivered here. The next
step is the real renderer entry and source/revision-bound shared capture/UI
integration from the committed plan, followed by actual scene and native
verification when desktop availability permits.
