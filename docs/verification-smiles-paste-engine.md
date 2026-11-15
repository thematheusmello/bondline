# Editable SMILES paste: engine and headless adapters

2026-11-15. This follows the [authoring source contract](research/aromatic-authoring-contract.md)
and [verified chemistry](verification-smiles-authoring-chem.md). It appends real native
Drawing content, rather than creating a conversion file or replacing a source-file session.
Desktop input/clipboard execution is a separate, still-open gate.

## Independent declarations and first execution

The original575-byte command declaration reaches UnknownCommand before implementation.
The expanded six tests reach0/6 at missing registry/command behavior. New-test setup
corrections are retained in ignored receipts: six required fields rather than a draft
count typo; existing complete_with_cancel API; public SMILES Open instead of a private
constructor; its post-Open revision and existing unsupported_content kind-error code.
No production API/error policy was changed to accommodate those mistakes.

The independent layout declaration15373B SHA256
`bd7a1a92e231176ce234c0017a3b2a9f644793bcb88fb62d7991ffd70b0cd4fd`
preserves its initial1311 bytes. It fails with exactly seven missing layout_authoring
E0599 errors before implementation. Six cases then pass on the first candidate, including
literal benzene/fixed-Kekulé/pyridine/pyrrole/backtracking graphs, analytic regular5/6-ring
geometry, source ownership, complete benzene+ethane, actual independent graph consumers
and full CPU/GPU page equality. Unsupported fused/isotope layout refuses the whole input.
Old one-component outputs and all four old converter refusals remain exact.

Root94719 exits0:6 layout+6 command passes. Command evidence covers append into nonempty
native-backed content, original fragment Arc retention, exact global IDs/directions/H,
one-step Undo/Redo, edit, Unicode Save/reopen, nonrecycling floors, source/kind guards,
overlap/paper/bounds, cancellation, stale/foreign candidates and close Request→Cancel ABA.
Notation is absent from Debug and automatic state; the schema has exactly six required
closed fields. Work completes off-lock and finishes once against captured source identity.

The independent wire declaration17656B SHA256
`0a35b43906f296edf065313d5668c4ba5918614ab35064836f31cd10211ec501`
retains its first1632 bytes. Before registry edits, the actual existing CLI with its exact
initial request exits2/unknown_command. That was a direct executable check; the Rust wire
test itself was not yet run while the IO source owner was editing.
Root62848 then executes all three new Rust wire tests successfully: actual CLI, initialized
stdio MCP and authenticated loopback append/edit/history/Unicode Save/reopen. Independent
Expat/RDKit reads the actual saved XML, checks the complete inspected graph and feeds its
pasted Fragment into the unchanged qualified authoring consumer: C6H6→C6H8. Complete
opaque multi-record SMILES sessions remain source-exact and refuse drawing paste.

## Implemented boundary

`structure.paste_smiles` is the single31st command. It requires expected_revision, page_id,
notation, bond_length_pt and origin_x_pt/origin_y_pt. At most64KiB is parsed as one complete
notation; names/additional records are not trimmed or silently selected. Initial chemistry
caps are64 atoms/128 bonds. Layout reuses bounded tree,3–8 S/D simple-ring and source-order
component-shelf geometry. Fixed Triple/cumulene ring geometry and unavailable topology
refuse; attached-tree guards and old converter policies remain separate.

A complete source-free or byte-proved native CDXML Drawing is required. Deferred work
retains its complete owner/baseline, validates layout, scans at most100000 document identity
items, remaps checked IDs and appends one complete Fragment. Floors advance only after
successful commit. No file publication, Save lease or trusted Save acknowledgment occurs.

The actual whole target page is certified before commit: present/on-paper atom positions,
global-ID geometry and all-fragment literal glyph/stroke ink. A temporary dense graph is
only a geometry view; original IDs locate actual scene strokes. Unknown old H stays None.
The page diagonal scales the existing epsilon. An exact analytic geometry-comparison
reservation and subsequent stroke pairs share100000 checks; exhaustion is resource_limit.
At most4096 atoms+bonds are admitted by the existing hit-region bound. This is a bounded
authoring certificate at1024 square, not a promise that every zoom renders successfully.

Source-only review finds no material defect in the candidate, source/history/cancel/ID
guards or adapter wiring. This is separate from execution. Logs and declaration receipts
remain under ignored target/aromatic-authoring/. No new parser, external dependency/version,
font, old fixture/oracle, corpus pin, proprietary asset, SDK or golden changes here.

## Local aggregate

First gate62848 stops83/1 at the old CLI exact catalogue;62137 stops182/1 at the old
command-library catalogue. Both omitted the intentional new command. All eight exact
catalogue sets now add only that name; old count assertions advance30→31. No assertions,
prior names, deadlines, concurrency or socket policy were removed or relaxed.

Corrected receiving71552 exits0: **1229 passed/0 failed/1 ignored**,179 result lines.
Rustfmt, warnings-denied workspace/vendor clippy, all workspace/vendor tests, layers,
licenses and corpus pass. Log149898B SHA256
`6181effc288002b4eadced066f640932053b01fdba50de23332ec5e77aab54cb`.
Runtime license closures are155/166/221 packages for Mac/Windows/Linux plus2 assets;
the pinned corpus remains8 files/5 bundles, with its broader chemistry/CDX omissions.
Rust1.94.1, incremental0/jobs2/dev+test debug0 and the literal pinned Python interpreter.
The unchanged MCP replacement-bind test passes here; this does not explain or repair the
earlier historical AddrInUse failures. Actions stays disabled. No new Windows/Linux or
desktop clipboard execution is claimed by this engine checkpoint.
