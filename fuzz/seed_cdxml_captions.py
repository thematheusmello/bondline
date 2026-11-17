#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Original bounded caption seeds; requires a NEW output directory. Stdlib only."""
import argparse
import hashlib
import json
from pathlib import Path

LIMIT = 16 * 1024
HEADER = "<CDXML CreationProgram='Original caption fuzz'><fonttable><font id='42' name='Roboto'/></fonttable><colortable><color r='1' g='1' b='1'/><color r='0' g='0' b='0'/><color r='.2' g='.4' b='.8'/></colortable><page id='90' DrawingSpace='Poster' Width='256.00' Height='256'><t id='21' p='72.00 96.000' Justification='Left'>"
FIRST = "<s font='42' size='12.00' face='0' color='4'>Phase β &amp; A</s>"
SECOND = "<s font='42' size='18' face='0' color='3'> q</s>"
FOOTER = "</t></page></CDXML>"
CAPTION = HEADER + FIRST + SECOND + FOOTER

# Exact standalone writer spelling for the current0.1.0 native caption profile.
# These are original declarations, not exports from a reference application.
NATIVE = '<?xml version="1.0" encoding="UTF-8"?><CDXML CreationProgram="Bondline 0.1.0"><fonttable><font id="0" name="Roboto"/></fonttable><colortable><color r="1" g="1" b="1"/><color r="0" g="0" b="0"/></colortable><page id="1" DrawingSpace="Poster" Width="256" Height="256"><t id="2" p="72 96" Justification="Left"><s font="0" size="12" face="0" color="3"></s></t></page></CDXML>'

entries = {}


def add(name, text, intent):
    data = text.encode("utf-8") if isinstance(text, str) else text
    assert name not in entries and len(data) <= LIMIT
    entries[name] = (data, intent)


add("explicit-two-run-unicode.xml", CAPTION, "caption text-only capability; global chemical authority false")
add("merged-cdata.xml", CAPTION.replace("Phase β &amp; A", "Phase β <![CDATA[& A]]>"), "decoded caption character data; owned leaf may be replaced")
add("empty-selfclosed.xml", CAPTION.replace(FIRST, "<s font='42' size='12.00' face='0' color='4'/>"), "empty first run, preserving style")
add("spaces.xml", CAPTION.replace("Phase β &amp; A", "  &#x9; &#xD;&#xA;  "), "line/space characters are model data; later renderer may refuse")
add("empty-paired.xml", CAPTION.replace("Phase β &amp; A", ""), "empty paired first run")
add("native-empty.xml", NATIVE, "known native full-model encoder spelling")
add("native-selfclosed-transition.xml", NATIVE.replace("></s>", "/>"), "caption may upgrade to Canonical after a proven text splice")
add("native-unicode.xml", NATIVE.replace("></s>", ">Phase β &amp; A</s>"), "known native Unicode run, no text loss")
add("missing-font-table.xml", CAPTION.replace("<fonttable><font id='42' name='Roboto'/></fonttable>", ""), "opaque: unresolved font")
add("unknown-font.xml", CAPTION.replace("name='Roboto'", "name='Other'"), "opaque: unknown font is retained")
add("font-alias.xml", CAPTION.replace("name='Roboto'", "name='Roboto Regular'"), "opaque: no invented font alias")
add("charset-override.xml", CAPTION.replace("name='Roboto'", "name='Roboto' charset='unknown'"), "opaque: extra resource data")
for field, token in [
    ("font", " font='42'"),
    ("size", " size='12.00'"),
    ("face", " face='0'"),
    ("color", " color='4'"),
    ("position", " p='72.00 96.000'"),
    ("justification", " Justification='Left'"),
]:
    add(f"missing-{field}.xml", CAPTION.replace(token, "", 1), "opaque: required caption/style field absent")
add("unsupported-face.xml", CAPTION.replace("face='0'", "face='1'", 1), "opaque: unsupported face remains source data")
add("unknown-run-field.xml", CAPTION.replace("size='12.00'", "size='12.00' Unknown='retained'"), "opaque: unknown run attribute")
add("unknown-root-field.xml", CAPTION.replace("<CDXML ", "<CDXML Unknown='retained' ", 1), "opaque: unknown root attribute")
add("unknown-page-field.xml", CAPTION.replace("<page ", "<page Unknown='retained' ", 1), "opaque: unknown page attribute")
add("negative-underflow-color.xml", CAPTION.replace("r='.2'", "r='-1e-9999'"), "opaque: negative nonzero source magnitude must not become admitted zero")
add("negative-underflow-size.xml", CAPTION.replace("size='12.00'", "size='-1e-9999'"), "opaque: nonpositive decoded text size")
add("mixed-chemical.xml", CAPTION.replace("</page>", "<fragment id='22'><n id='23' NodeType='Element' Element='6' Charge='0' Radical='None' p='192 192' NumHydrogens='4'/></fragment></page>"), "opaque: mixed foreign chemical/caption source has no complete caption authority")
add("unknown-object.xml", CAPTION.replace("</page>", "<graphic id='22' Type='unsupported'/></page>"), "opaque: unknown visible source object")
add("mixed-run-child.xml", CAPTION.replace("Phase β &amp; A", "Phase <x/> A"), "opaque: mixed owned leaf content is refused")
add("comment-in-run.xml", CAPTION.replace("Phase β &amp; A", "Phase <!--retained--> A"), "opaque: comments are not owned run character data")
add("comment-in-text.xml", CAPTION.replace(FIRST, "<!--retained-->" + FIRST), "opaque: Text mixed/comment content is refused")
add("namespaced-selector.xml", CAPTION.replace("<CDXML ", "<CDXML xmlns:f='urn:caption-fuzz' ", 1).replace(" color='4'", " f:color='4'", 1), "opaque: qualified selector is not an ordinary color attribute")
add("namespaced-trivia.xml", CAPTION.replace("<CDXML ", "<CDXML xmlns:f='urn:caption-fuzz' ", 1).replace("</page>", "<f:note>retained</f:note></page>"), "opaque: namespace/source coverage guard")
add("default-empty-declaration.xml", CAPTION.replace("<CDXML ", "<CDXML xmlns='' ", 1), "opaque: namespace declaration retains lexical guard")
add("value-escaping.txt", "β & < > \" ' \t\r\n", "arbitrary decoded Writer text/selector input")
add("value-invalid-character.bin", b"before\x00after", "Writer text/attribute refusal must be atomic")
add("value-invalid-utf8.bin", b"\xff\x80", "invalid UTF8 input")
for name, value in [
    ("zero", "0"), ("u16-max", "65535"), ("u16-overflow", "65536"),
    ("exact-integer", " 65535.00 "), ("fraction", "1.25"), ("nan", "NaN"),
]:
    add(f"selector-{name}.txt", value, "standalone exact selector/domain boundaries")
prefix = (HEADER + "<s font='42' size='12' face='0' color='3'>").encode("utf-8")
suffix = ("</s>" + FOOTER).encode("utf-8")
add("exact-16k.xml", prefix + b"A" * (LIMIT - len(prefix) - len(suffix)) + suffix, "IO/source/parser bound; renderer glyph coverage is separate")

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("output", type=Path)
args = parser.parse_args()
args.output.mkdir(parents=True, exist_ok=False)
source = args.output / "source"
corpus = args.output / "corpus"
source.mkdir()
corpus.mkdir()
manifest = {
    "license": "MIT OR Apache-2.0",
    "provenance": "Original inline Bondline caption declarations; no external files or assets",
    "input_limit_bytes": LIMIT,
    "files": [],
}
for name, (data, intent) in entries.items():
    for directory in [source, corpus]:
        with (directory / name).open("xb") as stream:
            stream.write(data)
    manifest["files"].append({"name": name, "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest(), "intent": intent})
with (args.output / "manifest.json").open("x", encoding="utf-8") as stream:
    json.dump(manifest, stream, ensure_ascii=False, indent=2)
    stream.write("\n")
print(json.dumps({"seed_count": len(entries), "max_seed_bytes": max(len(data) for data, _ in entries.values()), "corpus": str(corpus), "manifest": str(args.output / "manifest.json")}))
