#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
# Original vectors; never use a committed fixture directory as a mutation corpus.
from pathlib import Path
from hashlib import sha256
import json, sys

entries = {
    'absent.xml': b'<CDXML><page/></CDXML>',
    'one-color.xml': b"<CDXML><colortable><color r='.25' g='.5' b='.75'/></colortable></CDXML>",
    'logical-fonts.xml': '<CDXML><fonttable><font id="0" name="μ &amp; A"/><font id="65535" name="" charset="opaque"/></fonttable></CDXML>'.encode(),
    'palette.xml': b"<CDXML><colortable><color r='1' g='1' b='1'/><color r='0' g='0' b='0'/><color r='.125' g='.5' b='.875'/></colortable></CDXML>",
    'empty-fonts.xml': b'<CDXML><fonttable/></CDXML>',
    'empty-colors.xml': b'<CDXML><colortable/></CDXML>',
    'duplicate-font-id.xml': b"<CDXML><fonttable><font id='1' name='A'/><font id='1.00' name='B'/></fonttable></CDXML>",
    'qualified-attributes.xml': b"<CDXML xmlns:x='urn:other'><fonttable><font x:id='invalid' id='7' x:name='wrong' name='right'/></fonttable></CDXML>",
    'rgb-whitespace.xml': b"<CDXML><colortable><color r=' &#9;.125&#13; ' g=' .5 ' b=' 1 '/></colortable></CDXML>",
    'misplaced.xml': b'<CDXML><page><fonttable/></page></CDXML>',
    'namespaced.xml': b"<CDXML><fonttable xmlns='urn:other'/></CDXML>",
    'empty-namespace-reset.xml': b"<CDXML><fonttable xmlns=''><font xmlns='' id='0' name='A'/></fonttable></CDXML>",
    'nonfinite.xml': b"<CDXML><colortable><color r='NaN' g='0' b='0'/></colortable></CDXML>",
    'text.xml': b"<CDXML><colortable>text<color r='0' g='0' b='0'/></colortable></CDXML>",
    'integer.txt': b' 65535.00 ',
    'fraction.txt': b'1.5',
    'rgb.txt': b' .125 ',
    'invalid-utf8.bin': b'\xff\xfe\x80',
}
head = b"<CDXML><fonttable><font id='0' name='"
tail = b"'/></fonttable></CDXML>"
entries['exact-16k.xml'] = head + b'x' * (16384-len(head)-len(tail)) + tail
base = Path(sys.argv[1])
base.mkdir(parents=True, exist_ok=False)
source = base/'source'; source.mkdir()
corpus = base/'corpus'; corpus.mkdir()
manifest = {'license': 'MIT OR Apache-2.0', 'limit_bytes': 16384, 'files': []}
for name, data in entries.items():
    assert len(data) <= 16384
    (source/name).write_bytes(data)
    (corpus/name).write_bytes(data)
    manifest['files'].append({'name':name,'bytes':len(data),'sha256':sha256(data).hexdigest()})
(base/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8')
print(json.dumps({'seeds':len(entries),'largest':max(map(len,entries.values())),'manifest':str(base/'manifest.json')}))
