# Existing-bond order edit: next bounded M2 gate

Research reviewed 2026-11-14 against Root checkpoint
`13222902f8360d1859a91d5875eea6f9298e9d27`. The older isolated worktree was not reset.
Root/other owners may already be implementing this gate; this record is a sourced plan,
not an implementation or native-verification report. Full M0-M5 remains active.
No reference program, asset library, binary or installer was inspected or downloaded.

## Sources and version boundary

- [GUIDE-25](https://chemistry.beloit.edu/classes/programs/ChemDraw_25_manual.pdf): public
  vendor-authored 2025 guide, 17,721,447 bytes, SHA-256
  `3f1ebc429df07efdd47b6218a148dc346a215043f60bf89ee9a1cb90629bd5b3`.
  Printed page N is one-based PDF page N+1. Relevant text:51,67,72-74,82-83,85-86,88,99.
  Eight pages were rendered and visually reviewed; exact images/text stay ignored under
  `target/bond-order-research/`. This is version25 evidence, not 26.1/build/platform parity.
- [Official support article](https://support.revvitysignals.com/hc/en-us/articles/44882776753300-ChemDraw-Double-clicking-on-an-Atom-adds-a-new-bond-instead-of-the-text-box-to-enter-the-label),
  updated 2026-01-03: atom-label entry changed to Alt+single-click from25.5.0.5789.
  That concerns atoms; it does not define bond-body modifiers or prove a 26.1 click matrix.
- [Pinned OpenSMILES](https://github.com/opensmiles/OpenSMILES/blob/9240a2887158699503605bc838e1de9a9f7883f4/opensmiles.asciidoc#L275-L312)
  defines neutral organic-subset carbon valence4. Existing BOND-EXTEND-AUTHORED-H sources
  in [the source registry](../clean-room/sources.md) distinguish literal from absent H.
  These justify an explicit bounded arithmetic policy, not general valence perception.

Bounded official help/support searches found no complete 26.1 old-order/tool/modifier
matrix. This is a research gap, not proof that such documentation does not exist.
No PhotoCraft source pattern is needed: Bondline already has one typed registry,
`replace_fragment`, complete-snapshot history and generic CLI/MCP/control dispatch.

## Public behavior, kept distinct

| Evidence | Documented functional meaning |
|---|---|
| GUIDE25 p85 | Single tool: click Single -> Double; further clicks reposition the Double components. Drawing over a Single is another Double route. No general1->2->3 click cycle is described. |
| p85-86 | Hover a bond with a bond/selection tool:2 sets Double,3 sets Triple. Triple also has a dedicated tool and a Solid/Dashed/Bold endpoint-to-endpoint drag across an existing Double. |
| p72-73 | Bond-hotspot1/2/3 select absolute Single/Double/Triple. l/c/r address Double placement separately. |
| p88 | Requested bond tool plus existing-bond click is a type-conversion route; the paragraph starts with a Single. Double component styles have separate rules. Matching-style center clicks and a position context menu change Double alignment. Three alignments are shown; their click traversal order is not given. |
| p67,74; p51 | Hotspot operations differ from generic tool switching. x chooses Solid away from hotspots; atom numeric keys have other actions. Hotkeys are configurable and case-sensitive; order, display and Double position are separate command concepts. |
| p82-83, visually checked together | Shift-click orientation has a continued qualifier: saturated double-bond ring tools. It is not evidence for generic bond-body Shift-click conversion. |
| p99 | Automatic H rectification is a separate preference; the stated ethane->ethylene example changes terminal CH3 to CH2. Atom-label alignment and terminal/implicit-H display are separate settings. Defaults and explicit-override precedence are not settled here. |

## Conservative first click map

For the app's supported plain carbon bonds, use this deliberately incomplete matrix.
The old-order/tool combinations absent below remain visible capability gaps.

| Selected tool | Existing Single | Existing Double | Existing Triple |
|---|---|---|---|
| Single | Set Double: direct p85 evidence | Alignment intent/gap; preserve data with a visible unsupported diagnostic | Unspecified: refuse |
| Double | Set Double: general p88 conversion procedure plus Double tool role | Matching-style alignment gap; refuse rather than claim a successful no-op | Unspecified: refuse |
| Triple | Set Triple: general p88 conversion procedure plus Triple tool role | Unspecified click outcome: refuse; hover3 or documented endpoint drag is the supported order route | Unspecified: refuse |

Double/Triple-on-Single follows the general requested-tool conversion procedure; it is
not a separate observed example or an exhaustive old-order/tool table. Do not generalize
it to all combinations. The Double alignment refusals are honest original scope guards,
not the reference's final behavior. Direct `bond.set_order(double)` or hotspot2 on an
already valid Double may be a no-op: an absolute setter has different intent from a click
that requests missing placement behavior. Use the existing unsupported-content error
contract, meaningful wording and exact state retention; add no successful alignment stub.
The documented endpoint drag is a later explicit UI slice if not included now.

## Shared operation and closed H policy

Root's agreed command is `bond.set_order` with closed required parameters:
`expected_revision`, positive `page_id`, `fragment_id`, `bond_id`, `order` as existing
`single|double|triple` strings and `hydrogen_policy=rectify_neutral_carbon`. No implicit
UI-tool default. This required policy is an original bounded authoring choice, not proof
of a reference preference's default state.

Reuse `commands/src/bond.rs`/`lib.rs`, the complete Drawing owner and atomic
`Document::replace_fragment`; no chemistry/style crate or model field is required.
Validate the exact target and source-free/byte-proven native CDXML authority. Imported
MOL/CDX and opaque XML remain guarded; an associated MOL filename does not change an
otherwise source-free document's authority. Reject SDF/SMILES content, stale revisions,
unsupported metadata/orders and missing targets before mutation.

Within the closed neutral positioned-carbon/Single-Double-Triple fragment profile, prove
for each affected endpoint that literal H plus all incident bond weights equals4. None,
contradictory literal overrides and overvalent results refuse. For old weight a and new
weight b, author `new_H=old_H+a-b` at both endpoints after checked nonnegative bounds;
retain every unrelated atom/count, coordinate, ID, endpoint direction, fragment and Arc.
This changes the authored counts under explicit permission; it never mutates inferred
source H or hides counts in the renderer. No new IDs/floors are consumed. A valid same-order
request preserves Arc/revision/history/redo/dirty/source/close state; validate before that
no-op. A real change commits exactly one complete-snapshot history step.

## UI routing and independent gates

Original safety policy, not guide event-level evidence: one unmodified, nonrepeat numeric
key press; no editing dialog/text focus; a fresh ordered visible bond hit; existing atom/
label priority, foreground, viewport, DPI, tool epoch and close guards. Do not turn digits
into generic tool selection or reuse stale idle hover positions. Mouse conversion requires
a same-logical-bond press/release within the current click threshold. Clicking an atom,
label or empty paper must not become an order edit. A held older gesture must not survive
a revision-changing order edit; reacquire scene/target/revision for later ordered events.
All routes invoke the one registry; UI mode stays transient and unchanged by an absolute
setter. The renderer/hits already support one logical bond with1/2/3 components.

Unexecuted acceptance requirements for this research owner:

1. Runtime missing-command/schema RED, then all old26 names plus the new closed command.
   Literal C2 H[3,3]->[2,2]->[1,1]->[3,3], stable identities/positions and one step per edit;
   valid no-op and stale/unknown-source/None/invalid-literal/overvalence failures are exact.
2. Original C3 graph tests including internal endpoints, plus unaffected fragments/Arcs,
   global-ID ceilings, saved/source distinctions and Undo/Redo/branch/reopen. Inspect actual
   saved CDXML with Expat/Decimal and saved MOL with unchanged raw/RDKit consumers: C2H6,
   C2H4,C2H2 and deliberate negative controls. Do not reuse a fixed-ethane-only assertion
   for changed chemistry or weaken old oracles/goldens to obtain a pass.
3. Actual egui frames for the bounded click matrix and hotspot1/2/3, same-order no-op,
   disabled/focus/modifier/repeat/stale/foreground cases, same-frame event ordering and
   one logical component hit. Keep approved Single pixels, glyph/readability/resource
   guards and all old attachment tests. CLI/stdio/authenticated control must edit and
   Save/reopen through the same registry; actual native execution remains a separate gate.

Remaining questions: complete26.1 click matrix; Triple click repetitions; exact alignment
cycle/order and style provenance; platform-specific modifiers; key repeat/release policy;
text-focus precedence; H-override/preference interaction; generic/custom hotkey behavior.
Alignment, styles, Eraser reduction, context menus, properties dialogs, rings and source-
faithful imported editing stay in the full roadmap. No functionality is verified by this plan.
