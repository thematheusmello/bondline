# Complete native chain interaction contract

2026-11-17, base f6f1cac. This locks original policies before the new producer and
independent acceptance. The first existing-JSON test29755 fails UnknownCommand at
ui.chain_tool. Full M0–M5 stays active. Follow native-chain-plan.md steps4–6; do not
enable either grey tool on the strength of prerequisites alone.

## Evidence and scope

GUIDE25 printed40–42/104–105 supplies document preferences, drag/count, attachment,
Add Chain, Alt-repeat, Snaking/physical Control, mouse-before-Control release,15°
overall-axis Fixed Angles and the different Fixed Lengths ON/OFF reversal sequences.
IUPAC2008 GR3.2/4.1 supplies horizontal-chain/outward-branch guidance, ordinary120° and
linear Single+Triple/two-Double centers. None specifies our defaults, quantization,
sampling, repeat lifetime or exact26.1/Mac algorithm. Existing pinned public sources
and clean-room boundaries apply; no proprietary program or assets are inspected/copied.
The reviewed13KB proposal is source-only, not execution; its recommendations are
superseded by this lock wherever stated below.

Use existing fixed/path builders, previews, source/H/ID/history rules and physical
renderer. No new model schema, parser, dependency, worker pool, file publication or
Save acknowledgment. The delivered Drawing Settings owner stays separate. A new
private ChainWorkflow owns only gesture, Add form and count-repeat presentation.

## Registry and weak contexts

Append ONE command `ui.chain_tool`, preserving every old42 record/schema/order:

- select: required string kind acyclic|snaking;
- constraints: required booleans fixed_lengths and fixed_angles;
- open_add: required expected_revision, positive page_id and target;
- cancel_add: required expected_revision and positive request_id.

All four are closed tagged objects. Target is closed free{mode,x_pt,y_pt} or
atom{mode,fragment_id,atom_id}; coordinates use existing±100000pt bounds. One effective
Chain{kind} uses the shared checked bond-tool epoch, retaining remembered bond order.
Both constraints initially true as original transient per-Engine policy; they survive
New/Open like remembered tools, never enter authored history. Constraints require an
active Chain mode. Source authority precedes equality/no-op; exact no-ops retain epoch.
Select/constraints work without a native host, as existing tool selection does.
Receipt ui_chain_tool_operation has kind,fixed_lengths,fixed_angles,epoch,changed.

Open Add requires attached host, current revision, exact native/source-free authority,
active Chain mode, valid page/target/H and no File/Atom/settings/Add/close owner. A checked
monotone request ID captures weak source/revision/generation, opaque association,
File/close/view/shared-tool epoch, kind and constraints. Request JSON has id,
input_revision,page_id,target,count,bond_length_pt,angle_degrees,current. Count seed6
including an anchor is ORIGINAL policy; effective values are latched. Stale local drafts
remain copyable with Add disabled. Cancel checks current revision/host/exact ID, not stale
source/mode/view; central committed-close guard remains. Only exact explicit Cancel marks
its fresh private cancellation token. Detach retires without marking; local Cancel can
dismiss an obsolete form. Open/Cancel do not author data or approve Save/close.

Chain context also records Add ownership (none for gestures, its exact ID for the form),
so opening a form invalidates a drag without recursive request inspection. UI facts add
actual logical/physical origin, dimensions, density, layer and page identity. Context
loss is latched; returning values or source/file association ABA cannot revive it.
Capture the monotone Add-open epoch even with no owner, plus existing Atom/settings
open epochs: None→Open→Cancel→None must not revive an older gesture/repeat proof.

## Preferences, fixed geometry and count

Resolve independently present document values; known native absence alone uses original
Bondline presets48pt/120°. Reading those presets never writes Options or dirties a file.
Unknown/inherited/opaque sources keep existing refusals. Changes affect later creation,
not old geometry. Fixed switches are chain-scoped Object > Chain constraints controls.

Let A be press anchor, Q current actual pointer, v=Q-A, w=v/|v|. With angles ON snap
atan2(w) to nearest15°, half ties increasing with normalized1e-12 band; otherwise u=w.
Use projected d=dot(v,u)>0. Let a=sin(theta/2), b=cos(theta/2), n=(-u_y,u_x), phase p=-1
initially (upper-first in y-down). m=max(1,floor((d/L)/a+0.5)), N=m+1 including anchor;
normalize count half-ties within1e-12 upward. Check finite/cap BEFORE integer casts.
N>64 is a visible refusal, not clamped/truncated. Zero displacement remains a click
until drag threshold; an out-and-back drag never becomes Add.

Lengths ON uses center-axis e=L(a*u+p*b*n), r=L(a*u-p*b*n) and alternating e/r edges;
odd m has an off-axis terminal. Preview/counter show that actual geometry. ChainSide=-p.
Lengths OFF forms unit points S_i=(i*a,(i mod2)*p*b), terminal V=S_m, rotates V onto u
and scales by d/|V|. Its terminal is A+d*u: Q unsnapped, orthogonal projection snapped.
This deliberate endpoint-axis distinction fixes C2/parity; do not use d/(m*a) or count
by nearest true endpoint span (nonmonotone for acute angles). Reuse fixed certificates.

ON phase may change only at unchanged count: sign cross(u,w) when snapped; sign
cross(previous valid raw axis,w) unsnapped. First/radial/antipodal and |cross|<=1e-12
retain phase. Previous raw axis is evidence only, never a geometric axis latch. Update
it on every valid move, including count changes. OFF ordinary motion retains phase.
Only an allowed Control+Alt rising edge reverses OFF/angles-ON Acyclic once; duplicate
reports do not retoggle. Other chord/constraint combinations refuse visibly.

## Snaking and modifiers

Seed from actual press modifiers, then process ordered ModifiersChanged at raw indices,
including while modal/ineligible. Never use final-frame modifiers for earlier moves.
Use physical ctrl on every platform; mac_cmd or command without ctrl and Shift are
unsupported. Alt-only is repeat only on a new click; a dragged Alt-click or other mid-drag
Alt refuses. An allowed Control+Alt chord is classified before the next move. A Control-only
MOVE already converted to Snaking cannot be retrospectively reversed when Alt arrives;
that incompatible chord cancels the walk. Control report alone does not freeze a corner.

Acyclic's first Control-only move enters temporary Snaking with its previous valid
concrete path, preserving every point, including OFF's prior variable length. With no
previous dragged path it starts at A. Mouse release while Control held may commit;
earlier Control release cancels. Dedicated Snaking is independent of Control. A later
release cannot undo a winning commit. Escape/focus loss/PointerGone/invalid surface cancels.

Snaking's first edge follows the actual press-to-pointer direction, snapped15° if enabled
(ORIGINAL seed policy, distinct from Acyclic's center axis). Afterwards normalize the
previous ACTUAL edge; candidates turn by ±(pi-theta). Choose greatest positive projection
t toward Q; exact ties prefer trans alternation then positive turn. Keep accepted prefix
and one separately disposable tip. On EVERY pointer/release sample:

1. While Q lies within/equal preferred L/2 of the penultimate accepted vertex AND actual
   pointer motion from the preceding sample has negative projection onto the last actual
   edge, pop the terminal; at most63 pops. A duplicate/no-motion sample cannot pop a short
   converted OFF prefix. Conversion supplies its previous actual pointer explicitly.
   Do this before advancing, never alternate pop/append again.
2. While selected t>=L, append ONE preferred-L step, then recompute headings; at most
   the remaining supported vertices. Equality uses exact represented arithmetic, no
   dimensional/pixel epsilon. Converted older edges are never rescaled.
3. ON keeps only accepted steps. OFF rebuilds T+t*heading when0<t<L, otherwise no tip.
   It never freezes a partial tip because of an inferred turn. Release processes its
   actual position and materializes that tip. A65th point including the tip refuses.

Each sample is atomic on math error; latest invalid candidate is visibly invalid, never
presented/committed as the last success. Final count<2 after a dragged retraction cancels,
not Add. Preserve raw reported samples; identical streams partitioned into different
frames agree. Different OS sampling/coalescing is explicitly not promised equivalent.

## Numeric Add, automatic poses and repeat

Add Chain shows one384-byte bounded Carbon count (including anchor) field, strict trimmed
ASCII decimal2..64, Add/Cancel and exact source context. Field/action-prefix and bounded
deferred-suffix behavior reuse current numeric editors. Native close waits; frame-start
and mid-packet form-open fences prevent later canvas/history/File actions on dismissal.

Automatic free/isolated placement has horizontal center axis and upper-first phase.
One old Triple requires first new Single180° opposite it; linear center takes precedence
over axis snapping. Ordinary one-neighbor Single/Double uses both theta candidates,
ranking outward new tails and then upper/right. Multi-neighbor candidates use largest
angular gaps, descending gap width, then upper/right and stable source IDs. Try bounded
scores after rounding their normalized comparison keys to1e-12 (a total ordering), not
an intransitive pairwise epsilon comparator. Try bounded
candidate sides against actual full-scene guards; if none works refuse without changing
old positions/orders/styles/H. Existing target-local literal C4 proof rejects twoDouble/H0,
unknown H, charge/isotope/radical or unsupported source/stereo. No new stereo perception.

UI asks the engine for numeric candidates, validates each through existing preview and
actual current physical/logical/readability/ink/paper guards, then applies the selected
bounded candidate under the same Engine guard. Exactly one registry authoring command
commits. Only successful Add seeds a count-repeat proof captured POST-commit under that
guard; drag does not seed it. Repeat resolves a fresh target/settings and commits once,
then advances its proof. Without a current proof Alt-click opens Add. Unrelated edits,
history/New/Open/import/tool/constraints/view/File/close/association or canvas/DPI/layer
change disarm repeat. Errors/Cancel retain count only while proof remains current.

## Acceptance before enabling tools

Independent engine/schema/error/no-op/epoch/weak-ABA/history/Save tests; actual CLI/stdio/
authenticated loopback discovery/invocation/wakeup; actual-App literal math, full preview,
drag/count/Add/repeat/Snaking/modifier-order/backtrack/reversal, one-edit history, protected
source/modal/worker/close and1×/2×/fractional origins; Unicode native files read by unchanged
raw/XML/chemistry consumers. Then inspect actual native free/attached/Add/repeat/turn/
settings/history/Save/reopen/refusal/recovery on a separate disposable app, preserving
both user drafts. New chemical goldens require human approval. No26.1 default/hotkey,
hardware/measured-DPI/other-platform/full-style or complete M0–M5 claim follows.
