# Status

2026-11-13: persistent implementation goal active. Full M0 is NOT complete.

The native desktop/GPU license blocker is resolved for the pinned candidate stack without
broadening the MIT/Apache-2.0/BSD policy. Both product roots pass declared runtime closure:
macOS arm64 143 packages, Windows x64 154, Linux x64 213, plus two SHA-verified bundled assets.
See docs/dependency-policy.md for the original compatibility adapters, fork and source evidence.
This declaration gate is distinct from a complete release-notice/source review.

Verified: 46 workspace tests, rustfmt, workspace/all-targets clippy with warnings denied,
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

The io crate now encodes validated RGBA8 frames as PNG. External Pillow 12.3.0 verifies
format, mode, dimensions and every decoded byte, including transparent pixels. This backend
is not yet wired as a registry/CLI/MCP/control capture command or native export workflow.

No chemical drawing or chemical-file compatibility is implemented. There are no chemical tools.
Missing M0 evidence: offscreen capture, complete public menu/tool/hotkey inventory,
licensed corpus/fuzz tooling, aggregate CI and actual three-platform CI execution, native
Windows/Linux workflow checks and human-approved render goldens. The browser candidate
still includes Unicode-licensed data and is not approved for shipment.

Next exact tasks:
1. Add render.capture to the existing typed registry. Parameters should identify page,
   viewport pixel dimensions and explicit CPU/GPU backend; return PNG plus adapter/revision
   evidence. Keep existing commands compatible and update shared discovery/wire tests.
2. Split preparation from expensive query completion: snapshot under the native engine lock,
   render/encode outside it, bound concurrent captures and cancel safely on shutdown. Verify
   immutable request revisions and native responsiveness. Renderer and PNG codec already
   have independent tests; do not create a second page-layout/encoding implementation.
3. Add corpus provenance/SHA fetch tooling, standalone lossless CDXML parsing and independent
   XML tests, then CI orchestration/workflows. No empty parser or successful no-op gates.
4. Extract the public guide's complete menu/tool/shortcut inventory with version/page evidence.

The graphics checkpoint is db94cd5; subsequent atomic commits add layers, model, commands,
CLI and native controls. The current PNG backend checkpoint includes this status record; use Git log
for its final hash. No confirmed Cargo/npm jobs remain running.

The disposable development bundle target/dev/Bondline.app (org.bondline.development) contains
the shared-scene build and is closed. Native test exec session 94971 exited 0; its listener at
127.0.0.1:50967 was independently confirmed closed. Earlier REA evidence is historical and
bound to its previous digest. Build artifacts and reports remain ignored.
