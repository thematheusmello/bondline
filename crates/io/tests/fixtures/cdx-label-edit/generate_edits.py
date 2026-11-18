# SPDX-License-Identifier: MIT OR Apache-2.0
"""Generate original CDX label edit witnesses using struct and FontTools."""

import argparse
from copy import deepcopy
from hashlib import sha256
import json
from pathlib import Path
import struct
import sys

HERE = Path(__file__).resolve().parent
PREVIEW = HERE.parent / "cdx-label-preview"
sys.path.insert(0, str(PREVIEW))
from generate import FONT_SHA, RESOURCE, authored, placed_box, prop, quantize, vectors

CASES = [
    ("water_to_nh", "water", (1, 2), (901, 7), 7, "N", 1),
    ("boxed_nh2_to_oh", "boxed_reordered", (1, 1), (901, 81), 8, "O", 1),
    ("boxed_cl_to_br", "boxed_reordered", (2, 2), (902, 81), 35, "Br", 0),
]


def one_record(records, kind, tag, parent=None, identity=None):
    index, = (i for i, record in enumerate(records)
              if record["kind"] == kind and record["tag"] == tag
              and record["parent"] == parent
              and (identity is None or record.get("id") == identity))
    return index


def target_atom(model, fragment_id, atom_id):
    fragment, = (f for f in model["pages"][0]["fragments"] if f["id"] == fragment_id)
    atom, = (a for a in fragment["atoms"] if a["id"] == atom_id)
    return atom


def rectangle(bounds):
    left, top, right, bottom = bounds
    return struct.pack("<4i", *(round(value * 65536) for value in
                               (top, left, bottom, right)))


def edit(source, model, target, scope, element, symbol, hydrogens):
    tree = RESOURCE.read(source)
    records = tree["records"]
    root = 0 if records[0]["tag"] == 0x8000 else None
    page = one_record(records, "start", 0x8001, root, 51)
    fragment = one_record(records, "start", 0x8003, page, scope[0])
    node = one_record(records, "start", 0x8004, fragment, scope[1])
    label = one_record(records, "start", 0x8006, node, 73)
    element_field = one_record(records, "property", 0x0402, node)
    hydrogens_field = one_record(records, "property", 0x042B, node)
    box_field = one_record(records, "property", 0x0204, label)
    text_field = one_record(records, "property", 0x0700, label)

    def payload(index):
        return source[slice(*records[index]["payload_range"])]

    atom = target_atom(model, *target)
    body = payload(text_field)
    old_run, = atom["authored_label"]["runs"]
    assert struct.unpack("<h", payload(element_field)) == (atom["element"],)
    assert struct.unpack("<H", payload(hydrogens_field)) == (atom["explicit_hydrogens"],)
    assert payload(box_field) == rectangle(atom["authored_label"]["bounding_box"])
    assert struct.unpack_from("<HH", body) == (1, 0)
    assert body[12:].decode("utf-8") == old_run["text"]
    point = atom["position"]
    text, glyphs = authored(symbol, hydrogens, old_run["style"]["size_pt"],
                            (point["x_pt"], point["y_pt"]))
    bounds = list(map(quantize, placed_box(glyphs)))
    replacements = {
        element_field: struct.pack("<h", element),
        hydrogens_field: struct.pack("<H", hydrogens),
        box_field: rectangle(bounds),
        text_field: body[:12] + text.encode("utf-8"),
    }
    changed = {i: prop(records[i]["tag"], part) for i, part in replacements.items()
               if part != payload(i)}
    edited = source[slice(*tree["header_range"])] + b"".join(
        changed.get(i, source[slice(*rec["range"])]) for i, rec in enumerate(records)
    )

    after = RESOURCE.read(edited)
    assert tree["framing"] == after["framing"]
    assert tree["header_hex"] == after["header_hex"]
    properties = []
    for i, (before, current) in enumerate(zip(records, after["records"], strict=True)):
        for key in ("kind", "tag", "id", "parent", "matched"):
            assert before.get(key) == current.get(key), (i, key)
        original_record = source[slice(*before["range"])]
        edited_record = edited[slice(*current["range"])]
        assert (original_record != edited_record) == (i in changed), i
        if i in replacements:
            assert edited[slice(*current["payload_range"])] == replacements[i]
            if i in changed:
                assert current["encoding"] == "short"
            properties.append({
                "record": i,
                "tag": before["tag"],
                "changed": i in changed,
                "before_range": before["range"],
                "after_range": current["range"],
                "before_encoding": before["encoding"],
                "after_encoding": current["encoding"],
                "before_hex": original_record.hex(),
                "after_hex": edited_record.hex(),
            })

    after_model = deepcopy(model)
    atom = target_atom(after_model, *target)
    atom.update(element=element, explicit_hydrogens=hydrogens)
    atom["authored_label"]["bounding_box"] = bounds
    atom["authored_label"]["runs"][0]["text"] = text
    witness = {
        "framing": tree["framing"],
        "before_sha256": sha256(source).hexdigest(),
        "after_sha256": sha256(edited).hexdigest(),
        "record_count": len(records),
        "source_owner_ids": {"page": 51, "fragment": scope[0], "node": scope[1], "text": 73},
        "properties": properties,
    }
    return edited, after_model, witness


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", type=Path, required=True)
    args = parser.parse_args()
    args.output_dir.mkdir(parents=True, exist_ok=False)
    previews, _ = vectors()
    manifest = {"font_sha256": FONT_SHA, "cases": {}}
    for name, before_name, target, scope, element, symbol, hydrogens in CASES:
        before, before_model, _ = previews[before_name]
        before_path = PREVIEW / (before_name + ".cdx")
        assert before_path.read_bytes() == before, before_path
        after, after_model, witness = edit(before, before_model, target, scope,
                                           element, symbol, hydrogens)
        after_path = args.output_dir / (name + ".cdx")
        after_path.write_bytes(after)
        manifest["cases"][name] = {
            "before": "../cdx-label-preview/" + before_path.name,
            "after": after_path.name,
            "params": {"page_id": 1, "fragment_id": target[0], "atom_id": target[1],
                       "element": element, "explicit_hydrogens": hydrogens},
            "before_model": before_model,
            "after_model": after_model,
            "source": witness,
        }
    (args.output_dir / "cases.json").write_bytes(
        (json.dumps(manifest, indent=2) + "\n").encode("utf-8")
    )
    print(f"{len(CASES)} original edit pairs, checked record by record")


if __name__ == "__main__":
    main()
