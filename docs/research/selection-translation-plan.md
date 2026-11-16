# Persistent selection and translation — next M2 contract proposal

2026-11-16. Source-only plan following native atom editor `b4fefaf`. No selection
command, tests, native tool or geometry mutation is implemented by this document.

Implement **persistent one-page atom-set selection + exact translation**, including actual Marquee/click→drag or nudge→Undo/Redo→native CDXML Save/reopen. This is a complete authoring increment, not selection/transform parity: lasso, rotate/resize/distort/clone, groups, snapping, reactions/text, foreign-source geometry and exact26.1/platform behavior remain required.

**Public basis:** pinned GUIDE25 printed88 distinguishes moving one atom from moving a bond's two endpoints;115–116 distinguish single-object click, double-click whole structure, Shift add/remove, Select All and deselection;120 gives drag move and1pt arrows/Shift10pt arrows. GUIDE25 PDF SHA`3f1ebc429df07efdd47b6218a148dc346a215043f60bf89ee9a1cb90629bd5b3`,17,721,447B, [public URL](https://chemistry.beloit.edu/classes/programs/ChemDraw_25_manual.pdf). IUPAC2008 GR1.1/4.3 supports legibility/consistent geometry, not UI gestures or numeric thresholds; cached HTML347,564B/SHA`32d2c2d694059e78b6b6a3ed8c274770377e73058d11c174951e7b4e1ce4f71e`, [primary URL](https://iupac.qmul.ac.uk/drawing/drawing.html). No reference binary/assets/new download or native flow inspected.

## Proposed API/model/history

- Two registry additions (34→36): `ui.selection`, a closed tagged enum with `activate{expected_revision}`, `set{expected_revision,page_id,atom_ids}`, `all{expected_revision,page_id}`, `clear{expected_revision}`, `inspect{expected_revision}`; and `selection.translate{expected_revision,expected_selection_epoch,delta_x_pt,delta_y_pt}`. No new crate/dependency/framework. Root must lock these names/actions before tests.
- Selection lives privately in Engine: sorted unique positive AtomIds on one PageId, full weak source/revision/generation stamp and checked epoch. Cap before collection at existing MAX_GRAPH_ITEMS100000 plus existing request/reply limits. Model IDs are already document-global. No model/history/source heap field or file bytes change for selection. Explicit replies carry selection+operation/epoch/changed metadata; ordinary state can omit it. Authored changed=false and existing MCP marker notification/reply barriers must coexist.
- One effective Selection/Bond tool mode integrates with remembered BondDrawOrder; no independent active booleans. Real `tool.selection`, `edit.select_all`, `edit.deselect` bindings use the tagged actions, not empty authoring targets. Existing bond tool activates bond mode/clears selection, including same remembered order; outside selection mode retain all old behavior. Effective mode/epoch joins captured tool freshness.
- Translate validates revision, complete selection owner+epoch, all IDs/page and finite new Point2 positions within±100000; clone complete StoredDocument, replace only affected same-ID fragments and commit ONCE. Exact arrays/IDs, HNone/0/metadata, directed bonds, topology, page extent and all unselected data remain. No H inference, auto-layout, snapping, scaling, identity-floor allocation, publication or Save/close ACK. Zero/rounded-identical displacement is a fully validated exact Arc/redo/history no-op.
- After own translation retain those IDs and requalify stamp/advance epoch, prechecking overflow atomically. Other authored edits/history/New/Open/import/page changes clear and advance selection; idle/view/pan/DPI do not clear it. That conservative history reset is original policy, not observed reference selection restoration. Old Save may publish truthfully but cannot clean/reassociate the newer state; existing save_slot remains untouched.
- Geometry mutation is source-free/fully mapped native CDXML ONLY: require is_fully_mapped AND source format absent/CDXML, including on zero delta. Never regenerate a partial source projection. MOL/CDX/opaque CDXML/SDF/raw or retained SMILES refuse; selected chemical pixels confer no authority. Cosmetic selection of actual read-only Drawing projections is separable/original and optional for first gate; raw document:null previews cannot yield editable IDs.

## Native boundary / exact first RED

Use existing GraphScene logical hits and PageView inverse physical mapping. Plain atom click selects it; bond click selects endpoints; double-click traverses actual connected component, never blindly the entire Fragment (Fragments may be disconnected). Shift-click toggles. Marquee anchor inclusion/both-endpoint bond selection is an explicit original initial rule—the guide's molecular inclusion/label extents are not precise enough. Overlay remains UI-only; no inert transform handles. A fresh press on an already selected target starts translation; first select, then move avoids guessing unselected-atom Shift-drag precedence. Reuse DrawingState gesture ownership, raw ordered events, complete source/selection/tool/view/close/canvas/DPI context and generation fence. Moves preview without Engine commits; release one command. Escape/focus/modal/context/off-canvas cancellation makes no history; re-read graph after each committed/history event. Text focus/modal/Save controls win over canvas/nudges. Real native acceptance is mandatory after headless gates.

**First proposed runtime RED:** existing Engine JSON only; create pairs(72,72)→(120,72) and(220,72)→(268,72), set atom2 O/H1, freeze complete owner/state, dispatch ui.selection set on page1 IDs[1,2] at actual revision. It must compile and fail UnknownCommand, not a missing future Rust API. No such test is authored/run here. Follow with translate(24,-12): literal(96,60)/(144,60), untouched second pair; one additional history step, same chemistry/IDs/H, Undo/Redo, validated no-op, invalid/stale/ABA/point/whole-source guards. Actual saved XML must pass unchanged independent Expat/Decimal coordinates and literal RDKit CO/CH4O + CC/C2H6. Then actual CLI/stdio/socket, App missing-tool RED and native tool→selection→move/nudge→history→Unicode Save/reopen. No native/render/CI success is claimed by this plan.

Exact current reuse: model/lib.rs123/203, graph.rs9/77/154 and point.rs37; commands/atom.rs17–58, lib.rs967/1011/1064, capture.rs15–54, controls.rs358 and bond_tool.rs31; io/document.rs258/350/402; render/hits.rs9/72; UI DrawingState110+ and its ordered canvas/reacquisition loop. The original detailed rationale and source-pin ledger remain under ignored `target/m2-selection-plan/`; this committed proposal is the durable continuation contract. The received6638-byte recommendation has SHA256 `6a65858028628f704529af67f300f0d67306d8167da7edd4e92b4a574d6cd441`. Open issues: exact26.1 Mac/tool/modifier precedence, groups/salts, molecular rectangle inclusion, shift-drag/repeat policy, partial-graph geometric/stereo interpretation, rotation/resize/pivot and history-selection restoration. Keep all parity rows open beyond the exact delivered subset.


## Locked first implementation contract, 2026-11-16

The proposals above are narrowed into the following explicit original policies for the
first test declarations; complete reference parity stays in the backlog.

- `ui.selection` has exactly activate/set/all/clear/inspect, all with required
  `expected_revision`; set additionally requires positive `page_id` and `atom_ids:Vec<u32>`;
  all requires positive `page_id`. Closed tagged variants, no surplus fields. Headless
  use is supported. Activate/set/all require source-free or completely mapped native
  CDXML Drawing; clear/inspect work across content kinds. All commands retain global
  committed-close refusal; transient mutations use the existing tool close/file-busy
  policy. Existing direct authoring during pending close retains its invalidation rules.
- Selection is optional `ui_selection:{active,epoch,page_id,atom_ids}` in ordinary state,
  omitted until its first actual change. `page_id` is null for an empty set; atom IDs are
  sorted, unique and one-page. Set rejects zero/duplicate/missing IDs and >100000 entries
  before selection publication, accepts arbitrary input order, and allows an empty set
  after validating the page. Nonempty selection retains a private weak CaptureStamp.
- activate/set/all enter the selection tool; clear empties targets without changing tool.
  The only other effective mode is the remembered bond order. Mode changes increment the
  existing checked BondToolState epoch; selecting the same remembered bond order while
  selection is active is a real mode change. No second active-tool flag or new tool epoch.
  `ui_selection_operation:{epoch,changed}` is an explicit transient receipt, with outer
  changed:false. Inspect/no-op receipts do not request a repaint; real selection/mode
  changes do after the existing authenticated publication boundary. No host is required.
- `selection.translate` has exactly required expected_revision, expected_selection_epoch,
  delta_x_pt and delta_y_pt. Revision is checked first. Missing/obsolete selection or
  epoch returns stale_selection; malformed targets/counts/nonfinite deltas return
  invalid_params, absent IDs/page return not_found, unsupported source/absent authored
  position returns unsupported_content. Selection epoch exhaustion has stable code
  ui_selection_epoch_exhausted. Keep existing global close/refusal precedence.
- Translation validates every selected target and finite bounded Point2 result before
  committing. Charge/isotope/radical/H/topology/array order/IDs are preserved, not
  reinterpreted. No chemistry inference, new identity, page resize, source regeneration
  bypass or explicit publication. Zero/rounded-identical motion is fully validated and
  preserves Arc/selection/redo/close/baselines. A changed move uses one existing commit,
  retains the same IDs with fresh source ownership and advances selection epoch once.
- Other successful authored edits/Undo/Redo/Open/New/import clear a nonempty selection
  and advance its checked epoch; no-op/failure does not. Selection tool remains active.
  Precheck overflow before any authored/namespace mutation. Save/view/pan/DPI do not
  clear targets. Their existing independent file/view proofs still cancel stale drags.
- Add exactly three native Drawing controls: tool.selection (Marquee selection),
  edit.select_all (Select All, primary+A), edit.deselect (Deselect, no declared shortcut).
  Existing bond controls are unselected while selection mode is active. SDF/raw SMILES
  default controls stay unchanged. Escape and arrow/Shift-arrow handling is native
  ordered input, respecting text focus, modal, file, close and frame-start fences.
- Native selection starts with atom click, bond-endpoint selection, connected-component
  double-click, Shift toggle, original anchor-based marquee, then a fresh selected-target
  drag or1pt/Shift10pt nudge. Empty click clears. Preview does not author; release commits
  once. Existing five-logical-unit graph picking and PageView mapping are reused.
  Native source/race/DPI/other-platform behavior is verified separately, never inferred
  from headless/App checks. Exact26.1 modifier and whole-word/molecule inclusion remains
  unresolved. No native screen or one-to-one parity claim follows from this contract.

Atomic multi-fragment replacement must validate the whole candidate once, preserving page
order and unrelated shared fragments; repeated whole-document validation per selected
fragment would make large moves quadratic. An additive model batch API gets its own
independent missing-API and atomic-error test before implementation. Native Escape
cancels an active preview while retaining prior selection; without a gesture it clears
targets through the shared command and keeps the selection tool. Text/modal handling
wins, including the dismissal frame. This is original policy, not exact26.1 precedence.
