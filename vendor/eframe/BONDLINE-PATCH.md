# Reviewed eframe fork

Upstream: emilk/egui 0.29.1, crate archive SHA-256 recorded in docs/dependency-policy.md.
License: MIT OR Apache-2.0, upstream notices retained next to this file.

Cargo.toml no longer enables egui-winit's `links` feature.
That feature pulls in a native browser launcher, URL/IDNA processing and Unicode-licensed
runtime data. Bondline does not yet offer external-link launching. Platform text clipboard,
accessibility and GPU backends are retained. Bundled upstream default fonts are disabled
by Bondline's dependency features, not relabeled. This is the same pinned fork mechanism
used by PhotoCraft, with Bondline's narrower runtime license policy.

The private, unused DummyStorage type and its implementation were deleted. It had no
callers in the published sources and caused a dead-code warning on the pinned compiler.

The shared native `Integration::on_window_event` now preserves actual modifier reports
in egui's ordered queue. It delegates once to the unchanged egui-winit translator, appends
the freshly mapped snapshot only for Winit ModifiersChanged, and returns the original
EventResponse. WGPU/Glow use this existing common route and per-viewport State. There is
no new egui-winit fork, duplicate platform mapping, invented key/reset, web emission,
dependency, feature, unsafe code or change to old global modifier/gesture semantics.

The [published integration source](https://github.com/emilk/egui/blob/fe368bacc4a15697e347607e73a56c0299b3d42d/crates/eframe/src/native/epi_integration.rs)
was13122B SHA256`93a20974162d0dd25ffecae4f2d6fb9af95de6282488b5965481f5e33f7db8b2`;
the patched13493B is`9aa4c94e17378f057c43514161a0646cb515d7b2f42adae63d119da72a40ab94`.
Read the [contract](../../docs/research/ordered-native-modifiers.md) and
[actual RED/GREEN limits](../../docs/verification-ordered-native-modifiers.md).
