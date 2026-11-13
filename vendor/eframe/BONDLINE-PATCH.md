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
