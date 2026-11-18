"""Independent OpenType positions + supersampled Pillow/FreeType coverage and RGB."""
import hashlib
import json
import sys
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont, __version__

assert __version__ == "12.3.0", __version__
font_path = Path(sys.argv[1])
vectors = json.loads(Path(sys.argv[2]).read_text())
assert hashlib.sha256(font_path.read_bytes()).hexdigest() == vectors["font_sha256"]
data = json.load(sys.stdin)
width, height = data["dimensions"]
assert 1 <= width <= 2048 and 1 <= height <= 2048
actual = Image.new("RGB", (width, height), "white")
draw = ImageDraw.Draw(actual)
for x, y, w, h, color in data["boxes"]:
    assert 0 <= x < x + w <= width and 0 <= y < y + h <= height
    assert color[3] == 255
    draw.rectangle((x, y, x + w - 1, y + h - 1), fill=tuple(color[:3]))
supersample = 4
reference = Image.new("RGB", (width * supersample, height * supersample), "white")
draw = ImageDraw.Draw(reference)
cursor, baseline = data["baseline"]
colors = set()
for run in data["runs"]:
    em, color = run["em"], tuple(run["rgb"])
    assert em * supersample == round(em * supersample)
    font = ImageFont.truetype(font_path, round(em * supersample), layout_engine=ImageFont.Layout.BASIC)
    factor = em / vectors["units_per_em"]
    previous = None  # Authored run boundaries intentionally reset pair kerning.
    for char in run["text"]:
        metrics = vectors["metrics"][char]
        if previous is not None:
            pair = previous + char
            assert pair in vectors["kern"], pair
            cursor += vectors["kern"][pair] * factor
        if metrics["bounds"] is not None:
            draw.text((cursor * supersample, baseline * supersample), char,
                      fill=color, font=font, anchor="ls")
            colors.add(color)
        cursor += metrics["advance"] * factor
        previous = char
reference = reference.resize((width, height), Image.Resampling.BOX)
scores = {}
palette = sorted(colors) + [(255, 255, 255)]
reference_colors = [min(palette, key=lambda c: sum((a-b)**2 for a, b in zip(p, c)))
                    for p in reference.get_flattened_data()]
for color in colors:
    # Nearest exact run colour versus white independently thresholds FreeType hints.
    expected = [p == color for p in reference_colors]
    actual_ink = [p == color for p in actual.get_flattened_data()]
    common = sum(a and b for a, b in zip(actual_ink, expected))
    union = sum(a or b for a, b in zip(actual_ink, expected))
    assert union and common / union >= 0.90, (color, common, union, common / union if union else None)
    scores[str(color)] = common / union
assert colors
assert set(actual.get_flattened_data()) <= colors | {(255, 255, 255)}, "unexpected ink color"
print(json.dumps({"freetype_color_ious": scores, "fonttools": vectors["fonttools"],
                  "end_x": cursor, "pillow": __version__}))
