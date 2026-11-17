# SMILES UI test readiness qualification

2026-11-17, while receiving the atomic drawing-settings command on contract dede5ee.
This correction changes test readiness and diagnostics only. It adds no product feature,
native/platform acceptance, worker-policy change or explanation of the MCP bind failure.

## Observed failure and evidence boundary

Full gate37811 fails the unchanged MCP replacement bind51300 after1012 passes. Only
not-started stages run separately in24093. That run then fails the UI library case
`all_records_paint_and_key_order_waits_for_a_worker_before_history_and_whole_source_save`:
267 UI cases pass/1 fails/1 is ignored. Its error says that record UI did not become ready,
with the existing pending-bond-order message. The whole partial run is486/1/1 across50
result lines. Log62631B SHA256
`8ed05d3ea3422fa42d74a8949cc8b937d4e2a5b2d5eced1ec3b82a4876f531d2`.

The old timeout has no wait-stage/job-state capture. The pending message was intentionally
set earlier in this test. Therefore the log alone does not identify its failing wait or
establish which schedule caused it. No new-command defect, native regression or causal
repair of this particular run is inferred.

## Demonstrated precondition gap and correction

[finish_order](../crates/ui/src/native_smiles_records_tests.rs) waits for the committed
revision, idle FileWorkflow and inactive executor, then pumps two frames. It omitted the
separate OrderWorkflow owner. Executor completion can precede send/repaint/thread exit;
FileWorkflow does not own that thread. Two fixed frames do not establish retirement.

The existing independently gated lifecycle case
`real_sent_order_result_remains_busy_and_blocks_close_until_producer_thread_retires`
in [the lifecycle tests](../crates/ui/src/native_smiles_order_lifecycle_tests.rs) establishes
committed revision3/inactive executor while the actual order producer remains busy.
The existing App frame-start fence correctly refuses Save during that interval. If the
test advances early, its Save As can be consumed by that fence and its chooser wait cannot
succeed. This proves the old readiness condition is insufficient; it does not identify
the uncaptured schedule of24093.

The helper now additionally requires `!f.app.drawing.order.is_busy()`. Both callers retain
their original actions, revision/file/executor checks, two warm frames and ten-second
deadline. No assertion or busy fence is removed, error cleared, retry added or worker
policy changed. `wait_for` also records its caller, owner/file/executor busy states,
revision and existing error on timeout, making a future failure more useful.

Source-only diagnosis3927B SHA256
`ea1e8037b921568455bf1264f354ced5f83cf3c6e56361589075b19af6c1ca4f` is ignored under
`target/drawing-settings-command/ui-readiness-review/`. Root owns all executions.

## Actual qualification and target selection

Job85282 runs the entire modified UI library at unchanged default parallelism:
**268/0/1**,3.61s. Its subsequent `--bins --tests` selection also runs the library again:
**268/0/1**,3.40s, plus50 passing binary/integration cases. That duplicate library run is
not additional unique coverage. No further UI retry is performed.

That script then stops before xtask tests because `-p xtask --lib` requests a library the
package does not have. This is a selector/setup error, not a failed xtask test. The log
is586/0/2 across9 runtime result lines,68993B SHA256
`99bb3cec3621aa736ddac1546892984b42b9519a11488fa0d2148c238e6f414a`.

Only then-unstarted stages run in74317, using `cargo test -p xtask --locked`, compatibility
runtime targets, workspace doctests, vendor tests/lint and the existing xtask gates. They
pass **55/0/0 across18 result lines**, with rustfmt, warnings-denied UI/vendor clippy,
layers, seven-asset licenses, unchanged155/166/221 runtime closures and pinned corpus.
Log10550B SHA256
`8b4ff4807c49feb5b80c265941cb2e7e5a873a6e43cf8d39c285a0f3278cd640`.

Both earlier failed runs stay failed; these results are not an aggregate pass. The
MCP assertions/socket/timing policy and exhausted diagnostic boundaries remain unchanged.
No product UI, parser, format, dependency, fixture, consumer, approved golden or copied
user app/window is changed by this test-only correction. The full M0–M5 goal remains
active, Actions OFF, and native settings/chain delivery remains outstanding.
