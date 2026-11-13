"""Independent test-only FreeType masks; tolerance chosen before comparing outputs."""
import json
import sys
from PIL import Image, ImageDraw, ImageFont, __version__

assert __version__ == "12.3.0", __version__
data = json.load(sys.stdin)
actual = Image.new("L", (400, 400))
draw = ImageDraw.Draw(actual)
for x, y, width, height in data["boxes"]:
    assert 0 <= x < x + width <= 400 and 0 <= y < y + height <= 400
    draw.rectangle((x, y, x + width - 1, y + height - 1), fill=255)
expected = Image.new("L", (400, 400))
draw = ImageDraw.Draw(expected)
main = ImageFont.truetype(sys.argv[1], 96)
sub = ImageFont.truetype(sys.argv[1], 64)
# Independent fontTools 4.61.1 vectors: units 2048; C ink x=119..1240,
# y=-20..1476, advance=1333; H advance=1460; 3 advance=1150.
origin = 200 - (119 + 1240) * (96 / 2048) / 2
baseline = 200 + (-20 + 1476) * (96 / 2048) / 2
if data["toward_right"]:
    three_x = origin - 1150 * (64 / 2048)
    h_x = three_x - 1460 * (96 / 2048)
else:
    h_x = origin + 1333 * (96 / 2048)
    three_x = h_x + 1460 * (96 / 2048)
for char, x, y, font in [("C", origin, baseline, main), ("H", h_x, baseline, main),
                          ("3", three_x, baseline + 24, sub)]:
    draw.text((x, y), char, fill=255, font=font, anchor="ls")
reference = [value >= 128 for value in expected.get_flattened_data()]
pixels = [value != 0 for value in actual.get_flattened_data()]
intersection = sum(a and b for a, b in zip(pixels, reference))
union = sum(a or b for a, b in zip(pixels, reference))
# Different rasterizers/hinting; require >=90% ink intersection-over-union at 96 em.
assert union and intersection / union >= 0.90, (intersection, union, intersection / union)
print(json.dumps({"freetype_ink_iou": intersection / union}))
