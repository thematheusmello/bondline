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
The predicate for editable CDXML captions keeps its previous scope. The native
inspector and canvas read-only branch recognize the CDX proof.

## Offscreen CLI and stdio MCP

The actual CLI opened the original 146-byte colour case, saved to the existing
Unicode path, saved a Unicode copy and reopened it. Both files equal the original
with SHA-256
`6cb0f22e15ec4b0fbc0aab277d811637480e12db6c6cc51bc35018b507c81f25`.
The independent struct reader checked the complete facts from both actual files.
An additional known-path Save advanced the owned input's controlled 2000-01-01
mtime and retained all 146 bytes, proving publication. Its receipt is in
`known-save-receipt/publication.json` below.

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

The exact-tree gate on `53642e9` later stopped at the existing MCP
`shutdown_without_clients_releases_the_listener_address` test. The observed
`replacement_bind` failure was `AddrInUse` on 127.0.0.1:64951. A post-failure
check found the port clear. The cause remains unresolved. The listener and
its test stayed byte-identical. The 181,214-byte `ci-final.log` has SHA-256
`956497907e35084d8ff13d9409f25cda3f9bac861eb4a0d2800d71e93beb7a6f`.
That attempt reported 1,463 passes and one failure before stopping. The remaining
97 workspace reports passed 749 tests with one ignored. The AccessKit tests,
clippy, layer, license, asset and corpus checks also passed separately. These
are two runs with one unresolved failure. The 98,404-byte `remaining.log`
has SHA-256 `21f90986dbe5bfe2474dc2bc43bab41d26f743fb159f624639c6c2502920c33b`.

The subsequent complete local gate on `cbaa4b058ca1a16606e94a06580a283dbaaff439`
passed **2,213/0/1** workspace tests across 326 reports and **3/0** AccessKit
tests. It also passed fmt, warnings-denied clippy, layers, licenses, assets and
the pinned corpus. The 277,379-byte `ci.log` in
`target/cdx-resources/next-text/caption-final.dw7Q4x` has SHA-256
`9dd44b53a63a23ddb623fe0a85defcbb5821715bf2bc2a8f7df50fda1025a915`.
The previous listener failure remains unexplained.

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

## Actual macOS Open, view and Save

The separate signed review bundle ran on Apple M3 Max with Metal. Its
executable SHA-256 was
`f9ae7e39d9f05f8aa4d5f9d6b401e4866cc1ac8ab7aa8ce0b65c0258c4bcb7da`.
It was built from `53642e9`, whose runtime sources are identical to the
final-gate revision above. Only this owned review instance was opened and quit.

- Select Cyclooctane, then open the 146-byte RGB fixture in the actual File
  panel. The native canvas displays the blue caption on white paper. The
  inspector reports one caption and read-only canvas. Blank click, drag and
  Option-click on the caption leave the complete adapter snapshots byte-identical.
- F5 goes to Actual 100%, F6 returns to Fit (125.78% at this window size), and
  entering 112.5 in the native zoom field gives 112.5%. View epochs advance
  from 0 to 3 while the revision, file, source and history stay exact.
- Native known-path Save advances the controlled 2000-01-01 mtime to
  `1795033675172974439` ns and retains all 146 bytes. Save As writes
  `caption-再開-λ.cdx` with the same bytes. The chooser and title show
  placeholder glyphs for the CJK characters while accessibility and the actual
  filename preserve them.
- Open the independent 12.05 pt fixture and observe the black `AV2` caption.
  Opening the nonwhite-palette fixture shows the old unsupported Text error.
  Cancel leaves the prior caption, revision, path and view intact. Reopen the
  Unicode copy and recover the original model at revision 3 with empty history.

The independent binary reader checked all four owned input files and the
actual Save As output against the frozen record facts. The two RGB files share
SHA-256 `6cb0f22e15ec4b0fbc0aab277d811637480e12db6c6cc51bc35018b507c81f25`.
On the attached review process, 768 × 768 captures of a declared 512 × 512
logical viewport matched CPU against Metal at both Fit and 112.5%. The
solid-colour ink bounds were `[155, 168, 213, 179]` and
`[89, 107, 164, 121]`, within four pixels of the independent FreeType bounds.
The reopened Metal frame matched the pre-Save 112.5% frame exactly. These
offscreen captures specify 1.5 px/logical unit. The native screenshots prove
visible layout and interaction at the observed window scale. Physical display
density variation remains unverified.

The owned app exited after Quit. The snapshots, independent check and PNGs
are in `target/cdx-resources/next-text/native-review.hdS0xZ/native-evidence`.
`verified.json` has SHA-256
`f33a63fc35af7f12df1d242ddb1cdbcbf628508c4a18041398db7eb57065ea0b`.
The 1,196-byte `prepared.json` receipt in the parent directory has SHA-256
`dbf0de7f0da5847c7239c13a6b53ad944f151cde7950e960d458010e6ebad167`.

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

Windows, Linux and browser execution remain open. The supported source
profile covers one explicit caption. General CDX typography and editing
remain separate M1 work.
