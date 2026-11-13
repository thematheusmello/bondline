"""Generate original IEEE reference data using Python's independent float conversion."""
import json
import math
import pathlib
import random
import struct

rng = random.Random(0xB0D11E)
vectors = []
for width, code in [(32, "f"), (64, "d")]:
    count = 0
    while count < 128:
        bits = rng.getrandbits(width)
        raw = bits.to_bytes(width // 8, "big")
        value = struct.unpack(">" + code, raw)[0]
        if not math.isfinite(value):
            continue
        literal = value.hex()
        assert struct.pack(">" + code, float.fromhex(literal)) == raw
        vectors.append({"width": width, "hex": literal, "bits": raw.hex()})
        count += 1
assert len(vectors) == 256
target = pathlib.Path(__file__).resolve().parents[1] / "compat/hexf-parse/tests/python-vectors.json"
target.parent.mkdir(parents=True, exist_ok=True)
target.write_text(json.dumps(vectors, indent=2) + "\n", encoding="utf-8")
assert json.loads(target.read_text(encoding="utf-8")) == vectors
print(f"Wrote {len(vectors)} independently generated float vectors")
