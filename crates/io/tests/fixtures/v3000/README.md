# Original V3000 vectors

`noncontiguous-cc.mol` is original MIT OR Apache-2.0 test data, not a copied vendor
example, sample library or reference application output. Exact source is 344 bytes,
SHA-256 `1867a5904e742030431b523a31167951ff998007a89b9c3d94ec88cf7d4c4380`.
Its atom source IDs 10/40 and bond source ID 7 deliberately differ from dense model IDs.
VAL 4 minus one single bond gives three authored hydrogens per carbon. At 24 points/raw
unit and origin(72,72), coordinates map to (42,60)/(78,60) drawing points. Git -text
preserves exact fixture bytes on Windows.

`continuations-and-extensions.mol` is a second original syntax vector (492 bytes,
SHA-256`ccbdcb1a92091a440556268220f533459b239ef5f538e48130cdd2fe906ea267`).
It continues a quoted unknown attribute across physical lines, decodes a doubled
quote and quoted counted-list values, and retains a stereo COLLECTION block. The
strict projection refuses it; both original files seed the isolated mutation corpus.

Grammar/field facts come from official BIOVIA Chemistry2026 CTfile Formats:
<https://www.3ds.com/assets/invest/2026-01/biovia_chemistry_2026_ctfile_formats.pdf>,
printed 7–10/12–13/32; PDF zero-based10–13/15–16/35. Cached PDF SHA-256 is
`959f2bbf5487e00fa7535ea7ee36f7ce04f39ea72920844641952276de146c94`.
Rust tests construct additional original continuation/quote/list/charge/radical/
valence/query/unknown-block/hostile vectors from those field facts. No proprietary
file is incorporated.

`../v3000_oracle.py` runs pinned external RDKit 2026.09.2, never linked/shipped.
Its generated 314-byte isotope/eight-decimal vector and 49,058-byte1000-atom vector
stay in process memory or ignored target/, not this fixture directory. The oracle
pins hashes and verifies graph/property/coordinate interoperability. Raw RAD1/3
and literal tokens remain independent because RDKit collapses singlet/triplet to
two radical electrons and discards unknown properties. It also truncates MASS 13.5
and rejects nonzero legacy counts ignored by the primary V3000 specification;
these are explicit diagnostics, not Bondline behavior to imitate.
