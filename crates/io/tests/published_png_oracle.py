"""Physical PNG contract for the original 512 pt propane drawing; independent of Bondline."""
import io
import math
import struct
import sys
import zlib
from PIL import Image

dpi = int(sys.argv[1])
transparent = sys.argv[2] == "true"
data = sys.stdin.buffer.read()
assert data[:8] == b"\x89PNG\r\n\x1a\n"
offset = 8
tags = []
while offset < len(data):
    size, = struct.unpack_from(">I", data, offset)
    tag = data[offset + 4:offset + 8]
    value = data[offset + 8:offset + 8 + size]
    crc, = struct.unpack_from(">I", data, offset + 8 + size)
    assert zlib.crc32(tag + value) == crc
    tags.append(tag)
    if tag == b"pHYs":
        ppm = round(dpi * 10000 / 254)
        assert struct.unpack(">IIB", value) == (ppm, ppm, 1)
    offset += 12 + size
assert offset == len(data) and tags.count(b"pHYs") == 1 and tags.index(b"pHYs") < tags.index(b"IDAT")
with Image.open(io.BytesIO(data)) as image:
    side = math.ceil(512 * dpi / 72)
    assert image.format == "PNG" and image.mode == "RGBA" and image.size == (side, side)
    paper = (0, 0, 0, 0) if transparent else (255, 255, 255, 255)
    ink = (0, 0, 0, 255)
    assert {colour for _, colour in image.getcolors(8)} == {paper, ink}
    for x in (0, side - 1):
        for y in (0, side - 1):
            assert image.getpixel((x, y)) == paper
    # Two 80 pt single bonds run (160,240)–(224,192)–(288,240).
    for px, py, colour in [(192,216,ink), (256,216,ink), (224,216,paper), (192,226,paper)]:
        assert image.getpixel((int(px * dpi / 72), int(py * dpi / 72))) == colour
