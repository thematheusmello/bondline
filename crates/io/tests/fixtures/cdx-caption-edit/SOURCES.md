# Original CDX caption edit witnesses

The fixtures and generator are MIT OR Apache-2.0. The generator
reproduces the extended single-caption input and builds the ordered
multi-caption input with the existing independent Python struct
writers. [Caption fixture provenance](../cdx-caption/SOURCES.md).

The CDXString contains a little-endian run count, a start and eight
style bytes per run, and a literal body. Each nonfinal run is ASCII,
so byte, scalar and UTF-16 offsets agree through the final start.
The oracle copies the original count and eight style bytes for each
run, changes the body of the selected run, and folds the new lengths
into the cumulative starts. It rereads the edited structure and
checks every non-target record byte for byte, including the original
header, orders, colors, resources and terminators. The changed Text
property uses its canonical short length prefix. [CDXString](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/DataType/CDXString.htm),
[Binary framing](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/IntroCDX.htm),
[Unicode encoding forms](https://www.unicode.org/versions/Unicode17.0.0/core-spec/chapter-3/).

| Case | Page / Text / Run | Body change | Starts | File bytes |
| --- | --- | --- | --- | --- |
| `single_grow` | 1 / 1 / 0 | Bondline → Bondline λ notes | [0] → [0] | 156 → 161 |
| `multi_shift` | 1 / 2 / 0 | AV → AVAV | [0, 2, 3] → [0, 4, 5] | 363 → 361 |
| `tail_unicode` | 1 / 2 / 2 | λΩé → λéπΩ | [0, 2, 3] → [0, 2, 3] | 363 → 361 |
| `multi_shrink` | 1 / 2 / 0 | AV → V | [0, 2, 3] → [0, 1, 2] | 363 → 358 |

Multi-caption encounter order is source IDs 91, 0, 17, 0, which
map to Text IDs 4, 1, 3, 2. Stable ascending source ID gives Text
IDs 1, 2, 3, 4, with encounter order breaking the anonymous tie.
Signed depths −20, −5, 4, 17 give paint order 1, 3, 4, 2. The
target is the second anonymous Text at (120, 228) pt. The two named
sibling captions share a position, with the green H covering the
back black H. [ZOrder](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/ZOrder.htm).

FontTools 4.61.1 computes ink bounds, advances and pair adjustments
from the pinned Roboto Regular font. The oracle checks every
multi-run join for zero pair adjustment and absence of a ligature
sequence beginning across the boundary. The manifest has model and
ink facts before and after each edit. Its blank middle run advances
the cursor by 3.9609375 pt. The blue first run in `multi_shift`
grows to x = 178.62890625 pt, shifting the unchanged orange tail
right by 29.02734375 pt. [BoundsPen](https://fonttools.readthedocs.io/en/latest/pens/boundsPen.html),
[GPOS](https://learn.microsoft.com/en-us/typography/opentype/spec/gpos),
[GSUB](https://learn.microsoft.com/en-us/typography/opentype/spec/gsub).

For rejection checks, starting the final run with T makes a space–T
pair adjustment of −40 design units. Replacing `Bond` with `f` in
the existing [two-run source](../cdx-caption/two_runs.cdx) makes an
f–l join with zero pair adjustment and a ligature in GSUB. The font
lacks U+1FAE0. At the target anchor and 24 pt size, a run of 80 W
crosses the paper boundary within the character budget. A nonfinal
run of 65,536 spaces exceeds both the unsigned start field and the
renderer character budget.

Reproduce the exact files from the repository root with FontTools
4.61.1 installed.

```sh
fixtures=crates/io/tests/fixtures/cdx-caption-edit
out=$(mktemp -d)
python3 -B "$fixtures/generate_caption_edits.py" --output-dir "$out/vectors"
diff -r -x SOURCES.md -x generate_caption_edits.py "$fixtures" "$out/vectors"
```

Manifest SHA-256 is `756cf5c5e487f2447e535acd3822f855493baca09231d6d1f3d052f1f7511c13`.
