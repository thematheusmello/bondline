# Direct-Page CDX caption verification

The [caption profile](research/cdx-caption-preview.md) reads a source-qualified
caption into the existing Page and StyledText model. It keeps the original binary
bytes, the model derived from them and a separate read-only display proof.
The shared renderer still checks each view. A detached model edit retires the
proof and exact Save until the model equals the baseline again.

## Independent sources and runtime tests

[Original fixtures](../crates/io/tests/fixtures/cdx-caption/SOURCES.md) freeze 12
positive models and 63 legacy refusals. Before production, the actual CLI refused
all 75 inputs. The first IO test build used private scene fields, so the test was
corrected to use the public scene, hit and text range methods before the runtime
probe. The resulting existing-API test run passed the old refusal test and failed
three Open/render/Save/drift tests. After production those same four tests passed.
They verify every complete positive model, all 63 historical codes and messages,
exact source pointer passthrough, CPU colour, Unicode files and restoration after
detached model drift.

The legacy elemental projection remains the fallback after the same syntax parse.
The predicate for editable CDXML captions keeps its previous scope. The actual
native inspector and canvas read-only branch recognize the new CDX proof. Native
window interaction remains wired pending a platform run.

## Offscreen CLI and stdio MCP

The actual CLI opened the original 146-byte colour case, saved to the existing
Unicode path, saved a Unicode copy and reopened it. Both files equal the original
with SHA-256
`6cb0f22e15ec4b0fbc0aab277d811637480e12db6c6cc51bc35018b507c81f25`.
The independent struct reader checked the complete facts from both actual files.

At 1024 × 1024, CPU and Metal produced identical decoded RGBA bytes. Their shared
RGBA SHA-256 is
`2380c7e952beca12dfbe73aa674014169bc8be1c6525a2db2b993c6463c9c63f`.
The measured solid-colour ink rectangle was `[184, 204, 268, 220]`. Independent
Pillow FreeType glyph measurement and the published baseline gave
`[182.46875, 203.90625, 267.46875, 220.90625]`, within the declared four-pixel
placement tolerance. These captures serve as current evidence. The human-approved
single-bond golden remains a separate case.

The first batched stdio probe returned an Open error and did not finish the
workflow. The qualified run initialized MCP and waited for each response before
its next call. It opened, captured the same pixels, refused a Text edit, accepted
a detached resize, refused the changed-source capture, and restored exact pixels
and model after Undo. This qualification changed only the verification driver.

## Authenticated loopback and local gates

The actual CLI and authenticated loopback test compare the same caption PNG.
Capture, a Text-edit refusal and a lossy-format Save refusal leave the inspected
state and destination unchanged. Detached resize retires preview and CDX Save.
Undo restores the exact frame and original model. Unicode Save As writes the
original bytes. The actual CLI inventory remains byte-identical at 49 records.

`cargo xtask ci` passed 2,210 workspace tests, zero failures and one ignored
across 325 reports, plus three macOS AccessKit tests. Formatting, warnings-denied
clippy, layers, licenses, assets and corpus passed. This run compiled before
the new socket test module was added. With that module included, the full
`sdf_capture_adapters` target passed 115 tests and CLI all-targets clippy passed.
The production sources are identical across the two runs.

The 278,402-byte aggregate log `ci.log` has SHA-256
`7b245365f5380310973223f310d91dc3606ce032da1bcec916446a72d790f370`.
The 14,599-byte adapter log `wire-suite.log` has SHA-256
`8a14040f2a6b57297092d6f7f041a92e1d9f7dcd79a2fc310ba301f2ef2886e7`.
Both are in the local evidence directory below.

## Exact source owners and fractional-scale views

Two additional command tests passed with warnings-denied all-targets clippy.
Opening the reversed-field source with the same model retires the old capture
and file contexts. Completing the prior Save leaves the new session unchanged
and keeps the old source bytes in the completed file. Reopening the first input
creates a fresh owner again.

At 1.5 pixels per logical unit, the 12.05 pt caption passes the existing Fit and
Actual views at 768 × 768 output. CPU and Metal captures match at each view.
Independent Pillow FreeType places the ink within four pixels of the expected
baseline for both cameras. The view and capture calls preserve the exact model,
source bytes and history. This is an offscreen density check.

## Sanitizer

The [fuzz recipe](../fuzz/README.md#direct-page-cdx-caption-projection) reused the 75
independent original inputs. ASan completed 10,000 runs in 2 seconds. Final
coverage was 2521, feature count 5397 and RSS 95 MiB. The maximum initial seed was
162 bytes, with 158 bytes the largest recorded mutation and final limit. The
configured cap was 16,384 bytes. Every seed path and original passed length and
SHA-256 checks after the run.

The ignored evidence directory is
`target/cdx-resources/next-text/preview-freeze.Ue3WwW`. The ASan log is 24,917
bytes with SHA-256
`4e1a284d40cddec312b713119d727d4a081bfd58ebc93625263b3ecf57dd3b6c`.
The successful wire summary in `wire-sequential-results/summary.json` has SHA-256
`894c19eef1fe90067f023c8db18f1ef894a2d4e698b8e973abd866a61cb9823e`.

Native window Open, read-only pointer behavior, Fit/Actual/measured-display density
capture and Unicode Save/reopen remain to be checked on an owned macOS review.
Windows, Linux and browser execution remain open. The supported source
profile covers one explicit caption. General CDX typography and editing
remain separate M1 work.
