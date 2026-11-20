# Creating a caption on a CDX chemical drawing

Reviewed 2026-11-20. The Page object can own both fragments and standalone
text. A node-owned label stays in its fragment when a chemist adds a note to
the page. Append the new Text immediately before the owning Page's closing
record. Keep the header, earlier properties, nested objects and terminators
byte-for-byte. [Page](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Page.htm),
[binary object structure and IDs](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/IntroCDX.htm).

Objects with omitted Z have effective depth zero. The supported chemical
drawings have that depth for their chemical ink. Give the first direct caption
depth one. If direct captions already exist at distinct positive depths, give
the new caption one more than their maximum. A genuinely empty page keeps
its established first depth zero. The field is signed 16-bit. Exhaustion
refuses the edit before history changes. [ZOrder](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/ZOrder.htm),
[mixed drawing order](cdx-mixed-caption.md).

Keep the existing creation policy for a fresh raw ID above every object in
the source, a stable application caption ID across Undo, the existing font
table, and an explicit left baseline in document points. The entire resulting
mixed page must re-import as the expected model, including every atom, bond,
label, old caption and paper dimension. A subsequent run edit must resolve
the new page-owned caption after saving and reopening. [Caption encoding and identity](cdx-caption-creation.md),
[existing mixed-page run edits](cdx-mixed-caption-edits.md).

With Text selected, a painted direct caption opens its run editor. Blank
paper opens a new caption draft and occupied chemical targets retain their
pointer behavior. The explicit coordinate command uses the established paper
and full-render checks. This is Bondline's existing interaction policy.
[Text tool](native-caption-creation.md).
