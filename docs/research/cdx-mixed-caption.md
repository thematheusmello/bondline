# CDX pages with chemical structures and captions

Reviewed 2026-11-20. A Page can own both Fragment and Text objects. A Node can
also own Text that labels the atom. To recover the whole drawing, resolve each
Text against its enclosing object. Two children of different containers can have
the same raw four-byte ID. A nonzero ID must be unique among the children of
its container, including siblings of different object types. Zero may denote an
unreferenced object. [Page](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Page.htm),
[Fragment](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Fragment.htm),
[Text](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Text.htm),
[binary identity](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/IntroCDX.htm).

The signed 16-bit Z property orders graphical objects from back to front.
An object that omits Z has effective depth zero. The initial mixed profile
admits the existing boxed or connected chemical drawing, whose Nodes, Bonds
and owned labels omit Z. Direct captions have distinct explicit positive
depths. This establishes that all caption ink is in front of the chemical
drawing even where glyphs overlap. Captions retain the existing stable Text
identity and layout rules. Tied, interleaved and unknown depths require
additional evidence. [ZOrder](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/ZOrder.htm),
[chemical preview](cdx-atom-label-preview.md), [caption identity](cdx-caption-creation.md).

Bondline checks both parts against the same parsed document and resources.
Both consume one page run budget. The combined drawing must pass its own
physical-paper and whole-page rendering checks. The current display grant
belongs to that exact source and model. Open and unchanged CDX Save retain
the entire original byte stream. Authoring changes will also need to prove
the preserved identities, depths, and byte ownership of unaffected peers.
[Original models, recorded refusals and independent overlap probes](../../crates/io/tests/fixtures/cdx-mixed-caption/SOURCES.md).
