# Source-backed chemical label defaults

2026-11-18, base7747111. This is the next M1 format prerequisite, not general foreign
label display/edit authority or completion of the full M0–M5 goal.

## Evidence and unresolved semantics

Primary SDK [LabelFont](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/LabelStyleFont.htm),
[LabelSize](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/LabelStyleSize.htm) and
[LabelFace](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/LabelStyleFace.htm)
define per-property containing-object fallback, then an external last-used style sheet.
The file alone therefore supplies no universal family, size or face. Font/face scalar
metadata is signed INT16, distinct from UINT16 run/resource IDs. The
[complex comparison](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/IntroExampleComplex.htm)
corroborates XML point sizes; it does not resolve every fractional/domain convention.
Positive finite XML sizes are an original checked-reader policy, not a claim about
binary units or all reference versions. Unsupported values remain in unchanged source.

The [Node page](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Node.htm) gives present child
Text priority over redundant numeric chemistry when reading a label. No imported label
may be replaced by derived Element/H ink merely because those fields exist. The archived
[Justification](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Justification.htm)
and LabelJustification fallback differs from the current official DTD's Left default.
Omitted Style-run font/size/face fallback is not established by those property pages.
Label defaults alone must never fill missing `s` fields or grant rendering permission.

Primary SDK geometry does not fully define a general Node-child Text baseline transform.
Public Apache Octet-Draw stores a left-label baseline, while the independent MIT26.1
measurement report distinguishes some stored positions from drawn baselines; Indigo's
writer is not a matching confirming oracle. These sources are qualified in the source
register/open questions. No font-, face- or version-independent offset is adopted.
The child-Text model/render/edit workflow stays required after the supported semantics
are established; this prerequisite must not activate a grey tool or widen IO authority.

## Locked standalone API

Add `Document::label_defaults(element) -> Result<LabelDefaults, LabelDefaultsError>`.
The target must be an ordinary owned element with one of the publicly identified default
roles: the actual CDXML root, n, b, t, table, spectrum, constraint or tlcplate. This is a
property lookup, not a complete object/containment certificate. Geometry remains outside
this first target/owner set: the linked SDK Geometry page incorrectly serves ObjectTag
content, and the available DTD supplies no Geometry declaration.

`LabelDefaults` getters return `font() -> Option<LabelDefault<i16>>`,
`size_pt() -> Option<LabelDefault<f64>>`, `face() -> Option<LabelDefault<i16>>`.
Each copyable `LabelDefault<T>` exposes `value()`, `owner()` and `attribute_value()`.
The latter is the XML-decoded attribute value; lexical quotes/whitespace/entities remain
available through its owned raw node and the original document bytes. It is not a
claim that decoded text equals lexical source. None means no value in this containment
chain, not zero, a bundled-font default, a DTD-injected value or an available style sheet.

Walk only this element and its ancestors, nearest first, independently for each property.
Exact unqualified attribute meaning is required. Qualified lookalikes remain raw data
and do not supply a default. Validate all encountered ordinary Label* values on legal
owners, even shadowed ones, before returning the nearest values. Existing parse_integer
plus checked i16 conversion owns font/face syntax; negative values remain signed data,
not interpreted font IDs/style bits. XML size accepts a positive finite f64 after XML
space trimming, without an invented binary scaling or extra maximum. Invalid ordinary
values return `InvalidProperty("LabelFont"|"LabelSize"|"LabelFace")`.

An ordinary Label* on a non-admitted containing owner returns
`UnsupportedOwner("LabelFont"|"LabelSize"|"LabelFace")`; never skip it and silently trust
a farther value. This includes page/group/fragment and foreign-namespace owner lookalikes.
For deterministic diagnostics, inspect nearest ancestors first and font/size/face in that
order on each ancestor. Foreign-document, non-element, unsupported-target and qualified
role lookalikes return InvalidElement. Cross-document identity is pointer ownership,
even for byte-identical documents. Namespace reset to empty restores ordinary meaning.

The view borrows nodes/attribute data, uses bounded existing XML ancestry, does not mutate,
allocate font assets, resolve missing resources, rewrite source, modify syntax admission,
model/history/registry, or change old Style-run absence. Callers must separately prove
resource availability, complete source coverage and layout/chemical meaning.

## Verification

Declare literal/provenance tests before production, observe missing-API RED, then add
independent Expat/Decimal owner/value expectations and wrong-owner/override/default/
namespace/domain controls. Cover every admitted role, partial defaults and unknown data,
signed INT16 limits vs UINT16 resources, fractional positive XML reader policy, invalid
shadowed defaults, error ordering, owned SetAttribute composition and exact source/no-op.
Extend the existing bounded standalone parser fuzz target with lookup/owner/source-purity
invariants and original separate source/corpus/manifest seeds. Run standalone and full
local gates; preserve any old MCP failure without retries/policy changes or causal claims.
No native result follows from this format-only increment. Actions remains OFF.
