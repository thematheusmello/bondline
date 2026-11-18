# SPDX-License-Identifier: MIT OR Apache-2.0
"""Build original implicit-Element CDX witnesses with Python struct and FontTools."""

import argparse
from copy import deepcopy
from hashlib import sha256
import json
from pathlib import Path
import struct
import sys

HERE = Path(__file__).resolve().parent
PREVIEW = HERE.parent / "cdx-label-preview"
sys.path.insert(0, str(HERE.parent / "cdx-label-edit"))
from generate_edits import FONT_SHA, RESOURCE, authored, one_record, placed_box
from generate_edits import prop, quantize, rectangle, target_atom, vectors

ELEMENT = 0x0402
HYDROGENS = 0x042B
BODY = 0x0700
BOX = 0x0204
CYANIDE_SCOPE = (901, 7)
BOXED_SCOPE = (901, 81)


def source_fields(raw, scope):
    syntax = RESOURCE.read(raw)
    records = syntax["records"]
    root = 0 if records[0]["tag"] == 0x8000 else None
    page = one_record(records, "start", 0x8001, root, 51)
    fragment = one_record(records, "start", 0x8003, page, scope[0])
    node = one_record(records, "start", 0x8004, fragment, scope[1])
    label = one_record(records, "start", 0x8006, node, 73)
    fields = {}
    for tag, owner in ((ELEMENT, node), (HYDROGENS, node), (BODY, label), (BOX, label)):
        indices = [i for i, r in enumerate(records)
                   if r["kind"] == "property" and r["parent"] == owner and r["tag"] == tag]
        assert len(indices) <= 1, (scope, tag)
        fields[tag] = next(iter(indices), None)
    return syntax, fields


def rewrite(raw, scope, edits, prefix=None):
    syntax, fields = source_fields(raw, scope)
    records = syntax["records"]
    after = raw[slice(*syntax["header_range"])] + b"".join(
        (prefix[1] if prefix and prefix[0] == i else b"") +
        edits.get(i, raw[slice(*r["range"])])
        for i, r in enumerate(records)
    )
    new_syntax, new_fields = source_fields(after, scope)
    new_records = new_syntax["records"]
    assert syntax["framing"] == new_syntax["framing"]
    assert syntax["header_hex"] == new_syntax["header_hex"]
    inserted = new_fields[ELEMENT] if prefix else None
    if prefix:
        assert fields[ELEMENT] is None and fields[HYDROGENS] == prefix[0]
        assert inserted is not None and new_fields[HYDROGENS] == inserted + 1
        assert after[slice(*new_records[inserted]["range"])] == prefix[1]
    rebased = dict(zip(
        (i for i in range(len(records)) if edits.get(i) != b""),
        (i for i in range(len(new_records)) if i != inserted),
        strict=True,
    ))
    for old, new in rebased.items():
        first, second = records[old], new_records[new]
        for key in ("kind", "tag", "id"):
            assert first.get(key) == second.get(key), (old, key)
        for key in ("parent", "matched"):
            owner = first.get(key)
            assert second.get(key) == (rebased[owner] if owner is not None else None), (old, key)
        first_bytes = raw[slice(*first["range"])]
        second_bytes = after[slice(*second["range"])]
        assert second_bytes == edits.get(old, first_bytes), old
        assert (first_bytes != second_bytes) == (old in edits), old

    properties = []
    for tag, old_index in fields.items():
        new_index = new_fields[tag]
        old = records[old_index] if old_index is not None else {}
        new = new_records[new_index] if new_index is not None else {}
        old_hex = raw[slice(*old["range"])].hex() if old else None
        new_hex = after[slice(*new["range"])].hex() if new else None
        properties.append({
            "tag": tag,
            "before_record": old_index,
            "after_record": new_index,
            "changed": old_hex != new_hex,
            "before_range": old.get("range"),
            "after_range": new.get("range"),
            "before_encoding": old.get("encoding"),
            "after_encoding": new.get("encoding"),
            "before_hex": old_hex,
            "after_hex": new_hex,
        })
    return after, {
        "framing": syntax["framing"],
        "before_sha256": sha256(raw).hexdigest(),
        "after_sha256": sha256(after).hexdigest(),
        "before_record_count": len(records),
        "after_record_count": len(new_records),
        "source_owner_ids": {"page": 51, "fragment": scope[0], "node": scope[1], "text": 73},
        "properties": properties,
    }


def set_atom(raw, model, target, scope, atomic_number, symbol, hydrogens, *, derive_implicit=False):
    syntax, fields = source_fields(raw, scope)
    records = syntax["records"]
    element, h = fields[ELEMENT], fields[HYDROGENS]
    assert h is not None
    assert (element is not None) == derive_implicit
    if derive_implicit:
        assert atomic_number == 6
    atom = target_atom(model, *target)
    old_label = atom["authored_label"]
    old_run, = old_label["runs"]
    if element is not None:
        assert bytes.fromhex(records[element]["payload"]["hex"]) == struct.pack("<h", atom["element"])
    assert bytes.fromhex(records[h]["payload"]["hex"]) == struct.pack("<H", atom["explicit_hydrogens"])
    assert bytes.fromhex(records[fields[BOX]]["payload"]["hex"]) == rectangle(old_label["bounding_box"])
    old_body = bytes.fromhex(records[fields[BODY]]["payload"]["hex"])
    assert struct.unpack_from("<HH", old_body) == (1, 0)
    assert old_body[12:].decode("utf-8") == old_run["text"]
    point = atom["position"]
    text, glyphs = authored(symbol, hydrogens, old_run["style"]["size_pt"],
                            (point["x_pt"], point["y_pt"]))
    bounds = list(map(quantize, placed_box(glyphs)))
    edits = {}
    for tag, payload in (
        (HYDROGENS, struct.pack("<H", hydrogens)),
        (BODY, old_body[:12] + text.encode("utf-8")),
        (BOX, rectangle(bounds)),
    ):
        index = fields[tag]
        record = records[index]
        if payload.hex() != record["payload"]["hex"]:
            extended = derive_implicit and record["encoding"] == "extended"
            edits[index] = prop(tag, payload, extended=extended)
    if derive_implicit:
        edits[element] = b""
    insertion = (h, prop(ELEMENT, struct.pack("<h", atomic_number))) if atomic_number != 6 else None
    after, witness = rewrite(raw, scope, edits, insertion)

    after_model = deepcopy(model)
    new_atom = target_atom(after_model, *target)
    new_atom.update(element=atomic_number, explicit_hydrogens=hydrogens)
    new_atom["authored_label"]["bounding_box"] = bounds
    new_atom["authored_label"]["runs"][0]["text"] = text
    return after, after_model, witness


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", type=Path, required=True)
    output = parser.parse_args().output_dir
    output.mkdir(parents=True, exist_ok=False)
    previews, _ = vectors()
    sources = {}
    for name, original, target, scope, hydrogens in (
        ("cyanide_implicit", "cyanide", (1, 2), CYANIDE_SCOPE, 0),
        ("boxed_implicit", "boxed_reordered", (1, 1), BOXED_SCOPE, 2),
    ):
        raw, model, _ = previews[original]
        assert (PREVIEW / (original + ".cdx")).read_bytes() == raw
        implicit, implicit_model, _ = set_atom(
            raw, model, target, scope, 6, "C", hydrogens, derive_implicit=True,
        )
        (output / (name + ".cdx")).write_bytes(implicit)
        sources[name] = (implicit, implicit_model, target, scope)
    manifest = {"font_sha256": FONT_SHA, "cases": {}}
    for name, input_name, output_name, atomic_number, symbol, hydrogens in (
        ("cyanide_c_to_si", "cyanide_implicit", "cyanide_si", 14, "Si", 0),
        ("boxed_c_h_only", "boxed_implicit", "boxed_h_only", 6, "C", 3),
        ("boxed_c_to_noncarbon", "boxed_implicit", "boxed_nh2", 7, "N", 2),
    ):
        raw, model, target, scope = sources[input_name]
        after, after_model, witness = set_atom(raw, model, target, scope,
                                              atomic_number, symbol, hydrogens)
        (output / (output_name + ".cdx")).write_bytes(after)
        manifest["cases"][name] = {
            "before": input_name + ".cdx",
            "after": output_name + ".cdx",
            "params": {"page_id": 1, "fragment_id": target[0], "atom_id": target[1],
                       "element": atomic_number, "explicit_hydrogens": hydrogens},
            "before_model": model,
            "after_model": after_model,
            "source": witness,
        }

    raw, model, target, scope = sources["cyanide_implicit"]
    _, fields = source_fields(raw, scope)
    missing_h, witness = rewrite(raw, scope, {fields[HYDROGENS]: b""})
    (output / "cyanide_missing_h.cdx").write_bytes(missing_h)
    legacy_model = deepcopy(model)
    target_atom(legacy_model, *target)["explicit_hydrogens"] = None
    legacy_model["pages"][0]["size"] = {
        "width_pt": 210.0 * 72.0 / 25.4, "height_pt": 297.0 * 72.0 / 25.4,
    }
    del legacy_model["drawing_style"]
    manifest["retained"] = {"cyanide_missing_h": {
        "file": "cyanide_missing_h.cdx",
        "sha256": witness["after_sha256"],
        "model": legacy_model,
        "source": witness,
    }}
    (output / "cases.json").write_bytes((json.dumps(manifest, indent=2) + "\n").encode("utf-8"))
    print("3 edit pairs and 1 unchanged-only source checked record by record")


if __name__ == "__main__":
    main()
