#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Original connected-label sources; never mutate the immutable source directory."""
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
fixtures = Path(__file__).resolve().parents[1]/'crates/io/tests/fixtures/connected-labels'
seeds = {path.name: path.read_text() for path in sorted(fixtures.glob('*.cdxml'))}
base = seeds['water.cdxml']
for name, old, new in [
    ('no-margin', " MarginWidth='2.000'", ''),
    ('zero-margin', "MarginWidth='2.000'", "MarginWidth='0'"),
    ('signed-zero', "MarginWidth='2.000'", "MarginWidth='-0.000'"),
    ('fraction-margin', "MarginWidth='2.000'", "MarginWidth='.125'"),
    ('huge-margin', "MarginWidth='2.000'", "MarginWidth='100000'"),
    ('negative-margin', "MarginWidth='2.000'", "MarginWidth='-1'"),
    ('underflow-margin', "MarginWidth='2.000'", "MarginWidth='-1e-9999'"),
    ('no-width', " LineWidth='1.50'", ''),
    ('local-margin', "<b id='900'", "<b MarginWidth='2' id='900'"),
    ('local-width', "<b id='900'", "<b LineWidth='1.5' id='900'"),
    ('explicit-solid', "<b id='900'", "<b Display='Solid' Display2='Solid' id='900'"),
    ('red-default', "color r='0' g='0' b='0'", "color r='1' g='0' b='0'"),
    ('unknown-root', '<CDXML ', "<CDXML Future='retain' "),
    ('namespace', '<CDXML ', "<CDXML xmlns:q='urn:original' q:MarginWidth='2' "),
    ('child-position', "<t id='21'", "<t p='160 160' id='21'"),
    ('unknown-font', "name='Roboto'", "name='Future Font'"),
    ('wrong-body', '>O</s>', '>NH</s>'),
]:
    assert old in base, name
    seeds[name+'.xml'] = base.replace(old, new, 1)
double = seeds['carbon_dioxide_centered.cdxml']
for name, old in [('double-no-length', " BondLength='40.00'"),
                  ('double-no-spacing', " BondSpacing='15.0'"),
                  ('double-no-center', " DoublePosition='Center'")]:
    assert old in double, name
    seeds[name+'.xml'] = double.replace(old, '', 1)
seeds['exact16k.xml'] = base.replace('</CDXML>', '<!--'+'x'*(16384-len(base.encode())-7)+'--></CDXML>', 1)
manifest = []
for name, text in seeds.items():
    raw = text.encode()
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
