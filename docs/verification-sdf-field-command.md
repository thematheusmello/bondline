# Shared existing-field SDF authoring

This bondline-rxn slice starts at9cc40d0, the accepted read-only query feature. It adds only
sdf.field.set_value, retaining all19 earlier commands for20 total. There is no UI editor,
navigation/delete, molecular editing, source-codec/model/renderer change, new dependency,
lock, fixture, golden or native operation. This is partial M1, not format or product parity.

## Contract and source safety

The typed closed required parameters are expected_revision, positive record_id, zero-based
existing field_index and value_lines. No field is located by duplicate-prone name maps.
Cheap metadata validation captures complete original/current content in an immutable plan
without modifying authored/transient state. Off-lock work counts authored LF and serialized
vector bytes, then reuses SdfDocument::with_field_value for the checked exact splice and
whole-source validation. Each aggregate input budget is65,536 bytes, matching the existing
inspection limits. [] means empty; spaces/tab data stays literal; original codec rules/codes
and global bounds remain intact. Changed lines use LF; every other source byte stays exact.

Completed candidates retain input Arc/revision/generation and cancellation. Finish requires
that full current identity and an uncommitted session before one existing history commit.
No-op reuses current outer/source/index identities and preserves revision, history/redo,
dirty/saved/source metadata, close approval and trusted file acknowledgement. Changed input
uses normal revision/history/accounting and permanent close/picker invalidation. Stale,
reversed, cancelled, invalid, revision-exhausted or post-close work changes nothing.

The same two-worker/reply drain handles the edit, with no new pool or save lease. A captured
Save still publishes its old complete bytes and keeps newer edits dirty; its lease survives
publication through finish/drop. A completed close-Save receipt remains valid across a
true no-op but cannot revive after edit→Undo ABA. Automatic results and Debug omit field
text; explicit query reads the current or captured historical data. Query and file contracts
remain unchanged. See control-protocol.md for exact budget meanings and error codes.

## Independent test-first evidence

Before production implementation, target/sdf-field-set-red.log exits101 with eight
intended failures: exact19-versus20 discovery and UnknownCommand(sdf.field.set_value) across
the prepared state/source/history/error workflows. The first GREEN log then passes all8.
Tests were written against original independently declared source spans and expected bytes.

The690-byte V2000 vector changes only383..387 to revised-LF/multiline-LF, producing704 bytes;
the777-byte mixed vector changes378..385, producing788. Both complete sources, untouched
other records/MOL/header/DT/EOL/delimiter bytes and duplicate order are compared literally.
Actual saved704-byte output passes the unchanged raw scanner/per-record RDKit consumer;
mixed output retains literal344-byte V3000 and shifted282-byte V2000 payloads consumed
independently. Existing DT-only supplier loss remains a diagnostic, never a fidelity oracle.
No original fixture, corpus pin or external consumer is rewritten.

Coverage includes Open→inspect→set→inspect→Save→Undo dirty/original-unmodified→Redo clean/
source-modified through the engine, real CLI, stdio MCP and authenticated socket. Actual
704/788-byte files are consumed independently. Other-record/first-DUP/DT-only and same-length
source changes prove explicit occurrence forwarding; opaque molecular data remains exact.
Empty/spaces/tab values, exact64KiB JSON, raw/escaped overflows, bad IDs/fields/shape/encoding/
control/newline/dollar-prefix/201-byte input, CRLF no-op/source identity, cancelled/reverse/
old-generation completion, historical queries, revision exhaustion, captured Save and
close receipt ABA are checked. Busy worker failure mutates nothing; reply-flush tests delay
close acknowledgement/repaint until publication. No native UI success is claimed.

## Executed checks

All commands use the absolute isolated RXN target with incremental0/dev-test debug0 and the
existing pinned BONDLINE_PYTHON interpreter. Cargo was paused after Root reported shared
free space at125MiB and resumed after Root reclaimed inactive artifacts; no worker cache/source/
evidence or fixture deletion occurred. The own target was never shared.

Ignored logs are sdf-field-set-red.log, sdf-field-set-first-green.log,
sdf-field-set-scoped.log, sdf-field-set-scoped-final.log, sdf-field-set-clippy.log and
sdf-field-set-clippy-final.log. Both scoped command/CLI/MCP passes are actual executions,
and scoped all-target clippy passes with -D warnings.

`cargo xtask ci` then printed the exact isolated workspace
`/Users/sarenaberg/Workspace/doozle/open-source-world/bondline-rxn` and exited0:608 workspace
tests plus3 macOS bridge tests =611 passed,0 failed/ignored. This adds16 tests to the
595-check9cc40d0 query base and excludes later independent SMILES/writer integration.
Workspace rustfmt/all-target clippy, strict layers, unchanged declared runtime license
closures/assets and the pinned corpus pass. All prior93 UI tests still pass unchanged;
this is not actual native field editing or new-source hosted evidence.

Ignored `target/sdf-field-set-ci.log` is74,491 bytes/SHA-256
bd85175b2b5fec6a6b9904c9cce4edf4daf2f235d3f2cd0a74b5b3d1412728b9.
The15 source/test files are frozen in `target/sdf-field-set-source-freeze.json`,
4,206 bytes/SHA-256
7f1b08c97b624900ec87d41f2c686708d68aca459705d13d56eca6104b9beabd.
Every source/test hash stayed unchanged during the aggregate. Final documentation is
additive; build/evidence outputs remain ignored. No live job belongs to this handoff.

## Remaining work

No field editor/selection/navigation/deletion, record identity across reorder/delete,
molecular preview/layout/authoring, property-name editing, format conversion or native
field workflow is implemented here. Reference existing-data retention/edit/export behavior
remains unresolved. Root owns independent review, atomic commit, receiving integration,
new-source hosted evidence and separate native assignments. No milestone is complete.


## Receiving Root verification, 2026-11-14

Root strictly verified all18 intentional files, seven logs, eight protected identities and
exact file set, committed the frozen producer as6cc6bf8, then received it after the
query on its newer native/SMILES writer+normalizer foundation. Independent read-only review
reporttarget/sdf-field-set-review.md7444B/SHA
e89d788d15505e215109301110f33550fc088ab0e948edafda63a56e8d9f63bd found no confirmed defect;
no reviewer test execution. Production/adapter tests preserve the reviewed hashes and all
later source/native evidence. Root73303 cargo xtask ci exits0:631workspace+3bridge=634,
all gates GREEN, correct absolute Root target/incremental0/default debug/pinned external
Python. Logtarget/sdf-field-set-integrated-ci.log77103B/SHA
a8ce07f845ed489e6c182d1bfcab36d2d97f81dfade3bd75e718599f54f37ba2.
The receiving registry has20 names. Exact receiving-source hosted checks remain pending;
there is still no native field editor/preview, record operation or complete M1 claim.
