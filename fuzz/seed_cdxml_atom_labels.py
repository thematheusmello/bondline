#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Original bounded Node-label vectors. Stdlib only; destination must be NEW."""
import argparse
import hashlib
import json
from pathlib import Path
import xml.parsers.expat as expat

LIMIT = 16 * 1024
HEADER = "<CDXML CreationProgram='Original owned label fuzz'><fonttable><font id='42' name='Roboto'/><font id='7' name='Other'/></fonttable><colortable><color r='1' g='1' b='1'/><color r='0' g='0' b='0'/><color r='.2' g='.4' b='.8'/></colortable><page id='90' DrawingSpace='Poster' Width='256' Height='256'><fragment id='51'><n id='73' Element='8' NumHydrogens='1' p='72 96'>"
TEXT = "<t id='21' p='71.5 96.25' Justification='Left' LabelJustification='Auto' LabelAlignment='Right'><s font='42' size='12.5' face='96' color='4'>OH</s><s font='7' size='18' face='0' color='3'>β &amp; q</s></t>"
CAPTION = "<t id='84' p='72 144' Justification='Left'><s font='42' size='12' face='0' color='4'>Caption</s></t>"
TAIL = "</n></fragment>" + CAPTION + "</page></CDXML>"
SOURCE = HEADER + TEXT + TAIL
NATIVE = '<?xml version="1.0" encoding="UTF-8"?><CDXML CreationProgram="Bondline 0.1.0"><fonttable><font id="0" name="Roboto"/></fonttable><colortable><color r="1" g="1" b="1"/><color r="0" g="0" b="0"/><color r="0.2" g="0.4" b="0.8"/></colortable><page id="1" DrawingSpace="Poster" Width="256" Height="256"><fragment id="2"><n id="3" NodeType="Element" Element="8" Charge="0" Radical="None" p="72 96" NumHydrogens="1"><t id="4" p="71.5 96.25" Justification="Left" LabelJustification="Auto" LabelAlignment="Right"><s font="0" size="12.5" face="96" color="4">OH</s></t></n></fragment></page></CDXML>'
entries = {}


def add(name, raw, expectation, note):
    raw = raw.encode('utf-8') if isinstance(raw, str) else raw
    assert name not in entries and 0 < len(raw) <= LIMIT
    # Independent XML syntax/bound check; never use a Bondline parser to seed itself.
    parser = expat.ParserCreate(namespace_separator='|')
    parser.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
    parser.ExternalEntityRefHandler = lambda *unused: 0
    parser.Parse(raw, True)
    entries[name] = (raw, expectation, note)


add('explicit-mixed.xml', SOURCE, 'owned', 'Node OH and numeric O/H1 distinct, ordered two runs and Page caption; no display authority')
add('native-label-data.xml', NATIVE, 'owned_native_data_codec', 'Original declaration of exact native0.1.0 data encoding, never editing/display authority')
add('empty-label.xml', SOURCE.replace('>OH</s>', '></s>'), 'owned', 'Present empty first label run is not absent')
add('empty-selfclosed-label.xml', SOURCE.replace("<s font='42' size='12.5' face='96' color='4'>OH</s>", "<s font='42' size='12.5' face='96' color='4'/>"), 'owned', 'Empty first run retains style')
add('decoded-whitespace.xml', SOURCE.replace('>OH</s>', '>  &#x9;&#xD;&#xA; &lt;&amp; β</s>'), 'owned', 'Owned decoded character data without layout claim')
add('cdata-run.xml', SOURCE.replace('>OH</s>', '><![CDATA[OH]]></s>'), 'owned', 'Source CDATA remains raw while decoded data is owned')
placement = [('position', " p='71.5 96.25'"), ('justification', " Justification='Left'"), ('label-justification', " LabelJustification='Auto'"), ('label-alignment', " LabelAlignment='Right'")]
absent = SOURCE
for name, token in placement:
    add('missing-' + name + '.xml', SOURCE.replace(token, '', 1), 'owned', 'Exact placement absence retained, no default injected')
    absent = absent.replace(token, '', 1)
add('all-placement-absent.xml', absent, 'owned', 'No p/Justification/LabelJustification/LabelAlignment; no inferred baseline')
for prop, old, names in [('Justification', 'Left', ['Right','Left','Center','Full','Above','Below','Auto','Best']), ('LabelJustification', 'Auto', ['Right','Left','Center','Full','Above','Below','Auto','Best']), ('LabelAlignment', 'Right', ['Auto','Left','Center','Right','Above','Below','Best'])]:
    for name in names:
        add(prop.lower() + '-' + name.lower() + '.xml', SOURCE.replace(prop+"='"+old+"'", prop+"='"+name+"'", 1), 'owned', 'Independent enum names, never numeric code conflation')
    add(prop.lower() + '-unknown.xml', SOURCE.replace(prop+"='"+old+"'", prop+"='Unknown'", 1), 'opaque', 'Unsupported enum blocks label projection, raw source retained')
for field, token in [('font', " font='42'"), ('size', " size='12.5'"), ('face', " face='96'"), ('color', " color='4'")]:
    add('missing-run-' + field + '.xml', SOURCE.replace(token, '', 1), 'opaque', 'Explicit run field required; no inherited/default fallback')
add('unresolved-font.xml', SOURCE.replace("font='42'", "font='43'", 1), 'opaque', 'No implicit font substitution')
add('unresolved-color.xml', SOURCE.replace("color='4'", "color='65535'", 1), 'opaque', 'No implicit palette fallback')
add('malformed-point.xml', SOURCE.replace("p='71.5 96.25'", "p='71.5 96.25 1'", 1), 'opaque', 'Exactly two bounded finite point components required')
add('nonfinite-point.xml', SOURCE.replace("p='71.5 96.25'", "p='NaN 96.25'", 1), 'opaque', 'Nonfinite placement remains raw')
add('multiple-child-text.xml', SOURCE.replace('</t></n>', "</t><t id='22'><s font='42' size='12.5' face='96' color='4'>other</s></t></n>", 1), 'opaque', 'Ambiguous multiple direct child Text refuses ownership')
add('unknown-text-field.xml', SOURCE.replace("<t id='21'", "<t id='21' Future='keep &amp; raw'", 1), 'owned_with_opaque_source', 'Known fields may project, unknown bytes remain authoritative')
add('qualified-text-field.xml', SOURCE.replace("<t id='21'", "<t xmlns:x='urn:foreign' x:LabelAlignment='Full' id='21'", 1), 'owned_with_opaque_source', 'Qualified attribute does not replace ordinary LabelAlignment')
add('qualified-size-only.xml', SOURCE.replace("<s font='42' size='12.5'", "<s xmlns:x='urn:foreign' font='42' x:size='12.5'", 1), 'opaque', 'Qualified run size does not become ordinary style')
add('empty-namespace-child.xml', SOURCE.replace("<t id='21'", "<t xmlns='' id='21'", 1), 'owned_with_opaque_source', 'Explicit empty namespace is ordinary child Text')
add('empty-namespace-label-only.xml', SOURCE.replace("<t id='21'", "<t xmlns='' id='21'", 1).replace(" Element='8'", '', 1), 'mapping_refused', 'Text-only Node cannot default to carbon even with namespace reset')
add('foreign-child.xml', SOURCE.replace("<t id='21'", "<x:t xmlns:x='urn:foreign' id='21'", 1).replace('</t></n>', '</x:t></n>', 1), 'opaque', 'Foreign Text element not ordinary Node label')
add('native-empty-label.xml', NATIVE.replace('>OH</s>', '></s>'), 'owned_native_data_codec', 'Exact native empty label remains present')
pad = LIMIT - len(SOURCE.encode('utf-8')) - len('<!---->')
assert pad >= 0
max_seed = SOURCE.replace('</page>', '<!--' + 'x' * pad + '--></page>', 1)
assert len(max_seed.encode('utf-8')) == LIMIT
add('exact-16k.xml', max_seed, 'owned', 'Maximum original input size; not proof of mutated reached length')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('destination', type=Path)
    dest = parser.parse_args().destination
    dest.mkdir(parents=True, exist_ok=False)
    (dest/'source').mkdir()
    (dest/'corpus').mkdir()
    manifest = []
    for name, (raw, expected, note) in entries.items():
        (dest/'source'/name).write_bytes(raw)
        (dest/'corpus'/name).write_bytes(raw)
        manifest.append({'name': name, 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest(), 'expected': expected, 'note': note})
    (dest/'manifest.json').write_text(json.dumps(manifest, indent=2)+'\n', encoding='utf-8')
    print(json.dumps({'seeds': len(entries), 'minimum_bytes': min(len(x[0]) for x in entries.values()), 'maximum_bytes': max(len(x[0]) for x in entries.values()), 'manifest_sha256': hashlib.sha256((dest/'manifest.json').read_bytes()).hexdigest()}, sort_keys=True))


if __name__ == '__main__':
    main()
