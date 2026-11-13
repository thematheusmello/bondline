# Bondline

An independent, offline chemical drawing application being built in Rust.

The compatibility target includes CDXML and CDX files used by ChemDraw Professional.
No application file-open/save or chemical drawing workflow is implemented yet. The standalone
CDXML crate preserves unchanged syntax; see [tested format scope](docs/file-formats.md).
Bondline is not affiliated
with, endorsed by, or sponsored by ChemDraw or Revvity Signals.

Current capabilities: native page sizing/rotation and Undo/Redo, independently sourced Apache-2.0
font, original icon, shared CPU/GPU page-scene reference, and strict native runtime/asset
license gates. Native macOS launch was visually inspected; Windows/Linux are cross-checked,
with native runtime validation pending. This is a development foundation, not a chemical editor yet.
The shared registry and CLI can inspect an in-memory page, set its dimensions, undo/redo,
and capture its shared scene as a CPU or GPU PNG with revision/backend evidence.

```bash
cargo run -p bondline-ui --bin bondline
cargo run -p bondline-ui --bin bondline -- --smoke-test
cargo run -p bondline-cli -- commands
cargo run -p bondline-cli -- run --cmd '{"command":"page.set_size","params":{"page_id":1,"width_pt":612,"height_pt":792}}'
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all -- --check
cargo xtask licenses
cargo xtask layers
```

The smoke command launches the actual wgpu window, reports its adapter, and closes after
three frames. [The dependency policy](docs/dependency-policy.md) records pinned forks,
original compatibility adapters and verification limits. There is no license exception
for ISC, Boost, Zlib, Unicode or OFL. The hypothetical browser graph is not yet approved.
Agent clients can launch `cargo run --quiet -p bondline-cli -- mcp` as a stdio subprocess;
it opens its own in-memory document. See [the command contract](docs/control-protocol.md).
For the active desktop document, launch with `--control-port 0`; the app reports and displays
its loopback address. This mode is opt-in and uses bounded newline JSON requests.

Current work: licensed corpus/CDXML, CI and the public menu/tool/shortcut inventory.
Workspace PNG tests need the external Pillow oracle described in [development.md](docs/development.md).
See [STATUS.md](STATUS.md) for actual progress and [ROADMAP.md](ROADMAP.md) for the full target.

Original code is dual-licensed MIT OR Apache-2.0. Third-party notices remain with their files.
