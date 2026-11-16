# Persistent double-bond placement prerequisite

2026-11-16. Contract locked before new declarations at185476d. This is a prerequisite
for aromatic tools, not completed benzene/cyclopentadiene or skeletal display. The full
parity target remains in ROADMAP.md. No reference binary, preset or artwork is used.

## Public evidence

- [SDK DoublePosition](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Bond_DoublePosition.htm):
  optional0x0603/INT16; directed Center/Right/Left. CDX0/1/2 and256/257/258 distinguish
  automatic/manual; XML uses identical names for both. Absence is unspecified.
- [IUPAC2008 GR1.10/GR3.5](https://iupac.qmul.ac.uk/drawing/drawing.html#GR110):
  sided doubles have an axis stroke and a parallel stroke with adjoining-angle-dependent
  shortening; ring context matters. The existing centered profile does not supply it.
- Pinned GUIDE25 printed101–103 supplies named unsaturated-ring roles and Shift phase;
  inset rendering and skeletal labels are needed for their conventional appearance.
  Shared-edge phase/order precedence remains undocumented. See sources.md and the
  existing public-source pins; no25/26.1 default is inferred.

## Locked first data/format slice

Add public model enum DoublePosition with Unspecified (default), Center, Right, Left.
Bond gains `double_position: DoublePosition`. JSON lower-case values; missing defaults
Unspecified and that value is omitted when serializing. Invalid values remain errors.
All old model JSON and absent native XML bytes must stay exact. This fixed-size field
adds no GUI dependency, chemistry inference, style sheet, heap owner or new crate.

This is direction/presence data only. It does not fabricate the CDX automatic/manual
bit from an XML name. The standalone CDX codec continues preserving that opaque data;
no new CDX projection/edit authority is introduced. A placement property on a non-Double
bond is retained as data, without guessing its active behavior or silently dropping it.

CDXML ElementalGraph maps exactly Center/Right/Left; absent means Unspecified. Unknown
spellings fail typed mapping atomically while the standalone syntax layer still retains
the entire source. Native writer emits the attribute only when specified. Native source
proof remains exact-byte re-encoding, never a broad completeness override. Unknown/
foreign source remains guarded; unchanged complete sources retain exact bytes.

MOL V2000/V3000 cannot express this depiction choice in the delivered subsets. Their
model writers must explicitly refuse any specified placement, including dormant or
Center metadata, before writing partial output. Delivered SDF/RXN writers wrap existing
MOL bytes; there is no model-taking writer to invent or claim tested. Their raw-source
round trips remain exact. Existing explicit graph-only SMILES export is a separately documented
lossy semantic export; it must not be confused with complete-document Save.

Until the shared renderer supports sided geometry, Left/Right drawings must refuse
scene construction rather than show the old centered profile. Explicit Center on a
Double uses the existing centered profile. Specified placement on other bond orders
also refuses rendering until its semantics are separately defined. No point, glyph,
paper, gap, visibility, work, cancellation or source-authority guard is weakened.

All existing Bond literals/construction sites must explicitly initialize Unspecified;
that compiler-driven change must not alter other values, old tests or oracle inputs.
Retained-memory accounting must use the actual Bond size; do not change history quotas.

## Test-first acceptance

1. Freeze a small public model missing-API RED separately from format runtime RED.
   An existing-JSON literal with double_position:right currently fails unknown_field;
   decode it against independently declared IDs/order/coordinates/H and require exact
   new field persistence plus old omitted-default JSON. Do not count a compile failure
   as evidence that an old supported case behaved incorrectly.
2. Before code, declare SDK-name CDXML vectors with Center/Left/Right and absence,
   literal old bytes, invalid spelling/unknown source preservation, direction reversal,
   native Save/reopen/source proof and independent Expat property checks. A reader/writer
   self round trip alone is insufficient. Preserve old fixtures/pins.
3. Literal positioned metadata must cause both MOL model writers to refuse without
   partial publication or replacing a session; raw SDF/RXN controls remain exact. Absence
   must retain every existing result. Explicit Center must not be silently discarded.
4. Independent renderer expectations: sided and non-Double specified metadata refuse
   without a partial scene; Center/absent Double retain the sealed centered profile and
   its independent stroke/glyph/gap controls. Old approved Single golden stays exact.
5. Run relevant focused RED/GREEN, full local fmt/clippy/workspace/xtask gates and
   bounded parser fuzz where the changed mapper is exercised. No hosted execution.

## Following dependencies, still required

A shared inset geometry/corridor/gap/hit path must use adjoining-angle bisectors and
actual component bounds. Conventional skeletal carbon presentation needs its own
source-backed policy without deleting literal H or changing old literal-label output.
Then extend the actual ring candidate with weighted C4 demands, phase and attachment
rules, one effective tool/context/history path and real preview/releases. Fused aromatic
systems must not blindly add three new Doubles or mutate old neighbors outside one
candidate; cyclopentadiene is C5H6 with an sp3 vertex. Delocalized circles, aromaticity
perception, general fusion/chair/sprout and complete native/platform parity stay visible.
