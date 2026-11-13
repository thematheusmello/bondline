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

The pure model now has a validated A4 page, stable page ID and no native/GUI dependency.
Its three tests pass against physical-unit expectations and malformed-input cases.

The shared registry now exposes inspect, page size, undo and redo with typed schemas.
Headless page editing/history is verified, including 128 fixed-seed property cases.

The CLI now discovers the same schemas and runs sequential JSON commands. Executable-level
tests verify page size -> undo -> redo and structured errors. License gates discover every
workspace product binary, so headless adapters cannot escape dependency review.

Native page presets/rotation and Edit history menus now call that registry. Actual macOS
interaction verified Letter page -> Cmd+Z -> Shift+Cmd+Z with dimensions observed in the
accessibility tree and the real wgpu window. There are no chemical drawing tools yet.

Missing: MCP/control, complete parity inventory, corpus,
CDXML parser, native CI workflows, native Windows/Linux execution and human-approved goldens.
The M5 browser candidate still includes Unicode-licensed data and is not approved for shipment.

Next executable tasks:
1. Route native actions and CLI/MCP/control through the registry.
2. Add corpus/CI tooling and finish the public-document parity inventory.
3. Add the standalone source-preserving CDXML parser with independent reference tests.

Development bundle target/dev/Bondline.app (org.bondline.development) was launched for
visual inspection and remains open on the disposable Letter-sized test page. It now contains
the page-control build. The earlier REA report refers to its previous digest. Inspect the
running process before replacing/relaunching it. Build artifacts are untracked.
