# Independent New-document command acceptance

This test-only slice starts at `69e4b37` plus five explicitly received production/test
inputs in an isolated worktree. Six independent command cases pass. The complete
project goal remains active; New supplies a missing entry to drawing, not M0/M1
completion. Root owns production, Git, integration and native execution.

## Public behavior and original policy

The public [Guide25](https://chemistry.beloit.edu/classes/programs/ChemDraw_25_manual.pdf),
printed13, documents creating a document through File > New with defaults or a style
sheet. Printed39 describes an untitled copy of the style sheet and a style-dependent
New label. The pinned17,721,447B PDF SHA256 is
`3f1ebc429df07efdd47b6218a148dc346a215043f60bf89ee9a1cb90629bd5b3`.
MF-001 and KM-001 in the parity checklist record New/printed13,39 and Ctrl+N/printed77.

Bondline's single-window replacement, blank default A4 page, clean blank baseline,
new generation, reset Fit camera and local ID namespace are explicit original policies.
They do not establish a26.1 stationery/default-style, multiwindow or Mac shortcut match.
No proprietary style sheet, asset or help prose is copied.

## Declared scope

The shared typed `document.new` has exactly required `expected_revision:u64` and optional
`discard_unsaved:bool=false`. It creates a fresh source-free Drawing even if the previous
document was already blank. Normal direct invocation refuses stale revision, dirty data
without explicit discard, pending close decisions and committed close. Native File > New
is separate presentation intent (`ui.file`/`new`); it must resolve drafts/unsaved data
before invoking the same core operation, rather than using close approval as replacement
authorization.

The six acceptance cases are independently declared against the existing public API:

1. Open the unchanged777B complete SDF, then New: clean source-free default Drawing,
   revision2 and exact unchanged input file.
2. Exactly30 registry names retaining all29 previous entries; closed mutating schema,
   false discard default, malformed parameters and qualified central New binding.
3. Fresh Drawing from source-free/native-CDXML/CDX/MOLV2000/MOLV3000/SDF/SMILES content;
   empty A4/IDs, new stamp, reset Fit/center with advancing view epoch, retained actual
   canvas/tool/native host and unchanged old snapshots/files. Repeated New clears both
   history directions, resets graph IDs and supplies the real clean Undo baseline.
   A post-New C–C drawing is saved/reopened through actual CDXML files.
4. Dirty Drawing/SDF, absent/false discard, stale/invalid input and all three pending/
   committed close states refuse atomically; explicit discard creates the fresh blank.
5. Previously completed deferred Open/field/remove candidates cannot install after New;
   a captured read-only query remains historical and cannot restore the old collection.
6. A captured old-generation Save remains truthful and holds the same lease through
   publication/finish across repeated New. It cannot associate its path or saved baseline
   with the new session. The subsequent current Save uses the released original lease.

The independent expected paper dimensions are `210*72/25.4` by `297*72/25.4` points.
Post-New graphs require local fragment/bond1 and atom1/2, neutral C/H3, Single, at literal
`(100,200)`/`(140,200)`. Existing `files_rdkit.py` independently uses Expat plus external
pinned RDKit2026.09.2 to check those coordinates, references, authored H, `CC`/`C2H6` and
every actual saved fragment. No oracle/fixture/golden/dependency is changed.

## Test-first and receiving evidence

The first1,184B test was frozen before Root implemented New, SHA256
`b6650834ac56d23dedd34d17e4699a15ba55c8bba796ccdb53573d3f7d969542`.
Worker8631 actually compiles, opens the original SDF and fails at runtime with
`UnknownCommand("document.new")`: exit101,0 passed/1 failed. The log is
`target/new-document-command/minimal-runtime-red.log`; no future Rust API or compiler
failure substitutes for this behavioral RED.

The additive six-case declaration19,435B/SHA256`e151e9ff…` remains archived. Source
review found that its unrelated SetZoom setup would attempt unsupported CDX preview.
Root approved only selecting SetZoom for SDF or fully mapped Drawings, preserving every
New assertion and the first1,184B prefix. That19,609B declaration SHA256 is
`7ffd3a7c42468fa3cb1d66fe4475e97863ca35ca1893ade749cc38bab3ad7fb7`.
This pre-runtime setup correction is not a producer failure or a new RED.

Root's real backend was received as exactly five files, with before/after identities in
`target/new-document-command/backend-port.json`. Its source manifest SHA256 is
`92d61648bc16fb2baf2dcbe9de076ea385c64c87c88375b7af1b87626e82ab8d`.
It contains command `lib.rs`, new command/helper tests, File intent routing and central
control metadata. No worker-authored implementation or backend stub is used.

First six-case run31162 exits101 with5 passed/1 failed: the all-content loop's unrelated
Triple-tool setup ran after opening CDX, whose existing tool-selection guard requires a
fully mapped Drawing (and similarly refuses collection kinds). The failure is preserved
in `first-six-runtime.log`; no New producer defect is claimed. Root approved moving only
that existing Triple setup before the optional Open while the Engine is still a fresh
Drawing. The original all-content cases, current-tool/host/camera/source/history checks
and first1,184B prefix remain exact. No production eligibility guard was broadened.

Final source is19,609B/SHA256
`ea4f275d66a535890734e500c4079b1808b0d5400ac49470ee6451c486a8ca91`.
Both earlier declarations are archived separately. Reversing only the tool setup move
and the approved view-setup eligibility correction reconstructs the original19,435B
six-case declaration exactly. These corrections are new-test setup, not fabricated
behavioral REDs or changes to intended New behavior.

Worker2243 runs all six cases against the exact real backend: **6 passed,0 failed,0
ignored**. Log: `target/new-document-command/six-final-runtime.log`. This includes actual
post-New Save/reopen, unchanged Expat/RDKit consumption of new and captured-old files,
explicit stale/historical results and unchanged old source files. Generation monotonicity
is tested behaviorally: an outstanding generation0 Save cannot associate with or change
the clean baseline after repeated New. No private Engine counter is exposed for this test.

Worker51093 also exits0 for warnings-denied command all-target clippy and workspace
formatting. Logs: `command-clippy.log` and `fmt.log` in the same evidence directory.
No production change was needed between the received backend and these scoped gates.

## Reproduction and limits

Use the worktree's own absolute Cargo target, incremental0, dev/test debug0 and literal
Root pinned `BONDLINE_PYTHON` path. Focused command:
`cargo test --locked -p bondline-commands --test new_document -- --nocapture`.
The initial435-input baseline is `target/new-document-command/baseline.json`.
Final validation preserves432 inputs unchanged plus exactly three authorized existing
Root replacements; the remaining two received Root files are new command/private-test
modules. The final receipt records those five sources separately from the two worker-
authored files, every historical declaration/log and the source reconstruction checks.

Native UI, complete adapter wire coverage, stationery/multiwindow semantics and other
platform workflows remain separate. Root owns existing catalogue additions and the full
integration gate; this worker does not alter old29-name assertions or invoke Actions.
All owned jobs are terminal; no native input or Git mutation was performed. Handoff:
`target/new-document-command/handoff.json`.
