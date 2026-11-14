# Registered native preview work

The five new integration tests and deterministic late-cancellation unit pass
headlessly on the real core port `803c83e633f19bddaa7af47123c9f47b8fc74d4a`.
Final serialized MCP tests pass all 48, with warnings-denied clippy and rustfmt.
A repeat parallel run failed an unchanged listener-shutdown assertion; its cause
is unestablished and Root's default workspace gate remains required. No native
workflow is claimed. Root owns integration and commits.

## Boundary

`CommandExecutor::prepare_ui_capture(&CaptureStamp, RenderCapture)` locks the
originating Engine itself. Call it after releasing the frame's Engine guard;
calling it while holding that mutex would deadlock. The host must count a pending
preview intent in its close busy gate while moving between the frame and this
method.

Under that one lock, preparation refuses committed close with `close_committed`,
other non-Open close contexts with `ui_stale_request`, and a nonmatching core
stamp with `stale_revision`. It then reserves one of the existing shared two
worker slots and prepares only `Command::Capture` through the existing registry.
The opaque core stamp supplies source/revision/generation identity; this adapter
does not manufacture a stamp or grant a new file/edit authority.

`UiCaptureWork` owns that originating Engine and registered work. Its
`complete_blocking(self, &Cancellation)` delegates to the same completion helper
as native File work. The permit spans actual work and Engine finish, and releases
on completion, cancellation, error or drop. It is not a Save reservation and does
not bind or acknowledge native File intent. No new thread pool or publication
framework is added.

The native host must continue counting its job after this method returns while
it decodes PNG data, constructs its image and retires/publishes the result. This
adapter does not certify current pixels: a historical PNG remains valid historical
output, and the UI must check its complete source/view/DPI key before installation.
The UI and post-command decode/channel retirement are separate ownership and
verification workstreams.

## Verified scoped behavior

The five new public integration tests verify registration before a thread starts;
the shared close busy gate and dropped-work release; real background completion;
two slots shared by executor clones and ordinary registered commands; invalid
preparation and actual-render failures; foreign/stale/A→B→A stamps; all close
contexts; early cancellation; and no authored/session mutation.

The complete-source case uses the existing original mixed V2000/V3000 SDF,
prepares a concurrent Save, changes one field, and later replaces the source
session after deleting only its test input file. Captured historical PNGs must
remain identical while current state stays unchanged. The actual saved source
must pass the unchanged external per-MOL consumers and full-byte comparison.
Accepted native File intent must remain unacknowledged, and SMILES source files
must not gain drawing capture authority.

A new private unit test splits a real returned `UiCaptureWork` only inside the
test to expose the complete→finish boundary. It cancels after actual completion
and checks the existing shared finish cancellation guard deterministically,
without a timing sleep or production hook. This is not a claim of an externally
observed thread interleaving. The public completion method delegates to the same
existing registered completion helper.

## Actual RED/GREEN and retained failures

On supplied base `a5993e0`,
`cargo test -p bondline-mcp --test ui_capture_work --locked` exited 101
(session 7478): `UiCaptureWork`, `prepare_ui_capture` and `capture_stamp` were
missing. This is compiler RED for the absent APIs, not runtime RED for an existing
implementation or a simulated stub. Log
`target/preview-registered-work-api-red.log`: 16,876 bytes, SHA-256
`e5f6a07147ad66fff1f3be50b60d1e8f8854451fdc6fd94974f6ffc067582e22`.
Additional assertions and concise PNG failure diagnostics were added before the
draft freeze; no existing expectations were weakened.

Cargo was then held until Root ported the real core. The frozen production and
test draft first passed all five integration tests (43743) and the private unit
(25216) without corrections. The first complete MCP run (36869) passed all 48:
19 unit, 24 existing control and five new integration tests.

Clippy then rejected one new test's nested boolean as `nonminimal_bool` (2283,
exit 101). The equivalent short-circuit decision was rewritten as an explicit
`if` local before the same negative assertion. No production code or expected
behavior changed. The failed log is 5,009 bytes, SHA-256
`d11ebd8689f589bd0a17eb3602f57a335aa17d1d86be7493bc2bea038e5dd3a8`.

The repeated default parallel MCP run on that final source (66585) passed 19
unit and 23 control tests, but failed the unchanged
`shutdown_cancels_idle_clients_and_releases_the_listener` assertion at
`tests/control.rs:1305`: a connection to the just-shut address unexpectedly
succeeded. The run stopped before the new integration test binary. Its log is
5,192 bytes, SHA-256
`c0c24f336c25fd2b4fbbd864056fd08c2a83ff0311e8343c6ce2cd35c2f4aec8`.
Concurrent ephemeral-port reuse is a possible explanation, not an observed
cause. No listener/control source or assertion was changed. The exact shutdown
test then passed in isolation, and one serialized full gate passed all 48.
This does not erase or diagnose the parallel failure. Root requires its final
default workspace gate to pass and will investigate a repeated failure without
weakening the intended shutdown coverage.

| Final-source check | Actual result | Ignored log |
|---|---|---|
| Exact unchanged shutdown test | 1 pass, exit 0 | `preview-registered-work-listener-isolated.log` |
| `cargo test -p bondline-mcp --locked -- --test-threads=1` | 48 pass, exit 0; session 12913 | `preview-registered-work-serial-tests.log` |
| `cargo clippy -p bondline-mcp --all-targets --locked -- -D warnings` | Exit 0; session 60627 | `preview-registered-work-clippy-green.log` |
| `cargo fmt -p bondline-mcp -- --check` | Exit 0 | `preview-registered-work-final-fmt.log` |

Final serialized log: 5,316 bytes, SHA-256
`ed7274cf73b169f2575aa6447225f9a0eb56426d8733e0bfa50c34e6e335202f`.
First parallel 48-pass log: 5,437 bytes, SHA-256
`1783a0dd0d093ff7bedf8d8e36156ff7c20640738e2ab26ae669369ec317b6e8`.
Final clippy log: 181 bytes, SHA-256
`62c6c910e9254050ee2eed02fc21a710a14be5e9d26b06ecef7c2018eef6a9d8`.

The unchanged listener source is 29,634 bytes, SHA-256
`8aaa463d2e59f8661f20927483136e85e9ae0f294db3b2640439806e365677f1`;
the unchanged control integration test is 71,150 bytes, SHA-256
`9e299e289a9dce7bf4b3a6805a1585f72dd994760a90ec514b451927391fb9ea`.

These checks used this worktree's absolute `CARGO_TARGET_DIR`, `CARGO_INCREMENTAL=0`,
`CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0` and the existing isolated
`BONDLINE_PYTHON`. No dependency, fixture, old test body, oracle, font or approved
golden was changed by this adapter slice. The old executor test body remains
byte-identical; the late-cancellation module is appended separately. The
pre-change manifest and exact old executor test-body hash
are retained in ignored `target/preview-registered-work-before.json`.

No native GUI/input/activation is permitted while the user is using the desktop.
No Windows/Linux interactive or new hosted evidence follows; GitHub Actions stays
disabled. No Git operation was performed by this producer. No job is running.
