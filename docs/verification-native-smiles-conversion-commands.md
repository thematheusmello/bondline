# Native SMILES conversion: independent core and worker acceptance

Test-only work from `ac11ce6` in the isolated
`bondline-native-conversion-command-tests` checkout. Root owns implementation,
integration, Git and native input. This record does not establish a native workflow or
general SMILES drawing import. Actions remains disabled.

## Declared contract

The existing `file.convert_smiles_to_cdxml` command remains the only conversion engine.
The new central File control requests `ui.file` action `convert_smiles`; claiming it
returns a conversion presentation plan. No shortcut or new command is introduced.
`Engine::prepare_ui_smiles_conversion(id, params)` requires its exact current Accepted
intent. Success consumes only that intent, retaining its epoch. Failure retains it.

The native preparation adapter must reserve the existing shared two-worker budget under
the originating Engine lock before invoking core preparation. Full budget cannot consume
the accepted intent. Its work retains that Engine, the registered work and the existing
Save reservation; it completes through the same real publisher and shared finish path.
No native Save request binding, Running phase, trusted Save/close acknowledgment,
automatic Open, saved baseline or source/edit authority follows from conversion.

The approved contract is copied unchanged under the ignored test receipts, with its
source identity in `minimal-freeze.json`. Existing source decisions and restricted
conversion scope are documented in
[the command verification](verification-smiles-conversion-command.md) and
[IO verification](verification-smiles-conversion-io.md). These tests add no parser,
layout, writer, filesystem helper or product API.

## Independent expectations

Six command cases declare:

- A dynamically dispatched native intent and the18th central control, with the existing
 30-command registry retained and no authored mutation.
- Conversion while an original777-byte mixed-version SDF has an unsaved occurrence edit:
 complete current content, source stamp, source file, history, dirty baseline and file
 association remain exact. A Unicode/misleading destination contains real new CDXML.
 The unchanged external Expat/Decimal/RDKit consumer checks two neutral carbons, literal
 H3/H3, Single adjacency and positions(72,72)/(120,72) from the declared48-point profile.
- Unclaimed/wrong-ID/wrong-action, stale revision, invalid geometry and busy Save
 preparation retain the exact intent. Generic native Open/Save binding still refuses
 conversion. A successful preparation followed by Drop releases ownership without ACK.
- Accepted context cannot survive edit→Undo, close→Cancel, New, host detach/reattach,
 Cancel or a direct Save association change. Refusals leave source/state exact.
- A conversion accepted before New can publish a truthful historical result without
 cleaning or associating the new dirty Drawing. Its Save lease survives publication
 until finish or Drop, independently of receipt consumption.
- Unsupported input, pre-cancellation and an existing output never overwrite files or
 save/replace the active session. Existing errors stay typed and ownership releases.

Two public MCP preparation cases declare full-budget-before-consumption, shared cloned
executor activity, unchanged generic binding refusal, origin retention through New,
repeated failed prepares without permit leakage, Drop/cancellation and a real successful
retry. They use two actual registered capture reservations to occupy the existing budget;
no private semaphore hook or replacement pool is introduced. The existing private binder
tests remain protected; no production visibility is widened to call them.

## Actual test-first evidence

All Cargo commands use this checkout's absolute target, incremental0, dev/test debug0
and the literal pinned Root `BONDLINE_PYTHON` interpreter. No targets are shared.

1. The first1,510-byte declaration uses only existing Engine APIs and dynamic dispatch.
 Its SHA256 is
 `f9378d123f6c7ccdc7f22d09e541ce013645f199fe0e198ae495cac7675617c4`.
 `cargo test --locked -p bondline-commands --test native_smiles_conversion
 native_conversion_intent_and_control_preserve_the_current_session -- --exact`
 runs worker26109, exits101 with0passed/1failed: existing `UiFileAction` rejects
 `convert_smiles` as `InvalidParams`. This is the intended runtime RED, not a missing
 Rust method or fake backend. The4,985-byte log SHA256 is
 `09b46c744d2773639c35cf1680f8b4cfbd70c9b0d38416f31aa49e4720e57d15`.
2. The six/two-case extension preserves that complete first prefix. Before receiving
 production, `cargo test --locked -p bondline-commands --test native_smiles_conversion
 --no-run` exits101 with sixE0599 errors for the agreed missing native conversion
 method/presentation variant. This compiler stage is separate from the runtime RED.
3. Worker47371 runs the public MCP test target with `--no-run` and exits101 with
 sixE0599 errors only for the agreed missing executor method. This is also an API
 compiler stage, not a second runtime behavior failure. Both declaration files remain
 byte-exact against `extended-freeze.json`; `missing-api-receipt.json` records the logs.

## Actual backend and GREEN evidence

Root authorized exactly four production inputs from the producer's838-byte manifest,
SHA256 `3513bf892211781b8b7170fb42d73cfb1c3647e7610ed444a19022e5acf39094`.
Every producer hash and each destination's original baseline was checked before copying;
the received hashes and both declaration files were checked afterward. No UI,
catalogue-test reconciliation or unannounced later source was copied. The ignored
`backend-port.json` retains the exact four before/after identities.

All declared tests pass on their first real implementation run, without changing any
test input, expected value or producer source:

| Run | Actual result | Log |
| --- | --- | --- |
| Commands42292, `cargo test --locked -p bondline-commands --test native_smiles_conversion` | exit0;6passed/0failed/0ignored | `core-first-real.log` |
| MCP82578, `cargo test --locked -p bondline-mcp --test ui_smiles_conversion` | exit0;2passed/0failed/0ignored | `worker-first-real.log` |
| Clippy33929, `cargo clippy --locked -p bondline-commands -p bondline-mcp --all-targets -- -D warnings` | exit0 | `clippy-first.log` |
| Format95050, `cargo fmt --all -- --check` | exit0 | `fmt-first.log` |

Every actual conversion file reaches the unchanged pinned external consumer. The tests
also compare receipt byte length to the complete published output, preserve the complete
unrelated SDF/input bytes, and require exact serialized current state after removing only
the explicit conversion receipt. Successful publication remains historical after New;
it cannot become a native Save receipt or alter the new baseline.

The bounded source cross-check covers the same four received files: core preparation
validates the current Accepted conversion action before calling the existing command,
consuming the intent only after success; the generic file binder stays unchanged. The
executor holds the originating lock while reserving the existing permit and preparing,
retains that Engine and reuses `complete_registered_blocking`. Existing completion still
finishes the shared conversion plan and releases its Save/worker ownership through normal
RAII. No additional material issue was found in those reviewed paths. This is not proof
of every race, poisoned-lock publication timing, thread-spawn failure or host behavior.

The first1,510-byte declaration is still an exact prefix. Both extended declarations are
byte-exact against their pre-implementation freeze. The final ignored handoff records
all logs and242 original protected source/configuration inputs:238 unchanged and only
the four authorized producer replacements. Original fixtures, consumers, approved golden,
dependencies/locks and old tests remain unchanged. The shared test helper/consumer is
reused, never rewritten. There is no native, full-workspace, fuzz or other-platform
execution claim; the native form/App workflow belongs to its separate owners. All own
jobs are terminal, and no Git or Actions operation was performed.
