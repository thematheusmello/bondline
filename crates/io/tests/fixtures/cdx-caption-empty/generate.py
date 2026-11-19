# SPDX-License-Identifier: MIT OR Apache-2.0
"""Reproduce empty CDX Poster, first-caption and after-Undo byte witnesses."""

import argparse
import hashlib
import json
from pathlib import Path
import struct
import sys

FIXTURES = Path(__file__).resolve().parent
ORACLE = Path("crates/cdx/tests/fixtures/resources/resource_oracle.py")
ROOT = next(parent for parent in FIXTURES.parents if (parent / ORACLE).is_file())
assert hashlib.sha256((ROOT / ORACLE).read_bytes()).hexdigest() == (
    "a5a2df85a9fc1a2df9ba57b1a7b5538c812a6f724e18d25a42a16d98d6ad5db2"
)
sys.path.insert(0, str((ROOT / ORACLE).parent))
from resource_oracle import read

HEADER = b"VjCD0100\x04\x03\x02\x01" + bytes(16)
END = bytes(2)
PAGE, TEXT, OBJECT_TAG = 0x8001, 0x8006, 0x8011
PAGE_ID, RAW_TEXT_ID = 90, 91
BODY = "Phase β & q"
BASELINE = (144.25, 216.75)
POSTER_SIZE = (512.25, 384.5)
NAME = b"org.bondline.caption-id"


def prop(tag, payload, extended=False):
    length = struct.pack("<HI", 65535, len(payload)) if extended else struct.pack("<H", len(payload))
    return struct.pack("<H", tag) + length + payload


def obj(tag, identity, *parts):
    return struct.pack("<HI", tag, identity) + b"".join(parts) + END


def font_entry(identity=0, charset=65001, name=b"Roboto"):
    return struct.pack("<HHH", identity, charset, len(name)) + name


def fonts(*entries, platform=1):
    return prop(0x0100, struct.pack("<HH", platform, len(entries)) + b"".join(entries), extended=True)


def fixed(value):
    return struct.pack("<i", int(value * 65536))


FONT = fonts(font_entry())
WIDTH = prop(0x0812, fixed(POSTER_SIZE[0]))
HEIGHT = prop(0x0813, fixed(POSTER_SIZE[1]), extended=True)
POSTER = prop(0x0811, b"\x01")
DIMENSIONS = (HEIGHT, POSTER, WIDTH)
WHITE = prop(0x0300, struct.pack("<HHHH", 1, 65535, 65535, 65535))


def envelope(framing, contents):
    if framing == "HeaderDocument":
        return HEADER[:22] + obj(0x8000, 0, contents) + END
    if framing.startswith("ExplicitDocument"):
        body = obj(0x8000, 17, contents)
    else:
        body = contents + END
    return HEADER + body + (END if framing.endswith("FileEnd") else b"")


def caption(live, selector):
    marker = obj(
        OBJECT_TAG, 0,
        prop(0x0008, NAME), prop(0x0D00, struct.pack("<h", 3)),
        prop(0x0D04, b"\x01"), prop(0x0D05, f"1:{live}".encode()),
    )
    return obj(
        TEXT, RAW_TEXT_ID,
        prop(0x0200, fixed(BASELINE[1]) + fixed(BASELINE[0])),
        prop(0x080C, b"\x00"), prop(0x000A, struct.pack("<h", 0)),
        prop(0x0700, struct.pack("<HHHHHH", 1, 0, selector, 0, 240, 0) + BODY.encode()), marker,
    )


def model(live):
    texts = []
    if live is not None:
        texts.append({
            "id": live,
            "position": dict(zip(("x_pt", "y_pt"), BASELINE)),
            "runs": [{"text": BODY, "style": {
                "font_family": "Roboto", "size_pt": 12.0, "face": 0, "rgb": [0.0] * 3,
            }}],
        })
    return {"pages": [{
        "id": 1, "size": dict(zip(("width_pt", "height_pt"), POSTER_SIZE)),
        "fragments": [], "texts": texts,
    }]}


def owned_fields(raw, records, owner):
    return {
        r["tag"]: raw[slice(*r["payload_range"])]
        for r in records if r["kind"] == "property" and r["parent"] == owner
    }


def opening(records, tag):
    return next(i for i, record in enumerate(records) if record.get("tag") == tag)


def matched_end(records, owner):
    return next(r["range"][0] for r in records if r.get("matched") == owner)


def verify(framing, selector, raw, base=None, live=None):
    fact = read(raw)
    assert fact["framing"] == framing
    head = 22 if framing == "HeaderDocument" else 28
    assert fact["header_range"] == [0, head]
    records = fact["records"]
    page = opening(records, PAGE)
    assert records[page]["id"] == PAGE_ID
    root = 0 if framing.startswith("ExplicitDocument") or framing == "HeaderDocument" else None
    assert records[page]["parent"] == root
    assert owned_fields(raw, records, page) == {
        0x0813: fixed(POSTER_SIZE[1]), 0x0811: b"\x01", 0x0812: fixed(POSTER_SIZE[0]),
    }
    query = fact["query"]
    font = next(entry for entry in query["fonts"] if entry["id"] == selector)
    assert (query["ok"], query["font_platform"], font["ordinal"], font["charset"]) == (True, 1, selector, 65001)
    assert font["name"]["hex"] == b"Roboto".hex()
    background = next(entry for entry in query["color_lookups"] if entry["index"] == 2)
    assert background["rgb"] == [65535] * 3
    current_end = matched_end(records, page)
    if base is None:
        assert [r["tag"] for i, r in enumerate(records) if i != root and r["kind"] == "start"] == [PAGE]
        return current_end, background["origin"]

    text = opening(records, TEXT)
    tag = opening(records, OBJECT_TAG)
    start = records[text]["range"][0]
    assert records[text]["parent"] == page and records[text]["id"] == RAW_TEXT_ID
    assert records[tag]["parent"] == text and records[tag]["id"] == 0
    assert owned_fields(raw, records, text) == {
        0x0200: fixed(BASELINE[1]) + fixed(BASELINE[0]), 0x080C: b"\0", 0x000A: END,
        0x0700: struct.pack("<HHHHHH", 1, 0, selector, 0, 240, 0) + BODY.encode(),
    }
    assert owned_fields(raw, records, tag) == {0x0008: NAME, 0x0D00: b"\x03\0", 0x0D04: b"\x01", 0x0D05: f"1:{live}".encode()}
    assert matched_end(records, tag) + 2 == matched_end(records, text)
    assert matched_end(records, text) + 2 == current_end
    assert start + 112 == current_end and raw[:start] + raw[current_end:] == base
    return start, background["origin"]


def file_fact(name, raw):
    return {"file": name, "byte_length": len(raw), "sha256": hashlib.sha256(raw).hexdigest()}


def vectors():
    implicit = "ImplicitDocument"
    ordinal_one = fonts(font_entry(9, 1252, b"Arial"), font_entry(1))
    cases = (
        ("implicit", implicit, 83, 0, FONT, b""),
        ("explicit", "ExplicitDocument", 89, 0, FONT, b""),
        ("header", "HeaderDocument", 83, 0, FONT, b""),
        ("implicit_file_end", "ImplicitDocumentFileEnd", 83, 0, FONT, b""),
        ("explicit_file_end", "ExplicitDocumentFileEnd", 89, 0, FONT, b""),
        ("ordinal_one_white", implicit, 94, 1, ordinal_one, WHITE),
    )
    manifest = {"cases": {}, "controls": {}}
    for name, framing, at, selector, table, palette in cases:
        before = envelope(framing, table + obj(PAGE, PAGE_ID, *DIMENSIONS) + palette)
        offset, origin = verify(framing, selector, before)
        assert at == offset and origin == ("entry" if palette else "absent_default")
        case = {"framing": framing, "insertion_offset": at, "font_selector": selector, "background_origin": origin}
        first = None
        for suffix, live in (("before", None), ("first", 1), ("after_undo", 2)):
            child = () if live is None else (caption(live, selector),)
            raw = envelope(framing, table + obj(PAGE, PAGE_ID, *DIMENSIONS, *child) + palette)
            assert verify(framing, selector, raw, before if child else None, live) == (at, origin)
            if live == 1:
                first = raw
            elif live == 2:
                assert [i for i, (a, b) in enumerate(zip(first, raw, strict=True)) if a != b] == [at + 107]
            file = f"{name}_{suffix}.cdx"
            case[suffix] = {**file_fact(file, raw), "model": model(live)}
            yield file, raw
        manifest["cases"][name] = case

    page_bytes = obj(PAGE, PAGE_ID, *DIMENSIONS)
    controls = (
        ("missing_font", page_bytes),
        ("empty_font", fonts() + page_bytes),
        ("font_platform0", fonts(font_entry(), platform=0) + page_bytes),
        ("font_id_ordinal_disagree", fonts(font_entry(2)) + page_bytes),
        ("font_not_utf8", fonts(font_entry(charset=1252)) + page_bytes),
        ("duplicate_font_table", FONT + FONT + page_bytes),
        ("font_on_page", obj(PAGE, PAGE_ID, *DIMENSIONS, FONT)),
        ("empty_palette", FONT + page_bytes + prop(0x0300, END)),
        ("nonwhite_palette", FONT + page_bytes + prop(0x0300, struct.pack("<HHHH", 1, 65535, 65535, 65534))),
        ("no_drawing_space", FONT + obj(PAGE, PAGE_ID, HEIGHT, WIDTH)),
        ("missing_width", FONT + obj(PAGE, PAGE_ID, HEIGHT, POSTER)),
        ("wrong_width_size", FONT + obj(PAGE, PAGE_ID, HEIGHT, POSTER, prop(0x0812, b"\0\x02"))),
        ("zero_height", FONT + obj(PAGE, PAGE_ID, prop(0x0813, fixed(0)), POSTER, WIDTH)),
        ("root_width", FONT + obj(PAGE, PAGE_ID, HEIGHT, POSTER) + WIDTH),
        ("duplicate_width", FONT + obj(PAGE, PAGE_ID, *DIMENSIONS, WIDTH)),
        ("opaque_page_property", FONT + obj(PAGE, PAGE_ID, *DIMENSIONS, prop(0x4141, b"\x01"))),
        ("opaque_page_child", FONT + obj(PAGE, PAGE_ID, *DIMENSIONS, obj(0xC141, 4))),
        ("two_pages", FONT + page_bytes + obj(PAGE, PAGE_ID + 1, *DIMENSIONS)),
        ("no_page", FONT),
    )
    for name, contents in controls:
        raw = envelope(implicit, contents)
        assert read(raw)["framing"] == implicit
        file = f"{name}.cdx"
        manifest["controls"][name] = {**file_fact(file, raw), "expected_open": "refused" if name == "two_pages" else "legacy_elemental"}
        yield file, raw
    yield "cases.json", (json.dumps(manifest, ensure_ascii=False, indent=2) + "\n").encode()


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
        print(f"{hashlib.sha256(raw).hexdigest()}  {len(raw):5}  {name}")
