"""Original MIT OR Apache-2.0 test-only raw SMILES file scanner; no chemistry consumer."""
import hashlib
import json
import re
import sys

raw = sys.stdin.buffer.read(64 * 1024 * 1024 + 1)
assert len(raw) <= 64 * 1024 * 1024
parts = raw.split(b"\n") if raw else []
if raw.endswith(b"\n"):
    parts.pop()
records = []
offset = 0
for number, part in enumerate(parts, 1):
    has_lf = offset + len(part) < len(raw)
    ending = b"\r\n" if has_lf and part.endswith(b"\r") else b"\n" if has_lf else b""
    body = part[:-1] if ending == b"\r\n" else part
    end = offset + len(part) + int(has_lf)
    if body and body[0] not in (0x20, 0x09, 0x0d):
        match = re.fullmatch(rb"([^ \t]+)([ \t]*)(.*)", body, re.DOTALL)
        assert match is not None
        notation, separator, data = match.groups()
        stop = offset + len(notation)
        record = {
            "line_number": number,
            "full_range": [offset, end],
            "notation_range": [offset, stop],
            "separator_range": [stop, stop + len(separator)] if separator else None,
            "application_data_range": [stop + len(separator), offset + len(body)] if separator else None,
            "line_ending_range": [offset + len(body), end],
            "notation_hex": notation.hex(),
            "application_data_hex": data.hex() if separator else None,
        }
        records.append(record)
    offset = end
assert offset == len(raw)
print(json.dumps({"sha256": hashlib.sha256(raw).hexdigest(), "byte_length": len(raw),
                  "physical_lines": len(parts), "records": records}, separators=(",", ":")))
