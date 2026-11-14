"""Original analytic pixel expectations; no runtime renderer or new golden is imported."""
import base64
import io
import json
import sys

import PIL
from PIL import Image

assert PIL.__version__ == "12.3.0"
p = json.load(sys.stdin)


def image(value):
    data = base64.b64decode(value, validate=True)
    with Image.open(io.BytesIO(data)) as result:
        result.verify()
    with Image.open(io.BytesIO(data)) as result:
        assert result.format == "PNG" and result.mode == "RGBA"
        assert result.size == (1024, 1024)
        return result.copy()


actual = image(p["cpu"])
gpu = image(p["gpu"])
expected = image(p["reference"])
assert actual.tobytes() == gpu.tobytes() == expected.tobytes(), "multiple-bond full RGBA differs"
for name in ("collapsed", "wrong_order", "wrong_h"):
    assert image(p[name]).tobytes() != expected.tobytes(), (name, "ineffective semantic control")

# A4's centered rounded paper fit and the original declared model coordinates.
pw = 210 * 72 / 25.4
ph = 297 * 72 / 25.4
paper_w = round(pw / ph * 960)
left = (1024 - paper_w) // 2
width = min(paper_w / pw, 960 / ph)
axis_y = 32 + 72 / ph * 960
center_x = left + 96 / pw * paper_w
factors = [-1.5, 1.5] if p["order"] == 2 else [-3, 0, 3]
offsets = [axis_y + factor * width for factor in factors]
rows = set()
for x in range(int(center_x) - 3, int(center_x) + 4):
    for y in range(int(axis_y) - 8, int(axis_y) + 9):
        ink = any(abs(y + 0.5 - position) <= width / 2 for position in offsets)
        assert actual.getpixel((x, y)) == ((0, 0, 0, 255) if ink else (255, 255, 255, 255)), (x, y, "parallel-strip/gap discrepancy")
        if ink:
            rows.add(y)
assert len(rows) == p["order"], (rows, "segments collapsed or disappeared")
assert all(any(abs(y + 0.5 - position) <= width / 2 for y in rows) for position in offsets)
print(json.dumps({"rgba_bytes": len(actual.tobytes()), "separate_ink_rows": sorted(rows), "controls": 3}))
