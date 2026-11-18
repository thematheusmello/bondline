#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Original CDX literal data and hostile variants, immutable outside writable corpus."""
import argparse
import hashlib
import json
import struct
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument('destination', type=Path)
root = parser.parse_args().destination
root.mkdir(parents=True, exist_ok=False)
(root/'source').mkdir()
(root/'corpus').mkdir()
repository = Path(__file__).resolve().parents[1]
fixtures = repository/'crates/io/tests/fixtures/cdx-literal-metadata'
seeds = {p.name: p.read_bytes() for p in sorted(fixtures.glob('*.cdx'))}
assert len(seeds) == 18
for name in ['unknown-implicit.cdx', 'original-ethane-explicit.cdx', 'original-ethane-header-document.cdx']:
    seeds['legacy-'+name] = (repository/'crates/cdx/tests/fixtures'/name).read_bytes()
base = seeds['methylammonium_explicit.cdx']
facts = json.loads((fixtures/'literal-facts.json').read_text())['methylammonium_explicit']
records = facts['records']
node = next(i for i, r in enumerate(records) if r.get('source_id') == 73)

def prop(tag, payload):
    count = len(payload)
    size = struct.pack('<HI',65535,count) if count >= 65535 else struct.pack('<H',count)
    return struct.pack('<H',tag)+size+payload

def field(tag):
    return next(r for r in records if r['kind']=='property' and r.get('parent')==node and r['tag']==tag)

for name, tag, payload in [
    ('negative-isotope',0x0420,b'\xff\xff'),
    ('short-isotope',0x0420,b'\x0d'),
    ('wide-isotope',0x0420,b'\x0d\0\0\0'),
    ('wide-charge',0x0421,b'\x01\0'),
    ('invalid-radical',0x0422,b'\x04'),
    ('wide-radical',0x0422,b'\x02\0'),
    ('short-H',0x042b,b'\x03'),
    ('wide-H',0x042b,b'\x03\0\0\0'),
    ('unspecified-node',0x0400,b'\0\0'),
]:
    r = field(tag)
    seeds[name+'.cdx'] = base[:r['start']]+prop(tag,payload)+base[r['end']:]
for tag in [0x0420,0x0421,0x0422,0x042b]:
    r = field(tag)
    payload = bytes.fromhex(r['payload_hex'])
    seeds[f'duplicate-{tag:04x}.cdx'] = base[:r['start']]+prop(tag,payload)+base[r['start']:]
for name, owner, tag, payload in [
    ('root',0,0x0420,b'\x0d\0'),
    ('page',90,0x0421,b'\xff'),
    ('fragment',51,0x0422,b'\x02'),
    ('bond',900,0x042b,b'\x03\0'),
    ('custom',400000001,0x0420,b'\x0d\0'),
]:
    r = next(r for r in records if r['kind']=='start' and r['source_id']==owner)
    seeds['wrong-owner-'+name+'.cdx'] = base[:r['end']]+prop(tag,payload)+base[r['end']:]
page = next(r for r in records if r['kind']=='start' and r['source_id']==90)
group = struct.pack('<HI',0x8002,999)+prop(0x0421,b'\xff')+bytes(2)
seeds['wrong-owner-group.cdx'] = base[:page['end']]+group+base[page['end']:]
seeds['truncated-tag.cdx'] = base[:-1]
seeds['truncated-payload.cdx'] = base[:field(0x0420)['payload_end']-1]
r = next(r for r in records if r['kind']=='property' and r['tag']==0x4000)
count = 131072-(len(base)-(r['end']-r['start']))-8
payload = bytes((i*17+11)%256 for i in range(count))
seeds['exact128k.cdx'] = base[:r['start']]+prop(0x4000,payload)+base[r['end']:]
assert len(seeds['exact128k.cdx']) == 131072
manifest = []
for name, raw in seeds.items():
    assert 0 < len(raw) <= 131072
    for directory in ['source','corpus']:
        (root/directory/name).write_bytes(raw)
    manifest.append({'name':name,'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()})
path = root/'manifest.json'
path.write_text(json.dumps(manifest,indent=2)+'\n')
print(json.dumps({'sources':len(manifest),'max_seed':max(e['bytes'] for e in manifest),'manifest_sha256':hashlib.sha256(path.read_bytes()).hexdigest()}))
