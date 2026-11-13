# Bondline

An independent, offline chemical drawing application being built in Rust.

The compatibility target includes CDXML and CDX files used by ChemDraw Professional.
No file compatibility or chemical drawing workflow is implemented yet. Bondline is not affiliated
with, endorsed by, or sponsored by ChemDraw or Revvity Signals.

Current capabilities: a native read-only blank workspace, independently sourced Apache-2.0
font, original icon, CPU/GPU clear-and-readback reference, and strict native runtime/asset
license gates. Native macOS launch was visually inspected; Windows/Linux are cross-checked,
with native runtime validation pending. This is a development foundation, not a chemical editor yet.

```bash
cargo run -p bondline-ui --bin bondline
cargo run -p bondline-ui --bin bondline -- --smoke-test
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all -- --check
cargo xtask licenses
```

The smoke command launches the actual wgpu window, reports its adapter, and closes after
three frames. [The dependency policy](docs/dependency-policy.md) records pinned forks,
original compatibility adapters and verification limits. There is no license exception
for ISC, Boost, Zlib, Unicode or OFL. The hypothetical browser graph is not yet approved.

Current work: extend the test-first foundation with the document/command engine and CDXML.
See [STATUS.md](STATUS.md) for actual progress and [ROADMAP.md](ROADMAP.md) for the full target.

Original code is dual-licensed MIT OR Apache-2.0. Third-party notices remain with their files.
