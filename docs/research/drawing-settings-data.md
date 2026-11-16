# Drawing settings: locked data/persistence prerequisite

2026-11-16, base65af201. Previous goal turn is PROGRESS (literal paths/previews,
1574/0/1 and bounded ASan). Full M0–M5 remains active. This first format/data increment
does not implement a settings command/form, stylesheet resolution or native default
application. Native chain controls stay disabled; native-chain-plan.md steps2–6 remain.

## Primary sources

- [BondLength](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/BondLength.htm):
  tag0x0805, CDXCoordinate, a creation/default length rather than permission to move
  existing coordinates; supported owner list includes Document and some child objects.
- [ChainAngle](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/ChainAngle.htm):
  tag0x0803, document INT32 degrees×65536 in CDX.
- [Coordinates](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/DataType/CDXCoordinates.htm)
  and [numeric types](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/DataType/CDXNumeric.htm):
  signed little-endian coordinate units are1/65536pt in CDX, decimal points in CDXML.
- [Published scalar comparison](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/IntroExampleComplex.htm)
  explicitly pairs raw chain-angle7864320 with XML120 and raw length1966080 with XML30.
  Thus XML angles are unscaled degrees, despite the binary-oriented property wording.
  Only these published encoding facts are used. No linked vendor binary/image/sample
  file is downloaded, copied or committed; tests independently author their own vectors.
- [Document](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Document.htm),
  [CDXML overview](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/IntroCDXML.htm), and
  [official DTD](https://static.chemistry.revvitycloud.com/cdxml/CDXML.dtd): root properties
  are optional, DTD CDATA #IMPLIED; numeric spellings may vary. The property pages give
  containing-object/last-used-style fallback, not one universal numeric default.
- GUIDE25 printed40–42/104–105 (existing pinned public PDF) documents active-document
  drawing settings/style sheets and Chain Angle1..179°. The full native settings/form/
  fixed toggles/modifier/default behavior is separate, unverified work.

Exact reviewed primary HTML and source pins are disposable under
target/drawing-style/source-proposal. BondLength16054B SHA256
5504659e84034e01e3b8a48658da2b2c53fb3c74d68cea988d88d46fc145582c;
ChainAngle16142B SHA25655c3e3eb8ac2d91eaff893fef65121b028ddd60a9f4d6ef955462008f88265ae;
coordinate types16807B SHA2564b0d0ceb94d14c6fc421132408a2371be3f8f88af1a67a8cdb473ab9335e308f.
Historical SDK evidence is not a26.1 runtime/default claim.

## Model contract

Add public model::DrawingSettings, fixed-size Copy/Default/PartialEq data with private
independent Option<f64> bond_length_pt and chain_angle_degrees. Default is both absent.
new(length,angle)->Result validates; by-value getters, is_empty and strict closed-object
serde are public. Missing/null model fields mean no modeled value; do not distinguish
Some(empty) from empty or introduce numeric fallback. Serialize only present properties.
Document owns this value, drawing_settings() returns it, set_drawing_settings(value)
stores the already-validated value. Omit the entire empty container from Document JSON
so every old absent snapshot/native-v0 byte profile remains exact. No L0 schema/IO/GUI
dependency. Clone/equality/history/retained-byte accounting include the fixed-size data.

Supported authored subset: finite bond length >0 through100000pt, and finite angle
1 through179 degrees, including fractional values. The angle range follows the guide;
the length cap is original bounded model policy, not a vendor/file-format maximum.
CDX fixed16.16 has a wider signed mathematical angle domain and a narrower length
representability limit; this increment does not claim a general CDX writer.
Invalid typed/model JSON values fail validation without changing prior data.

This pair is known authored data or known data in a source projection, not effective
inherited preferences. A missing modeled field in an incomplete source projection can
also mean an unsupported/opaque source value. The backing/completeness proof remains
authoritative; never label that omission a resolved default or native authority.

## CDXML and CDX projection/persistence

Read only direct document-root BondLength/ChainAngle, in unscaled points/degrees.
Project each supported explicit scalar independently into the model. Unparseable,
nonfinite or out-of-supported-subset root values stay unprojected in the authoritative
raw source. Preserve the previous successful foreign Open/byte passthrough behavior;
do not turn an unmodeled preference into an invented value or hard mapping failure.
Such raw attributes cannot match the native writer and therefore remain incomplete.
Nested overrides are not flattened; they likewise remain opaque/incomplete.

Native CDXML writer emits CreationProgram then present BondLength then ChainAngle,
using finite canonical decimal scalar spellings. Both absent produces exactly the old
root bytes. Existing full-byte re-encoding equality alone qualifies native editability;
do not weaken it for whitespace, alternative ordering/spellings, unknown properties,
Text/style inheritance or partial geometry. Unchanged source saves exact bytes;
supported native edits regenerate only with the same existing authority proof.

CDX projection also reads the two recognized root tags, for both supported implicit
document properties and a direct explicit document object. Decode only one unambiguous
four-byte signed little-endian payload per property, divided by65536, then use the same
supported model validation. Duplicate, other-width, unsupported-domain and nonroot values
stay opaque/unprojected; preserve existing successful binary Open and exact passthrough.
Do not read past fields, infer width, choose a duplicate or normalize a source. Unknown
properties remain in the standalone parser/backing. CDX stays noneditable/incomplete;
no framing/EOF, regeneration authority or general CDX writer change is included.

## Loss and behavior boundaries

Any nonempty authored settings must refuse whole-document MOL regeneration in
StoredDocument::to_mol_as, for V2000/V3000 and every native/source-backed path, before
fragment-only reuse can omit context. Preserve the existing forbidden-version precedence.
Lower Fragment MOL writers remain chemical-graph codecs with their existing signatures;
do not invent SDF/RXN or MOL fields for preferences. Raw unchanged MOL/SDF/RXN remains.

Whole-model equality must invalidate exact-source CDXML/CDX/MOL export and synchronized
SMILES projection/save after a style-only drift. Keep these existing guards and source
classes. Explicit SMILES fragment export already declares document_context/geometry
loss; that distinct semantic operation remains available only under its old authority.
Do not silently turn it into whole-document persistence or broaden opaque sources.

Settings changes never move atoms, recalculate H, resize pages or rescale existing
strokes/labels/captures. Present values are stored but not yet applied by native creation
tools in this data-only increment; their original pose policies remain unverified for
styled documents. Native defaults/settings UI and complete styles are separate delivery
gates, not functionality established by new serialized fields.

## Independent acceptance and next work

First existing-API runtime test loads an independently authored root-style CDXML and
expects its serialized model to contain the two literal values; baseline ignores them
and must fail before producer. Separate new-type compile qualification from that RED.
Then cover model presence/range/strict JSON/old-default bytes, original XML+Decimal
properties, scalar raw-CDX/stdlib checks, partial/invalid/nested/opaque preservation,
exact native profile classification and actual current/baseline/save separation.
Exercise both whole-MOL versions, styled source-backed drift (including fragment-only
reuse), retained SMILES drift and unchanged explicit semantic export policy. Old graph/
RDKit consumers ignore root settings and cannot prove this metadata's preservation.
Use a new small independent root-field reader before unchanged chemical consumers;
never weaken old ones to accept a new feature. Bounded projection fuzz uses original
seeds in a fresh ignored directory; no fixture/vendor samples are mutated or copied.

No registry command is added by this first format increment: count40, controls31/20/14/10,
catalogue12 delivered/128 planned remain. Follow immediately with the source-qualified
atomic settings command, then ordered native modifiers and full settings/chain workflows
with independent App and actual native/platform acceptance. Full M0–M5 stays incomplete.
