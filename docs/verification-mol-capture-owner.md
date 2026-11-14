# Complete deferred Drawing capture ownership

2026-11-14, Rust1.94.1/macOSarm64. This prerequisite retains source authority for later
display classification; it does not fix hydrogen labels by itself.

## RED and first GREEN

`cargo test --locked -p bondline-commands --test mol_capture_owner` failed101, first
session77980 and retained replay `target/mol-capture-owner-red.log`. Both new tests failed
their weak-source lifetime assertion: after dropping Engine, a prepared Drawing capture
held only Page and the original MOL bytes were already gone. The original740-byte SDF
fixture supplies its exact0..371 ethanol MOL span without alteration. Initial test SHA256
`af9e4a6cdb9cf4e9bf08c700142f7a96f392f7615fd7aaa72ace9748b7cf4276`;
RED replay log SHA256 `9bebc02f6aedbf4a7059f492867e74d7ea51b9b6e0d092b415e57ae8bc939102`.

After retaining the complete Arc<StoredContent> for every capture, the same two tests
passed in14581. Standard-library Weak::upgrade independently observes actual source
lifetime: work keeps it alive after Engine drops, and completion, cancellation and dropped
work release it. A completed historical capture can be finished without modifying another
Engine. A later rustfmt-only edit wraps the test signature/assertion; no expectation changes.
The initial formatting-only failure is retained in `target/mol-capture-owner-fmt-initial.log`.

Package tests in62405 pass201,0failed/ignored across31 result lines. Its subsequent clippy
found only type_complexity in the new test helper; a generic TestResult alias replaced
the repeated error type without changing assertions. Final71559 passes warnings-denied
all-target command-package clippy, workspace rustfmt check and both focused tests. The
failed lint log is retained as `target/mol-capture-owner-clippy-initial.log`. Full receiving
workspace, format and license gates follow the complete display integration separately.

## Scope

Drawing and SDF now use one capture constructor and complete-content ownership. Existing
page/kind/completeness/dimension checks remain before work. Projection/rendering still runs
off-lock; the retained source does not enter PNG JSON, grant a Save lease, acknowledge a
native request or alter state/history/close policy. Temporary worker ownership is outside
the existing retained-history accounting, as for SDF work; it is not a hard process RSS cap.

This is original lifecycle policy following the existing complete-SDF capture boundary,
with Rust [Arc](https://doc.rust-lang.org/std/sync/struct.Arc.html) and
[Weak::upgrade](https://doc.rust-lang.org/std/sync/struct.Weak.html#method.upgrade) semantics.
No format default, native behavior, new golden or chemistry correctness is inferred.
Ordinary source-backed MOL label routing and native canvas acceptance follow separately.
