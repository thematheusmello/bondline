# Derived SDF capture command

2026-11-14. The five lifecycle tests pass on the real integrated data/renderer route.
The full command-package tests/clippy/fmt pass; the workspace receiving gate is pending.
GitHub Actions stays off.
No native window, input, new golden or Windows/Linux execution is claimed.

The existing read-only `render.capture` command accepts SDF virtual page1. Its typed
parameters, registry entry and PNG response are unchanged. Preparation captures the
complete StoredContent owner and historical result; the existing executor performs
all projection, sidecar resolution, scene construction, CPU/GPU work and PNG encoding
off the engine lock. The source survives work without acquiring a Save reservation.
Completion returns historical authored/file metadata and current close/UI metadata,
as for existing drawing captures. Cancellation remains checked through finish.

The separate [data producer](verification-sdf-preview-model.md) and
[resolved label renderer](verification-sdf-display-render.md) supply every supported
record, original geometry directions, bounded fresh preview IDs and display-only H.
Source/model None and literal counts remain authoritative; no derived Page becomes
StoredDocument, drawing state, export authority or a history entry. Actual original
and edited SDF bytes remain available to whole-source Save. Unsupported later records
or unsafe depiction refuse the complete capture, rather than hiding a record.

This is a restricted headless display path. The native SDF workspace remains its
metadata table. No native preview cache, interaction, field editor, general bond-order,
charged/isotope/radical/stereo rendering or chemical clean-up is delivered here.

## TDD and receiving evidence

The first lifecycle test ran against the old implementation before the production
route was added. Session80755 exited101: `WrongContent { required: "drawing",
actual: "sdf" }`, zero passes/one failure. The complete output is retained in
`target/sdf-capture-lifecycle-red.log`. The known-H777-byte fixture in this lifecycle
test establishes source/history authority only; it does not substitute for independent
ordinary VAL-absent CCO/CN image acceptance.

Additional lifecycle tests cover field edits/Undo/Redo, historical capture after new
Open and deletion of the original path, early/late cancellation, independent Save
leases, refusal to bind native file requests and unchanged drawing/edit/export guards.
First real GREEN62736 exits0 with all five lifecycle tests passing; no production or
assertion change followed that run. Independent capture/adapter evidence and the full
receiving gate remain pending.

Package gate77463 exits0:183 command tests, warnings-denied all-target clippy and
rustfmt pass. The initial package attempt41810 stopped at one legacy unit assertion:
45 unit tests passed, but the640×480 capture returned unsupported_scale rather than
the prior unsupported_content. The new SDF route now reaches the existing readable-font
bound. Only that specific capture-code expectation and the test name were updated;
all edit/cross-format refusals, original source/state/no-I/O assertions and the capture
refusal itself remain. No production bound or drawing/SMILES guard was relaxed. The
failed output remains in target/sdf-capture-command-legacy-guard-failure.log; clippy/fmt
did not run in that stopped attempt. The successful rerun is recorded separately.

The [committed plan](research/sdf-preview-plan.md) and public sources there distinguish
BIOVIA source semantics, the original closed display-H profile, IUPAC attachment rules
and original row packing from inferred reference behavior. No reference assets or
proprietary implementation are reused.
