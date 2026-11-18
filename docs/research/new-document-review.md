# New-document safety review

Bounded read-only review from `69e4b37`, using exact captured inputs. The independent
command handoff `5559d24a…` remains frozen; no source, old test, oracle or dependency
was modified. No test or native execution is attributed to this review.

## Core and presentation: no material finding

- Final `new_document.rs:25–43` checks expected revision, Open close state, explicit dirty-data
  permission and checked revision/generation/view increments before replacing state.
  `44–60` installs distinct current/saved source-free snapshots, resets association,
  history/ID floors, Fit camera and derived extent/focus. Native host, save reservation,
  selected tool and actual canvas dimensions are not reconstructed or reset.
- Existing Save finish in `files.rs:619–662` retains the same reservation through
  publication and finish. Its generation guard prevents a published older save from
  associating its path/baseline with New. It still returns a truthful historical receipt.
  The previously executed independent six-case command acceptance includes this exact
  generation0→repeated-New case and unchanged external file consumers; it was not rerun.
- `file_ui.rs:250–267` requires the current origin-qualified request, Accepted phase
  and New action before invoking the same core command. The currentness check includes
  generation/revision/path/close context; association/baseline changes invalidate old
  acceptance. Wrong action, unclaimed request, cancellation and replay refuse. New
  invalidates its Accepted intent without manufacturing a file receipt or successful
  native Save acknowledgment. It does not approve process/window close.

The three frozen private counter tests check exact current/saved Arc retention, stamp,
view and generation after each overflow. The three captured presentation tests check
unclaimed/wrong/dirty/cancelled/replayed/stale acceptance and absence of false Save/close
acknowledgment. Those source assertions and the existing independent cases support the
bounded conclusion; they are not universal concurrency or native proof.

## UI Save continuation: no material finding in the reviewed path

The actual frozen UI was read, including every independent App case and all eight
finite lifecycle cases. `FileWorkflow::poll` first waits for the exact retained NewSave
thread to finish and join, then consumes its result and calls `finish_ui_file`.
`finish_new_save` requires that trusted acknowledgment, current source/revision/close
and file epochs, matching path/format/receipt, clean state and an armed continuation.
It invokes the same core New with `discard_unsaved:false`; it never discards changed
data because a Save button was merely activated or bytes were published.

Cancel/close disarm while retaining the thread barrier. Historical replacement cannot
retarget New. Poisoned-lock cleanup cancels, waits for retirement, drains the result
without acknowledging or replacing the session, and reports successful Save separately
from New. Failure aborts New before allowing an ordinary Save retry. The eighth case
tests actual pre-cancelled Save and a successful ordinary retry that must not resume New.
These are source/test-body observations and supplied execution evidence, not reviewer
reruns or native verification.

The SDF editor adds only fresh `file.new` target mappings. Its existing source/file/close
qualification and Apply/Discard/Cancel behavior remain; a changed draft resolves before
a newly qualified New request. Four App cases cover source kinds, actual drawing/files,
draft Apply→actual SDF Save→New, ordered Undo→New and stale replacement. Old tests are
not rewritten: the View fixture retains its original prefix plus one child declaration,
and the eighth finite case appends to the exact seven-case prefix.

## P2: old-surface pointer input authors in an unseen new Drawing

One additional path is a reproduced defect. In frozen
`lib.rs:1938`, App-start `files.poll` may finish Save and create New before the workspace
branch is selected at1952. Raw pointer events were already sampled on the previous busy
SDF/SMILES surface. They can then enter the fresh Drawing event loop without a prior-
surface input fence. Invalidating old gesture/hit state does not itself reject a newly
sampled press/drag against the not-yet-shown Drawing.

The independent App owner froze one additive test before execution: inject a drag only
while the pre-frame Engine is SDF, stop on the first Drawing frame and require it empty;
a following fresh gesture must work. Actual worker53938 exits101,0 passed/1 failed:
the first Drawing already contains an unintended C–C. This is a product defect, not a
test locator/setup correction. The26,124B test SHA256 is
`8a2221fba0af6b2f2d474b8a1b308242477e1100daad79c064a7f3ef9345aba7`,
with the four-case22,237B prefix exact. Its1,785B failure log SHA256 is
`7b945f39121b49b50804a96faf3a3686f0fb69241fc3f53907bb4d6ed137288b`.

Root owns the fix; no fix or duplicate runtime was performed by this reviewer. The
transition needs to reject input sampled on the old surface while preserving deliberate
ordered New→drawing input and a following fresh gesture. The ignored
`fifth-red/captured-inputs.json` records the independent source/log/receipt copies.

The first narrow poll fence is separately captured: `lib.rs`94,963B/SHA256
`aea0939d5edfc450abb95bbc791acbd6487a4f5766a42defcb18726c06483fa5`.
It compares stamps immediately before/after file polling, clears stale gesture/hover/
navigation and suppresses that pass's canvas pointer authoring while retaining typed
menu/key processing. Producer18706 reports the unchanged five App cases GREEN. This
addresses New executed *during poll*; it cannot see a registry New already committed
between frames, nor ordinary Open whose Engine finish runs in the worker before polling.

The sixth additive actual-App case confirms the same defect across the other New
interface: warm the old SDF surface, dispatch core `document.new` through the shared
Engine between frames, then inject the old-surface drag in the first Drawing frame.
Worker71635 exits101: the first Drawing contains an unintended C–C. Its28,497B source
SHA256 is `fd423e4632d6719db5fb6e1b012a493e47990ddb5b64b56047bb3eb7762c66a4`,
retaining the complete five-case prefix. The1,789B log SHA256 is
`8b07981dd0d7de43acb375fc1fa781a85ab53b4e9dff7719233a12f5d8cefb5c`.
`sixth-red/captured-inputs.json` preserves those exact independent inputs and supplied
runtime evidence; no reviewer execution is attributed.

Root accepted a persistent last-presented session namespace in existing DrawingState.
Full CaptureStamp/revision is too broad because normal authoring changes it. A namespace
must become presented only after actual surface presentation, not on an invalid or
undisplayed viewport path. The exact Rust-only getter is now captured:
`new_document.rs`2,246B/SHA256
`efd5bf9406a15779ea64bced6f4b0b1366a2335ce92739f6a5a847bf2f546ada`.
Its only change from the original command is `session_generation()`, documented as
New/Open-monotonic within one Engine, not global identity or command authority. Root
reports missing-API compiler RED then namespace test9173 GREEN.

### Resolution in the exact final snapshot

Final UI `lib.rs`95,024B/SHA256
`b7797a875ff8bdfb0d3d1c662c298979c38033dc8b33bb5bd17838419409728e`
replaces the poll-only flag with one `Option<u64>` in existing DrawingState. At719–726,
a different previously presented namespace clears gesture, hover, hotspot and navigation
before input. Pointer canvas authoring is fenced while indexed typed menu/key commands
keep their original order. No prior namespace preserves initial startup behavior.

The read-only SMILES and SDF branches record their laid-out source surface at921/933.
Drawing records only after a successful graph's paint path at1822. Invalid outer screen,
source completeness, missing page, viewport or graph failures return without adopting
the new Drawing namespace. A rejected first frame therefore cannot silently acknowledge
an unseen Drawing; a following successfully painted frame rearms fresh input. Existing
hover invalidation never clears the namespace itself. The App retains the same shared
Engine; callers of the public compatibility helper must likewise retain DrawingState
with that Engine or start a fresh DrawingState when replacing it. The local generation
getter does not identify unrelated Engines.

The reviewed collection-to-Drawing same-pass early returns request an egui discard;
pinned egui0.29.1 `Context::run` calls `RawInput::take`, which drains the event vector
before a retry. I found no concrete old-event replay bypass in those source paths. This
is not proof of native presentation, visibility under arbitrary host embedding or every
zero-size/multipass combination. Successful egui graph painting is the reviewed tag
boundary, not an OS screenshot acknowledgment.

Producer52121 reports all six unchanged App cases GREEN on that exact final source.
Independent receiver84175 also passes6/0/0, including both reproduced failures and
following fresh gestures; its1,346B log SHA256 is
`d3841023267dc5cc02131c515de687e5c35dc8ee54a27f05e1d4285fdee41789`.
The28,497B declaration remains byte-exact against the sixth RED. The receiver reports
all-target UI clippy36705 and fmt exit0. Workflow0cdc58af…, its New child3fe94cd4… and
SDF editor882cb5d9… remain byte-exact. The source review finds the two reproduced P2
cases addressed and no additional material issue within the captured paths. Aggregate
gates and native validation remain their owners' work. All execution in this paragraph
is supplied producer/receiver evidence; no such execution is attributed to this review.

Exact core sources are the five-file manifest SHA256
`92d61648bc16fb2baf2dcbe9de076ea385c64c87c88375b7af1b87626e82ab8d`.
`target/new-document-review/core-inputs.json` records those five files and four unchanged
context files; `presentation-test-input.json` records the exact three-case presentation
source read from Root. Final core `lib.rs`1b500cad… reconstructs v1c61adc2f… by removing
only the32B test-catalogue New row; production is unchanged.

UI captures: original manifest`9d36cb3f…` and `ui-source-freeze-v2.json` retain all exact
sources. V2 changes only an existing FileJob reference instead of three separate
arguments, with exact inverse proof, plus the eighth finite case. Four production pins
are `lib.rs`00346a31…, `file_workflow.rs`0cdc58af…, its New child3fe94cd4… and
`sdf_editor.rs`882cb5d9…. `ui-final-inputs.json` and `ui-v2-inputs.json` record full hashes
under the ignored review directory. These are review copies, not production changes.
