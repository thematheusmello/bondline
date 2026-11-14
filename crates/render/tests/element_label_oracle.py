"""Independent test-only OpenType metric vectors and FreeType label masks."""

import hashlib
import json
import math
import sys
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont, __version__

assert __version__ == "12.3.0", __version__
assert hashlib.sha256(Path(sys.argv[1]).read_bytes()).hexdigest() == (
    "56a45233d29f11b4dfb86d248e921939d115778f87325e7ae8cc108383d6664d"
)
# advance,xMin,yMin,xMax,yMax: independent stdlib cmap4/hmtx/loca/glyf read
# of the pinned font. This table is not imported from the Rust renderer.
metrics = {
    "C": (1333, 119, -20, 1240, 1476),
    "H": (1460, 169, 0, 1288, 1456),
    "N": (1460, 169, 0, 1288, 1456),
    "O": (1408, 118, -20, 1289, 1476),
    "F": (1132, 169, 0, 1071, 1456),
    "l": (497, 156, 0, 341, 1536),
    "B": (1275, 169, 0, 1160, 1456),
    "r": (693, 140, 0, 663, 1102),
    "S": (1215, 80, -20, 1138, 1476),
    "i": (497, 141, 0, 360, 1476),
    "e": (1085, 93, -20, 1011, 1102),
    "g": (1149, 96, -426, 1010, 1102),
    "L": (1102, 169, 0, 1052, 1456),
}
data = json.load(sys.stdin)
symbol = data["symbol"]
# CL is a deliberate wrong-case rejection control, not an element symbol.
assert symbol in {"N", "O", "F", "Cl", "CL", "Br", "Si", "He", "Og"}
left = data["hydrogen_left"]
hydrogens = data["hydrogens"]
assert isinstance(left, bool) and isinstance(hydrogens, int) and 0 <= hydrogens <= 65535
actual = Image.new("L", (800, 400))
draw = ImageDraw.Draw(actual)
for x, y, width, height in data["boxes"]:
    assert 0 <= x < x + width <= 800 and 0 <= y < y + height <= 400
    draw.rectangle((x, y, x + width - 1, y + height - 1), fill=255)
# Pillow's FT_LOAD_DEFAULT pixel-grid hints differ from unhinted design outlines.
# Integrate a4x FreeType reference back to the same96-em canvas; no shape translation.
supersample = 4
reference = Image.new("L", (actual.width * supersample, actual.height * supersample))
draw = ImageDraw.Draw(reference)
main = ImageFont.truetype(sys.argv[1], 96 * supersample, layout_engine=ImageFont.Layout.BASIC)
sub = ImageFont.truetype(sys.argv[1], 64 * supersample, layout_engine=ImageFont.Layout.BASIC)
factor = 96 / 2048
anchor = symbol[-1] if left else symbol[0]
_, xmin, ymin, xmax, ymax = metrics[anchor]
prefix = sum(metrics[c][0] for c in symbol[:-1]) if left else 0
origin = 400 - prefix * factor - (xmin + xmax) * factor / 2
baseline = 200 + (ymin + ymax) * factor / 2
cursor = origin
glyphs = []
for character in symbol:
    glyphs.append((character, cursor, baseline, main))
    cursor += metrics[character][0] * factor
if hydrogens:
    digits = str(hydrogens) if hydrogens > 1 else ""
    digit_advance = 1150 * (64 / 2048)
    if left:
        cursor = origin - len(digits) * digit_advance
        h_origin = cursor - metrics["H"][0] * factor
    else:
        h_origin = cursor
        cursor = h_origin + metrics["H"][0] * factor
    glyphs.append(("H", h_origin, baseline, main))
    for digit in digits:
        glyphs.append((digit, cursor, baseline + 24, sub))
        cursor += digit_advance
for character, x, y, font in glyphs:
    draw.text((x * supersample, y * supersample), character, fill=255, font=font, anchor="ls")
reference = reference.resize(actual.size, Image.Resampling.BOX)
actual_ink = [value != 0 for value in actual.get_flattened_data()]
reference_ink = [value >= 128 for value in reference.get_flattened_data()]
intersection = sum(a and b for a, b in zip(actual_ink, reference_ink))
union = sum(a or b for a, b in zip(actual_ink, reference_ink))
glyph_ious = []
for character, x, y, font in glyphs:
    x0, y0, x1, y1 = font.getbbox(character, anchor="ls")
    box = (math.floor(x + x0 / supersample) - 1,
           math.floor(y + y0 / supersample) - 1,
           math.ceil(x + x1 / supersample) + 1,
           math.ceil(y + y1 / supersample) + 1)
    a = [value != 0 for value in actual.crop(box).get_flattened_data()]
    b = [value >= 128 for value in reference.crop(box).get_flattened_data()]
    joined = sum(left or right for left, right in zip(a, b))
    common = sum(left and right for left, right in zip(a, b))
    assert joined, ("empty glyph", character)
    glyph_ious.append((character, common / joined))
# Same predeclared FreeType/ab_glyph threshold as the unchanged carbon oracle.
if union and intersection / union < 0.90:
    diagnostic = Path(__file__).resolve().parents[3] / f"target/neutral-label-failures-{supersample}x"
    diagnostic.mkdir(parents=True, exist_ok=True)
    name = f"{symbol}-{left}-{hydrogens}"
    actual.save(diagnostic / f"{name}-actual.png")
    reference.save(diagnostic / f"{name}-reference.png")
    (diagnostic / f"{name}.json").write_text(json.dumps(data), encoding="ascii")
assert union and intersection / union >= 0.90, (symbol, left, hydrogens, intersection / union)
# Per-glyph scores are diagnostic; thin hinted glyphs have a different error budget.
# The tested H2/H3 control uses its own matching positive ROI in Rust.
print(json.dumps({"freetype_ink_iou": intersection / union,
                  "minimum_glyph_iou": min(iou for _, iou in glyph_ious)}))
