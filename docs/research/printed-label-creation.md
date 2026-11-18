# Explicit label creation on imported skeletal atoms

Locked 2026-11-18 on `7113237`. The full M0–M5 goal remains active. This is a
bounded M1 workflow, not completion of label tools, foreign defaults or chemistry.

## Evidence and original choices

The public [Node specification](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Node.htm)
describes contained Text as the displayed label, and chemical-property derivation when
Text is absent. The [Text specification](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Text.htm)
and [DTD](https://static.chemistry.revvitycloud.com/cdxml/CDXML.dtd) permit child Text and
explicit style runs. Keep both existing carbon-display flags when adding Text. The
general precedence rule supports this choice; the archived page does not separately
specify every modern flag/Text interaction or a version26.1 pixel algorithm.

[IntroCDX Objects](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/IntroCDX.htm#Objects)
specifies four-byte object identifiers, container uniqueness and recommended document
uniqueness. The Node/Text property tables' UINT16 id wording is contradictory; do not
narrow object IDs to match font-resource IDs. Existing resource documentation separates
UINT16 font IDs from chemical objects. No reference executable or asset is used.

Roboto face0, a canonical literal body, whole element-token anchoring, no Text p,
Left/Left placement and exact checked outline bounds remain original Bondline policies.
The new command takes an explicit style. It uses matching existing font/palette resources
only (deterministic lowest matching ID); unavailable resources refuse. It neither reads
neighbour defaults nor edits resource tables. The native form visibly states its original
Roboto Regular12pt black authoring preset. This is not a foreign-file default claim.

## Renderer and precise source authority

`create_atom_label(&Atom, TextStyle)` requires absent authored data and generates checked
owned data through the existing positioning implementation. It does not mutate the atom
or certify source, chemistry or a physical page. An existing authored label takes display
precedence over dormant carbon flags. Skip only its own skeletal classification, retaining
all topology/element maps so OTHER hidden carbons still receive the old C2/C4/angle checks.
Every old no-label path, glyph/ink/clearance/hit rule and tolerance stays unchanged.

Preserve old foreign Boxed, Connected and Mixed admission with explicit absence fences
for flags on authored nodes. A separate `Flagged` profile / `has_flagged_atom_labels`
requires at least one flag-bearing authored label, at least one supported bond, complete
existing source/resource/style proof, and CURRENT Hidden neutral/natural/nonradical,
known-H/C4 carbon for every remaining no-Text node. Zero remaining hidden nodes is valid.
Recognize a complete Flagged source before the canonical fallback; producer text grants
nothing. Global full authority stays false and unrelated writers/loss guards stay closed.

The new capability may also prove a Mixed baseline plus current inserted labels. Keep
the immutable original source/baseline: never rebase it after creation. Baseline Some
retains presence/style/flags and allows the old point/Element/H/body/box deltas. Baseline
None may gain one supported label, retain every other identity/metadata/flag field, and
subsequently compose point and numeric/body/box changes. Some→None is never allowed.
Remaining no-Text atoms must be currently hidden after every neighbour/geometry edit.

Reserve fresh positive globally unused object IDs for ALL baseline eligible no-Text
atoms in source order. Inventory every original object ID, including child Text/captions,
but exclude font IDs. Emit only currently created children. Thus reverse creation order,
undo/redo and later edits cannot rename an already emitted child. Reopening preserves
actual IDs; hypothetical unused reservations need not persist. Append through the existing
checked raw-QName/self-closing/paired `AppendChildren` path, compose original-node splices,
and completely re-import for exact current-model and precise-capability certification.

## One registry operation and native transaction

Append command49 `atom.create_authored_label`, typed `AtomCreateLabel`, with required
expected_revision/page_id/fragment_id/atom_id/element/explicit_hydrogens/style. Style is
a closed object with font_family/size_pt/face/rgb. Old48 metadata/schema/order stay exact.
Authority/revision/current-hidden target checks precede value/no-op processing. Repeated
creation on an already labelled node refuses. Explicit Element/H may change in the same
creation, so one native Apply is one history entry. Preserve both flags, position, IDs,
other metadata and current style ownership. Existing command48 remains existing-label
only and edits newly created labels without legacy flag restoration.

Reuse `ui.atom_label` input schemas, exact event-local Alt targeting and all old weak
source/model/file/view/tool/modal/close/Cancel/ABA owners. Open an eligible hidden target
with a visibly stated creation appearance; an optional creation_style response field is
omitted on old forms. Apply dispatches49 through the same registry under the same Engine
guard; printed forms retain48 and legacy numeric forms retain their old behavior.
Cancel/failed Apply publish nothing. A candidate must pass source re-import and one final
settings/style-aware physical-page certificate before ordinary commit. Selection may
consume the new precise capability, but ring/chain/bond/page/style tools gain no authority.
Static catalogue16/124 is unchanged.

## Independent acceptance

Freeze original bytes, FontTools boxes/body, source/model IDs and Expat/Decimal/RDKit
facts before producer changes. Cover middle CH2 creation, retained flags, deliberate
blue/black styles, self-closing and paired/comment nodes; order3→2→4 versus2→3→4;
partial/all-labelled Save/reopen; old48 edits, point/box moves, exact history/redo/no-op,
captured older Save and stale/modal ownership. The dense-ID source occupies object1..13
but font14: reserve child14/15/16, so first app3 gets15 and later app2 may use14.

Declare runtime REDs through existing APIs separately from missing-new-API compiler
failures. Preserve unknown source/children/namespaces/features, active flag/H/C4/C2/angle,
resource/style, paper/ink/work/cancellation, whole-format and unrelated-mutator refusals.
Compare actual CPU/GPU scenes with independent source/font/geometry controls. Exercise
CLI, initialized MCP, authenticated socket and the actual own native Open/Alt/Apply/
history/Unicode Save/reopen/Cancel flow. Native acceptance never touches an older user
app/drawing. Human golden approval, other-platform execution and full parity stay separate.

Use atomic commits, complete local gates and bounded parser/profile ASan. Actions stays
OFF; no dependency, lock, vendor, font, asset, old golden or publication change is needed.
