# Fixed-ring H2 attachment: independent renderer acceptance

The4,903-byte test declaration in `crates/render/tests/literal_ring_attachment.rs` is
SHA256 `abcbe490c1be0d7f7d9e40e11d0a73b93e7ee2a020e76dbf86a9b3bbb5049717`.
It was frozen on `d76a0ab` before receiving a renderer change. It constructs a regular
C6/H2 hexagon directly from public model types; it calls no SMILES layout/API/search.
All authored H remain Some(2), with separate unknown-H control data.

The original analytic fixture has radius/side64pt, center(128,128), a preferred vertical
left edge and a256pt square page.576px gives the exact transform `p -> 32+2p`,24px em
and2px strokes. Atom5 is the top carbon(288,160); Atom4 is its lower-left neighbor.
Pixel center(270.5,170.5) lies0.344px from that bond's axis, within its1px half-width.
The unchanged font's independently checked C/H/subscript2 ink is absent at that pixel.
The ordinary renderer must therefore retain black Bond4 ink and pick Bond4 there.

Pinned FreeType/Pillow12.3.0 uses the unchanged4x/BASIC/BOX policy and original OpenType
metric vectors. This is an external glyph-gap qualification, not a fitted renderer image
or new golden. Roboto SHA256 remains
`56a45233d29f11b4dfb86d248e921939d115778f87325e7ae8cc108383d6664d`.
The existing [IUPAC attachment basis](https://iupac.qmul.ac.uk/drawing/drawing.html),
GR2.1.5–2.1.7, motivates preserving the bond's chemical-element attachment. Numerical
test geometry and display policy are original Bondline choices, not reference defaults.

## Observed RED and exact correction

Baseline25524 exits101: **1passed/1failed**. The ordinary literal fixture produces WHITE
and an Atom5 hit at the required bond pixel. The same-count resolved fixture produces
BLACK/Bond4; its CPU/GPU comparison passes. A separate skeletal fixture with authored
H=None also produces BLACK/Bond4 and CPU/GPU agreement, retaining None. That control is
never substituted for the failing authored-H2 document.

The failing3,801-byte log SHA256 is
`3a67702228382a85b29f1b2ca5c23bf45cfefcdd9f5f22f7391b6bad31a5b5b8`.
Literal CPU/GPU equality is not reached in the RED after the decisive pixel failure.
The first declaration, source and font qualification remain archived.

Root's33,741-byte `page.rs`, SHA256
`5e618385af247189e634de5c557f0a01c78c21f5a1351a1a3323069f78a27973`, was received
after verifying the exact prior654669a5 source. The sole one-character predicate change
extends the existing authored-H attachment-safe label path from degree>=3 to degree>=2.
It changes no masks, font, counts, coordinates, inference, collision policy or budget.
The old source and exact delta are preserved separately from unchanged protected inputs.

Independent12857 exits0: **2passed/0failed/0ignored** with the original tests unchanged.
The literal H2 fixture now retains the exact black pixel/Bond4 hit, equals the same-count
attachment-safe Scene and CPU frame, and passes actual GPU byte equality. The separate
unknown-H control stays unchanged. All-target renderer clippy88289 and package rustfmt
check exit0. Root's full renderer/workspace gates are separate evidence.

The source receipt's checks and write completed, but its final status print initially
used the wrong metadata key and raised KeyError. The actual receipt/source/delta/all
49 other protected inputs were independently revalidated; no product or test change
was made for that reporting setup error.

## Scope

This proves the fixed C6 incident-H conflict. The initial mono-substituted5/7-member
`LabelOverlap` observations belong to different input/phase diagnostics; no causal or
universal label-failure claim follows. Preferred symmetry phases may behave differently.
The independent ring IO declaration/oracle remains unchanged by the renderer task.

The50-input baseline and final49 unchanged-input ledgers include existing tests, fonts,
golden, consumers and locks. No old assertion, asset, dependency or fixture changed.
Ignored `target/ring-label-attachment/` retains source pins, original RED, exact receipt,
final logs and explicit setup classification. All own jobs are terminal. This is bounded
offscreen rendering/picking evidence, not native input, all-angle/all-viewport behavior,
Windows/Linux execution or a human-approved ring golden. Root owns Git and integration.
