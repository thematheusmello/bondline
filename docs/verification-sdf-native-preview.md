# Registered SDF preview UI wiring

2026-11-14. The actual native application now schedules read-only SDF capture through
the existing registry. This record includes component, independent texture/paint acceptance and the full local
receiving gate below. No native
window or input was used because the user remains active. Actions stays off.

## Implementation boundary

BondlineApp::update_workspace is the real frame body, delegated to by eframe::App::update.
It polls file work, synchronizes preview state under the Engine lock, paints controls,
and drains close only when file/executor/preview work and pending intents are absent.
After releasing that lock, it starts accepted file work and registers pending preview
work before its thread is spawned. The old no-executor public workspace helper retains
its metadata summary; it is not the native preview acceptance path.

One weak-content/revision/generation/close-epoch/view/DPI key owns at most one pending
intent, one actual worker and one ready texture or keyed error. View dimensions use one
uniform physical scale, capped by2048 and the current texture limit. Resize, source/field/
history/reopen/DPI/close changes clear old pixels/errors immediately and cancel work.
Install requires the current complete key and Open close state. Close Cancel cannot revive
an older job because close_epoch is part of the key, as well as the cancellation token.

The worker uses UiCaptureWork, the bounded own-PNG bridge, existing native
eframe::icon_data::from_png_bytes and ColorImage conversion. All projection/render/PNG/
decode/color work stays off the UI/Engine lock; the UI only installs a matched texture.
Main-label readability uses the actual renderer em times actual logical/image ratios,
retaining8..256 logical units rather than treating device pixels as logical points.
No second SDF parser, chemistry renderer, loader dependency or thread pool was added.

The job remains busy through command completion, decode, channel publication and actual
thread retirement. The UI joins only after JoinHandle::is_finished; a nonfinished job
causes a bounded repaint request, not a blocking join. Cancellation keeps the real job
until retirement. Busy preparation retries without spawning a thread or caching a source
failure. Cancel/Retry and the Records disclosure are original transient presentation
controls; their broader public control-addressability remains part of the incomplete M0
inventory contract. Neither preview pixels nor IDs grant atom/bond/Save authority.

Unsupported data/ink/readability errors remain keyed and explicit; no record is hidden
and no stale image is used as fallback. Normal close drains workers. Forced host destruction
cancels any remaining job cooperatively; it does not promise instantaneous interruption
of existing parser/decoder calls or block the UI joining unfinished work.

## TDD and scoped evidence

Root session15891 exits101 for the missing PreviewKey API, before implementation. This
is a compiler RED, not an invented native/image failure. Session90506 passes the three
component cases for uniform physical sizing/full logical identity, invalid inputs and
close/cancel epoch changes without authored edits. No production correction followed.

Package29922 passes101 UI tests, warnings-denied all-target clippy and rustfmt. Existing
UI tests/fixtures remain unchanged; the old static helper is explicitly separate from
the native path. Logs/source hashes are in target/preview-ui-wiring-receipt.json. The
three component tests do not establish actual texture installation or native behavior;
the independent acceptance owner tests the real frame and real registered image next.

Read-only source review found no confirmed material lifecycle defect; it is separate from
runtime evidence. During review Root corrected deferred busy repaint and precomputed the
available rectangle before its mutable UI call. No chemistry, source codec, dependency,
font, old oracle or approved golden changed in this wiring slice.

## Public engineering sources and limits

- [Pinned eframe0.29.1 PNG helper](https://github.com/emilk/egui/blob/0.29.1/crates/eframe/src/icon_data.rs)
  and [pinned Context texture API](https://github.com/emilk/egui/blob/0.29.1/crates/egui/src/context.rs)
  were inspected alongside actual vendored/locked sources. They support the decoder and
  owned texture path; the UI supplies its own dimension/resource checks before calls.
- [Rust JoinHandle documentation](https://doc.rust-lang.org/std/thread/struct.JoinHandle.html#method.is_finished)
  supports nonblocking completion checks before join. The live page is1.100.0; the pinned
  1.94.1 page was inaccessible. Actual code/tests compile with installed1.94.1, not an
  inferred library-version upgrade. Dropping a handle detaches a thread; normal close
  therefore retains and retires it explicitly.
- The8..256 main-em, view-key/epoch/packing/cancellation and original control policies are
  Bondline decisions, not claimed reference-product defaults. Existing BIOVIA/IUPAC/H
  sources and display-profile limits remain in the linked source/renderer records.

No native, Windows/Linux, hosted workflow or new golden approval follows. Full M0/M1 and
M2–M5 remain incomplete; current CI/fuzz/interop evidence retains its prior exact scopes.

## Receiving frame and aggregate evidence

[Independent acceptance](verification-sdf-native-preview-tests.md) passes all six fixed
cases unchanged at20466, with107 UI tests/clippy/fmt at37563. Actual registered source
pixels are independently decoded and compared to the uploaded opaque ColorImage; that
same texture must occur in the actual native-frame mesh/UV/clip instructions. Real staged
success/decode-error/close-epoch jobs retain lifetime gates through thread retirement.
No expectation, probe, threshold or production correction was needed after first compile.

Root source62e9160 passes default cargo xtask ci66690:822workspace+3Mac bridge=825,
0failed/ignored,129 result lines. Fmt/clippy/layers/unchanged runtime closures+two assets/
eight-file corpus pass. Log99164B/SHA
`a077c4974b180f1be9eea0d246a2e591857472e788ad2ef7731f8f82b5aced92`.
All541 frozen non-Markdown workspace/source/test/config/assets remain exact; isolated
fuzz/future disabled-workflow tooling has separate gates afterward. This preserves the
standard default parallel test settings. The earlier unexplained repeated MCP shutdown
failure remains documented; a later pass establishes no cause or fix.

Root verified core5files/151protected/5logs/4render inputs, work4files/664protected/
11inputs+11logs, acceptance2files/536exact+8received+6newbackend/4logs and module bridge.
A receiving script initially misclassified that bridge; premature CI8628 was stopped130,
its27782B/SHA312ac7d76ea87078ec7fd566e03087a2a6bc2c0b4263bb309c52ec224f9da97b log
retained, and the corrected real receive preceded fresh825 gate. It is not a passing run.

The new capture-envelope fuzz target additionally passes a bounded10000-run ASan smoke;
see verification-capture-image-fuzz.md for its distinct corpus/tool/decoder limits. No new
native-window/input/Windows/Linux/hosted execution or golden approval occurred. Native
preview is wired/headlessly tested, with actual OS workflow acceptance still unverified.
