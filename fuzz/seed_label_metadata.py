#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Keep original metadata cases and hostile controls outside the writable corpus."""
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
fixtures = Path(__file__).resolve().parents[1]/'crates/io/tests/fixtures/label-metadata'
seeds = {p.name: p.read_bytes() for p in sorted(fixtures.glob('*.cdxml'))}
assert len(seeds) == 12
base = seeds['mixed_a.cdxml'].decode()
for name, old, new in [
    ('unknown-source', '<CDXML ', "<CDXML Future='retain' "),
    ('missing-run-font', "font='42'", ''),
    ('missing-run-size', "size='12'", ''),
    ('missing-run-face', "face='0'", ''),
    ('missing-run-color', "color='3'", ''),
    ('shadowed-negative-font', "LabelFont=' +&#55;.000 '", "LabelFont='-1'"),
    ('shadowed-unresolved-font', "LabelFont=' +&#55;.000 '", "LabelFont='999'"),
    ('negative-face', "LabelFace='96'", "LabelFace='-1'"),
    ('invalid-size', "LabelSize='10.00'", "LabelSize='0'"),
    ('wrong-owner', '<page ', "<page LabelFont='42' "),
    ('qualified-owner', '<CDXML ', "<CDXML xmlns:x='urn:unused' "),
    ('body-drift', '>OH</s>', '>NH2</s>'),
]:
    assert old in base, name
    seeds[name+'.xml'] = base.replace(old,new,1).encode()
seeds['exact16k.xml'] = base.replace('</CDXML>', '<!--'+'x'*(16384-len(base.encode())-7)+'--></CDXML>',1).encode()
for selector in range(5):
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
