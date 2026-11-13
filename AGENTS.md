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

## Research, provenance and implementation

- Target one-to-one Professional desktop screens, control roles, icon roles, workflows and
  functionality, with Bondline branding and the existing offline/open-source commitments.
  Use public documentation, screenshots, videos and authorized artifact/runtime inspection.
  The user's 2026-11-13 clarification supersedes the earlier program-binary inspection ban.
- Use REA for artifact/runtime questions and ordinary source tools for open-source repositories.
  Record exact source/build/version/platform, URLs or artifact hashes, video timestamps,
  observations, inferences and unknowns. A screenshot or static string alone proves no flow.
- Obtain research artifacts from official/public or otherwise authorized sources. Use existing
  entitlements for licensed runtime access; do not bypass activation/access controls. Record
  actual access blockers and continue independent public research. Keep reference downloads,
  screenshots, installers and REA reports in ignored .cache/ or target/, never shipped by default.
- Implement original Rust logic and original redistributable assets matching documented roles
  and appearance. Do not transplant proprietary source, decompiled code, fonts, templates,
  sample libraries, help prose or icon files into Bondline without applicable redistribution
  rights and the asset/dependency policy. Reference caching is distinct from product reuse.
- Never use proprietary product marks in Bondline's UI or executable names. README may
  identify compatibility and must state non-affiliation. Parity/source records may name references.
- Shipped dependencies and bundled assets must permit use under MIT, Apache-2.0 or BSD.
  Do not broaden this allowlist or change license labels to pass a check. Preserve notices.
  Build-only tools are distinct from shipped runtime dependencies. GPL/LGPL/AGPL software
  may be run only as external test programs, never linked or copied into the shipped application.
- Record sources and behavior decisions in docs/clean-room/sources.md (historical path).
  Unresolved behavior goes in docs/clean-room/open-questions.md; do not guess from memory.
  The updated method does not claim strict isolation from runtime/artifact observations.

## Architecture

The intended crate graph is model/cdxml/cdx -> chem/render -> io -> commands -> mcp -> cli/ui.
Packages use bondline- prefixes. cdxml and cdx have no workspace dependencies; formats
preserve unknown data and must never silently drop it. Native I/O stays out of model/parsers.
The UI, CLI, loopback channel and MCP call one typed JSON command registry.
StoredDocument now owns source/model/baseline and Engine history retains it. Keep that backing
when wiring import; a cloned elemental projection alone loses unknown data. Require failed-
import/source-retention tests before exposing imports. Retained payload/container accounting
now enforces 128 prior snapshots and a 128 MiB threshold, deduplicating shared source/model/
fragment allocations and including baselines/reserved vectors. Keep current data even when
it alone exceeds the threshold; this excludes allocator headers, caches and external copies.
Update accounting/tests whenever a new model field owns heap data; do not claim a hard RSS cap.
Unchanged-source export is guarded; never write stale XML after a model edit. StoredDocument::
to_cdxml borrows unchanged source or encodes new native elemental data. Do not bypass it by
regenerating stored.model(): that projection omits unknown data. A generated profile is fully
mapped only if re-encoding reproduces every input byte. Its cached classification now permits
headless editing/rendering; unknown/extended/noncanonical files remain guarded and preserved.
Keep the pinned v0 fixture editable when changing the encoder/version; add migrations instead
of refreshing it. Native file wiring now uses those complete snapshots. Poster Width/Height
are drawing extents, not inferred paper/printer settings.
xtask checks layers and licenses, fetches a pinned SHA-256 corpus, and orchestrates CI.
No GUI dependencies below ui; wgpu belongs in render. Add crates when useful behavior lands,
not empty successful-looking API stubs. Document any isolated OS FFI exception explicitly.
Loopback connections require the fresh session token emitted only in opt-in startup stdout.
Never commit or expose actual tokens in logs, screenshots or verification records. Stdio MCP
is separate and opens no listener. Keep file access bounded and explicitly authorized.

Current map: model (L0, pages, Point2 and validated elemental atom/bond/fragment graphs);
cdxml (L0, source-preserving syntax, integral/source-ID decoding, checked attribute
replacement and a bounded new-syntax writer);
cdx (L0, bounded flat binary syntax, exact source passthrough and checked opaque property
replacement; no model mapping or application commands);
render (L1, ordered CPU/GPU rectangles/strokes, limited authored C-C labels and approved golden); io (L2,
source-backed CDXML projections, native elemental encoding, owned snapshots, PNG encoding and
bounded native CDXML filesystem I/O; see docs/native-file-io.md for publication/access limits);
commands (L3, registry, complete stored-snapshot Arc history and transient close policy); mcp (L4, stdio/native loopback, registered work and reply publication barriers); cli/ui (L5,
headless and native adapters). Other planned crates do not exist yet. `compat/libloading`
and `compat/clipboard-win` opt into reviewed OS unsafe; `compat/hexf-parse` forbids
unsafe. These three original compatibility packages retain upstream package names solely
for Cargo patch routing. The eframe/winit source forks and Apache font retain upstream notices.
The pinned winit macOS application-quit callback is the other reviewed OS-boundary exception:
two standard main-thread AppKit visibility actions, with exact source/pin/native evidence in
vendor/winit/BONDLINE-PATCH.md. No unsafe enters Bondline's UI/model/command/format crates.
The pinned accesskit_macos string-value bridge is another reviewed native boundary; its
exact source/notices, nullable NSString checks, bounds and tests are in that vendor's
BONDLINE-PATCH.md. cargo xtask ci includes its native tests/clippy on macOS from the root lock.

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

MOLV2000 now has a source-preserving codec, owned snapshots, atomic I/O and shared commands.
Its transform is explicit, headers/raw source stay authoritative, and unsupported projection
must fail rather than erase data. Associated file.format is independent of source.format and
filename extension. Open defaults24pt/unit and origin(72,72)pt; source-backed Save keeps its
captured frame/headers. Multiple fragments, changed page layout and source metadata loss
remain refused. Do not call this complete M1 support. Pinned MOL fixtures have -text Git
attributes: preserve exact hashes on Windows, never normalize them in an oracle/parser.
Declared valence yields exact authored H by the published finite arithmetic; do not infer
periodic-table valence. Preserve the documented normative M ISO/RDKit discrepancy, and the
original/independent test vectors. The corpus now actually checks one MOL input, not only
its hash; CDX corpus and chemistry calculations remain missing.

The current thirteen-command registry includes bond.create for independent C-C fragments with
three explicitly authored H per carbon, plus native CDXML/MOL open/save. It does not attach to
existing atoms. Save complete StoredDocument data; file.dirty uses the saved snapshot, while
source.modified describes original backing. Prepare/complete/finish separates I/O from
revision-checked apply. Count current+saved snapshots and never call a publication error rollback.
Identity floors must survive Undo branches and advance only after successful commits.
Native gestures consume ordered events; press_origin alone loses a drag completed between
frames. Canvas interaction must not advance layout beyond visible error text.
Use Engine::controls() / ui.controls as the authoritative delivered File/Edit/Page bindings;
do not duplicate labels, shortcut tables, preset parameters or enabled rules in the UI.
ui.file is presentation intent, distinct from actual document.open_cdxml/save_cdxml or MOL I/O.
Trusted file plans bind ui_request_id; ui_request_current is captured before their own save
changes state. Native close-Save requires both receipt markers plus format/revision/path/
close-epoch checks. Intervening format/baseline changes stay invalid after returning to old
values. A direct save command never fabricates a trusted presentation receipt.
Headless engines have no host. Keep request IDs and context guards through native selection,
worker preparation and retry. Reserve/prepare under one engine lock before spawning; only
the matching current request may restore a failed picker. A host acknowledgement does not
prove visible pixels or saved bytes. Keep unaddressed dialog fields/job controls explicit.
Close intent/epoch never belongs in authored history or serialized CDXML. Request/Discard/
Cancel must use document.close; successful actions advance close_epoch without changing
authored revision. A close-specific Save must retain its captured epoch/revision and matching
current receipt; Cancel/new Request must not reuse it. Native complete_close is a trusted
host acknowledgement under the shared Engine mutex, after executor activity, UI file jobs
and pending file actions are absent. Register deferred work/replies under that same mutex;
release response activity after write/flush. Never restore UI-private close approval or let
an accepted file action start after Committed. Headless close intent does not kill stdio.
The user approved crates/render/tests/goldens/single-bond.png for its exact scene/hash only.
Its JSON and README record provenance; external Pillow compares every CPU/GPU RGBA byte.
Do not regenerate that golden without renewed human approval. Other images remain unapproved.
requirements-dev.txt now pins external RDKit/NumPy as well as Pillow. Select an isolated
interpreter with BONDLINE_PYTHON as described in docs/development.md; missing consumers fail
instead of skipping. The RDKit test verifies engine-created CDXML interoperability for
ethane only, not an implemented chemistry engine or complete differential corpus coverage.

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
Keep Cargo target directories separate for each worktree. Shared artifacts caused a real
wrong-checkout CI run and stale metadata; runtime workspace discovery does not make shared
binary outputs safe. xtask locates the invoked workspace through Cargo, and ci prints its
root. Verify that root before counting results. Use local Git commits. Small conventional
commits: one command or format feature, with its
tests, provenance and relevant docs. Keep main green; record red evidence without committing
failing main. Commit all intentional project files, never caches, secrets or build outputs.
Do not push or publish without an explicit request. Update STATUS.md with verified state,
actual failures, remaining work, next executable steps and live job handles before stopping.
Currently implemented xtask gates: layers, licenses, corpus and ci. Corpus checks the delivered
CDXML syntax/elemental projection subset plus pinned file integrity; its output identifies missing CDX/chemistry and broader format
coverage. The aggregate gate passes locally and on all three hosted platforms at 3d3dd44.
Hosted native-window smoke passes on all three platforms, plus Linux ASan, after the
Linux runtime-package correction. These are window/API checks, not interactive Windows/
Linux chemical-workflow evidence; see docs/ci.md.
Development bundles
and REA reports under target/ are disposable build outputs and must not be committed.
Parser fuzzing is isolated under fuzz/; read its README for pinned toolchain/tool versions,
seed setup and bounded runs. Do not mutate committed fixture directories as fuzz corpora.
