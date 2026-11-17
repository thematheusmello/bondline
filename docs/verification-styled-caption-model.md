# Styled caption model receiving

Model-only increment on `b34e786`, in the isolated `bondline-caption-model` worktree,
2026-11-17. This implements the data interfaces in the shared
[caption contract](research/styled-captions.md). It grants no source, font, rendering,
chemical-label, file, command or native-UI authority. The complete M0–M5 goal remains open.

`TextId` is a positive application identity independent of atom/font IDs. `TextStyle`
retains a literal family, point size, face bits and unquantized RGB. `TextRun` retains
literal XML-legal Unicode, including empty/multiline text. `StyledText` retains a validated
baseline and ordered runs. The constructors and closed, map-only JSON decoders share
validation; text copies preserve IDs, positions, styles and untouched runs. Font availability,
face meaning, layout and glyph support are deliberately separate renderer obligations.

The original data policy admits 1..256 UTF-8 bytes per font name, finite point sizes above
zero through 100000, finite RGB channels within 0..1, 1..4096 runs and at most 1 MiB of
UTF-8 text per object. Across a Document, text objects and runs each cap at 100000 and
caption UTF-8 text caps at 64 MiB. XML characters follow
[XML 1.0 production 2](https://www.w3.org/TR/2008/REC-xml-20081126/#charsets);
the model preserves legal text without normalization or chemical interpretation.

Page text arrays are omitted when empty. The test checks the complete prior default
Document JSON byte string. Append/replace validate document-wide text IDs and budgets
before mutation. Missing pages/targets, duplicate IDs and resource errors leave all data
unchanged. Equal replacement retains the current Arc; other replacements retain unrelated
Arcs. `Document::retained_bytes` includes text handles, unique text payloads, reserved run
vectors, text Strings and family Strings, deduplicated across snapshots. This excludes
allocator/Arc headers and is not an RSS guarantee.

## Independent declarations and results

The first literal two-run Unicode/colour/style declaration was 1545 bytes, SHA256
`82e0db90d8817e4a5dda2ce4a95dc8314d2e4c7d5726a8b9b5e6c4b2d826782b`.
Before production, its test run exited101 with the intended E0432 new-type imports and
E0599 `append_texts`/`Page::texts`. The original file and log remain in
`target/caption-model/first/`. This is an API compile RED, not an application runtime RED.

The separate pre-production limit declaration was 11396 bytes, SHA256
`ace9d4a25f3ad9cb40a7f4a61df5b8c535bacfae1697294d2e9f2a34174d1a0b`.
Its isolated compile RED contains missing-API errors and an E0282 inference cascade; that
cascade disappeared when the real types landed, without a test-source correction.

The first implemented receiving run compiled both targets and passed 7/8 cases. The last
accounting assertion mistakenly expected deduplication after replacing the sole shared
text: no shared payload remained. Its original source/log and reason remain in
`target/caption-model/accounting-setup-before/`. The qualified setup adds one unrelated
caption before the snapshot and an explicit Arc-sharing assertion, retaining the strict
dedup inequality. No production accounting change was made for this setup failure.
The implementation also uses saturating construction sums to avoid a 32-bit overflow
when rejecting the maximum run-count cross-product. A later focused case verifies IDs
across two pages, and another checks the complete old JSON and over-limit atomic append;
these are passing-first post-draft controls, not additional claimed REDs.

Commands ran only in this isolated worktree, with its absolute `target/cargo`,
`CARGO_INCREMENTAL=0`, `CARGO_BUILD_JOBS=1` and DEV/TEST debug0:

- `cargo test -p bondline-model --locked`: **51 passed/0 failed/0 ignored**, including
  10 new cases and the existing model tests; no external consumer or GUI test was run.
- `cargo clippy -p bondline-model --all-targets --locked -- -D warnings`: exit0.
- `cargo fmt -p bondline-model -- --check` and `git diff --check`: exit0.

Logs, original declarations, the exact shared contract readpoint, source/test hashes and
terminal receipts remain under `target/caption-model/`. Old tests, parsers, renderer,
fixtures, fonts/assets, dependencies/locks, workflows and MAIN were not changed.
Root must integrate source/loss guards before admitting caption-bearing models to existing
writers, then verify the full foreign file/render/edit/history/Save/reopen workflow.
No native/platform, font-fidelity, aggregate workspace or milestone completion is claimed.
Actions remains OFF; no app was opened and no push/publication was performed.
