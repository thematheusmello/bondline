# SPDX-License-Identifier: MIT OR Apache-2.0
"""Original caption stacking witnesses. No Bondline writer is used."""

import argparse
from hashlib import sha256
import json
from pathlib import Path
import struct
import subprocess

from caption_oracle import RESOURCE
from generate_captions import (
    HEADER, ROBOTO, WHITE, font_data, model, obj, palette_data, prop, string_data,
)

BLUE = (16384, 32768, 49152)
POINT = (80.25, 100.75)
Z_ORDER = 0x000A


def caption(identity, depth, *, body="H", point=POINT, ink=0, face=0, extra=()):
    x, y = point
    fields = [
        prop(0x0200, struct.pack("<ii", int(y * 65536), int(x * 65536))),
        prop(0x080C, bytes(1)),
        prop(0x0700, string_data(body.encode("utf-8"), [(0, 0, face, 240, ink)])),
    ]
    if depth is not None:
        fields.append(prop(Z_ORDER, struct.pack("<h", depth)))
    return obj(0x8006, identity, *fields, *extra)


def binary(*children, page_fields=(), root_fields=()):
    extent = struct.pack("<i", 512 * 65536)
    page = obj(
        0x8001, 55, prop(0x0811, b"\1"), prop(0x0812, extent),
        prop(0x0813, extent), *page_fields, *children,
    )
    return (
        HEADER
        + prop(0x0100, font_data(ROBOTO, 1))
        + prop(0x0300, palette_data([WHITE, BLUE]))
        + b"".join(root_fields)
        + page
        + bytes(2)
    )


def text(identity, *, body="H", point=POINT, blue=False):
    rgb = tuple(n / 65535 for n in BLUE) if blue else (0.0, 0.0, 0.0)
    expected = model(body, rgb=rgb, point=point)["pages"][0]["texts"][0]
    return {**expected, "id": identity}


def reference(*paint_order):
    document = model()
    document["pages"][0]["texts"] = list(paint_order)
    return document


def vectors():
    # The smaller authored ID belongs to the upper caption, independent of record/Z order.
    back = caption(0xFFFFFFFF, -32768)
    front = caption(17, 32767, ink=3)
    overlap = reference(text(2), text(1, blue=True))
    separate = (142.5, 110.25)
    explicit = prop(Z_ORDER, struct.pack("<h", 7))
    return {
        "front_first": (binary(front, back), overlap),
        "back_first": (binary(back, front), overlap),
        "omitted_below_positive": (
            binary(caption(17, 7, ink=3), caption(0xFFFFFFFF, None)), overlap,
        ),
        "separate_anonymous": (
            binary(caption(0, 0, body="AV2"), caption(0, -3, body="Phase λ", point=separate, ink=3)),
            reference(text(2, body="Phase λ", point=separate, blue=True), text(1, body="AV2")),
        ),
        "explicit_single": (binary(back), reference(text(1))),
        "omitted_overlap": (binary(caption(9, None), caption(17, None, ink=3)), None),
        "omitted_disjoint": (binary(caption(9, None), caption(17, None, point=separate)), None),
        "explicit_tie": (binary(caption(9, 32767), front), None),
        "zero_tie": (binary(caption(9, None), caption(17, 0, ink=3)), None),
        "omitted_with_negative": (binary(caption(17, None, ink=3), back), None),
        "duplicate_owner_field": (binary(caption(9, -3, extra=(explicit,)), front), None),
        "duplicate_sibling_id": (binary(caption(17, -3), front), None),
        "page_owner": (binary(front, back, page_fields=(explicit,)), None),
        "root_owner": (binary(front, back, root_fields=(explicit,)), None),
        "width_one": (binary(caption(9, None, extra=(prop(Z_ORDER, b"\xff"),)), front), None),
        "width_four": (binary(caption(9, None, extra=(prop(Z_ORDER, bytes(4)),)), front), None),
        "grouped_peer": (binary(front, obj(0x8002, 91, back)), None),
        "bad_peer": (binary(front, caption(9, -3, face=1)), None),
    }


def order_facts(data):
    # Decode boundaries and ownership through the independent resource reader.
    source = RESOURCE.read(data)
    records = source["records"]
    return [
        {
            "owner_record": record["parent"],
            "owner_tag": records[record["parent"]]["tag"] if record["parent"] is not None else None,
            "source_id": records[record["parent"]]["id"] if record["parent"] is not None else None,
            "payload": data[slice(*record["payload_range"])].hex(),
            "value": struct.unpack("<h", data[slice(*record["payload_range"])])[0]
            if record["payload_range"][1] - record["payload_range"][0] == 2 else None,
        }
        for record in records if record["kind"] == "property" and record["tag"] == Z_ORDER
    ]


def legacy_open(cli, path):
    request = {"command": "document.open_cdx", "params": {"path": str(path)}}
    result = subprocess.run(
        [str(cli.resolve()), "run", "--cmd", json.dumps(request)], check=False,
        capture_output=True, timeout=15,
    )
    assert result.returncode == 2, (path, result.returncode, result.stderr)
    error = json.loads(result.stderr)["error"]
    assert error["code"] == "unsupported_content"
    return {"code": error["code"], "detail": error["message"].split(" during Decode for ", 1)[0]}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--baseline-cli", type=Path)
    args = parser.parse_args()
    args.output_dir.mkdir(parents=True, exist_ok=False)
    cases = {}
    for name, (data, expected) in vectors().items():
        path = args.output_dir / (name + ".cdx")
        path.write_bytes(data)
        cases[name] = {
            "sha256": sha256(data).hexdigest(),
            "model": expected,
            "z_order": order_facts(data),
        }
        if args.baseline_cli is not None:
            cases[name]["legacy"] = legacy_open(args.baseline_cli, path.resolve())
            assert path.read_bytes() == data
    (args.output_dir / "cases.json").write_text(
        json.dumps(cases, ensure_ascii=False, indent=2) + "\n", encoding="utf-8",
    )


if __name__ == "__main__":
    main()
