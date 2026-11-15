# Native SMILES file conversion — implementation plan

Baseline`ac11ce6`,2026-11-15. Status: **in progress**. The existing independently checked
`file.convert_smiles_to_cdxml` stays the sole conversion operation. This increment adds
native presentation and lifecycle ownership; it does not complete M1 or reduce the full
M0–M5 target. General SMILES drawing import/layout and clipboard workflows remain required.

The [live reference review](live-ui-review-2026-11-15.md) observes separate Edit Copy As
and Paste Special SMILES roles, not an executed conversion. Bondline's create-only file
converter is an original workflow with a restricted profile, not either clipboard role.
The [existing command evidence](../verification-smiles-conversion-command.md),
[file evidence](../verification-smiles-conversion-io.md) and source register own chemical,
format, geometry and publisher policies. No parser/layout/fixture/oracle/dependency change
is planned in this increment.

## Contract

- Central File control`file.convert_smiles`, label **Convert SMILES file…**, no shortcut,
  dispatches existing`ui.file` with action`convert_smiles`. Registry stays30 commands;
  Drawing exposes18 controls, readonly SDF12. Six Page/bond-tool controls require Drawing.
- Native host adopts `UiFilePlan::SmilesConversion`. Specialized
  `Engine::prepare_ui_smiles_conversion(id,params)` validates the exact current Accepted
  ConvertSmiles intent, then prepares the existing eight-parameter command. Success alone
  consumes that presentation intent. Failure leaves it accepted. Generic
  `prepare_ui_file` and `Prepared::bind_ui_request` keep refusing conversion; no Save
  binding/ACK/file_operation, history, source, association or saved-baseline mutation.
- `CommandExecutor::prepare_ui_smiles_conversion` reserves the existing worker under the
  origin mutex before core preparation. Its typed work reuses registered completion,
  current save lease and existing worker budget. Validate source/revision/path/generation/
  close/request context before spawn, not only before releasing the frame guard.
- One child owner inside existing FileWorkflow holds form/browse/queued/job/result state;
  no App field, second chooser, executor or pool. Reuse a narrowly restricted FileChooser:
  **Choose SMILES input** / **Choose new CDXML output**. Browsing does not discard current
  changes and never offers overwrite. All eight command parameters remain explicit.
- Form fields: Input file, Output file, Bond length(pt), Origin X/Y(pt), Page width/
  height(pt). Original defaults48/72/72/exactA4. Bounded numeric text parses on Convert;
  invalid/nonfinite geometry fails visibly rather than clamping or publishing. Explain
  one connected1–12 neutral carbon chain with single bonds, unchanged active document and
  required new CDXML destination. Existing complete-input validation remains authoritative.
- Work starts after the frame Engine guard releases. Actual JoinHandle/channel/cancellation
  ownership lasts through retirement, including close and poison. Modal input stays fenced
  through Convert/Cancel/Close; numeric1/2/3 must not edit an underlying bond. Staleness
  disarms unstarted work. Retry requires an explicit fresh qualified intent. Publication
  results stay truthful, even historical or after cancellation; postpublication failure
  must not falsely promise no file exists.
- Result headings **Conversion complete** / **Conversion failed** support clear outcomes.
  Success shows actual path/counts and directs explicit File → Open. No automatic Open,
  New, overwrite or saved-baseline update. Ordinary Open protects unsaved data separately.

## Test and integration sequence

Independent command/MCP and actual-App declarations precede production. Their initial
existing-API runtime REDs are real missing enum/control failures, separate from later
missing new Rust method compilation. Root CLI and stdio/authenticated-loopback checks add
transport evidence, not a claim that headless clients display native dialogs.

Keep success/error/history/input bytes, actual external output consumers, exact accepted-
intent authority, full-budget refusal, old generic binder refusal and finite cancellation/
publication/retirement/close/poison gates. Actual-App cases cover Unicode browsing, existing
destination and unsupported input refusal, invalid numeric text, explicit editable reopening,
modal input and stale-source handling. Preserve all original assertions/timeouts/fixtures.

Root's first new loopback test incorrectly expected18 controls after opening readonly SDF.
That new-test setup is corrected to12: baseline11 plus the new control, with the original
declaration/log/inverse retained. Corrected baseline still fails11-versus12. No product
change or old assertion is involved. Recorded runtime/compiler stages remain distinct.

Receive frozen source/test manifests, run focused checks and the unchanged local aggregate,
then build and inspect the actual native workflow and independently consume its saved file.
Update parity/provenance/continuation records and commit intentional files locally. Actions
remains disabled. No unexecuted gate, native journey or complete feature is claimed here.

## Public engineering evidence

Rust's official [JoinHandle documentation](https://doc.rust-lang.org/std/thread/struct.JoinHandle.html#method.is_finished)
distinguishes channel delivery from thread lifetime; dropping a handle detaches, while
checking completion before join supports prompt retirement. The official
[Receiver documentation](https://doc.rust-lang.org/std/sync/mpsc/struct.Receiver.html#method.try_recv)
distinguishes an empty queue from disconnection and documents buffered messages after
sender loss. Keep the actual handle, not a receipt-only busy flag. These reviewed pages
identify Rust1.100; the selected APIs are marked stable since1.61 and1.0. Versioned1.94.1
page retrieval was unavailable. Actual repository compilation/tests still use pinned1.94.1;
no new stable-page API or toolchain upgrade is implied.
