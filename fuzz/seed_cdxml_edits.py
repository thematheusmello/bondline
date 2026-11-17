#!/usr/bin/env python3
"""Original syntax-only seeds. Run later with one NEW output directory; never overwrite."""
import argparse
import hashlib
import json
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument("output", type=Path)
args = parser.parse_args()
args.output.mkdir(parents=True, exist_ok=False)
corpus = args.output / "corpus"
corpus.mkdir()
seeds = {
    "empty": b"",
    "selfclosing": b"<CDXML/>",
    "mixed-quotes": b"<?xml version='1.0'?>\r\n<CDXML\tkeep = '00255.00'\nother=\"a&amp;b\" />\r\n",
    "opaque-namespaces": b"<CDXML xmlns:f='urn:bondline:original-fuzz'><f:data f:p='raw'><![CDATA[x < y & z]]></f:data></CDXML>",
    "comments-pi": b"<!--original seed--><?bondline seed?><CDXML><!--child--><x/><?bondline child?></CDXML>\n",
    "nested-paired": b"<CDXML keep='1'><x p='1 2'><y/></x><z></z>tail</CDXML>",
    "namespaced-parent": b"<CDXML xmlns:f='urn:parent'><f:x><f:y/></f:x></CDXML>",
    "nested-append-offset": b"<CDXML><owner/></CDXML>",
    "default-namespace-owner": b"<CDXML><scope xmlns='urn:destination'><old/></scope></CDXML>",
    "fragment-default-reset": b"<plain xmlns=''/>",
    "fragment-default-declared": b"<plain xmlns='urn:explicit'/>",
    "value-escaping": "μ & < > \" ' \t\r\n".encode("utf-8"),
    "value-invalid-character": b"bad\x00value",
    "invalid-utf8": b"\xff\x80",
    "fragment-siblings": b"<x a='1'/><y/>",
    "fragment-selfcontained": b"<f:x xmlns:f='urn:fragment'><f:y/></f:x>",
    "fragment-comment": b"<!-- <!DOCTYPE not-markup> --><?bondline original?>",
    "fragment-malformed": b"<x>",
    "fragment-declaration": b"<?xml version='1.0'?><x/>",
    "fragment-doctype": b"<!DOCTYPE x><x/>",
    "fragment-parent-escape": b"</CDXML><outside/><CDXML>",
    "fragment-inherited-only": b"<f:child/>",
    "attribute-limit": ("<CDXML " + " ".join(f"a{i}='v'" for i in range(256)) + "/>").encode("ascii"),
    "deep-document": b"<CDXML>" + b"<x>" * 255 + b"</x>" * 255 + b"</CDXML>",
    "value-boundary-16384": b"A" * (16 * 1024),
}
manifest = []
for name, value in seeds.items():
    assert len(value) <= 16 * 1024
    path = corpus / name
    with path.open("xb") as stream:
        stream.write(value)
    manifest.append({"name": name, "bytes": len(value), "sha256": hashlib.sha256(value).hexdigest()})
with (args.output / "manifest.json").open("x", encoding="utf-8") as stream:
    json.dump({"provenance": "Original Bondline syntax-only declarations; no external fixtures/assets", "seeds": manifest}, stream, indent=2)
    stream.write("\n")
print(json.dumps({"seed_count": len(seeds), "max_seed_bytes": max(map(len, seeds.values())), "corpus": str(corpus)}))
