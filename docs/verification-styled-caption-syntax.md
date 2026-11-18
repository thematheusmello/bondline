# Standalone styled-Text syntax receiving

2026-11-17; isolated branch `feat/styled-caption-cdxml`, base `b34e786`.
This implements only the standalone parser portion of Root's
`docs/research/styled-captions.md` integration contract. Read the
[implementation](../crates/cdxml/src/text.rs) and
[independent tests](../crates/cdxml/tests/styled_text.rs).

## Scope and semantics

`Document::styled_text(Node)` exposes `TextObject` and its ordered borrowed
`StyleRun` views. Each keeps the exact original element. Runs expose decoded text
and optional font, size_pt, face and color; missing selectors remain None. Unknown
attributes stay in the nodes, not silently removed or treated as understood.

The receiver must own the exact ordinary t node, including when another parsed
Document has identical input bytes. At least one direct ordinary s is required.
Only XML whitespace may separate runs; other text, elements, comments and processing
instructions refuse. A run accepts only character-data children, including decoded
references/CDATA; mixed content refuses before a partial view can escape. Empty runs
remain valid. Empty namespace resets and qualified lookalike attributes retain their
distinct meanings. Attribute lookup uses expanded namespace meaning.

The public [Text](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Text.htm) and
[Style](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Style.htm) definitions inform the
original view: font/face/color use exact UINT16 decoding through the existing integer
helper; XML size is a finite positive point value. The view does not impose the separate
model/render point cap or infer binary CDX scale/offsets. No inherited defaults, font
availability, face rendering, palette resolution, chemical meaning, IO edit permission
or caption-profile completeness is granted. Existing syntax admission/UTF8/DTD/entity
policy and global64MiB/node/depth/attribute limits are unchanged. Application text/run/
glyph budgets belong to subsequent model/render integration.

Existing [SetText syntax edits](verification-source-preserving-xml-text.md) operate on
selected s leaves. The view adds no mutation operation or stale-source bypass.

## Test-first evidence

All evidence paths below are relative to this worktree's ignored
`target/styled-caption-syntax/`.

1. The first original public-API declaration was frozen at1,125B, SHA256
   `7be2beb84936cd6b1692a8926b37ffa280af42933cdca14ea5a55134e5a65ce2`.
   Session82801 exits101 at the sole intended E0599 missing Document::styled_text.
   This is a missing-API compiler RED, not a runtime parser defect.
2. All nine cases and the independent checker were frozen before implementation.
   The initial declaration was an exact prefix then. Expanded receiving exits101 on
   missing TextObject/TextError exports and styled_text methods, with two downstream
   inference errors caused by those absent types. No expectation/setup change was needed
   after implementation. Frozen source:12,150B/SHA256
   `5b23a15d32b43b0c4ec92cd65bee355e9c3a30411888da22cade2bb330bb7fea`.
3. A test-only Expat/Decimal process was independently qualified on two manually declared
   positives and seven wrong text/font/size/face/color/order/type controls before any
   parser payload use. Its driver exits0; every control has the exact expected exit.
   The durable test repeats these controls. Another case qualifies original and edited
   literal XML before passing actual view values to the checker. It compares exact edited
   bytes, then independent decoded run values; this is not a sole own roundtrip.
4. First implementation session6850 passes9/0/0 unchanged. The formatted full standalone
   package session43254 passes **67/0/0 across14 result lines**, including all58 prior tests.
   Formatting changes no expectations; the original unformatted declaration remains
   archived, rather than claiming a final byte-exact first prefix after rustfmt.

The cases cover Unicode/entities/CDATA, empty/missing properties without defaults,
resource independence/missing logical font, all three UINT16 domains, positive finite
size extremes, malformed scalar errors, qualified attributes, empty namespaces, exact
node ownership, strict mixed content/later errors, source no-ops and existing SetText
composition. A4097-run input and a1MiB+1 text leaf establish that the standalone view does
not accidentally enforce later model budgets; old parser-bound tests remain in the full
package. No font glyph, appearance or chemical-identity oracle is implied.

## Commands and results

Cargo used the absolute worktree target, CARGO_INCREMENTAL=0, CARGO_BUILD_JOBS=1,
CARGO_PROFILE_DEV_DEBUG=0 and CARGO_PROFILE_TEST_DEBUG=0. Test consumers used the literal
BONDLINE_PYTHON path
`/Users/sarenaberg/Workspace/doozle/open-source-world/bondline/.cache/oracles/format-python/bin/python`.
No interpreter was installed, resolved into another path or made into a shipped dependency.
The durable test helper retains the existing python/python3 fallback when no override is set.

| Check | Result |
|---|---|
| cargo test -p bondline-cdxml --test styled_text --locked -- --nocapture before API | sole E0599, exit101 |
| same target with expanded declarations before API | missing exports/methods plus downstream inference, exit101 |
| original Expat/Decimal checker qualification | 2 positives/7 negatives qualified, exit0 |
| same new target after implementation | 9 passed/0 failed/0 ignored |
| cargo test -p bondline-cdxml --locked after format | 67 passed/0 failed/0 ignored, exit0 |
| cargo clippy -p bondline-cdxml --all-targets --locked -- -D warnings | exit0 |
| cargo fmt -p bondline-cdxml -- --check | exit0 |

This is scoped library evidence. No full-workspace xtask gate, fuzz run, native app,
Windows/Linux interaction, reference program, font/asset/golden change or Actions run
was performed. Root owns wider receiving and application integration. All worker
execution handles are terminal.

## Preservation and pins

Only the new module, two lib module/reexport lines, new tests/checker and this note are
intentional tracked changes. Removing those two lines recovers exact old lib.rs7,330B,
SHA256 `0e115c0d33545465c64a3b633849daff5b6c3809c0e528c340232cf7bd4398a9`.
Old resource/numeric/parser/edit/writer logic and tests, model, dependencies/locks,
workflows/fonts/assets/corpus remain unmodified. No model or GUI dependency enters cdxml.
Baseline/protected/source receipts retain exact inputs under the ignored evidence directory.

| Source | Bytes | SHA256 |
|---|---:|---|
| `crates/cdxml/src/text.rs` | 4782 | `ca00b07ff466107e56a69d866d336a41a9e678eccc722a5f659a8a94a50b25a0` |
| `crates/cdxml/src/lib.rs` | 7389 | `2b00d09a80a4426052a7224b4da51bb2e7a2d7c9abf6dbcd396176bbc9ad42df` |
| `crates/cdxml/tests/styled_text.rs` | 13813 | `e7174a7c110448a60138574c5908724a9c52c24bcb481026fce95b2c6af94369` |
| `crates/cdxml/tests/styled_text_oracle.py` | 2231 | `88be752b674a828a17d5c9c3788006a987775b565442e02acc89c2131c49c51f` |

| Evidence file | Bytes | SHA256 |
|---|---:|---|
| `first-api-red.log` | 839 | `dd73e221442c61e3031191d526d9f53633bc769ef08d98b4898b7a5be380910b` |
| `expanded-api-red.log` | 3864 | `1287f6f2cede5dd5992fa89f45ea4abcb83a72f7f3d5bec2f6268d0b0883c12c` |
| `oracle-qualification.json` | 4360 | `7dc4882b675f0482aed683cc5cfe337576e17dec02752e6e3e6a5edd44e421a8` |
| `oracle-qualification.log` | 59 | `5afedd006b1082ffe6a637fdf5bff0e169afec29e3ba3caa4138ec5c38cd406d` |
| `first-runtime.log` | 1113 | `3c994ae81f4c843d8450fedc2982270bdf6d10ad1b2cf340012b3fbf74bd11ee` |
| `full-cdxml.log` | 8464 | `9ec45b1752f861a3f5f14b25af2cb458c0c041eaa692e53dd4951321a35bb7e7` |
| `clippy.log` | 306 | `a2b4c614f4f54fb46bd0e2816b2fcba06bda0cd6a7e3a80898082aefb8bd47e9` |
| `fmt-check.log` | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
