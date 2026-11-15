# Native SMILES conversion safety review

Bounded independent review from `ac11ce6`, with the original core/worker acceptance
handoff `ac3a6723…` held unchanged. Sources were copied and hashed before review, not
read from a moving implementation. Root owns product fixes, integration and native input.

## P2: dropped conversion owner can still publish

The first UI snapshot adds a retained conversion Job to FileWorkflow. Its ordinary
`Drop` implementation cancels `self.job` but omits `self.conversion.job`. Dropping the
new Job therefore detaches its JoinHandle without requesting cancellation. A registered
converter that has not yet performed I/O can continue and create its output afterward.

Root authorized one additional private regression, preserving all six core, two public
MCP and five independent App cases. The test uses the real Accepted conversion intent,
registered `UiSmilesConversionWork`, actual Job owner and actual create-only publisher.
Only the worker's entry before I/O is gated. It drops FileWorkflow, releases the gate,
observes completion and verifies unchanged input/current state before asserting that no
output was created. Gate release also has a Drop fallback; the intended failing path
finishes the worker before any assertion, with no active registered work left.

Runtime19237 exits101:0passed/1failed. The actual converter reports success with a real
publication receipt and the output exists; `cancellation_at_drop=false`. This is a
product ownership defect, not a new-test locator or timing correction. The new4,251B
test SHA256 is
`f2d2ccea24ae257a0d8b549da52c0f0fb8e976b1fc5221e2fc52e835c5f2b67a`.
Its4,101B log SHA256 is
`daa4038861cc282d4b29a6c8199b82a276f9da28d8842618dbdfff5030326c5e`.
The original25,781B conversion module is retained exactly plus an85B cfg-only child
bridge. No product fix was made by this reviewer. Root owns the narrow owner-cancellation
correction. This case establishes owner destruction,
not an actual native window-close race or an overwrite/source mutation.

### Fixed by the existing owner's cancellation path

The reviewed correction adds `self.conversion.cancel()` to FileWorkflow's existing Drop.
A private-boundary helper requests its Job's cancellation token without joining or
blocking. The one-line call's inverse reconstructs the original31,633B/d6e39fcd… file
exactly. The helper bytes match the held26,528B/820c0824… conversion module exactly.
The producer's initial parent-field-access compilerE0616 stage is retained separately;
it is not the behavioral RED. No visibility beyond the existing private boundary was
opened, and no worker, listener, reservation or public API was introduced.

For an independent causal check, this reviewer received only the exact call/helper over
the archived first UI, retaining the same85B child bridge and4,251B test. Removing those
two code additions restores the failing sources exactly. No unrelated field/stale-form
fix or producer worker test was needed. Own runtime92618 passes1/0/0 unchanged;
other test targets were filtered, not counted as additional passes. Its causal module is
25,988B/SHA256 `44817cc42febe0ffe1ea23af5e0f52a2a4f6975c03211ac1fb07010e6791743b`.
UI all-target warnings-denied clippy18188 and the owned-test rustfmt check exit0.
This resolves the reproduced P2 for pre-I/O owner destruction. Cancellation remains a
request, not rollback of a file already published.

## No additional material finding in the captured paths

- Accepted native preparation uses the existing conversion command. It validates the
 exact current Accepted action, consumes the presentation only after successful prepare
 and preserves its epoch. The originating mutex covers the existing worker reservation
 before that consumption. Generic Open/Save binding and trusted Save ACK remain separate.
 The previously frozen six/two-case acceptance independently passes these paths and real
 output consumers; it was not repeated for this source review.
- Form context records full CaptureStamp plus revision, close/file epochs, path and
 format. Source, history, association and close ABA invalidate an unstarted request.
 Submit remains deferred until the frame Engine guard is released; the native adapter
 rechecks under the origin lock rather than trusting the earlier UI check.
- Pending/job presence blocks FileWorkflow authoring and close. Normal retirement checks
 `JoinHandle::is_finished`, joins, then reads the real result. Late cancellation preserves
 a real successful receipt; historical results explain that the current document was
 not replaced. Failures and uncertain/poisoned results tell the user to check the
 destination before retrying. No conversion result calls `finish_ui_file`, creates a
 Save receipt, auto-opens output or changes saved/current/history data.
- Retry is explicit: after a consumed request, the form must still match its original
 source/file/close context before requesting and claiming a fresh native ID. It does
 not silently rebase stale parameters onto another source. Busy preparation retains an
 accepted intent and performs no automatic retry or overwrite escalation.
- Both browsing roles reuse the existing FileChooser, with fixed SMILES input/CDXML
 output roles. Submit uses create-only `overwrite:false`; conversion selection consumes
 only the selected path, never ordinary Save work or a dirty-current-document discard.
 The ordinary `failed_with_format` overwrite-retry path is not used for conversion.
- The foreground form is also an explicit workflow authoring block, not just a painted
 window. Its actions replay at event indices alongside existing File/SDF decisions.
 Source inspection supports Convert/Cancel/Close fencing without new drawing authority;
 the five actual-App cases are separate behavioral evidence, not duplicated here.

These conclusions concern source paths read, not universal concurrency, platform or
native proof. The original lifecycle test checks queued work after source replacement.
The later12,610B/2d137795… producer test file was also captured and all four bodies read:
real CCC output is independently consumed before result-ready threads are held; busy/close
remain blocked until actual retirement; late cancel/New retain a historical success;
close waits; and poisoning after the real result exists drains without Save ACK/source
mutation. Producer98209 passes those four, then16744 passes them plus the unchanged Drop
case. These are supplied executions, not reviewer reruns. The poison case is after real
completion, not proof of every lock-poison/publication ordering.

## Captured evidence and boundaries

The first UI manifest is1,046B/SHA256
`8fbf864f0c32739d0a32b0be8f8dc79fc338446ff117f74715c09cd7ce724528`:
lib4138d921…, file_workflowd6e39fcd…, conversionc67847fb…, filesbae00fc9…,
SDF editorf21a5f54… and its initial lifecycle test227957b6…. The four core/MCP inputs
remain the838B manifest3513bf89… from the prior acceptance. Full paths, byte lengths
and hashes are in the ignored `target/native-smiles-conversion-review/captured-inputs.json`.
Copies under `ui-first/` and `backend/` remain immutable; only those exact six UI files
were received for the authorized regression, followed by the cfg-only bridge.

All five independent App cases were read from the26,224B source596b0597…. Its initial
missing-control RED and import/tool-setup records are preserved separately. Its first
real run9075 passes one case and stops four at a new helper's immediate InlineTextBox
readback: both the old chooser and new form handle AX SetValue after TextEdit layout.
That alone establishes no conversion failure. Probe65111 shows immediate parent AX value
and the following frame's correct child paint for both surfaces. The approved26,514B/
9b9af85d… helper change adds that immediate parent assertion and the existing bounded
paint wait, retaining the original equality and every workflow assertion. Its exact
290B addition was compared; no expectation or timeout was removed.

The full held UI manifest161,296B/19c2c595… was captured separately. Its bounded additional
diff applies manual field setters before layout and explicitly requests repaint on
changed/rejected transactions. It dismisses only unstarted stale Accepted forms, retaining
running/publication outcomes; this preserves the agreed stale-submit/dismissal policy
instead of weakening the fifth test. Its supplied five-App log b0f239a7… passes5/0/0.
The earlier helper/stale failure logs remain preserved. These full-held changes were read,
not silently substituted into the causal Drop run. There is no reviewer native idle-paint
or full-App execution claim.

No native input, proprietary program inspection, Git, dependency, oracle, fixture, approved golden or
Actions change was performed. The only new execution is the authorized owner-Drop
regression and scoped checks above. The two reproduced states and both causal sources
are retained; no additional material issue was found within the captured paths. Original
core/worker acceptance stays frozen. Root owns aggregate integration and native evidence;
all reviewer jobs are terminal with no live handles.
