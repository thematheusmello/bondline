# Native Single-ring integration contract

2026-11-16. Source proposal following checked command ac340db; Root locks the first integration policies below before independent API/App RED. No native ring tool or execution is delivered by this document. Reuse the actual `ring.create` candidate builder for plain free, atom-shared and bond-shared construction, with one effective tool mode and an ordinary rendered candidate preview. No SMILES conversion, temporary file, detached fallback, new worker, widget family, dependency or asset is needed. Command/render/precision tests already sealed are not repeated here.

## Evidence and scope

The pinned [GUIDE25](https://chemistry.beloit.edu/classes/programs/ChemDraw_25_manual.pdf) printed101–103/423–424 supplies small-ring tool roles, orientation dragging, atom/bond attachments and separate spiro/sprout/chair/aromatic workflows (17,721,447 B, SHA256 `3f1ebc429df07efdd47b6218a148dc346a215043f60bf89ee9a1cb90629bd5b3`). [IUPAC2008](https://iupac.qmul.ac.uk/drawing/drawing.html) GR3.3.1–3.4.2 and4.2.4 supplies regular small polygons, preferred poses, opposed fused interiors and opposed spiro bisectors (347,564 B, SHA256 `32d2c2d694059e78b6b6a3ed8c274770377e73058d11c174951e7b4e1ce4f71e`). These facts do not specify a48pt default, pointer threshold, unique pose/tie, modifier precedence or observed26.1 behavior. The policies below are original Bondline choices, proposed for explicit lock.

This slice retains neutral carbon/Single rings3..8 and current source/H/precision/occupancy limits. Aromatic and chair tools, sprout preferences/Alt-sprout, hetero rings, general fused-face solving, larger rings, snapping, styles and ring hotkeys remain required follow-on work. No native/platform/parity or milestone completion follows from this document.

## One tool authority and minimal APIs

Introduce one authoritative private Engine field using the public read-only Rust type `EffectiveTool::{Bond, Selection, Ring{size}}`. Keep the remembered `BondDrawOrder` and existing checked tool epoch; no `ring.active` or independent selection authority. Preserve `Engine::selection()->&SelectionState`: its cached `active` is a compatibility projection updated only by authoritative mode transitions. Consumers deciding effective mode must read the enum. This avoids cloning large selection vectors on every query.

`ui.ring_tool` is one closed command with exactly required `size:u8`3..8, no expected_revision (same transient current-window convention as ui.bond_tool). It returns optional `ui_ring_tool_operation:{size,epoch,changed}`, outer changed:false. Same Ring/size is a fully validated no-op; another size or mode changes the single tool epoch. Choosing the remembered Bond order while Ring is active is a real switch, even when that order did not change. Selection activate/set/all enter Selection; clear retains the current mode; history/Open/New retain the mode but invalidate old gestures through source/revision/generation. Leaving Selection clears targets and prechecks both counters before any state change. Existing source-free/native-CDXML geometry guards, close/file-busy and epoch error codes remain authoritative.

Six central controls: `tool.ring.3`…`.8`, labels Cyclopropane, Cyclobutane, Cyclopentane, Cyclohexane, Cycloheptane, Cyclooctane; menu Tools, no shortcuts, selected iff the effective Ring size matches. Reuse28px painted tool responses, accessible names/toggled state/focus and original procedural polygon glyphs. Do not require full label text inside the64px rail. Controls appear only for source-free or byte-proven native CDXML; retained SMILES remains bond-order-only. Next registry count38; default controls native29 / retained20 / SDF14 / raw SMILES10. Source-capability checks must not globally authorize geometry on MOL/CDX/SMILES projections merely because they can render or report fully mapped.

Suggested Rust-only seams (names may be locked by Root):

```rust
Engine::effective_tool() -> EffectiveTool
Engine::ring_context(page_id: u32) -> Result<RingContext, Error>
RingContext::is_current(&Engine) -> bool
Engine::preview_ring_create(&RingCreate) -> Result<StoredDocument, Error>
```

The preview delegates to the same private candidate builder as `ring.create`, including revision/source/target/H/geometry/maxima rules; it discards provisional maxima and never commits or spends floors. Execution remains the existing registry command. `RingContext` privately captures weak CaptureStamp, Ring size + tool epoch, page, view/close/file epochs and opaque file-context proof; it requires Open, no outstanding ui.file intent/job and the supported source. No serde/raw pointer/source text/strong source lease. Do not reuse SelectionContext, which deliberately requires Selection active. Native canvas/DPI/layer/origin qualifiers remain HoverContext facts.

## Exact proposed click/drag policy

A press uses the fresh actual GraphScene hit with the existing five-logical-unit tolerance and exact PageView inverse. Atom and Bond hits select their corresponding typed placement; any ineligible target reports its typed error. It never becomes Free or targets another ID.

| Hit | Click (never crossed drag threshold) | Fresh drag (threshold crossed once) |
| --- | --- | --- |
| Paper | A=press point, L48. For6, B=A+(0,−48); for3/4/5/7/8, B=A+(48,0). Side positive. Six has the vertical left edge; others the horizontal top edge. | A stays fixed, B=cursor document point, positive side. Orientation/length follow the actual first edge; no snap/rescale. |
| Atom | A=authored atom center, L48. Isolated atom uses the exact Free pose. With incident bonds, compute outward direction below and positive first-edge angle=outward−interior/2. | A stays the authored center, B=cursor document point, positive side. No relocation or detached fallback. |
| Single bond | Keep authored A→B/length/ID. Choose initial side below using at most two same-builder candidates. | Keep A/B fixed. Cursor algebraic cross-product half-plane chooses positive/negative; exact-axis ties retain the last side (or click default before any nonzero side). |

Use the existing SDK logical click-distance threshold and latch whether it was ever crossed, as repaired in selection. Returning to the press point does not create a surprise default click ring. A degenerate dragged first edge reports/refuses; it does not substitute48pt. Plain primary is the supported Ring gesture. In Ring mode, modified primary gestures report unsupported scope and do not fall through to Bond drawing or the Alt-atom form. Alt-sprout remains missing; ordinary global shortcuts and Alt-atom editing in Bond/Selection retain their existing role. This is an original first policy, not measured26.1 precedence.

For an atom with neighbors, use actual incident outgoing **unit** vectors, regardless of their lengths: `u = −normalize(sum(unit(neighbor−A)))`. No valence or H is inferred here; the core builder still validates the target. With `alpha=(n−2)π/n`, positive-side first edge is `theta=atan2(u.y,u.x)−alpha/2`. For the existing analytic six-ring A=(120,168), this yields outward150°, first edge90°, B=(120,216): the independently pinned spiro pose. A terminal atom points the new ring center away from its neighbor.

**Cancelling incident bisector proposal for lock:** choose a perpendicular to the smallest-ID positioned neighbor axis, selecting the normal pointing up; if its y is exactly zero, select right. This preserves old atoms and puts a new center in one deterministic half-plane of an opposed chain. Isolated atoms do not use this tie. A zero/coincident neighbor vector refuses rather than inventing a direction. For near-opposed unit sums, recommend a fixed dimensionless1e-12 zero test solely for default orientation stability; this is an original numeric proposal, not a chemistry/occupancy tolerance or source rule. Root may choose exact-zero instead, but must settle it before expectations are frozen. Explicit drag remains available when the default is ambiguous/crowded.

For a bond click, inspect positioned incident neighbor vectors at its two endpoints, excluding the target edge. If all nonzero signed distances lie on one side, prefer the opposite side; otherwise prefer positive. This is a bounded local rule for ordinary convex ring edges, not cycle/face perception. Build positive/negative candidates with the same target through `preview_ring_create` and ordinary scene guards; choose the preferred one if valid, otherwise the other if valid. Both errors mean no edit. On the pinned six-ring edge1, the interior neighbors are positive, so negative produces the independent fused page. Old partial-H or unusual cycles can make both candidates fail; retain that honest limit, and never weaken masks/occupancy or move old atoms to make it work.

## Preview, ordered release and source safety

Build at most one final held-pointer visual candidate per frame (two for initial bond-side choice), not one per queued PointerMoved. Each complete ordered release separately validates its own candidate before committing; multiple completed gestures in one packet must not be dropped or combined. Cache only by full source/context + exact params/view/canvas/DPI; a stale or failed candidate cannot reuse the last green preview. Paint the complete candidate Page through the same physical density/fractional-origin transform, literal labels, masks, strokes, graph budgets and logical8..256em check as current drawing. The current empty scene can have no labels, so the **new candidate itself** needs that logical-em check. A shared private UI readability helper can serve normal and candidate scenes without adding an IO dependency to UI. No line-only successful preview or raster enlargement.

For native release, require the current same-candidate scene to pass these display guards, then revalidate RingContext and execute ring.create once under the Engine guard. Preview conveys no Save/commit authority. This is a native visibility condition; the headless command retains its locked ability to author bounded geometry outside paper or current display scale. Errors leave source/history/selection/floors exact.

Keep the existing ordered `(event_index,Control(id))` route. Resolve effective mode and command parameters at that boundary after preceding authoring; never eagerly execute tool actions or cache old expected_revision. Release→Tool/Undo/Save commits the earlier ring first; Tool/Undo/Save→old release cancels it. File/history/atom-editor/text/modal policies win, including a frame-start fence through dismissal. Source Open/New after poll and between frames, file association ABA, close/cancel, view/tool-size ABA, canvas/DPI/layer changes, off-canvas movement, PointerGone and focus loss cancel the disposable gesture. No ring-specific Save lease/ACK, worker/close retirement or implicit commit-on-close is added.

Only the enum-selected workflow receives canvas events; reset the other disposable owners on transitions. A small RingWorkflow alongside existing Bond/Selection gesture storage is sufficient; it is not another active-mode flag. Native numeric order shortcuts retain their existing role, never become undocumented ring-size hotkeys; any actual authored event invalidates an armed ring proof. Close clears preview and uses the current complete authored document/normal dirty decision.

## Exact integration map / next evidence

- `commands/bond_tool.rs` BondToolState/select_bond_tool and `selection.rs` replace_selection/deactivate_selection: single mode/epoch authority with borrowed derived compatibility state.
- `commands/ring.rs` create/polygon and `lib.rs` RingCreate/commit: expose the same candidate as a noncommitting Rust preview; keep the already sealed builder/precision tests unchanged.
- `commands/controls.rs` controls: six typed bindings and effective selected values; existing bond selected check `!selection.active` must become mode==Bond so Ring cannot light a Bond control.
- `commands/lib.rs` registry/result and `mcp/src/control.rs` existing transient marker: additive command38 and changed-only publication/repaint, old wire fields omitted when None.
- `ui/lib.rs` bond_tool_buttons/control_command, HoverContext, raw replay, source-generation fence, scene_for/readability, inspector and update_workspace cancel paths; one small `ui/ring.rs` may own disposable parameters/preview. No FileWorkflow action or new dialog.

Root should lock the proposed defaults/ties/modifiers/native visibility condition, then receive independent ui.ring_tool/API and actual-App missing Cyclohexane RED before product UI. Reuse the sealed analytic free/fused/spiro render and precision evidence. New App/native evidence must cover all three real placements, actual preview/cancel and chronology, retained source/history and Unicode Save/reopen/independent consumers. Desktop availability remains a prerequisite for Root-owned actual native input; App or Metal headless results do not substitute. No duplicate test matrix is declared here.

The research worker wrote only ignored target/ring-native-plan/ records, without product/test/dependency/Git/Cargo/consumer/CUA/native execution. Root adopts this durable record after reviewing source at ac340db; the code map and public pins above remain sufficient when disposable receipts are removed.


## Root locks for the first integration

The preceding source facts retain their version/pin scope. These choices are original
Bondline policy, locked before tests; no vendor default or native success is inferred.

- Public Rust `EffectiveTool` has exactly Bond, Selection, Ring{size:u8}; its Engine
  field is private. `Engine::effective_tool()` is a cheap read-only getter. Selection's
  borrowed active cache is derived only from these transitions, never a second tool
  authority. One existing checked tool epoch covers every real mode/size change.
- Register `UiRingToolSelect(SelectRingTool{size:u8})` as ui.ring_tool. Required closed
  size3..8; `RingToolOperation{size,epoch,changed}` is an optional explicit receipt,
  outer changed:false. Normal state omits an absent receipt. Changed-only loopback
  notification follows the existing publication/activity-release boundary. Full
  eligibility validation still precedes same-mode no-op. No native host is required.
- Adopt the six controls/counts,48pt positive-side free/isolated click poses and atom
  unit-neighbor/bisector rules above. Sum norm≤1e-12 is the original cancelling-axis
  rule. Stable smallest-ID neighbor perpendicular chooses up, then right on exact tie.
  Coincident/missing neighbor direction refuses rather than moving data. Explicit drag
  is not replaced by a guessed click/default after threshold crossing.
- Adopt the local-neighbor side preference, preferred/other same-builder+ordinary-
  render candidate validation for bond clicks, and actual cursor half-plane for bond
  drags. This is not a general face solver or proximity-based atom fusion.
- In Ring mode, every modified primary gesture is unsupported in this first slice,
  including Alt; it must not silently open another form or create a detached/bond
  object. Existing global keyboard shortcuts and non-Ring Alt editing remain intact.
  Exact26.1 modifier priority and the missing aromatic/sprout tools stay open.
- Lock Rust seams `Engine::ring_context(page_id)->Result<RingContext,Error>`,
  `RingContext::is_current(&Engine)` and `Engine::preview_ring_create(&RingCreate)`.
  RingContext requires actual Ring mode, supported page/source, Open and no ui.file
  request, with weak source and full listed proofs. Preview itself is read-only,
  host/mode-independent for an explicit request, uses the same validated builder and
  current revision, and refuses committed close before stale revision. It cannot
  spend floors, clear selection, author, save, change baseline or acknowledge close.
- The candidate itself must pass physical and logical readability/paper/ink/resource
  guards before native release. New labels on an empty old page cannot bypass the
  logical-font check. This UI visibility gate does not restrict the headless command's
  documented point-domain ability. No weak mask, raster scaling or silent fallback.
- Preserve ordered release/tool/history/Save behavior, threshold latch, ordinary
  off-canvas cancellation and every source/modal/frame-start/DPI fence. Prove these
  through frozen independent command/context/App/actual-adapter tests, then execute
  real native free/fused/spiro/history/Unicode persistence when desktop is available.

The received12899-byte proposal has SHA256
`7d61308eb38d161d176ecf98db75b3947a7ef10bda0aacb2bcfdd12ac37d1d7a`.
No product/API/App tests or native input is performed by this contract commit.
