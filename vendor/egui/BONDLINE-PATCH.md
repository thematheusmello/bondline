# Reviewed egui focus fix

Upstream: emilk/egui0.29.1, MIT OR Apache-2.0. Published crate archive340450B,
SHA-256`53eafabcce0cb2325a59a98736efe0bf060585b437763f8c476957fb274bb974`.
VCS`fe368bacc4a15697e347607e73a56c0299b3d42d`, path`crates/egui`.

All83 published `src/`, normalized Cargo.toml and README files were copied exactly,
then one guard changed in `src/memory/mod.rs::Memory::set_focus_lock_filter`: the
current focus owner may install its filter during its first focused pass. The previous
`had_focus_last_frame` condition discarded that first installation. An immediate arrow
then changed both slider value and spatial focus, so the opposite arrow missed the slider.
No public API, feature, dependency version, license label, asset or unsafe code changes.
Root Cargo.lock only routes the existing0.29.1 package to this reviewed path.

Published memory source SHA
`adbfbf412fc9201f7ca2f10795383d2ffe85c61afe9528bbd61e4d031225297c`;
patched SHA`62899ec4b99713a47341ea680fd458c703e41934bc300fc49e82a02fe6e22d46`.
See the [exact public source](https://github.com/emilk/egui/blob/fe368bacc4a15697e347607e73a56c0299b3d42d/crates/egui/src/memory/mod.rs).

The archive omitted full license files. Adjacent LICENSE-MIT and LICENSE-APACHE were
fetched unchanged from that same published VCS revision, preserving copyright and notices:

- [MIT](https://github.com/emilk/egui/blob/fe368bacc4a15697e347607e73a56c0299b3d42d/LICENSE-MIT),
 1092B/SHA`95ca92f5f8ea5231f1580b3a2a799e8260af3114b900e1def5355a7f44bcf60c`.
- [Apache](https://github.com/emilk/egui/blob/fe368bacc4a15697e347607e73a56c0299b3d42d/LICENSE-APACHE),
 10850B/SHA`8173d5c29b4f956d532781d2b86e4e30f83e6b7878dce18c919451d6ba707c90`.

Independent bare-egui evidence: unpatched desired assertions70877:1passed/1failed;
unchanged assertions25670:2passed after the guard correction. Root's unchanged actual
Bondline pan test also reproduces the failure before the patch; all9 view regressions
now pass in47749. `crates/ui/tests/slider_focus.rs` retains the independent immediate-
focus case and settled-focus control using only the already reviewed Apache Roboto font.
These are deterministic SDK/UI checks, separate from native-window execution.

Ignored upstream receipts/probes/logs are disposable under `target/view-egui-upstream/`
and the research worker's `target/view-navigation-review/`; do not ship credentials or
claim this is an upstream accepted fix. Future upgrades must rerun these regressions.
