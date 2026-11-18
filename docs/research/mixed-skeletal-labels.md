# Mixed authored labels and skeletal carbon

Locked2026-11-18 on basef16b79f. Continue full M1 and M0–M5. This connects foreign
explicit labels and explicit skeletal carbons through the existing display, selection,
label-edit, history and file workflows. It does not complete foreign defaults/styles,
hidden-node label creation, general chemistry or reference26.1 parity.

## Public evidence and original policy

The primary [Node specification](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Node.htm)
gives child Text precedence for display; absent Text only permits deriving a label when
appropriate. Projection's missing owned label is not proof that source Text is absent.
Inspect the owned XML children and reject every unsupported/multiple/unknown child.

The [property table](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/TableOfProperties.htm)
distinguishes terminal degree0/1 and nonterminal degree>1 carbon-label flags. The
[official DTD](https://static.chemistry.revvitycloud.com/cdxml/CDXML.dtd) provides yes/no
syntax. Neither establishes a safe omitted/root-flag inheritance rule for this workflow.
Require an explicit active node-level no; dormant presence/absence remains separate.
[NumHydrogens](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Atom_NumHydrogens.htm)
describes absent-H inference, which this increment does not implement: retain known
literal H and the existing checked Single/Double/Triple incident-weight+H=4 proof.

[IUPAC2008 GR2.1.1–2.1.2](https://iupac.qmul.ac.uk/drawing/drawing.html#GR211), GR0.1 and
GR4.1 supply skeletal and mandatory-label guidance. Reuse Bondline's existing original
degree/C4, isolated-C/C2, inclusive170° and two-Double policies. These are documented
depiction policies, not universal reference pixels or a general valence check. Missing
foreign fonts remain unresolved; the new capability may avoid needing one only where
the shared renderer proves that the CURRENT atom produces no glyph or mask.

The existing Connected root ink/width/margin, relevant length/spacing, resource, palette,
explicit run and no-p/Left/Left/Roboto face0/body/whole-token-box certificates stay exact.
No producer string, guessed style sheet or repaired numeric label substitutes for source
proof. No reference binary/export/icon/font/template is used.

## Shared renderer prerequisite

Add the pure `hidden_carbon_atoms(fragment, cancel) -> Result<BTreeSet<AtomId>, Error>`.
Reuse unchanged `skeletal::labels(fragment, None, cancel)` and return only atoms without
an authored label whose result is Hidden. Check cancellation initially and per collected
atom, including empty/no-flag paths. There is no H sidecar or inference. Do not duplicate
angle, graph-degree, C2 or C4 logic, change pixels/hits/clearance or expose the private enum.
Absence from this set is not proof that a glyph exists; legacy paths differ. The set
grants no source, chemistry, physical-page or mutation authority by itself.

## Separate complete source capability

Add a distinct Mixed profile and `StoredDocument::has_mixed_atom_labels`. Preserve old
foreign Boxed all-explicit/zero-bond and Connected all-explicit semantics literally;
source-free/exact-native richer conventions also stay unchanged. Mixed requires at least
one complete explicit authored label, at least one no-Text atom and at least one supported
bond. Every no-Text atom must be a CURRENT member of the shared hidden-carbon set.

Reuse the complete lexical/features/node/key/scoped-ID/resource/model binding proof and
Connected style/black-bond/pure noncrossing geometry checks. Each hidden source node must
have no element children, explicit Element6/NumHydrogens, ordinary owned NodeType/p and
the exact represented flag fields. No unknown/qualified/default property is ignored.
Missing/true active flags, unknown/invalid H, noncarbon/no-Text, visible mandatory carbons,
unsupported Text and absent foreign ink/font assumptions cannot gain admission.

Re-prove CURRENT hidden status in the getter AND writer after every delta. A hidden C
moved into the mandatory collinear band must refuse before a default font can appear.
Changing a labelled neighbor to carbon can make a previously hidden methyl atom part of
an isolated C2 component; command48 must then refuse atomically too. One fixed capture
or a valid baseline is insufficient evidence for either transition.

Mixed permits only the existing point deltas on no-Text atoms and synchronized existing
Element/H/body/box deltas on explicit labels. Preserve label presence, topology, IDs,
flags, styles/resources/captions/pages and all unrelated data. Splice changed existing
source leaves in one batch and completely re-import with complete-model equality and
the precise capability (or the existing exact-native equivalent). Preserve all unchanged
lexemes, source IDs, unknown-source refusals and whole-format loss guards. Global full
authoring remains false; no general chemical/page/style/caption writer is enabled.

## Existing command and UI integration

Extend only the old precise-label consumers: display/capture, selection availability,
existing command48/form for an EXISTING authored label, and native read-only/Alt routing.
All48 serialized metadata/schema/order and catalogue16/124 stay exact. No new tool,
shortcut, settings field or authoring command. Ring/chair/chain still require their old
full-native predicate. Hidden carbons remain selection/translation targets but command48
and its form must refuse them; child insertion and foreign label styles are separate work.
Inspector text must describe printed-label editing rather than promise hidden-node editing.

Reuse current raw point/box translation, rounded-identical no-op, weak owners, source/file/
view/modal/close/DPI/surface fences, one final source+settings/style-aware whole-page
certificate, immutable preview candidate and ordinary history/Save ownership. Exact
event-local Alt-primary still precedes Selection and unsupported chords remain refused.
No form, preview or failed move may publish partial changes or acknowledge newer Save.

## Independent acceptance and integration order

Freeze original source bytes, source-to-model IDs, geometry/font boxes, literal formulas
and exact output vectors before producer changes. Main scene: OH(96,160)–CH2(168,208)–
CH2(240,160)–CH2(312,208)–NH2(384,160), C glyphs hidden,512pt square, explicit1.25pt
line/2pt margin/48pt preferred length/18% spacing, black bonds and explicit blue labels.
Independently constructed/direct unsanitized RDKit plus Expat/Decimal/FontTools consume
the actual sources and outputs. Literal C3H9NO does not certify general chemistry.

Declare existing-API runtime REDs for display/capture/source edits and existing commands/
App/adapters separately from missing-new-API compiler failures. Cover labelled endpoint
edits, hidden-only/subset/all/compound moves, exact source/no-op/history/captured Save,
glyphless hidden-anchor hits, CPU/GPU and Unicode Save/reopen. Negatives include raw child
Text that projection cannot own, every source/style/flag/H drift, current170°/C2 changes,
crossing/degenerate/off-paper/ink failures, hidden-node form edits and all other mutators.

Land the shared pure renderer prerequisite with direct independent ID/geometry tests;
then frozen IO tests/source capability; then existing command/UI workflows. Use isolated
worktrees/targets, bounded parser/profile fuzzing and complete local gates. Inspect one
own disposable native workflow, preserve user apps/drawings, record platform/golden scope
and all failures exactly. Never weaken old tests, regenerate references to fit output,
rerun the protected MCP failure blindly or enable Actions. Full M0–M5 stays active.
