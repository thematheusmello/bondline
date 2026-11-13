# Bondline agent instructions

## Goal and orientation

Implement an independent offline Rust chemical structure and reaction editor, native on
macOS, Windows and Linux, eventually also WebAssembly. Original code is MIT OR Apache-2.0.
The reference is publicly documented Professional 26.1 behavior, not proprietary code.
Read STATUS.md, ROADMAP.md, docs/architecture.md, docs/parity-checklist.md and
docs/clean-room/sources.md before selecting work. Continue existing work; do not restart.
The persistent project goal is active. M0's native license blocker is resolved; this does
not complete M0. See docs/control-protocol.md for the delivered command/adapter contract and
docs/verification.md for exact RED/GREEN and platform evidence.

## Clean room: mandatory

- Use published specifications, public guides, open standards and appropriately licensed
  open-source engineering references. Record decisions and their sources in the same commit.
- Never download, execute, inspect, decompile or disassemble ChemDraw program binaries.
  Never copy its templates, sample libraries, icons, fonts or help text. Independently
  authored, openly licensed interchange test documents are allowed with provenance.
- Never use proprietary product marks in Bondline's UI or executable names. README may
  identify compatibility and must state non-affiliation. Parity/source records may name references.
- Shipped dependencies and bundled assets must permit use under MIT, Apache-2.0 or BSD.
  Do not silently broaden this allowlist or change license labels to pass a check.
  Preserve notices. Build-only tools are distinct from shipped runtime dependencies.
- GPL/LGPL/AGPL software may be used only as external test tools, never linked or copied.
- Unknown behavior goes in docs/clean-room/open-questions.md; do not guess from memory.

## Architecture

The intended crate graph is model/cdxml/cdx -> chem/render -> io -> commands -> mcp -> cli/ui.
Packages use bondline- prefixes. cdxml and cdx have no workspace dependencies; formats
preserve unknown data and must never silently drop it. Native I/O stays out of model/parsers.
The UI, CLI, loopback channel and MCP call one typed JSON command registry.
xtask checks layers and licenses, fetches a pinned SHA-256 corpus, and orchestrates CI.
No GUI dependencies below ui; wgpu belongs in render. Add crates when useful behavior lands,
not empty successful-looking API stubs. Document any isolated OS FFI exception explicitly.
Loopback connections require the fresh session token emitted only in opt-in startup stdout.
Never commit or expose actual tokens in logs, screenshots or verification records. Stdio MCP
is separate and opens no listener. Keep file access bounded and explicitly authorized.

Current map: model (L0, pages, Point2 and validated elemental atom/bond/fragment graphs);
cdxml (L0, source-preserving syntax, exact integral/source-ID decoding and checked
existing-attribute replacement);
render (L1, shared CPU/GPU page scenes); io (L2,
source-backed CDXML node geometry and validated PNG encoding); commands
(L3, registry and Arc snapshot history); mcp (L4, stdio and native loopback adapters); cli/ui (L5,
headless and native adapters). Other planned crates do not exist yet. `compat/libloading`
and `compat/clipboard-win` alone opt into reviewed OS unsafe; `compat/hexf-parse` forbids
unsafe. These three original compatibility packages retain upstream package names solely
for Cargo patch routing. The eframe source fork and Apache font retain upstream notices.

## Test first, then implement

For each behavior: write one test using independent expectations, observe the intended
RED failure, implement minimally, observe GREEN, then refactor. Record commands/results
in docs/verification.md. A round trip through only our own reader and writer is insufficient.
Use independent parsers, published reference vectors, externally run chemistry tools,
analytic cases, approved goldens and property tests as appropriate.
Documentation/configuration-only work needs relevant checks, not artificial tests.
Keep invalid input, resource bounds, error handling, accessibility and data preservation.
No production panic/unwrap/expect/todo/unimplemented and no unsafe outside an explicitly
reviewed OS boundary. Borrow where practical; use typed Result errors in libraries.

To add a command: first add its independent behavioral/error/history tests, register its
typed closed parameter object once in command_registry!, implement it below the UI, wire
delivered adapters, update CLI/MCP wire tests as needed, then update provenance and parity.
For UI changes, launch the actual native app and inspect the workflow. Eguis's input tests
and source cross-checks do not substitute for native platform execution.

## Pick and finish the next task

Take the highest-priority unblocked item in ROADMAP.md and the parity checklist. Read its
source, callers, tests, fixtures, config and exports. Implement the smallest complete slice
through the shared registry and delivered adapters. Check meaningful success/error cases.
Track missing, wired and verified separately; wired is not evidence of correct behavior.
Never call all M0 complete while desktop/GPU, cross-platform or inventory evidence is missing.
Do not spawn subagents unless the user or other applicable instructions explicitly authorize it.

## Commands and commits

When the workspace exists: cargo fmt --all -- --check;
cargo clippy --workspace --all-targets --locked -- -D warnings;
cargo test --workspace --locked; cargo xtask layers; cargo xtask licenses;
cargo xtask corpus; cargo xtask ci. Until implemented, a documented future command is not a pass.
Use local Git commits. Small conventional commits: one command or format feature, with its
tests, provenance and relevant docs. Keep main green; record red evidence without committing
failing main. Commit all intentional project files, never caches, secrets or build outputs.
Do not push or publish without an explicit request. Update STATUS.md with verified state,
actual failures, remaining work, next executable steps and live job handles before stopping.
Currently implemented xtask gates: layers, licenses, corpus and ci. Corpus checks the delivered
CDXML subset plus pinned file integrity; its output identifies missing MOL/CDX/chemistry
coverage. The aggregate gate passes locally; hosted workflows are configured but unexecuted.
Development bundles
and REA reports under target/ are disposable build outputs and must not be committed.
Parser fuzzing is isolated under fuzz/; read its README for pinned toolchain/tool versions,
seed setup and bounded runs. Do not mutate committed fixture directories as fuzz corpora.
