# SPDX-License-Identifier: MIT OR Apache-2.0
"""Original CDX child Text witnesses. No product writer or proprietary inputs."""

import argparse
from hashlib import sha256
import json
from pathlib import Path
import struct
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "cdx-caption"))
from caption_oracle import RESOURCE
from generate_captions import HEADER, ROBOTO, WHITE, font_data, obj, palette_data, prop, string_data
from generate_ordered import BLUE, legacy_open


def numeric(tag, value, encoding):
    return prop(tag, struct.pack(encoding, value))


def scaled(value):
    return int(value * 65536)


def point(position):
    x, y = position
    return prop(0x0200, struct.pack("<ii", scaled(y), scaled(x)))


def run(start=0, *, size=240, font=0, face=0, colour=0):
    return (start, font, face, size, colour)


def label(
    body=b"C", runs=(run(),), *, identity=301, position=None, bounds=None,
    justify=None, label_justify=None, alignment=None, order=None,
    add=(), reverse=False, extended=False,
):
    fields = list(add)
    if isinstance(body, str):
        body = body.encode()
    if body is not None:
        fields.append(prop(0x0700, string_data(body, runs), extended))
    if position is not None:
        fields.append(point(position))
    if bounds is not None:
        l, t, r, b = bounds
        fields.append(prop(0x0204, struct.pack("<4i", *map(scaled, [t, l, b, r]))))
    for tag, value, fmt in ((0x0701, justify, "<b"), (0x0823, label_justify, "<b"),
                            (0x0705, alignment, "<b"), (0x000A, order, "<h")):
        if value is not None:
            fields.append(numeric(tag, value, fmt))
    if reverse:
        fields.reverse()
    return obj(0x8006, identity, *fields)


def node(*children, identity=73, pos=(96.25, 160.5), element=6, h=None,
         isotope=None, charge=None, radical=None, node_type=None, add=()):
    fields = [] if pos is None else [point(pos)]
    for tag, value, fmt in (
        (0x0400, node_type, "<h"), (0x0402, element, "<h"),
        (0x0420, isotope, "<h"), (0x0421, charge, "<b"),
        (0x0422, radical, "<B"), (0x042B, h, "<H"),
    ):
        if value is not None:
            fields.append(numeric(tag, value, fmt))
    return obj(0x8004, identity, *fields, *add, *children)


def document(*children, fonts=ROBOTO, palette=None, platform=1,
             root_extra=(), page_extra=(), framing="implicit"):
    fragment = obj(0x8003, 51, *children)
    page = obj(0x8001, 90, fragment, *page_extra)
    tables = []
    if fonts is not None:
        tables.append(prop(0x0100, font_data(fonts, platform)))
    if palette is not None:
        tables.append(prop(0x0300, palette_data(palette)))
    body = b"".join([*tables, *root_extra, page])
    if framing == "implicit":
        return HEADER + body + bytes(2)
    if framing == "implicit-end":
        return HEADER + body + bytes(4)
    if framing == "explicit":
        return HEADER + obj(0x8000, 811, body)
    if framing == "explicit-end":
        return HEADER + obj(0x8000, 811, body) + bytes(2)
    if framing == "header":
        return HEADER[:22] + obj(0x8000, 0, body) + bytes(2)
    raise ValueError(framing)


def xy(position):
    return None if position is None else dict(zip(("x_pt", "y_pt"), map(float, position)))


def style(text, *, family="Roboto", size=12, colour=(0, 0, 0), face=0):
    return {"text": text, "style": {"font_family": family, "size_pt": float(size),
            "face": face, "rgb": [n / 65535 for n in colour]}}


def owned(*styles, position=None, bounds=None, justify=None, label_justify=None, alignment=None):
    item = {"position": xy(position), "justification": justify,
            "label_justification": label_justify, "label_alignment": alignment,
            "runs": list(styles)}
    if bounds is not None:
        item["bounding_box"] = list(map(float, bounds))
    return item


def atom(identity, authored, *, element=6, pos=(96.25, 160.5), h=None,
         isotope=None, charge=0, radical="none"):
    values = {"id": identity, "element": element, "position": xy(pos), "isotope": isotope,
              "charge": charge, "radical": radical, "explicit_hydrogens": h}
    if authored is not None:
        values["authored_label"] = authored
    return values


def model(*atoms, bonds=()):
    return {"pages": [{"id": 1, "size": {"width_pt": 595.2755905511812,
            "height_pt": 841.8897637795276},
            "fragments": [{"id": 1, "atoms": list(atoms), "bonds": list(bonds)}]}]}


def witnesses():
    bounds = (-7.25, -3.125, 15.5, 12.75)
    label_position = (-1.5, 3.125)
    opaque = prop(0x4102, b"original label\0\xff")
    settings = {"position": label_position, "bounds": bounds, "justify": -1,
                "label_justify": 0, "alignment": 4, "order": -17, "add": (opaque,)}
    expected = owned(style("C"), position=label_position, bounds=bounds,
                     justify="right", label_justify="left", alignment="above")
    conflict = model(atom(1, expected, element=7, isotope=15, charge=1, h=3, radical="doublet"))
    conflict_node = lambda **kwargs: node(label(**settings, **kwargs),
                                               element=7, isotope=15, charge=1, h=3, radical=2)
    one = node(label())
    second_pos = (144.25, 160.5)
    double = obj(0x8005, 900, numeric(0x0604, 149, "<I"), numeric(0x0605, 73, "<I"),
                 numeric(0x0600, 2, "<H"), numeric(0x0603, 257, "<h"))
    dual_label = owned(style("CH"), style("2", face=32))
    dual = model(atom(1, dual_label, h=2), atom(2, dual_label, h=2, pos=second_pos),
                 bonds=[{"id": 1, "begin": 2, "end": 1, "order": "double",
                         "double_position": "right"}])
    enum_names = ("right", "left", "center", "full", "above", "below", "auto", "best")
    align_names = ("auto", "left", "center", "right", "above", "below", "best")
    enum_nodes = []
    enum_model = []
    for i, name in enumerate(enum_names):
        align = i % len(align_names)
        pos = (96.25 + i * 30, 250.5)
        enum_nodes.append(node(label(b"N", justify=i - 1, label_justify=6 - i, alignment=align),
                               identity=100 + i, pos=pos, element=7))
        item = owned(style("N"), justify=name, label_justify=enum_names[7-i],
                     alignment=align_names[align])
        enum_model.append(atom(i + 1, item, pos=pos, element=7))
    return {
        "literal_conflict": (document(conflict_node()), conflict),
        "reordered_conflict": (
            document(conflict_node(reverse=True, extended=True), framing="explicit-end"), conflict,
        ),
        "explicit_empty": (
            document(node(label(b"", identity=0), identity=0, element=None, pos=None),
                     root_extra=[numeric(0x0823, 0, "<b")], framing="header"),
            model(atom(1, owned(style("")), pos=None)),
        ),
        "connected_pair": (
            document(node(label(b"CH2", [run(), run(2, face=32)]), h=2),
                     node(label(b"CH2", [run(), run(2, face=32)]),
                          identity=149, pos=second_pos, h=2), double, framing="explicit"),
            dual,
        ),
        "font_face_tail": (
            document(node(label("Aλ漢", [run(), run(1, font=1, size=255, face=66, colour=3)])),
                     fonts=[*ROBOTO, (1, 65001, b"Fixture Serif")],
                     palette=[WHITE, BLUE], framing="implicit-end"),
            model(atom(1, owned(style("A"),
                       style("λ漢", family="Fixture Serif", size=12.75, face=66, colour=BLUE)))),
        ),
        "unresolved_join": (
            document(node(label(b"AV", [run(), run(1)]))),
            model(atom(1, owned(style("A"), style("V")))),
        ),
        "separate_enum_fields": (document(*enum_nodes), model(*enum_model)),
        "missing_text": (document(node(label(None))), None),
        "duplicate_text": (
            document(node(label(), label(b"R", identity=302))), None,
        ),
        "duplicate_value": (
            document(node(label(add=[prop(0x0700, string_data(b"N", [run()]))]))), None,
        ),
        "fragment_text": (document(node(), label()), None),
        "page_peer": (document(one, page_extra=[label()]), None),
        "child_property_on_atom": (document(node(label(), add=[numeric(0x0701, 0, "<b")])), None),
        "non_element_node": (document(node(label(), node_type=4)), None),
        "wrong_position_width": (document(node(label(add=[prop(0x0200, bytes(4))]))), None),
        "wrong_box_width": (document(node(label(add=[prop(0x0204, bytes(15))]))), None),
        "inverted_box": (document(node(label(bounds=(17, 18, 14, 19)))), None),
        "invalid_alignment": (document(node(label(alignment=-1))), None),
        "invalid_justification": (document(node(label(label_justify=7))), None),
        "wrong_justify_width": (document(node(label(add=[numeric(0x0823, 0, "<h")]))), None),
        "unsupported_caption_field": (document(node(label(add=[numeric(0x080C, 0, "<b")]))), None),
        "missing_fonts": (document(one, fonts=None), None),
        "misowned_font_table": (
            document(node(label(add=[prop(0x0100, font_data(ROBOTO, 1))])), fonts=None),
            None,
        ),
        "ordinal_conflict": (
            document(node(label(b"C", [run(font=7)])), fonts=[*ROBOTO, (7, 65001, b"Second")]),
            None,
        ),
        "duplicate_font_id": (
            document(one, fonts=[*ROBOTO, (0, 65001, b"Second")]), None,
        ),
        "wrong_platform": (document(one, platform=0), None),
        "wrong_charset": (document(one, fonts=[(0, 1252, b"Roboto")]), None),
        "undecided_font_name": (document(one, fonts=[(0, 65001, "Typé".encode())]), None),
        "missing_colour": (document(node(label(b"L", [run(colour=4)])), palette=[WHITE]), None),
        "zero_styles": (document(node(label(b"L", []))), None),
        "ambiguous_start": (document(node(label("AéHN", [run(), run(3)]))), None),
        "bad_peer": (document(one, node(label(alignment=7), identity=149, pos=second_pos)), None),
        "unsupported_child": (document(node(label(add=[obj(0x8011, 9)]))), None),
        "too_many_styles": (
            document(node(label(b"C" * 4097, [run(i) for i in range(4097)]))), None,
        ),
    }


def source_facts(source):
    """Verify original bytes using the independent struct resource and record reader."""
    raw = RESOURCE.read(source)
    records = raw["records"]
    owners = {}
    for i, rec in enumerate(records):
        if rec["kind"] != "start" or rec["tag"] not in (0x8004, 0x8005, 0x8006):
            continue
        parent = rec["parent"]
        owners[i] = {"tag": rec["tag"], "id": rec["id"], "parent": parent,
                     "parent_id": records[parent]["id"], "properties": []}
    for rec in records:
        if rec["kind"] == "property" and rec["parent"] in owners:
            start, end = rec["payload_range"]
            payload = source[start:end]
            identity = payload.hex() if len(payload) <= 256 else "sha256:" + sha256(payload).hexdigest()
            owners[rec["parent"]]["properties"].append([rec["tag"], start, end, identity])
    return {"framing": raw["framing"], "owner_records": owners, "resources": raw["query"]}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", required=True, type=Path)
    parser.add_argument("--baseline-cli", required=True, type=Path)
    args = parser.parse_args()
    args.output_dir.mkdir(parents=True, exist_ok=False)
    cases = {}
    for name, (data, expected) in witnesses().items():
        path = args.output_dir / (name + ".cdx")
        path.write_bytes(data)
        cases[name] = {
            "sha256": sha256(data).hexdigest(), "model": expected, "source": source_facts(data),
            "legacy": legacy_open(args.baseline_cli, path.resolve()),
        }
        assert path.read_bytes() == data
    (args.output_dir / "cases.json").write_text(
        json.dumps(cases, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
