# Two cyclohexane-chair workflows: locked contract

2026-11-17, base59e3403. Previous goal turn is verified progress; full M0–M5 remains
active. The initial contract precedes the new runtime REDs and producer; the reference-
edge correction below is separately qualified before native implementation. Both existing grey
chair entries stay disabled until shared commands, independent App/files/adapters and
bounded actual native workflows are checked. No proprietary program/assets are used.

## Public facts and original choices

[GUIDE25](https://chemistry.beloit.edu/classes/programs/ChemDraw_25_manual.pdf),
printed101–102, lists two chair tools, generic atom/bond fusion, horizontal click and
drag orientation. Printed72–73's9/0 entries are **bond-hotspot** operations, not global
tool selectors. Shift's alternating-bond phase belongs to the two unsaturated tools;
chairs are explicitly excluded from Ctrl-delocalization. Alt-sprout and its preference
are separate still-missing ring workflows, not silently supplied by this increment.
Exact26.1/Mac defaults, templates, coordinates and modifier timing are not established.
The existing PDF17721447B/SHA2563f1ebc429df07efdd47b6218a148dc346a215043f60bf89ee9a1cb90629bd5b3
and layout728816B/SHA2566af3ecb27a0b6c19f2b64005ad95582ba7795876c54a42bf210232f75e84c43f stay pinned.

[IUPAC1996 chair definition](https://iupac.qmul.ac.uk/stereo/BC.html) supplies the
coplanar1,2,4,5 and opposite-side3/6 criterion. [IUPAC2008 GR3.3.1/4.2.6](https://iupac.qmul.ac.uk/drawing/drawing.html#GR426)
distinguishes perspective and its substituent placement from regular angle bisection.
[IUPAC2006 ST-3.1–3.3](https://iupac.qmul.ac.uk/drawing/stereo.html#ST3) supports clear
noncrossing perspective with optional bold emphasis; perspective can imply configuration.
These are functional/scientific facts, not permission to copy diagrams. New2006 HTML
223719B/SHA256b702d3493a3a64f4090cf75c697301d56ff3b5ae7861dc97d604286b7cd51cee;
1996 mirror18215B/SHA256bd28027e8f4e74badd7102be115e5378e30d5f7ea072db9ebec72e9cf6531844.
GoldBook
raw access returns403, so no body claim follows from its search record.

Root reviews the source-only9129-byte proposal SHA256
53b2ba719b085ac046fe8600b669bf347608db54fab3a8d587ce8b910924c514.
Its particular camera/indexing suggestion is **not adopted**. All geometry, scale,
phase rejection, pose/hotkey mapping/ties and lifecycle choices below are original
Bondline policy. No reference icons, templates, help prose or samples are redistributed.

## API and authority

Extend existing ring.create and ui.ring_tool; command count stays43. Preserve public
RingCreate, NamedRingCreate, RingKind, RingPhase, their constructors, field/null/default
semantics and existing two JSON alternatives. Add disjoint closed chair alternatives:

- ChairKind::One/Two serialize only as cyclohexane_chair_1/cyclohexane_chair_2.
- ChairRingCreate has required expected_revision,page_id,kind,hydrogen_policy,placement;
  the existing RingPlacement is reused. There is no size or phase field.
- SelectChairRingTool has only required kind. RingCreateRequest/SelectRingToolRequest
  gain Chair variants/From conversions, preserving old branches and getters.
- EffectiveTool::ChairRing{kind} joins the one effective mode/shared checked epoch;
  remembered bond order persists. No second selected-kind authority.
- RingToolOperation remains exactly size,epoch,changed, with derived6 for chairs and
  outer changed:false. Real changes/no-ops/source/selection epochs retain old rules.

Schemas and runtime must agree on the third closed object. Reject null, wrapper kinds,
both selectors, surplus size/phase and unknown fields without authoring. Actual CLI
comparison must preserve all41 unrelated records/schema/order; only ring.create and
ui.ring_tool acquire their additive branches. Native Drawing controls36→38, retained/
SDF/raw remain21/15/11. Only TD-027.1/.2 bind tool.ring.chair_1/.2, giving16/124 among
the unchanged140 entries/124 roles. Historical inventories stay historical.

Reuse the shared immutable ring append/H/IDs/source/history path. Six Single edges:
free6C/H2; atom adds5C and consumes2H; Single-edge adds4C and consumes1H at both old
ends. Existing target literal-C4, natural/neutral/nonradical/source guards stay. A Double
or Triple shared edge refuses; never rewrite it or fall back to a detached ring. Preserve
all old arrays/IDs/geometry/flags/styles/unrelated data. New degree-two carbons use literal
H2 and nonterminal-label=false; no new stereotype or bond style is inferred.
The new dormant terminal-label flag remains None/omitted, exactly the Atom default.

These are editable constitutional chair depictions, not a cis/trans classifier, conformer
optimizer, stereo-preserving ring flip or arbitrary perspective-substitution solver.
Keep this limit visible in guidance and parity. Existing unsupported stereo remains
opaque/refused. Full stereo/wavy/wedge workflows remain required by M2/M3.

## Original analytic geometry

An ideal chair has p_i=(r*cos(i*pi/3),r*sin(i*pi/3),s*(-1)^i*h), with
r=1,h=sqrt(2)/8,s=+1 for One and−1 for Two. Adjacent3D lengths are sqrt(9/8) and
outgoing-bond cosine−1/3; atoms1,2,4,5 are coplanar and3/6 lie oppositely. This is an
independently derived seed, not traced art or a force-field/energy result.

Project X=x,Y=g*(z−y/4),g=4/sqrt(17) (y-down), an original orthographic view with unit
perpendicular image axes. Set q=sqrt(3)/8. Final source indices are
One[1,0,5,4,3,2],Two[3,2,1,0,5,4], preserving six-cycle adjacency:

    One:
    P0=(1/2,-g*(h+q))  P1=(1,g*h)        P2=(1/2,g*(q-h))
    P3=(-1/2,g*(q+h))  P4=(-1,-g*h)      P5=(-1/2,g*(h-q))
    Two:
    P0=(-1,g*h)        P1=(-1/2,-g*(h+q)) P2=(1/2,g*(h-q))
    P3=(1,-g*h)        P4=(1/2,g*(h+q))  P5=(-1/2,g*(q-h))

Both are noncrossing positive-area projections, distinct horizontal poses. Their
reference edges e=(1/2,+g*(2*h+q)) for One and(1/2,−g*(2*h+q)) for Two are outer end-
bonds; D=|e|²=(7+sqrt(6))/17. Normalize to T0=(0,0),T1=(1,0), with
T_i=(dot(Pi−P0,e)/D,cross(e,Pi−P0)/D). For requested A,B with E=B−A:

    actual_i = A + T_i.x*E + sign*T_i.y*J(E), J(E)=(-E.y,E.x)
    sign=+1 for positive side,−1 for negative.

Keep A/B exact. Foreshortened edges are intentionally unequal; do not weaken or reuse
the old regular-polygon equal-edge/angle certificate. A separate six-point similarity
certificate checks actual rounded normalized coordinates against the analytic template
within1e−10, finite/nonzero reference scale, distinct vertices and model bounds. Refuse
rounded collapse/loss without imposing a minimum mathematical length. Existing global
center occupancy and native full-scene stroke/glyph/paper/readability guards stay.

Both projected signed areas have the declared side. All other vertices lie strictly
in the declared reference-edge half-plane, so ordinary outward attachment has an
exterior ray rather than a concave anchor inside the chair.
The Root original SVG preview validates the visual design only, not a product or golden.
No physical3D state, stereo assignment or rendered depth is added to the document.

The initial same-shape indexing[2,1,0,5,4,3] used a concave reference edge. Core2/modal2
passes50201 establish only that first declared API/projection and guard fix. Independent
App source review then shows centroid-outward ordinary attachment would intersect the
old terminal bond. Actual registry-output test67687 fails at a vertex across the edge.
Before native producer/expanded App acceptance, Root changes only the reference indexing
as above, preserving the3D seed, camera and projected polygon. Independent Python derives
new strict vectors/D and supporting-edge checks before adoption. Earlier test/source/
vector snapshots stay immutable; only new unshipped vector expectations change with this
explicit policy correction. No target restriction, tolerance relaxation or old regular/
unsaturated behavior changes. Ordinary terminal and spiro success remain acceptance goals.

## Native click, drag, attachment and hotspot keys

Free click uses a horizontal pose: its kind's first-edge direction normalize(e), length from
independently present root BondLength or original48pt only for known native absence.
No stored default is materialized. ChainAngle and chain Fixed switches do not affect
chairs. Preserve all older regular/unsaturated click behavior. Drag replaces only this
first-edge vector with the actual press→pointer vector, rotating/scaling the same whole
candidate; vertical orientation is available by that rotation.

Atom click uses the existing normalized outward-neighbor vector and deterministic
smallest-ID/cancelling-axis rule. Orient the template's anchor→centroid vector along
that outward vector, retaining its projected angles and reference length. It is an
original placement policy, not regular-angle bisection or inferred substituent stereo.
An isolated atom uses the horizontal default. Bond placement preserves exact old B→E;
reuse local occupied-side preference, preferred/opposite complete candidates and actual
ink/paper validation. No automatic old-structure movement, stereo restyling or face solve.

Only unmodified primary boundaries are accepted for chairs. Shift/Control/Alt/Command
or combinations visibly refuse/cancel with no atom-form/bond/phase fallback. Keep old
sized and unsaturated modifier behavior unchanged. PointerGone/Escape/focus/source/view/
tool/File/close/DPI/layer loss cancels; one release is one ring.create edit. Whole preview
and release use the same candidate; no alternate successful old preview may commit.

9 maps One,0 maps Two, only unmodified nonrepeat keys at a fresh actual bond hotspot.
Use the existing ordered hotspot position/context and actual graph/paper/layer picking,
not final-frame future pointer state. No pointer/bond means no edit, no mode selection.
Text/modal/busy/close/invalid-source/active-gesture contexts cannot author. Same shared
chair candidate and side trials run immediately under the current Engine guard. Retain
the active tool, use one history edit and clear hotspot after authoring as other edits do.
Expose this contextual meaning in guidance/parity; do not assign global9/0 tool shortcuts.

Ring weak contexts additionally capture Atom/settings/Add open epochs, even with no
remaining owner. None→Open→Cancel→None must not revive an older ring gesture. This
strengthens the common intended modal fence without changing source/File/Save authority.
No worker, FilePlan, Save lease/receipt, close ACK, new model/parser or dependency.

## Independent receiving before unblocking

1. Existing JSON chair-kind runtime RED and actual-App disabled-chair RED precede producer.
   Separate setup/type/consumer qualifications from behavior failures.
2. Independently compute3D length/dot/coplanarity/projection and literal2D coordinates;
   both kinds/sides, horizontal/vertical/scale, no crossings, precision/occupancy failures.
   Preserve every old regular/unsaturated oracle and constructor/schema branch.
3. Independent constitution/metadata/history/IDs/source guards and Unicode files. Free
   C6H12, ethane-atom C7H14, cyclohexane-spiro C11H20, ethane-edge C6H12 and fused C10H18
   are checked with raw Expat/Decimal before unchanged RDKit. RDKit is not a perspective
   stereo oracle. Existing format-loss guards stay explicit.
4. Actual CLI/initialized stdio/authenticated loopback, strict schema, selected mode,
   no-op/repaint and saved files. Registry43 unchanged;41 unrelated metadata records exact.
5. Actual App controls/free/drag/atom/bond/hotspot9/0/cancel/Undo/Redo/files, ordered future
   pointer/key/modal/source/ABA/stale surface, full1×/2×/fractional ink and picking. Real
   CPU/GPU output equality, no new golden. Review source and run all local gates.
6. Separate own native Mac app: both horizontal/vertical tools, attachment and9/0,
   invalid-modifier recovery, preferences, Undo/Redo, Unicode Save/reopen and actual-file
   readers. Preserve both user bundles/drawings. Native/hardware/DPI/other-platform scope
   stays exact; full M0–M5 and remaining grey tools remain incomplete. Actions stays OFF.
