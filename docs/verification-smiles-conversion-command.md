# Shared create-only SMILES conversion

2026-11-14. Verified headless command slice in the supplied isolated
`bondline-mol-v3000` checkout, branch `feat/smiles-conversion-command`, initial base
`c2b739a`, with Root's real reviewed IO converter received at `6a73107`. Root owns
Git/integration and the receiving aggregate.
No GUI/input/activation, installation, dependency/lock, model, notation/layout, renderer,
approved golden or workflow-setting change is made by this command slice. GitHub Actions
remains disabled by user instruction; verification here is local only.

## Contract under test

One registry command `file.convert_smiles_to_cdxml` has eight required closed parameters:
expected revision, explicit input/output paths and explicit length/origin/page geometry.
Runtime validation covers finite direct typed Rust numbers as well as JSON schemas.
Its name explicitly selects the restricted `neutral_carbon_path_v1` policy: supported
bare/bracket H notation is normalized into fixed authored counts under the existing
closed profile. This adds no general/live H inference, default geometry, overwrite
option or source-backed SMILES Open.
Its discovery/MCP read-only flag is false because it writes disk.

The command calls the separately implemented and reviewed
`io::files::convert_smiles_to_cdxml_file`. No command-side parser, layout, writer,
filesystem publisher or backend stub is added. The native operation owns bounded input,
validates the complete notation, lays out its accepted1–12 carbon/Single path and creates
new native CDXML through the existing atomic publisher. Input bytes/mode stay intact.

`FilePlan::ConvertSmiles` owns validated geometry, captured revision/generation and the
same per-Engine save reservation as native Save. `Completed::Conversion` retains it
through publication until finish/drop. Finish returns the latest Engine state plus an
optional historical conversion receipt and performs no authoring, saved-baseline, source,
history, association, dirty, close, native-intent or trusted-ack mutation. A later Open or
committed close cannot erase a real published receipt or be relabeled by one. Source input
is read during worker completion; input revision identifies Engine preparation time.

The UI file binder is explicitly fallible: conversion and nonfile plans cannot silently
accept a native Open/Save ID. Accepted native selection rejects conversion before acquiring
a lease. The same two workers and reply/close publication barriers serve all interfaces.
No native conversion control, source-backed application SMILES format or arbitrary layout
is delivered by this headless command.

## Independent expectations

The first behavioral test supplies original complete `CCC\r\n` bytes, expects actual new
native XML metadata/counts and unchanged complete Engine/input. Its actual output faces
the unchanged `smiles_layout_oracle.py` Expat/raw Decimal/40-digit analytic L48/origin72/
RDKit consumer, with independent literal coordinates and C3H8/count expectations.
It uses a Unicode/misleading output suffix, with the explicitly named converter remaining
authoritative. A separate explicit L40/origin(100,200)/Letter case correctly exercises
the unchanged legacy fixed-ethane file oracle.

Additional tests specify required/closed schemas and direct nonfinite/invalid numbers;
whole native and mixed SDF source/history/redo/close preservation; native Save/conversion
lease in both directions, including published/drop/finish/error/cancel states; stale and
unsupported complete input without output; historical identity and latest state after
Open/committed close; refused picker binding; and current versus permanently stale trusted
Save acknowledgments after an Undo ABA. No new test substitutes an empty/first SDF drawing.

Actual Engine, CLI, stdio MCP and authenticated loopback workflows convert `CC\n`, explicitly
Open the output, extend terminal C-H3 under the required H3→H2 policy, Undo, Redo, Save and
reopen. Independent literal state checks require stable IDs, positions(72,72)/(120,72)/
(160,112), H3/H2/H3 and Single adjacency. An original external-only Expat/Decimal/RDKit
consumer checks actual saved XML digits, references, H and canonical external molecular
meaning/formula for the2/3-atom cases. Existing oracles/pins are not changed.

MCP unit tests hold both worker permits, prove rejection releases an unexecuted conversion
lease, cancel before work, and check cross-executor Engine reservations. A gated response
flush proves close stays blocked after worker completion until the receipt is delivered,
without repaint or session changes. These are headless adapter/lifecycle tests, not native
UI or reference-product evidence.

## Actual evidence so far

All Cargo calls use this checkout's absolute target, incremental0, dev/test debug0 and
Root's pinned external interpreter via BONDLINE_PYTHON. No artifacts are shared across
worktrees. The first runtime test command was:

```bash
cargo test --locked -p bondline-commands --test smiles_conversion
```

RED83957 exits101:0passed/1failed with
`UnknownCommand("file.convert_smiles_to_cdxml")`. The original
`target/smiles-conversion-command-red.log` is retained. This failure occurred before the
registry/plans/backend wiring. No backend stub or duplicated IO was used to make it fail.

The real IO source was then received from Root; no stub was compiled. Scoped commands
and actual adapters run through this one API. Full commands and adapter checks used:

```bash
cargo test --locked -p bondline-commands -p bondline-mcp -p bondline-cli
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all -- --check
```

- First real compile/focused56362 exits101:6passed/3failed. Those three new tests used
  the old consumer tied to H3-only ethane at(100,200)/(140,200), incompatible with the
  actual L48/origin72 converted fixtures and C3 interior H2. This was a test-vector/
  oracle mismatch, not a production defect or new behavioral RED. The log remains
  `target/smiles-conversion-command-focused.log`. Production and that old oracle were
  unchanged; only new tests selected the correct independent consumers described above.
- Focused GREEN86039 exits0:10passed/0failed/0ignored. New oracle self-checks take actual
  generated bytes and require rejection of changed coordinate, literal H, bond order
  and appended NUL, while the unchanged actual output passes before/after. Log
  `target/smiles-conversion-command-focused-green.log`,1295B/SHA256
  `d8e3f15ef54d90d6c81e082d0603e809fc1f45448be05b4f4e86e9bd83d8ee43`.
- Initial scoped62286 exits101 on an existing exact SDF registry set expecting20 names.
  CLI/stdio conversion workflows had passed. All three old SDF exact name fixtures
  then received only the new command's literal entry; every old name and other
  assertion remains. This routine registry-fixture migration is recorded separately
  from production behavior. `target/smiles-conversion-command-adapters.log` is retained.
- Scoped GREEN44077 exits0:219passed/0failed/0ignored across28 suites. All old/new
  commands, actual CLI/stdio/authenticated loopback workflows and the two new worker/
  response-flush guards pass. Log `target/smiles-conversion-command-adapters-green.log`,
 25125B/SHA256 `1cdbff3f5d497916325143b0846d3d85a09b5bb7fe725ef01ab930d5278678be`.
- Workspace all-target warnings-denied clippy and workspace fmt check87218 both exit0;
  `target/smiles-conversion-command-clippy.log` and
  `target/smiles-conversion-command-fmt.log`. The former also compiles the unchanged
  thin UI against the new optional result field and command variant. No UI source changed.

There are16 new tests:10 public command/actual-output cases,1 private fallible-binder
case,1 CLI,1 stdio,1 authenticated loopback, and2 MCP worker/reply cases. Registry
discovery is exactly21 commands with all20 prior names retained. Optional `conversion`
is omitted from ordinary inspect/state and all old response fields remain meaningful.
No production correction was needed after the first real compile.

The final ignored handoff identifies every intentional source/test/doc/log and protected
hash, including unchanged received IO. Root's independent review/receiving full gate is
separate from this owner's219-test affected-package gate. No full-workspace test/fuzz,
new Windows/Linux execution, native conversion UI or interactive evidence is claimed
here. No GUI/input/activation or Actions operation ran; all own Cargo jobs are terminal.

Independent source review reports no confirmed production/oracle defect. Root's ignored
`target/smiles-conversion-command-peer-review.md` is13691B/SHA256
`6309f7ba557c375d5b075995de5ce20a3edb1eb333cc7de4aa2a4e009e015428`.
It keeps the three registry-fixture changes explicit and separates source review from
the actual scoped executions above. No source changes were made during that review.
