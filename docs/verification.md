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

