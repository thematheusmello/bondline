# Shared browser workspace

2026-11-20, started on main6fd6633 and integrated mainf64a20e. This work uses the real egui workspace, command registry,
models, authoring and history. It does not substitute an illustrative canvas or enable
native file paths in the browser. Initial actual wasm UI check fails because desktop MCP
Tokio networking reaches mio. Target-specific network/server isolation retains desktop
semantics. Native file workers are unavailable; browser controls retain that host gate.

## Sources and host decisions

The pinned [eframe0.29.1 source](https://github.com/emilk/egui/tree/0.29.1/crates/eframe/src/web)
and [WebRunner](https://docs.rs/eframe/0.29.1/eframe/struct.WebRunner.html) specify Canvas
startup and shared App callbacks. Actual vendored APIs were inspected; this web reader
cannot open docs.rs, so the local pinned source is the API evidence. The official
[wasm-bindgen no-bundler guide](https://rustwasm.github.io/docs/wasm-bindgen/examples/without-a-bundler.html)
supports generated JS/WASM assets. JS contains bootstrap/platform coordination only;
all authoring and layout uses the same Rust UI and registry.

[Origin-local storage](https://developer.mozilla.org/en-US/docs/Web/API/Window/localStorage)
is recovery, not a filesystem or native Save association. Use one bounded1MiB real CDXML
draft under the versioned browser key. Write only after a real model snapshot changes;
preserve malformed/opaque/oversized old data and disable saving visibly. Keep the exact
observed stored body. An externally changed body retires writing before replacement.
The [Web Locks API](https://w3c.github.io/web-locks/) holds one cooperative writer lease
per origin/page lifetime. A second tab or unsupported storage/locking remains visibly
unable to save its edits. Unrecoverable in-tab edits retain the browser unload prompt;
undo history and view state are not persisted. No native Save/close receipt is issued.
New/restored browser sessions explicitly request100% through the shared `ui.view` command.
This original host policy keeps the drawing readable in a660px-tall viewport. Explicit
in-session zoom remains unchanged on resize. Existing View Fit/Actual controls and the
renderer scale refusal remain shared; no label-size guard is weakened.

The [service-worker lifecycle](https://developer.mozilla.org/en-US/docs/Web/API/Service_Worker_API/Using_Service_Workers)
supports an independently hash-versioned, scope-specific static cache. Only own-origin
assets are fetched. A loaded package must reload and edit while offline; a live local
server alone is not offline-startup evidence. HTTP localhost or HTTPS is required for
trusted browser APIs. No account, remote service or proprietary runtime is used.
Updates use the normal waiting lifecycle until old clients close; no forced activation
mixes a live editor's old JS with a new WASM bundle. Fetch consults only the current
scope's versioned cache. Actual offline tests also reject all server requests because
CDP page-offline alone leaves a service worker's independent network target available.

## Pinned SDK compatibility and license closure

Actual current Chrome rejects the wgpu22 requestDevice descriptor containing removed
maxInterStageShaderComponents. [WebGPU](https://gpuweb.github.io/gpuweb/) uses the newer
maxInterStageShaderVariables; [upstream wgpu releases](https://github.com/gfx-rs/wgpu/releases)
distinguish scalar components from variable locations. Bondline's pinned MIT/Apache
backend keeps legacy browsers on the original field. Modern browsers advertise a
conservative component budget equal to their variable count: any scalar could consume
one location. The browser UI clamps its descriptor to that bound and sends the modern
key. It never multiplies variables by four or claims arbitrary component/variable
interchangeability. Browser validation remains authoritative; desktop code is unchanged.
The actual egui shaders and page workflow must run before this is accepted.

The actual wasm runtime graph initially contains Unicode-licensed unicode-ident only
through wasm-bindgen-shared's unused identifier helper module. Runtime consumers use
tys and module_path_hash. A pinned fork retains the complete identifier implementation
and dependency on native build-tool targets, excluding them on wasm. There is no ASCII
replacement, Unicode-table copying or license allowlist expansion. lib.rs/tys.rs schema
bytes remain exact, keeping the installed matching0.2.130 CLI. The four-target runtime
license audit selects actual browser cdylib roots, not the unsupported native CLI binary.
Archive/VCS and modified-file hashes belong to the adjacent vendor patch notices.
Packaging runs the strict gate before generating assets and retains all selected runtime
license files plus Roboto. Missing archive notices for AccessKit/profiling come from the
exact published VCS roots, with checked copies in `docs/licenses/browser-runtime`; four
egui family crates reuse the exact same-revision upstream notices already retained in
`vendor/egui`. Missing or changed provenance refuses packaging, not silent substitution.

## Acceptance scope

Exercise actual WebGPU paint, pointer bond creation, keyboard Undo/Redo and exact real
CDXML recovery through reload, including offline reload. Check the stored source with
independent Expat/Decimal/RDKit. Cover the second tab, externally replaced/corrupt data,
quota failures and native-file refusal. Capture the actual executable canvas separately
from illustrations or approved goldens. Keep native fmt/clippy/tests, strict license and
layer gates. New Windows/Linux/browser hardware, all advertised native tool gestures,
accessibility, file import/export, general recovery/migration and complete M5 remain open
unless separately executed. Actions stays off. No push/publication by the worker.
