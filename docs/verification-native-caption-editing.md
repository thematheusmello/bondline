# Editing existing captions in the app

Base `f3d81ee`, 2026-11-19. The
[public-source contract](research/native-caption-editing.md) separates the
version 25 Text workflow from Bondline's painted-run and draft rules.
The independent file witnesses are the existing
[CDX pairs](../crates/io/tests/fixtures/cdx-caption-edit/SOURCES.md) and
[explicit CDXML caption](../crates/io/tests/fixtures/captions/README.md).

## Independent checks

- Render tests use FontTools outline interiors and an analytic Roboto H
  stem to check half-open pixel cells, overlap, blanks, post-edit run
  positions, and identical paint and pick clipping. The cropped, aspect
  changed and fractional-density cases use independent coordinates.
  CPU and GPU pixels match at 1.5×. The old render tests and goldens pass.
- Command tests exercise attached-host discovery and precise request IDs,
  no-op/history, exact CDX bytes, a bounded draft, and hidden File
  Open/Cancel and host replacement. The authenticated socket test checks
  post-publication repaint for Select, Open and Cancel before applying and
  saving the independently expected CDX bytes.
- App frames test topmost overlap and ordinary run clicks at 1× and 2×,
  source and camera replacement, paper/atom/bond/drag misses, Apply,
  Cancel, no-op, Undo/Redo and genuine file-picker Save As and Open. The
  UTF-8 CDX tail and escaped CDXML text save to Unicode filenames and
  reopen. Independent expected bytes establish file output.
- The ordinary `t` shortcut enters Text and the draft owns subsequent
  typing. Bounded ordered text before Apply determines its edit. A
  rejected Apply retains a later bounded replacement for explicit retry.
  Wheel panning in Text mode uses the existing camera.

## Test-first record

The three JSON command tests first fail at unknown `ui.caption`. The
five renderer tests first fail to compile at the missing hit API, then
pass with the same declarations. The entire renderer suite passes
200 tests. Initial App setup corrects AccessKit's action name to
`Default`. The next executable run fails four cases before wiring the
editor. One report records an unintended chemistry edit from the
retained Bond tool. An initial App witness misses an orange glyph at
both densities. The locked independent Ω interior at (188, 216.5) pt
supplies the intended painted run with the same expected target.
The intentional shortcut RED and rejected-Apply RED each precede
their change. The first wheel assertion expected zoom. The qualified
wheel test checks exact pan center and epoch, fails before routing
wheel to the shared camera, and passes afterward.

The first aggregate stops at an older CDX resource test that compares
the entire registry with a 49-record fixture. The second reaches the
App suite and stops at two older rail tests that count Text as planned.
The qualified comparisons retain the exact historical command prefix
and verify the added command explicitly. The rail tests verify the Text
button's unavailable state separately from the 123 planned entries.

To reproduce the focused checks with the
[pinned Python environment](development.md):

```sh
cargo test --locked -p bondline-render --test caption_hits
cargo test --locked -p bondline-commands --test ui_caption
cargo test --locked -p bondline-ui --lib native_caption_tests::
cargo test --locked -p bondline-mcp --test caption_control
cargo xtask ci
```

Local receipts are under `target/native-caption-editing`.

The rebuilt CLI returns 50 records with the exact previous 49-record byte
prefix. Initialized stdio MCP 2024-11-05 returns those names and schemas,
and refuses the host-only Select on its headless engine. The CLI registry
SHA-256 is `7152f0e0470bd4410f6aeeb736e5a6074e2c63d96fe951710d23948ea2bd4ba5`.

## Final local gate

`cargo xtask ci` exited successfully on `b26f5d5`. The workspace
passed **2,280 tests** with no failures and one existing ignored case
across 339 reports. AccessKit passed three tests. Formatting,
warnings-denied Clippy, layering, licenses, seven assets and the
pinned eight-file corpus passed. The 286,311-byte
`ci-qualified.log` SHA-256 is
`0c2c1cc0b06cf52ea3d6a3d3ef6b9742ea5a79b38fbeaa5e63a7d1eb1180cf23`.

Command crate type-checks passed on Windows MSVC, Linux GNU and
WebAssembly. AddressSanitizer completed 10,000 CDX runs in one second
with 211 unchanged input files. The largest recorded new or reduced
mutation was 894 bytes under a 65,536-byte limit. The `fuzz.log`
SHA-256 is `e1712e2724aef645fb33cddc5d170726dcb704e75d482760fb95f92ca0346e19`.
All owned verification jobs exited. GitHub Actions remained off.

## Coverage boundary

The form edits a single existing plain run. Painted pixels determine
the native hit and a blank run has no pointer target. Creation, line
controls, typography, chemical formatting and general foreign source
styles remain separate work. App-frame density and keyboard cases
exercise the App implementation. Physical display, live platform input
and exact Desktop 26.1 interaction require separate evidence.

The isolated signed Mac review built successfully. The launch guard
observed recent desktop input and kept the review unlaunched. The
private work area contains the original independent CDX witness and
the guarded launch receipt. New OS-level input remains pending.
