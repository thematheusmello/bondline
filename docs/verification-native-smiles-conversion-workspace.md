# Restricted native SMILES conversion workspace

2026-11-15. Isolated base `ac11ce6`, with one shared command registry and the existing
restricted `file.convert_smiles_to_cdxml` implementation. This is an original File-menu
conversion flow, not a general SMILES drawing import or reference-product screen-parity
claim. No native window or operating-system input was used by this worker. Root owns
integration, native acceptance, adapter catalogue reconciliation and Git.

## Delivered behavior and boundaries

The central `file.convert_smiles` control presents **Convert SMILES file…**, without a
shortcut. Its transient `ui.file` action accepts one explicit form, the existing chooser
for input/output selection, and one registered background conversion. The form declares
the supported connected chain of 1–12 neutral carbon atoms with Single bonds, unchanged
current document, and create-only CDXML destination. It supplies all eight existing command
parameters: captured revision, input/output paths, bond length, origin and page dimensions.
Defaults 48/72/72/A4 are original application policy. Each text field is byte-bounded; paths
are nonempty UTF-8 without NUL and at most 32,768 bytes. Numeric fields are checked for finite
values on explicit Convert. File/SMILES parsing and publication remain in the actual worker.

The specialized Engine/executor preparation validates the current Accepted conversion
intent under the originating mutex and reserves the existing worker budget before consuming
that intent. Failure leaves it addressable. It calls the existing registered conversion,
retains the existing save reservation, and creates neither a native Save binding nor trusted
Save/close acknowledgment. The generic binder and `prepare_ui_file` refusal assertions are
unchanged. Conversion never opens the output automatically or changes current/saved source,
authored history, association or dirty state. Retry is explicit and qualified again.

FileWorkflow keeps the actual JoinHandle/result until thread retirement. Cancellation,
source replacement, close and poisoned-engine cleanup retain a published success receipt,
including its actual path/counts and historical/cancelled qualification. Unstarted obsolete
forms are dismissed with text-focus cleanup, exposing the replacement document or close
decision. Running/published work retains its owner and truthfully reports its result.
Destroying FileWorkflow cancels a conversion job without blocking on join.

## Test-first and correction record

Evidence is retained under this worktree's ignored `target/native-smiles-conversion/`.
Independent test owners preserve their corresponding baseline receipts; their outcomes
below are attribution, not additional local runs.

| Stage | Actual result and qualification |
|---|---|
| Independent core 26109 / App 86758 | Runtime RED: unknown `convert_smiles` action / missing actionable File-menu control. No future Rust API was needed. |
| Independent expanded core 3593 / MCP 47371 | Missing-specialized-API compiler RED, separate from the runtime failures. |
| Own 13402 | Missing UI queue/start integration compiler RED. |
| Own 60967 | Producer compiler error: moved field-error value plus unused import; corrected before the first successful UI compile. |
| Own 1720 | Initial queued-work/source-replacement lifecycle test GREEN; stale full SDF cannot spawn or publish. |
| Independent 42292 /82578 | Six core and two public executor cases GREEN on exact four-file backend, without expectation correction. |
| Own 22127 / independent 9075 | First five actual-App cases: 1 pass/4 fail at immediate inline-text lookup; conversion had not run in those failures. |
| Own 7988 | After new-field AX-before-layout/repaint producer fix: 3 pass/2 fail, old chooser inline-text timing and obsolete-form disappearance. |
| Own 38967 | Unchanged obsolete-source/close App case GREEN after the explicitly adopted unstarted-form cleanup. |
| Independent metadata 65111 | Existing chooser parent AX value changes immediately; painted inline value follows repaint. This distinguishes the old chooser timing from the new-field producer correction. |
| Own 13231 | Approved helper-only test correction: immediate parent-value assertion plus bounded painted-value wait, retaining the original final inline assertion. Five actual-App cases GREEN. |
| Own 98209 | Four finite actual-publication/thread-retirement checks GREEN on their first run: additional coverage after implementation, not a RED claim. |
| Independent 19237 | Actual Drop RED: destroying the owner before I/O did not cancel real registered work, which then created the output. |
| Own 63728 | First narrow Drop correction failed compilation at the private child-field boundary. No assertion changed. |
| Own 16744 | Child cancellation helper plus existing FileWorkflow Drop call: unchanged independent Drop case and all four publication gates GREEN. |
| Root 51446 | Actual recovery RED: a successful explicit retry created independently valid output but retained the earlier existing-destination error as the current inspector banner. |
| Own 84596 | Clear the prior global message only when explicit valid Convert creates pending work; unchanged original five plus Root's recovery case GREEN. No finish-time clearing of unrelated later errors. |
| Independent 3958 | Remote-cancel candidate GREEN on its first execution against the held implementation. No input-gating correction is justified by that case. |
| Own 69907 | All seven actual-App cases and six finite lifecycle cases GREEN together: 13 passed. |

Independent App source lineage is preserved: 26,224 bytes/SHA256
`596b0597908d714de96ac30133c05a6d8f7f6dbb42f8d432c77b8bdf09221746`
to the approved 26,514-byte helper correction/SHA256
`9b9af85d528691a338697554fdd7591d4a8da6d6dca1ac4c0b2fe896f60c20de`.
Removing that one helper insertion reconstructs the prior source. The original final inline
assertion, test actions, state/history/source assertions and initial 2,067-byte test prefix
remain. No old fixture, consumer, golden or product behavior was changed for that timing
oracle correction.

The subsequent independent sixth preserves that entire 26,514-byte prefix. The final
merged parent is 30,295 bytes/SHA256
`763e7762ab810371af6df109c10befbbe1ee9b6b9b25f5a20a63fea1f23aca7a`:
the exact 30,232-byte six-case source followed by Root's 63-byte recovery child declaration.
The separate recovery test is 1,820 bytes/SHA256
`e96aa5c6e8591385845fa105c481607d63dca9408ce62f8896a34e0a0d502a47`.
No existing case's input or assertion changed during that merge.

## Decisive checks

The five actual-App cases exercise central discovery/form, Unicode input/output browsing,
default and explicit 40pt/(100,200)/Letter geometry, dirty native-source/history preservation,
actual output interoperability, explicit ordinary Open/edit/Undo/Save/reopen, unsupported
`CC=O`, existing-destination refusal, nonfinite geometry, explicit retry, numeric 1/2/3 text
entry, modal Convert/Cancel/Close input packets, and stale/unstarted source/close refusal.
The additive remote-cancel case requires the first old-modal packet to leave the authored
document/output/tool unchanged, then permits fresh next-frame authoring. The recovery case
requires independently valid actual output and no old failure banner after explicit retry.

Finite gates run the real executor conversion, send its actual result, then hold the worker
thread before retirement. They require busy/close barriers to persist, no fabricated Save
ACK, exact 788-byte current SDF formed by the independently declared 777-byte-source splice,
unchanged original source and history, and truthful late-cancel/New/close/poison outcomes.
The unchanged external `io/tests/smiles_layout_oracle.py` consumes actual new CDXML. Its
Expat/raw Decimal/analytic/RDKit checks require the literal three-atom coordinates,
Single-bond carbon graph, C3H8 and counts. There are no synthetic success receipts or new
golden images. The independent Drop gate releases real registered work before asserting its
cancelled completion and absent output.

The initial own lifecycle case remains unchanged at 2,556 bytes/SHA256
`227957b6f1169a16b06f1ab18bb00d834e6a007d0691229925a8542ad8c2d33f`;
four additional finite gates are 12,610 bytes/SHA256
`2d13779568ec0f75289b814771e221c444ce70052d5c28e974d9021a83f10fbf`;
the independent Drop case is 4,251 bytes/SHA256
`f2d2ccea24ae257a0d8b549da52c0f0fb8e976b1fc5221e2fc52e835c5f2b67a`.

## Checks and remaining evidence

All Cargo calls use this checkout's own absolute target, incremental 0, dev/test debug 0 and
the literal pinned Root interpreter through `BONDLINE_PYTHON`. No outputs are shared.

```bash
cargo test -p bondline-ui --lib view_tests::native_conversion --locked
cargo test -p bondline-ui --lib file_workflow::smiles_conversion --locked
cargo test -p bondline-ui --lib conversion --locked
cargo test -p bondline-ui --locked
cargo clippy -p bondline-ui -p bondline-commands -p bondline-mcp --all-targets --locked -- -D warnings
cargo fmt --all -- --check
```

Warnings-denied relevant all-target clippy 85674 and workspace fmt 32204 exit 0. First default
full UI package 90164 exits 101: 170 passed/1 failed/1 ignored; the unchanged SMILES record
Unicode Save/reopen case times out without an application error. The same unchanged case
passes isolated in 0.14s. Its cause is not established and neither assertion nor timeout was
changed. The failed package log remains evidence; a later receiving gate is separate.

The final unchanged default-parallel UI package 11963 exits 0: 223 passed/0 failed/1 ignored
across nine suites (library 173/0/1 plus 50 integration tests). It includes the previously
timed-out case. Final relevant all-target warnings-denied clippy 5265 and workspace fmt 58807
both exit 0. These later successes do not relabel the earlier failure or establish its cause.
The ignored test is the existing explicit offscreen application-capture utility.

The held `ui-fixed-source-freeze.json` records all nine product inputs, five test/bridge
inputs, 897 unchanged baseline files and the exact nine changed baseline paths. Removing the
test-only parent bridge reconstructs its baseline exactly. Removing the Drop call/helper
and 85-byte test suffix reconstructs the pre-Drop files. Dependencies, locks, old fixtures,
consumers and approved golden are unchanged. The later `recovery-fix-source-freeze.json`
records the single product correction, and `combined-final-inputs.json` records the exact
merged acceptance source. The final handoff manifest pins the resulting inputs, logs and
new verification document. Root's aggregate and native checks are separate receiving work.

The independent remote-cancel candidate first passes unchanged on the held implementation;
it establishes no additional producer defect or RED. Native application execution, native chooser
interaction and cross-platform chemical workflows remain unverified by this worker.
GitHub Actions remain disabled; this record claims local headless/egui checks only.
