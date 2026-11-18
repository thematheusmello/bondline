# Ring sprout: source and interaction lock

2026-11-17, base `ec84717`. Previous goal turn is verified progress; full M0–M5
remains active. This contract precedes new runtime REDs and producer code. Actions
stays OFF; both older user applications/drawings remain protected. No proprietary
program, code, template, icon or help prose is inspected or adopted.

## Public facts and original scope

[GUIDE25](https://chemistry.beloit.edu/classes/programs/ChemDraw_25_manual.pdf),
printed101–102, distinguishes shared-atom/spiro attachment from a separate ring joined
by one bond. Alt-click requests sprout; File > Preferences… > Building/Display has
the sprout preference. Printed32/99 distinguish application preferences from current-
document settings. No chair exclusion for sprout is found. Shift phase belongs to
the two unsaturated tools; the chair exception concerns Ctrl-delocalization.
The pinned PDF17721447B/SHA256
`3f1ebc429df07efdd47b6218a148dc346a215043f60bf89ee9a1cb90629bd5b3`
and layout728816B/SHA256
`6af3ecb27a0b6c19f2b64005ad95582ba7795876c54a42bf210232f75e84c43f`
remain the versioned evidence, not proof of exact26.1/Mac defaults.

[IUPAC GR4.2.1](https://iupac.qmul.ac.uk/drawing/drawing.html#GR421) supplies ordinary
exterior substituent geometry; [GR4.2.6](https://iupac.qmul.ac.uk/drawing/drawing.html#GR426)
requires separate perspective treatment. The published SDK property/document lists
and official DTD expose no sprout preference property. SpiroOrHigher is query metadata,
not this control. No interoperable chemical-file attribute is invented.

Root reviews the frozen research9123B/SHA256
`94a581724fdf85b34d9adb746943552d04a3ee3cf9ccd9a6e4cf6d6167c13ae8`
and architecture8562B/SHA256
`543f3dba46c65362a0141de0a4b8f0437a8f470ca556a0416d04b8b37e8ac198`.
The exact interface, pose, timing, combination, preference lifetime and rendering
policies below are original Bondline choices. Alt inversion, Alt-drag, mixed modifiers,
restart persistence and exact modern defaults remain unestablished public behavior.

## Atomic authoring boundary

Add `ring.sprout`, preserving every existing43 command record/schema/order and public
RingCreate/NamedRingCreate/ChairRingCreate/RingPlacement API. Do not loosen their closed
objects or reinterpret shared-atom placement. A `RingSprout` parameter object has all
these required fields, with no defaults or extras:

- expected_revision, page_id, fragment_id, atom_id;
- hydrogen_policy exactly rectify_neutral_carbon;
- ring: a closed internally tagged object, kind exactly polygon with size3..8;
  benzene or cyclopentadiene with optional non-null phase double_first/single_first;
  or cyclohexane_chair_1/cyclohexane_chair_2 with no other fields;
- ring_begin_x_pt, ring_begin_y_pt, ring_end_x_pt, ring_end_y_pt: the new cycle's
  first edge endpoints, independent of the implicit old-root-to-new-anchor connector;
- side exactly positive/negative.

The Rust names are Command::RingSprout(RingSprout), with SproutRing::Polygon{size},
::Benzene{phase}, ::Cyclopentadiene{phase}, ::CyclohexaneChair1{} and
::CyclohexaneChair2{}. Engine::preview_ring_sprout(&RingSprout) returns the same
Result<StoredDocument,Error> shape as the existing ring preview, without authoring.

Use one immutable candidate and normal commit. Require current revision, supported
whole native/CDXML source and existing target-local positioned natural/neutral/nonradical
carbon with consistent literal C4 and at least1H. Preserve old arrays/IDs/positions/
display flags/bond orders/directions/styles and unrelated objects; only old root H drops1.
Append n new ring atoms to that same Fragment, then one old-root→new-vertex0 Single
connector, then n perimeter bonds in cycle order with closure last. No new Fragment ID
is allocated. New vertex0 H is4−cycle incident weight−1; other H is4−cycle incident weight.
All new atoms use nonterminal-label=false and absent terminal flag. Named Doubles keep
the existing inward directed style. Phase refers to first PERIMETER edge, never connector.
Named ranking uses the existing bounded order with a new-root incident budget3; the old
target H need not pay those cycle weights. Chairs retain the original orthographic shape.

Reuse source proof, existing regular/chair rounded certificates, exact page-center
occupancy, checked global identity floors, resource bounds and whole-source history.
The connector must be finite/nonzero. No mathematical minimum length or pixel/paper
restriction is added to headless authoring. Native full-scene ink/paper/readability
validation remains separate. Failures/preview spend no IDs, reserve no source/Save
authority and leave content/saved/history/file/selection exact. No worker or new model/
parser/IO encoding is required. Two existing authoring commands are not an atomic sprout.

## Application preference and presentation

Add one `ui.preferences` command with closed actions:

- inspect: no fields/host required, no mutation;
- set: required expected_epoch and sprout_rings_when_clicking boolean, no host required;
- open: required expected_revision, attached host, Open/idle foreground ownership;
- apply: required expected_revision, exact request_id and sprout_rings_when_clicking;
- cancel: required expected_revision and exact request_id; an obsolete matching form
  remains cancellable at the actual current revision.

ApplicationPreferences exposes sprout_rings_when_clicking and checked epoch. The original
initial value is false, preserving old shared-atom behavior. This preference survives
same-Engine New/Open/history/Save and host detach/reattach, independent of document source.
It is session-owned; between-launch storage and shared multi-window ownership remain
explicit future preferences work, not a fabricated CDXML property or a claimed26.1 rule.
It never dirties content/source, advances authored revision/history, changes selection/
effective mode/remembered bond order, or changes an existing drawing. A real set checks
both preference and shared-tool counters before mutation and advances each once; no-op
preserves both, contexts, redo and model sharing. Stale expected_epoch refuses even if
the supplied boolean happens to equal current. Committed close remains guarded.

The presentation follows File > Preferences… with a Building/Display section and one
implemented checkbox, Use Defaults, OK and Cancel. Draft changes/reset are local; OK
dispatches the registered apply action under the held Engine guard. Cancel never sets
the preference. Broader preference categories remain missing, not fake working pages.
Use Defaults sets this draft to the original false value. State clearly that these
preferences apply to all documents in the current session.

Reuse existing native weak-draft/modal/activation-prefix/retirement/close patterns.
Request IDs/open epoch are checked separately from preference epoch. Hidden
Open→Cancel→None invalidates older Ring/Bond/Selection/Chain input proofs even when the
boolean is unchanged. Capture source/generation, opaque association, File/close/view/
shared-tool and preference version; never revive stale Apply after ABA. Exact explicit
Cancel uses a request-specific serde-skipped cancellation token; context loss instead
retains a discardable stale draft with OK disabled. No Save receipt or close ACK is added.
Normal snapshots expose only an active ui_preferences request; explicit preference
replies also carry preferences state and a truthful optional ui_preferences_operation
with action, changed and optional request_id. Inspect/no-op set do not wake the host;
real set and open/apply/cancel wake AFTER reply publication and activity release.
Inspect and no-op Set omit the operation. A real Set reports action=set, changed=true
without request_id. Open/Cancel report their exact request_id and changed=false;
Apply reports its exact request_id and whether the preference changed, including false
when an unchanged draft closes. The outer authored changed remains false throughout.

Engine::preferences() returns ApplicationPreferences. UiPreferencesRequest exposes
id, input_revision, sprout_rings_when_clicking, current and a serde-skipped exact-Cancel
token through was_cancelled(). Engine::ui_preferences_request(),
ui_preferences_request_current(id) and finish_ui_preferences(id) mirror the existing
weak-form getters/retirement, never authoring or acknowledging Save. The preference
epoch counts real boolean changes; a separate checked presentation-open epoch supplies
request IDs and hidden-modal invalidation. Opening/cancelling alone changes neither
the setting version nor the shared-tool epoch. The normal state preferences field is
absent; explicit replies carry it, while an active request appears in ui_preferences.

The only new native binding is file.preferences. After both commands land, registry45
retains old43 records exactly; controls39/22/16/12 replace38/21/15/11. Tool catalogue
stays140 entries/124 roles with16 bindings/124 planned. No new tool is falsely enabled.

## Native sprout construction

On an atom-targeted press with any of the ten ring tools, sprout = preference OR Alt.
Alt forces rather than inverts. Bond/background ordinary actions remain unchanged;
Alt there visibly refuses with no detached/atom-editor fallback. Ctrl/Command combinations
remain unsupported. Named Shift alone retains its perimeter-phase meaning, including
preference-driven sprout. Alt+Shift remains unsupported in this first original policy;
its exact reference behavior stays open. Regular/chair Shift refusals remain.

Latch attachment and named phase at ordered press. A sprout-armed release may retain or
release Alt; late Alt on an originally ordinary/shared-atom press refuses without an edit
and cannot convert it to sprout. Unsupported release
chords cancel/refuse. Bond/Selection Alt atom editing, Chain Alt repeat, and fresh9/0 bond
fusion keep their independent behavior. No new atom hotkeys are claimed.

Click uses the existing outward unit-neighbor/smallest-ID cancelling-axis policy, or
rightward for an isolated atom. Both connector and first cycle edge use independently
present root BondLength, or original48pt only for known native absence. Put the new
cycle anchor one length outward; orient its centroid further outward. Regular polygons
use the existing interior-half-angle; chairs use their existing projected centroid.
Do not apply regular bisectors to chairs, move old geometry or infer stereochemistry.
No new snap/angle preference or axial/equatorial solver is supplied.

A drag uses the literal old-root→pointer vector as connector, with its length also
scaling the cycle reference edge; rotate the same centroid-outward construction.
Returning through a zero connector refuses instead of committing an older good preview.
This coupling is explicit original policy, not a documented Alt-drag algorithm.
Preview and release share the same full ring.sprout candidate, actual current camera,
physical origin/density/layer, glyph/stroke/paper/readability guards and one commit.
Source/tool/preference/view/File/modal/close/PointerGone/Escape/focus/DPI/zero-surface loss
cancels; equal restored values cannot revive authority.

## Test-first receiving and atomic integration

Freeze independent expectations before each producer: existing-API JSON UnknownCommand
and actual-App Alt/menu missing behavior REDs; literal C6H12→C12H22 sprout versus
C11H20 spiro; C2H6→C8H16/cyclohexane and C8H10/benzene, both chairs, Cp phases,
ordinary/ring/isolated/limited-H targets, sparse metadata/IDs, hostile fields/precision/
occupancy/source/close, exact Undo/Redo and actual Unicode files through unchanged raw
Expat/Decimal and external RDKit. Constitution checks do not prove perspective stereo.

Land the complete headless ring.sprout command/preview with its core, actual adapters,
render/file evidence and local gates as one focused commit; controls/catalogue remain
unchanged at that checkpoint. Then continue immediately through the separate preference/
native integration, retaining the frozen full contract rather than stopping at a prerequisite.
Test direct/no-op/stale preference, retained document/source/history through New/Open,
hidden modal/preference ABA, all modifier/target/phase paths, full previews, physical
guards, action-prefix/focus/defaults/cancel/staleness/close ownership and unrelated Alt owners.
Use bounded parser fuzz and exact old metadata/inventory checks; do not weaken old tests
or relabel setup qualifications as behavior REDs. Run full local gates and a separate
own native Mac workflow with independent actual-file readers before claiming native scope.
Human hardware/DPI, other platforms, full preferences/styles/stereo, exact26.1 behavior
and full M0–M5 remain required; no new approved golden or hosted run is assumed.
