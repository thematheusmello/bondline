# Independent simple-ring SMILES layout acceptance

This is the pre-product test declaration from `492905a`, not a delivered layout or native
workflow claim. The additive proposed API is
`SmilesDocument::layout_neutral_simple_ring(length, origin, page, &Cancellation)`.
The original path/acyclic APIs, scripts, fixtures, locks and approved golden remain exact.

The agreed original profile is one connected graph with exactly one simple3–8 atom cycle,
all cycle edges Single, at most64 total atoms and attached acyclic trees. It reuses the
closed neutral nonisotopic H/C/N/O/F/Cl/Br/I fixed-H interpretation, including physical-H
nodes and supported Single/Double/Triple attached edges. It does not add general ring
chemistry, aromaticity, cyclic multiple bonds, stereochemistry, fused/spiro/bridged graphs,
large rings, component packing or source-backed SMILES editing authority.

Origin specifies the minimum x/y of **all atom centers**, after deterministic local
layout. Every bond retains the requested length; no fitting scale or H removal is allowed.
The same ordinary literal/native drawing scene must equal the resolved-H attachment/paper
scene before acceptance. A different safer preview cannot certify an unsafe actual drawing.

## First declaration and intended RED

The first3,231-byte test, SHA256
`122bdee99c4619c940f2ce417856e0324687feb45cae5e8c83e7340391ef4073`, declares original
`C1CCCCC1\r\n`: six C/H2 atoms, dense IDs, five directed sequential edges and closure
1→6. It checks all six64pt lengths,120° interiors,128pt opposite chords, minimum-center
origin and complete raw/IR retention. It uses no normalization result as expected data.

Actual14483 exits101 solely with E0599, missing `layout_neutral_simple_ring`. The4,001-byte
log SHA256 is `138dcd36efbe354481bc13e6f8bc04bb9bec39ee5d30205fd62518ab51651097`.
The expanded declaration retains that entire prefix byte-for-byte.

## Eight independent cases

1. The literal regular cyclohexane above.
2. Every supported ring size3–8, literal C/H2 graph, equal edges/radius/interiors, convex
   closure, preferred hexagonal vertical-left edge and other-size horizontal-bottom edge.
3. One substituent on each size follows the exterior radial bisector, snapping to a30°
   multiple only when within5°. This computes the actual radial direction rather than
   guessing a source-ID phase.
4. Equal geminal60° pairs and the square90° alternative; unequal hexagonal ethyl/methyl
   uses larger radial/smaller±60°. Two directly attached branched roots use exterior
   trisection: the six-ring example has an80° external pair and radial bisector. Ordinary
   symmetry±30° is not treated as an IUPAC mandate.
5. Twelve literal graph/H/formula vectors cover methyl/exocyclic C=C/C=O, acetyl and
   nitrile chains, piperidine/morpholine, physical H, contextual geminals, a real ring
   expressed across a dot and a qualified unknown internal Double outside the cycle.
   Actual source bond order/direction and every atom field remain exact.
6. Repeated output, rigid origin translation, bracket/ring-number equivalents and source
   pointer/range/chain-start/IR retention. A ring digit is never mistaken for a cycle.
7. Refusals cover acyclic input, multiple components/cycles, fused/spiro/bridged/large
   rings, cycle Double/Triple, aromatic/charged/isotopic/radical-ambiguous/outside-profile
   input and unsupported internal Double context. Length/cancel/cap codes are literal.
  65 nodes (including leading unsupported P) must hit `SimpleRingAtomLimit` before
   normalization.64 valid nodes with impossible negative minimum origin must reach the
   geometry stage, not the atom ceiling. Four paper edges also test glyph/stroke refusal.
8. Actual generated CDXML faces the new ring consumer and unchanged general Expat consumer,
   then reopens byte-exact and fully mapped with the same ordinary editable native model.
   Wrong-H and wrong-length negative controls must refuse. This alone is not a command,
   UI, file-publication or arbitrary-edit geometry certificate.

The broader fixtures use explicit64pt/144pt/1000pt-square reference settings to isolate
topology/angle correctness from accidental default-page fit. The first C6 case still uses
A4. These are original test policies, not reference defaults or a promise that every
64-atom graph fits. Scene checks are1024² CPU/attachment checks; no new GPU or native claim
is inferred. If a renderer prerequisite blocks a declared positive, report it rather than
silently narrowing the profile or changing the expected graph/H/template.

## New independent external consumer

`python -B crates/io/tests/smiles_ring_oracle.py ORIGINAL_SMILES [OPTIONS_JSON]` reads actual
CDXML from stdin. The closed optional keys are `atoms`, `bonds`, `formula`, `length`,
`origin`, `page`, matching the acyclic consumer. Output keeps `atoms`, `bonds`, `formula`,
truthful `stereo_none`, `no_assigned_stereo`, actual `bond_stereo`, `physical_h`, and adds
`ring_size`. Defaults remain48pt/(72,72)/A4 when not overridden.

The new script retains the project-owned general raw/metadata/reference/H/chemical/stereo
checks in a separate profile; the old script is not widened. Independent source RDKit
one-ring perception plus connected E=N identifies the actual cycle, including cross-dot
notation. Raw Decimal geometry independently checks every bond and closure, circumradius,
interior angles, convexity, orientation, radial/contextual branches, nonincident crossings,
vertices on unrelated bonds, minimum origin and paper bounds. Cycle multiple bonds refuse.
Attached unknown Double retains the reviewed antiparallel source-context guard; assigned
bond/atom configurations remain forbidden. External RDKit is never linked or shipped.

Pinned independent qualification47543 exits0: all twelve literal source vectors match
RDKit2026.09.2 elements/H/formula/edges/cycle IDs. An independently authored regular-hexagon
XML seed passes. Ten negative controls cover H, requested length, actual coordinate,
missing closure, unit-length-but-nonregular geometry, incorrect hexagonal orientation and
unsupported source topologies/orders. These are actual-file **oracle** controls, not
Bondline product outputs or layout GREEN. No new fixture or image is shipped.

Expanded no-run21181 exits101 only at ten E0599 missing-method/variant errors. This is an
absent-API compile stage, not eight runtime failures. All190 protected original identities
are checked before any product receipt. Ignored `target/smiles-ring-layout/` keeps original
declarations, exact pins, first REDs, source vectors, manual XML controls and qualification.

## Source basis and limits

The existing public [IUPAC2008 HTML](https://iupac.qmul.ac.uk/drawing/drawing.html) is the
347,564-byte cached source SHA256
`32d2c2d694059e78b6b6a3ed8c274770377e73058d11c174951e7b4e1ce4f71e`.
GR3.3.1/3.4.2 supports regular small-ring and preferred-edge conventions; GR4.2.1/4.2.3
supports the stated contextual substituent families. The source does not prescribe a
unique whole diagram. Exact source-ID phase, bounded search, atom ceiling, minimum-origin,
font/viewport and initial symmetric candidate choices are original Bondline policy.
GR3.3.2's larger-ring requirements remain outside this profile rather than being flattened
to regular polygons. The pinned OpenSMILES source and qualified RDKit/source distinctions
from the acyclic record remain applicable; no upstream code, template, figure or font is
transplanted. Original script reuse is project code, not proprietary source.

No product code, dependency, old oracle/fixture/golden, native input, Actions or Git change
was made by this test owner. No API/backend GREEN, publication or native platform result
is claimed at this declaration checkpoint. All own jobs are terminal.

## Added ring-boundary coverage after the first implementation

One ninth case is appended after the complete original28,493-byte source, with every
original assertion and the twelve-example table unchanged. It declares
`CC(C)(C)C1CCCCC1` (C10H20): dense atom2 is an off-ring degree-four carbon; its ring
neighbor5 reaches the cycle5–10 with closing edge5→10. This exercises the new boundary
signature guard that a parent-only tree traversal cannot safely evaluate through a cycle.
Literal H counts are `[3,0,3,3,1,2,2,2,2,2]`; metadata and directed edges are explicit.

The case checks supported four-coordinate angle families, full regular-ring/Scene
certificates, source/IR retention, actual CDXML through the unchanged new ring consumer,
and exact native reopen/classification. Pinned independent RDKit confirms the literal
elements/H/full graph/formula and the off-ring/cycle identity distinction. The original
ring oracle remains byte-exact. This is added coverage after implementation, not an
invented product RED; no runtime success is claimed before the actual backend is received.
An initial append patch did not match a rustfmt-wrapped function signature and changed
no file; the subsequent exact EOF append proves the entire old source prefix unchanged.
