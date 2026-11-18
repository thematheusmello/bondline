# Skeletal carbon display and explicit label restoration

2026-11-16, based on data caa8d60 and desktop checkpoint27b87a5. This locks the next
shared-renderer/existing-command increment before tests or product changes. Complete
aromatic tools, terminal/H preferences, styles and M0–M5 remain separate required work.

## Source and original decisions

[SDK0x0443](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/TableOfProperties.htm) defines
ShowNonTerminalCarbonLabels for carbon with more than one bond; the
[official DTD](https://static.chemistry.revvitycloud.com/cdxml/CDXML.dtd) supplies node/root
yes/no. [IUPAC2008 GR2.1.1–2.1.2](https://iupac.qmul.ac.uk/drawing/drawing.html#GR211)
supports ordinary skeletal C4/H conventions and requires unambiguous labels. Its
[GR0.1](https://iupac.qmul.ac.uk/drawing/drawing.html#GR01) defines collinear within170–190°;
[GR4.1](https://iupac.qmul.ac.uk/drawing/drawing.html#GR41) requires every two-Double carbon
to be labeled. These are representation standards, not a reference26.1 algorithm.
Root/node inheritance and explicit child-Text precedence remain unknown and source-backed.

The reviewed proposal14544B SHA256f4de1d48c7dad49352d156152c22a3552cea20f1ba5535b6d8654ae5009f3824
and receipt7220B SHA2564e22bba04ae6d047f0b8baddced5a4bb91ad26344fc96b693918e6c3ca5202f3
pin primary source bytes/callers. The following tri-state compatibility, proof refusal,
numerical boundary and conditional authoring rules are original Bondline policy.

## One shared label decision

- None uses the exact old label/pixel/refusal path. A specified flag on noncarbon or
  actual graph degree0/1 is dormant and uses that same path, with metadata retained.
  Degree counts incident Bond objects, including physical-H bonds, not stroke count or
  weighted order. Isolated known-H carbon/C2 labels therefore remain visible.
- Active means neutral natural nonradical carbon with degree>=2 and Some(flag). Existing
  metadata/order/position guards still run. Require known literal H, or the existing
  complete validated display-count sidecar; never choose H or mutate the model.
- Active true displays the full known C/H label through existing placement/clearance.
  No new C4 restriction is imposed on a deliberately printed literal count.
- Active false requires checked W+h=4, where actual incident Single/Double/Triple weights
  are1/2/3. Unknown H, unsupported orders or failed integrity refuse unsupported_content;
  a public sidecar's complete coverage alone is not a C4 proof.
- After that proof, show the full known label for any two-Double carbon and for degree2
  identical-order rays whose included angle is170–180°. Otherwise suppress the label.
  Mandatory CH2 stays CH2; a cumulene H0 center prints C. Do not erase chemical counts.
- Classify from authored f64 outward rays, normalized with hypot before dot products.
  Use inclusive dot<=-0.984807753012208 (the rounded cosine170° threshold), independent
  of storage/directed endpoint order, camera or DPI. No acos, squared tiny lengths,
  geometry repair, larger junction mask or tolerance weakening. Existing physical
  f32/pixel/readability/whole-paper checks still judge the actual produced scene.

Suppression removes glyph pixels and every white label/supplement mask together; stable
AtomHit anchors/IDs and logical Bond hits remain. Existing real stroke/gap/role joins,
distinct ink, nonincident collisions, bounds, quotas, source completeness and cancellation
remain strict. Incidence is collected once per fragment, not by a bond scan per atom.
None/dormant scenes must retain their exact old geometry, labels, masks and goldens.

label_em_px describes only actual labels: None for an entirely glyphless scene. Existing
native readable_graph then has no font claim to assess; stroke/gap guards remain. Do not
invent a nominal font to label a hidden glyph readable. The separate SDF-preview Some(em)
requirement is unchanged; this native-data increment does not alter its format policy.

## Explicit authoring through existing interfaces

atom.set_label restores an existing false to true on its target, alongside supplied
element/H; preserve None as None and true as true. Same element/H + false→true is one
real edit/history/revision. Repeated true and legacy None same-count calls retain old
validated no-op sharing/redo/dirty/close behavior. Other atoms, metadata, topology and
coordinates remain exact. Changing to a heteroatom still retains the explicit true datum
as dormant. No new command, parameter, UI form, source authority or neighbor inference.

The existing exact-target native editor permits unchanged valid fields to Apply and
already calls this registry command. Alt-targeting a hidden anchor must still work;
Cancel leaves false, Apply restores, and history/Save/reopen retain the standard data.
CLI, initialized stdio MCP and authenticated loopback must execute the same transaction.
Old source/epoch/modal/event/worker/close safeguards remain. The user's open desktop app
and unsaved draft stay untouched; offscreen App evidence is not native OS acceptance.

## Independent acceptance and continuation

Observe real runtime RED before implementation. Original bent chain/regular Single and
Kekulé rings require actual hidden glyphs/masks, analytic strokes/pixels, stable atom hits,
literal model/source and CPU/GPU agreement. Separate None/true/dormant and identical-ray/
cumulene, degree-vs-weight, physical-H, unknown/invalid-C4/sidecar, scale/bounds/cancel
controls qualify the policy without weakening old guards. Upgrade only the prior DATA
blanket-refusal test positively, retaining its exact inputs and stronger label assertions;
its collinear CCO center must stay labeled. Keep its legacy/cancel cases unchanged.

Actual Unicode Open→capture→same-count explicit label→Undo/Redo→Save/reopen and existing
CLI/MCP/socket/native-editor routes must prove flag/H/ID/coordinate retention with exact
properties and unchanged independent graph/RDKit consumers. Run local fmt/clippy/tests,
layer/license/corpus gates; preserve natural MCP failure evidence without retries.
Then implement weighted benzene/cyclopentadiene candidates and real tools, opting only
new intended skeletal atoms into false. Preserve shared existing atom flags; shared-edge
phase/order precedence still needs sourced rules or explicit original policy. Actions
stays off; native/platform evidence and the complete parity target remain outstanding.
