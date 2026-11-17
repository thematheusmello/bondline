#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
# Original Bondline vectors. Stdlib only; the destination must not already exist.
from pathlib import Path
from hashlib import sha256
import json, sys

ORIGINAL = b'''<?xml version="1.0" encoding="UTF-8"?>
<CDXML CreationProgram="Bondline 0.1.0">
 <page id='91' Width='612.00' Height="792.000" DrawingSpace='Poster'>
  <fragment id="400">
   <b id='900' E='207' Order='1' B='101' />
   <n Radical='None' id='101' p='100.00 200.000' NumHydrogens='3' Charge='0' Element='6' NodeType='Element'/>
   <n Element="6" NumHydrogens="3" NodeType="Element" p="140 200" id="207" Radical="None" Charge="0"></n>
  </fragment>
 </page>
</CDXML>
'''
SCOPED = b'''<CDXML CreationProgram='Bondline 0.1.0'><page id='4294967295' Width='612.00' DrawingSpace='Poster' Height='792.000'><fragment id='7'><!--a--><b id='900' E='207' B='101' Order='1'/><n id='101' NodeType='Element' Element='6' Charge='0' Radical='None' p='72.00 96.000' NumHydrogens='3'/><n id='207' NodeType='Element' Element='6' Charge='0' Radical='None' p='120 96' NumHydrogens='3'/></fragment><fragment id='8'><n id='101' NodeType='Element' Element='8' Charge='0' Radical='None' p='192 192' NumHydrogens='2'/></fragment></page></CDXML>'''
CANONICAL = b'''<?xml version="1.0" encoding="UTF-8"?><CDXML CreationProgram="Bondline 0.1.0"><page id="1" DrawingSpace="Poster" Width="612" Height="792"><fragment id="2"><n id="3" NodeType="Element" Element="6" Charge="0" Radical="None" p="100 200" NumHydrogens="3"/><n id="4" NodeType="Element" Element="6" Charge="0" Radical="None" p="140 200" NumHydrogens="3"/><b id="5" B="3" E="4" Order="1"/></fragment></page></CDXML>'''
assert len(ORIGINAL) == 464
assert sha256(ORIGINAL).hexdigest() == 'ec78b5b51154ee6c5f74efe65452943281be66f892f22927453e180177480f93'
assert sha256(CANONICAL).hexdigest() == 'ead553df9d8c6bf7a371e7e36909a9c8895f34b9231f20c3c3a796680d64bbc6'

entries = []
def seed(name, category, expected, data):
    assert len(data) <= 16384
    entries.append((name, category, expected, data))
seed('original-forward-464.xml', 'positive/original-sparse-forward-reference', 'complete', ORIGINAL)
seed('scoped-u32max.xml', 'positive/repeated-scoped-ids-and-max-id', 'complete', SCOPED)
seed('native-v0-literal.xml', 'positive/original-native-v0-literal', 'complete', CANONICAL)
for name, width in [('half', b'.500'), ('two', b'2.0'), ('maximum', b'100000')]:
    seed('style-'+name+'.xml', 'positive/explicit-document-line-width', 'complete', ORIGINAL.replace(b'<CDXML ', b'<CDXML LineWidth="'+width+b'" ', 1))
for name, width in [('zero', b'0'), ('nonfinite', b'NaN')]:
    seed('style-'+name+'.xml', 'opaque/unsupported-line-width', 'opaque', ORIGINAL.replace(b'<CDXML ', b'<CDXML LineWidth="'+width+b'" ', 1))
seed('style-object-override.xml', 'opaque/unmodelled-object-line-width', 'opaque', ORIGINAL.replace(b'<n ', b'<n LineWidth="2" ', 1))
seed('empty-selfclosing-page.xml', 'positive/selfclosing-parent', 'complete', b"<CDXML CreationProgram='Bondline 0.1.0'><page id='91' DrawingSpace='Poster' Width='612.00' Height='792.000' /></CDXML>")
seed('empty-paired-fragment.xml', 'positive/paired-parent-and-comment', 'complete', b"<CDXML CreationProgram='Bondline 0.1.0'><page id='91' DrawingSpace='Poster' Width='612.00' Height='792.000'><fragment id='7'><!--stay--></fragment></page></CDXML>")
seed('optional-flags-and-side.xml', 'positive/optional-settings-and-fields', 'complete', ORIGINAL.replace(b'<CDXML ', b'<CDXML BondLength="48.00" ChainAngle="90.0" ', 1).replace(b' Order=\'1\'', b' DoublePosition="Right" Order=\'1\'', 1).replace(b"<n Radical=", b"<n Isotope='13' ShowTerminalCarbonLabels='no' ShowNonTerminalCarbonLabels='yes' Radical=", 1))
seed('absent-h-position.xml', 'positive/unknown-h-and-position', 'complete', b"<CDXML CreationProgram='Bondline 0.1.0'><page id='91' DrawingSpace='Poster' Width='612' Height='792'><fragment id='7'><n id='101' NodeType='Element' Element='6' Charge='0' Radical='None'/></fragment></page></CDXML>")
seed('unknown-root.xml', 'opaque/unknown-metadata', 'opaque', ORIGINAL.replace(b'<CDXML ', b'<CDXML Future="opaque" ', 1))
seed('processing-instruction.xml', 'opaque/processing-instruction', 'opaque', b'<?future opaque?>'+ORIGINAL[ORIGINAL.index(b'<CDXML'):])
seed('explicit-xml-prefix.xml', 'opaque/namespace-declaration', 'opaque', ORIGINAL.replace(b'<CDXML ', b'<CDXML xmlns:xml="http://www.w3.org/XML/1998/namespace" ', 1))
seed('doctype.xml', 'opaque/doctype', 'opaque', b'<!DOCTYPE CDXML>'+ORIGINAL[ORIGINAL.index(b'<CDXML'):])
seed('whitespace-cdata.xml', 'opaque/cdata', 'opaque', ORIGINAL.replace(b'</fragment>', b'<![CDATA[ ]]></fragment>', 1))
seed('missing-native-default.xml', 'opaque/missing-native-emitted-key', 'opaque', ORIGINAL.replace(b" Charge='0'", b'', 1))
seed('drawing-space-mismatch.xml', 'opaque/fixed-value-mismatch', 'opaque', ORIGINAL.replace(b"DrawingSpace='Poster'", b"DrawingSpace='Pages'", 1))
seed('root-empty-namespace.xml', 'rejected/existing-root-namespace-admission', 'parse_or_projection_error', b"<CDXML xmlns=''/>")
seed('truncated.xml', 'rejected/truncated-xml', 'parse_or_projection_error', b'<CDXML><page')
seed('selectors-zero.bin', 'structured-only/all-selector-zeroes', 'structured_only', bytes(8))
seed('selectors-max.bin', 'structured-only/all-selector-bits', 'structured_only', bytes([255])*8)
seed('invalid-utf8.bin', 'rejected/invalid-utf8-and-structured-selectors', 'parse_or_projection_error', b'\xff\xfe\x80')
head = b'<CDXML CreationProgram="Bondline 0.1.0" Future="'
tail = b'"><page id="91" DrawingSpace="Poster" Width="612" Height="792"/></CDXML>'
boundary = head+b'x'*(16384-len(head)-len(tail))+tail
assert len(boundary) == 16384
seed('exact-16k.xml', 'boundary/accepted-opaque-16384-bytes', 'opaque', boundary)

base = Path(sys.argv[1])
base.mkdir(parents=True, exist_ok=False)
source = base/'source'; source.mkdir()
corpus = base/'corpus'; corpus.mkdir()
manifest = {'license':'MIT OR Apache-2.0','limit_bytes':16384,'files':[]}
for name,category,expected,data in entries:
    (source/name).write_bytes(data)
    (corpus/name).write_bytes(data)
    manifest['files'].append({'name':name,'category':category,'expected':expected,'bytes':len(data),'sha256':sha256(data).hexdigest()})
# Outside the mutation corpus; source copies are never libFuzzer's working inputs.
(base/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8')
print(json.dumps({'seeds':len(entries),'largest':max(len(data) for _,_,_,data in entries),'manifest':str(base/'manifest.json')}))
