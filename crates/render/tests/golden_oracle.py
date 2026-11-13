"""Decode the human-approved reference independently and compare all raw RGBA bytes."""
import hashlib
import sys
from pathlib import Path
from PIL import Image, __version__

assert __version__ == "12.3.0", __version__
path = Path(sys.argv[1])
assert hashlib.sha256(path.read_bytes()).hexdigest() == sys.argv[2], "reference hash changed"
with Image.open(path) as image:
    assert image.format == "PNG" and image.size == (1024, 1024) and image.mode == "RGBA"
    expected = image.tobytes()
actual = sys.stdin.buffer.read(len(expected) + 1)
assert len(actual) == len(expected), (len(actual), len(expected))
assert actual == expected, "rendered pixels differ from the human-approved reference"
