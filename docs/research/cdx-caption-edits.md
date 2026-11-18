# Editing existing styled CDX captions

Reviewed 2026-11-18 after the [explicit run profile](cdx-caption-runs.md).
Reuse the existing full-source caption proof and the `text.set_run`
command. Address one existing Text object using its established model ID.
Source identity uses the stable source-ID sort, with encounter order
for anonymous IDs, followed by a separate paint-depth sort. An edit
keeps that identity and every original depth and position. [ZOrder](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/ZOrder.htm).

The Text property holds a CDXString. Keep its UINT16 run count and
each run's eight style bytes. Changing the text changes the cumulative
start counts of later runs. Require ASCII in all nonfinal runs so the
new offsets agree under character, UTF-16 and byte counting. They
must fit UINT16. The final run may contain supported UTF-8. Preserve
the existing nonempty-run contract, including allowed runs containing
only spaces. [Text](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Text.htm),
[CDXString](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/DataType/CDXString.htm).

Splice the one property's new payload into the original bytes. Preserve
an exact textual no-op and its original length form. For a real change,
the standalone batch uses the published short or extended length.
Require a complete re-import with the exact caption profile and expected
model. The same check proves resources, per-run layout, cross-run glyph
joins and the entire paper. Commit the bytes and model as one owned
history entry. Undo and Redo restore those snapshots exactly. [Binary framing](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/IntroCDX.htm),
[Caption rendering contract](cdx-caption-runs.md).

Use independent before/after records and complete models for single and
multiple runs, Unicode tail edits, later start shifts and captions whose
source order differs from ID and paint order. Verify reopened renders,
malformed and ambiguous edits and loss refusals. The user-facing native
caption editor and new Text creation remain separate workflows.
