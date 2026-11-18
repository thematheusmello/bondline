# SPDX-License-Identifier: MIT OR Apache-2.0
"""Original CDX font/palette source bytes; independent Python struct only."""

import argparse
import hashlib
import json
import struct
from pathlib import Path

SCRIPT = Path(__file__).resolve().parent
REPO = next(
    p
    for p in SCRIPT.parents
    if (p / "Cargo.toml").is_file() and (p / "crates/cdx").is_dir()
)
p = argparse.ArgumentParser(description=__doc__)
p.add_argument(
    "--output-dir", type=Path, default=REPO / "target/cdx-resources/reproduced"
)
args = p.parse_args()
OUT = args.output_dir
OUT.mkdir(parents=True, exist_ok=True)
HEADER = b"VjCD0100\x04\x03\x02\x01" + bytes(16)


def prop(tag, payload, extended=False):
    return (
        struct.pack("<H", tag)
        + (
            struct.pack("<HI", 65535, len(payload))
            if extended or len(payload) >= 65535
            else struct.pack("<H", len(payload))
        )
        + payload
    )


def obj(tag, identity, *parts):
    return struct.pack("<HI", tag, identity) + b"".join(parts) + bytes(2)


def fonts(entries, platform=1):
    return struct.pack("<HH", platform, len(entries)) + b"".join(
        struct.pack("<HHH", i, c, len(n)) + n for i, c, n in entries
    )


def colors(entries):
    return struct.pack("<H", len(entries)) + b"".join(
        struct.pack("<HHH", *rgb) for rgb in entries
    )


def graph(extra=None, owner=None):
    def fields(kind):
        return [extra] if owner == kind else []

    atom1 = obj(
        0x8004,
        7,
        *fields("node"),
        prop(0x0400, struct.pack("<h", 1)),
        prop(0x0402, struct.pack("<h", 6)),
        prop(0x0200, struct.pack("<ii", 96 * 65536, 72 * 65536)),
        prop(0x042B, struct.pack("<H", 3)),
    )
    atom2 = obj(
        0x8004,
        65535,
        prop(0x0400, struct.pack("<h", 1)),
        prop(0x0402, struct.pack("<h", 6)),
        prop(0x0200, struct.pack("<ii", 96 * 65536, 120 * 65536)),
        prop(0x042B, struct.pack("<H", 3)),
    )
    bond = obj(
        0x8005,
        900,
        prop(0x0604, struct.pack("<I", 7)),
        prop(0x0605, struct.pack("<I", 65535)),
        prop(0x0600, struct.pack("<H", 1)),
    )
    fragment = obj(
        0x8003,
        51,
        *fields("fragment"),
        atom1,
        atom2,
        bond,
        prop(0x4002, b"original resources fragment\0\xfc"),
    )
    custom = obj(
        0xC123,
        400000001,
        *fields("custom"),
        prop(0x4003, b"original resources opaque\0\xff"),
    )
    extras = []
    if owner == "group":
        extras.append(obj(0x8002, 91, extra))
    if owner == "text":
        extras.append(obj(0x8006, 92, extra))
    if owner == "nested_document":
        extras.append(obj(0x8000, 93, extra))
    return obj(0x8001, 90, *fields("page"), fragment, *extras, custom)


def scene(tables=(), framing="ExplicitDocument", wrong=None, owner=None):
    body = (
        b"".join(tables)
        + prop(0x4001, b"original resources root\0\xfe")
        + graph(wrong, owner)
    )
    if framing == "ImplicitDocument":
        return HEADER + body + bytes(2)
    if framing == "ExplicitDocument":
        return HEADER + obj(0x8000, 0, body)
    if framing == "HeaderDocument":
        return HEADER[:22] + obj(0x8000, 0, body) + bytes(2)
    if framing == "ImplicitDocumentFileEnd":
        return HEADER + body + bytes(4)
    if framing == "ExplicitDocumentFileEnd":
        return HEADER + obj(0x8000, 0, body) + bytes(2)
    raise ValueError("framing")


BASE_FONTS = [
    (0, 0, b""),
    (7, 32768, b"Bondline plain\0\xff"),
    (65535, 65535, b"\x80\xfeBondline wide"),
]
BASE_COLORS = [(12345, 23456, 34567), (40000, 50000, 60000), (0, 32768, 65535)]
ft = fonts(BASE_FONTS)
ct = colors(BASE_COLORS)
framing_cases = {
    "tables_implicit": "ImplicitDocument",
    "tables_explicit": "ExplicitDocument",
    "tables_header": "HeaderDocument",
    "tables_implicit_file_end": "ImplicitDocumentFileEnd",
    "tables_explicit_file_end": "ExplicitDocumentFileEnd",
}
sources = {
    name: scene([prop(0x0100, ft), prop(0x0300, ct)], framing)
    for name, framing in framing_cases.items()
}
sources.update(
    {
        "tables_reordered": scene(
            [
                prop(0x0300, ct),
                prop(0x0100, fonts([BASE_FONTS[2], BASE_FONTS[0], BASE_FONTS[1]])),
            ]
        ),
        "tables_extended": scene([prop(0x0100, ft, True), prop(0x0300, ct, True)]),
        "absent": scene(),
        "empty_tables": scene([prop(0x0100, fonts([])), prop(0x0300, colors([]))]),
        "font_empty_mac": scene([prop(0x0100, fonts([], 0))]),
        "palette_empty": scene([prop(0x0300, colors([]))]),
        "palette_one": scene([prop(0x0300, colors([(4369, 32768, 65535)]))]),
        "palette_nondefault": scene(
            [prop(0x0300, colors([(65535, 0, 32768), (32768, 65535, 1)]))]
        ),
        "max_name": scene(
            [
                prop(
                    0x0100,
                    fonts(
                        [(7, 65535, bytes((i * 53 + 29) & 255 for i in range(65535)))],
                        0,
                    ),
                )
            ]
        ),
        "user_tags_only": scene([prop(0x4100, b"\x02\xff\x00"), prop(0x4300, b"\xff")]),
    }
)
max_ids = [0, 65535] + [i for i in range(1, 65535) if i != 32768]
assert len(max_ids) == 65535
max_font_entries = [
    (ident, (0, 32768, 65535)[i % 3], b"") for i, ident in enumerate(max_ids)
]
max_color_entries = [
    ((i * 73 + 11) & 65535, (i * 257 + 32768) & 65535, 65535 - i) for i in range(65535)
]
sources["max_fonts"] = scene([prop(0x0100, fonts(max_font_entries))])
sources["max_colors"] = scene([prop(0x0300, colors(max_color_entries))])
negative = {
    "invalid_font_header": scene([prop(0x0100, b"\x01")]),
    "invalid_font_count": scene(
        [prop(0x0100, struct.pack("<HH", 1, 2) + struct.pack("<HHH", 7, 0, 1) + b"x")]
    ),
    "invalid_font_name": scene(
        [prop(0x0100, struct.pack("<HHHHH", 1, 1, 7, 0, 5) + b"abc")]
    ),
    "invalid_font_trailing": scene([prop(0x0100, ft + b"\x00")]),
    "invalid_font_max_count": scene([prop(0x0100, struct.pack("<HH", 1, 65535))]),
    "invalid_color_header": scene([prop(0x0300, b"\x01")]),
    "invalid_color_count": scene(
        [prop(0x0300, struct.pack("<H", 2) + struct.pack("<HHH", 1, 2, 3))]
    ),
    "invalid_color_trailing": scene([prop(0x0300, ct + b"\x00")]),
    "invalid_color_max_count": scene([prop(0x0300, struct.pack("<H", 65535))]),
    "unsupported_platform_2": scene([prop(0x0100, fonts([], 2))]),
    "unsupported_platform_65535": scene([prop(0x0100, fonts([], 65535))]),
    "duplicate_font_id": scene(
        [prop(0x0100, fonts([(7, 0, b"first"), (7, 65535, b"second")]))]
    ),
    "duplicate_font_late": scene(
        [prop(0x0100, fonts([*max_font_entries[:-1], (0, 65535, b"")]))]
    ),
    "duplicate_font_table": scene([prop(0x0100, ft), prop(0x0100, ft)]),
    "duplicate_color_table": scene([prop(0x0300, ct), prop(0x0300, ct)]),
    "duplicate_before_invalid_font": scene([prop(0x0100, b"\x01"), prop(0x0100, ft)]),
    "owner_before_invalid_font": scene(
        [prop(0x0100, b"\x01")], wrong=prop(0x0300, ct), owner="page"
    ),
}
for tag, payload, label in [(0x0100, ft, "font"), (0x0300, ct, "color")]:
    for owner in [
        "page",
        "group",
        "fragment",
        "node",
        "text",
        "custom",
        "nested_document",
    ]:
        negative[f"owner_{label}_{owner}"] = scene(
            wrong=prop(tag, payload), owner=owner
        )
all_sources = {**sources, **negative}
manifest = {
    "base": "dd57e2a",
    "contract": "391639c docs/research/cdx-resources.md",
    "method": "Original Python struct only; no product implementation/output",
    "cases": [],
}
for name, raw in all_sources.items():
    (OUT / (name + ".cdx")).write_bytes(raw)
    manifest["cases"].append(
        {
            "name": name,
            "query_valid": name in sources,
            "bytes": len(raw),
            "sha256": hashlib.sha256(raw).hexdigest(),
        }
    )
(OUT / "resource-vectors.json").write_bytes(
    (json.dumps(manifest, indent=2) + "\n").encode()
)
print(
    json.dumps(
        {
            "sources": len(all_sources),
            "positive": len(sources),
            "query_negative": len(negative),
            "manifest_sha256": hashlib.sha256(
                (OUT / "resource-vectors.json").read_bytes()
            ).hexdigest(),
        }
    )
)
