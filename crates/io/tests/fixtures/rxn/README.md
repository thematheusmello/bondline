# Original legacy RXN regression vectors

These are independently authored Bondline test documents, licensed MIT OR Apache-2.0.
They were constructed from the fixed-field definitions in BIOVIA Chemistry 2026 CTfile
Formats, not copied from a vendor sample, template, diagram or application output.
The official specification is linked in docs/clean-room/sources.md (CTFILE2026/RXN).

| File | Bytes | SHA-256 |
|---|---:|---|
| mapped-identity-v2000.rxn | 696 | cadf07801a6516c416bc43823c81b3e2f690c85bc0614d80efa13f5e138093c8 |
| mixed-mol-v2000-wrapper.rxn | 739 | 5b8fca7c1c008a272680557ea04ca37f5e48d7b198b4412bd9ab8b3fe33b8403 |

The first vector was generated independently before the Rust writer: Python formatted
published I3 counts and f10.4 coordinates, then checked atom-line width/map columns.
It contains one reactant carbon, one product carbon and one nitrogen reagent. Both
carbons carry atom-map 1; nitrogen carries map 0. An original opaque M XYZ property
on the reagent tests whole-payload preservation; its meaning is not asserted. The
wrapper uses all three numeric count fields, the published role order and literal
metadata. No chemistry execution, reaction balancing or stereochemistry is implied.

The second vector wraps the project's original noncontiguous-ID V3000 two-carbon MOL
and original V2000 ethane MOL, unchanged. Those fixtures retain their own provenance
in ../v3000/README.md and ../mol/README.md. Its outer container is legacy RXN, its
embedded molecule versions differ, and its exact six-character counts omit a zero
reagent field. These two accepted extensions are independently observed in RDKit
2026.09.2 output; the primary legacy RXN pages do not establish mixed MOL encodings
or an omitted third field as normative. Canonical writing adds the explicit zero
reagent count while preserving both payloads exactly.

crates/io/tests/rxn_oracle.py checks independent raw spans/fields plus actual RDKit
molecule and reaction consumers. Generated external vectors remain in ignored cache,
not this fixture directory. RDKit reaction output requires separateAgents=True to
retain reagent roles; default folding and permissive trailing-data parsing are only
diagnostics. Neither consumer is used as an opaque-data fidelity oracle.

These files are exact-byte inputs: .gitattributes disables line-ending conversion.
Copy seeds to ignored .cache/fuzz/rxn before mutation. Never make this directory the
writable fuzz corpus. The parser/writer does not map reactions into the application.
