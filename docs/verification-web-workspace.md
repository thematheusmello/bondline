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
