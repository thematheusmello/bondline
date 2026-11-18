# CDX literal atom metadata: next M1 contract

Locked2026-11-18 after verified metadata code `c965607`/full98153. This work is **planned**:
no new CDX decoder, fixture, consumer agreement or native acceptance is claimed here.
Implement literal data in the existing read-only projection, retaining every source
byte. No new model type, command, display/edit authority or chemical inference is needed.

## Public evidence and bounded policy

Fresh archived public SDK sources:
[Isotope](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Atom_Isotope.htm),
[Charge](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Atom_Charge.htm),
[Radical](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Atom_Radical.htm),
[NumHydrogens](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Atom_NumHydrogens.htm),
[IntroCDX](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/IntroCDX.htm),
[numeric types](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/DataType/CDXNumeric.htm),
[NodeType](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Node_Type.htm) and
[Element](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Node_Element.htm).
Eight fresh200 receipts are ignored research, manifest4828B/SHA256
`1ebc79d957c8edcd0624e0582fd1f735c9d4dc61df0668e4fc7c85bb642a2764`.
The bounded preparation report11091B/SHA256
`2288874e15c088af99bbab3e3b8577d21b29f651af2a56ccd2f83cc5b0a8497e`
records current code and wider-width questions. Documentation is not bundled/relicensed
as an asset. No reference executable/export/font/sample is inspected or copied.

All four properties belong to Node. Preserve existing Element-node restrictions and
old absent NodeType/Element behavior. Binary payloads are little-endian; framing length
encoding and numeric payload width are separate. The original bounded decoder policy is:

| Tag | Accepted payload | Model value / absence |
|---|---|---|
|0420 Isotope|signed16,2 bytes|0/absent→None;1..32767→NonZeroU16; negative refuses. No nuclide-existence claim.|
|0421 Charge|signed8,1 byte; documented4-byte recovery below|−128..127; absent0.|
|0422 Radical|unsigned8,1 byte|0/absent→None;1→Singlet;2→Doublet;3→Triplet; other values refuse. Singlet/Triplet retain distinct diradical states.|
|042B NumHydrogens|unsigned16,2 bytes|present0..65535→Some(value); absent→None. Inline H is separate from drawn H nodes.|

SDK omission requests H inference; Bondline has not implemented that inference and must
retain None. Domain extrema are literal-data tests, not chemical-validity statements.
Explicit isotope0/charge0/radical0 may share modeled values with absence, while original
source presence/bytes/identity remain distinct. Existing Atom fields suffice.

### Four-byte Charge policy

The property-specific SDK page explicitly prescribes signed savedINT32 arithmetic
shift24 when this field has four bytes. It documents the8.0 writer mistake and return to
INT8 at8.0.6. Apply that property-specific instruction by payload width, without a
producer/version-string waiver. It takes precedence here over generic numeric-width
guidance. Do not reinterpret the payload as an ordinary full-value INT32.

Freeze collisions before production: little-endian `01 00 00 00`→0,
`00 00 00 01`→1, `00 00 00 FF`→−1, `AA BB CC 7F`→127 and `00 00 00 80`→−128.
Retain all low bytes exactly on unchanged Save; no CDX encoding/normalization is added.
Independent struct evidence must assert this algorithm. Qualify the external consumer's
actual legacy behavior separately; never change expected values to match that consumer.
General wider integer encodings remain unsupported by this bounded projection; the SDK
permits more than the canonical widths, so do not call every rejected width malformed.

## Ownership, errors and retained authority

Only `crates/io/src/cdx.rs` needs production changes: constants, read_atom's admitted
properties/projection, and Layout::read's GLOBAL Node-owner list. Both lists must change
so the same known fields on root/Document/Page/Group/Fragment/Bond/custom owners refuse.
Do not weaken existing unknown-user-tag, identity, parent, Text/query/stereo/bond/style
guards. Reuse checked payload/record access; no unchecked indexing, allocation or panic.

Keep DuplicateProperty on the second known field, including equal zeroes. Missing
optional fields never become MissingProperty. Payload widths outside the table return
UnsupportedProperty; accepted-width invalid isotope/radical values return InvalidProperty.
Truncated tags/short or extended lengths/payloads remain standalone syntax errors.
These are explicit original adapter policies, not universal reference error semantics.

StoredDocument owns complete CDX source and its baseline. Full/display/edit stay false;
unchanged/no-op snapshots borrow source, any model drift refuses to_cdx and restoration
recovers original bytes. Cross-format/CDXML/MOL and render/authoring refusals stay exact.
Preserve source generation/identity even when default-valued files have equal models.
Registry49 actual bytes/schema/order and catalogue16/124 stay unchanged. Actions OFF.

## TDD and integration order

1. Freeze original independent Python-struct sources/facts/SHA before the producer. Use
   existing accepted explicit/header-implied envelopes, short/extended property lengths,
   source IDs, opaque bytes, absent/defaults/extrema and the declared Charge collisions.
2. Run existing public IO APIs RED separately from any missing-new-API compiler probe.
   Independently decode actual source/saved bytes, owners/tags/widths/signed values and
   presence. Qualify pinned external RDKit CDX isotope/charge/H/doublet observations on
   chemically valid original cases. Electron count alone cannot prove singlet/triplet.
3. Cover every wrong parent, duplicate, truncation/domain/width and unchanged raw/no-op/
   drift/restore/cancellation/atomic-save/cross-format/display refusal. Six old literal
   valid-field witnesses in cdx_graph/cdx_document/cdx_metadata may become stronger
   positive data assertions only after reviewing each exact case. Other negatives stay.
4. Add the minimal decoder. Verify actual CLI/initialized MCP/authenticated socket CDX
   Open→inspect→Unicode unchanged Save→reopen, exact metadata and independent files.
   New native acceptance must use a different own app and protect all user reviews.
5. Run focused/full IO, bounded fuzzing and complete local gates. Record real consumer/
   platform limits, update source/parity/status records and push each atomic commit.

This does not finish CDX Text/resources/styles/multiple bonds/query/stereo/editing/render,
general CDXML/interchange or any remaining M0–M5 requirement. Missing-run typography
inheritance remains open; do not return to it without stronger source evidence.
