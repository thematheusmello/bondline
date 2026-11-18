# SPDX-License-Identifier: MIT OR Apache-2.0
"""Original plain-bond CDX sources built only with Python struct."""
from pathlib import Path
import argparse,hashlib,json,struct
SCRIPT=Path(__file__).resolve().parent
REPO=next(p for p in SCRIPT.parents if (p/'Cargo.toml').is_file() and (p/'crates/io').is_dir())
p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output-dir',type=Path,default=REPO/'target/cdx-bonds/reproduced');args=p.parse_args();OUT=args.output_dir;OUT.mkdir(parents=True,exist_ok=True)
HEADER=b'VjCD0100\x04\x03\x02\x01'+bytes(16)
def prop(tag,payload,extended=False):return struct.pack('<H',tag)+(struct.pack('<HI',65535,len(payload)) if extended else struct.pack('<H',len(payload)))+payload
def obj(tag,identity,*parts):return struct.pack('<HI',tag,identity)+b''.join(parts)+bytes(2)
def atom(identity,x,h):
 fields=[prop(0x0400,struct.pack('<h',1)),prop(0x0200,struct.pack('<ii',int(160.5*65536),int(x*65536))),prop(0x0402,struct.pack('<h',6))]
 if h is not None:fields.append(prop(0x042b,struct.pack('<H',h)))
 return obj(0x8004,identity,*fields,prop(0x4003,b'original bond atom\0\xff'+struct.pack('<I',identity)))
def scene(order=None,display=None,display2=None,position=None,h=None,reverse=False,reorder=False,extended=False,envelope='explicit'):
 fields=[prop(0x0604,struct.pack('<I',149 if reverse else 73)),prop(0x0605,struct.pack('<I',73 if reverse else 149))]
 for tag,value in [(0x0600,order),(0x0601,display),(0x0602,display2),(0x0603,position)]:
  if value is not None:fields.append(prop(tag,struct.pack('<H',value),extended))
 if reorder:fields.reverse()
 bond=obj(0x8005,900,*fields,prop(0x4004,b'original plain bond\0\xfe'))
 fragment=obj(0x8003,51,prop(0x4002,b'original plain fragment\0\x80'),atom(73,96.25,h),bond,atom(149,144.25,h))
 custom=obj(0xc123,400000001,prop(0x4005,b'original plain opaque\0\xfd'),obj(0xc124,400000002))
 page=obj(0x8001,90,prop(0x4001,b'original plain page\0\xfc'),fragment,custom)
 body=prop(0x4000,b'original plain root\0\xfb')+page
 if envelope=='implied':return HEADER+body+bytes(2)
 if envelope=='embedded':return HEADER[:22]+obj(0x8000,0,body)+bytes(2)
 assert envelope=='explicit';return HEADER+obj(0x8000,0,body)
def right(**kwargs):return scene(order=2,display=0,display2=0,position=257,h=2,**kwargs)
sources={
 'single_absent':scene(envelope='implied'),
 'single_solid':scene(order=1,display=0),
 'ethene':scene(order=2,h=2),
 'ethyne':scene(order=4,display=0,h=1),
 'auto_center':scene(order=2,display=0,position=0,h=2),
 'auto_right':scene(order=2,display=0,display2=0,position=1,h=2),
 'auto_left':scene(order=2,display2=0,position=2,h=2),
 'manual_center':scene(order=2,display=0,position=256,h=2),
 'manual_right':right(),
 'manual_left':scene(order=2,display2=0,position=258,h=2),
 'reversed_right':right(reverse=True),
 'reordered_right':right(reorder=True),
 'extended_right':right(extended=True),
 'embedded_right':right(envelope='embedded'),
 'implied_right':right(envelope='implied'),
 'ethene_unknown_h':scene(order=2),
}
manifest={'base':'baa3d90','contract':'6c9fff0 docs/research/cdx-plain-bonds.md','method':'Original Python struct only; no product parser/writer/output','cases':[]}
for name,raw in sources.items():
 (OUT/(name+'.cdx')).write_bytes(raw);manifest['cases'].append({'name':name,'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()})
(OUT/'bond-vectors.json').write_text(json.dumps(manifest,indent=2)+'\n')
print(json.dumps({'sources':len(sources),'manifest_sha256':hashlib.sha256((OUT/'bond-vectors.json').read_bytes()).hexdigest()}))
