#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Original label-default lookup seeds. Requires a NEW output directory; stdlib only."""
import argparse
import hashlib
import json
from pathlib import Path

LIMIT = 16 * 1024
entries = {}


def add(name, source, intent):
    data = source.encode("utf-8") if isinstance(source, str) else source
    assert name not in entries and len(data) <= LIMIT
    entries[name] = (data, intent)


add("all-absent.xml", "<CDXML><page><fragment><n><t><s/></t></n></fragment></page></CDXML>", "absence remains unresolved; no Style-run filling")
add("independent-owners.xml", "<CDXML LabelFont='-1' LabelSize='10' LabelFace='96'><page><fragment><n LabelSize='14.5'><t LabelFace='0'><s>OH</s></t></n></fragment></page></CDXML>", "font/root, size/node, face/text with decoded-value provenance")
add("partial-font.xml", "<CDXML LabelFont='0'><n><t><s/></t></n></CDXML>", "present zero font; missing size/face stay absent")
add("partial-size.xml", "<CDXML LabelSize='.25'><n><t><s/></t></n></CDXML>", "positive fractional XML reader policy; no extra size maximum")
add("partial-face.xml", "<CDXML LabelFace='-32768'><n><t><s/></t></n></CDXML>", "signed face remains literal data")
add("decoded-provenance.xml", "<CDXML LabelFont=' &#45;1.00&#9;' LabelSize='&#x20;1.25&#xA;' LabelFace='&#43;0000.0'><n><t><s/></t></n></CDXML>", "decoded attribute values differ from lexical source references")
add("same-property-overrides.xml", "<CDXML LabelFont='1' LabelSize='10' LabelFace='96'><n LabelFont='-1' LabelSize='12' LabelFace='1'><t LabelFont='0' LabelSize='14.5' LabelFace='0'><s/></t></n></CDXML>", "nearest valid owner independently wins each property")
add("explicit-u16-run.xml", "<CDXML LabelFont='-1' LabelSize='24' LabelFace='-32768'><fonttable><font id='65535' name='Roboto'/></fonttable><n><t><s font='65535' size='12' face='65535' color='0'>OH</s></t></n></CDXML>", "signed defaults remain distinct from unsigned run/resource selectors")
add("qualified-lookalikes.xml", "<CDXML xmlns:q='urn:labels' LabelFont='-1' q:LabelFont='bad'><n q:LabelSize='bad'><t LabelSize='2.5' q:LabelSize='bad' q:LabelFace='bad'><s/></t></n></CDXML>", "qualified attributes are retained but do not supply ordinary defaults")
add("empty-namespace-reset.xml", "<CDXML xmlns:q='urn:labels' LabelFont='3'><q:n xmlns='urn:foreign'><t xmlns='' LabelSize='2.5'><s/></t></q:n></CDXML>", "empty namespace restores ordinary target; transit owners have no Label properties")
add("foreign-owner.xml", "<CDXML xmlns:q='urn:labels' LabelFont='3'><q:n LabelSize='12'><t><s/></t></q:n></CDXML>", "ordinary scalar on a namespace-qualified owner is unsupported")
add("foreign-default-owner.xml", "<CDXML LabelFont='3'><n xmlns='urn:foreign' LabelFont='4'><t xmlns=''><s/></t></n></CDXML>", "namespace-qualified owner cannot be skipped even with an ordinary descendant")
add("qualified-target.xml", "<CDXML xmlns:q='urn:labels'><q:n LabelFont='3'/></CDXML>", "qualified role is InvalidElement as target")
add("nested-fake-root.xml", "<CDXML LabelFont='3'><CDXML LabelFont='4'><t><s/></t></CDXML></CDXML>", "only the actual CDXML root has root-owner authority")
add("unknown-retained.xml", "<CDXML Unknown='keep' LabelFont='3'><!--untouched--><page><n data='μ &amp; β'><t><s><![CDATA[OH]]></s></t></n></page></CDXML>", "lookup changes no unknown data or source lexemes")
add("nearest-error.xml", "<CDXML LabelFont='bad'><n><t LabelSize='0'><s/></t></n></CDXML>", "nearest owner diagnosed before farther root")
add("property-error-order.xml", "<CDXML><n><t LabelFont='bad' LabelSize='0' LabelFace='bad'><s/></t></n></CDXML>", "font then size then face diagnostic order on one owner")
add("owner-before-value.xml", "<CDXML LabelFont='bad'><group LabelSize='bad'><n><t><s/></t></n></group></CDXML>", "unsupported nearer owner wins before parsing its value or farther root")

for role in ["n", "b", "t", "table", "spectrum", "constraint", "tlcplate"]:
    add(f"role-{role}.xml", f"<CDXML LabelFont='-1' LabelSize='12' LabelFace='0'><{role} LabelSize='14.5'/></CDXML>", "every admitted target/owner role has independent nearest defaults")
for owner in ["page", "group", "fragment", "geometry", "other"]:
    add(f"unsupported-{owner}.xml", f"<CDXML LabelFont='3'><{owner} LabelFace='0'><n><t><s/></t></n></{owner}></CDXML>", "ordinary Label scalar on unsupported containing owner refuses")
for property_name, valid, invalid in [("LabelFont", "-32768", "65535"), ("LabelSize", ".25", "-1e-9999"), ("LabelFace", "32767", "32768")]:
    add(f"shadowed-invalid-{property_name}.xml", f"<CDXML {property_name}='{invalid}'><n {property_name}='{valid}'><t><s/></t></n></CDXML>", "invalid shadowed legal owner must still refuse")
for name, value in [
    ("i16-min", "-32768"), ("i16-max", "32767"), ("i16-low-overflow", "-32769"),
    ("i16-high-overflow", "32768"), ("u16-max", "65535"), ("minus-one", "-1"),
    ("integer-lexeme", " +00012.00 "), ("fraction", "1.25"),
]:
    add(f"font-{name}.xml", f"<CDXML LabelFont='{value}'><n><t><s/></t></n></CDXML>", "signed INT16 boundaries and exact integer lexical domain")
for name, value in [
    ("fraction", ".25"), ("subnormal", "5e-324"), ("large-finite", "1e300"),
    ("zero", "0"), ("negative-zero", "-0"), ("negative-underflow", "-1e-9999"),
    ("positive-underflow", "1e-9999"), ("nan", "NaN"), ("infinity", "Infinity"),
    ("overflow", "1e309"),
]:
    add(f"size-{name}.xml", f"<CDXML LabelSize='{value}'><n><t><s/></t></n></CDXML>", "positive finite XML size reader policy only; raw source retained")
add("invalid-character.bin", b"before\x00after", "Writer refusal of XML-illegal decoded attribute characters")
add("invalid-utf8.bin", b"\xff\x80", "invalid source UTF8")
prefix = b"<CDXML LabelFont='-1' LabelSize='10' LabelFace='96'><n LabelSize='14.5'><t LabelFace='0'><s>"
suffix = b"</s></t></n></CDXML>"
add("exact-16k.xml", prefix + b"A" * (LIMIT - len(prefix) - len(suffix)) + suffix, "exact bounded source; lookup has no renderer or chemical authority")

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("output", type=Path)
args = parser.parse_args()
args.output.mkdir(parents=True, exist_ok=False)
source = args.output / "source"
corpus = args.output / "corpus"
source.mkdir()
corpus.mkdir()
manifest = {"license": "MIT OR Apache-2.0", "provenance": "Original inline Bondline label-default declarations; no external files/assets", "input_limit_bytes": LIMIT, "files": []}
for name, (data, intent) in entries.items():
    for directory in [source, corpus]:
        with (directory / name).open("xb") as stream:
            stream.write(data)
    manifest["files"].append({"name": name, "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest(), "intent": intent})
with (args.output / "manifest.json").open("x", encoding="utf-8") as stream:
    json.dump(manifest, stream, ensure_ascii=False, indent=2)
    stream.write("\n")
print(json.dumps({"seed_count": len(entries), "max_seed_bytes": max(len(data) for data, _ in entries.values()), "corpus": str(corpus), "manifest": str(args.output / "manifest.json")}))
