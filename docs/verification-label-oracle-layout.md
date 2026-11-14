# Portable external label layout oracle

The Mac job132333092799 in hosted run43557351022, source983ffee, failed two external
label comparisons at the unchanged90% ink-IoU threshold. This correction is test-only,
on root metadata merge4fcea85; it changes no production renderer, font, scene or golden.

## Reproduction and cause

The actual root interpreter is Python3.13.0 with Pillow12.3.0, FreeType2.14.3 and RAQM
available. Ignored target/label-layout/python-basic supplies BASIC only when a script's
truetype call has no explicit layout engine, then runs that unchanged script. With
BONDLINE_PYTHON pointing to that wrapper, the unchanged four label unit tests produce
two passes and two failures, session5226 exit101, target/label-oracle-basic-red.log.
The terminal failure is3035/3501=0.8668951728077692; H1/false is
2569/2892=0.8883125864453665. Both exactly match the reported hosted values. This
reproduces the mechanism locally; it does not separately attest the hosted wheel's flags.

[Pillow12.3.0 source](https://github.com/python-pillow/Pillow/blob/12.3.0/src/PIL/ImageFont.py#L245-L256)
selects RAQM by default when available and BASIC otherwise. Its
[getlength documentation](https://github.com/python-pillow/Pillow/blob/12.3.0/src/PIL/ImageFont.py#L298-L314)
states that BASIC returns whole-pixel advances. The fetched12.3.0 file is63750bytes,
SHA24fa5feeb91b4bf63eaad0ebba08a8161e9c889d9fd056a37c928134097b9649,
identical to the installed ImageFont.py. The versioned readthedocs URL was inaccessible;
the exact release source was available publicly. No dependency was installed or changed.

At96em BASIC reports C62/H68, while RAQM reports62.484375/68.4375. At64em each digit
reports36 under BASIC and35.9375 under RAQM. The generalized oracle had changed its
original fixed design advances to getlength, coupling placement to this build option.
The public [parent00e5b02 oracle](https://github.com/thematheusmello/bondline/blob/00e5b02/crates/render/tests/label_oracle.py)
used the original unhinted C/H/3 constants. Its fetched1752bytes hash to
53ba9861ff1839c71345eef49241e3c49fd48b8b9e97f35df299955feaaf0e62.

## Independent metrics and correction

fontTools is absent from this interpreter; its historical4.61.1 C/H/3 vectors were
not described as rerun. A new ignored stdlib-only probe independently reads the pinned
TTF's cmap4, head/hhea, hmtx and loca/glyf headers, using the published
[cmap](https://learn.microsoft.com/en-us/typography/opentype/spec/cmap),
[head](https://learn.microsoft.com/en-us/typography/opentype/spec/head),
[hhea](https://learn.microsoft.com/en-us/typography/opentype/spec/hhea),
[hmtx](https://learn.microsoft.com/en-us/typography/opentype/spec/hmtx),
[loca](https://learn.microsoft.com/en-us/typography/opentype/spec/loca) and
[glyf](https://learn.microsoft.com/en-us/typography/opentype/spec/glyf) definitions.
This does not read metrics from Bondline or copy its parser/layout implementation.

The source font is515100bytes, SHA
56a45233d29f11b4dfb86d248e921939d115778f87325e7ae8cc108383d6664d,
2048 units/em. C/H glyphs40/45 advance1333/1460; digits0–9 map to21–30 and each
advance1150. C's bounds remain119,-20,1240,1476. Those unhinted values give the
previous fractional advances independently. The bounded probe is
target/label-layout/ttf_metrics.py, SHA
1ec531da54b3ccba670fe5a6e73b06089385c7b9b0d0374e3847f6ae084f85bd;
its result ttf-metrics.json hashes to
83c86327c365e9aa6324bba0fe5dc6f0a6f6e13a69729d1387841a9d147a10b3.

The oracle now explicitly uses always-available BASIC FreeType rasterization, while
placing C/H/digits using those independently verified design-unit advances. It does
not search offsets, align against actual pixels, require RAQM, skip a consumer or
lower0.90. Both orientations, all authored0/1/2/3/12/65535 counts, invented-H and
reversed-digit negatives remain intact. One new Rust unit checks the independently
decoded ten digit IDs/advances; production code is unchanged.

## Actual checks and limits

Forced-BASIC label units now pass5 tests, session71760 exit0,
target/label-oracle-basic-green.log. The two terminal IoUs are0.966025255562237 and
0.9668973818838399. The whole render package passes31 tests with the ordinary pinned
interpreter, session45993 exit0, target/label-oracle-render-green.log. This includes
CPU/GPU analytic cases, the same human-approved single-bond image and its corrupted-
pixel negative. Render all-target clippy-Dwarnings and workspace rustfmt pass; logs
are target/label-oracle-clippy.log and target/label-oracle-fmt.log. Root uses its own
absolute target and CARGO_INCREMENTAL=0/default debuginfo; no full gate was rerun here.

The six production files still concatenate to
8c2143f18f47ce48bb1beaae86c0194c6169f0695588bff495a53c107fea832b.
The font and approved golden SHA1b650231527f47db5dc7c368644939907d2b038d32f25e77578318f483521bbc
are unchanged. No new image approval or native app rerun is implied. Root owns the new
full/hosted checks; no cross-platform pass is substituted from this local result.
Exact file/probe/log identities are in target/label-oracle-layout-handoff.json.
No worker Git/GUI operation, other-worktree write, dependency install or live job remains.

### Receiving-root aggregate

After the source-only CDX metadata merge and sanitizer-seed configuration, the exact
corrected root source passes487 workspace tests plus3 native bridge tests,490 total.
`cargo xtask ci` session31715 exits0; target/font-layout-corrected-ci.log SHA
1b2d0e0e1af11134bd977bf71cddd3184c57ba4f32690d7f0780f4949260d0ee records formatting,
all-target clippy, tests, layers, unchanged license closures/assets and independent corpus
consumers. The handoff's three original file hashes were checked before this additive
paragraph; its six production files, font and approved golden still match exactly.
The source983ffee hosted Mac/Windows failures remain recorded; a new matching hosted run
is required before advancing main. No native workflow is claimed from this test-only delta.
