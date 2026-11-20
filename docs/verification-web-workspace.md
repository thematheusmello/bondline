# Shared browser workspace verification

2026-11-20. The complete M5 target remains incomplete. GitHub Actions stays off.

## Native transport isolation

The original `cargo check --target wasm32-unknown-unknown -p bondline-ui --lib`
failed in mio: the UI's shared MCP work types enabled desktop Tokio networking.
The MCP crate now keeps synchronous work/lease contracts available on both targets,
while native TCP, stdio server and Tokio runtime entry points are target-specific.
No listener protocol, authentication, timeout or protected test assertion changes.

On main `f64a20e` plus the transport isolation, pinned Rust1.94.1 on macOS arm64:

- `cargo test --locked --offline -p bondline-mcp --lib`:22 passed.
- `cargo clippy --locked --offline -p bondline-mcp --all-targets -- -D warnings`:passed.
- `cargo check --locked --offline -p bondline-mcp --target wasm32-unknown-unknown`:passed.
- `cargo fmt --all -- --check`:passed for the workspace sources.

The checks used the isolated `target/web-workspace/target` directory. Disposable logs
are in the web worktree's `target/web-workspace-evidence/{mcp-native,mcp-native-clippy,
mcp-wasm-check}.log`. The checked dependency graph includes the in-progress pinned
browser SDK forks; their separate changes/evidence do not confer desktop UI or
complete browser acceptance. Existing commands emit eight wasm-only unused warnings;
the old vendored graphics SDK also emits upstream warnings. The selected native MCP
clippy gate is clean; a warning-free whole browser graph is not claimed.

## Pinned browser SDK

The first real Chrome startup failed with `requestDevice` rejecting the removed
`maxInterStageShaderComponents` field. `vendor/wgpu` now negotiates the current
WebGPU field conservatively; no native backend source changes. The actual editor
then paints through eframe/WebGPU. Browser app limit clamping belongs to the next
host increment, not to a desktop capability claim.

The first actual wasm runtime audit found77 packages and denied only unicode-ident's
conjunctive Unicode license. `vendor/wasm-bindgen-shared` preserves the native
build-tool identifier implementation and excludes it from wasm runtime. The schema
files remain byte-exact and wasm-bindgen-cli0.2.130 generates the functioning host.
The same strict audit on the actual browser cdylib graph now passes76 packages;
native graphs pass155/166/221 for macOS/Windows/Linux, with7 existing assets checked.
No allowlist extension or dependency version upgrade is involved. The actual fourth-
target audit and host are the following integration increment; these counts describe
that combined working tree, not a browser product shipped by the SDK commit alone.

All144 published wgpu files and11 published shared-helper files were independently
compared with cached archive bytes. Exactly one wgpu backend and two shared-helper
files differ; complete licenses, VCS identities and before/after hashes are in the
adjacent BONDLINE-PATCH.md records. `lib.rs`, `tys.rs`, the shared build script and
schema approval remain original. Disposable `vendor-integrity.json` holds the
machine comparison. No modern/legacy limit equivalence beyond the documented
conservative bound, legacy browser execution or upstream acceptance is claimed.

## Actual shared editor

The first host checkpoint944ed26 is based on mainf64a20e plus transport33fa36f and SDK463b0c6.
`WebHandle` starts eframe WebRunner and the same Rust `show_workspace`, drawing state
and Engine. It adds no authoring command; that checkpoint's registry remains53. Native file
presentation, filesystem requests and desktop transports stay unavailable in the browser.
`web_recovery` uses the real CDXML codec and never acknowledges native Save.

Executed with Rust1.94.1, wasm-bindgen-cli0.2.130, Node26.5.0 and an owned headless
Chrome157.0.8103.79 profile on macOS arm64. Chrome's test flags enable headless WebGPU
and force DPR1; this is actual egui/WebGPU paint, not a CPU bitmap substitute or a
reference-product screen. No existing user app/profile/drawing was touched.

`package_web.py --profile dev` produces the actual library/JS and local static assets.
The executed f64a20e host bundle is
`fc431ed9dd2bf42bf8dc38bc89a1307c9d6aea2b96bbcca9940cfbcbad472468`.
WASM:14,402,333B/SHA-256
`9e8df27cd7854cc7c5ac315a6c20743781da9f6a60dbdbbe0da06882f076412a`.
The manifest hashes each asset, including863,077B of runtime/font/source-patch notices.
Notice SHA-256:`fadb6bc2df4431e5a942ddd8e0ced1efefd51c5c2429031adf0bcad68e9d0e2e`.
Packaging runs the unchanged strict policy first; the new target selector audits the
actual browser cdylib and all76 normal non-proc-macro runtime packages. Seven license
tests pass. Missing archive notices have exact public-VCS receipts; unreviewed missing
notices refuse packaging. This is not a whole-release legal/source certification.

`web_workspace_browser.mjs` drives real pointer/keyboard/menu events and the same registry:

1. Drag120 logical pixels at100%:one C–C fragment, two C/H3 atoms, one history entry.
2. Open View→Show Analysis Window with the real menu:visible `C2H6`, unchanged model.
3. Primary-Z/Primary-Shift-Z restore exact empty/authored model and recovery source.
4. Reload the cached package offline with both page networking disabled and every local
   server request rejected. It reopens the same editable model with fresh history and
   no invented file path. Offline notices retain their exact hash; unrelated origin
   caches are not consulted. The server witnessed7 rejected worker requests.
5. Repeat offline reload at1160×660 after the1160×800 workflow. New/restored sessions
   start at100%; visible View→Fit retains the existing readability refusal, and
   View→Actual size recovers ink. Real pointer input creates a second fragment; keyboard
   Undo/Redo and final Undo retain the exact original model and436B stored source.
6. Exhaust real origin storage:quota5,242,394 filler characters with8 actual quota
   exceptions. The next real edit reports unsaved state and preserves the prior draft.
   Clear only the test filler; Undo restores the original exact source.
7. A second tab cannot acquire the writer lock; its real engine edit stays in memory
   and cannot overwrite the first tab's draft. It reports unsaved in-tab changes.
8. An externally replaced invalid value revokes writing on the next edit; fresh startup
   also preserves corrupt data and refuses subsequent replacement.
9. Native file/presentation requests, non-string and oversized requests refuse without
   editing. Both the JS-unit preflight and existing1MiB UTF-8 command bound are checked.

Independent `web_workspace_source.py` uses Expat/Decimal and RDKit2026.09.2 on the actual
saved source:unique raw IDs, explicit C/H3,120pt horizontal bond, physical A4 extents,
RDKit `CC` and `C2H6`. Source436B/SHA-256
`5a2d441ffc7ae8822863fef0a57f9f2de8d351d514d5e055c6536a5d87f54c88`.
This is separate from the Rust codec round trip. The two recovery unit tests are included
in the native UI gate:369 passed,0 failed,1 pre-existing ignored. Native UI+xtask
all-targets Clippy with warnings denied passes; native MCP22/0 and its Clippy gate pass.
Workspace fmt, layer and four-target license checks also pass before the host commit.
`cargo clippy -p bondline-ui --lib --target wasm32-unknown-unknown` also passes with
the existing native-only unused warnings; it is not a warnings-denied wasm gate.

Disposable final reports/logs/images are under the web worktree's
`target/web-workspace-evidence/flow-server-offline`, `source-server-offline.json`,
`package-final.log`, `ui-native.log`, `ui-native-clippy.log` and `licenses-main.log`.
The earlier `flow-final` unrelated-cache assertion used page-only CDP offline mode;
its failure exposed a test-boundary gap because workers have an independent fetch
target. The stronger server-rejection run above supersedes that offline claim. Earlier
mio/WebGPU and low-Fit RED receipts remain separate. No old assertion, golden, asset,
dependency allowlist or listener policy was weakened. All owned jobs/profiles/servers
are terminal at handoff; Actions stays off.

## Limits and continuation

This verifies the initial browser workflow, not every enabled shared tool or full M5.
Browser file import/download/export, accessibility, other engines/hardware/DPI,
non-headless operation, optimized release/performance, cache upgrades and broader
editor workflows still need acceptance. Undo and explicit view state reset on reload;
the draft is origin-local recovery, not a durable file backup. Unsupported/corrupt
stored data is retained and saving is disabled. Native library regression checks are
separate from launching the desktop app; no new desktop launch is claimed here.
Wasm-only unused warnings and upstream SDK warnings remain explicit. The full workspace
aggregate and main integration are the receiving root's next gate, not inferred from
these focused component passes. A screenshot is not a newly approved golden.

## Integration with main6dbd4e6

Main's delivered mixed skeletal movement and molecular masses are retained; only STATUS
needed a textual merge resolution. The browser adds no registry entry. Main's
`analysis.properties` is record54, while `analysis.formula` retains its earlier response.
The old actual browser bundle fails the new mass-query assertion, recorded separately
in `flow-mass-red`; rebuilding the same shared UI against current main passes it.

`flow-main-unload` repeats all nine actual workflow stages, including both viewports,
offline server rejection, Fit/Actual recovery, pointer/history, real quota, competing
tabs and corrupt/external replacement. The Analysis menu now paints molecular weight
30.070 and exact mass30.046950Da. The real query returns30.07 and30.04695019338.
The unchanged436B source independently gives RDKit30.07 and30.046950192; the prior
declared1e-7Da edition tolerance is retained. The source geometry/formula checks stay exact.
An actual pointer activation in the second tab enables Chrome's real beforeunload
dialog. Canceling that dialog retains its unsaved document before the test undoes and
closes the owned tab. This supplements host-state assertions with an actual unload flow.

Combined dev bundle:
`98a25eef73fbae05cabc5d3dcb258c7c74d70198d4b8848da84559857e862f8f`.
WASM14,656,923B/SHA-256
`b137b5e96a8087530b14c42c408ec48afc40af5cac5e7e0ff5a430a5dd26dbe3`.
Runtime/font/source-patch notices865,546B/SHA-256
`d45f8793f46e96e1c771789887110e7f245639ea3590e797676ad84a859e8c96`.
The same strict audit now passes157/168/223/78 packages for macOS/Windows/Linux/wasm,
plus7 existing assets; the two additional runtime packages are main's already-reviewed
MIT elements_rs/strum. The packager retains their actual archive notices too.

The combined package/flow/source receipts are `package-main-6db.log`,
`flow-main-unload/report.json` and `source-main-6db.json` in the same ignored evidence
directory. The final image in docs/media is this combined actual Analysis window.
Full combined local CI is the next gate; no complete aggregate is inferred yet.

## Raw browser Unicode boundary

The complete c770a6f local CI passed:workspace2,406/0/1 plus the separate AccessKit3/0,
all fmt/Clippy/layers/license/asset/corpus gates. Log `ci-main-6db.log`, SHA-256
`f7543341de8908595bd50a8a58034fe794dbf05eee67098aed13d247b38c7f29`.
This predates the following wasm-only boundary correction and later main integration.

An actual independent caption fixture with a lone UTF-16 surrogate inserted into an
unrelated XML comment exposed data loss. The generated Storage String binding replaced
the surrogate before Rust parsed it. The host admitted that normalized source, and
editing Phase A→Phase B overwrote the unrelated stored code unit. `flow-utf16-first-red`
records admitted=true/changed=true/retained=false; this is not a mock or self round trip.

The host now reads a raw JS value, checks the shared size bound and the pinned SDK's
UTF-16 validity API before conversion, then checks the exact UTF-8 bound. Startup and
compare-before-write use the same reader. JSON strings also reject invalid UTF-16.
Parser, model, registry, native source and authoring permissions are unchanged.
`flow-after-cleanup` passes all11 stages:malformed startup is refused and retained;
valid β/supplementary Unicode in a comment survives an accented caption edit; an
externally substituted malformed counterpart cannot exploit normalization to pass the
source equality check. Earlier pointer/history/Analysis/offline/660px/quota/conflict/
unload cases remain unchanged and pass. Wasm Clippy passes with the known unused warnings.

Its dev bundle is`11bed9e599653f4d34243f3e790337954913e3479927882acbf23727b820770e`,
WASM14,664,032B/SHA-256
`6233518ab7abdca1a1d33ffa1690eadcbe1edae2a24e98cb7b04786860996fd7`.
Runtime graphs/notices are unchanged from the main6dbd4e6 checkpoint. The intermediate
`flow-utf16-{red,green,complete}` attempts stopped at a failed Cache.addAll install and
unresolved readiness promise, not at the tested Unicode assertions. The worker domain
recorded a NetworkError and redundant registration. They are retained as failures; a
full rerun after disk cleanup passes. No causal code fix for those installation failures
is claimed. Current main and final aggregate are the next integration gate.

## Integration with maindce8cb0

PR30/31/33's MOL coverage, independent corpus chemistry and elemental analysis are
retained. The actual browser still exposes54 registry entries. The11-stage workflow
passes on the combined tree (`flow-dce-cache-quota`), including both Unicode boundaries,
real pointer/history and Analysis, server-offline reload at both heights, quota and
unload Cancel. The optional elemental query returns H6.048/20.113069504489527% and
C24.022/79.88693049551047%, independently declared from the conventional weights.
The current actual Analysis screenshot includes that table. The independent source
reader (`source-dce.json`) still verifies the exact436B source and mass quantities.

Executed dev bundle:`b60162b6dfb3010692657120493d2424e3c7245fcbff0717374b3826dcb3b4cc`.
WASM14,736,786B/SHA-256
`9ad43ff80197f4237e0da9174641823df23fa83fcd344256888e7fd7113b3d66`.
Runtime graph/notices stay157/168/223/78 and7 assets. The normal first-install
Cache.addAll path also had intermittent NetworkErrors despite complete HTTP200
responses; the failed receipts include worker/cache diagnostics. A subsequent unchanged
run passes, so neither disk cleanup nor diagnostic setup is claimed as a causal fix.
The final aggregate and bounded failure handling remain the next receiving work.
