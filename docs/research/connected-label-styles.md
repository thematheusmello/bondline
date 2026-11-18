# Connected explicit-label styles

Contract 2026-11-18, base61be8e6. Continue full M1 and M0–M5; this is not general foreign
style or reference-pixel parity. Preserve the earlier boxed-label and unknown-source
contracts except for the independently qualified connected branch described here.

## Sources and distinction from native defaults

The public SDK [MarginWidth](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/MarginWidth.htm)
defines coordinate-valued whitespace around labels, with containing-owner then external
style-sheet fallback. [LineWidth](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/LineWidth.htm),
[BondSpacing](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/BondSpacing.htm) and
[BondLength](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/BondLength.htm)
likewise provide no universal missing foreign value. GUIDE25 printed42 describes erasure
of attached bonds around labels, separately from their authored positions. XML units are
points; binary coordinates are signed fixed16.16. Root-only support avoids the archived
SDK/DTD disagreement about Node MarginWidth and root BondSpacingAbs ownership.

[ForegroundColor](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/ForegroundColor.htm)
defaults to resource3, not universally black. Missing Display is Solid; missing Display2
agrees with Solid here. Missing DoublePosition is automatic/unspecified, not Center.
Every admitted foreign atom still owns the existing explicit no-p/Left/Left/Roboto face0
literal Element/H/ink-box certificate. Unlabelled foreign font defaults remain unresolved.

Public original Octet and ReShiki sources support outline-aware shortening and common
multiple-bond cut planes. The separately pinned MIT author's26.1 Helvetica measurement
report contradicts universal AABB trimming, but its fitted band/margin constants and
formula-face conventions do not prove Roboto behavior. No fitted constant, font, exported
fixture, proprietary asset or upstream implementation is copied. Source body/license
hashes and the exact qualifications are retained under ignored target/connected-labels.
Record actual implementation and verification separately in the receiving document.

## Owned data and loss boundaries

Add independent optional `DrawingStyle.margin_width_pt`, checked getter and preserving
`with_margin_width(Option<f64>)`. Keep `new(width)`, old builders and absent JSON/native
bytes exact. Finite0..100000pt is an original bounded nonnegative interpretation policy;
Some0 differs from None. Map-only deserialization remains closed and checked. Root CDXML
projection/encoding and per-property Preserved splices retain the value and exact unused
lexemes. Unsupported/nested/qualified/invalid fields stay opaque. Reject negative nonzero
underflow rather than treating it as explicit zero; true signed-zero lexemes stay zero.
Whole-MOL and retained-source/CDX/SMILES loss guards must include the new style member.
CDX style projection and local/inherited styles remain separate work. No new command,
menu, settings field or catalogue binding is required for this imported-file workflow;
all48 registry records and existing form behavior stay exact.

Until the outline renderer is integrated, explicit margin on a bond-bearing page must
refuse rendering rather than silently ignore newly owned data. This is a temporary
guard in the data checkpoint, removed only with its actual runtime RED→GREEN evidence.

## Original physical clearance policy

Use the actual glyph placements used by rasterization, including native derived label
layouts. Preserve every existing glyph pixel, source box, atom point and AtomHit mask.
Do not enlarge click ownership or use a whole label AABB as final geometry. In a bond
frame let a be along-axis distance, c normal distance, o a component offset, w its butt
halfwidth and M the explicit margin scaled to canvas units. For an outline point q:

```
d = max(0, abs(c - o) - w)
cut(q) = a + sqrt(M*M - d*d)   when d <= M
```

Take the maximum over actual quadratic/linear outline boundaries and all components at
each labelled endpoint, then one shared axial cut for every rail. The source axis stays
unchanged. Existing finite BUTT caps get no invented axial round-cap radius. M0 uses the
closed d0 strip case. This is an original Euclidean ink-clearance rule, not a claim that
the reference uses it for all fonts, open glyphs, angles, counts or chemical text. Keep
that reference-parity question open; never substitute AABB or Helvetica fit constants.

Use bounded conservative curve subdivision with cancellation. Target upper/lower cut
interval width1e-4 canvas units; account separately for float32 coordinate rounding with
8*f32::EPSILON*max(1,absolute canvas coordinate). At most131072 curve/work nodes per page
and depth64; exhaustion refuses SceneTooComplex, never publishes an uncertified cut.
Nonfinite geometry, consumed bonds or failed real-ink/paper/gap checks refuse atomically;
never shrink requested margins, hide an arm or change H to make a scene fit.

For explicit margin, shorten the actual rendered strokes, rebuild their hit polygons and
use those same components in paper/gap/ink certificates. Preserve raw axis metadata only
where it describes authoring/junctions. Keep old1px masks for AtomHit and existing label/
nonincident collision ownership, but do not paint them as white erasers: that would impose
an undocumented pixel floor on small/zero margins. Real-ink occlusion then uses actual
painted glyph spans. None keeps its entire old masking/primitive/guard path exact.

## Complete foreign source grant

Preserve the existing foreign Boxed zero-bond capability literally. Add a distinct
Connected source profile and `has_connected_atom_labels` capability, sharing the complete
lexical/node/key/scoped-ID/resource/model proof. Every foreign
node must still have its explicit certified label. Bond-bearing scenes need explicit
root LineWidth and MarginWidth; Double/Triple additionally need explicit BondLength and
BondSpacing. Doubles require explicit Center; Single/Triple retain unspecified placement.
Only absent or explicit Solid Display/Display2 and absent or resolved-black bond colour
are supported. A present short palette is not an absent palette. Reject local styles,
Abs, attachments, unknowns, namespaces and crossing/degenerate geometry; do not infer
crossing order/erasure from root margin. Geometry admission must not depend on one fixed
capture's physical scale or move signed source coordinates.

Connected requires at least one bond. Global full authoring stays false; native and
source-free Boxed behavior stays unchanged. Display and the existing command48/native
atom form may consume either precise capability, without changing registry metadata.
Only command48 may synchronize existing Element/H/body/box. Source/current/saved/history
owners, source IDs, styles, captions, geometry, membership, array order and bond directions
stay unchanged. The full final source re-import plus one final page certificate precedes
one ordinary commit. True no-op preserves source bytes/sharing/redo/weak owners. Other
chemical/page/style/caption tools remain refused. Source-free/exact-native conventions
remain richer; producer text alone supplies no authority.

## Independent acceptance

Original FontTools/Expat/Decimal sources cover H–O–H→H–NH–H, O=C=O and H–C≡N. Constructed
and actual direct external RDKit CDXML parsing independently establish H2O/H3N/CO2/CHN,
literal atoms/H and bond orders; no valence inference is added to our editor. Endpoint
oracles must measure actual outlines independently, with declared flattening/error bounds,
and reject AABB countermodels. Include oblique O/H/I/N/F/Cl/Br, native and authored sizes,
flat/stacked labels, multiple common cut planes, widths, zero/fractional margins and DPI.

Prove unchanged None pixels/bytes/metadata, actual CPU/GPU/picking/paper/ink behavior,
source guards/crossings/colour/explicit-owner negatives, property history and captured
Save ownership. Exercise actual CLI/stdio/authenticated socket and Unicode files, real
App input and own-window native edit/history/Save/reopen when available. No new human
golden is approved by generated oracles; Actions stays OFF and protected apps stay untouched.
