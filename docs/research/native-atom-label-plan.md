# Native elemental atom-label presentation

2026-11-16; continuation of the bounded `atom.set_label` command in `525e117`.
The complete M0–M5 goal stays open. This plan describes original incremental
presentation, not complete reference Text-tool or screen parity.

## Evidence and intended workflow

The [elemental authoring contract](atom-label-authoring-contract.md) pins GUIDE25,
IUPAC GR2.1, OpenSMILES and SDK literal-H semantics. Its two official25.5 knowledge-base
articles distinguish Alt-click entry from Text-tool repetition. Their exact raw public
pins are retained there. A fresh web read of both article pages failed on2026-11-16;
no installed reference app, binary or asset is inspected. Exact26.1 Mac/tool precedence
remains unresolved. Alt-primary activation here is an explicit Bondline input policy,
not evidence of that missing reference behavior.

Fresh Alt-primary input on an existing atom or its label ink opens one **Atom label**
form. Reuse current graph picking, canvas/view transforms and namespace fencing.
Empty paper and bonds never substitute another atom. The form shows the exact target,
**Element**, **Attached hydrogens**, **Apply** and **Cancel**. Element uses the existing
exact-case118-symbol catalog; attached H is explicit decimal0..65535. Empty/invalid
fields and physical H with positive attached H cannot Apply. No valence inference,
neighbor adjustment, alias parser or automatic charge correction is introduced.

Drafts use the existing checked, whole-reject `SdfDraftText` buffer with additive
per-instance byte limits2/5; its existing65536-byte constructor remains unchanged.
Values stay copyable after a stale context. No draft change creates history or a Save
baseline. Use existing text-widget behavior and bounded undo rather than another editor.

## Shared presentation contract

Register one additional command `ui.atom_label`, closed and tagged by `action`:

```json
{"action":"open","expected_revision":1,"page_id":1,"fragment_id":1,"atom_id":2}
{"action":"cancel","expected_revision":1,"request_id":1}
```

The existing attached native-host signal is reused; headless hosts report
`ui_unavailable`. Open validates revision, Open close state, idle file presentation,
complete source authority and the exact neutral/nonisotopic/nonradical target. It queues
one request; an existing request, including a stale one, refuses with `ui_busy`.
No executable empty-target binding or `ui.file` atom-target overload is added.

Optional wire `ui_atom_label` contains only `id`, `input_revision`, `page_id`,
`fragment_id`, `atom_id`, `element`, `explicit_hydrogens` and `current`.
IDs increase on Open. Optional `ui_atom_label_operation` identifies actual presentation
changes for the existing authenticated control wakeup. Authoring `changed`, revision,
history, complete source, saved baseline, view and tool remain unchanged.

An internal weak source/revision/generation capture plus opaque file-context identity,
file/close/view epochs and tool state qualify the request. Changes latch it stale rather
than retargeting its original fields. Cancel validates current revision and exact request
ID, clearing even a stale request. Wrong IDs refuse atomically. The global committed-close
registry refusal remains. Host-only exact-ID retirement can release presentation after
committed close; it grants no authored edit, Save or close acknowledgment. Host detachment
clears pending presentation so another host cannot adopt it.

Native Apply revalidates the complete request under the same Engine lock and invokes
the existing `atom.set_label` registry command exactly once. Successful mutation or
validated no-op retires only that form. Errors retain the draft. No worker or scheduler
is added for this bounded cheap operation. Future foreign-source editing needs its own
source-synchronization/worker contract before any native control can authorize it.

## Input and lifecycle gates

Opening, text changes, Apply and Cancel respect raw event order. A later field event
cannot change an earlier Apply. A form present at frame start fences the underlying
canvas, tool/history/view and File input through dismissal. File publication and native
close wait until explicit Apply or Cancel; the first form does not silently discard or
auto-apply a draft. External state changes retain recovery text with Apply disabled.
Engine poison preserves recoverable UI text and cannot grant publication authority.
Actual native close is cancelled while the form is unresolved. Remote committed close
must still permit exact host retirement, without changing authored data.

## Test-first acceptance

Independent initial central dispatch and actual-App declarations are frozen before
implementation. Root observes real UnknownCommand/missing-form runtime failures before
writing product code. Expanded tests cover closed schema, source/target guards, one
request, stale/ABA contexts, cancellation/host retirement and truthful control wakeups.
Actual-App tests exercise fresh atom/label hits, original fields, bounded whole-reject
paste, explicit OH/NH2 edits, Cancel, stale recovery and packet/close/File fences.
Actual Unicode CDXML Save/reopen is independently consumed using existing Expat/RDKit
tools. Keep old tests, fixtures, consumers, approved golden and parser fuzz targets exact
except mechanically necessary registry discovery metadata and test-module bridges.

After focused GREEN, run the full local gate, build and inspect the actual native app,
and consume its actual saved output independently. Keep one preview and protect existing
unsaved work. Windows/Linux, measured DPI, full inline labels, aliases, repeat-label,
charges/isotopes/radicals, selection/rings/styles and all remaining milestones stay visible.
