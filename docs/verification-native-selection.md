# Native selection acceptance and cancellation fixes

2026-11-17, base `0c9e6af`; full M0–M5 remains active, Actions OFF.
The [predeclared native workflow](research/native-selection-acceptance.md) continues the
[existing command/App contract](verification-selection-authoring.md). TD-002 is Marquee;
TD-001 Lasso stays missing. Version25 public facts and original Bondline policies remain
separate; no proprietary executable, asset, font, template or sample is inspected.

## Baseline actual desktop workflow

Root copies the already built normal executable into a new unique own review bundle.
It is24941856B/SHA256
`56feca8c1436238150fedcf03a34d3982aea2ca48730e1f9502bf4fd5e885cde`, matching the preceding
build's product code at0c9e6af. This is not a new build or a post-fix native claim. Platform
is macOS27.2/AppleM3Max,1160×832 window, Actual100%; input is OS-injected, not human hardware
or measured2×/DPI. No listener is started. Explicit own titlebar/keyboard focus precedes
the first drag; later F5 precedes canvas actions. The two older user apps stay protected.

From an empty drawing, two80pt C2 bonds are drawn through the UI. Alt-click on the first
terminal opens Atom label; literal O/H1 is visibly confirmed before Apply. The actual
Unicode original file saves cleanly. Marquee activation, atom click, Shift add/remove,
bond endpoints, empty click, first-atom rectangle, reverse-direction second-pair rectangle,
component double-click, CmdA, Edit > Deselect/Select All and Escape show the expected blue
selection overlays while retaining Saved and4atoms/2bonds/2fragments. The first double-
click checks separate Fragments; same-Fragment disconnected components are a separate gate.
Additive Shift-marquee is not exercised through this native wrapper and stays App-only.

Select the methanol pair, then drag(24,-12)pt. Only that pair moves. One CmdZ restores the
original; CmdShiftZ restores the move. After reselecting, Right then Shift+Down adds(1,10)pt.
Each nudge has its own Undo; two Undos restore the moved drawing. Save As uses distinct
fresh Unicode paths, each visibly confirmed before Save. Native Open of the nudged file
restores a clean editable drawing; selecting the ethane pair and Up moves only it by1pt.
That edit saves under a sixth fresh path. Explicit Quit and a later separate app inventory
observe the own review absent. No action/query touches either older user drawing.

## Independent actual saved-file evidence

Each actual file is691B. Python Expat checks exact root/page, source XML IDs, ordered
atoms/edges and every non-position attribute. Decimal checks exact integer positions
and80pt lengths; no geometry tolerance is needed here. External pinned RDKit2026.09.2
reads each CDXML directly: CO/CH4O and CC/C2H6, literal H3/1 and3/3, unchanged chemistry.
The first two positions change as predeclared; the second pair stays(350,477)/(430,477)
until the explicit reopened edit. File/selection state is not inferred from our own reader.

| Native output | First pair positions, pt | Second pair positions, pt | SHA256 |
| --- | --- | --- | --- |
| selection-original-λ.cdxml | (150,277)/(230,277) | (350,477)/(430,477) | `f819b8fcffaf5a5a73cbb10c24145ff12d679353dec7b5a0cf57319576d9296e` |
| selection-moved-λ.cdxml | (174,265)/(254,265) | unchanged | `9965231cfde602a5bda975455925857f10722f76b43f10b907ff0bbca1dd0468` |
| selection-undo-λ.cdxml | original | unchanged | `f819b8fcffaf5a5a73cbb10c24145ff12d679353dec7b5a0cf57319576d9296e` |
| selection-nudged-λ.cdxml | (175,275)/(255,275) | unchanged | `d2f114ab203bb928827bb7ade416e7c909f2ff618c7dec977d53b31228d01d28` |
| selection-restored-λ.cdxml | moved | unchanged | `9965231cfde602a5bda975455925857f10722f76b43f10b907ff0bbca1dd0468` |
| selection-reopened-edit-λ.cdxml | nudged | (350,476)/(430,476) | `82fed8b9ff2512ec80fb4a05476e5e2f95fec2368b75a3ce382f09fbcf58ba73` |

Undo/original and restored/moved are byte-exact pairs. Ignored raw files/consumer report
live under target/native-selection-acceptance/native/, not in the licensed corpus or
approved goldens. No screenshot substitutes for coordinate/chemistry assertions.

## Independent cancellation regressions and actual RED/GREEN

The bounded source review freezes10 inputs,4784B report/SHA256
`f444a897ec8ebb7c5051746a2080d9a276f20fa0361be891e5a4a63ac09c8f31`.
It proposes two hypotheses, not executed failures. Root then adopts independent declared
tests before changing producers; the original parent tests receive only child bridges.

| Declaration | Original bytes | SHA256 |
| --- | --- | --- |
| Two actual-App cases |8094|`c83f51637e745662163ac556af052dd2821684fd3267e279da61f94c94e510f8`|
| Hidden-modal command context |2678|`1ca7ba196640afb710dde6da46d1c39db9d11580e62d93f67a58680a509891f1`|
| Active-modal command supplement |3713|`f1dab6974f8ac07b0469c2993723124886e8c1e30a0393cd388744398371c72d`|

Command11533 actually fails0/1: a real hidden Atom Open→exact Cancel leaves the old
selection proof current. App27637 actually fails0/2: the moved preview survives that
hidden Atom cycle, and a width0 frame followed by identical restored surface revives it.
The latter proves the SDK received zero width and that the lost frame has no canvas.
Both App families first prove a real moved black bond stroke and an ordinary no-loss
commit control; cancellation cannot pass with an unarmed/vacuous gesture. Supplement
66346 actually fails0/1: an active Atom modal admits a fresh selection context. The
Atom-first loops stop there; these initial REDs are not independent executed settings-
branch failures. Settings and the already guarded Preferences control run in GREEN.

SelectionContext now captures existing monotone Atom/settings-open epochs, compares them
without recursive getters and refuses raw active ownership. Exact Cancel permits fresh
proofs but cannot revive old ones. No new serialized field, epoch, command or source
authority is introduced. Add Chain is not a manufactured hidden cycle: Open Add requires
Chain mode, and its necessary tool switch already stales a Selection proof.

One private cancel_canvas_gestures consolidates15 old Ring/Chain/Bond/hover cancellation
blocks plus the missing-graph guard and additionally resets disposable SelectionWorkflow.
Generic invalidate_hover remains hover-only. The existing chain lose_surface call remains
at its old boundary, preserving count-repeat lifetime policy. Persistent selection,
document/source/history, selection epoch and tool stay untouched by cancellation.

Unchanged declarations now pass command46899 **4/0** (both old+both new cases) and App19297
**14/0** (all12 old+2 new cases). They require original ink/no gesture before stale release,
exact whole state/selection/CaptureStamp/modelArc/file bytes through refusal and successful
fresh recovery. No old vector, assertion, deadline, consumer, geometry or tolerance changes.
These are public command and real-App synthetic inputs, not native race/DPI execution.

## Aggregate and bounded delta review

Fresh full local `cargo xtask ci`36217 exits0: **1717 passed/0 failed/1 ignored across249
result lines**. The215726B log SHA256 is
`ba1eab4167faebed304afd0ecc67593970911abc6be01658b3a0979d3db83338`.
Formatting, warnings-denied workspace/vendor lint/tests, layers, seven assets, unchanged
strict155/166/221 runtime license closures and pinned corpus pass. The old offscreen
artifact case stays ignored. No unchanged failed gate or exhausted socket diagnostic is
repeated; the passing natural MCP test does not explain the historical AddrInUse failure.

Independent review verifies the two frozen production before/after pairs and finds no
material issue. Report3862B/SHA256
`13db869650f55a938870eb52de2d9a984360fd25bd583e44ecf87ced09312ed7`.
It checks raw ownership/no getter recursion, all16 helper sites, unchanged Chain surface
hook and generic hover invalidation. This is source review, not executed native evidence.

## Final rebuilt native workflow

Build26772 exits0 in4.60s. The separate normal executable is24941856B/SHA256
`0fa9d4ff4e8bc50da033a82c9519dedc66636653c56a6a1531b8b59a64ca20c2`;
the22453296B CLI SHA256 is
`7d0a8915d4df78d359ed9537f4753d1b96f68ee18365fae1ce27852545a16bb7`.
The rebuilt CLI's42370B registry output is byte-exact to the preceding45-command output,
SHA256 `5dbb59f2688083a37d88876c5063326b4874b3e06381b4688b6592d858a23099`.
Controls39/22/16/12 and catalogue16 available/124 planned stay unchanged.

The initial manually combined fixture is valid XML/RDKit but outside the current native
canonical byte profile. Native Open preserves it and explicitly refuses faithful preview/
editing; its663B SHA256 is
`ca3a543dd6da96a45f575fb04531c5bba71012f3ebe10d15af485063551bbff4`.
This is a fixture/source-profile qualification, not a selection failure. The source guard
is not broadened or bypassed. A first consumer setup invocation uses system Python without
RDKit and stops before any fixture write; the pinned oracle interpreter is then used.

The actual rebuilt CLI uses existing structure.paste_smiles(CO.CC,80pt,150/200) and
document.save_cdxml to create a separate canonical single-Fragment fixture. Independent
RDKit confirms four atoms/two bonds, CC.CO/C3H10O before native Open. This headless fixture
does not replace the earlier empty-document native creation workflow. In the rebuilt Mac
app, double-click selects only CO or only CC despite one Fragment; CmdA selects both and
Escape clears. Visible Document Settings and Atom dialogs own arrows/Escape and retain
the selected pair and Saved state. Their cancelled Save As is byte-exact to the input.

A fresh selected CC drag(24,-12)pt works after both modal cancellations. One CmdZ restores
the complete original, CmdShiftZ restores the moved Saved state. Reselect, Left then
Shift+Up adds(-1,-10)pt. Unicode Save/reopen restores clean editable state; selecting CO
and Down moves only that pair by1pt and saves under another fresh Unicode path.
All native output filenames are visibly confirmed before Save; no overwrite is used.

Each final file is663B. Independent Expat/Decimal checks retain every non-position field,
source ID, directed edge/order, literal H and80pt length exactly. RDKit reads the actual
files as one four-atom/two-bond molecule, CC.CO/C3H10O, with two components CO/CC and
literal H3/1/3/3. These are independent consumers, not just our own round trip.

| Final component file | First pair positions, pt | Second pair positions, pt | SHA256 |
| --- | --- | --- | --- |
| canonical / cancelled | (150,200)/(230,200) | (310,200)/(390,200) | `0bf68d8e576598dd9f5f6fa589a9ab62fc85b956672725cb8f2727d7609e1fb1` |
| moved | unchanged | (334,188)/(414,188) | `f805d8a4b4f22a277bdfad07e3f5690d3db31a954d221f54b1fe29eb15e6f4dc` |
| nudged | unchanged | (333,178)/(413,178) | `c0477e4fe87f9c1f298613a2b38f0699205e45e0c73477748cc102e214d441c0` |
| reopened-edit | (150,201)/(230,201) | nudged | `b6e5fdf7662b19f01bbec1d8fa477263c44e020d2dae4389b142cf8ed727c702` |

Filenames are selection-components-<row>-λ.cdxml; cancelled/input are byte-exact. Explicit
Quit and a later separate app inventory observe only this review stopped. The two older
user apps/drawings stay protected. No owned execution job remains.

## Limits and continuation

Hidden modal/surface races are command/App evidence, not actual native held-button race
proof. Native additive Shift-marquee, Shift10pt horizontal arrows, hardware/measured2×/
DPI, Windows/Linux and exact26.1 behavior remain unverified. Lasso, transforms/snapping/
groups and broader objects/source editing remain missing. The newly observed noncanonical
CDXML refusal retains the existing M1 mapping gap; broaden it only with source-preserving
mapping/writing tests, never by weakening the canonical-profile guard alone.
No dependency/font/vendor/lock/corpus/workflow, golden, listener policy or publication
change. No parser/format changes or new fuzz run; the targeted state regressions and
full local gate qualify this slice. Actions stays OFF; full M0–M5 stays active/incomplete.
