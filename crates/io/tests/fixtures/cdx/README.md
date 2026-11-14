# Original CDX metadata vectors

These independently authored Bondline test documents are MIT OR Apache-2.0. The
generator is `../../cdx_metadata_oracle.py`; no native document, vendor metadata,
template, font, warning text or asset was copied. Public format definitions and
qualified public SDK implementation evidence are recorded under CDX-METADATA in
`docs/clean-room/sources.md`.

| File | Bytes | SHA-256 |
|---|---:|---|
| metadata-explicit.cdx | 323 | 148133b6c2bf5a4bde92c9a8adff636e3f2ef9bf19238b9a1f99003e9e5d7c05 |
| metadata-header.cdx | 319 | 9af210eb978d085b143fad97074d55aa6512c44e02370d4a41e0376d06176a39 |

Both encode the same original analytic scene. Source objects are fragment100,
nodes101/102 and bond103. The two positions are exactly (10.25,-3.5) and
(26.25,8.5) points. AtomID metadata70001/90005 deliberately differs from those
outer source identities and fresh application IDs1/2. Missing element/order use
the documented carbon/single defaults; missing explicit H remains absent in
Bondline. External RDKit perceives CC/C2H6; Bondline does not calculate it.

The fragment bounds are top=-8,left=4,bottom=12,right=32 points; fragment/node/
bond Z values are12/-2,5/7. These source-only graphical fields do not reorder the
model or infer paper size. Both atom and bond CIP bytes are1 (symmetric/N); the
negative matrix separately distinguishes absent/U0/N1 and refuses stereo states.
The zero-style warning has newly authored text. Original opaque custom properties
and an original custom object test complete source retention.

The 323-byte vector uses a28-byte header followed by an explicit root; the
319-byte vector embeds that root after a22-byte prefix and has a separate file
end. Independent Python `struct` decoders check framing, all metadata widths,
primitive values and exact generator bytes. Mandatory external RDKit2026.09.2
consumes both unchanged fixtures. These tests do not establish general CDX
compatibility, editable metadata, stereo perception or fidelity of a regenerated
file. Imported CDX is always incompletely mapped; unchanged export retains every
byte and authoring/cross-format export remains refused.

`.gitattributes` disables line-ending conversion for these exact files. Copy them
into ignored fuzz storage before mutation; never use this directory as a writable
fuzz corpus. No existing fixture, pin or approved render golden was changed.
