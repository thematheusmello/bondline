#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Original no-p BoundingBox vectors; immutable source stays outside mutable corpus."""
import argparse
import hashlib
import json
from pathlib import Path
import xml.parsers.expat as expat

SOURCE = '<CDXML CreationProgram="Original boxed fuzz"><fonttable><font id="42" name="Roboto"/></fonttable><colortable><color r="1" g="1" b="1"/><color r="0" g="0" b="0"/><color r="0.2" g="0.4" b="0.8"/></colortable><page id="90" DrawingSpace="Poster" Width="512" Height="512"><fragment id="51"><n id="73" NodeType="Element" Element="8" Charge="0" Radical="None" NumHydrogens="1" p="84.1220703125 95.734375"><t id="21" BoundingBox="80.69140625 91.3515625 95.796875 100.1171875" Justification="Left" LabelJustification="Left"><s font="42" size="12" face="0" color="4">OH</s></t></n></fragment></page></CDXML>'
BOX = '80.69140625 91.3515625 95.796875 100.1171875'
LIMIT = 16384
entries = {}


def add(name, text, state):
    raw = text.encode('utf-8')
    assert name not in entries and 0 < len(raw) <= LIMIT
    parser = expat.ParserCreate(namespace_separator='|')
    parser.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
    parser.ExternalEntityRefHandler = lambda *unused: 0
    parser.Parse(raw, True)
    entries[name] = (raw, state)


add('explicit.xml', SOURCE, 'qualified literal OH box; no general chemistry authority')
add('edited.xml', SOURCE.replace('Element="8"', 'Element="7"').replace('NumHydrogens="1"', 'NumHydrogens="2"').replace(BOX, '80.84375 91.3515625 103.26171875 100').replace('>OH</s>', '>NH2</s>'), 'qualified numeric/body/box NH2 delta')
add('producer-absent.xml', SOURCE.replace(' CreationProgram="Original boxed fuzz"', ''), 'producer alone supplies no authority')
add('producer-other.xml', SOURCE.replace('Original boxed fuzz', 'Independent original data'), 'complete proof independent of producer')
add('integral-hex.xml', SOURCE.replace('Element="8"', 'Element="0x08"').replace('NumHydrogens="1"', 'NumHydrogens="01"').replace('font="42"', 'font="0x2a"'), 'typed integral domains, exact raw lexemes')
add('decoded.xml', SOURCE.replace('>OH</s>', '>&#79;H</s>'), 'decoded text agrees; raw entity preserved')
add('comment.xml', SOURCE.replace('<fragment ', '<!-- original nonsemantic comment --><fragment '), 'comment retained')
unlabeled = SOURCE.replace('</fragment>', '<n id="97" NodeType="Element" Element="7" Charge="0" Radical="None" NumHydrogens="0" p="220 220"/></fragment>')
add('unlabeled-node.xml', unlabeled, 'opaque foreign label defaults; no borrowed native glyph style')
add('producer-no-defaults.xml', unlabeled.replace('Original boxed fuzz', 'Bondline 0.1.0'), 'producer text cannot supply source-free/exact-native conventions')
second = '<n id="97" NodeType="Element" Element="8" Charge="0" Radical="None" NumHydrogens="1" p="184.1220703125 95.734375"><t id="21" BoundingBox="180.69140625 91.3515625 195.796875 100.1171875" Justification="Left" LabelJustification="Left"><s font="42" size="12" face="0" color="4">OH</s></t></n><b id="900" B="97" E="73" Order="1"/>'
add('boxed-bond.xml', SOURCE.replace('</fragment>', second+'</fragment>'), 'opaque foreign bond defaults even when every atom has explicit text')
for name, replacement in [
    ('wrong-width', '80.69140625 91.3515625 96.046875 100.1171875'),
    ('wrong-height', '80.69140625 91.3515625 95.796875 100.25'),
    ('reversed', '95.796875 91.3515625 80.69140625 100.1171875'),
    ('degenerate', '80 91 80 100'),
    ('nonfinite', 'NaN 91 95 100'),
    ('extra-number', BOX + ' 1'),
    ('missing-number', '80.69140625 91.3515625 95.796875'),
]:
    add(name+'.xml', SOURCE.replace(BOX, replacement), 'opaque unsupported box data/geometry')
for name, old, new in [
    ('explicit-p', '<t id="21"', '<t p="80 100" id="21"'),
    ('alignment', '<t id="21"', '<t LabelAlignment="Auto" id="21"'),
    ('body', '>OH</s>', '>NH2</s>'),
    ('wrong-h', 'NumHydrogens="1"', 'NumHydrogens="2"'),
    ('wrong-font', 'name="Roboto"', 'name="Other"'),
    ('unknown-font', 'font="42"', 'font="43"'),
    ('formula-face', 'face="0"', 'face="96"'),
    ('missing-face', ' face="0"', ''),
    ('missing-font', ' font="42"', ''),
    ('missing-size', ' size="12"', ''),
    ('missing-color', ' color="4"', ''),
    ('wrong-size', 'size="12"', 'size="24"'),
    ('unknown-color', 'color="4"', 'color="65535"'),
    ('missing-point', ' p="84.1220703125 95.734375"', ''),
    ('missing-box', ' BoundingBox="'+BOX+'"', ''),
    ('charge', 'Charge="0"', 'Charge="1"'),
    ('radical', 'Radical="None"', 'Radical="Doublet"'),
    ('unknown-owner-key', '<n id="73"', '<n Future="keep" id="73"'),
    ('unknown-run-key', '<s font="42"', '<s Future="keep" font="42"'),
    ('root-style-default', '<CDXML ', '<CDXML LabelFont="42" '),
    ('namespace', '<CDXML ', '<CDXML xmlns:q="urn:original" '),
    ('explicit-z', '<t id="21"', '<t Z="0" id="21"'),
]:
    add(name+'.xml', SOURCE.replace(old, new), 'raw retained; no unsupported style/default/geometry/source authority')
for name, prefix in [('doctype', '<!DOCTYPE CDXML>'), ('pi', '<?original vector?>')]:
    add(name+'.xml', prefix+SOURCE, 'syntax retained, closed source profile refused')
padding = '<!--' + 'x'*(LIMIT-len(SOURCE.encode())-7) + '-->'
add('exact16k.xml', SOURCE.replace('<fragment ', padding+'<fragment ', 1), 'original exact16KiB source, not reached-mutation evidence')

args = argparse.ArgumentParser()
args.add_argument('destination', type=Path)
root = args.parse_args().destination
root.mkdir(parents=True, exist_ok=False)
(root/'source').mkdir()
(root/'corpus').mkdir()
manifest = []
for name, (raw, state) in entries.items():
    (root/'source'/name).write_bytes(raw)
    (root/'corpus'/name).write_bytes(raw)
    manifest.append({'name':name,'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'declaration':state})
p = root/'manifest.json'
p.write_text(json.dumps(manifest, indent=2)+'\n')
print(json.dumps({'seeds':len(manifest),'max_bytes':max(x['bytes'] for x in manifest),'manifest_sha256':hashlib.sha256(p.read_bytes()).hexdigest()}))
