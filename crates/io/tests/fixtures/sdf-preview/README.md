# Original SDF preview source

SPDX-License-Identifier: MIT OR Apache-2.0. These are original Bondline test bytes;
the repository's [MIT](../../../../../LICENSE-MIT) or
[Apache-2.0](../../../../../LICENSE-APACHE) terms apply at your option. No vendor
example, application output, sample library, artwork or template was copied.

`valence-absent-mixed.sdf` contains a V3000 ethanol and a V2000 methylamine.
Every atom has unspecified/unmarked valence. Noncontiguous V3000 IDs 10/40/90
and 7/15 are source identities, distinct from fresh mapper and preview IDs.
Ethanol's O bond is vertical; methylamine's C-to-N bond points up and left in
display coordinates. These directions must not be rotated or mirrored to make
label placement easier.

The source follows the published [BIOVIA Chemistry2026 CTfile Formats](https://www.3ds.com/assets/invest/2026-01/biovia_chemistry_2026_ctfile_formats.pdf),
printed 9–13,64–70,97–99. It is 740 bytes, SHA-256
`e5cbdc14329c59bad55148c5de37058de90f651af10966c30cf8b89b3b5efd39`.
Literal record ranges are 0..410 and 410..740; contained MOL ranges are 0..371
and 410..713. LF bytes remain exact through the local `.gitattributes` rule.

Unchanged external `mol_oracle.py` reads each actual raw MOL and reports
CCO/C2H6O/H[3,2,1] and CN/CH5N/H[3,2]. The separate named-only full-supplier
check has two records and preserves these ordinary NOTE fields. That check
does not replace raw-span tests or certify duplicate/DT-only field fidelity.
