# Owned Node child Text data

2026-11-18, basea546e46. Continue the full M0–M5 goal through the next required M1
model/format boundary. This increment owns supported authored child Text and loss guards;
it does not guess label geometry/chemistry or complete the later renderer/native editor.

## Evidence and authority

The primary [Node](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Node.htm) and
[Text](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Text.htm) specifications distinguish
contained Text from redundant numeric chemical fields. Preserve both independently;
a present child label, including empty text, must not become derived Element/H ink.
Existing [placement/default questions](../clean-room/open-questions.md#chemical-label-defaults-and-placement-2026-11-18)
remain. Its `p` is owned raw point data, not an asserted caption baseline or Node centre.
[Justification](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Justification.htm)
and [LabelJustification](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/LabelJustification.htm)
have the same named values; [LabelAlignment](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/LabelAlignment.htm)
has a distinct enum/meaning. Retain these independently with exact absence. No DTD or
last-used style-sheet default is injected; explicit Style fields alone supply owned runs.

## Locked model interface

Add checked `AtomLabel`, `TextJustification` and `LabelAlignment` to the pure model.
TextJustification has Right/Left/Center/Full/Above/Below/Auto/Best. LabelAlignment has
Auto/Left/Center/Right/Above/Below/Best. Both serialize named snake_case strings, not
conflated integer codes. A node label has no invented application TextId: it is owned
by its stable AtomId, separate from Page caption identities and source XML object IDs.

`AtomLabel::new(position: Option<Point2>, justification: Option<TextJustification>,
label_justification: Option<TextJustification>, label_alignment: Option<LabelAlignment>,
runs: Vec<TextRun>) -> Result<Self,TextError>` retains these five fields. Getters have the
same field names; `runs()` returns a slice. `with_run_text(usize,String)` and
`with_position(Option<Point2>)` return checked copies; neither infers chemical values.
Use the existing TextRun/TextStyle values and their XML character/font/size/face/RGB
validation. Require1..4096 runs and at most1MiB total UTF-8 text. Closed map-only JSON uses
these exact field names; option fields serialize null when absent and may be omitted on
input. Unknown keys/invalid values/tuples refuse. Do not reinterpret present empty strings.

Atom adds `pub authored_label: Option<Arc<AtomLabel>>`, defaultNone and omitted from old
JSON. Old no-label data/serialized bytes remain exact. `Document::has_authored_atom_labels`
checks presence only, not visual/chemical agreement. Fragment and Document bounds prevent
a per-atom run/text cross-product. Across all owned labels plus Page captions, count at
most100000 objects,100000 runs and64MiB text. Appends/replacements/JSON decoding validate
atomically before mutation. Retained history counts unique label Arc payloads, reserved
run Vec/String/font capacity exactly once across shared fragments/snapshots. All legacy
ID/missing-page/fragment and existing caption budget behaviors remain.

## IO, loss and render boundary

Project at most one ordinary direct t per elemental n through the standalone styled-text
view. All run font/size/face/color fields must be explicit and resources resolvable;
unsupported/missing styles, unknown enums, malformed points or extra child t stay opaque
without a new hard file-open failure. Keep original raw source authoritative. Native
encoding enumerates both caption and Node resources, emits optional p/Justification/
LabelJustification/LabelAlignment exactly when present and all ordered decoded runs.
No-label output stays byte-exact. Source object IDs remain source IDs; native generated
IDs are per snapshot as before. Unknown source properties are never silently regenerated.

A model with any authored Node label does NOT gain general authoring/display capability.
`StoredDocument::is_fully_mapped` must stay false for those models, including source-free/
canonical data, until layout/chemical agreement and dependent-edit policies are verified.
No new source profile or command/schema/tool binding is added. All current H/Element/
position edits, style/page operations and caption text.set_run must refuse through the
existing complete-source guards rather than leave stale authored labels. Normal file
Open/unchanged Save remains available. Raw native model encoding is a data codec, not
chemical editing authority. Preserve old legacy data-only setter behavior when labelNone.

Direct CPU/GPU scene producers also refuse any authored Node label with the existing
UnsupportedChemicalContent, before painting derived labels. Cancellation/dimension/error
ordering for old data stays. Whole MOL V2000/V3000, including low-level fragment writers,
refuses any present label before data loss. Retained SMILES H/order replacement includes
this field in its strict metadata comparison; CDX stays unchanged-only. Explicit graph-only
SMILES export keeps its existing declared context/geometry loss receipt and no Save effect.

## Verification and next workflow

Initial independent model/IO/render declarations precede producer changes. Use literal
JSON, raw XML/Expat/Decimal and mixed caption/Node resources, every alignment and missing
position/field, unknown/multiple/unresolved retention, full-bound budget/accounting and
source/history/no-op/failed-edit tests. Actual CLI/initialized stdio/authenticated socket
Open/Unicode Save/reopen verify data without claiming faithful display or chemistry.
Extend bounded parser/profile fuzzing with separate original source/corpus/manifest.
Run fmt, warnings-denied clippy, workspace and xtask gates; keep unrelated MCP failures
and test-setup qualifications explicit. No hosted Actions, push or publication.

Next prove supported Node-child layout/semantic agreement, source-preserving synchronized
chemical edits, every H/position mutator, font/default/face resolution, masks/hits/DPI and
actual native rendering/editing. The full Text tool stays grey and the entire M0–M5 scope
remains active; this necessary data boundary is not the requested final application.
