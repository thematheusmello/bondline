# Status

2026-11-13: goal active; native graphics compatibility checkpoint. Full M0 is NOT complete.

Verified on this Mac: Rust 1.94.1 workspace, native startup using Apple M3 Max / Metal,
three-frame real-window smoke exit, actual visual/accessibility inspection of a read-only
blank workspace, independent CPU/GPU RGBA reference, and a pinned Apache-2.0 font.
Declared runtime license closure passes macOS arm64, Windows x64 and Linux x64 without
broadening policy. Original loader/clipboard/hex-float adapters replace the disallowed
runtime code; fork/pin and source evidence are in docs/dependency-policy.md.
The graphics checkpoint is committed as db94cd5. `cargo xtask layers` now enforces strict
lower-layer dependencies, standalone formats and transitive GUI isolation.

Whole-workspace Windows (including tests) and Linux source cross-checks passed after
the common render extraction. Windows and host clippy pass with warnings denied.
These are NOT native Windows/Linux runtime verification.
Tests cover IEEE boundary vectors and 256 Python-generated float cases, malformed inputs,
OS loader vs standard-library PID, UTF-16/CF_HTML validation and resource/hash/license gates.

Missing: document/command engine, CLI/MCP/control, complete parity inventory, corpus,
CDXML parser, native CI workflows, native Windows/Linux execution and human-approved goldens.
The M5 browser candidate still includes Unicode-licensed data and is not approved for shipment.

Next executable tasks:
1. Add model + shared registry with typed schemas and exact undo/redo property tests.
2. Route native actions and CLI/MCP/control through it; add layer/corpus/CI tooling.
3. Add the standalone source-preserving CDXML parser with independent reference tests.

Development bundle target/dev/Bondline.app (org.bondline.development) was launched for
visual inspection. It may still be open; inspect the running process before relaunching.
Do not confuse that old executable with a newly built binary. Build artifacts are untracked.
