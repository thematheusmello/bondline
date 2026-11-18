# Explicit-style atom label creation: renderer receiving record

2026-11-18, locked renderer increment on base `7113237`. This is a pure-data/layout
prerequisite for printed-label creation, not the source writer, command or native workflow.

The public [Node specification](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Node.htm),
description paragraph beginning “When reading files”, states that a contained Text object
is used for the displayed label; chemical properties drive label derivation when Text is
absent. It was read online before the change. Keeping stored carbon-display flags dormant
for explicit child Text follows this precedence. No foreign style/default, valence,
source capability or reference-pixel rule is inferred from it.

`create_atom_label(&Atom, TextStyle) -> Result<AtomLabel, Error>` requires no existing
authored label and a deliberately supplied supported Roboto/face0 style. A private typed
element-token seed delegates to the existing canonical body, whole-element-token anchor,
GPOS and exact Bezier-bound generator. The seed never enters a document. The returned
label has no child point, Left/Left, no Alignment, exact supplied style and generated ink
box; input IDs, coordinates, Element/H, flags and every other atom field remain unchanged.
It neither infers H/defaults nor grants source, physical-page or mutation authority.

Only two shared depiction restrictions change: authored layout no longer rejects flag
presence alone, and the skeletal classification loop skips atoms that already have child
Text. Its complete atom/adjoining maps remain intact, so other hidden carbons still see
the current degree, carbon neighbours and mandatory angles/C2 exceptions. The labelNone
classification code, page branch order, geometry, raster, hit/mask ownership, work budgets,
scale/paper/ink checks and cancellation paths remain unchanged.

The former flag-only Cl negatives in `boxed_label_geometry.rs` are deliberate positive
upgrades using their same inputs: they now require valid raw body/box and an identical
generated label. Every other old negative and numeric tolerance stays exact. Old foreign
Boxed/Connected/Mixed source profiles require their separately owned explicit authored-
flag absence fences when this shared renderer prerequisite is integrated. The new flagged
source capability is a separate IO change; this lane does not claim those source gates.

## Independent declarations and actual RED/GREEN

The existing-API [printed flag tests](../printed_flags.rs) were frozen before producer
changes, SHA256 `77057035a0d0ebb80e5fe3b70c8ab5ab9c3d38e9a27998fb27e7c1c690ff0bda`.
On unchanged `7113237`, they actually execute and fail 0 passed / 2 failed / 0 ignored
with UnsupportedChemicalContent. Their unchanged GREEN covers all nine None/false/true
flag pairs on independently boxed CH4 and Cl, exact unflagged-reference CPU pixels and
input metadata, plus a printed carbon whose dormant false flag would fail C4. The second
unprinted carbon in that isolated C2 component must remain mandatory Visible; deleting
the printed neighbour from incidence would fail this test.

The [new API declarations](../atom_label_creation.rs), final pre-producer SHA256
`ee57f37d0441ad853e7871a1d007345eeae58a0a0f2c2c1373bde2089e8e4c6e`, have a separate
missing-API **compiler RED**, E0432 with consequent E0282; no runtime failure is claimed
for an absent function. The tests reuse all 156 existing independent FontTools box vectors
under all nine flag pairs, with a deliberately supplied different green RGB. They require
the exact body/style/placement, independent bounds within the original 1e-10 comparison,
and unchanged input. A second test refuses existing label replacement, unknown font/face,
missing point/H, charge, isotope, radical and attached H on a physical H atom. These tests
are unchanged after implementation. No new fixture/font/reference image is generated.

Focused creation/flag/old-box checks pass 7/0/0 after the producer. Logs are retained in
ignored `target/label-creation/render/printed-flags-red.log`, `missing-api-red.log` and
`focused-green.log`.

The complete `cargo test -p bondline-render` run passes 195/0/0 across 40 test executable/
doc-test reports, including all 191 prior cases. `cargo clippy -p bondline-render
--all-targets -- -D warnings`, `cargo fmt --all -- --check` and `git diff --check` pass.
The full and lint logs are `target/label-creation/render/full-render.log` and `clippy.log`.
The existing external font/pixel tests use the pinned format-Python environment; the new
tests reuse frozen data and need no external program. No full workspace or native result
is claimed by these renderer-scoped gates.

Source insertion/resources/IDs, complete file/model/history, current hidden targets,
subsequent edits, all-labelled transitions, native controls and platform execution remain
separate integration work. No dependency, asset, font, old golden or Actions change.
