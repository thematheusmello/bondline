# Actual application-frame offscreen capture

2026-11-14. Development-only capture of the actual `BondlineApp::update_workspace`
output at1600×1000physical pixels, DPI1. No native window, OS input, activation,
dialog or hosted workflow is involved. This captures the complete application
client frame, not the operating system's title bar. It is not a mockup or a new
approved golden. The actual capture passed and was visually inspected as a local
image. It shows the current controls and literal-H Single/Double/Triple structures
on white paper, with Double selected and actual status guidance.

`crates/ui/src/visual_capture_tests.rs` is an ignored-by-default test. It builds
the ordinary App fields, uses the existing font/theme configuration, and creates
an original source-free420×280pt page and three separate Single/Double/Triple
carbon fragments through the shared registry. It selects the existing Double
tool. Three actual egui frames settle layout/font textures. The final frame must
contain the real brand/File/Edit/Page controls, visible paper and exactly six
actual chemical stroke polygons; drawing frames must preserve the Engine state.

The helper consumes all warm-frame font texture deltas, tessellates the actual
final shapes, and passes them to the existing `eframe::egui_wgpu::Renderer` and
wgpu22.1.0. Rendering uses RGBA8Unorm, one sample and native-default dithering.
The API requires the safe `RenderPass::forget_lifetime` conversion; the pass is
dropped before copy/readback. Adapter selection and readback have ten-second
bounds. There is no new dependency, production capture command or shipped API.
The exact APIs were inspected in the pinned egui-wgpu0.29.1 renderer, eframe's
existing reexports/native options and Bondline's existing GPU readback code.

With the pinned external interpreter selected through `BONDLINE_PYTHON`:

```bash
CARGO_TARGET_DIR="$PWD/target" CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test -p bondline-ui --lib visual_capture_tests::actual_application_frame_to_png --locked -- --ignored --exact
```

Default outputs are ignored `target/visual-capture/current-app.rgba`, `.png` and
`.json`. Raw output is create-only; a later capture requires a fresh bounded
`BONDLINE_VISUAL_CAPTURE_NAME` containing letters, digits or hyphens. Existing
pinned Pillow12.3.0 converts only the actual readback bytes, verifies an opaque
frame and reopens PNG to compare every RGBA byte. The JSON records actual adapter,
dimensions, render settings, registry seed commands, complete current state and
encoding hashes. No control listener/token exists in this capture.

The six previously frozen multiple-bond acceptance tests, existing production,
oracles, fixtures, fonts, notices and approved golden remain protected. Native
interaction stays paused while the user is using the desktop. This artifact can
show current appearance; it does not verify native event delivery, new workflows,
reference-product visual parity or complete M0–M5.

## Actual artifact and evidence

Capture session17183 exited0:1pass/0fail/69filtered. The actual adapter reports
Apple M3 Max, Metal, IntegratedGpu. The GPU-encoded1600×1000 frame was converted
by the existing external Pillow12.3.0; reopening PNG reproduced all6,400,000 raw
RGBA bytes exactly. This is not a timing or performance claim.

| Ignored artifact | Bytes / SHA-256 |
|---|---|
| `target/visual-capture/current-app.png` | 20656 / `91dfefbf92fac7d3c7613d1f820a93eebc12e5f143974d77656c3208a9d8349f` |
| `target/visual-capture/current-app.rgba` | 6400000 / `87e643cad16bc2f8e73cb30a40fdcd09341ecc5f4ab045b8be9e9924fcd94c65` |
| `target/visual-capture/current-app.json` | 5421 / `c2d0209f307ae7669bc0beb5f2d6ca3e8878f1b8c8262dcf0af1c1f9e8063a2a` |
| `target/visual-capture-second.log` | 441 / `34fa8b0a88cd7157f1e8704f15b89b8aa6c5b28611bd31289596b62dc13ff2b0` |
| `target/visual-capture-clippy.log` | 178 / `459024ac30098438174bcc6b190c0598fa40e6500c4e0f03935adfe2185af295` |

The first explicit run63603 compiled but failed before GPU execution because
the new harness incorrectly expected one white rectangle with a gray stroke.
Actual existing `page_scene` emits separate gray196 border and inset-white
rectangles; the actual UI paints both using `rect_filled`. Only the new harness
was corrected to require the large gray border and exact one-pixel white inset.
The page dimensions, six real component polygons, chrome, state invariance and
pixel/readback requirements remained. This is a harness setup correction, not a
RED→GREEN claim for previously missing application behavior. Initial source
12,628bytes/SHA-256`a09ba456334eda34a330d0944c43db539bb7c985c0ca513a5097a331b32f2820`
and the913-byte failure log/SHA-256
`ac7311ea0b454c9f3427406e21b022b83976477849c7feecc3da7aa105551400`
are retained under `target/visual-capture-*`.

The corrected capture helper is12,949bytes, SHA-256
`575a6bfb18837a0fd877c1a646c7498fe18b0ccd74147011e61abb382f520a09`.
All-target UI clippy session72117 exited0 with no warnings. The initial package
fmt check reported only the Root-owned module bridge's ordering relative to
`mod preview`; no body or owned-file formatting error. Its360-byte failure log,
SHA-256`7dc26467e726e71a56f763a6d8af1d3b3f63b2c7271e9b31f76132a983df3b21`,
is retained alongside the final formatting correction/check.

Root then moved only the new test module declaration after `mod preview`.
Final UIlib is50,598bytes, SHA-256
`1ec6266fdd544dff25380973498bd0e420bce386521205261aebba8c1bf0e5bb`.
Removing only the added test-module bridge reconstructs exact prior production
source`d57cc3bf6506c06bfdf90397c8dbc29c3db7c62f23e3c24d010c2c622a35e117`.
Final package fmt session70611 exited0; its log is empty/SHA-256
`e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.
No recapture is needed for that declaration-order correction. The PNG and helper
retain their exact hashes above; all jobs are terminal.

The final ignored `target/visual-capture-handoff.json` records both owned files,
all artifact/log/receipt identities,708 exact protected baseline files and the
sole Root-owned module bridge among709 original paths. No baseline file is
missing and no unclassified new source exists. No old acceptance assertion,
fixture, oracle, font, notice, lock or approved image changed.

## Updated workspace capture

Root subsequently supplied the actual styled workspace source, with the same
capture-only module bridge. Styled UIlib SHA-256 is
`d02f88f72d5839719d3ca2d378e78b536b6c49fcac5310ab27eed169d9fef5b7`;
removing only that bridge reproduces exact Root runtime source
`540d4e0893ae438a0140480c098418b6511da19d8bb3deb3db170058659693da`.
Root's new presentation tests are10,894bytes/SHA-256
`0994a92f350e1c4de5a77ae24456da2d166998b4133a87d15fc9bc303b03a795`.
The468-byte port receipt is
`target/visual-capture/styled-source-port.json`, SHA-256
`f8a345230536f418d8c59a32a8ad1ef03c37a2f28732fb5ea5d39098384e449b`.
The original six multiple-bond tests and capture helper remain byte-for-byte
unchanged. This worker made no style or product change.

The same explicit capture command with
`BONDLINE_VISUAL_CAPTURE_NAME=updated-workspace` passed on its first styled run,
session98327:1pass/0fail/71filtered. The actual1600×1000 frame was inspected as a
local image. It shows the real light header, teal branding/Double selection,
compact Tools rail and Document inspector, including the actual file/page and
6atoms/3bonds/3fragments counts. The original white chemical paper, literal H
labels and actual status guidance remain visible. Adapter/settings/Pillow are
unchanged. Both captures contain exactly identical registry seed commands and
authored Engine state; this was checked against their actual JSON receipts.

| Ignored styled artifact | Bytes / SHA-256 |
|---|---|
| `target/visual-capture/updated-workspace.png` | 47207 / `ac9c56efe350e5019e83b167ee5e5483d856deb23fa59c43e8a57a6df52f7f1c` |
| `target/visual-capture/updated-workspace.rgba` | 6400000 / `680b0d06f7d18b0c57915b1595ac4712691cdc563b8841d812a105c98a32ccdf` |
| `target/visual-capture/updated-workspace.json` | 5427 / `594f0d8c997310c3cbeb6c490d9ad3cac54aefcf0fd95bdaecadba668234b879` |
| `target/visual-capture-styled-first.log` | 441 / `66489441ea18e5a0a6a8a61ff5cb638cae43a60490538727db427f414524b527` |

All three earlier `current-app` artifacts retain their original exact hashes.
No recapture overwrote them, and no native window/input/activation or hosted
workflow ran. The styled capture verifies actual rendering/source invariance;
Root owns the separate presentation/gesture/workspace quality gates. It grants
no new golden approval or reference-product visual-parity claim.

Package fmt --check also exited0 on the received styled source; its log is empty.
The original6,332-byte verification record is retained as an exact prefix copy in
`target/visual-capture-initial-verification.md`. The separate styled handoff is
`target/visual-capture-styled-handoff.json`. All capture jobs are terminal.
