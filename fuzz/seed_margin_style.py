#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Original margin-data corpus; immutable sources stay outside writable corpus."""
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
base = (Path(__file__).resolve().parents[1]/'crates/io/tests/fixtures/bondline-v0.cdxml').read_text()
seeds = {}
for name, value in [
    ('zero','0'), ('signed-zero','-0.000'), ('fraction','0.125'),
    ('fixed-quantum','0.0000152587890625'), ('max','100000'), ('negative','-1'),
    ('underflow','-1e-9999'), ('nan','NaN'), ('infinite','inf'),
    ('above-max','100001'), ('invalid','unknown'),
]:
    seeds[name+'.xml'] = base.replace('<CDXML ', "<CDXML MarginWidth='"+value+"' ", 1)
seeds['absent.xml'] = base
for name, owner in [('node','n'), ('bond','b'), ('page','page')]:
    seeds[name+'.xml'] = base.replace('<'+owner+' ', '<'+owner+" MarginWidth='2' ", 1)
seeds['qualified.xml'] = base.replace('<CDXML ', "<CDXML xmlns:q='urn:original' q:MarginWidth='2' ", 1)
seeds['unknown.xml'] = base.replace('<CDXML ', "<CDXML Future='retain' MarginWidth='2' ", 1)
seeds['exact16k.xml'] = base.replace('</CDXML>', '<!--'+'x'*(16384-len(base.encode())-7)+'--></CDXML>', 1)
manifest = []
for name, text in seeds.items():
    raw = text.encode()
    parser = expat.ParserCreate(namespace_separator='|')
    parser.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
    parser.ExternalEntityRefHandler = lambda *unused: 0
    parser.Parse(raw, True)
    assert 0 < len(raw) <= 16384
    for directory in ['source','corpus']:
        (root/directory/name).write_bytes(raw)
    manifest.append({'name':name, 'bytes':len(raw), 'sha256':hashlib.sha256(raw).hexdigest()})
p = root/'manifest.json'
p.write_text(json.dumps(manifest, indent=2)+'\n')
print(json.dumps({'sources':len(manifest), 'max_seed':max(x['bytes'] for x in manifest), 'manifest_sha256':hashlib.sha256(p.read_bytes()).hexdigest()}))
