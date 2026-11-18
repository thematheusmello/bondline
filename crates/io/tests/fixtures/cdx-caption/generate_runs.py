# SPDX-License-Identifier: MIT OR Apache-2.0
"""Original explicit-run witnesses and independent font checks."""

import argparse
from hashlib import sha256
import json
from pathlib import Path
import struct

import fontTools
from fontTools.ttLib import TTFont

from caption_oracle import RESOURCE
from generate_captions import ROBOTO, WHITE, model, obj, prop, scene, string_data
from generate_ordered import BLUE, binary, caption, legacy_open

HERE = Path(__file__).resolve().parent
RED = (65535, 0, 0)
BLACK = (0, 0, 0)
FONT_SHA256 = "56a45233d29f11b4dfb86d248e921939d115778f87325e7ae8cc108383d6664d"


def run(start, *, size=240, ink=0, font=0, face=0):
    return (start, font, face, size, ink)


def single(body, runs, **options):
    if isinstance(body, str):
        body = body.encode("utf-8")
    return scene(text_replace={0x0700: string_data(body, runs)}, **options)


def projected(segments, *, identity=1, point=(80.25, 100.75)):
    text = model(point=point)["pages"][0]["texts"][0]
    text["id"] = identity
    text["runs"] = [
        {
            "text": body,
            "style": {
                "font_family": "Roboto", "size_pt": float(points), "face": 0,
                "rgb": [channel / 65535 for channel in colour],
            },
        }
        for body, points, colour in segments
    ]
    return text


def document(*texts):
    d = model()
    d["pages"][0]["texts"] = list(texts)
    return d


def vectors():
    front_point = (130.0, 145.0)
    front = obj(
        0x8006, 73,
        prop(0x0200, struct.pack("<ii", int(front_point[1] * 65536), int(front_point[0] * 65536))),
        prop(0x080C, bytes(1)),
        prop(0x0700, string_data(b"H g", [run(0, ink=3), run(2)])),
        prop(0x000A, struct.pack("<h", 2)),
    )
    return {
        "mixed_runs": (
            single("AV Hg", [run(0, size=480, ink=3), run(2, size=320), run(3, size=640, ink=4)],
                   palette=[WHITE, RED, BLUE]),
            document(projected([("AV", 24, RED), (" ", 16, BLACK), ("Hg", 32, BLUE)])),
        ),
        "unicode_tail": (
            single("H λΩé", [run(0), run(2, size=260, ink=3)], palette=[WHITE, BLUE]),
            document(projected([("H ", 12, BLACK), ("λΩé", 13, BLUE)])),
        ),
        "font_one": (
            single("Hg", [run(0), run(1, font=1)], fonts=[*ROBOTO, (1, 65001, b"Roboto")],
                   framing="ExplicitDocumentFileEnd", reverse=True, extended=True),
            document(projected([("H", 12, BLACK), ("g", 12, BLACK)])),
        ),
        "ordered_peer": (
            binary(front, caption(17, -2, body="AV")),
            document(projected([("AV", 12, BLACK)]),
                     projected([("H ", 12, BLUE), ("g", 12, BLACK)], identity=2, point=front_point)),
        ),
        "legacy_two_runs": (
            (HERE / "two_runs.cdx").read_bytes(),
            document(projected([("Bond", 12, BLACK), ("line", 12, BLACK)])),
        ),
        "initial_gap": (single("HgH", [run(1), run(2)]), None),
        "duplicate_start": (single("HgH", [run(0), run(0), run(2)]), None),
        "descending_start": (single("HgHg", [run(0), run(2), run(1)]), None),
        "empty_tail": (single("Hg", [run(0), run(2)]), None),
        "past_tail": (single("Hg", [run(0), run(65535)]), None),
        "ambiguous_utf8": (single("AéHN", [run(0), run(3)]), None),
        "split_utf8_scalar": (single("AéH", [run(0), run(2)]), None),
        "late_invalid_utf8": (single(b"Hg\xff", [run(0), run(1)]), None),
        "late_font_ordinal": (
            single("Hg", [run(0), run(1, font=7)], fonts=[*ROBOTO, (7, 65001, b"Roboto")]),
            None,
        ),
        "late_charset": (
            single("Hg", [run(0), run(1, font=1)], fonts=[*ROBOTO, (1, 1252, b"Roboto")]),
            None,
        ),
        "late_face": (single("Hg", [run(0), run(1, face=1)]), None),
        "late_ink": (single("Hg", [run(0), run(1, ink=4)], palette=[WHITE]), None),
        "late_em": (single("Hg", [run(0), run(1, size=159)]), None),
        "pair_kern": (single("AV", [run(0), run(1)]), None),
        "space_pair_kern": (single("V To", [run(0), run(2)]), None),
        "ligature_fi": (single("fi", [run(0), run(1)]), None),
        "ligature_fl": (single("fl", [run(0), run(1)]), None),
        "ligature_ff": (single("ff", [run(0), run(1)]), None),
        "ligature_st": (single("st", [run(0), run(1)]), None),
        "run_limit": (single("H" * 4097, [run(i) for i in range(4097)]), None),
    }


def source_runs(data):
    records = RESOURCE.read(data)["records"]
    result = []
    for rec in records:
        if rec["kind"] != "property" or rec["tag"] != 0x0700:
            continue
        start, end = rec["payload_range"]
        count, = struct.unpack_from("<H", data, start)
        body_at = start + 2 + 10 * count
        assert body_at <= end
        words = list(struct.iter_unpack("<5H", data[start + 2:body_at]))
        result.append({
            "owner_id": records[rec["parent"]]["id"],
            "count": count,
            "words": words if count < 32 else [words[0], words[-1]],
            "body_sha256": sha256(data[body_at:end]).hexdigest(),
        })
    return result


def font_facts(path):
    assert fontTools.__version__ == "4.61.1"
    assert sha256(path.read_bytes()).hexdigest() == FONT_SHA256
    font = TTFont(path)
    cmap = font.getBestCmap()
    ascii_glyphs = {cmap[code]: chr(code) for code in range(32, 127)}
    marks = font["GDEF"].table.GlyphClassDef.classDefs
    glyph_chars = {glyph: chr(code) for code, glyph in cmap.items() if marks.get(glyph) != 3}
    ligatures = set()
    for lookup in font["GSUB"].table.LookupList.Lookup:
        for sub in lookup.SubTable:
            for head, choices in getattr(sub, "ligatures", {}).items():
                for choice in choices:
                    sequence = [head, *choice.Component]
                    for i, (left, right) in enumerate(zip(sequence, sequence[1:])):
                        if left in ascii_glyphs and right in glyph_chars:
                            # Every such join in this font starts its ligature.
                            assert i == 0
                            ligatures.add(ascii_glyphs[left] + glyph_chars[right])
    assert ligatures == {"ff", "fi", "fl", "st"}, ligatures

    gpos = font["GPOS"].table
    system = next(r.Script.DefaultLangSys for r in gpos.ScriptList.ScriptRecord if r.ScriptTag == "DFLT")
    indices = [
        lookup for index in system.FeatureIndex
        if gpos.FeatureList.FeatureRecord[index].FeatureTag == "kern"
        for lookup in gpos.FeatureList.FeatureRecord[index].Feature.LookupListIndex
    ]
    kern = {}
    for pair in ("AV", "V ", " H", " T", " λ", "Hg", " g", "dl"):
        first, second = (cmap[ord(c)] for c in pair)
        amount = 0
        for index in indices:
            lookup = gpos.LookupList.Lookup[index]
            assert lookup.LookupType == 2 and lookup.LookupFlag == 0
            for sub in lookup.SubTable:
                if first not in sub.Coverage.glyphs:
                    continue
                if sub.Format == 1:
                    records = sub.PairSet[sub.Coverage.glyphs.index(first)].PairValueRecord
                    value = next((r.Value1 for r in records if r.SecondGlyph == second), None)
                    if value is None:
                        continue
                else:
                    assert sub.Format == 2
                    a, b = sub.ClassDef1.classDefs.get(first, 0), sub.ClassDef2.classDefs.get(second, 0)
                    value = sub.Class1Record[a].Class2Record[b].Value1
                amount += value.XAdvance
                break
        kern[pair] = amount
    assert kern == {
        "AV": -87, "V ": 0, " H": 0, " T": -40, " λ": 0, "Hg": 0, " g": 0, "dl": 0,
    }, kern
    return {"sha256": FONT_SHA256, "ascii_ligature_joins": sorted(ligatures), "pair_units": kern}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--font", type=Path, required=True)
    parser.add_argument("--baseline-cli", type=Path)
    args = parser.parse_args()
    facts = font_facts(args.font)
    args.output_dir.mkdir(parents=True, exist_ok=False)
    cases = {}
    for name, (data, expected) in vectors().items():
        file = "../two_runs.cdx" if name == "legacy_two_runs" else name + ".cdx"
        path = args.output_dir / file
        if name == "legacy_two_runs":
            path = HERE / "two_runs.cdx"
        else:
            path.write_bytes(data)
        cases[name] = {
            "file": file, "sha256": sha256(data).hexdigest(), "model": expected,
            "source_runs": source_runs(data),
        }
        if args.baseline_cli is not None:
            cases[name]["legacy"] = legacy_open(args.baseline_cli, path.resolve())
            assert path.read_bytes() == data
    (args.output_dir / "cases.json").write_text(
        json.dumps({"font": facts, "cases": cases}, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )


if __name__ == "__main__":
    main()
