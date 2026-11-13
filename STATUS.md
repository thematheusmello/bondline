# Status

2026-11-13: persistent implementation goal active. Full M0 is NOT complete.

The native desktop/GPU license blocker is resolved for the pinned candidate stack without
broadening the MIT/Apache-2.0/BSD policy. Both product roots pass declared runtime closure:
macOS arm64 143 packages, Windows x64 154, Linux x64 213, plus two SHA-verified bundled assets.
See docs/dependency-policy.md for the original compatibility adapters, fork and source evidence.
This declaration gate is distinct from a complete release-notice/source review.

Verified: 44 workspace tests, rustfmt, workspace/all-targets clippy with warnings denied,
actual layer gate, and whole-workspace Windows/Linux all-targets source checks. These are
NOT native Windows/Linux runtime passes. CPU/GPU clear and ordered page-scene pixels agree on Apple M3 Max/Metal.

Delivered behavior:
- Pure model: validated physical page dimensions and stable page ID.
- Shared typed registry: document.inspect, page.set_size, history.undo, history.redo.
- Exact history: 128 fixed-seed generated edit sequences, no-op/redo/overflow/boundary checks.
- CLI: schema discovery, sequential JSON execution and structured errors, tested as a process.
- Native UI: Page presets/rotation and Edit history actions use the registry. Actual Mac
  Letter -> Cmd+Z -> Shift+Cmd+Z verified through accessibility and a real-window screenshot.
- Stdio MCP: official SDK, same registry, process-local engine, 1 MiB input bound. Independent
  wire client verifies negotiation, discovery, edits/errors, oversized-line recovery and EOF.

Desktop loopback control is now verified: opt-in IPv4 localhost, bounded frames, shared native
engine/history, repaint notification, concurrent-client cap and clean shutdown. An independent
socket edit changed the actual Mac canvas; native Undo was visible to another socket request.

The native canvas now consumes the shared render page scene. CPU and instanced GPU paths
match a hand-authored 8x8 pixel grid; layered clipping, alpha source replacement, odd pitches,
one-pixel frames and resource bounds are checked. Native page rotation was inspected again.
See docs/rendering.md for exact scope and the pending human golden gate.

No chemical drawing or file compatibility is implemented. There are no chemical tools.
Missing M0 evidence: offscreen capture, complete public menu/tool/hotkey inventory,
licensed corpus/fuzz tooling, aggregate CI and actual three-platform CI execution, native
Windows/Linux workflow checks and human-approved render goldens. The browser candidate
still includes Unicode-licensed data and is not approved for shipment.

Next exact tasks:
1. Add registry-driven offscreen page capture with independent CPU/GPU/PNG evidence.
2. Keep capture work outside the native engine lock so the window stays responsive.
3. Add corpus provenance/SHA fetch tooling, standalone lossless CDXML parsing and independent
   XML tests, then CI orchestration/workflows. No empty parser or successful no-op gates.
4. Extract the public guide's complete menu/tool/shortcut inventory with version/page evidence.

The graphics checkpoint is db94cd5; subsequent atomic commits add layers, model, commands,
CLI and native controls. The current loopback checkpoint includes this status record; use Git log
for its final hash. No confirmed Cargo/npm jobs remain running.

The disposable development bundle target/dev/Bondline.app (org.bondline.development) contains
the shared-scene build and is closed. Native test exec session 94971 exited 0; its listener at
127.0.0.1:50967 was independently confirmed closed. Earlier REA evidence is historical and
bound to its previous digest. Build artifacts and reports remain ignored.
