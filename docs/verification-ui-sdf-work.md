# Registered native SDF work acceptance

Independent final focused execution passes7/7 on the exact frozen source. Relevant
MCP all-target warnings-denied clippy and workspace formatting exit0. No test, input,
expectation or tolerance correction was needed after the real implementation arrived.
This is adapter evidence, not a delivered native field editor or native-window workflow.

2026-11-15, supplied base `9fc6944`. Tests precede implementation of a narrow native
adapter for the existing SDF query and field command. This owner changes only a new
integration test and this record, with ignored evidence. No native interaction, Git,
Actions, installation, dependency, original fixture/oracle/golden or production edit.

## Locked boundary

`CommandExecutor::prepare_ui_sdf(&CaptureStamp, captured_close_epoch:u64, Command)`
returns `UiSdfWork`, whose `complete_blocking(&Cancellation)` uses the existing Engine
prepare/finish lifecycle. Only `SdfRecordInspect` and `SdfFieldSet` are accepted; this
registers no new command or version. Preparation checks originating complete-source
identity and Open close context/epoch under the same Engine mutex, reserves the shared
two-worker budget before spawning, then prepares the existing command. Call after
releasing the UI frame's Engine lock.

The host must retain its pending/job/publication barrier and independently guard any
installed response against current selection/source context. An accepted query can
return historical selected fields. An accepted earlier field edit retains existing
core authority to commit before native close acknowledgement and invalidate approval.
This adapter neither grants a Save lease nor binds/acknowledges a native file request.

Agreed refusal codes: wrong command family `invalid_params`; wrong content kind
`unsupported_content`; foreign/stale stamp `stale_revision`; stale close epoch or
NeedsDecision/Approved `ui_stale_request`; Committed `close_committed`; full shared
worker budget `busy`. Tests isolate invalid conditions except the explicitly specified
busy-before-command-preparation case.

## Independent expectations

The original777-byte mixed SDF is SHA-256
`165e6f5d5cff37dae6315a07843d84733aa42dbbc497b2920da1702bad9526a0`.
Existing `sdf_record_test_support.rs` supplies unchanged literal ordered duplicate,
empty, combined/DT-only and spaces-value query expectations. The target is occurrence2
of record1, the second DUP; no name/property dictionary selects it.

The predeclared replacement changes only source `[378,385)` (`second` plus LF) to
`reviewed\nline two\n`. Expected complete output is788 bytes, SHA-256
`e63cb9adf5cbc62b5b0be94992246c0d10b94cda405e49b9514993e7d9f9c361`.
Full-byte equality is independent of the production parser/writer. A pinned external
Python SHA/raw-span check and unchanged actual MOL consumers supplement it: V3000
`[0,344)`, shifted V2000 `[473,755)`. The old V2000-only raw oracle is not misused for
mixed data, and RDKit's duplicate/DT-only full-supplier limitations are not preservation
proof. The preimplementation literal-vector oracle itself exited0; this is vector
validation, not a UI-work GREEN.

Seven frozen cases cover:

- Ordered explicit query and reservation visible before spawn, without retaining the
  Engine mutex or changing ordinary state/source.
- Literal second-DUP edit, exact source/metadata/history, no-op byte identity and
  CaptureStamp, Undo/Redo and independently checked actual saved output.
- Shared slots with real capture, busy-before-work, drop, validation/completion error
  and pre-cancellation release for both supported families.
- Wrong-family immediate-command safety, source identity, Drawing/SMILES wrong kind,
  Request→Cancel close-epoch changes and every non-Open close state.
- Host close blocking before spawn, query/drop retirement and accepted-earlier edit
  invalidation of clean close approval.
- Complete historical selected-record data after original path removal/content
  replacement, with stale native field work unable to apply to the replacement.
- No Save lease and no native file picker binding or successful completion ACK.

Late candidate cancellation and core stale-finish tests remain unchanged in the command
crate. This new test does not use timing sleeps/hooks to pretend it observed that internal
phase, and it does not duplicate all existing command/parser/resource assertions.

## TDD checkpoint

Initial test3,027 bytes/SHA-256
`769fb399ece51242249e019caf5d9a4814ddd4e67eb49c4db3d24f4118853990`
was frozen before production work. Independent session92804:

```sh
cargo test -p bondline-mcp --test ui_sdf_work --locked
```

exited101 with exactly E0599, missing `CommandExecutor::prepare_ui_sdf`. This is the
intended API/compiler RED, not a runtime or native failure. Log:
`target/ui-sdf-work/initial-red.log`. There was no stub or fabricated result.

The full additive seven-test source is20,766 bytes/SHA-256
`1b113da2dc7912f031db88a217d56c0bab23734fe397ff4b05037792a58ab78c`;
its initial3,027-byte prefix is exact. It was frozen before Root implementation.
`target/ui-sdf-work/full-freeze.json` and the initial source/log retain this ordering.
The independent focused GREEN on the real two-file MCP implementation is recorded below.
No full workspace/native pass is claimed. All current owner jobs are terminal.

## Actual unchanged GREEN and received source

Root separately observed full-suite missing-method/type compiler RED54259. Its first
implementation81894 failed E0624 for a private Engine method; only Root production was
corrected to use a public state snapshot. Neither is a runtime result or a test-setup
correction. Root's next40265 passes the exact seven. This owner's first full focused
execution after receipt also passes unchanged.

`target/ui-sdf-work/backend-port.json` is693 bytes/SHA-256
`62f6b6183ef7e3e113d23472ec5935cae88967acbd20d70845c7a5a68df8306b`.
Exact received Root-owned files:

| File | SHA-256 |
|---|---|
|`crates/mcp/src/work.rs`|`2402950d3a282f97f50a80f9955265fe34a53b477efa445f6b96637109b49791`|
|`crates/mcp/src/lib.rs`|`4e9a1107e580a1dc8987905da45a0610c64077814c9fa6ce171e37071ccedf55`|

All838 old source identities were classified:836 remain byte-exact and two are these
explicit received replacements whose old hashes match the baseline. The prior native
acceptance plan, fixtures, original six preview tests, oracles, fonts, locks and approved
golden remain exact. This owner made no production edit.

| Session | Command | Actual result | Ignored log |
|---|---|---|---|
|80166|`cargo test -p bondline-mcp --test ui_sdf_work --locked`|7 passed,0 failed,0 ignored; exit0|`target/ui-sdf-work/focused-green.log`|
|57326|`cargo clippy -p bondline-mcp --all-targets --locked -- -D warnings`|exit0|`target/ui-sdf-work/clippy.log`|
|2026|`cargo fmt --all -- --check`|exit0|`target/ui-sdf-work/fmt.log`|

The scoped run used the worktree's own absolute `target/ui-sdf-work-build`, incremental0,
dev/test debug0 and the existing Root pinned `BONDLINE_PYTHON`. Actual788-byte saved
output, whole-source bytes/ordered metadata, independent raw SHA and both unchanged MOL
consumers passed. No runtime fixture, assertion, source profile or limit was weakened.
No unrelated UI/workspace suite was repeated. Root owns aggregate integration and any
future UI/native validation. Final source/protected/log identities are recorded in
`target/ui-sdf-work/final-handoff.json`; all own jobs are terminal.

## Receiving-root integration

Root session98860 `CARGO_INCREMENTAL=0 cargo xtask ci` exits0:968 passed,0 failed,
1 ignored across147 result lines, including the3 macOS AccessKit bridge tests. Formatting,
all-target warnings-denied clippy, layers, licenses and pinned corpus pass. Log
`target/ui-sdf-work/root-ci.log` is116,607 bytes/SHA-256
`daf2650e0e1e5eff916117a5cb8e30517d0133ab9cbb957f6d92f4bab14706f3`.
The same154/165/220 native runtime declarations and2 assets remain allowed; no dependency,
lock, font, fixture, oracle, approved golden, command count or wire schema changed.

A separate read-only peer review found no confirmed bridge defect; source/close guards,
shared permit lifetime, cancellation, history and Save authority were checked. The ignored
report SHA-256 is`f695ef724c1f43fb8c48684c25181359712984ba35aef3a2ac158a69c011a1c8`.
Future UI installation, drafts and actual thread retirement remain separately unverified.
No hosted/Windows/Linux interaction or native field workflow is claimed. Actions stays off.
