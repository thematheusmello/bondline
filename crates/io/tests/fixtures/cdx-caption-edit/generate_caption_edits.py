# SPDX-License-Identifier: MIT OR Apache-2.0
"""Generate exact original CDX caption edits with struct and FontTools."""

import argparse
from copy import deepcopy
from hashlib import sha256
from itertools import accumulate
import json
from pathlib import Path
import struct
import sys

HERE = Path(__file__).resolve().parent
CAPTIONS = HERE.parent / "cdx-caption"
REPO = HERE.parents[4]
sys.path[:0] = [str(CAPTIONS), str(REPO / "xtask/oracles")]
from caption_oracle import RESOURCE
from clearance_oracle import FONT, UPEM, cmap, font, glyph_bounds, kern
from generate_captions import HEADER, ROBOTO, WHITE, font_data, obj, palette_data
from generate_captions import prop, string_data, vectors as caption_vectors
from generate_ordered import caption
from generate_runs import BLACK, document, font_facts, projected, run

POINT = (120.0, 228.0)
BLUE = (13107, 26214, 52428)
ORANGE = (52428, 13107, 0)
GREEN = (0, 39321, 13107)
TEXT = 0x0700
CASES = [
    ("single_grow", "single_before", "single_after", 1, 0, "Bondline λ notes"),
    ("multi_shift", "multi_before", "multi_after", 2, 0, "AVAV"),
    ("tail_unicode", "multi_before", "tail_after", 2, 2, "λéπΩ"),
    ("multi_shrink", "multi_before", "shrink_after", 2, 0, "V"),
]


def multi():
    first = caption(91, 4, body="H", point=(96.25, 100.75), ink=5)
    anonymous = caption(0, -20, body="AV2", point=(248.0, 176.25))
    back = caption(17, -5, body="H", point=(96.25, 100.75))
    starts = [run(0, size=480, ink=3), run(2, size=320, font=1), run(3, size=640, ink=4)]
    x, y = POINT
    target = obj(
        0x8006, 0,
        prop(0x000A, struct.pack("<h", 17)),
        prop(TEXT, string_data("AV λΩé".encode("utf-8"), starts), extended=True),
        prop(0x080C, bytes(1)),
        prop(0x0200, struct.pack("<ii", int(y * 65536), int(x * 65536))),
    )
    extent = struct.pack("<i", 512 * 65536)
    page = obj(
        0x8001, 55,
        prop(0x0813, extent, extended=True), prop(0x0812, extent),
        prop(0x0811, b"\1"), first, anonymous, back, target,
    )
    fonts = prop(0x0100, font_data([*ROBOTO, (1, 65001, b"Roboto")], 1), extended=True)
    colors = prop(0x0300, palette_data([WHITE, BLUE, ORANGE, GREEN]))
    raw = HEADER + obj(0x8000, 17, page, fonts, colors) + bytes(2)
    model = document(
        projected([("AV2", 12, BLACK)], identity=1, point=(248.0, 176.25)),
        projected([("H", 12, BLACK)], identity=3, point=(96.25, 100.75)),
        projected([("H", 12, GREEN)], identity=4, point=(96.25, 100.75)),
        projected([("AV", 24, BLUE), (" ", 16, BLACK), ("λΩé", 32, ORANGE)],
                  identity=2, point=POINT),
    )
    return raw, model


def fields(raw):
    tree = RESOURCE.read(raw)
    records = tree["records"]
    root = 0 if records[0]["tag"] == 0x8000 else None
    page, = (i for i, r in enumerate(records)
             if r["kind"] == "start" and r["tag"] == 0x8001 and r["parent"] == root)
    texts = {}
    for index, r in enumerate(records):
        if r["kind"] == "start" and r["tag"] == 0x8006 and r["parent"] == page:
            texts[index] = r["id"]
    ordered = sorted(texts, key=texts.get)
    mapped = {record: ordinal for ordinal, record in enumerate(ordered, 1)}
    bodies, depths = {}, {}
    for index, r in enumerate(records):
        parent = r["parent"]
        if r["kind"] != "property" or parent not in texts:
            continue
        identity = mapped[parent]
        if r["tag"] == TEXT:
            assert identity not in bodies
            bodies[identity] = index
        elif r["tag"] == 0x000A:
            assert identity not in depths
            depths[identity], = struct.unpack("<h", raw[slice(*r["payload_range"])])
    assert len(bodies) == len(texts)
    if len(texts) > 1:
        assert len(set(depths.values())) == len(texts)
    encounter = [{"record": index, "source_id": identity, "text_id": mapped[index],
                  "depth": depths.get(mapped[index])}
                 for index, identity in texts.items()]
    return tree, bodies, {
        "page_source_id": records[page]["id"],
        "encounter": encounter,
        "source_id_order": list(range(1, len(texts) + 1)),
        "paint_order": sorted(bodies, key=lambda identity: depths.get(identity, 0)),
    }


def run_table(raw, record):
    payload = raw[slice(*record["payload_range"])]
    count, = struct.unpack_from("<H", payload)
    body_start = 2 + 10 * count
    assert count >= 1 and body_start < len(payload)
    rows = list(struct.iter_unpack("<H8s", payload[2:body_start]))
    starts = [start for start, _ in rows]
    assert starts[0] == 0 and starts == sorted(set(starts))
    body = payload[body_start:]
    assert starts[-1] < len(body) and body[:starts[-1]].isascii()
    ends = starts[1:] + [len(body)]
    return payload[:2], rows, [body[start:end].decode("utf-8")
                               for start, end in zip(starts, ends, strict=True)]


def text_for(model, identity):
    expected, = (item for item in model["pages"][0]["texts"] if item["id"] == identity)
    return expected


def changed(raw, model, identity, run_index, new_text):
    before, bodies, order = fields(raw)
    assert order["paint_order"] == [item["id"] for item in model["pages"][0]["texts"]]
    target = bodies[identity]
    old_record = before["records"][target]
    count, rows, bodies = run_table(raw, old_record)
    assert bodies == [item["text"] for item in text_for(model, identity)["runs"]]
    assert bodies[run_index] != new_text
    bodies[run_index] = new_text
    segments = [body.encode("utf-8") for body in bodies]
    assert all(segments) and all(s.isascii() for s in segments[:-1])
    starts = list(accumulate((len(part) for part in segments[:-1]), initial=0))
    payload = count + b"".join(
        struct.pack("<H", start) + style
        for start, (_, style) in zip(starts, rows, strict=True)
    ) + b"".join(segments)
    start, end = old_record["range"]
    edited = raw[:start] + prop(TEXT, payload) + raw[end:]
    after, new_indices, new_order = fields(edited)
    assert after["framing"] == before["framing"]
    assert after["header_hex"] == before["header_hex"]
    assert new_order == order
    assert new_indices[identity] == target
    for i, (old, new) in enumerate(zip(before["records"], after["records"], strict=True)):
        for key in ("kind", "tag", "id", "parent", "matched"):
            assert old.get(key) == new.get(key), (i, key)
        old_bytes = raw[slice(*old["range"])]
        new_bytes = edited[slice(*new["range"])]
        assert (old_bytes != new_bytes) == (i == target), i
    new_record = after["records"][target]
    new_count, new_rows, new_bodies = run_table(edited, new_record)
    assert new_count == count and new_bodies == bodies
    assert new_rows == [(start, style) for start, (_, style) in zip(starts, rows, strict=True)]
    assert new_record["encoding"] == "short"

    new_model = deepcopy(model)
    text_for(new_model, identity)["runs"][run_index]["text"] = new_text
    owner_record = old_record["parent"]
    return edited, new_model, {
        "framing": before["framing"],
        "before_sha256": sha256(raw).hexdigest(),
        "after_sha256": sha256(edited).hexdigest(),
        "record_count": len(before["records"]),
        "order": order,
        "owner_record": owner_record,
        "owner_source_id": before["records"][owner_record]["id"],
        "text_property_record": target,
        "before_range": old_record["range"],
        "after_range": new_record["range"],
        "before_encoding": old_record["encoding"],
        "after_encoding": new_record["encoding"],
        "run_count_hex": count.hex(),
        "before_starts": [start for start, _ in rows],
        "after_starts": starts,
        "style_hex": [style.hex() for _, style in rows],
        "before_record_hex": raw[slice(*old_record["range"])].hex(),
        "after_record_hex": edited[slice(*new_record["range"])].hex(),
    }


def ink_facts(model, ligature_pairs):
    marks = font["GDEF"].table.GlyphClassDef.classDefs
    results = []
    for text in model["pages"][0]["texts"]:
        cursor = text["position"]["x_pt"]
        baseline = text["position"]["y_pt"]
        runs = []
        previous_run = None
        for run in text["runs"]:
            body = run["text"]
            if previous_run:
                pair = previous_run[-1] + body[0]
                assert kern(*pair) == 0 and pair not in ligature_pairs, pair
            previous_run = body
            rgb = [round(channel * 255) for channel in run["style"]["rgb"]]
            scale = run["style"]["size_pt"] / UPEM
            start = cursor
            previous = None
            bounds = []
            for character in body:
                name = cmap[ord(character)]
                assert marks.get(name) != 3
                if previous:
                    cursor += kern(previous, character) * scale
                outline = glyph_bounds(character)
                if outline is not None:
                    left, bottom, right, top = outline
                    bounds.append([cursor + left * scale, baseline - top * scale,
                                   cursor + right * scale, baseline - bottom * scale])
                cursor += font["hmtx"][name][0] * scale
                previous = character
            box = None
            if bounds:
                box = [min(rect[0] for rect in bounds), min(rect[1] for rect in bounds),
                       max(rect[2] for rect in bounds), max(rect[3] for rect in bounds)]
                assert all(1 < v < 511 for v in box), box
            runs.append({"rgb8": rgb, "start_x_pt": start, "end_x_pt": cursor, "bounds_pt": box})
        results.append({"text_id": text["id"], "runs": runs})
    return results


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", type=Path, required=True)
    output = parser.parse_args().output_dir
    output.mkdir(parents=True, exist_ok=False)
    originals, _ = caption_vectors()
    previous = originals["caption_extended"]
    assert previous[0] == (CAPTIONS / "caption_extended.cdx").read_bytes()
    original_sources = {"single_before": previous, "multi_before": multi()}
    for name, (raw, _) in original_sources.items():
        (output / (name + ".cdx")).write_bytes(raw)
    facts = font_facts(FONT)
    forbidden_joins = set(facts["ascii_ligature_joins"])
    assert kern("f", "l") == 0 and "fl" in forbidden_joins
    assert 0x1FAE0 not in cmap
    wide_advance = font["hmtx"][cmap[ord("W")]][0] + kern("W", "W")
    wide_right = glyph_bounds("W")[2]
    assert POINT[0] + 24 * (79 * wide_advance + wide_right) / UPEM > 511
    manifest = {"font_sha256": facts["sha256"], "cases": {}}
    for name, in_name, out_name, identity, run_index, text in CASES:
        raw, model = original_sources[in_name]
        edited, expected, witness = changed(raw, model, identity, run_index, text)
        file = out_name + ".cdx"
        (output / file).write_bytes(edited)
        manifest["cases"][name] = {
            "before": in_name + ".cdx",
            "after": file,
            "params": {"page_id": 1, "text_id": identity, "run_index": run_index, "text": text},
            "before_model": model,
            "after_model": expected,
            "source": witness,
            "before_ink": ink_facts(model, forbidden_joins),
            "after_ink": ink_facts(expected, forbidden_joins),
        }
    (output / "cases.json").write_bytes(
        (json.dumps(manifest, ensure_ascii=False, indent=2) + "\n").encode("utf-8")
    )
    print(f"{len(CASES)} original edit pairs with exact source and FontTools facts")


if __name__ == "__main__":
    main()
