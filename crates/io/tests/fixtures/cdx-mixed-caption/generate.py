# SPDX-License-Identifier: MIT OR Apache-2.0
"""Original CDX mixed-page witnesses using Python struct and the pinned Roboto outlines."""

import argparse
import copy
import hashlib
import json
from pathlib import Path
import struct
import subprocess
import sys

HERE = Path(__file__).resolve().parent
ROOT = next(p for p in HERE.parents if (p / "crates/io/tests/fixtures/cdx-label-preview").is_dir())
FIXTURES = ROOT / "crates/io/tests/fixtures"
ORACLE = ROOT / "crates/cdx/tests/fixtures/resources/resource_oracle.py"
assert hashlib.sha256(ORACLE.read_bytes()).hexdigest() == (
    "a5a2df85a9fc1a2df9ba57b1a7b5538c812a6f724e18d25a42a16d98d6ad5db2"
)
sys.path[:0] = [str(ORACLE.parent), str(FIXTURES / "cdx-caption")]
from resource_oracle import raw_records
from generate_captions import obj, prop

PAGE, FRAGMENT, NODE, TEXT = 0x8001, 0x8003, 0x8004, 0x8006
POSITION, Z, BODY = 0x0200, 0x000A, 0x0700
BLACK, RED, BLUE = [0, 0, 0], [255, 0, 0], [0, 0, 255]
DEFAULT_POSITION = (80.25, 100.75)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def start(records, tag):
    return next(i for i, r in enumerate(records) if r.get("tag") == tag and r["kind"] == "start")


def end(records, index):
    return next(r for r in records if r.get("matched") == index)


def caption(depth=2, *, raw_id=0xFFFFFFFF, point=DEFAULT_POSITION, size=12, color=0, extra=b""):
    x, y = point
    return obj(TEXT, raw_id,
               prop(POSITION, struct.pack("<ii", round(y * 65536), round(x * 65536))),
               prop(0x080C, b"\0"), prop(BODY, struct.pack("<6H", 1, 0, 0, 0, size * 20, color) + b"H"),
               prop(Z, struct.pack("<h", depth)) if depth is not None else b"", extra)


def marked(live, *, extra=b""):
    return obj(0x8011, 0, prop(0x0008, b"org.bondline.caption-id"), prop(0x0D00, b"\3\0"),
               prop(0x0D04, b"\1"), prop(0x0D05, f"1:{live}".encode("ascii")), extra)


def expected_caption(point=DEFAULT_POSITION, size=12, rgb=BLACK, live_id=1):
    return {"id": live_id, "position": {"x_pt": point[0], "y_pt": point[1]}, "runs": [{
        "text": "H", "style": {"font_family": "Roboto", "size_pt": float(size), "face": 0,
                               "rgb": [value / 255 for value in rgb]},
    }]}


def place(source, child, at):
    records = raw_records(source)[2]
    offset = (records[start(records, FRAGMENT)]["range"][0] if at == "before"
              else end(records, start(records, PAGE))["range"][0])
    return source[:offset] + child + source[offset:]


def model_with_texts(case, texts):
    model = copy.deepcopy(case["model"])
    model["pages"][0]["texts"] = texts
    return model


def optical_probes(label_glyph, caption_point, font_sha):
    # For H these contours have only straight horizontal and vertical edges.
    # Inside cells have no outline in their expanded sampling rectangle.
    from fontTools import __version__
    from fontTools.pens.recordingPen import RecordingPen
    from fontTools.ttLib import TTFont

    assert __version__ == "4.61.1"
    path = ROOT / "assets/fonts/Roboto-Regular.ttf"
    assert digest(path.read_bytes()) == font_sha
    font = TTFont(path)
    pen = RecordingPen()
    font.getGlyphSet()[font.getBestCmap()[ord("H")]].draw(pen)
    assert pen.value[0][0] == "moveTo" and pen.value[-1][0] == "closePath"
    assert all(op == "lineTo" for op, _ in pen.value[1:-1])
    vertices = [args[0] for _, args in pen.value[:-1]]
    unit = 24 / font["head"].unitsPerEm
    font.close()
    assert label_glyph["character"] == "H" and label_glyph["size_pt"] == 24

    def edges(origin, zoom):
        points = [((origin[0] + x * unit) * zoom, (origin[1] - y * unit) * zoom) for x, y in vertices]
        lines = list(zip(points, points[1:] + points[:1]))
        assert all(x == u or y == v for (x, y), (u, v) in lines)
        return lines

    def cell(lines, x, y):
        # Allow 1/32 pixel of extra clearance beyond the whole pixel.
        if any(min(x1, x2) <= x + 1.03125 and max(x1, x2) >= x - 0.03125
               and min(y1, y2) <= y + 1.03125 and max(y1, y2) >= y - 0.03125
               for (x1, y1), (x2, y2) in lines):
            return None
        return bool(sum(a == b and a > x + 0.5 and min(u, v) <= y + 0.5 < max(u, v)
                        for (a, u), (b, v) in lines) % 2)

    probes = []
    for zoom in (1, 2):
        lower = edges(label_glyph["origin_pt"], zoom)
        upper = edges(caption_point, zoom)
        candidates = {"overlap": [], "caption_only": [], "label_only": []}
        # The N and 2 of the underlying NH2 sit outside this H-only test interval.
        for y in range(int(114 * zoom), int(132 * zoom)):
            for x in range(int(157 * zoom), int(173 * zoom)):
                a, b = cell(lower, x, y), cell(upper, x, y)
                if (a, b) == (True, True):
                    candidates["overlap"].append([x, y])
                elif (a, b) == (False, True):
                    candidates["caption_only"].append([x, y])
                elif (a, b) == (True, False):
                    candidates["label_only"].append([x, y])
        assert all(candidates.values()), (zoom, candidates)
        probes.append({
            "size_px": 512 * zoom, "zoom_percent": 100 * zoom, "center_pt": [256, 256],
            "samples": [{"kind": role, "xy": coords[len(coords) // 2],
                         "rgb": RED if role == "label_only" else BLUE}
                        for role, coords in candidates.items()],
        })
    return probes


def vectors():
    folder = FIXTURES / "cdx-label-preview"
    label_cases = json.loads((folder / "cases.json").read_bytes())
    cases = label_cases["renderable"]
    out = {}
    def source(name):
        raw = (folder / (name + ".cdx")).read_bytes()
        assert digest(raw) == cases[name]["sha256"]
        return raw

    box = source("boxed")
    for name, case in cases.items():
        data = source(name)
        out[name + "_after"] = (place(data, caption(), "after"), model_with_texts(case, [expected_caption()]))
    out["boxed_before"] = (place(box, caption(), "before"), model_with_texts(cases["boxed"], [expected_caption()]))
    assert digest(out["boxed_before"][0]) == "f985cf119aa69a12b0ceb90fa9ce7e31dc49662d4a1c37cdceb991e89020b383"
    assert digest(out["boxed_after"][0]) == "eb4525fa4ab3cbad1c63eff56e01b1f6565f7d6dc122b092c539d4d76b1efed4"

    lower = cases["boxed"]["ink"][0]
    assert lower["rgb8"] == RED
    glyph = next(g for g in lower["glyphs"] if g["character"] == "H")
    point = (glyph["origin_pt"][0] + 3, glyph["origin_pt"][1])
    overlay = caption(7, raw_id=73, point=point, size=24, color=5)
    expected = model_with_texts(cases["boxed"], [expected_caption(point, 24, BLUE)])
    for at in ("before", "after"):
        out["overlap_" + at] = (place(box, overlay, at), expected)
    optics = optical_probes(glyph, point, label_cases["font_sha256"])

    out["two_captions"] = (
        place(box, overlay + caption(), "before"), model_with_texts(cases["boxed"], [
            expected_caption(live_id=2), expected_caption(point, 24, BLUE, live_id=1)]))
    out["marked_sparse_caption"] = (
        place(box, caption(raw_id=23, extra=marked(19)), "after"),
        model_with_texts(cases["boxed"], [expected_caption(live_id=19)]))
    # The inherited order between zero-depth chemical objects stays in the old geometry proof.
    controls = {
        "caption_negative": caption(-1), "caption_zero": caption(0), "caption_without_z": caption(None),
        "duplicate_depth": overlay + caption(7),
        "duplicate_named_page_child": caption(raw_id=901),
        "duplicate_live_id": caption(2, raw_id=23, extra=marked(1)) + overlay,
        "unknown_identity_property": caption(extra=marked(19, extra=prop(0x4001, b"extra"))),
        "opaque_caption_property": caption(extra=prop(0x4001, b"opaque")),
        "duplicate_caption_z": caption(extra=prop(Z, b"\1\0")),
        "short_caption_z": caption(None, extra=prop(Z, b"\1")),
        "bare_caption_peer": caption() + obj(TEXT, 12, prop(BODY, b"\0\0x")),
        "unknown_page_peer": caption() + obj(0xc011, 9, prop(0x4009, b"extra")),
        "unsupported_group": caption() + obj(0x8002, 3),
    }
    # ID zero is unassigned, so it makes a positive control for the same-parent collision check.
    out["anonymous_caption"] = (place(box, caption(raw_id=0), "after"),
                                model_with_texts(cases["boxed"], [expected_caption()]))
    for name, chunk in controls.items():
        out[name] = (place(box, chunk, "before"), None)
    for name in ("contradictory_ink_box", "local_margin", "off_paper", "raw_label_z"):
        out["chemical_" + name] = (place((folder / (name + ".cdx")).read_bytes(), caption(), "after"), None)
    # These are all distinct raw sources even when the expected model and pixels agree.
    assert len({raw for raw, _ in out.values()}) == len(out)
    return out, optics


def baseline(cli, path):
    request = {"command": "document.open_cdx", "params": {"path": str(path.resolve())}}
    result = subprocess.run([str(cli.resolve()), "run", "--cmd", json.dumps(request)],
                            capture_output=True, timeout=15)
    assert result.returncode == 2, (path, result.returncode)
    error = json.loads(result.stderr)["error"]
    assert error["code"] == "unsupported_content", (path, error)
    return {"code": error["code"], "detail": error["message"].split(" during Decode for ", 1)[0]}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--output-dir", type=Path)
    mode.add_argument("--check", type=Path)
    parser.add_argument("--baseline-cli", type=Path)
    args = parser.parse_args()
    dst = args.check or args.output_dir
    if not args.check:
        dst.mkdir(parents=True, exist_ok=False)
    def emit(name, data):
        path = dst / name
        if args.check:
            assert path.read_bytes() == data, name
        else:
            path.write_bytes(data)
    cases, optics = vectors()
    manifest, legacy = {}, {}
    for name, (data, expected) in cases.items():
        framing, _, records = raw_records(data)
        page = start(records, PAGE)
        direct = [r for r in records if r["kind"] == "start" and r["parent"] == page]
        entry = {"sha256": digest(data), "framing": framing,
                 "page_children": [[r["tag"], r["id"]] for r in direct], "model": expected}
        if name in ("overlap_before", "overlap_after", "two_captions"):
            entry["optical_probes"] = optics
        manifest[name] = entry
        emit(name + ".cdx", data)
        if args.baseline_cli:
            legacy[name] = baseline(args.baseline_cli, dst / (name + ".cdx"))
            assert (dst / (name + ".cdx")).read_bytes() == data
    assert len({v["framing"] for v in manifest.values() if v["model"] is not None}) == 5
    def write_json(name, value):
        emit(name, (json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode())
    write_json("cases.json", manifest)
    if args.baseline_cli:
        write_json("legacy-open.json", legacy)
    print(f"{len(cases)} originals, {sum(v['model'] is not None for v in manifest.values())} qualified models")


if __name__ == "__main__":
    main()
