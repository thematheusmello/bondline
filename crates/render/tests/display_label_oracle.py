"""Independent fixed OpenType vectors + pinned FreeType for stacked H labels."""

import hashlib
import json
import sys
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont, __version__

assert __version__ == "12.3.0", __version__
assert hashlib.sha256(Path(sys.argv[1]).read_bytes()).hexdigest() == (
    "56a45233d29f11b4dfb86d248e921939d115778f87325e7ae8cc108383d6664d"
)
# Independent pinned OpenType hmtx/glyf facts, not read from renderer output.
metrics = {
    "C": (1333, 119, -20, 1240, 1476),
    "N": (1460, 169, 0, 1288, 1456),
    "O": (1408, 118, -20, 1289, 1476),
    "H": (1460, 169, 0, 1288, 1456),
}
data = json.load(sys.stdin)
symbol = data["symbol"]
assert symbol in {"C", "N", "O"}
assert data["side"] in {"above", "below"}
count = data["hydrogens"]
assert isinstance(count, int) and 0 <= count <= 65535
actual = Image.new("L", (800, 400))
draw = ImageDraw.Draw(actual)
for x, y, width, height in data["boxes"]:
    assert 0 <= x < x + width <= 800 and 0 <= y < y + height <= 400
    draw.rectangle((x, y, x + width - 1, y + height - 1), fill=255)

# Same uniform4x/BASIC/BOX policy,96/64em and0.90 threshold as the protected
# elemental oracle. No fitted translation or per-case threshold adjustment.
scale = 4
reference = Image.new("L", (800 * scale, 400 * scale))
draw = ImageDraw.Draw(reference)
main = ImageFont.truetype(sys.argv[1], 96 * scale, layout_engine=ImageFont.Layout.BASIC)
sub = ImageFont.truetype(sys.argv[1], 64 * scale, layout_engine=ImageFont.Layout.BASIC)
factor = 96 / 2048
_, xmin, ymin, xmax, ymax = metrics[symbol]
glyphs = [(symbol, 400 - (xmin + xmax) * factor / 2,
           200 + (ymin + ymax) * factor / 2, main)]
if count:
    _, xmin, ymin, xmax, ymax = metrics["H"]
    center_y = 200 + (-96 if data["side"] == "above" else 96)
    origin = 400 - (xmin + xmax) * factor / 2
    baseline = center_y + (ymin + ymax) * factor / 2
    glyphs.append(("H", origin, baseline, main))
    cursor = origin + metrics["H"][0] * factor
    for digit in str(count) if count > 1 else "":
        glyphs.append((digit, cursor, baseline + 24, sub))
        cursor += 1150 * (64 / 2048)
for character, x, y, font in glyphs:
    draw.text((x * scale, y * scale), character, fill=255, font=font, anchor="ls")
reference = reference.resize(actual.size, Image.Resampling.BOX)
a = [pixel != 0 for pixel in actual.get_flattened_data()]
b = [pixel >= 128 for pixel in reference.get_flattened_data()]
intersection = sum(x and y for x, y in zip(a, b))
union = sum(x or y for x, y in zip(a, b))
assert union and intersection / union >= 0.90, (symbol, count, data["side"], intersection / union)
print(json.dumps({"freetype_ink_iou": intersection / union}))
