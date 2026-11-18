# SPDX-License-Identifier: MIT OR Apache-2.0
"""Original binary CDX caption witnesses, Python struct only, no product output.

This is a proposed bounded profile, NOT application acceptance evidence.
Existing output bytes are checked before any overwrite; reproduce to distinct output.
"""

import argparse
import hashlib
import json
import struct
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = next(p for p in HERE.parents if (p / "Cargo.toml").is_file())
HEADER = b"VjCD0100\x04\x03\x02\x01" + bytes(16)
WHITE = (65535, 65535, 65535)
ROBOTO = [(0, 65001, b"Roboto")]


def prop(tag, data, extended=False):
    prefix = (
        struct.pack("<HI", 65535, len(data))
        if extended
        else struct.pack("<H", len(data))
    )
    return struct.pack("<H", tag) + prefix + data


def obj(tag, identity, *parts):
    return struct.pack("<HI", tag, identity) + b"".join(parts) + bytes(2)


def font_data(entries, platform):
    return struct.pack("<HH", platform, len(entries)) + b"".join(
        struct.pack("<HHH", identity, charset, len(name)) + name
        for identity, charset, name in entries
    )


def palette_data(entries):
    return struct.pack("<H", len(entries)) + b"".join(
        struct.pack("<HHH", *entry) for entry in entries
    )


def string_data(body, runs):
    return (
        struct.pack("<H", len(runs))
        + b"".join(struct.pack("<HHHHH", *entry) for entry in runs)
        + body
    )


def scene(
    *,
    framing="ImplicitDocument",
    reverse=False,
    extended=False,
    fonts=ROBOTO,
    platform=1,
    palette=None,
    body=b"Bondline",
    selector=0,
    size=240,
    face=0,
    ink=0,
    point=(80.25, 100.75),
    page_replace=None,
    text_replace=None,
    root_extra=(),
    page_extra=(),
    text_extra=(),
    text_children=(),
    root_children=(),
    move_font_to_text=False,
    text_wrapper=False,
    header_opaque=False,
):
    page_replace = page_replace or {}
    text_replace = text_replace or {}
    font = [] if fonts is None else [prop(0x0100, font_data(fonts, platform), extended)]
    tables = [] if move_font_to_text else font
    if palette is not None:
        tables.append(prop(0x0300, palette_data(palette), extended))
    page_fields = [
        (0x0811, b"\1"),
        (0x0812, struct.pack("<i", 512 * 65536)),
        (0x0813, struct.pack("<i", 512 * 65536)),
    ]
    x, y = point
    text_fields = [
        (0x0200, struct.pack("<ii", int(y * 65536), int(x * 65536))),
        (0x080C, b"\0"),
        (0x0700, string_data(body, [(0, selector, face, size, ink)])),
    ]

    def patched(fields, replacements):
        return [
            (tag, replacements.get(tag, data))
            for tag, data in fields
            if replacements.get(tag, data) is not None
        ]

    page_fields = patched(page_fields, page_replace)
    text_fields = patched(text_fields, text_replace)
    if reverse:
        page_fields.reverse()
        text_fields.reverse()
    text = obj(
        0x8006,
        73,
        *(prop(tag, data, extended) for tag, data in text_fields),
        *(font if move_font_to_text else []),
        *(prop(tag, data, extended) for tag, data in text_extra),
        *text_children,
    )
    if text_wrapper:
        text = obj(0x8002, 55, text)
    page = obj(
        0x8001,
        90,
        *(prop(tag, data, extended) for tag, data in page_fields),
        text,
        *(prop(tag, data, extended) for tag, data in page_extra),
    )
    extra = [prop(tag, data, extended) for tag, data in root_extra]
    contents = (
        page + b"".join(tables) + b"".join(extra)
        if reverse
        else b"".join(tables) + b"".join(extra) + page
    )
    contents += b"".join(root_children)
    header = HEADER if not header_opaque else HEADER[:12] + b"\1" + HEADER[13:]
    if framing == "ImplicitDocument":
        return header + contents + bytes(2)
    if framing == "ExplicitDocument":
        return header + obj(0x8000, 17, contents)
    if framing == "HeaderDocument":
        return header[:22] + obj(0x8000, 0, contents) + bytes(2)
    if framing == "ImplicitDocumentFileEnd":
        return header + contents + bytes(4)
    if framing == "ExplicitDocumentFileEnd":
        return header + obj(0x8000, 17, contents) + bytes(2)
    raise ValueError(framing)


def model(body="Bondline", *, size=12.0, rgb=(0.0, 0.0, 0.0), point=(80.25, 100.75)):
    return {
        "pages": [
            {
                "id": 1,
                "size": {"width_pt": 512.0, "height_pt": 512.0},
                "fragments": [],
                "texts": [
                    {
                        "id": 1,
                        "position": {"x_pt": point[0], "y_pt": point[1]},
                        "runs": [
                            {
                                "text": body,
                                "style": {
                                    "font_family": "Roboto",
                                    "size_pt": size,
                                    "face": 0,
                                    "rgb": list(rgb),
                                },
                            }
                        ],
                    }
                ],
            }
        ]
    }


def vectors():
    positive = {
        "caption_ascii": (scene(), model()),
        "caption_ascii_explicit": (
            scene(framing="ExplicitDocument", reverse=True),
            model(),
        ),
        "caption_ascii_header": (scene(framing="HeaderDocument"), model()),
        "caption_ascii_implicit_file_end": (
            scene(framing="ImplicitDocumentFileEnd"),
            model(),
        ),
        "caption_ascii_explicit_file_end": (
            scene(framing="ExplicitDocumentFileEnd"),
            model(),
        ),
        "caption_extended": (scene(extended=True), model()),
        "caption_fractional": (scene(body=b"AV2", size=241), model("AV2", size=12.05)),
        "caption_greek": (scene(body="Phase λ".encode("utf-8")), model("Phase λ")),
        "caption_short_white": (scene(palette=[WHITE]), model()),
        "caption_rgb16": (
            scene(palette=[WHITE, (16384, 32768, 49152)], ink=3),
            model(rgb=(16384 / 65535, 32768 / 65535, 49152 / 65535)),
        ),
        "caption_font_one": (
            scene(fonts=[(9, 0, b"Unused family"), (1, 65001, b"Roboto")], selector=1),
            model(),
        ),
        "caption_shifted": (scene(point=(104.25, 88.75)), model(point=(104.25, 88.75))),
    }
    # Each refusal is one decisive named control; no Cartesian expansion.
    negative = {
        "missing_font_table": scene(fonts=None),
        "font_ordinal_only": scene(fonts=[(7, 65001, b"Roboto")], selector=0),
        "font_id_only": scene(fonts=[(7, 65001, b"Roboto")], selector=7),
        "font_max_id_only": scene(fonts=[(65535, 65001, b"Roboto")], selector=65535),
        "font_unknown_charset": scene(fonts=[(0, 0, b"Roboto")]),
        "font_other_charset": scene(fonts=[(0, 1252, b"Roboto")]),
        "font_other_family": scene(fonts=[(0, 65001, b"Unused family")]),
        "font_nul_name": scene(fonts=[(0, 65001, b"Roboto\0")]),
        "font_nonutf8_name": scene(fonts=[(0, 65001, b"Roboto\xff")]),
        "font_platform0": scene(platform=0),
        "font_duplicate_id": scene(
            fonts=[(0, 65001, b"Roboto"), (0, 65001, b"Unused family")]
        ),
        "font_wrong_owner": scene(move_font_to_text=True),
        "palette_empty": scene(palette=[]),
        "palette_nonwhite2": scene(palette=[(65535, 0, 0)]),
        "palette_missing_ink3": scene(palette=[WHITE], ink=3),
        "missing_poster": scene(page_replace={0x0811: None}),
        "drawing_pages": scene(page_replace={0x0811: b"\0"}),
        "missing_width": scene(page_replace={0x0812: None}),
        "width_wrong_size": scene(page_replace={0x0812: struct.pack("<h", 512)}),
        "width_zero": scene(page_replace={0x0812: bytes(4)}),
        "width_negative": scene(page_replace={0x0812: struct.pack("<i", -65536)}),
        "width_wrong_owner": scene(
            page_replace={0x0812: None},
            text_extra=[(0x0812, struct.pack("<i", 512 * 65536))],
        ),
        "missing_position": scene(text_replace={0x0200: None}),
        "position_wrong_size": scene(text_replace={0x0200: bytes(4)}),
        "position_wrong_owner": scene(
            text_replace={0x0200: None},
            page_extra=[(0x0200, struct.pack("<ii", 100 * 65536, 80 * 65536))],
        ),
        "missing_justification": scene(text_replace={0x080C: None}),
        "justification_right": scene(text_replace={0x080C: b"\xff"}),
        "justification_auto": scene(text_replace={0x080C: b"\5"}),
        "justification_wrong_size": scene(text_replace={0x080C: bytes(2)}),
        "root_left_local_absent": scene(
            text_replace={0x080C: None}, root_extra=[(0x080C, b"\0")]
        ),
        "missing_text": scene(text_replace={0x0700: None}),
        "text_wrong_owner": scene(
            text_replace={0x0700: None},
            page_extra=[(0x0700, string_data(b"Bondline", [(0, 0, 0, 240, 0)]))],
        ),
        "missing_run_count": scene(text_replace={0x0700: b"\1"}),
        "truncated_run": scene(text_replace={0x0700: b"\1\0" + bytes(9)}),
        "zero_runs": scene(text_replace={0x0700: string_data(b"Bondline", [])}),
        "two_runs": scene(
            text_replace={
                0x0700: string_data(b"Bondline", [(0, 0, 0, 240, 0), (4, 0, 0, 240, 0)])
            }
        ),
        "run_start1": scene(
            text_replace={0x0700: string_data(b"Bondline", [(1, 0, 0, 240, 0)])}
        ),
        "face_bold": scene(face=1),
        "face_formula": scene(face=96),
        "size_zero": scene(size=0),
        "invalid_utf8": scene(body=b"Bondline\xff"),
        "incomplete_utf8": scene(body=b"A\xce"),
        "empty_text": scene(body=b""),
        "embedded_nul": scene(body=b"A\0B"),
        "line_feed": scene(body=b"A\nB"),
        "unicode_bom": scene(body="\ufeffBondline".encode("utf-8")),
        "unicode_emoji": scene(body="A😀B".encode("utf-8")),
        "unknown_root_property": scene(
            root_extra=[(0x4001, b"original caption opaque\0\xff")]
        ),
        "user_font_alias": scene(root_extra=[(0x4100, b"\2\xff\0")]),
        "unknown_text_property": scene(
            text_extra=[(0x7FFE, b"original caption text opaque\0\xff")]
        ),
        "unknown_root_object": scene(
            root_children=[obj(0xC123, 903, prop(0x4002, b"opaque\0\xfe"))]
        ),
        "text_under_group": scene(text_wrapper=True),
        "nested_text": scene(
            text_children=[
                obj(
                    0x8006,
                    74,
                    prop(0x0700, string_data(b"Hidden", [(0, 0, 0, 240, 0)])),
                )
            ]
        ),
        "utf8text_same": scene(
            text_extra=[(0x0709, string_data(b"Bondline", [(0, 0, 0, 240, 0)]))]
        ),
        "utf8text_conflict": scene(
            text_extra=[(0x0709, string_data(b"Different", [(0, 0, 0, 240, 0)]))]
        ),
        "shadowed_legacy_right": scene(text_extra=[(0x0701, b"\xff")]),
        "root_caption_style": scene(
            root_extra=[(0x080B, struct.pack("<HHHH", 0, 0, 240, 0))]
        ),
        "text_box": scene(
            text_extra=[
                (
                    0x0204,
                    struct.pack(
                        "<iiii", 90 * 65536, 80 * 65536, 104 * 65536, 130 * 65536
                    ),
                )
            ]
        ),
        "rotation_zero": scene(text_extra=[(0x0205, bytes(4))]),
        "page_background_black": scene(page_extra=[(0x0302, struct.pack("<h", 0))]),
        "duplicate_text": scene(
            text_extra=[(0x0700, string_data(b"Bondline", [(0, 0, 0, 240, 0)]))]
        ),
        "duplicate_position": scene(
            text_extra=[
                (0x0200, struct.pack("<ii", int(100.75 * 65536), int(80.25 * 65536)))
            ]
        ),
        "header_opaque": scene(header_opaque=True),
    }
    return positive, negative


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=REPO / "target/cdx-resources/next-text/reproduced-captions",
    )
    args = parser.parse_args()
    args.output_dir.mkdir(parents=True, exist_ok=True)
    positive, negative = vectors()
    manifest = {
        "scope": "Original proposed caption preview witnesses, not application evidence",
        "method": "Public SDK layout and Python stdlib struct only; no product imports/output",
        "positive_names": list(positive),
        "negative_names": list(negative),
        "cases": [],
    }
    for name, raw in {
        **{key: row[0] for key, row in positive.items()},
        **negative,
    }.items():
        path = args.output_dir / (name + ".cdx")
        if path.exists() and path.read_bytes() != raw:
            raise ValueError("refuse to change existing original source: " + name)
        path.write_bytes(raw)
        manifest["cases"].append(
            {
                "name": name,
                "file": path.name,
                "preview_candidate": name in positive,
                "byte_length": len(raw),
                "sha256": hashlib.sha256(raw).hexdigest(),
                "model": positive[name][1] if name in positive else None,
            }
        )
    path = args.output_dir / "caption-vectors.json"
    path.write_bytes(
        (json.dumps(manifest, ensure_ascii=False, indent=2) + "\n").encode("utf-8")
    )
    print(
        json.dumps(
            {
                "output": str(args.output_dir),
                "positive": len(positive),
                "negative": len(negative),
                "manifest_sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
            }
        )
    )


if __name__ == "__main__":
    main()
