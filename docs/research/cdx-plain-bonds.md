# Plain CDX bond data: M1 contract

Locked2026-11-18 onbaa3d90, implemented in37d094d. The requirements below retain their
preimplementation contract scope; [receiving](../verification-cdx-plain-bonds.md) records
independent fixtures, runtime RED/GREEN, actual files/native scope and local gate results.
This is a read-only projection prerequisite, not faithful CDX depiction or authoring.
Existing Atom/Bond/DoublePosition fields and the unchanged source snapshot suffice;
no new command, model type or dependency is needed.

## Primary evidence

Fresh public SDK [Bond](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Bond.htm),
[Order](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Bond_Order.htm),
[Display](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Bond_Display.htm),
[Display2](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Bond_Display2.htm),
[DoublePosition](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Bond_DoublePosition.htm),
[Begin](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Bond_Begin.htm),
[End](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Bond_End.htm) and
[numeric types](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/DataType/CDXNumeric.htm)
are reviewed. Eight fresh200 receipts have2957B manifest SHA256
`3e0665ad7298e2b15300f01926a1d9505af8733b7558a3d9df3bdbba7f9870a3`.
Ignored research is not a bundled asset. No reference binary/sample/artwork is inspected.

Order0600 is a two-byte bitmask:1 Single,2 Double,4 Triple;3 is single-or-double query,
not Triple. Absence means Single. Wider/higher/query/unspecified kinds remain outside
this plain subset even where the current model has another BondOrder variant.

Display0601 and Display2 0602 are signed16 enumerations. Missing Display means Solid;
missing Display2 inherits the first line's Display. The model has no non-Solid display
field. Only absence/explicit0 Solid can be admitted, with both checks required; do not
fall back to Solid for an unsupported encoded style or silently discard it.

DoublePosition0603 has six explicit values:0/1/2 automatic Center/Right/Left and256/257/
258 manual Center/Right/Left. Side is relative to the authored Begin→End direction.
Absence stays Unspecified; no layout algorithm or implicit Center is introduced. The
model stores side/presence only. Automatic/manual and exact scalar bytes remain distinct
in immutable source identity; no source-regenerating writer or broader loss permission
may follow from model equality.

## Bounded admission and errors

All four fields belong only to Bond. Add0601/0602/0603 to BOTH the global owner classifier
and the local Bond admitted list, retaining strict duplicate checking even for equal0.
Wrong root/Document/Page/Group/Fragment/Node/custom owners refuse UnsupportedProperty.
Existing endpoint direction, same-fragment resolution, identity/geometry, source-only
metadata and every unknown/query/stereo/object restriction remain unchanged.

Only a Double may carry explicit Display2 or DoublePosition, including zero-valued
fields. Reject either on Single/Triple as UnsupportedProperty. This is an original
bounded support rule for otherwise ambiguous/dormant data, not a universal malformed-file
claim. Explicit Display0 is allowed on all three plain orders. Unknown/non-Solid display
and unrecognized position values return UnsupportedProperty, without pretending to
support the SDK's fallback behavior. New fields accept exactly2 payload bytes; other
widths return UnsupportedProperty, since generic SDK numeric guidance allows more.
Order retains its existing wrong-width InvalidProperty and unsupported-value
UnsupportedBondOrder behavior. Preserve syntax/truncation/resource error precedence.

Map a Double's explicit six position values to the existing side enum. A Single/Triple
always has Unspecified here. Keep the absence/explicit-Center and equal-model automatic/
manual source pairs separate. Global full/display/chemical-authoring/cross-format
permissions remain false. Unchanged Save preserves every raw byte, field order/width/
presence/framing/envelope/source ID and opaque payload. Model drift refuses CDX output;
Undo restores source. The legacy detached-page/history operation stays available.
All49 actual command records/schema/order and catalogue16/124 stay exact; Actions OFF.

## Independent tests before production

1. Freeze original Python-struct sources/facts/SHA: absent versus explicit Single/Solid,
   Double/Triple, all six DoublePosition values plus absence, Begin/End reversal, equal-
   model auto/manual pairs, property reorder, short/extended lengths and existing three
   envelopes. Include literal H2/H1 and missing-H cases without inference. A strict
   independent record reader must account for all bytes/owners/field presence and build
   the complete expected A4 document without Bondline code.
2. Qualify actual pinned RDKit on original valid ethene/ethyne, explicit direction and
   supported envelopes. Order/adjacency agreement does not prove side, auto/manual state,
   rendering or general stereo; independently decoded source facts prove those fields.
3. Observe runtime RED using existing public APIs, then add the minimal IO implementation.
   Cover every unsupported order bit/combo, non-Solid display, new field width, duplicate,
   wrong owner, non-Double position/Display2, truncation and source identity. Only the old
   order2 witness in cdx_graph may become a stronger positive, after observing its failure;
   order0/3/FFFF and all other old guards stay exact.
4. Exercise unchanged/no-op/model-drift/restore/cancellation/atomic Save/cross-format and
   display/chemical-authoring refusals through IO, command/App and actual CLI/initialized
   MCP/authenticated socket. Check actual Unicode bytes with the independent reader.
   Use a different own app for bounded native Open/Save/reopen; protect all user reviews.
5. Extend the existing parser fuzzer with raw-source order/display/position invariants;
   freeze separate seed sources and measure actual mutation sizes. Run local gates, record
   any unchanged listener failure without retries/waivers, and push each atomic commit.

Broader bond kinds, wedge/hash/dash/stereo, complete CDX typography/colours/styles,
faithful preview, source-preserving chemical edits and full M0–M5 remain required.
