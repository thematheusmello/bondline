# New-document workspace verification

This original File → New workflow starts at `69e4b37` and uses Root's final
five-file shared backend. The central **New Document** binding and primary-N
shortcut come from the existing public menu/shortcut inventory
([MF-001/KM-001](parity-checklist.md)). The single-session replacement dialog,
A4 defaults and reset policy are explicit Bondline choices, not complete
Professional26.1 window or stationery parity.

## Qualified replacement and Save

The UI uses the central `ui.file` New request, not a replacement Engine or a
close request. The same monotonic request ID qualifies its **New document**
modal and **Save / Discard Changes / Cancel** actions. Clean New and explicit
Discard use `Engine::create_ui_document`, which checks the accepted request
before invoking the shared `document.new` command. A changed SDF field draft
first resolves through its existing Apply/Discard/Cancel route. Apply then
opens a fresh dirty-document decision; unapplied text never silently disappears.

Save retires the accepted New intent and creates the ordinary qualified Save
intent under the same Engine guard. Its source/revision/close context and exact
Save request ID remain in one New-saving phase. The existing chooser, FileJob,
worker budget, actual file command, save lease and trusted completion remain
authoritative. Only the current successful receipt, a successful
`finish_ui_file` acknowledgment, unchanged source/revision/close/request context,
and an exact clean path/format match may invoke `document.new` without discard.
No file acknowledgment is fabricated for New itself.

New-saving additionally retains its actual thread handle. A published result
does not release its owner: the UI waits for `is_finished`, joins, then consumes
the truthful result and acknowledgment. Cancel, stale source or close disarms
the New continuation while preserving that Save/result owner until retirement.
A failed or cancelled Save leaves the document intact. An explicit ordinary
retry may save successfully, but cannot resume the abandoned New action.
Engine-poison cleanup can disarm, cancel and retire this handle without taking
the unavailable Engine lock; it cannot authorize replacement or manufacture an
acknowledgment. Existing ordinary FileJob fixtures and policy remain unchanged.

Modal actions replay at their raw input index. Drawing → New reacquires the
current document/camera/graph before later permissible actions. Synchronous
collection → New drops remaining old-surface authoring events and repaints the
Drawing before accepting a fresh gesture. The publication follow-up below
extends this to local Save completion and shared-command New between frames.

## Observed test-first evidence

- Independent App44995 compiled and executed real Open777→File, then failed at
  the absent **New Document** role. Owner51361 observed four baseline failures
  against the original UI: missing role/binding or no fresh document. No future
  Rust API stood in for that behavioral RED.
- Own63359 exited101 with eight intended missing `choose_new`/`NewChoice`
  compiler errors. Its two initial private tests were3,443 bytes/SHA256
  `4b6eba69541e469732f70c958187ff4833db86eefbcea1d6f0489642fdc9e6ce`.
  Actual implementation86842 then passed those two.
- First App41173 passed two and failed two. A probe showed the central New
  binding enabled with no file/close/job conflict, while the first popup frame
  was egui's invisible disabled sizing pass. The second failure matched the
  existing clean workspace title **New document**, rather than the modal.
  Root approved only a bounded enabled-node wait and a modal-specific negative
  with absence of its actionable Discard button. Production was not changed.
- Corrected App59433 passed three; its intentional dirty-source replacement
  setup then correctly received `UnsavedChanges`. Root approved adding explicit
  `discard_unsaved:true` to those two setup Open calls, retaining every source,
  revision, state and no-retargeting assertion. App41069 passed all four after
  those corrections, without a product change.

The approved four-case source is22,237 bytes/SHA256
`e5cadc837d32a24689e6893ad3c0d36da2a9d6dd76b3e63b7b4a0a10a682c724`.
It covers clean SDF/SMILES replacement, actual fresh carbon authoring, dirty
Cancel/Discard/Save, Unicode chooser Save/reopen, changed-draft decisions,
Undo→New order, and stale replacement including identical source bytes.
Its real saved files are checked with unchanged independent XML/chemical
consumers; the checks are not only Bondline reader/writer round trips.

The sizing probes25541/37501 retain their exact diagnostic-only sources and
logs. Pinned `vendor/egui/src/menu.rs:189`, `containers/area.rs:397` and
`containers/area.rs:548` explain the initial sizing pass; no vendor source changed.
Removing the temporary prints reconstructed the original four UI production
hashes exactly. The original21,072-byte test, corrected22,097-byte test and all
failed logs remain in ignored receipts, separately from behavioral REDs.

Five additional real publication/retirement tests then passed4395 alongside
the two initial cases. Their held worker performs the actual registered Save,
sends its actual result and waits on a finite channel. They prove New waits
for thread retirement, late cancellation retains the committed Save, close
waits, source replacement does not get retargeted, and poisoned-engine cleanup
does not create New. An external Python XML parser verifies the actual saved
640×480 page before and after replacement. These are post-implementation
additional checks, not five further claimed RED→GREEN cycles.

## Lint correction and current local gates

The first full UI package49532 passed207/0/1. Own clippy95428 and independent
clippy73180 then rejected only an eight-argument completion helper. The narrow
fix passes the existing `&FileJob` and reads the same ID/format/action locally;
no wrapper or suppression was added. The exact inverse reconstructs both
previous production files. Root's final backend library differs from its first
port only by the new command name in its existing cfg-test catalogue.

An approved eighth finite case was frozen before execution: a real registered
Save is cancelled before I/O, preserving the dirty source and original file;
a subsequent ordinary successful retry saves without creating New. Run5772
passed all eight cases. Run8630 passed the same four App cases, clippy94633
exited0 with warnings denied, and package rustfmt exited0. The earlier207/0/1
package result predates the equivalent signature change/eighth test; it is not
reported as an aggregate result for later changes.

```sh
cargo test -p bondline-ui --lib file_workflow::new_document_tests --locked
cargo test -p bondline-ui --lib view_tests::new_document --locked
cargo test -p bondline-ui --locked
cargo clippy -p bondline-ui --all-targets --locked -- -D warnings
cargo fmt -p bondline-ui -- --check
```

Use the owning workspace's absolute target directory, incremental0, dev/test
debug0 and the pinned external `BONDLINE_PYTHON` from [development.md](development.md).
`target/new-document-ui/` holds hashes, source freezes, inverse proofs and logs.
The pre-completion-frame freeze is `ui-source-freeze-v2.json`, SHA256
`6c919c0138303f9d745178adc5194651451635c861fb7f5185b9c79841584229`.

## Publication and last-presented source namespace

Two further independently authored App tests found real product gaps after
the original four cases passed. In53938 (own63641), a pointer packet sampled
on the read-only SDF surface reached a Drawing created by Save completion at
frame start and authored an unintended C–C fragment. The test stops at the
first Drawing frame, including an incorrectly edited one; later idle frames
cannot conceal it. The one-frame poll guard first fixed that case and all five
passed18706, with an unchanged default UI package209/0/1 at59080.

The next case71635 published actual `document.new` under the shared Engine
lock between frames, without a fixture helper or warm frame. The poll-only
guard again allowed the old SDF pointer packet to author an unintended fragment.
Own correctly filtered run reproduced the same line730 failure. Initial94118
used a wrong filter and ran zero tests; that separate log is not RED or GREEN
evidence. The actual between-frame failure is preserved in
`sixth-own-runtime-red.log`.

The final implementation replaces the poll-only guard. Existing DrawingState
retains one optional last-presented session generation. Root's Rust-only
`Engine::session_generation()` advances on New/Open and stays unchanged for
ordinary edits, history, camera and tool changes. This is a same-Engine UI cache
key, not globally unique identity, persistence or command authority. All full
CaptureStamp checks for jobs, source retention and Save remain unchanged.

The readonly viewers record the generation of their laid-out source surface.
Drawing records it after successful scene assembly/painting, including ordered
raw New reacquisition. Invalid/unshown Drawing paths do not claim a new painted
source. When a previously presented namespace differs, old gesture/hover/hotspot
state is cleared and that frame's stale canvas pointer authoring is dropped.
Typed menu/keyboard commands retain their ordered dispatch; global SDK input
is not rewritten. A fresh next-frame Actual/gesture still draws and saves the
expected geometry. There is no new App field, job pool or public command.

All six unchanged App cases passed52121 on the final namespace source. The
28,497-byte source has SHA256
`fd423e4632d6719db5fb6e1b012a493e47990ddb5b64b56047bb3eb7762c66a4`;
the prior26,124-byte five-case file and22,237-byte four-case file are exact
prefixes. Their actual781-byte SDF output and fresh editable Drawing exports
use the unchanged independent consumers. Shared CLI/MCP/loopback routing is
Root's separate transport evidence; these frame tests do not invent a listener
or claim OS input.

Final-source package68304 exposed two unchanged old workflow timeouts under
parallel execution (SDF Unicode reopen and SMILES Unicode Save/reopen). Both
passed once unchanged in isolation. The failed package and isolated logs remain;
contention is not established and no timeout/assertion/source workaround was
introduced. The follow-up default package, final lint results and all exact
source/protected-file identities are recorded in the final ignored handoff.

Unchanged default follow-up38580 passed **210/0/1**:160 library tests,50 integration
tests and the existing single ignored visual capture. It includes all six App
and eight finite New cases. Final warnings-denied all-target UI clippy59191 and
rustfmt exited0. No source, assertion or timeout changed between the failed
parallel package, its two isolated passes and the passing default follow-up;
the cause of those two timeouts remains unestablished. All producer jobs ended.

The final namespace source and Root's exact generation API are frozen in
`target/new-document-ui/namespace-final-source-freeze.json`. Final transfer,
protected-file proofs, source/log hashes and all prior failures are in
`target/new-document-ui/handoff.json`. Existing old fixture/oracle/golden hashes
and all old test bodies remain unchanged; `view_tests.rs` has only the appended
child module and SdfEditor only its two required New intent mappings.

All execution here is headless actual-App/workflow testing on macOS. No native
window, OS input, Actions, Git, installation, dependency, SDK, font, old fixture,
oracle or golden change was made by this producer. Interactive New on macOS,
Windows and Linux remains unverified by this slice. Root owns aggregate/native
verification, commits and the feature/parity records.
