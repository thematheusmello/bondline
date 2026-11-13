# Dependency policy and graphics evidence

Original code: MIT OR Apache-2.0. Shipped runtime code and bundled assets must have an
applicable MIT, Apache-2.0, BSD-2-Clause or BSD-3-Clause permission. A disjunction needs
one allowed choice; a conjunction must satisfy every requirement. No ISC, Boost, Zlib,
Unicode or OFL exception is silently added. Metadata declarations are checked separately
from source/asset provenance and notices; a label alone is not a legal/source review.

## Reference research

PhotoCraft's Cargo.toml uses exact egui/winit fork revisions. Its ATTRIBUTION.md documents
ISC icons and OFL fonts, so copying its dependency set would not meet this policy.
VoiceStudio's model-review workflow distinguishes declared licenses, exact bytes, review
state and fingerprint changes; its AGPL code is not reused. Bondline adopts the reviewed
fork/pin and evidence practices, not those projects' broader permitted license sets.

## Native solution, 2026-11-13

- eframe/egui 0.29.1 and wgpu 22.1.0 are compatible with Rust 1.94.1. The newer graphics
  families introduce more disallowed runtime components. Every eventual upgrade reruns gates.
- `vendor/eframe`: MIT OR Apache-2.0. Published archive SHA-256:
  `8ac2645a9bf4826eb4e91488b1f17b8eaddeef09396706b2f14066461338e24f`.
  Disable only the optional native external-URL launcher, eliminating URL/IDNA Unicode data.
  Keep accessibility and native text clipboard. Delete orphaned private DummyStorage.
  Full upstream licenses are adjacent; see BONDLINE-PATCH.md for the exact changes.
- `vendor/winit`: the unchanged 0.30.13 package plus one macOS application-delegate
  callback, retaining original notices. Archive SHA-256 and VCS identity are recorded in
  its BONDLINE-PATCH.md. Native Quit now requests cancellable client close approval; its
  two standard main-thread AppKit actions are the isolated reviewed unsafe extension.
  No graphics version, declared license, font or transitive package is changed.
- `vendor/accesskit_macos`: exact 0.17.4 with a bounded editable-string SetValue bridge.
  MIT/Apache source and the Chromium-derived BSD notice are retained; missing archive
  notices were fetched as exact blobs from its published VCS revision. Three source files
  change; no public API/dependency upgrade. Its BONDLINE-PATCH.md records archive/notice hashes,
  new unsafe NSString checks, tests and native evidence. The derived BSD notice is additional
  to its manifest's declaration, not replaced by it.
- `compat/libloading`: original API adapter over MIT `dlopen2 0.9.0`. It does not copy
  upstream ISC code. Preserve symbol lifetimes and lookup errors (including EGL version
  negotiation), validate pointer sizes/C strings, and support native platform loader flags.
  Windows already-loaded lookup owns the GetModuleHandleExW reference directly; it never
  reopens a name through the loader search path. GetProcAddress borrows that live reference.
- `compat/hexf-parse`: original exact IEEE-754 hexadecimal conversion from published WGSL
  literal rules. It does not copy CC0 code. Inexact, overflow and underflow values fail.
- `compat/clipboard-win`: original Win32 adapter for the pinned Arboard consumer, replacing
  Boost code. Transaction ownership uses a thread-bound message window. Global allocations
  are bounded, locked and transferred/freed with RAII; text and HTML offsets are validated.
- Pin gpu-descriptor 0.3.0 in Cargo.lock to retain its allowed hashbrown dependency. 0.3.1
  still brought in foldhash and was rejected by the Windows audit. No fake license labels.
- Disable eframe default fonts. Bundle only Apache-2.0 Roboto from googlefonts/roboto-2
  revision `38062f4b4a0be4346d07a928408da21602545e9e`; SHA and notice are in assets/manifest.json.
  Bondline's window icon is original procedural geometry.
  The unused eframe fallback icon remains part of the pinned library source; its upstream
  MIT/Apache declaration, bytes and notice are also checked in the asset manifest.

OS ABI calls are the only new unsafe exception, isolated in the loader and Win32 clipboard
adapters, the pinned winit macOS quit callback and AccessKit string-value bridge. All carry safety reasons;
Bondline's UI/model/format/command crates continue to forbid unsafe.
Adapters deliberately implement the inspected pinned consumers' surface, are not published
as general replacements, and need reinspection when those consumers change.

## Verification scope

`cargo xtask licenses` resolves `cargo tree --target ... --edges normal,no-proc-macro`
for each native product target, then evaluates those packages' SPDX declarations. This
avoids metadata's feature overapproximation. Build tools and unselected target dependencies
are not shipped runtime code. Never report a declaration check as a full source review.
Product binaries are discovered from Cargo workspace membership/targets; the current audit
covers both the desktop and CLI roots, including the shared command engine.

Native license closure has passed on macOS arm64, Windows x64 MSVC and Linux x64 targets.
Session authentication pins getrandom 0.3.4 (MIT OR Apache-2.0) with its std feature and
subtle 2.6.1 (BSD-3-Clause), default features disabled. Only subtle adds a new lockfile
package; getrandom was already resolved. Their published APIs, error behavior and license
declarations were inspected. The source-backed io geometry adapter now reaches the existing
standalone CDXML crate and its pinned MIT/AP2 roxmltree and MIT quick-xml packages from
product roots. Native file I/O now reuses rustix 1.1.6 and adds macOS-only exacl 0.13.0 plus
Windows-only winsafe 0.0.29 and windows-permissions 0.2.4. They supply fallible OS wrappers;
no unsafe is added to io and Linux does not link libacl. Exact archive/VCS pins, author and
the missing windows-permissions notice are recorded in native-file-io.md. Its published
MIT declaration passes the existing declaration gate; the missing LICENSE is a release
notice-audit gap, not a fabricated notice or completed source audit.
Latest runtime closures are 153/164/219 packages for macOS/Windows/Linux,
plus the same two checked assets. This retains the original allowlist.
Hosted aggregate/window checks pass on all three targets at product checkpoint ccf8c50.
File I/O has native Mac/Windows and Linux Docker/hosted execution; interactive Windows/Linux
file gestures remain unverified.
Source cross-checks alone are not OS execution. The WebAssembly delivery remains M5: its current
candidate graph includes Unicode-licensed identifier data and is not approved for shipment.
