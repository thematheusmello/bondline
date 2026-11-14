# Source-aware MOL drawing display

2026-11-14. This is the shared IO producer from
[the MOL display plan](research/mol-display-plan.md). It does not wire a command,
UI graph acquisition or native workflow. No GUI/input/activation, reference
runtime or GitHub Actions ran; the user is using the desktop and Actions stays off.

## Delivered contract

```rust
bondline_io::display::drawing_page_scene(
    width: u32,
    height: u32,
    stored: &StoredDocument,
    page_id: PageId,
    cancel: &Cancellation,
) -> Result<GraphScene, DisplayError>
```

The producer borrows the complete owner, checks cancellation and source
completeness, and selects the actual current Page by its existing PageId. It does
not reparse, remap IDs, translate, pack, resize or replace any authored graph.
Only actual `stored.mol_version().is_some()` backing selects resolved display H;
an associated filename, file-format choice or metadata string is not an input.
Source-free and byte-proven native CDXML drawings use the unchanged literal
renderer, including its None behavior and original renderer error messages.
An incomplete retained source refuses before returning any partial scene.

MOL display reuses the original closed neutral C4/N3/O2/F1/Cl1/Br1/I1 policy,
subtracting checked integer bond-order sums. Derived counts are transient,
private facts, not authored H, a normalizer or a source encoding authority.
Some(count), including zero and unusual literal counts, remains authoritative
for all 118 modeled elements; None remains None. Literal counts are not a claim
of chemical validity. Charge, isotope, radical, unresolved non-profile H,
coincident bonds, unsupported geometry or rendering remain typed refusals.
Double/Triple contribute their actual weights to the common data resolver, but
the renderer still refuses those orders. No unsupported order becomes Single.

The common topology/count resolver now lives privately in display.rs.
`HydrogenOrigin` moves there with an unchanged public reexport from
`sdf::preview::HydrogenOrigin`, so old SDF callers keep the same type/path.
SDF wraps the shared failures back into its original PreviewError variants,
codes, static features and source record context. Its strict MOL mapping,
aggregate caps, packing, page growth, ID remapping and all existing tests stay exact.

One necessary private policy distinction preserves existing compatibility:
MOL drawings accept explicit physical-H zero as the literal renderer already
could, including an isolated H0. Unknown physical-H counts still require exactly
one Single bond and no attached H, and contribute once to the neighbour sum.
SDF retains its original stricter exactly-one-Single requirement even for literal
H0. This is a declared original display-profile distinction, not newly inferred
universal chemical semantics. A real RED exposed the narrowing before this fix.

The returned GraphScene contains paint/picking data, not a source lease. Its
caller remains responsible for retaining the immutable owner during deferred
work and rejecting stale revision/generation/view results. No model, source,
file association, saved/history state, edit permission or Save guard changes.

`DisplayError::code()` exposes cancelled, not_found, unsupported_content,
resource_limit and the existing render mappings. MissingPage carries PageId.
InvalidDimensions maps to invalid_params, LabelScale to unsupported_scale,
LabelPlacement/LabelOverlap/UnsupportedChemicalContent to unsupported_content;
other renderer errors retain render_failed. In particular the existing native
InvalidViewport distinction is not relabeled. Render sources remain typed.

Before allocating MOL topology or resolved-H maps, a cheap saturating sum of
current Page atoms plus bonds preserves the renderer's public MAX_HIT_REGIONS
4096 preflight, with cancellation checked per fragment. Over-limit pages return
the same Render(SceneTooComplex) even if an earlier atom has unresolved H or a
later bond has coincident endpoints. This is an existing bound, not a new budget;
the renderer still validates primitive count, attachment, paper ink and scale.

## Independent expectations and provenance

The unchanged original 740-byte SDF fixture has SHA-256
`e5cbdc14329c59bad55148c5de37058de90f651af10966c30cf8b89b3b5efd39`.
Only its literal raw MOL spans are opened here:

| Raw span | Actual version | Default individual-MOL coordinates, pt | Independent H |
|---|---|---|---|
| 0..371 | V3000 ethanol | (42,60), (78,60), (78,24) | 3,2,1 |
| 410..713 | V2000 methylamine | (108,108), (72,72) | 3,2 |

The actual source bytes face the unchanged external mol_oracle.py using the
existing pinned Python/RDKit. Literal expected SMILES/formula are CCO/C2H6O and
CN/CH5N; original raw coordinates and per-atom H are asserted independently.
The same actual bytes then pass the new producer, exact source passthrough and
unchanged cross-format refusal. No parser/writer round trip supplies expectations.

Tests compare actual GraphScene and CPU pixels with the existing resolved
renderer fed independently specified counts. The geometry/count assertions and
external consumer establish the input facts; the renderer comparison establishes
routing and does not independently validate its glyph rasterizer. Actual atom
hits retain original IDs. These are offscreen CPU checks, not GPU/native proof or
a new approved image. Old renderer/glyph/oracle/golden bytes remain unchanged.

Additional tests cover all 118 literal-zero elements, unusual H14 and a current
u16::MAX literal without weakening the MOL VAL encoder limit; actual custom
frame/page and non-dense current IDs; source-free/native CDXML literal None;
known physical H; opaque CDXML/unregenerable MOL refusal; metadata/overvalence/
unknown-H/multiple-order refusal; pre-cancellation, missing PageId, scale, paper
ink refusal and an empty MOL page. Above-hit-budget source cases use both 4097 atoms and
2049 atoms plus 2048 bonds, proving the existing bound precedes derived topology
failures. Literal H0 compatibility is checked against
the old renderer and the separately unchanged SDF refusal.

The shared policy's existing public primary sources remain those recorded in
[the source register](clean-room/sources.md) and
[original SDF data evidence](verification-sdf-preview-model.md):

- [BIOVIA Chemistry2026 CTfile Formats](https://www.3ds.com/assets/invest/2026-01/biovia_chemistry_2026_ctfile_formats.pdf),
  printed 9–13 and 64–70, registered PDF SHA-256
  `959f2bbf5487e00fa7535ea7ee36f7ce04f39ea72920844641952276de146c94`:
  actual coordinates, version and declared valence differ from this original
  closed default-H display policy.
- [OpenSMILES v1.0 organic subset at immutable 9240a288](https://github.com/opensmiles/OpenSMILES/blob/9240a2887158699503605bc838e1de9a9f7883f4/opensmiles.asciidoc#L275-L309),
  registered raw SHA-256
  `00f0423da612de913770133aa13e456d0cebc562dd50166534a8a7f9a834bd21`,
  and [RDKit primary valence documentation](https://www.rdkit.org/docs/RDKit_Book.html#valence-calculation-and-allowed-valences):
  support the restricted neutral profile, not universal MOL perception.
- [IUPAC2008 GR2.1.1 and attachment rules](https://iupac.qmul.ac.uk/drawing/drawing.html#GR211),
  registered cached HTML SHA-256
  `32d2c2d694059e78b6b6a3ed8c274770377e73058d11c174951e7b4e1ce4f71e`:
  faithful H labels and attachment motivate retaining the actual source directions.

The public primary pages were re-opened during this slice; the listed hashes
are the prior registered raw pins, not new hashes of a web-rendered extraction.
No upstream/proprietary source, help prose, diagram, font or asset is copied.
No dependency, lockfile, parser, writer, old fixture, oracle or approved golden changes.

## Actual TDD and package checks

Commands use Rust 1.94.1, the absolute worktree target, CARGO_INCREMENTAL=0,
CARGO_PROFILE_DEV_DEBUG=0, CARGO_PROFILE_TEST_DEBUG=0 and the existing pinned
BONDLINE_PYTHON interpreter. Exact logs and source hashes are frozen under target.

1. Intended missing-API RED 11132 exits 101 with E0432: bondline_io::display is absent.
   The actual two-version coordinate/H test is already present.
2. Minimal producer GREEN 6447 exits 0 for that test, without a stub.
3. Literal isolated-H0 RED 36778 exits 101 after the old literal renderer succeeds:
   the shared strict SDF physical-H topology would narrow drawing compatibility.
4. The private physical-H policy correction gives GREEN 20573, two tests passed.
5. Expanded 94805 exits 101 with nine passes and one test-input error: V3000 already
   refuses an 81-character header. The new incomplete-source case is corrected to
   the existing accepted-but-unregenerable V2000 header profile; no production
   behavior or assertion is relaxed. Expanded 28031 then exits 0, all 10 pass.

6. First full package run 6019 exits 0: 281 passed, zero failed/ignored, including
   all 12 existing SDF preview tests and the 10 new MOL display tests.
7. Clippy 88828 exits 101 on six unnecessary static-error closures after removing
   record context. Only those eager/lazy constructors are refactored; no check,
   static feature, result code or bound changes. Final all-target clippy 2031 exits
   0 with warnings denied; rustfmt check exits 0.

8. Review identified a material ordering risk: H resolution could precede the
   renderer's cheap hit budget. New RED 45353 exits 101: old SceneTooComplex but
   new unsupported_content on a fully mapped 4097-atom MOL with unresolved P.
   The existing-bound preflight gives GREEN 35521, including the separate combined
   2049-atom/2048-bond case. No quota or old validation is weakened.

9. Final package run 60534 exits 0: **282 passed**, zero failed/ignored, with all
   11 new MOL display cases and every existing SDF test unchanged. All-target
   warnings-denied clippy 16510 and IO rustfmt check exit 0 on the final source.

```sh
cargo +1.94.1 test --locked -p bondline-io
cargo +1.94.1 clippy --locked -p bondline-io --all-targets -- -D warnings
cargo +1.94.1 fmt --package bondline-io -- --check
```

The protected-before manifest records 298 existing source/test/oracle/config
inputs. Exactly the two owned lib.rs and sdf/preview.rs files change; the other
296 remain byte-exact. The entire SDF prepare/packing/projection tail stays exact
(8698 bytes, SHA-256
`2bce19bc061b7d4198966fc1c8ca9c5044ae256c61c6a27176e596d461a2ca7d`), as do its
public PreviewError enum/code/context implementations (1988 bytes, SHA-256
`cd52467d505babb19eccf4c1542edda2013e16dd37aa9b8454a1b6bd73612b52`).
Final source/log/diff hashes and terminal handles are in the ignored frozen handoff.
No hard RSS or native-thread latency guarantee is inferred from these bounded checks. No command, UI graph acquisition, GPU/native acceptance, general chemistry,
full CTfile parity or whole-M1 completion is claimed by this data producer.
