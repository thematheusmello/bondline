# Integer-bond ring rendering control

2026-11-15, Apple M3 Max / Metal, Rust1.94.1. This checks the existing renderer before
aromatic authoring implementation. It passed first; it is not a reproduced renderer bug.

`crates/render/tests/kekule_ring_control.rs` authors an original regular six-carbon ring
with64pt sides, fixed H1, alternating Single/Double and A4 paper. Independent analytic
projection checks every stroke, nine black strip corridors, three white inter-stroke
gaps, atom/bond picking, complete CPU/GPU RGBA equality and unchanged model data.
It reuses the existing SVG-STROKES/WGSL-COVERAGE pixel conventions; no source-product
style, font, icon, template or new golden is adopted.

Run with CARGO_INCREMENTAL=0, CARGO_BUILD_JOBS=2, DEV/TEST_DEBUG=0:

```sh
cargo test -p bondline-render --test kekule_ring_control --locked -- --nocapture
cargo clippy -p bondline-render --test kekule_ring_control --locked -- -D warnings
```

First test session11410 exits0:1 passed/0 failed,0.25s test time. Clippy37921 exits0.
The frozen8535-byte test remains SHA256
`95487817359d1f9a4262a33921a2c8806661855256670789a6d0314050d28f9f`.
Test log614B SHA256`fabf57b494416f9eca8df4acdc94a0af166596ca69f278784f698cb3932b09ed`;
clippy log764B SHA256`b2a1c0f0545a77c2752ffd972748e5e02585fd9475fb0af41d3f1d2ac5aa21a4`.
Ignored receipts/logs are in target/aromatic-authoring/render-control/.

The control justifies using the existing integer-bond depiction for the separately
specified [authoring projection](research/aromatic-authoring-contract.md). It proves no
aromatic chemistry, layout, editable paste, native clipboard, preferred reference ring
style, native DPI or cross-platform behavior. The earlier full workspace gate's MCP
replacement-bind failure remains unresolved; this targeted pass is not an aggregate pass.
