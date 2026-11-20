# Bondline browser workspace

The native egui workspace also runs as a Rust/WASM library on WebGPU. Its drawing,
commands and history use the same engine. Native file dialogs and paths are unavailable;
there is no emulated filesystem. The current browser delivery is an initial subset of M5.

Install the `wasm32-unknown-unknown` Rust target and wasm-bindgen-cli exactly matching
Cargo.lock (currently0.2.130). Cached Cargo dependencies are required for the offline build.

```sh
python3 xtask/oracles/package_web.py --profile dev
python3 -m http.server --bind 127.0.0.1 --directory target/bondline-web 8080
```

Open `http://127.0.0.1:8080/`. The dev profile is the executed workflow; optimized release
packaging remains unverified.
The packager first runs the strict four-target license gate. It retains the actual wasm
runtime notices and font license in `THIRD-PARTY-NOTICES.txt`; missing or changed source
notice pins fail packaging. The output contains local static assets with a bundle/hash
manifest. After the first
successful service-worker installation, the same origin can reload offline. A stable
localhost port or HTTPS origin matters: different origins have different browser drafts.
No account or hosted application service is needed. Bundle updates wait until existing
tabs close, so an active editor keeps its matching JS/WASM assets.

A bounded1MiB CDXML draft is retained automatically after real model changes. It is browser
recovery, not a local-file Save receipt or a permanent backup. The page holds one browser
writer lease. Another tab, blocked/quota-limited storage or an externally replaced draft
cannot silently replace previous data. Corrupt/unsupported old data stays preserved.
Unsaved in-tab edits retain the browser's unload prompt. Undo history and view state start
fresh after reload; browser data-clearing/private modes follow browser storage policies.
New/restored browser sessions start at100%. The visible View menu offers Fit to window
and Actual size; Actual size recovers readable ink when Fit is below the renderer limit.
Explicit user zoom is retained within the current session, including window resizes.

`window.bondline.execute(JSON.stringify({command:'document.inspect',params:{}}))` calls the
real registry. `registry()`, `canvas_geometry()` and `host_status()` expose bounded host
inspection. Requests are bounded before UTF-8 ownership. Native-only operations refuse;
they do not gain file authority through the browser adapter. Future import/export and
broader browser tool/platform acceptance remain open. Full M0–M5 is incomplete.

[Research/decisions](../docs/research/web-workspace.md) distinguish the current SDK/runtime
compatibility fixes and actual browser evidence from unverified platform capabilities.
GitHub Actions remains disabled during development.

The repeatable acceptance workflow uses a fresh owned headless Chrome profile:

```sh
node xtask/oracles/web_workspace_browser.mjs target/bondline-web target/web-workspace-browser
"$BONDLINE_PYTHON" xtask/oracles/web_workspace_source.py target/web-workspace-browser/drawn.cdxml
```

`BONDLINE_CHROME` may select another Chrome executable; `BONDLINE_PYTHON` must be the
repository's pinned RDKit2026.09.2 oracle Python. The test checks actual WebGPU painting,
pointer/keyboard/history, the real Analysis window, independent CDXML, offline reloading
with all local server requests rejected,1160×800/660 viewports, quota errors and competing
tabs. It uses the public CDP interface and no Node packages. Browser accessibility, other
engines/hardware, release optimization, native file workflows and complete M5 are open.
