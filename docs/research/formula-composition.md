# Resolved-model composition prerequisite

2026-11-17; base `8e030bf`, isolated `feat/formula-composition` worktree. This
locks an additive model-only analysis before implementation. It does not admit
new source data, infer hydrogens or validate chemistry, produce formula text or
masses, or complete M3. No registry, UI, file format or dependency changes belong
to this increment.

## Sources and conventions

- [OpenSMILES v1.0](https://raw.githubusercontent.com/opensmiles/OpenSMILES/9240a2887158699503605bc838e1de9a9f7883f4/opensmiles.asciidoc),
  commit `9240a2887158699503605bc838e1de9a9f7883f4`, sections 3.1.2–3.1.4 and
  3.6: attached-H counts and separate H nodes are different representations;
  count-specified H has unspecified isotope and neutral charge. Numbered
  isotopes remain distinct from unspecified natural isotope composition. Its
  notation accepts isotope numbers without establishing that the nuclide exists.
  Only these semantic facts inform original code; no specification prose/assets
  or GFDL implementation material is included in the product.
- [Published CDXML NumHydrogens](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Atom_NumHydrogens.htm),
  archived SDK mirror, description/absence rules: inline label H excludes
  separately drawn H nodes; source omission can require valence interpretation.
  Bondline's unresolved model `None` therefore cannot safely mean zero.
- [RDKit MolOps API](https://www.rdkit.org/docs/cppapi/MolOps_8h_source.html),
  `getFormalCharge` and `getMolFormula`: the former sums atom formal charges;
  the latter explicitly separates isotope handling from default collapsed
  formula output. `CalcMolFormula(..., separateIsotopes=True,
  abbreviateHIsotopes=False)` is an external test tool, not shipped logic or
  Bondline's future formatting policy. Exact installed version and cached source
  hashes are recorded in the verification note/ignored receipts.

The current IUPAC Gold Book term endpoints returned HTTP403 and the old mirror
HTTP502 during research. No inaccessible definition or mass table is relied on.
Formula display ordering, isotope typography and empirical reduction are deferred;
this API exposes numeric composition, not a molecular-formula string.

## Locked model/API behavior

`analyze_fragment(&Fragment, cancelled: impl Fn() -> bool)` returns a checked,
immutable `Composition`. It exposes elemental totals, isotope-resolved bins
including an explicit `None` bucket, and `total_charge()` in elementary-charge
units. Counts are `u64`; charge is `i64`. Public count records contain validated
`Element`, optional positive `NonZeroU16` isotope and count as applicable. Results
are deterministically sorted by atomic number, then `None` before increasing mass
number. Zero-count entries are omitted. This is an original numeric API order,
not Hill/display order. No document/page wrapper is added without a real caller.

Each model node contributes one atom of its element/isotope. Its
`Some(explicit_hydrogens)` contributes that many isotope-unspecified, neutral H
atoms in addition; physical H nodes are counted once through their own records.
Every atom, including physical H, must have `Some(H)`. The first unresolved atom
in authored array order produces `CompositionError::UnresolvedHydrogens(AtomId)`;
no result with silently missing H is returned. A physically implausible but
structurally valid explicit count remains literal data: analysis is not valence
checking, and those controls are not advertised as chemically valid molecules.

`Atom.isotope=None` is never replaced by a predominant isotope. Explicit 1H/12C
remain separate numbered bins. Inline H does not inherit its parent's isotope.
All 118 model elements and all positive-u16 isotope values are representable;
there is no isotope-existence/abundance/mass lookup. Atom charges are summed
exactly, including negative values, independent of radical, bond order and
connectivity. Geometry, IDs, array order, labels and every input field are retained
without mutation. Empty/disconnected fragments are valid counting inputs; equal
composition does not imply equal topology or chemical identity.

Cancellation is checked before work, throughout the atom scan and before
publishing a result. Typed cancellation and checked-arithmetic overflow are
failures, never partial results. The full model bound is 100,000 atoms/bonds,
not the unrelated 64-atom authoring bound. In particular 100,000 physical-H nodes
each carrying the literal maximum inline count total 6,553,600,000 H; `u32` is
insufficient. Such extreme controls establish arithmetic, not chemical validity.
At most one extra unspecified-H bin is introduced beyond the input isotope bins.
No topology search is needed; bounded map accumulation uses O(n log n) time and
O(n) storage under the existing model bound.

## Acceptance and integration boundary

Freeze a literal public-API test and observe its missing-API compiler RED before
production. Add literal isotope/physical-H/charge, unknown-H and cancellation,
empty/isomer, all-element/maximum-field and 100,000-item controls. Qualify an
original external RDKit checker against manually declared positive and semantic
negative inputs before giving it analysis output. Use explicit-bracket reference
molecules with implicit H disabled/absent; high-level RDKit formula descriptors
and independent expected bins jointly check that no H or isotope is lost.

Later commands must independently guard complete source and the selected graph
before calling this library. This function neither resolves source authority nor
authorizes Save, source regeneration or valence inference. It owns no persistent
cache or snapshot; future retained/cached analysis would need its own accounting.
