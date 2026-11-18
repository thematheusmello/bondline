# SPDX-License-Identifier: MIT OR Apache-2.0
"""Generate original CDX label preview vectors from independent FontTools metrics."""

import argparse
from hashlib import sha256
import json
from pathlib import Path
import struct
import subprocess
import sys

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[4]
sys.path[:0] = [str(HERE.parent / "cdx-node-labels"), str(REPO / "xtask/oracles")]
from generate_labels import HEADER, RESOURCE, ROBOTO, WHITE, atom, font_data, label, node
from generate_labels import obj, owned, palette_data, prop, run, style
from clearance_oracle import FONT_SHA, authored, placed_box

SYMBOLS = {1: "H", 6: "C", 7: "N", 8: "O", 17: "Cl"}
PALETTE = (WHITE, (0, 0, 0), (65535, 0, 0), (0, 0, 65535), (0, 40092, 0))
POSTER = [(0x0811, b"\1"), (0x0812, struct.pack("<i", 512 * 65536)),
          (0x0813, struct.pack("<i", 512 * 65536))]


def coordinate(value):
    return struct.pack("<i", round(value * 65536))


def quantize(value):
    return round(value * 65536) / 65536


def packed_fields(fields, reverse):
    fields = list(fields)
    if reverse:
        fields.reverse()
    return [prop(tag, raw, extended=reverse and i % 2 == 0)
            for i, (tag, raw) in enumerate(fields)]


def scene(fragments, *, framing="implicit", reverse=False, width=None, margin=None,
          length=None, spacing=None, angle=None, doubles=256):
    page = {"id": 1, "size": {"width_pt": 512.0, "height_pt": 512.0}, "fragments": []}
    model = {"pages": [page]}
    ink = []
    raw_fragments = []
    next_atom, next_bond = 0, 0
    for fragment_id, (vertices, bonds) in enumerate(fragments, 1):
        raw_atoms, raw_bonds = [], []
        atoms, edges = [], []
        source_to_model = {}
        for source_id, element, hydrogens, position, color in vertices:
            next_atom += 1
            source_to_model[source_id] = next_atom
            em = 24.0
            body, glyphs = authored(SYMBOLS[element], hydrogens, em, position)
            bounds = placed_box(glyphs)
            rounded = list(map(quantize, bounds))
            shift = [rounded[0] - bounds[0], rounded[1] - bounds[1]]
            for glyph in glyphs:
                glyph["origin_pt"] = [a + b for a, b in zip(glyph["origin_pt"], shift)]
            rgb = PALETTE[color - 2]
            ink.append({"atom_id": next_atom, "fragment_id": fragment_id,
                        "rgb8": [round(channel / 257) for channel in rgb],
                        "glyphs": glyphs, "bounds_pt": placed_box(glyphs)})
            raw_atoms.append(node(
                label(body, (run(size=480, colour=color),), identity=73, bounds=rounded,
                      justify=0, label_justify=0, reverse=reverse, extended=reverse),
                identity=source_id, pos=position, element=element, h=hydrogens, node_type=1,
            ))
            atoms.append(atom(next_atom, owned(style(body, size=em, colour=rgb),
                                               bounds=rounded, justify="left", label_justify="left"),
                              element=element, pos=position, h=hydrogens))
        for source_id, begin, end, order in bonds:
            next_bond += 1
            fields = [(0x0600, struct.pack("<H", order)),
                      (0x0604, struct.pack("<I", begin)), (0x0605, struct.pack("<I", end))]
            if order == 2:
                fields += [(0x0603, struct.pack("<h", doubles)), (0x0601, b"\0\0"),
                           (0x0602, b"\0\0")]
            raw_bonds.append(obj(0x8005, source_id, *packed_fields(fields, reverse)))
            edges.append({"id": next_bond, "begin": source_to_model[begin],
                          "end": source_to_model[end], "order": {1: "single", 2: "double", 4: "triple"}[order],
                          **({"double_position": "center"} if order == 2 else {})})
        raw_fragments.append(obj(0x8003, 900 + fragment_id, *raw_atoms, *raw_bonds))
        page["fragments"].append({"id": fragment_id, "atoms": atoms, "bonds": edges})

    resources = [(0x0100, font_data(ROBOTO, 1)), (0x0300, palette_data(PALETTE))]
    authored_style = {}
    for tag, value, key in [(0x0807, width, "line_width_pt"), (0x0808, margin, "margin_width_pt")]:
        if value is not None:
            resources.append((tag, coordinate(value)))
            authored_style[key] = float(value)
    if spacing is not None:
        resources.append((0x0804, struct.pack("<h", spacing)))
        authored_style["bond_spacing_percent"] = spacing / 10
    if authored_style:
        model["drawing_style"] = authored_style
    settings = {}
    for tag, value, key in [(0x0805, length, "bond_length_pt"), (0x0803, angle, "chain_angle_deg")]:
        if value is not None:
            resources.append((tag, coordinate(value)))
            settings[key] = float(value)
    if settings:
        model["drawing_settings"] = settings

    prefix = b"".join(packed_fields(resources, reverse))
    page_bytes = obj(0x8001, 51, *packed_fields(POSTER, reverse), *raw_fragments)
    body = page_bytes + prefix if reverse else prefix + page_bytes
    envelope = {
        "implicit": HEADER + body + bytes(2),
        "implicit-end": HEADER + body + bytes(4),
        "explicit": HEADER + obj(0x8000, 411, body),
        "explicit-end": HEADER + obj(0x8000, 411, body) + bytes(2),
        "header": HEADER[:22] + obj(0x8000, 0, body) + bytes(2),
    }
    return envelope[framing], model, ink


def change(raw, owner, tag, replacement):
    records = RESOURCE.read(raw)["records"]
    rec = next(r for r in records if r["kind"] == "property" and r["tag"] == tag
               and ((r["parent"] is None and owner == 0x8000)
                    or r["parent"] is not None and records[r["parent"]]["tag"] == owner))
    start, end = rec["range"]
    old = raw[start:end]
    return raw[:start] + (replacement(old) if callable(replacement) else replacement) + raw[end:]


def vectors():
    boxed = [([(81, 7, 2, (147.375, 123.125), 4)], []),
             ([(81, 17, 0, (350.5, 277.75), 5)], [])]
    water = [([(91, 1, 0, (191.5, 270.25), 5), (7, 8, 0, (256, 218.25), 4),
               (81, 1, 0, (321.75, 270), 6)], [(62, 91, 7, 1), (52, 7, 81, 1)])]
    dioxide = [([(90, 8, 0, (145.5, 248.25), 5), (27, 6, 0, (256, 248.25), 4),
                 (11, 8, 0, (366.75, 248.25), 6)], [(53, 27, 90, 2), (54, 27, 11, 2)])]
    cyanide = [([(91, 1, 0, (141, 309.5), 6), (7, 6, 0, (251, 309.5), 4),
                 (81, 7, 0, (371, 309.5), 5)], [(62, 91, 7, 1), (52, 7, 81, 4)])]
    multi = {"width": 1.5, "margin": 2, "length": 48, "spacing": 125}
    yes = {
        "boxed": scene(boxed),
        "boxed_reordered": scene(boxed, framing="explicit-end", reverse=True),
        "water": scene(water, framing="implicit-end", width=1.5, margin=1.75),
        "dioxide_manual": scene(dioxide, framing="explicit", **multi),
        "dioxide_automatic": scene(dioxide, framing="explicit-end", doubles=0, **multi),
        "cyanide": scene(cyanide, framing="header", angle=120, **multi),
    }
    box, water_raw, co2 = (yes[name][0] for name in ("boxed", "water", "dioxide_manual"))
    no = {"header_reserved_byte": box[:12] + b"\1" + box[13:]}
    for name, source, owner, tag, replacement in [
        ("missing_width", box, 0x8001, 0x0812, b""),
        ("width_at_root", box, 0x8001, 0x0812, b""),
        ("short_height", box, 0x8001, 0x0813, prop(0x0813, b"\0\1")),
        ("negative_height", box, 0x8001, 0x0813, prop(0x0813, coordinate(-512))),
        ("pages_space", box, 0x8001, 0x0811, prop(0x0811, b"\0")),
        ("duplicate_width", box, 0x8001, 0x0812, lambda old: old * 2),
        ("nonwhite_paper", box, 0x8000, 0x0300, prop(0x0300, palette_data(((65534, 65535, 65535), *PALETTE[1:])))),
        ("blue_default_bonds", water_raw, 0x8000, 0x0300, prop(0x0300, palette_data((WHITE, PALETTE[3], *PALETTE[2:])))),
        ("absent_label_box", box, 0x8006, 0x0204, b""),
        ("contradictory_ink_box", box, 0x8006, 0x0204, lambda old: old[:16] + coordinate(211)),
        ("explicit_child_point", box, 0x8006, 0x0204, lambda old: old + prop(0x0200, coordinate(123.125) + coordinate(147.375))),
        ("label_right", box, 0x8006, 0x0823, prop(0x0823, b"\xff")),
        ("explicit_alignment", box, 0x8006, 0x0204, lambda old: old + prop(0x0705, b"\1")),
        ("label_numeric_disagreement", box, 0x8004, 0x0402, prop(0x0402, struct.pack("<h", 8))),
        ("unknown_h", box, 0x8004, 0x042B, b""),
        ("unknown_text_face", box, 0x8006, 0x0700, lambda old: old[:10] + b"\1\0" + old[12:]),
        ("opaque_label_property", box, 0x8006, 0x0204, lambda old: old + prop(0x4001, b"extra")),
        ("opaque_root_property", box, 0x8000, 0x0100, lambda old: old + prop(0x4001, b"extra")),
        ("unsupported_label_color", box, 0x8000, 0x0300, prop(0x0300, palette_data((WHITE,)))),
        ("charged_atom", box, 0x8004, 0x042B, lambda old: old + prop(0x0421, b"\1")),
        ("isotopic_atom", box, 0x8004, 0x042B, lambda old: old + prop(0x0420, b"\x0f\0")),
        ("raw_label_z", box, 0x8006, 0x0204, lambda old: old + prop(0x000A, b"\1\0")),
        ("missing_width_style", water_raw, 0x8000, 0x0807, b""),
        ("missing_margin", water_raw, 0x8000, 0x0808, b""),
        ("negative_margin", water_raw, 0x8000, 0x0808, prop(0x0808, coordinate(-1))),
        ("wrong_width_style", water_raw, 0x8000, 0x0807, prop(0x0807, b"\1")),
        ("duplicate_width_style", water_raw, 0x8000, 0x0807, lambda old: old * 2),
        ("local_margin", water_raw, 0x8005, 0x0600, lambda old: old + prop(0x0808, coordinate(3))),
        ("no_preferred_length", co2, 0x8000, 0x0805, b""),
        ("no_relative_spacing", co2, 0x8000, 0x0804, b""),
        ("zero_relative_spacing", co2, 0x8000, 0x0804, prop(0x0804, b"\0\0")),
        ("relative_spacing_width", co2, 0x8000, 0x0804, prop(0x0804, b"\x7d")),
        ("unspecified_double", co2, 0x8005, 0x0603, b""),
        ("sided_double", co2, 0x8005, 0x0603, prop(0x0603, struct.pack("<h", 257))),
    ]:
        no[name] = change(source, owner, tag, replacement)
    without_page_width = no["width_at_root"]
    no["width_at_root"] = (without_page_width[:len(HEADER)] + prop(0x0812, coordinate(512))
                           + without_page_width[len(HEADER):])
    no["other_text_owner"] = change(box, 0x8001, 0x0812, lambda old:
        old + obj(0x8006, 27, prop(0x0700, b"\0\0text")))
    no["group_peer"] = change(box, 0x8001, 0x0812,
                             lambda old: old + obj(0x8002, 22, obj(0x8003, 71)))
    offpage = [([(8, 6, 2, (505, 480), 5)], [])]
    no["off_paper"] = scene(boxed + offpage)[0]
    crossing = [([(1, 1, 0, (90, 88), 4), (2, 1, 0, (421, 420), 5),
                  (3, 1, 0, (89, 422), 6), (4, 1, 0, (423, 87), 5)],
                 [(41, 1, 2, 1), (51, 3, 4, 1)])]
    no["crossing_axes"] = scene(crossing, width=1.5, margin=2)[0]
    return yes, no


def source_facts(raw):
    tree = RESOURCE.read(raw)
    nodes = [[r["tag"], r.get("id"), r["parent"]]
             for r in tree["records"] if r["kind"] in ("start", "property")]
    return {"framing": tree["framing"], "tree": nodes}


def open_baseline(cli, path):
    command = {"command": "document.open_cdx", "params": {"path": str(path.resolve())}}
    out = subprocess.run([str(cli.resolve()), "run", "--cmd", json.dumps(command)],
                         capture_output=True, check=False, timeout=15)
    if out.returncode == 0:
        return {"model": json.loads(out.stdout)["document"]}
    assert out.returncode == 2, (path, out.returncode, out.stderr)
    error = json.loads(out.stderr)["error"]
    return {"code": error["code"], "detail": error["message"].split(" during Decode for ", 1)[0]}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", required=True, type=Path)
    parser.add_argument("--baseline-cli", required=True, type=Path)
    args = parser.parse_args()
    args.output_dir.mkdir(parents=True, exist_ok=False)
    yes, no = vectors()
    manifest = {"font_sha256": FONT_SHA, "renderable": {}, "retained": {}}
    for name in sorted(yes | no):
        visible = name in yes
        raw = yes[name][0] if visible else no[name]
        file = args.output_dir / (name + ".cdx")
        file.write_bytes(raw)
        case = {"sha256": sha256(raw).hexdigest(), "source": source_facts(raw)}
        baseline = open_baseline(args.baseline_cli, file)
        if visible:
            _, model, ink = yes[name]
            assert "model" in baseline, (name, baseline)
            assert baseline["model"]["pages"][0]["fragments"] == model["pages"][0]["fragments"]
            manifest["renderable"][name] = {**case, "model": model, "ink": ink}
        else:
            manifest["retained"][name] = {**case, "legacy": baseline}
    file = args.output_dir / "cases.json"
    file.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n",
                    encoding="utf-8")
    print(f"{len(yes)} renderable and {len(no)} control fixtures")


if __name__ == "__main__":
    main()
