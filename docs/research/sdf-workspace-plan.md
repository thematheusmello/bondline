# Next M1 SDF workspace gate

Research/plan only, 2026-11-15, supplied base `9fc6944`. No implementation, Cargo test,
native execution or reference-program inspection occurred. Root owns Git and native input;
desktop availability remains pending. Actions stays disabled. This follows the existing
[preview plan](sdf-preview-plan.md), not a replacement for the full M0-M5 roadmap.

## Public evidence and limits

| Source | Verified evidence; implication |
|---|---|
| [BIOVIA Chemistry2026 CTfile Formats](https://www.3ds.com/assets/invest/2026-01/biovia_chemistry_2026_ctfile_formats.pdf), printed97-99/PDF indices100-102 | Official URL opened; cached1,580,820B/SHA `959f2bbf5487e00fa7535ea7ee36f7ce04f39ea72920844641952276de146c94` rehashed and all three pages rendered/visually read. Records combine a MOL block and optional data; V2000/V3000 may mix. Data identity can be a name, DT number or both, with additional header text. Values span lines of at most200 characters, end at a blank line, and a dollar-prefix delimiter ends the record. This specifies file semantics, not a property-editor screen. |
| [GUIDE25](https://chemistry.beloit.edu/classes/programs/ChemDraw_25_manual.pdf), printed256-258 | Web fetch refused the17,721,447B size; historical public cache rehashed to `3f1ebc429df07efdd47b6218a148dc346a215043f60bf89ee9a1cb90629bd5b3`. Pages were rendered/visually read. The Windows Excel add-in imports all records into structure/data columns, provides progress/cancellation and exports selected table data. These are **Excel workflows**, not proof of a Professional desktop field panel, its preservation rules or26.1 defaults. |
| [Official May2026 capability chart](https://assets.revvitysignals.com/asset/0b9656e8-339d-47ab-9834-6c08beb6abee/Full-Dot-Chart-ChemDraw-Suite-of-Products.pdf), PDF page10/printed9 | URL opened; cached862,778B/SHA `3a90af112d7f25623decddb3d28bc68229ffedb12be51a5c92208e149d8100e3` rehashed and the tier columns visually read. SDF read/save is marked for Professional; SDF property viewing is marked only for Signals. This is advertised/version-qualified capability, not an observed flow or field-edit contract. |

Public web searches also found cloud Collection property mapping/edit/export. Those are
cloud/tier-qualified workflows, not desktop parity evidence or an offline dependency.
The historical installed observation in `sources.md` remains historical only: it reports
both molecules visible, but establishes neither field retention nor a record browser.
No new binary/runtime evidence is used here. No source prose, example diagram or asset
is transplanted. Existing source-register IDs CTFILE-2026/GUIDE-25 and the chart audit
remain authoritative; Root should add this page/tier qualification when integrating.

## First finish actual native preview acceptance

The real path is `BondlineApp::update_workspace` -> `PreviewWorkflow` -> registered
`render.capture`, not the public no-executor summary helper. Current headless tests prove
real upload/paint instructions and controlled retirement; they do not prove OS pixels.
The preview remains read-only, fitted, and independent of `ui.view`; SDF pan/current-view
capture is not delivered. Logical main-em8..256 and uniform2048px cap stay unchanged.

Use ignored copies of these unchanged original MIT OR Apache-2.0 inputs:

| Input | Literal expected result |
|---|---|
| `io/tests/fixtures/sdf-preview/valence-absent-mixed.sdf`,740B, SHA `e5cbdc14329c59bad55148c5de37058de90f651af10966c30cf8b89b3b5efd39` | Two records, V3000 CCO then V2000 CN;5atoms/3bonds total, one NOTE occurrence each. Display H3/2/1 and3/2 while authored H stays None. Derived positions `(72,108),(108,108),(108,72)` then `(108,216),(72,180)`pt. Both row regions and faithful OH/NH2 attachment must appear; H may be placed left/above as required by existing label policy. No source rotation/mirroring. |
| `io/tests/fixtures/sdf/mixed-v2000-v3000.sdf`,777B, SHA `165e6f5d5cff37dae6315a07843d84733aa42dbbc497b2920da1702bad9526a0` | Two ethane records, V3000/V2000, field counts6/1. Retain EMPTY, two separate DUP occurrences, combined MASS/DT0009, DT0012-only and three spaces, plus second-record ORDINARY. Two drawings; a property dictionary cannot certify retention. |
| `io/tests/fixtures/sdf/ordered-fields-v2000.sdf`,690B, SHA `f95838cbb55c107c649ce2eb99ac62e191f747ce6994a171e64019f8e6c53d1a` | Two V2000 ethanes, field counts7/0; empty second title, original delimiter suffix/padding and DT0009/duplicate/space values retained. This fixture alone does not establish ordinary unspecified-H chemistry. |

Paths above are relative to `crates/`. Before input, Root checks availability, exact owned
bundle/PID/window, fresh input ownership and source/binary hashes. Preserve all existing
user windows/files/cursor; stop injection on interference. Use a fresh token/session and
never print credentials. Do not close pre-existing preview applications.

1. Through the actual chooser Open740B. Record native window/logical content area/DPI and
   actual preview bounds, source revision/generation, labels/count summary and eventual
   own-window image. Both molecular regions must be visible and readable. Start at the
   actual available window size; enlarge only the view if the readability guard refuses.
   Never shrink the chemical page, suppress H or lower the guard to obtain success.
   Tie screenshot regions to literal geometry and the existing independently checked
   registered PNG; a generic nonblank screenshot is insufficient. No new golden approval
   is implied. Expanding Records must expose both versions/counts without authoring.
2. Drag/click the preview and try drawing/order tools: parent stays `document:null`, with
   no graph edit, selection authority, history/revision/dirty or source change. Native
   field edit is not yet a UI feature; controller-driven edits must be labelled as such.
3. Resize across a readable/unreadable boundary. Old pixels must clear while a new key is
   pending; a current small-area error is explicit. Restore size and Retry. Actual DPI
   changes require an available real display/scale route; simulated egui DPI is separate.
4. Open syntax-valid but unprojectable later-record data from an ignored original derived
   vector (retain first record, add isotope metadata to the second using the existing
   negative pattern). Expect a whole-preview error naming record2, no retained old first-
   record picture and exact source Save. A syntactically malformed later record instead
   fails Open atomically and preserves the prior session; do not confuse these outcomes.
5. Open777B/690B, use known Save and Unicode `.sdf`/`.sd` Save As/reopen, and independently
   verify **every saved byte**, raw ordered fields and each contained MOL. Use unchanged
   `io/tests/{sdf_oracle,mol_oracle,sdf_mixed_oracle}.py` only for their actual subsets.
   The full RDKit supplier loses DT-only data in the original fixture: never replace raw
   checks with its last-value properties. Reopen must retain SDF kind and both records.
6. Replace SDF with Drawing/SMILES and reopen identical SDF bytes. Old preview/error/hits
   must disappear on source identity/generation changes; later fresh output may reappear.
   Field-only set/Undo/Redo can additionally use the controller, recorded separately from
   native field UI. An unchanged image after metadata edit is not proof the old job was
   safely retired; source/key/lifecycle evidence is separate.
7. Exercise Cancel/Retry and close Request/Cancel while work is **actually observed in
   flight**, then verify no late image or premature final close. Tiny fixtures may finish
   before Cancel is reachable: record that route unverified rather than infer success.
   Existing channel-gated real-App tests remain decisive controlled decode/publication/
   thread-retirement evidence. Any future native worker gate needs its own reviewed test
   build/provenance; do not add artificial delays or claim spinner disappearance proves
   JoinHandle retirement. Close only owned sessions and record actual cleanup handles.

Record success/failure/unverified per route with source/binary/input/output hashes and
unmodified own-window captures. Controller, offscreen CPU/GPU, egui-frame and native
observations stay separate. Windows/Linux are unverified until actually executed;
Actions stays off. A native failure becomes the next exact TDD regression, not a waived gate.

## Smallest complete implementation slice after that gate

Deliver an **original occurrence-based record inspector and existing-value editor** over
the two existing registry commands. This advances a useful offline SDF workflow; public
sources above do not establish that exact Professional26.1 UI. Keep the all-record preview
and full source owner, rather than replacing them with a selected-record Drawing.

- Select a source-local record by ordinal; show its version and ordered field occurrences,
  literal header/name/DT digits, empty/multiline/space values and counts. Reuse
  `sdf.record.inspect` pagination:1..64 fields,1MiB result object, per-value64KiB raw/JSON
  disclosure limits. `value_lines:null` with an omission reason is not empty data and must
  not become an editable blank. Do not parse title/MOL/data again in the UI.
- Edit one existing occurrence in a bounded multiline draft, with explicit Apply/Cancel.
  Apply calls `sdf.field.set_value`; one Apply is one history action. Preserve spaces;
  empty editor maps to `[]`. Reject internal blank lines/newline/control/dollar-prefix,
  non-ASCII or over200-byte lines exactly as the existing codec does. Preserve both64KiB
  aggregate input checks. Changed values use LF; unchanged logical values preserve the
  exact old EOL/source Arc. Do not silently trim, wrap or rename values/headers.
- Record/field/page selection and drafts are transient, source-scoped state. Use the full
  CaptureStamp plus selected ordinal/occurrence, request nonce and close epoch to reject
  historical query completions. Open of identical bytes is still a new generation. Own
  field Apply may retain the ordinal selection, but must refresh inspection/preview keys.
  A stale Apply reports conflict and preserves the draft; it must not overwrite new data.
- Reuse `CommandExecutor` registered work, worker budget, cancellation, publication and
  UI close barriers. `prepare_command` is currently crate-private: add only the necessary
  bounded UI preparation bridge for these real typed operations, reserving under the
  Engine mutex **before** spawn, as `prepare_ui_capture` already does. Do not call
  `execute_blocking` while painting or spawn unregistered work. Keep a pending UI intent
  busy through preparation, result delivery and actual thread retirement; decode/field
  payload work stays off-lock. Queries/edit jobs acquire no Save lease; Save still does.
- A local unapplied draft must not vanish on record switch/Open/native close. Choose and
  test an explicit Apply/Discard/Cancel draft transaction integrated with the existing
  close/file barrier; do not mark it saved or treat it as Engine history before Apply.
  Give selector, ordered rows, pagination, editor and actions stable accessible IDs/names,
  keyboard focus and errors. Data actions remain the shared commands; any new transient
  registered UI intent needs its own closed parameters/discovery, not a shadow mutator.

No new format crate, chemistry inference, generic job framework or record writer is
needed. Integration points: new `ui/src/sdf_workspace.rs` plus actual-frame tests;
`ui/src/lib.rs`/`preview.rs` and FileWorkflow close gating; the minimal MCP work bridge;
existing `commands/src/{sdf_query,sdf_edit}.rs` only if a demonstrated contract gap needs
repair. Keep source codecs, old fixtures/oracles/font/golden and locks unchanged.

## First executable TDD acceptance and remaining work

First add a real `BondlineApp::update_workspace` test, then observe the missing-UI RED:
Open the fixed777B source, select record1's **second DUP at zero-based field_index2**,
inspect first/second values separately, edit to `["reviewed","line two"]`, Apply, Save
through actual FileWorkflow, drain/retire, and reopen. Independent literal expected output
is `input[..378] + b"reviewed\nline two\n" + input[385..]`:788B/SHA
`e63cb9adf5cbc62b5b0be94992246c0d10b94cda405e49b9514993e7d9f9c361`.
This plan's ignored stdlib byte-splice receipt predates implementation; no Bondline writer
or supplier generated the expectation. Require unchanged first DUP, all other headers/
values/MOL/version/record bytes, the same chemical image at matching view geometry, one commit, dirty/save baseline
truth, Undo777B/Redo788B and independent raw/per-MOL consumers. Test the same workflow in
the actual native UI after the shared implementation passes; a controller Apply is not UI.

Then cover empty/three-spaces/DT-only/CRLF no-op, omission/pagination, over-budget values,
cancel/error/retry, external edit and identical-byte Open during query/edit, same-packet
Apply/Save/history ordering, draft navigation/close, worker-busy and post-publication
retirement. Preserve existing historical query and trusted close-Save semantics. Preview
IDs never authorize chemistry edits. Use relevant scoped tests/clippy/fmt, then Root local
CI; do not create an artificial RED for documentation or broaden hosted execution.

Still missing afterward: field add/remove/rename, record add/delete/reorder and stable
identity design across those edits, selected-record/export subsets, search/sort/filter,
spreadsheet interchange, full chemical editing/unsupported metadata/large-file workflows,
SDF navigation and complete26.1 screen/flow evidence. BIOVIA does not make ordinal IDs
persistent; do not promise identity across reorder. Broader M1 and M2-M5 remain required.
New source-register/parity/STATUS updates belong to Root integration; this plan marks none
of these workflows implemented or verified.
