# SPDX-License-Identifier: MIT OR Apache-2.0
"""Independent binary records for original methanol, ethanol and amino-alcohol drawings."""

import argparse
import json
import math
from pathlib import Path
import runpy
import sys
from xml.etree import ElementTree as ET

HERE = Path(__file__).resolve().parent
XML = HERE.parent / "mixed-labels"
G = runpy.run_path(str(HERE.parent / "cdx-node-labels/generate_labels.py"))
sys.path.insert(0, str(HERE.parents[4] / "xtask/oracles"))
from clearance_oracle import authored, one_case, placed_box


def ethanol():
    root = ET.parse(XML / "methanol.cdxml").getroot()
    fragment = root.find(".//fragment")
    carbon = fragment.find("n")
    carbon.set("NumHydrogens", "2")
    carbon.set("ShowTerminalCarbonLabels", "yes")
    carbon.set("ShowNonTerminalCarbonLabels", "no")
    extra = ET.fromstring(ET.tostring(carbon))
    extra.attrib.update(
        id="61", p="56 144", NumHydrogens="3", ShowTerminalCarbonLabels="no"
    )
    fragment.insert(0, extra)
    first = ET.fromstring(ET.tostring(fragment.find("b")))
    first.attrib.update(id="899", B="61", E="73")
    fragment.insert(3, first)
    return ET.tostring(root, encoding="utf-8")


def records(xml, case):
    prop, obj, numeric = (G[key] for key in ("prop", "obj", "numeric"))
    fragment = ET.fromstring(xml).find(".//fragment")
    vertices = []
    for n in fragment.findall("n"):
        element, h = int(n.get("Element")), int(n.get("NumHydrogens"))
        xy = tuple(map(float, n.get("p").split()))
        flags = [
            (tag, b"")
            for name, tag in [
                ("ShowTerminalCarbonLabels", 0x0442),
                ("ShowNonTerminalCarbonLabels", 0x0443),
            ]
            if n.get(name) == "yes" and case in ("methanol", "amino_alcohol", "ethanol")
        ]
        child = n.find("t")
        if child is not None:
            bounds = tuple(map(float, child.get("BoundingBox").split()))
            text, curves = authored({8: "O", 7: "N"}[element], h, 12, xy)
            assert child.find("s").text == text and tuple(placed_box(curves)) == bounds
            if case == "overlap" and element == 7:
                dx, dy = -277, 5
                xy = (xy[0] + dx, xy[1] + dy)
                bounds = tuple(
                    coord + (dx, dy)[i % 2] for i, coord in enumerate(bounds)
                )
            if case == "bad_box":
                bounds = (bounds[0] - 3, *bounds[1:])
            if case == "literal_utf8":
                text = "Oη"
            if case == "missing_label_h":
                h = None
            children = [
                G["label"](
                    text,
                    [G["run"](size=240, colour=4)],
                    identity=int(child.get("id")),
                    bounds=bounds,
                    justify=0,
                    label_justify=0,
                    extended=True,
                )
            ]
        else:
            children = []
            if case != "literal_h":
                h = None
            if case == "active_flag":
                flags = [(0x0442, b"")]
            if case == "malformed_flag":
                flags = [(0x0442, b"\0")]
            if case == "hidden_oxygen":
                element = 8
            if case == "coincident":
                xy = (208, 144)
        vertices.append(
            G["node"](
                *children,
                identity=int(n.get("id")),
                pos=xy,
                element=None if element == 6 else element,
                h=h,
                node_type=1,
                add=[prop(t, b) for t, b in flags],
            )
        )
    bonds = [
        obj(
            0x8005,
            int(b.get("id")),
            numeric(0x0600, 1, "<H"),
            numeric(0x0604, int(b.get("B")), "<I"),
            numeric(0x0605, int(b.get("E")), "<I"),
        )
        for b in fragment.findall("b")
    ]
    fonts = [*G["ROBOTO"], (1, 65001, "Étude λ".encode())]

    def coord(tag, value):
        return numeric(tag, int(value * 65536), "<i")

    page = obj(
        0x8001,
        90,
        prop(0x0811, b"\x01"),
        coord(0x0812, 512),
        coord(0x0813, 512),
        obj(0x8003, 51, *vertices, *bonds),
    )
    return (
        G["HEADER"]
        + prop(0x0100, G["font_data"](fonts, 1))
        + prop(
            0x0300, G["palette_data"]([G["WHITE"], (0, 0, 0), (13107, 26214, 52428)])
        )
        + coord(0x0805, 48)
        + numeric(0x0804, 180, "<h")
        + coord(0x0807, 1.25)
        + coord(0x0808, 2)
        + page
        + bytes(2)
    )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", type=Path)
    out = parser.parse_args().output_dir
    if out:
        out.mkdir(parents=True, exist_ok=True)
    controls = {
        n: (XML / (n + ".cdxml")).read_bytes() for n in ("methanol", "amino_alcohol")
    }
    controls["ethanol"] = ethanol()
    cases = [
        "methanol",
        "ethanol",
        "amino_alcohol",
        "literal_h",
        "hidden_oxygen",
        "missing_label_h",
        "bad_box",
        "overlap",
        "coincident",
        "active_flag",
        "malformed_flag",
        "literal_utf8",
    ]
    files = {"ethanol.cdxml": controls["ethanol"]}
    for case in cases:
        control = controls.get(
            case, controls["amino_alcohol" if case == "overlap" else "methanol"]
        )
        data = records(control, case)
        G["RESOURCE"].read(data)
        files[case + ".cdx"] = data
    clearance = {}
    for name, control in controls.items():
        frag = ET.fromstring(control).find(".//fragment")
        nodes = {n.get("id"): n for n in frag.findall("n")}
        cuts = []
        for bond in frag.findall("b"):
            ends = [nodes[bond.get(key)] for key in ("B", "E")]
            ends_xy = [tuple(map(float, n.get("p").split())) for n in ends]
            bounds = []
            for i, (node, anchor) in enumerate(zip(ends, ends_xy, strict=True)):
                if node.find("t") is None:
                    bounds.append([0.0, 0.0])
                    continue
                element = int(node.get("Element"))
                x, y = ends_xy[1 - i]
                angle = math.degrees(math.atan2(y - anchor[1], x - anchor[0]))
                witness = one_case(
                    name,
                    {8: "O", 7: "N"}[element],
                    element,
                    int(node.get("NumHydrogens")),
                    12,
                    angle,
                    2,
                    [0],
                    width=1.25,
                    anchor=anchor,
                )
                bounds.append(witness["common_cut_pt_interval"])
            cuts.append(bounds)
        clearance[name] = cuts
    files["clearance.json"] = (json.dumps(clearance, indent=2) + "\n").encode()
    for name, data in files.items():
        file = (out or HERE) / name
        if out:
            file.write_bytes(data)
        else:
            assert file.read_bytes() == data, file
    print(f"{len(cases)} original CDX encodings, FontTools label boxes")


if __name__ == "__main__":
    main()
