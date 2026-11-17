# Styled caption workflow receiving

2026-11-17, baseb34e786 plus eff16ae. This integrates one bounded M1 workflow, not
completion of Text-tool parity or M0–M5. Read the [contract](research/styled-captions.md),
[model](verification-styled-caption-model.md), [syntax](verification-styled-caption-syntax.md)
and [renderer](verification-styled-caption-render.md) receiving records. Actions stays OFF.

## Delivered boundaries

Checked Page text objects retain literal Unicode, ordered styles, IDs and baseline
positions. Native CDXML encoding writes their font/palette resources rather than dropping
them. A separate foreign caption-only capability proves every source object/property,
one explicit Poster page, white default background, Left baseline and explicit regular
Roboto runs. It keeps global chemical completeness false. Unknown/mixed/default/font/
style data remains retained, unsupported and unchanged-only.

`text.set_run` appends command47. It retains all previous46 metadata/schema/record-order
bytes, uses closed required5 parameters and preserves styles/positions/unrelated source.
Final source-write and whole-page render certificates precede one normal history commit.
Source/revision/close/target errors are atomic; equal text preserves Arcs, redo, source,
close and Save contexts. Captured Save remains truthful about its captured revision.
Foreign caption page/style/chemical edits stay disabled/refused. Source-free/canonical
captions have a complete native encoder; whole-MOL/CDX/retained-SMILES guards prevent loss.

Shared display/view/capture and the actual native canvas use current caption data.
Physical and logical checks include every run's min/max em, even blank runs. There is no
Text hit target, insertion/deletion/reordering, inline rich editor, font control or new
catalogue binding. Catalogue16/124 stays; the full Text tool remains grey. Native canvas
pointer input is read-only, while File/history/view commands keep their ordered route.

## Independent REDs and qualifications

Separate model/parser missing-API REDs and renderer runtime missing-ink RED precede their
isolated commits3f001095/33b8ea07/0f15dad6, retained on their original worktrees/branches.
Root receives all three as one coherent workflow. Original source/test pins, independent
FontTools4.61.1/Pillow12.3.0 GPOS/outline expectations, minimum colour IoU0.9653998865570051,
three CPU/Metal equality cases and the37 unchanged renderer inputs are recorded above.
No golden is regenerated or new human approval inferred.

Root logs live under `target/foreign-text/receiving/`:

- `io-red.log`: actual0/2, foreign caption ignored and native encoder dropping owned
  text. After implementation and a closure-signature compile correction, the original
  two cases pass; final focused IO including the regression below passes3/0.
- `writer-api-red.log`: intended missing `Writer::text` API. Its three decoded-text,
  illegal-character/size-atomicity and adjacent-node-budget cases pass after implementation.
  Escaping is independently consumed through actual native XML by Expat/Decimal.
- `command-qualified-red.log`: actual unknown `text.set_run`, then first1/0. The initial
  fixture's root bgcolor assumption was corrected against the primary future-only SDK
  page before profile production; the same missing-command runtime result remains.
- The expanded backend declaration originally proposed refusing raw same-style text
  deltas. Existing preserved-model policy and the locked caption contract support those
  deltas. Source review qualifies it into a stronger exact-one-leaf/Expat/source-purity
  positive before execution; geometry/style/ID/membership/root/page negatives remain.
- Expanded backend execution first passes7/8:4097 non-space glyphs hit paper bounds
  before the glyph count. Keep that strict paper refusal and add4097 spaces to reach
  the independent work budget. Qualified backend passes8/0; this is a vector/error-order
  qualification, not a weakened budget or a product fix.
- Source review independently declares `canonical_caption_transition`. A first filter
  selects zero tests, proving nothing; the exact named test then fails0/1 with
  UnsupportedEdit. A self-closing empty run can legitimately become exact native XML
  after SetText. The final writer now accepts either independently proven Caption or
  Canonical reopening, still requiring exact complete-model equality. IO3/0 passes.
- `app-red.log`: actual0/3, absent canvas caption ink and missing logical minimum-em
  rejection. The display capability and both-endpoint DPI checks fix these; App3/0 then
  passes. A subsequent independent keyboard Undo/Redo/Save declaration adds coverage.
  Its unexecuted source-modified=false expectation is corrected against existing Save
  ownership and actual native state: saved dirty=false does not rebase original source.
- The post-draft24-case generated command property checks run choice, literal Unicode/
  escaping, complete snapshots, exact source bytes and full Undo/Redo sequences. It is
  passing-first additional coverage, not another claimed production RED.

Three actual CLI/initialized stdio/authenticated-loopback tests pass, with manual Unicode
files, exact original/current/saved bytes, independent Expat/Decimal and Pillow PNG checks.
No chemical toolkit is used to invent chemistry for nonchemical captions. Actual CLI
metadata is44407B/SHA256
`8d5a7031cfc0ce8e444878ed058ee6080dcff601f48cdfd8149648eee0b575df`;
the old43493-byte46-record array remains its exact serialized prefix before the new
array delimiter. Original frozen fixture SHA256
`132f9311082cc0dbabc0cdb0f3ecba56200ba6b4ea2a6d28f2312c50c44bb993`.
Current inventory expectations change explicitly; historical OLD arrays stay unchanged.

## Actual Mac acceptance

Native access returns for this review. An isolated own bundle at
`target/foreign-text/native/Bondline Caption Review.app`, identifier
`org.bondline.styled-caption-review`, runs the real rebuilt Rust binary SHA256
`517f8ed80686840d35381e7cee4f068ffa0974c7e93d08fe002bb9e064ca9cfe`.
Its original wrapper only supplies opt-in control-port0 and private startup redirection.
macOS27.2/26B123, Apple Silicon; no measured DPI or performance result is claimed.

CUA observes the actual own window: File Open `caption-λ.cdxml` paints blue Phase A,
one caption, Saved/CDXML and Caption preview at251.56% Fit. Authenticated shared command
edits Phase β & q; the native window repaints and becomes dirty. Actual Command-Z restores
Phase A and Saved; Shift-Command-Z restores the edited ink. Native Save As writes
`caption-saved-λ.cdxml`; a paper drag leaves the entire inspect result identical. Native
Open of that saved file preserves edited ink, resets history, and rebinds the391-byte
unmodified source. These are actual OS interactions plus a remote authoring command,
not an implemented Text editor or human hardware-input certification.

Independent stdlib Expat/Decimal reads the actual saved file and checks font42/Roboto,
point size12, face0, colour4/RGB0.2/0.4/0.8, source IDs90/21, page256, p72/96 and Left.
Only `Phase A`→`Phase β &amp; q` changes; every other original byte stays exact.
Original382B/SHA256
`ebbeb0a4bf04bac79b1161782b7b329c253b915033af950909a57d412e6e02f0`;
saved391B/SHA256
`638016d98fe4c03afbabd8e46e0300362dc2ff59d5440cff104e9dc8605b6c0c`.
Redacted request/state and build/file receipts remain in the ignored native directory;
startup credentials stay private and uncommitted. Screenshots were observed through CUA,
not substituted illustrations/goldens or new committed media.

Initial background Command-O required Raise before input; the AX diff temporarily lags
the visible file picker, so fresh screenshots guide coordinates. These are driver/setup
qualifications, not attributed product fixes. Command-W closes the review; its follow-up
AX read reports procNotFound, then native inventory confirms only this review app stopped.
No older user app/drawing or proprietary reference program is selected, altered or closed.

## Bounded parser fuzzing

Pinned cargo-fuzz0.13.2/libFuzzer0.4.13, nightly-2026-04-01, AddressSanitizer, seed671308429,
10000 runs per target, max_len16384/timeout5/RSS2048. The original stdlib recipe creates
42 seeds, separate source/corpus and manifest, and refuses an existing destination.
Parse/Profile add typed text/Writer selectors and ownership, complete caption semantics,
same-style leaf edits/restoration, eight authority drifts and Caption→Canonical checks.

First parse fails on its qualified-attribute seed: the new harness incorrectly uses
roxmltree's local-name lookup for ordinary `color`, expecting Some4 instead of correct
None. The product and existing qualified-attribute unit tests remain unchanged. The
helper now explicitly selects no-namespace attributes and also asserts qualified data
retention. The first correction's unsupported tuple API fails compilation; the actual
installed attributes API supplies the corrected lookup. Fixed-artifact replay passes.
Fresh qualified parse passes10000/6s, cov4496/ft10720/RSS466MiB; Profile passes10000/87s,
cov5625/ft11622/RSS509MiB. Both actual NEW/REDUCE records reach16384B. All126 immutable
source copies across the initial/fresh/profile directories remain hash-exact.

Qualified parse log64731B/SHA256
`8d0d157ce8cec5fee3f09e65fbcab12cc8ea050c7b65f44675b73f823714f614`;
profile84885B/SHA256
`26b9abf36e5ce571f5b426b26b7f80146057bb1b87faf8701d6c42adf5e0e52a`.
The first failed8867-byte log and artifact remain; SHA256
`5793dc2ad547335eb33ff175d417a5aa0856c2bcef599dd4a1c05402da734d85`.
See [fuzz commands](../fuzz/README.md). This does not prove general64MiB, render fuzz,
native race, full chemistry or cross-platform acceptance.

## Local aggregate and continuation

Initial `cargo xtask ci` passes fmt and warnings-denied workspace clippy, then stops at
the unchanged MCP shutdown replacement_bind127.0.0.1:58833 AddrInUse. Its failure remains
recorded; no listener assertion, bind policy, timing, retry or exhausted diagnostic is
changed. It records1217 passed/1 failed/0 ignored across199 result lines. Log154693B/SHA256
`56902ca0d24c9e403f7229cfcb2507b401495591acaa2e42bb5b712d104f0cd4`.

Complete `cargo test --workspace --locked --no-fail-fast`74577 records1906/1/1 across286
result lines. All new49 cases, including generated history and the fourth App keyboard
case, pass. Its sole failure is an old UI conversion assertion still expecting registry46
instead of47, missed by the scoped inventory migration. The exact current assertion is
updated; its conversion/schema/refusal checks and historical frozen arrays stay intact.
The unchanged MCP shutdown test passes naturally in this workspace run, explaining no cause.
Log238678B/SHA256
`548d5e10cf20c244bb3b190b129ebe19d6886458e3dfa6f44e4f6520debda3f6`.

Qualified11820 passes the entire UI library322/0/1, workspace warnings-denied clippy,
rustfmt, vendor3/0 plus vendor clippy, layers, strict runtime licenses, seven assets/
notices, pinned corpus and isolated fuzz formatting. Log40517B/SHA256
`c36f6ae8b8e8868685c9ed1a03fdaf9e3ddce6bd7ca49325e313955f1090a95b`.
Every component passes at its relevant current source; duplicate UI executions are not
summed into an aggregate. **There is no clean `cargo xtask ci` aggregate** for this
receiving. Do not hide either failed run or claim the natural MCP pass fixes its failure.

Execution uses Root's actual workspace, Rust1.94.1, incremental0/jobs2/dev+test debug0,
the isolated format-python interpreter and seed671308429. Locks differ from b34e786 only
by eff16ae's direct already-pinned ttf-parser edge; no package/version changes. Workflow,
protected listener test/policy, vendor, fonts/assets and old goldens remain exact. All
Root jobs and workers are terminal; isolated model/parser/render worktrees remain
completed provenance. No push, publication or hosted action occurs.

The full M0–M5 goal remains active. Next finish effective chemical Label font/size/face
and child Text ownership with independent foreign/source/retention/render/history cases;
then the full native Text editor, mixed diagrams, CDX/general interchange and the remaining
roadmap. Unknown/default/charset/shaping behavior stays guarded. Native width/spacing,
measured DPI/races, Windows/Linux/browser and distribution remain required. No milestone,
full26.1 parity, new golden approval, dependency-license exception or hosted run is claimed.
