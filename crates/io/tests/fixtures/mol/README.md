# MOL V2000 test provenance

`rdkit-CH3.mol` is the already reviewed RDKit methyl-radical corpus document, copied byte
for byte from `Code/GraphMol/FileParsers/test_data/CH3.mol` at commit
`c686c2c945df370c2f95ebe20aae2872a36f7c17` (170 bytes, SHA-256
`afd2352697ca20c0d580c4981fe8bd24da36c4e1661e9d2aee501a40364fd42d`). Its BSD-3-Clause
notice is retained at `docs/corpus/rdkit-license.txt`; `docs/corpus/manifest.json` records
the approved source. This is a chemical parser regression, not a proprietary sample library.

The fixed-column strings in `tests/mol.rs` and the Python consumer are original
MIT OR Apache-2.0 test code. Numeric field decisions follow the official BIOVIA Chemistry
2026 CTfile Formats, printed pages 62, 64–70 and 90:
https://www.3ds.com/assets/invest/2026-01/biovia_chemistry_2026_ctfile_formats.pdf
The copyrighted PDF and rendered reference tables remain ignored research artifacts.

The specification says that an M ISO property supersedes all isotope values in the atom
block and that absence from that property means natural abundance. The original mixed
property vector tests that normative mapping. External RDKit 2026.09.2 instead retains a
legacy mass difference on an atom omitted from the M ISO list. A separate diagnostic
records this exact input discrepancy; it is not called input-round-trip agreement.
Canonical output uses zero legacy mass-difference fields and absolute M ISO entries, and
the external reader then agrees with the normative model. Exact raw passthrough is never
normalized. Charge and radical global precedence agree in the focused external probes.

Element-symbol facts follow IUPAC's 4 May 2022 periodic table:
https://iupac.org/wp-content/uploads/2022/05/IUPAC_Periodic_Table_A3-04May22.pdf
Only atomic-number/symbol facts are implemented, not its artwork, masses or isotope tables.
All 118 symbols are independently checked against the pinned external RDKit consumer.

## Original ethane vector

ethane-v2000.mol is an independently authored MIT OR Apache-2.0 fixed-column test vector,
not output from our writer or a reference application. Two carbon atoms at raw(0,0,0) and
(1.5,0,0), one single bond and declared valence4 encode CC/C2H6. Independent RDKit checks
its graph, hydrogen counts and coordinates. It is also copied to an ignored writable fuzz
corpus in CI. Source field definitions are the same official CTfile2026 pages above.

SHA256 `828c1b58b433ed3778bdb0ce7e7d02315f45850507c976a55c82fcebeebf57fc`, 282bytes.
