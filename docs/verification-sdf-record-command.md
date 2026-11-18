# Independent SDF record-removal command acceptance

This is test-only acceptance for the next M1 slice, based on source `379fed0` plus
six explicitly received production files in `bondline-sdf-record-command-tests`.
The six independent cases, warnings-denied command clippy and formatting pass. The
complete M0–M5 target remains active; native UI and cross-platform record-removal
behavior are unverified here.

The contract is the shared typed command `sdf.record.remove` with exactly
`expected_revision` and a positive source-local ordinal `record_id`. Removal produces
one complete immutable SDF candidate, retains the original backing and reindexes the
remaining ordinal records. It supplies no drawing, file publication or trusted close
acknowledgment. No new result field is required. A surviving record remains SDF.

The format basis is the existing [BIOVIA Chemistry2026 CTfile Formats](https://www.3ds.com/assets/invest/2026-01/biovia_chemistry_2026_ctfile_formats.pdf),
printed97–99, with the repository's pinned source receipt and qualifications in the
[SDF workspace plan](research/sdf-workspace-plan.md). The one-or-more-record rule
requires refusing the last record's removal. Source-preserving deletion and ordinal
reindexing are original Bondline policy; this test does not establish a reference
desktop deletion workflow or universal SDF compatibility.

## Independent inputs and declarations

No fixture, writer, parser, oracle, dependency or golden is changed. The cases use
the original redistributable vectors and fixed raw ranges:

| Source | Exact identity | Record ranges / retained MOL lengths |
| --- | --- | --- |
| `ordered-fields-v2000.sdf` |690B, SHA256 `f95838cbb55c107c649ce2eb99ac62e191f747ce6994a171e64019f8e6c53d1a` |0..424 /282B;424..690 /261B |
| `mixed-v2000-v3000.sdf` |777B, SHA256 `165e6f5d5cff37dae6315a07843d84733aa42dbbc497b2920da1702bad9526a0` |0..462 V3000 /344B;462..777 V2000 /282B |

Both possible deletions are declared for both sources. The expected result is the
unchanged opposite raw span, including complete MOL bytes, empty title, exact data
headers/DT digits, duplicate order, spaces, blank terminators and delimiter. Fresh
ordinal1 is tested against the *retained* original record's independent field list,
not against a name map or regenerated properties.

Saved output must equal the literal retained bytes. Existing `sdf_oracle.py` checks
the V2000 candidates' raw framing/ordered fields and per-record RDKit graphs;
`mol_oracle.py` checks their `CC`/`C2H6` graph. Existing `v3000_oracle.py` checks the
retained344B V3000 MOL's two neutral carbons and one Single bond. Its deliberately
unsanitized graph does not independently establish a formula. Exact-byte equality
checks the V3000 record's data, which the V2000-only raw scanner cannot consume.
The unchanged raw/per-MOL consumers use external pinned RDKit2026.09.2; its full
supplier's last-value dictionary remains unsuitable as a retention oracle.

## Observed test-first result

The first test was frozen before Root began the command implementation:

- `crates/commands/tests/sdf_record_remove.rs`:1,194B, SHA256
  `5d1cec940ea065fc6689155f16c1c500365e04ca99f2d9c721e90d45a741d219`.
- Worker82734: actual `cargo test --locked -p bondline-commands --test sdf_record_remove
  explicit_second_record_removal_keeps_the_complete_literal_first_record -- --nocapture`
  compiled and opened the complete777B SDF, then failed at runtime with
  `UnknownCommand("sdf.record.remove")`. Exit101;0 passed/1 failed. This is the intended
  RED, not a missing Rust API or compiler/setup failure.
- Log: `target/sdf-record-remove-first-runtime-red.log`; exact declaration archive:
  `target/sdf-record-remove-first-declared.rs`.

The additive six-case declaration is20,494B/SHA256
`6e8d3f7c69a574ac7c38f78a8e96dee935af9566cb3f315af3c28f59ddfe370e`.
Its first1,194B remain byte-exact. The six cases require:

1. The initial minimal literal second-record removal.
2. Exactly29 registry names, preserving28 old names; closed two-field mutating schema.
3. Four literal deletions through actual Save, external consumers, Undo/Redo and reopen;
   original/current/saved distinctions, document:null and metadata-only automatic state.
4. Atomic wrong-kind/stale/zero/missing/sole-record refusals, validating the target before
   rejecting an empty result (`not_found` versus `invalid_sdf`).
5. Off-lock candidates, reverse completion, historical record query, cancellation,
   same-byte Open ABA, foreign source ownership and accepted versus committed close.
6. A captured two-record Save publishes its own original bytes despite a later removal,
   retains the same save reservation through finish, then saves the current one-record
   SDF without fabricating UI request or close acknowledgment markers.

Root receiving35470 initially rejected two new assertions that called private
`Engine::close_state()` (E0624). The source-only correction uses the existing public
`engine.state().close`; the `Approved`/`Open` expectations and every other input and
assertion remain exact. The old20,494B declaration and Root's1,245B failure log are
archived locally. Corrected source is20,494B/SHA256
`7ce7c254e02cbf82966cd19887b355a34b1a4f3ff180a50be84538c8a829e533`,
with the original1,194B prefix unchanged. Worker3317's baseline `--no-run` exits0;
`target/sdf-record-remove-public-state-compile.log` records compilation only, not a
behavioral GREEN. No production visibility was widened.

## Actual backend and scoped GREEN

Root's exact six-path manifest `target/sdf-record-remove/backend-freeze.json`, SHA256
`ae51112fc080f92201486d9c9c203855acbdb980a7fbdd726801944ac1089ffd`,
was verified before copying. Every receiving file first matched its recorded baseline.
The received files are IO's SDF document codec; commands' `sdf_edit.rs`, `lib.rs`,
`capture.rs` and `files.rs`; and MCP's native-work whitelist `work.rs`. There is no
stub, test-specific producer or production modification by this acceptance worker.
`target/sdf-record-remove-backend-port.json` records all before/after identities.

The first actual six-case run13481 exited101 with5 passed/1 failed. Its only mismatch
was a new assertion expecting `Open` after an earlier clean close approval was invalidated
by a successful removal. Existing `ClosePolicy::state` keeps such approval as
`NeedsDecision`; `invalidate_close` marks it unapproved rather than deleting it. This
is the same existing setter/commit policy. Root approved changing only this final new
state expectation. The before-removal `Approved`, failed `complete_close`, complete
source/history/cancel checks and original1,194B prefix remain exact. This correction
addresses an unsourced test expectation, not a producer defect or production fix.
The13481 failure and its20,494B source are retained separately.

Final source is20,503B/SHA256
`98401254b1aad683da0a067316c2af1fd1a1f5e732e715ebb2212dac6cb7d7fe`.
Reversing only the two public-accessor corrections and the one close expectation
reconstructs the original20,494B six-case declaration exactly.

| Worker | Actual command | Observed result / local log |
| --- | --- | --- |
|1266 |`cargo test --locked -p bondline-commands --test sdf_record_remove -- --nocapture` |6 passed/0 failed/0 ignored; `target/sdf-record-remove-six-green.log` |
|68186 |`cargo clippy --locked -p bondline-commands --all-targets -- -D warnings` |exit0; `target/sdf-record-remove-clippy.log` |
|68186 |`cargo fmt --all -- --check` |exit0; `target/sdf-record-remove-fmt.log` |

These tests actually published four literal single-record results plus captured-old
and current saves, reopened the saved sources without rewriting them, and invoked the
unchanged external consumers described above. The supported close case commits from
an earlier approval, requires a fresh decision and cannot reuse that approval for
trusted completion. The committed-close case refuses the late candidate unchanged.

The baseline receipt `target/sdf-record-remove-baseline.json` records428 protected
inputs. Final validation requires422 unchanged baseline inputs and exactly the six
authorized production replacements above. Old tests, fixtures/oracles, goldens,
manifests, locks and dependencies remain unchanged. Root separately updates registry
count metadata and executes integration; this worker did not repeat the full command
package or claim its old28-name assertions pass against the new29-name registry.

## Reproduction and limits

Use this checkout's absolute `target`, `CARGO_INCREMENTAL=0`, dev/test debug0 and the
existing pinned `BONDLINE_PYTHON` interpreter. The scoped command is
`cargo test --locked -p bondline-commands --test sdf_record_remove -- --nocapture`.
Warnings-denied all-target command clippy and formatting follow actual focused GREEN.

All owned jobs are terminal. The final hash/source/log/protected-input receipt is
`target/sdf-record-remove-handoff.json`; earlier declaration, compile and runtime
failures remain available as historical evidence.

No record browser/delete UI, native execution, adapter-specific wire acceptance,
record insertion/reorder, persisted record IDs, chemistry calculation or complete M1
support is established by this isolated test slice. Root owns production, integration,
global status and Git. GitHub Actions and native input remain unused.
