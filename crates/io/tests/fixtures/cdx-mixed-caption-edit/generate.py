# SPDX-License-Identifier: MIT OR Apache-2.0
"""Independently reconstruct the complete source for each expected run-edit state."""
import argparse
from hashlib import sha256
from pathlib import Path
import struct
import sys

HERE = Path(__file__).resolve().parent
ROOT = next(p for p in HERE.parents if (p / "crates/io/tests/fixtures/cdx-label-preview").is_dir())
ORACLE = ROOT / "crates/cdx/tests/fixtures/resources/resource_oracle.py"
assert sha256(ORACLE.read_bytes()).hexdigest() == "a5a2df85a9fc1a2df9ba57b1a7b5538c812a6f724e18d25a42a16d98d6ad5db2"
sys.path.insert(0, str(ORACLE.parent))
from resource_oracle import raw_records

PAGE, NODE, TEXT, BODY, Z = 0x8001, 0x8004, 0x8006, 0x0700, 0x000a
STAGES = (
    ("before", "H ", "λΩé", False),
    ("after_prefix", "H H ", "λΩé", False),
    ("after_both", "H H ", "λéπΩ", False),
    ("after_tail", "H ", "λéπΩ", False),
    ("extended_before", "H ", "λΩé", True),
)


def prop(tag, value, extended=False):
    header = struct.pack("<HHI", tag, 65535, len(value)) if extended else struct.pack("<HH", tag, len(value))
    return header + value


def obj(tag, raw_id, *fields):
    return struct.pack("<HI", tag, raw_id) + b"".join(fields) + bytes(2)


def single(records, tag, owner, kind):
    index, = (i for i, r in enumerate(records)
              if r.get("tag") == tag and r["parent"] == owner and r["kind"] == kind)
    return index


def position(x, y):
    return prop(0x0200, struct.pack("<ii", round(y * 65536), round(x * 65536)))


def target_record(records):
    # Raw ID 73 occurs on three distinct Text objects. The proper owner selects exactly one.
    page = single(records, PAGE, None, "start")
    aliases = [(i, r["parent"]) for i, r in enumerate(records)
               if r["kind"] == "start" and r.get("tag") == TEXT and r["id"] == 73]
    assert [records[parent]["tag"] for _, parent in aliases] == [NODE, NODE, PAGE]
    direct, = (index for index, parent in aliases if parent == page)
    return single(records, BODY, direct, "property")


def source(prefix, tail, extended):
    base = (ROOT / "crates/io/tests/fixtures/cdx-label-preview/boxed.cdx").read_bytes()
    assert sha256(base).hexdigest() == "b7d89d7d117b449e25cfcb61a9221f2a75d1cc7cf05b74073a3f921af162f2ee"
    records = raw_records(base)[2]
    page = single(records, PAGE, None, "start")
    at, = (r["range"][0] for r in records if r.get("matched") == page)
    assert prefix.isascii() and tail
    target_string = struct.pack("<11H", 2, 0, 0, 0, 480, 5, len(prefix), 0, 0, 240, 0) + (prefix + tail).encode()
    identity = obj(0x8011, 0, prop(0x0008, b"org.bondline.caption-id"), prop(0x0d00, b"\3\0"),
                   prop(0x0d04, b"\1"), prop(0x0d05, b"1:19"))
    target = obj(TEXT, 73, position(158.947265625, 131.65625), prop(0x080c, bytes(1)),
                 prop(BODY, target_string, extended), prop(Z, struct.pack("<h", 7)), identity)
    peer = obj(TEXT, 9, prop(Z, struct.pack("<h", 2)), position(80.25, 100.75), prop(0x080c, bytes(1)),
               prop(BODY, struct.pack("<6H", 1, 0, 0, 0, 240, 0) + b"H"))
    raw = base[:at] + target + peer + base[at:]
    parsed = raw_records(raw)[2]
    r = parsed[target_record(parsed)]
    a, b = r["payload_range"]
    assert raw[a:b] == target_string
    return raw


def check_delta(before, after):
    old, new = raw_records(before)[2], raw_records(after)[2]
    index = target_record(old)
    assert index == target_record(new) and len(old) == len(new)
    a, b = old[index]["range"]
    c, d = new[index]["range"]
    assert before[:a] == after[:c] and before[b:] == after[d:]
    ao, _ = old[index]["payload_range"]
    co, _ = new[index]["payload_range"]
    assert before[ao:ao+12] == after[co:co+12]  # count, start 0, first style
    assert before[ao+14:ao+22] == after[co+14:co+22]  # second style


def main():
    arg = argparse.ArgumentParser(description=__doc__)
    arg.add_argument("--output-dir", type=Path)
    dst = arg.parse_args().output_dir
    data = {name: source(a, b, ext) for name, a, b, ext in STAGES}
    for old, new in (("before", "after_prefix"), ("after_prefix", "after_both"), ("before", "after_tail"),
                     ("after_both", "after_tail"), ("after_tail", "before"), ("extended_before", "after_prefix")):
        check_delta(data[old], data[new])
    for name, raw in data.items():
        file = (dst or HERE) / (name + ".cdx")
        if dst is None:
            assert file.read_bytes() == raw
        else:
            dst.mkdir(exist_ok=True, parents=True)
            file.write_bytes(raw)
        print(name, len(raw), sha256(raw).hexdigest())


if __name__ == "__main__":
    main()
