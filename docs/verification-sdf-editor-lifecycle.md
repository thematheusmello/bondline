# SDF editor publication and retirement gates

These two private UI lifecycle tests use the real registered `sdf.field.set_value`
operation. They verify publication, cancellation and source replacement independently
of a native window. They do not establish native editor, dialog or accessibility
behavior, or complete the SDF workspace.

## Independent source expectations

The unchanged original mixed SDF fixture is 777 bytes (SHA-256
`165e6f5d5cff37dae6315a07843d84733aa42dbbc497b2920da1702bad9526a0`).
The expected edit is its literal prefix `[0,378)`, the bytes
`reviewed\nline two\n`, and its literal suffix `[385,777)`. The resulting
788 bytes have SHA-256
`e63cb9adf5cbc62b5b0be94992246c0d10b94cda405e49b9514993e7d9f9c361`.
This expectation is not regenerated with the SDF writer. The unchanged replacement
fixture is 690 bytes (SHA-256
`f95838cbb55c107c649ce2eb99ac62e191f747ce6994a171e64019f8e6c53d1a`).
The tests compare every byte of the active source and actual temporary input files.
Existing chemistry consumers, fixtures and oracles were not changed or rerun here.

## Finite registered-work gates

`crates/ui/src/sdf_editor_lifecycle_tests.rs` is a child of `sdf_editor` through
three test-only module lines. It uses private editor state without a public test
hook. One shared gate prepares the actual `UiSdfWork`, calls its real
`complete_blocking`, publishes the actual result and authoritative post-commit
stamp, then holds the worker on a channel. This distinguishes published results
from thread retirement without a sleep-based ordering assumption. Channel waits
and retirement polling have five-second failure bounds; cleanup cancels, releases
and joins the owned thread outside an engine guard or UI frame.

- In the cancellation case, Apply has already produced the exact 788-byte source,
  revision 2, one undo entry and a dirty file before Cancel Work is requested.
  Even after shared Close Request/Discard approves close and the executor permit
  has retired, the still-live editor worker blocks final host close acknowledgement.
  Cancel removes the waiting Save. Retirement keeps the committed source and history,
  issues a refresh, starts no UI file request, leaves the original physical file at
  777 bytes and reports that the value was applied before cancellation completed.
- In the historical case, another real `document.open_sdf` replaces the source with
  the 690-byte fixture after publication but before UI retirement. The late result
  cannot replace the new revision, source, history or clean association. The original
  text and historical-success status remain available for recovery, the waiting
  Save is dropped, and a subsequent Apply refuses the stale draft. Both files remain
  byte-exact.

The harness checks the same editor/executor barriers used by the native host; it
is not an actual operating-system close event or a complete App input-flow test.

## Observed RED and GREEN

All Cargo runs used Rust 1.94.1, the worktree's absolute `target`,
`CARGO_INCREMENTAL=0`, and dev/test debug information disabled. The pinned
`BONDLINE_PYTHON` interpreter was available, although these lifecycle tests do not
invoke it.

```sh
cargo +1.94.1 test --locked -p bondline-ui --lib sdf_editor::lifecycle_tests::
cargo +1.94.1 clippy --locked -p bondline-ui --all-targets -- -D warnings
cargo +1.94.1 fmt --package bondline-ui -- --check
```

The original test was frozen before its first execution at 10,740 bytes,
SHA-256 `1a7b448f4b3cdf1eed6a14e79e3524b0d38ff66f4b9f43ba43c197279668316f`.
Session 29790 exited 101: the historical case passed; the cancellation case failed
at its final truthful-warning assertion after all actual commit/source/history,
close barrier and no-Save assertions passed. The complete first log is retained
in `target/sdf-editor-lifecycle-tests/first-runtime.log` (SHA-256
`2f8032dab7bba7c2496b546608d6610d888b51cb0cfe3827e00b6dc684c142d4`).
Root reproduced that failure and supplied the production correction. With those
received sources, session 71739 exited 0 with both unchanged tests passing.

The first all-target UI clippy run (82400) found only `duplicate_mod` on the new
reuse of the existing bounded temporary-directory helper. With explicit Root
authorization, its test-only `allow(dead_code)` declaration gained
`clippy::duplicate_mod` and a specific reuse rationale. The original file and exact
lint-only diff remain in the ignored evidence directory. All bytes following
`mod support;`, including every helper, test body and assertion, remain exact.
The final test file is 10,788 bytes, SHA-256
`86bc383a9a4b68dff38bf0c141500969ac88122e0f70e5f261156ea7fef7e611`.

Final checks all exited 0: both tests in `final-runtime-confirmed.log` (2 passed,
0 failed), all-target UI clippy in `final-clippy-lint-classified.log`, package
format check in `final-package-fmt-confirmed.log`, and the new file's separate
non-mutating rustfmt check in `final-owned-fmt-confirmed.log`. This worker did not
run a full workspace or native acceptance gate.

## Received production fixes and preserved inputs

A bounded source recheck confirmed the received editor reads the actual job
cancellation flag only after its worker is finished. A successful changed result
now carries the truthful cancellation notice; historical success retains recovery
text and clears the waiting continuation. The received fields workflow queues both
Next and Previous until its refreshed page query returns, and stores continuation
warnings in a separate notice rather than the query-error field. The Previous
App regression and the broader editor matrix are separate Root/peer evidence;
this worker did not rerun them.

Root's final backend port receipt records editor SHA-256
`f80c1c334161de8ffd855f475d6db05e7497ddb79195a94759e95822456aed9e`,
fields `d9d755d47073747d4b35d61e4065b14f01ad7de1e5f5a334c3ec5d18434decf7`,
and draft text `578f6a6ab03a3363b8ccb529feda01c7416279d471cb72d18b84cbc168b01d66`.
These are explicit received changes, not an unchanged-production claim. Of the
19 separately protected inputs, 17 remain exact; fields and draft text are the
other two explicit received ports. The UI main module, file workflow, MCP work
bridge, command edit/close/file contracts, original fixtures/oracles, both lockfiles,
font and approved golden remain exact. No runtime dependency, shipped API, native
input, Actions state or Git state was changed by this test slice. Final ignored
`target/sdf-editor-lifecycle-tests/final-handoff.json` records exact files, logs,
port classifications and terminal job state.
