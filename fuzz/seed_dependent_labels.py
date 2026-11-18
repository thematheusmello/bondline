#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Original dependent-label sources plus explicit structured-selector controls."""
import argparse
import hashlib
import json
from pathlib import Path
import xml.parsers.expat as expat

args = argparse.ArgumentParser()
args.add_argument('destination', type=Path)
root = args.parse_args().destination
root.mkdir(parents=True, exist_ok=False)
(root/'source').mkdir()
(root/'corpus').mkdir()
fixtures = Path(__file__).resolve().parents[1]/'crates/io/tests/fixtures/dependent-labels'
seeds = {p.name: p.read_bytes() for p in sorted(fixtures.glob('*.cdxml'))}
base = seeds['connected.cdxml'].decode()
for name, old, new in [
    ('missing-margin', " MarginWidth='2.000'", ''),
    ('unknown-source', '<CDXML ', "<CDXML Future='retain' "),
    ('qualified-point', '<CDXML ', "<CDXML xmlns:q='urn:original' q:p='160 160' "),
    ('foreign-child-p', "<t id='21'", "<t p='160 160' id='21'"),
]:
    assert old in base
    seeds[name+'.xml'] = base.replace(old, new, 1).encode()
seeds['exact16k.xml'] = base.replace('</CDXML>', '<!--'+'x'*(16384-len(base.encode())-7)+'--></CDXML>', 1).encode()
manifest = []
for selector in range(4):
    seeds[f'selector-{selector}.bin'] = bytes([selector])
for name, raw in seeds.items():
    if not name.endswith('.bin'):
        parser = expat.ParserCreate(namespace_separator='|')
        parser.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
        parser.ExternalEntityRefHandler = lambda *unused: 0
        parser.Parse(raw, True)
    assert 0 < len(raw) <= 16384
    for directory in ['source', 'corpus']:
        (root/directory/name).write_bytes(raw)
    manifest.append({'name': name, 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()})
p = root/'manifest.json'
p.write_text(json.dumps(manifest, indent=2)+'\n')
print(json.dumps({'sources': len(manifest), 'max_seed': max(x['bytes'] for x in manifest),
                  'manifest_sha256': hashlib.sha256(p.read_bytes()).hexdigest()}))
