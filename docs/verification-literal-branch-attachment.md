# Literal branched-H attachment: independent renderer acceptance

Two original public-geometry tests in `crates/render/tests/literal_branch_attachment.rs`
were frozen on `ad68dc9` before receiving a renderer change. The6,128-byte declaration
has SHA256 `13ef3b6cd7e34418f2205003ee2d387a7cf8b5bd41ef06db4b9fe6148d088852`.
They use the ordinary literal drawing entry, actual CPU/GPU rendering and shared picking;
no resolved-only preview can substitute for the actual drawing scene.

The first case positions C/H1 at(128,128)pt with three88pt Single-bond rays at
0°/120°/240° to literal C/H3 terminals. A256pt square paper at576px maps points by
`p -> 32+2p`, with24px label em and2px strokes. At(308,288),20px along the right bond,
picking must return Bond1; its full analytic horizontal ink corridor must remain black,
the adjacent off-axis corridor white, and H ink must remain visible elsewhere. The test
also retains all authored IDs, directions, positions and literal H counts.

These numerical choices and corridor policies are original test geometry, not a reference
product default. The attachment requirement follows the existing public
[IUPAC2008 GR2.1.5–2.1.7 basis](https://iupac.qmul.ac.uk/drawing/drawing.html) recorded
in [the source register](clean-room/sources.md): bonds must identify the element rather
than an attached H label. No figure, asset, font or upstream code was transplanted.

An independent pinned FreeType qualification uses the unchanged bundled Roboto and the
existing4x/BASIC/BOX policy. At this exact scale, C ink occupies x281..293, H in the old
right-hand position x298..310; (308,288) is outside C and is black H ink. The H glyph has
87 pixels. The test's minimum20-pixel retained-H condition is well below that independent
glyph size and above the at-most-two pixels of unrelated ray ink in those candidate boxes.
Font SHA256 remains
`56a45233d29f11b4dfb86d248e921939d115778f87325e7ae8cc108383d6664d`.

The second case first proves the existing opposed N/H1 scene's stacked-H location at
(288,264), including actual CPU/GPU equality. An unrelated O/H0 placed at that location
must make the whole scene refuse `LabelOverlap`. This control already passes on the
baseline: `Label.mask` includes the union of the stacked glyph bounds. No ignored-H
collision defect or collision-loop fix is claimed.

## Actual RED and unchanged GREEN

- Baseline59064 exits101:1passed/1failed. The first test reproduces Atom1 instead of
  Bond1 at(308,288); the overlap control passes. Its3,813-byte log SHA256 is
  `9a51610742c674715b679f8f82c608167c15309ff12b8561cb5afb7e871cda66`.
  The star stops at picking before its final GPU assertion; the control's GPU comparison
  executes. Neither that failed star nor the source probe proves universal angle behavior.
- Root's33,741-byte `page.rs`, SHA256
  `654669a5fd8952e6a6662f40ff3f08676855394029a3fd9dc4f1ad8f30184863`, was received only
  after verifying the original33,681-byte source against the protected ledger. An exact
  one-predicate replacement enables the existing attachment-aware label producer for
  authored-H atoms with at least three neighbors as well as multiple-bond scenes. No
  collision/count/coordinate/model change accompanies it. The original source is archived.
- Independent38605 exits0: **2passed/0failed/0ignored**, with the frozen tests unchanged.
  The star now passes its bond hit, all black/white corridor checks, retained-H and actual
  CPU/GPU byte equality. The overlap control still refuses.
- All-target renderer clippy64772 with `-D warnings` and package rustfmt check exit0.
  These are scoped independent checks; Root's full renderer/workspace runs are separate.

Commands use Rust1.94.1, the checkout's absolute target, incremental0, dev/test debug0,
and the literal pinned `BONDLINE_PYTHON` path. All49 protected original inputs were
checked before receipt; the sole authorized `page.rs` replacement is separate from the
48 originals that remain exact. Existing tests, unknown-H fixtures, font, approved golden,
consumer scripts and both locks are unchanged. No old assertions or test bridge were edited.

Ignored `target/branch-label-attachment/` retains the declarations, baseline failure,
font qualification, original source, exact delta, receiving logs and final hash receipt.
All own jobs are terminal. This is bounded offscreen renderer/picking evidence, not native
UI, all-angle/all-viewport chemistry, a new approved golden or Windows/Linux execution.
No Git, Actions, dependency, asset or native-input change was made by this worker.
