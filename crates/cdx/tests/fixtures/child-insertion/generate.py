# SPDX-License-Identifier: MIT OR Apache-2.0
"""Reproduce the original child insertion vectors and check their raw ownership."""

import argparse
import hashlib
from pathlib import Path
import struct
import sys

FIXTURES = Path(__file__).resolve().parent
sys.path.insert(0, str(FIXTURES.parent / "resources"))
from resource_oracle import raw_records

PREFIX = b"VjCD0100\x04\x03\x02\x01"
HEADER = PREFIX + bytes(16)


def prop(tag, payload, extended=False):
    length = struct.pack("<HI", 65535, len(payload)) if extended else struct.pack("<H", len(payload))
    return struct.pack("<H", tag) + length + payload


def obj(tag, identity, *parts):
    return struct.pack("<HI", tag, identity) + b"".join(parts) + bytes(2)


def tree(*extra_children):
    nested = obj(0xC131, 74, prop(0x4131, b"x\0\0z", extended=True))
    group = obj(0x8002, 19, nested)
    graphic = obj(0x8007, 73, prop(0x4022, b"\0\x80"))
    page = obj(0x8001, 55, prop(0x4101, b"\0\0\x11", extended=True), group, graphic, *extra_children)
    return (page, prop(0x4102, b"\0\0\x12", extended=True), obj(0x8001, 56))


def child(identity=74):
    return obj(0xC132, identity, prop(0x4132, b"\0\x12"), prop(0x4133, b""))


def read_vector(raw, framing, at, has_child=False):
    actual, header_length, records = raw_records(raw)
    assert actual == framing
    assert header_length == (22 if framing == "HeaderDocument" else 28)
    page = next(i for i, event in enumerate(records) if event.get("id") == 55)
    end = next(event for event in records if event.get("matched") == page)
    assert end["range"] == [at + (18 if has_child else 0), at + (20 if has_child else 2)]
    if has_child:
        index, record = next((i, event) for i, event in enumerate(records) if event.get("tag") == 0xC132)
        assert record["parent"] == page and record["id"] == 74 and record["range"] == [at, at + 6]
        assert [event["tag"] for event in records[index + 1:index + 3]] == [0x4132, 0x4133]
        assert [event["parent"] for event in records[index + 1:index + 3]] == [index, index]
        assert records[index + 3]["matched"] == index


def vectors():
    cases = (
        ("implicit", "ImplicitDocument", 87, lambda parts: HEADER + b"".join(parts) + bytes(2)),
        ("explicit", "ExplicitDocument", 93, lambda parts: HEADER + obj(0x8000, 17, *parts)),
        ("header", "HeaderDocument", 87, lambda parts: HEADER[:22] + obj(0x8000, 0, *parts) + bytes(2)),
        ("implicit_file_end", "ImplicitDocumentFileEnd", 87, lambda parts: HEADER + b"".join(parts) + bytes(4)),
        ("explicit_file_end", "ExplicitDocumentFileEnd", 93, lambda parts: HEADER + obj(0x8000, 17, *parts) + bytes(2)),
    )
    ordinary = tree()
    edited = tree(child())
    for name, framing, at, envelope in cases:
        before = envelope(ordinary)
        after = envelope(edited)
        read_vector(before, framing, at)
        read_vector(after, framing, at, has_child=True)
        assert after[at:at + 18] == child() and after[:at] + after[at + 18:] == before
        yield f"{name}_before.cdx", before
        yield f"{name}_after.cdx", after

    duplicate = HEADER + b"".join(tree(child(73))) + bytes(2)
    _, _, records = raw_records(duplicate)
    duplicates = [event for event in records if event.get("id") == 73]
    assert len(duplicates) == 2 and duplicates[0]["parent"] == duplicates[1]["parent"] == 0
    yield "duplicate_direct_id.cdx", duplicate

    late = HEADER + ordinary[0] + child() + ordinary[1] + ordinary[2] + bytes(2)
    _, _, records = raw_records(late)
    misplaced = next(event for event in records if event.get("tag") == 0xC132)
    assert misplaced["range"] == [89, 95] and misplaced["parent"] is None
    yield "after_page_end.cdx", late


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", type=Path, help="Reproduce bytes into this directory, or check the frozen bytes by default")
    output = parser.parse_args().output_dir
    for name, raw in vectors():
        if output is None:
            assert (FIXTURES / name).read_bytes() == raw, name
        else:
            output.mkdir(parents=True, exist_ok=True)
            (output / name).write_bytes(raw)
        print(f"{hashlib.sha256(raw).hexdigest()}  {len(raw):3}  {name}")
