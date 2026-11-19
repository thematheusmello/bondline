# SPDX-License-Identifier: MIT OR Apache-2.0
"""Check proposed direct-Page Text creation against an independent raw reader."""

import argparse
import hashlib
from pathlib import Path
import struct
import sys

FIXTURES = Path(__file__).resolve().parent
sys.path.insert(0, str(FIXTURES.parents[3] / "cdx/tests/fixtures/resources"))
from resource_oracle import raw_records

BEFORE_SHA = "05b1a1c45dbd11611223a5907c17986eea5f675008c71e6b2116c508bc5699c5"
NAME = b"org.bondline.caption-id"
BODY = "Phase β & q".encode()
POSITION = struct.pack("<ii", 867 * 16384, 577 * 16384)


def prop(tag, payload):
    return struct.pack("<HH", tag, len(payload)) + payload


def obj(tag, raw_id, *children):
    return struct.pack("<HI", tag, raw_id) + b"".join(children) + bytes(2)


def payload(raw, record):
    start, end = record["payload_range"]
    return raw[start:end]


def children(raw, records, owner):
    return {
        record["tag"]: payload(raw, record)
        for record in records
        if record["kind"] == "property" and record["parent"] == owner
    }


def vectors():
    baseline = (FIXTURES.parent / "cdx-caption-edit/single_before.cdx").read_bytes()
    assert len(baseline) == 156 and hashlib.sha256(baseline).hexdigest() == BEFORE_SHA
    framing, header, records = raw_records(baseline)
    assert (framing, header) == ("ImplicitDocument", 28)
    font_table = struct.pack("<HHHHH", 1, 1, 0, 65001, 6) + b"Roboto"
    assert children(baseline, records, None)[0x0100] == font_table
    page = next(i for i, record in enumerate(records) if record.get("tag") == 0x8001)
    assert records[page]["id"] == 90
    at = next(record["range"][0] for record in records if record.get("matched") == page)
    assert at == 152
    raw_id = max(record.get("id", 0) for record in records) + 1
    assert raw_id == 91
    text_fields = {
        0x0200: POSITION,
        0x080C: b"\x00",
        0x000A: struct.pack("<h", 1),
        0x0700: struct.pack("<HHHHHH", 1, 0, 0, 0, 240, 0) + BODY,
    }
    for name, value, persistence in (
        ("first.cdx", b"1:2", True),
        ("after_undo.cdx", b"1:3", True),
        ("implicit_persistence.cdx", b"1:3", None),
        ("duplicate_live_id.cdx", b"1:1", True),
        ("unknown_version.cdx", b"2:3", True),
        ("nonpersistent.cdx", b"1:3", False),
    ):
        tag_fields = {0x0008: NAME, 0x0D00: struct.pack("<h", 3)}
        if persistence is not None:
            tag_fields[0x0D04] = bytes([persistence])
        tag_fields[0x0D05] = value
        annotation = obj(0x8011, 0, *(prop(k, v) for k, v in tag_fields.items()))
        caption_fields = tuple(prop(k, v) for k, v in text_fields.items())
        insertion = obj(0x8006, raw_id, *caption_fields, annotation)
        output = baseline[:at] + insertion + baseline[at:]
        decoded, _, output_records = raw_records(output)
        assert decoded == framing and output[:at] + output[at + len(insertion):] == baseline
        caption = next(i for i, record in enumerate(output_records) if record.get("id") == raw_id)
        tag = next(i for i, record in enumerate(output_records) if record.get("tag") == 0x8011)
        assert output_records[caption]["parent"] == page
        assert output_records[tag]["parent"] == caption and output_records[tag]["id"] == 0
        assert children(output, output_records, caption) == text_fields
        assert children(output, output_records, tag) == tag_fields
        assert sum(record.get("matched") == tag for record in output_records) == 1
        assert not any(record["kind"] == "start" and record["parent"] == tag for record in output_records)
        yield name, output


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", type=Path)
    destination = parser.parse_args().output_dir
    for name, raw in vectors():
        if destination is None:
            assert (FIXTURES / name).read_bytes() == raw, name
        else:
            destination.mkdir(parents=True, exist_ok=True)
            (destination / name).write_bytes(raw)
        print(f"{hashlib.sha256(raw).hexdigest()}  {len(raw):3}  {name}")
