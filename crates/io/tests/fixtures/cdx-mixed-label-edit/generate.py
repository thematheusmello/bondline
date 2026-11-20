# SPDX-License-Identifier: MIT OR Apache-2.0
"""Replay existing FontTools geometry and owner-scoped CDX record oracles against mixed paper."""
import argparse
from copy import deepcopy
import json
from pathlib import Path
import runpy
import struct

HERE = Path(__file__).resolve().parent
FIXTURES = HERE.parent
helpers = runpy.run_path(str(FIXTURES / "cdx-label-edit/generate_edits.py"))
edit, find, reader = (helpers[name] for name in ("edit", "one_record", "RESOURCE"))
models = json.loads((FIXTURES / "cdx-label-preview/cases.json").read_bytes())["renderable"]
BOXED, WATER = models["boxed"]["model"], models["water"]["model"]
OH = ((1, 1), (901, 81), 8, "O", 1)
BR = ((2, 2), (902, 81), 35, "Br", 0)
NH = ((1, 2), (901, 7), 7, "N", 1)


def read(file):
    return (FIXTURES / file).read_bytes()


def vectors():
    # The existing oracle checks all non-target records, including opaque encoding
    # choices, byte for byte and compares each target against the prior atom model.
    for name, source, graph, change in [
        ("overlap_before_oh", "cdx-mixed-caption/overlap_before.cdx", BOXED, OH),
        ("extended_br", "cdx-mixed-caption-edit/extended_before.cdx", BOXED, BR),
        ("prefix_br", "cdx-mixed-caption-edit/after_prefix.cdx", BOXED, BR),
        ("water_after_nh", "cdx-mixed-caption/water_after.cdx", WATER, NH),
    ]:
        raw, model, _ = edit(read(source), graph, *change)
        yield name, raw, model

    created = read("cdx-chemical-caption-create/chemical_first.cdx")
    first, model, _ = edit(created, BOXED, *OH)
    yield "chemical_first_oh", first, model
    both, model, _ = edit(first, model, *BR)
    yield "chemical_first_both", both, model
    second, model, _ = edit(created, BOXED, *BR)
    yield "chemical_first_br", second, model

    current = read("cdx-chemical-caption-create/mixed_first.cdx")
    first, model, _ = edit(current, BOXED, *OH)
    yield "mixed_first_oh", first, model
    both, model, _ = edit(first, model, *BR)
    yield "mixed_first_both", both, model

    records = reader.read(current)["records"]
    root = 0 if records[0]["tag"] == 0x8000 else None
    page = find(records, "start", 0x8001, root, 51)

    def owned(tag, parent):
        field = find(records, "property", tag, parent)
        return slice(*records[field]["payload_range"])

    front = find(records, "start", 0x8006, page, 903)
    depth = owned(0x000a, front)
    upper = bytearray(current)
    assert upper[depth] == struct.pack("<h", 8)
    upper[depth] = struct.pack("<h", 32767)
    yield "depth_max_before", bytes(upper), BOXED
    changed, model, _ = edit(bytes(upper), BOXED, *OH)
    yield "depth_max_oh", changed, model

    # Move only the second Node/label so Cl is on paper while Mg would cross its left edge.
    fragment = find(records, "start", 0x8003, page, 902)
    node = find(records, "start", 0x8004, fragment, 81)
    label = find(records, "start", 0x8006, node, 73)
    point, bounds = owned(0x0200, node), owned(0x0204, label)
    y, x = struct.unpack("<2i", current[point])
    top, left, bottom, right = struct.unpack("<4i", current[bounds])
    displacement = round(338.5 * 65536)
    edge = bytearray(current)
    edge[point] = struct.pack("<2i", y, x - displacement)
    edge[bounds] = struct.pack("<4i", top, left - displacement, bottom, right - displacement)
    moved = deepcopy(BOXED)
    atom = helpers["target_atom"](moved, 2, 2)
    atom["position"]["x_pt"] = (x - displacement) / 65536
    atom["authored_label"]["bounding_box"] = [(left - displacement) / 65536, top / 65536,
                                               (right - displacement) / 65536, bottom / 65536]
    assert atom["authored_label"]["bounding_box"][0] == 2.888671875
    yield "edge_cl_before", bytes(edge), moved
    changed, model, _ = edit(bytes(edge), moved, (2, 2), (902, 81), 12, "Mg", 0)
    assert helpers["target_atom"](model, 2, 2)["authored_label"]["bounding_box"][0] == -3.404296875
    yield "edge_mg_off_paper", changed, model


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", type=Path)
    parser.add_argument("--rdkit", action="store_true", help="also check the binary chemical graphs")
    args = parser.parse_args()
    if args.output_dir:
        args.output_dir.mkdir(parents=True, exist_ok=True)
    if args.rdkit:
        from rdkit import Chem, rdBase
        assert rdBase.rdkitVersion == "2026.09.2" and Chem.HasChemDrawCDXSupport()
        rdkit_check = runpy.run_path(str(FIXTURES / "cdx-label-edit/check_rdkit.py"))["check"]
        params = Chem.CDXMLParserParams()
        params.format = Chem.CDXMLFormat.CDX
        params.removeHs = params.sanitize = False
    for name, raw, model in vectors():
        path = (args.output_dir or HERE) / (name + ".cdx")
        if args.output_dir:
            path.write_bytes(raw)
        else:
            assert path.read_bytes() == raw, path
        if args.rdkit:
            rdkit_check(raw, model, params)
        print(name, len(raw))


if __name__ == "__main__":
    main()
