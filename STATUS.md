# Status

2026-11-13: persistent implementation goal active. Full M0 is NOT complete.

The native desktop/GPU license blocker is resolved for the pinned candidate stack without
broadening the MIT/Apache-2.0/BSD policy. Both product roots pass declared runtime closure:
macOS arm64 145 packages, Windows x64 156, Linux x64 215, plus two SHA-verified bundled assets.
See docs/dependency-policy.md for the original compatibility adapters, fork and source evidence.
This declaration gate is distinct from a complete release-notice/source review.

Verified: 60 workspace tests, rustfmt, workspace/all-targets clippy with warnings denied,
actual layer gate, and whole-workspace Windows/Linux all-targets source checks. These are
NOT native Windows/Linux runtime passes. CPU/GPU clear and ordered page-scene pixels agree on Apple M3 Max/Metal.

Delivered behavior:
- Pure model: validated physical page dimensions and stable page ID.
- Shared typed registry: document.inspect, page.set_size, history.undo, history.redo,
  render.capture (explicit CPU/GPU preview).
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

The io crate encodes validated RGBA8 frames as PNG. External Pillow 12.3.0 verifies all
decoded pixels, including transparent RGB and a hand-authored page grid. Registry/CLI/MCP/
loopback capture now returns PNG base64, input revision and actual backend/adapter evidence.
Adapters snapshot under the engine lock and render outside it. Two-worker budgets, explicit
cancellation, dropped-future cancellation and permit lifetime are tested. No chemical export.

Actual Mac control capture: 512x512 CPU/GPU pixels matched; a 2048x2048 Metal capture retained
revision 1 while a page edit reached revision 2 (0.301 ms round trip in one local observation).
Native Undo reached revision 3, and the next PNG exactly matched the original portrait.
This is workflow evidence, not a throughput benchmark. Human golden review is pending.

M1 has started: standalone CDXML unchanged-byte preservation passes original-fixture tests
with external Expat, BOM/CRLF and resource/error checks. Streaming limits precede tree
construction; internal DTD subsets are rejected and external identifiers never resolve.
A 10000-run ASan fuzz smoke completed without failure (seed 3641879538). The corpus tool
now fetches two pinned BSD-3-Clause RDKit documents and verifies hashes on download/cache.
One CDXML round trip and its complete element/attribute/text view agree with external Expat.
The MOL file has integrity coverage only, printed explicitly. No CDX corpus entry yet.
No edited writer,
chemical mapping, application file workflow or CDX/MOL compatibility is implemented.
There are no chemical tools.
Missing M0 evidence: complete public menu/tool/hotkey inventory,
broader corpus/chemistry coverage, actual three-platform CI execution, native
Windows/Linux workflow checks and human-approved render goldens. The browser candidate
still includes Unicode-licensed data and is not approved for shipment.

Next exact tasks:
1. Extract the public guide's complete menu/tool/shortcut inventory with version/page evidence.
   The current 20-row index explicitly remains incomplete; never infer labels from memory.
2. Continue M1 with checked source-preserving attribute edits and independent tests, then
   chemical model mapping/open-save through the registry. Add session authorization before
   exposing private file data on loopback. Research/pin a licensed CDX fixture before decoding.
3. Record the pending human golden response without treating silence as approval; no new
   golden has been approved. CPU/GPU candidate PNGs are under ignored target/captures/.
   The displayed GPU candidate is page-gpu.png: 512x512 pixels, 72x144 pt page, PNG SHA-256
   5cad7d0f59bbee036f8d55319129814cb85efcd4d572e9f35622be77ca519781. Approval would apply
   only to this blank-page preview; add a durable golden and regression check after approval.

The graphics checkpoint is db94cd5; subsequent atomic commits add layers, model, commands,
CLI and native controls. The current CI checkpoint includes this status record; use Git log
for its final hash. No confirmed Cargo/npm jobs remain running.

The disposable development bundle target/dev/Bondline.app (org.bondline.development) contains
the capture build and is closed. Native test exec session 56032 exited 0; its listener at
127.0.0.1:53031 was independently confirmed closed. Earlier REA evidence is historical and
bound to its previous digest. Build artifacts and reports remain ignored.

CI: the real aggregate `cargo xtask ci` passes locally (60 tests and all existing gates).
Its fail-fast behavior was checked with a deliberate formatting defect and restored source.
The three-platform native/window workflow and separate Linux ASan job are wired, not run.
Pinned action manifests and runner labels were researched; YAML parses with PyYAML 6.0.3.
actionlint and hosted executions remain unverified. No push, workflow dispatch or publication.
