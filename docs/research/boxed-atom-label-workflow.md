# Box-positioned literal atom label workflow

2026-11-18, base3b4dc17. Continue the full M0–M5 target with a real supported Node
child-Text rendering/edit/history/Save workflow. This does not resolve arbitrary explicit
Text p, formula faces, inherited defaults, general labels or full native/reference parity.

## Source contract and original policies

Primary [2DPosition](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/2DPosition.htm)
resolves an absent Text p from BoundingBox before its Node. Primary
[BoundingBox](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/BoundingBox.htm)
defines the enclosing graphical rectangle; [coordinates](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/DataType/CDXCoordinates.htm)
defines point units, x right/y down and left/top/right/bottom rectangle order. Use this
positive path without inferring a universal Node-child p offset. Previous contradictory
Octet/Indigo/ReShiki p conventions remain qualified, not resolved by this feature.

The original checked profile requires absent t.p, present nondegenerate BoundingBox,
explicit Justification=Left and LabelJustification=Left, absent LabelAlignment, exactly
one explicit Roboto Regular face0 run and no carbon-display flags, isotope, charge or
radical. Literal body must equal the numeric element symbol followed by H/count when
NumHydrogens is known and positive (count1 omits the digit); physical H requires0.
Digits stay on the baseline: face0 is literal text, not chemical script formatting.
Known H/count agreement grants no valence or general chemical-validity inference.

Use the bundled SHA-pinned font. Derive actual outline bounds using OpenType curves,
advances and existing GPOS kerning, not loose control-point bounds. Reconstruct baseline
from the whole ink rectangle; qualify that the complete element-symbol union's ink-box
centre equals Node.p. This centre check is an original admission policy, not a universal
reference centering claim. Match coordinates within one CDX fixed16.16 point quantum
(1/65536pt), plus128*f64::EPSILON*max(1,absolute coordinate) rounding. It does not excuse
changed size, unsupported placement, body/style/default ambiguity or clipping.

## Model and render interface

AtomLabel adds private optional bounding_box:[f64;4]. `bounding_box()` returns the copy;
`with_bounding_box(Option<[f64;4]>)` is checked. Require finite ordered coordinates within
±100000pt, with degenerate boxes retained as data but not rendered. Existing new signature
is unchanged and startsNone. Omit None from old JSON, accept omitted/null input, reject
wrong shape/invalid bounds. Other checked copies preserve the box. Existing combined
text budgets and unique Arc/capacity accounting stay, including the larger inline payload.

Render adds `position_atom_label(&Atom)->Result<AtomLabel,Error>` to create a canonical
literal run/body and whole box around the unchanged Node element anchor, preserving its
one run's exact style and supported placement fields. It ignores an old box for this
explicit generation, but never accepts unknown p/default/face/flags. It is not valence
inference. `validate_atom_label(&Atom)->Result<(),Error>` requires raw body/box to match
that generated certificate. Missing boxes and all old unsupported labels still refuse.
The shared graph producer paints authored run glyphs/colours, clips with actual masks,
retains Atom hits, em min/max and all ink/paper/gap/collision/raster/cancellation guards.
Use existing caption GPOS/raster helpers where possible; old labelNone pixels stay exact.

## Source, commands and native integration

Global is_fully_mapped staysfalse for every authored label. Add separate closed complete
source capability `has_boxed_atom_labels()` for qualified CDXML/native/source-free data.
It validates all lexical objects/keys/scoped positive IDs/resources/page/style/model
correspondence and the complete label geometry/body profile; names alone grant nothing.
Native data encoding owns BoundingBox. Unsupported/raw sources stay unchanged-only.
An admitted foreign writer changes only existing Node Element/NumHydrogens, its one s
body and its t BoundingBox via one syntax batch. Numeric no-ops retain every source byte.
Delta permission preserves all other content, identities, geometry, styles and context;
complete re-import certification follows writing. No source ID is an application ID.

Append command48 `atom.set_authored_label`, using the same flat target/element/H parameter
shape as AtomSetLabel, without changing any old47 metadata/schema/order/description.
Only an existing qualified label target may use it. Source/revision/target authority
precedes domain/no-op; validated no-op preserves Arc sharing, redo, selection, close,
file and weak contexts. Changed edit generates body/box, preserves Node/bond geometry and
IDs, certifies final complete source+page once and commits one ordinary history entry.
The old atom.set_label stays refused for authored labels. Every other dependent H/position/
style/page/creation/caption operation keeps its existing refusal. MOL loss and retained
SMILES guards stay; no new export or partial source authority is inferred.

Reuse exact-target ui.atom_label and the existing bounded Element/H native form; its old
generic presentation metadata/JSON remains exact. Choose command48 under the same Engine
lock/request/source/file/view/tool/close/epoch proof; old targets keep command34. A qualified
read-only canvas supports exact Alt-primary atom-target press before mode gestures, even
when File Open retains a prior Ring/Chain/Selection tool. Other canvas authors remain
refused. Preserve frame-start/action-prefix/layer/surface/DPI/modal guards, explicit Cancel,
stale copyable drafts and target focus. No new grey Text-tool binding is claimed.

## Acceptance

TDD starts with existing-API runtime REDs and missing-API qualifications kept separate.
Use original FontTools BoundsPen/hmtx/GPOS + Expat/Decimal vectors, Pillow pixel masks,
independent numeric/formula checks and deliberate inconsistent-box/body/font/source
controls. Verify actual CPU/GPU/PNG, current physical view/hits/masks, complete source
edits/history/capturedSave/no-op/stale contexts and Unicode files through API/CLI/MCP/
authenticated socket. Exercise native Open from a prior non-Bond tool, Alt-target/edit,
Cancel, history, source-preserving Save/reopen and protected user windows when access
permits. No guessed native or other-platform pass, no changed old golden/font/dependency.
Run local gates/bounded fuzzing and commit atomic increments; Actions stays OFF.
