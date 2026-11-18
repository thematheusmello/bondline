#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Original mixed-source controls, with immutable sources outside writable corpus."""
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
fixtures = Path(__file__).resolve().parents[1]/'crates/io/tests/fixtures/mixed-labels'
seeds = {p.name: p.read_bytes() for p in sorted(fixtures.glob('*.cdxml'))}
base = seeds['amino_alcohol.cdxml'].decode()
for name, old, new in [
    ('missing-active', " ShowNonTerminalCarbonLabels='no'", ''),
    ('active-true', "ShowNonTerminalCarbonLabels='no'", "ShowNonTerminalCarbonLabels='yes'"),
    ('missing-h', "NumHydrogens='2' p='168 208'", "p='168 208'"),
    ('charged-hidden', "Element='6' Charge='0'", "Element='6' Charge='1'"),
    ('isotope-hidden', "Element='6' Charge='0'", "Element='6' Isotope='13' Charge='0'"),
    ('unknown-source', '<CDXML ', "<CDXML Future='keep' "),
    ('hidden-child', "p='168 208' ShowNonTerminalCarbonLabels='no'/>", "p='168 208' ShowNonTerminalCarbonLabels='no'><t id='6000' p='168 208'><s font='999'>C</s></t></n>"),
    ('nonblack-bonds', "<color r='0' g='0' b='0'/>", "<color r='.1' g='0' b='0'/>"),
]:
    assert old in base
    seeds[name+'.xml'] = base.replace(old,new,1).encode()
seeds['exact16k.xml'] = base.replace('</CDXML>', '<!--'+'x'*(16384-len(base.encode())-7)+'--></CDXML>',1).encode()
for selector in range(5):
    seeds[f'selector-{selector}.bin'] = bytes([selector])
manifest = []
for name,raw in seeds.items():
    if not name.endswith('.bin'):
        parser = expat.ParserCreate(namespace_separator='|')
        parser.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
        parser.ExternalEntityRefHandler = lambda *_: 0
        parser.Parse(raw,True)
    assert 0<len(raw)<=16384
    for directory in ['source','corpus']:
        (root/directory/name).write_bytes(raw)
    manifest.append({'name':name,'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()})
p=root/'manifest.json'
p.write_text(json.dumps(manifest,indent=2)+'\n')
print(json.dumps({'sources':len(manifest),'max_seed':max(e['bytes'] for e in manifest),'manifest_sha256':hashlib.sha256(p.read_bytes()).hexdigest()}))
