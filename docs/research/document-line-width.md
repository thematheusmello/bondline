# Explicit document line width

2026-11-17; base `ab912a2`. This increment continues M1/M2; it does not complete either.

## Public basis and independent policy

The published [LineWidth property](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/LineWidth.htm)
identifies document and object ownership and inheritance. The published
[coordinate datatype](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/DataType/CDXCoordinates.htm)
defines decimal CDXML values in physical points (72 per inch); CDX uses signed fixed16.16.
Those archived specifications do not establish a universal missing width: unresolved
values depend on a style sheet. They are format evidence, not observed 26.1 execution.
GUIDE25 printed 40–43 separately documents document-wide Drawing settings and Line Width.
No proprietary program, icon, template, font or sample is used.

Bondline's supported data domain is finite `0 < line_width_pt <= 100000`, an original
bound shared with authored coordinates. `DrawingStyle` stores optional root width,
separately from creation-only `DrawingSettings`. Empty JSON and root attributes remain
omitted. None preserves the original Bondline 1pt appearance; it is not a foreign default.
Root-only support does not flatten object overrides or resolve unknown styles/fonts.

## Complete data and command path

Preserve all old native bytes and public pair-command records. Native CDXML emits a
present `LineWidth` after existing creation keys. The closed source-preserving profile
accepts that one additional root key only after full lexical, key, ID and typed proof.
Unsupported/invalid root values, nested styles, other style keys and foreign semantics
remain unchanged-only. Typed comparisons keep unchanged numeric lexemes exact. Changed
values, insertion and removal use the existing batch writer and semantic certification.

Append `document.set_document_settings` to the registry with four required closed keys:
`expected_revision`, `bond_length_pt`, `chain_angle_degrees`, `line_width_pt`. The three
values are nullable; explicit null clears them. Apply source/revision/close checks before
domain/no-op handling. One ordinary history commit replaces the full triple. A width
change first runs the existing bounded whole-page authoring/ink certificate with actual
style; unrenderable ink refuses atomically. Unchanged width retains the pair's old behavior.
No-ops preserve sharing, redo, selection, leases and weak contexts. Errors retain their
existing typed scale/content/resource meanings. No Save/close acknowledgement is added.

The old `document.set_drawing_settings` pair and `apply_ui_drawing_settings` remain
unchanged and preserve style. The existing source-qualified Document Settings request
gains width; `apply_ui_document_settings` invokes the combined registry command once.
The actual form has three 384-byte fields and shares the ordered input router with the
unchanged two-field Atom form. Prefix, stale-draft, explicit Cancel and modal fences stay.
No planned toolbar tool is enabled by this settings work.

## Ink, cameras and preservation

Separate authored ink width from camera scale, original label em (12pt), atom positions
and original multiple-line separation (3pt). Keep centered offsets and sided endpoint
trims independent of width. The legacy public helpers retain their old results. Actual
new ink drives stroke corners, hits, paper bounds, congestion and visible-pixel checks.
For separation s and width w, centered Double/Triple reserve envelopes s+w/2s+w; the
white span is s-w. Closed gaps fail `BondScale`; do not thin ink or move geometry to fit.
Preserve legacy arithmetic when resolved width equals the old geometric unit.

Carry style through Fit/Zoom and physical/fractional cameras. Complete-owner IO display
overrides a caller's stale style, and Engine page views supply current style to selection,
ring, chain and plain-bond previews. Whole-page paste validation uses candidate style.
The existing CPU/GPU stroke primitives require no shader, font, dependency or cache change.

Whole-document MOL refuses explicit style before raw reuse or regeneration, with the
old forbidden-version precedence retained. CDX and synchronized SMILES model-drift
guards remain. Explicit fragment-only exports keep their declared context loss. CDX
style projection/writing, other properties, journal presets and foreign inheritance
remain required future work.

See [receiving evidence](../verification-document-line-width.md) for actual execution,
qualifications, native/platform limits and continuation; source-only proposals are not passes.
