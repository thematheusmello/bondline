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
