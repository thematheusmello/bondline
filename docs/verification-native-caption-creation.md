# Creating a single-line caption

Base `ad2cde9`, 2026-11-19. The [public workflow and file contract](research/native-caption-creation.md)
records the version 25 creation workflow and the CDXML baseline, resources and identity rules.
The [original Expat, FontTools and RDKit witnesses](../crates/io/tests/fixtures/caption-create/SOURCES.md)
fix the expected coordinates, paint bounds, text, IDs and unchanged chemical graph.

## Command, file and paint evidence

- The first command call failed at unknown `text.create`. The first executable IO tests
  passed two controls and failed both sparse native ID expectations. A separate compiler
  probe found the creation capability missing. Six focused IO cases and all 581 IO
  tests then passed, with warnings-denied IO Clippy and rustfmt.
- Five command cases verify exact independent authored models and bytes, refusal of
  noncanonical and CDX creation, bounds and stale requests, single-edit history, and
  persistence of a sparse ID after Undo and a new edit. One render integration case
  checks the 1088 px frame at 2 px/pt against FontTools within 2 px, unchanged backdrop,
  byte-identical CPU/GPU PNGs and exact repaint after reopening.
- Three App-frame cases add press-origin positioning at 1× and 2×, Cancel, drag and
  modified-click refusal, chemical hit exclusion, Unicode Save/reopen, and existing-run
  editing on the reopened caption. Mixed native CDXML retains the previous styled runs,
  resources and graph. Qualified imported CDX and foreign CDXML still permit existing-run
  editing. Noncanonical mixed XML retains its original bytes and refuses creation.
- The first App run failed to find the New caption form in two cases. The third
  reached an unsupported Actual Size request on an opaque input. That test now checks
  the view refusal before exercising the previous paper location and raw Open New
  request. All eleven new and retained caption App-frame cases passed together.
- The actual CLI and initialized stdio MCP run the shared operation and reopen the
  independently expected Unicode-path files. The stdio probe waits for each file job's
  reply before sending the next dependent command. Its first batched attempt exposed
  the expected stale revision and busy responses. All 51 CLI and MCP schemas match.
  The first 49 serialized CLI records are byte-exact and the four existing `ui.caption`
  action schemas retain their order and values. `open_new` is its fifth action, and
  `text.create` is record 51. Registry SHA-256 is
  `124b9ae144dbf53e3362e7b830af8beb8fe3e8b827f792c2df1f912179cc7ac3`.

## Actual macOS application

The isolated, locally signed Bondline build opened an original 512 × 512 pt
blank CDXML with the file picker. The `t` shortcut selected Text. A blank-paper
click opened New caption and Apply painted `Phase β & q`. Undo and Redo restored
the full models and dirty states. Undo and Cancel left the redo entry available.
The next caption had live Text ID 2. Save As wrote `caption-created-é.cdxml`.
Open reloaded the same editable model with empty history. Clicking its painted P
opened Edit caption with Text ID 2 and the exact body. Cancel and Quit closed
only the disposable review.

Independent Expat and Decimal parsed the actual 419-byte file with one direct Page
Text, raw ID 3, left justification, black 12 pt Roboto Regular, and stored baseline
`143.90062111801242 217.04347826086956` pt. The native click was planned near the
independent `(144.25, 216.75)` pt witness and both coordinates fall within 1 pt.
The original input matched the pinned bytes. Saved SHA-256 is
`2cf9c0d61e6a44ee0f231e57e2dab99250c6941fed53748cec0338a1befd08d1`.
The authenticated loopback records establish exact model history, the reopened
run target and identical 1088 px CPU and Apple M3 Max Metal PNGs. Capture SHA-256 is
`857a0424c0a8550eb25091545732521e29ae7aaae79a4a99aa349a56f8ef3bd5`. The screenshot
and click sequence records one current macOS configuration. The App-frame 1×/2×
checks provide separate density evidence.

## Reproduce the focused checks

Set the [pinned Python interpreter](development.md) before workspace checks.

```sh
cargo test --locked -p bondline-io --test native_caption_identity
cargo test --locked -p bondline-commands --test caption_creation --test caption_creation_ink --test ui_caption
cargo test --locked -p bondline-render --test caption_hits
cargo test --locked -p bondline-ui --lib native_caption_tests::
cargo test --locked -p bondline-cli --test caption_creation
cargo xtask ci
```

Local receipts are under `target/caption-create` and `target/caption-creation`.
The first two aggregate runs stopped on three stale 50-entry inventory assertions.
The updates retain the historical discovery prefix, check the new record and keep
the uniqueness assertions. The third run stopped on two old attached-host control
counts in the ring App tests. Those tests now require all 40 controls and the available
Text binding alongside their existing ring assertions. Both focused reruns pass.
Production code was identical on those runs.

## Final local gate

The next aggregate stopped at the unchanged MCP listener replacement-bind
test on port 53514. The later `cargo xtask ci` on `a98dc47` passed **2,297**
workspace tests across 343 reports and the three AccessKit tests. One existing
workspace case stays ignored. Formatting, warnings-denied Clippy, layers,
licenses, assets and the pinned corpus passed. The listener failure's cause
remains unknown. GitHub Actions stayed off.

`ci-retry.log` has 288,592 bytes and SHA-256
`67b467f0d4e93b264bdcfd399b479e0798655ae762da83fd911a08c85446c11f`.
Command crate type-checks passed on Windows MSVC, Linux GNU and WebAssembly.

## CDXML source sanitizer

The previous empty-run splice check now uses its original dense wire ID. The
expanded invariant checks exact native sparse identity, opaque byte retention after
a lexical change, and rejection when an edit would change the projected ID.
AddressSanitizer ran `cdxml_profile` over 48 unchanged seeds, including all six
creation fixtures. The first campaign reached 5,712 runs in its 300-second limit.
The qualified campaign reused its generated corpus, raised the time limit and
completed **10,000 runs in 406 seconds**. The input limit was 16,384 bytes and
the largest recorded new or reduced mutation was 14,602 bytes. The qualified log
has SHA-256 `0eaac4ce4249dc15e41bffc4847d698abc29a3710098bab877046b356c1cfb89`.

Caption creation currently uses an explicit single-line style and baseline on eligible
source-free and exact native CDXML pages. CDX object creation, multiline, formatting,
style inheritance, exact 26.1 interaction, physical DPI and runtime acceptance on
Windows, Linux and WebAssembly remain open. The regression suite uses its existing
human-approved references.
