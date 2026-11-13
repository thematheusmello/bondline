# Verification log

2026-11-13, bootstrap documentation: no application checks yet. Starting workspace was empty.
Rust 1.94.1 is installed. Public specifications and reference license practices were inspected.
Future behavioral entries must identify the RED command/failure, GREEN command/result,
independent reference, relevant environment and remaining limitations. No invented passes.

## Graphics license boundary (in progress)

- RED `cargo test -p xtask`: missing allowlist implementation. GREEN: SPDX OR/AND, legacy
  spelling and target-tree parsing tests pass. RED `cargo xtask licenses`: real forbidden
  native dependencies. Selected-target Cargo trees replaced metadata's feature overapproximation.
- RED `cargo test -p hexf-parse -p libloading`: missing parser/loader entry points. GREEN:
  IEEE-754 bit vectors, inexact/overflow/underflow rejection, system PID vs std::process::id,
  missing library/symbol and invalid type/name tests pass. Raw-symbol ownership test also
  observed RED before implementation.
- RED `cargo test -p clipboard-win`: missing UTF-16/CF_HTML validators. GREEN: known Unicode,
  malformed UTF-16, termination, exact fragment and out-of-range tests pass on macOS.
  This verifies portable parsing, not native Windows clipboard execution.
- Native declared license closure passes all three desktop targets after rejecting gpu-descriptor
  0.3.1 (still foldhash), then pinning 0.3.0. The hypothetical browser graph is not approved.
- Native macOS GPU readback matched independently specified RGBA bytes: Apple M3 Max,
  Metal, IntegratedGpu. This tests clear/readback, not chemical rendering or complete parity.
- UI startup configuration and font/page paint tests observed RED for missing functions,
  then passed. Native app launch and visual inspection are still pending at this entry.

Follow-up evidence:
- Native `cargo run -p bondline-ui --bin bondline -- --smoke-test` exited 0 after the real
  window rendered three frames; adapter JSON: Apple M3 Max, Metal, IntegratedGpu.
- Actual macOS development bundle was opened through native computer use. Observed title,
  readable bundled font, white portrait page and Page 1/A4 status. This was AI visual
  inspection of a read-only prototype; no human-approved golden or drawing workflow claim.
- GUI-free render crate: CPU expected pixels, invalid dimensions and GPU readback agree
  exactly for the tested empty canvas. No chemical primitives are implemented yet.
- Shader parser 64 KiB bound observed RED (oversized zero literal accepted), then GREEN.
  Expanded independent Python float.hex/struct vectors: 128 binary32 and 128 binary64 pass.
- Asset gate observed RED for a missing verifier, then GREEN for known SHA-256, changed bytes,
  disallowed licenses and confined paths. Real font hash and notice presence pass.
## Graphics checkpoint follow-up, 2026-11-13

Workspace tests passed (18 tests, including the 256-case IEEE vector test and actual Metal
readback). Native runtime/asset license gate passed: macOS 109, Windows 120, Linux 187
runtime packages. Whole-workspace Linux source check passed after extracting the render
crate. The DX12-enabled Windows check exposed the safe `open_already_loaded` API mismatch
(E0133); the adapter now owns an existing module reference directly, and adds a Windows-only
PID/absent-module test. This test is source-checked here, not executed on Windows.

GREEN: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D
warnings`, `cargo test --workspace --locked`, Windows all-targets check and clippy, Linux
workspace check. Latest macOS real-window smoke also exited 0 with Metal. No confirmed
background Cargo jobs remain after polling these checks.

## Layer gate, 2026-11-13

RED: `cargo test -p xtask layers --locked` failed for the absent graph validator. GREEN:
three tests reject equal/upward edges, misclassified/new packages, format-to-workspace
dependencies and a transitive GUI dependency; valid lower-layer edges pass. The actual
all-features Cargo resolution passes `cargo xtask layers`. Focused xtask clippy passes
after collapsing the nested condition required by the pinned compiler.

## Page model, 2026-11-13

RED: `cargo test -p bondline-model` failed for missing PageId/PageSize/Document APIs. GREEN:
three tests pass against independent physical-unit expectations (72 points per inch, A4
210 by 297 mm), invalid dimensions including non-finite values, checked deserialization,
nonzero IDs and failed-edit atomicity. Focused model clippy and actual layer gate pass.

## External artifact inspection, 2026-11-13

REA 6.9.0 ran as an external development tool under an already installed Node 26.12.1.
`rea audit-release-exposure target/dev/Bondline.app --json` completed with exit 0. This
selected the earlier read-only development bundle, not any proprietary program. Evidence
ID `ev_a770a1af9d192bd3602b3081084097908158b625f86fbdcf35ff496f2a904c99`, directory
digest `b691b23c9b8b3f131dc99480ffe78828adc22e9ce00ba78dad1b4560ff8d86d9`. No source-map,
credential-marker or debug-companion condition was observed within the reported bounds.
Go/Rust metadata remained unknown; hardening, hardened runtime and Android were not assessed.
Raw report: ignored build output `target/rea-graphics-audit.json`. This is neither a license
audit nor a security pass. No REA code is bundled with the app.

## Page command and snapshot history, 2026-11-13

RED: `cargo test -p bondline-commands` failed for the missing registry/engine. GREEN: typed
closed schemas, published physical page dimensions, malformed/oversized requests, failed-edit
atomicity, read-only inspection, redo preservation and branch replacement pass. Proptest runs
128 fixed-seed sequences of up to 63 edits; each edit has independently asserted dimensions,
exact undo/redo restoration, and full-sequence restoration. Boundary tests additionally check
the declared 128-snapshot cap and atomic rejection at revision overflow. No desktop or transport
verification is implied by these headless tests.
