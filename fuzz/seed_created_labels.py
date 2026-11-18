#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Original creation sources, immutable beside the writable mutation corpus."""
import argparse
import hashlib
import json
from pathlib import Path
import xml.parsers.expat as expat

parser = argparse.ArgumentParser()
parser.add_argument('destination', type=Path)
root = parser.parse_args().destination
root.mkdir(parents=True, exist_ok=False)
(root/'source').mkdir()
(root/'corpus').mkdir()
fixtures = Path(__file__).resolve().parents[1]/'crates/io/tests/fixtures/created-labels'
seeds = {p.name: p.read_bytes() for p in sorted(fixtures.glob('*.cdxml'))}
assert len(seeds) == 20
base = seeds['created_3_black.cdxml'].decode()
for name, old, new in [
    ('unknown-source', '<CDXML ', "<CDXML Future='retain' "),
    ('missing-active', " ShowNonTerminalCarbonLabels='no'", ''),
    ('active-true', "ShowNonTerminalCarbonLabels='no'", "ShowNonTerminalCarbonLabels='yes'"),
    ('missing-font', 'font="42"', 'font="999"'),
    ('unsupported-face', 'face="0"', 'face="1"'),
    ('body-drift', '>CH2</s>', '>NH2</s>'),
    ('object-collision', 'id="2" BoundingBox=', 'id="21" BoundingBox='),
]:
    assert old in base
    seeds[name+'.xml'] = base.replace(old,new,1).encode()
seeds['exact16k.xml'] = base.replace('</CDXML>', '<!--'+'x'*(16384-len(base.encode())-7)+'--></CDXML>',1).encode()
for selector in range(6):
    seeds[f'selector-{selector}.bin'] = bytes([selector])
manifest = []
for name, raw in seeds.items():
    if not name.endswith('.bin'):
        xml = expat.ParserCreate(namespace_separator='|')
        xml.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
        xml.ExternalEntityRefHandler = lambda *_: 0
        xml.Parse(raw,True)
    assert 0 < len(raw) <= 16384
    for directory in ['source','corpus']:
        (root/directory/name).write_bytes(raw)
    manifest.append({'name':name,'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()})
path = root/'manifest.json'
path.write_text(json.dumps(manifest,indent=2)+'\n')
print(json.dumps({'sources':len(manifest),'max_seed':max(e['bytes'] for e in manifest),'manifest_sha256':hashlib.sha256(path.read_bytes()).hexdigest()}))
