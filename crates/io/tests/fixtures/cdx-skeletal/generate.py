# SPDX-License-Identifier: MIT OR Apache-2.0
"""Original CDX skeletal chain, one-field controls and binary/optical expectations."""
import argparse
from pathlib import Path
import runpy
import struct

HERE = Path(__file__).resolve().parent
helpers = runpy.run_path(str(HERE.parent / "cdx-node-labels/generate_labels.py"))
obj, prop, node, label = [helpers[k] for k in ("obj", "prop", "node", "label")]
TERM, INTERIOR = 0x0442, 0x0443
ids = (81, 7, 91)
positions = [(160,240), (224,192), (288,240)]
hydrogens = (3, 2, 3)


def coordinate(n):
    return struct.pack("<i", n * 65536)


def raw(test):
    points = list(positions)
    if test == "collinear":
        points[1] = (224,240)
    if test == "off_paper":
        points[0] = (-16,240)
    vertices = []
    for i, (identity, xy, h) in enumerate(zip(ids, points, hydrogens)):
        fields, children = [], []
        element = 6 if i == 1 else None  # two published missing-Element carbon defaults
        isotope, charge, radical = None, None, None
        if test == "terminal_visible" and i == 0:
            fields.append(prop(TERM, b""))
        if test == "interior_visible" and i == 1:
            fields.append(prop(INTERIOR, b""))
        if test == "dormant_true" and i == 1:
            fields.append(prop(TERM, b""))
        if test == "dormant_nonterminal" and i == 0:
            fields.append(prop(INTERIOR, b""))
        if test == "malformed_dormant" and i == 1:
            fields.append(prop(TERM, bytes(1)))
        if test == "duplicate_dormant" and i == 1:
            fields += [prop(TERM, b"")] * 2
        if test == "malformed_false" and i == 1:
            fields.append(prop(INTERIOR, bytes(1)))
        if test == "duplicate_flag" and i == 1:
            fields += [prop(INTERIOR, b"")] * 2
        if test == "all_h_missing" or (test == "missing_h" and i == 1):
            h = None
        if test == "inconsistent_h" and i == 1:
            h = 3
        if test == "heteroatom" and i == 1:
            element, h = 8, 0
        if test == "charge" and i == 0:
            charge = 1
        if test == "isotope" and i == 0:
            isotope = 13
        if test == "radical" and i == 0:
            radical = 2
        if test == "custom_node" and i == 1:
            fields.append(prop(0x4001, b"Opaque"))
        if test == "child_text" and i == 1:
            children.append(label("X", identity=300, runs=(helpers["run"](size=240),)))
        vertices.append(node(*children, identity=identity, pos=xy, element=element, h=h,
                             node_type=1, charge=charge, isotope=isotope, radical=radical, add=fields))
    pairs = [(0,1),(1,2)]
    if test == "ethane":
        vertices = vertices[:2]
        vertices[1] = node(identity=ids[1], pos=points[1], element=6, h=3, node_type=1)
        pairs = [(0,1)]
    bonds = [obj(0x8005, 52 + k, prop(0x0600, struct.pack("<H",1)),
                 prop(0x0604, struct.pack("<I",ids[b])), prop(0x0605,struct.pack("<I",ids[e])))
             for k, (b,e) in enumerate(pairs)]
    fragment = obj(0x8003, 901, *(vertices + bonds))
    page = obj(0x8001, 51, prop(0x0811,b"\x01"), prop(0x0812, coordinate(512)),
               prop(0x0813,coordinate(512)), fragment)
    root = [prop(0x0100, helpers["font_data"](helpers["ROBOTO"], 1)),
            prop(0x0300,helpers["palette_data"]((helpers["WHITE"],(0,0,0)))),
            prop(0x0808, coordinate(2))]
    if test == "custom_root":
        root.append(prop(0x4001,b"Opaque"))
    if test != "missing_width":
        root.append(prop(0x0807, coordinate(4)))
    if test == "root_flag":
        root.append(prop(INTERIOR,b""))
    return helpers["HEADER"] + b"".join(root) + page + bytes(2)


cases = ["propane", "dormant_true", "terminal_visible", "interior_visible", "malformed_false",
         "duplicate_flag", "root_flag", "missing_h", "all_h_missing", "inconsistent_h", "heteroatom",
         "charge", "child_text", "collinear", "ethane", "off_paper", "missing_width", "dormant_nonterminal", "malformed_dormant",
         "duplicate_dormant", "custom_root", "custom_node", "isotope", "radical"]


def control():
    # Actual XML flags are explicit. The binary twins represent false by absence.
    atoms = "".join(f'<n id="{i+3}" NodeType="Element" Element="6" Charge="0" Radical="None" '
                    f'p="{x} {y}" NumHydrogens="{h}" ShowTerminalCarbonLabels="no" '
                    f'ShowNonTerminalCarbonLabels="no"/>' for i, ((x, y), h) in
                    enumerate(zip(positions, hydrogens, strict=True)))
    return ('<?xml version="1.0" encoding="UTF-8"?><CDXML CreationProgram="Bondline 0.1.0" '
            'LineWidth="4" MarginWidth="2"><page id="1" DrawingSpace="Poster" Width="512" '
            'Height="512"><fragment id="2">' + atoms + '<b id="6" B="3" E="4" Order="1"/>'
            '<b id="7" B="4" E="5" Order="1"/></fragment></page></CDXML>').encode()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", type=Path)
    output = parser.parse_args().output_dir
    files = {name + ".cdx": raw(name) for name in cases}
    files["optical-control.cdxml"] = control()
    if output:
        output.mkdir(parents=True, exist_ok=True)
    for name, data in files.items():
        if name.endswith(".cdx"):
            # Independent reader checks complete framing, matched record bounds and source bytes.
            helpers["RESOURCE"].read(data)
        file = (output or HERE) / name
        if output:
            file.write_bytes(data)
        else:
            assert file.read_bytes() == data, file
    print(f"{len(files)-1} binary witnesses and one CDXML optical control")


if __name__ == "__main__":
    main()
