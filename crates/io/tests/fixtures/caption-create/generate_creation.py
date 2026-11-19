# SPDX-License-Identifier: MIT OR Apache-2.0
"""Generate original native caption creation bytes, Expat identity and FontTools ink."""

import argparse
from hashlib import sha256
import json
from pathlib import Path
import sys
from xml.etree.ElementTree import TreeBuilder
from xml.parsers import expat
from xml.sax.saxutils import escape

HERE = Path(__file__).resolve().parent
sys.path[:0] = [str(HERE.parent / "cdx-caption-edit"), str(HERE.parent / "cdx-node-labels")]
from generate_caption_edits import FONT, font_facts, ink_facts
from generate_labels import atom

HEADER = '<?xml version="1.0" encoding="UTF-8"?><CDXML CreationProgram="Bondline 0.1.0">'
PAGE = '<page id="1" DrawingSpace="Poster" Width="512" Height="512">'
END = "</page></CDXML>"
COLOR_BASE = [(1, 1, 1), (0, 0, 0)]
COLOR_MIXED = [*COLOR_BASE, (0.2, 0.4, 0.8)]
TEXT = "Phase β & q"
POINT = (144.25, 216.75)
EXISTING_POINT = (80.75, 124.25)
PRESET = [(TEXT, 12, 3)]
EXISTING = [("Sample ", 14, 3), ("λ", 18, 4)]
COMMENT = "<!-- independent lexical variation -->"
POSITIONS = [(100, 200), (140, 200)]
CASES = [
    ("first", "blank", "first"),
    ("after_undo", "blank", "after_undo"),
    ("mixed_append", "mixed_before", "mixed_after"),
]


def write_palette(colors):
    return (
        '<fonttable><font id="0" name="Roboto"/></fonttable><colortable>'
        + "".join(f'<color r="{r}" g="{g}" b="{b}"/>' for r, g, b in colors)
        + "</colortable>"
    )


def write_text(identity, point, runs):
    x, y = point
    return (
        f'<t id="{identity}" p="{x} {y}" Justification="Left">'
        + "".join(f'<s font="0" size="{size}" face="0" color="{color}">{escape(body)}</s>'
                  for body, size, color in runs)
        + "</t>"
    )


def write_fragment():
    atoms = "".join(
        f'<n id="{i}" NodeType="Element" Element="6" Charge="0" Radical="None"'
        f' p="{x} {y}" NumHydrogens="3"/>'
        for i, (x, y) in enumerate(POSITIONS, 3)
    )
    return f'<fragment id="2">{atoms}<b id="5" B="3" E="4" Order="1"/></fragment>'


def model_caption(identity, point, runs, colors):
    return {
        "id": identity, "position": dict(zip(("x_pt", "y_pt"), point, strict=True)),
        "runs": [
            {"text": body, "style": {"font_family": "Roboto", "size_pt": float(size),
                                     "face": 0, "rgb": list(map(float, colors[color - 2]))}}
            for body, size, color in runs
        ],
    }


def model(*captions, mixed=False):
    fragments = []
    if mixed:
        fragments.append({
            "id": 1,
            "atoms": [atom(i, None, element=6, pos=pos, h=3)
                      for i, pos in enumerate(POSITIONS, 1)],
            "bonds": [{"id": 1, "begin": 1, "end": 2, "order": "single"}],
        })
    page = {"id": 1, "size": {"width_pt": 512.0, "height_pt": 512.0}, "fragments": fragments}
    if captions:
        page["texts"] = list(captions)
    return {"pages": [page]}


def xml_witness(raw, expected):
    tree = TreeBuilder()
    parser = expat.ParserCreate()
    parser.StartElementHandler = tree.start
    parser.EndElementHandler = tree.end
    parser.CharacterDataHandler = tree.data
    comments = []
    parser.CommentHandler = comments.append
    parser.Parse(raw, True)
    root = tree.close()
    page, = root.findall("page")
    expected_page, = expected["pages"]
    assert root.tag == "CDXML" and root.attrib == {"CreationProgram": "Bondline 0.1.0"}
    assert page.attrib == {"id": "1", "DrawingSpace": "Poster", "Width": "512", "Height": "512"}
    texts = list(page.findall("t"))
    expected_texts = expected_page.get("texts", [])
    assert [node.tag for node in root] == (["fonttable", "colortable", "page"] if texts else ["page"])
    prefix = []
    for node in page.iter():
        if node.tag in ("page", "fragment", "n", "b"):
            prefix.append((node.tag, int(node.attrib["id"])))
    assert [value for _, value in prefix] == list(range(1, len(prefix) + 1))
    if expected_page["fragments"]:
        fragment, = page.findall("fragment")
        assert [node.tag for node in page] == ["fragment", *(["t"] * len(texts))]
        parsed_atoms = fragment.findall("n")
        assert [a.attrib for a in parsed_atoms] == [
            {"id": str(i), "NodeType": "Element", "Element": "6", "Charge": "0",
             "Radical": "None", "p": f"{x} {y}", "NumHydrogens": "3"}
            for i, (x, y) in enumerate(POSITIONS, 3)
        ]
        bond, = fragment.findall("b")
        assert bond.attrib == {"id": "5", "B": "3", "E": "4", "Order": "1"}
    else:
        assert [node.tag for node in page] == ["t"] * len(texts)

    fonts = {f.attrib["id"]: f.attrib["name"] for f in root.findall("fonttable/font")}
    colors = [[float(c.attrib[ch]) for ch in ("r", "g", "b")]
              for c in root.findall("colortable/color")]
    assert fonts == ({"0": "Roboto"} if texts else {})
    expected_colors = COLOR_MIXED if expected_page["fragments"] else COLOR_BASE
    assert colors == ([list(c) for c in expected_colors] if texts else [])
    witnessed = []
    for text, expected_text in zip(texts, expected_texts, strict=True):
        wire_id = int(text.attrib["id"])
        assert wire_id == len(prefix) + expected_text["id"]
        x, y = map(float, text.attrib["p"].split())
        assert {"x_pt": x, "y_pt": y} == expected_text["position"]
        assert text.attrib["Justification"] == "Left"
        for run, expectation in zip(text, expected_text["runs"], strict=True):
            assert run.tag == "s" and len(run) == 0
            assert run.text == expectation["text"]
            attrs = run.attrib
            assert {
                "font_family": fonts[attrs["font"]],
                "size_pt": float(attrs["size"]),
                "face": int(attrs["face"]),
                "rgb": colors[int(attrs["color"]) - 2],
            } == expectation["style"]
        witnessed.append({"xml_id": wire_id, "text_id": expected_text["id"],
                          "p": text.attrib["p"], "styles": [node.attrib for node in text]})
    return {"sha256": sha256(raw).hexdigest(), "byte_length": len(raw),
            "xml_prefix": prefix, "texts": witnessed, "comments": comments}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", required=True, type=Path)
    output = parser.parse_args().output_dir
    output.mkdir(parents=True, exist_ok=False)
    preset1 = model_caption(1, POINT, PRESET, COLOR_BASE)
    preset2 = model_caption(2, POINT, PRESET, COLOR_BASE)
    old = model_caption(1, EXISTING_POINT, EXISTING, COLOR_MIXED)
    mixed_prefix = (
        HEADER + write_palette(COLOR_MIXED) + PAGE + write_fragment()
        + write_text(6, EXISTING_POINT, EXISTING)
    )
    sources = {
        "blank": (HEADER + PAGE + END, model()),
        "first": (HEADER + write_palette(COLOR_BASE) + PAGE
                  + write_text(2, POINT, PRESET) + END, model(preset1)),
        "after_undo": (HEADER + write_palette(COLOR_BASE) + PAGE
                       + write_text(3, POINT, PRESET) + END, model(preset2)),
        "mixed_before": (mixed_prefix + END, model(old, mixed=True)),
        "mixed_after": (mixed_prefix + write_text(7, POINT, PRESET) + END,
                        model(old, preset2, mixed=True)),
        "mixed_noncanonical": (mixed_prefix.replace(PAGE, COMMENT + PAGE, 1) + END,
                               model(old, mixed=True)),
    }
    witnesses = {}
    for name, (xml, expected) in sources.items():
        raw = xml.encode("utf-8")
        (output / (name + ".cdxml")).write_bytes(raw)
        witnesses[name] = xml_witness(raw, expected)
    assert sources["first"][0].replace('<t id="2"', '<t id="3"', 1) == sources["after_undo"][0]
    assert sources["mixed_noncanonical"][0].replace(COMMENT, "", 1) == sources["mixed_before"][0]
    assert sources["mixed_after"][0].startswith(sources["mixed_before"][0].removesuffix(END))

    facts = font_facts(FONT)
    ligatures = set(facts["ascii_ligature_joins"])
    params = {"page_id": 1, "x_pt": POINT[0], "y_pt": POINT[1], "text": TEXT}
    manifest = {"font_sha256": facts["sha256"], "cases": {}}
    for name, before, after in CASES:
        before_model, after_model = sources[before][1], sources[after][1]
        manifest["cases"][name] = {
            "before": before + ".cdxml",
            "after": after + ".cdxml",
            "params": params,
            "before_model": before_model,
            "after_model": after_model,
            "source": {"before": witnesses[before], "after": witnesses[after]},
            "after_ink": ink_facts(after_model, ligatures),
        }
    manifest["retained"] = {"mixed_noncanonical": {
        "file": "mixed_noncanonical.cdxml", "model": sources["mixed_noncanonical"][1],
        "source": witnesses["mixed_noncanonical"],
    }}
    (output / "cases.json").write_bytes(
        (json.dumps(manifest, ensure_ascii=False, indent=2) + "\n").encode("utf-8")
    )
    print("3 caption creation witnesses and 1 lexical refusal checked with Expat and FontTools")


if __name__ == "__main__":
    main()
