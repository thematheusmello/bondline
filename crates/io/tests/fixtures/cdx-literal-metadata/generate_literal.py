# SPDX-License-Identifier: MIT OR Apache-2.0
"""Original Python-struct CDX atom-field vectors; no product imports or output."""
from pathlib import Path
import argparse,hashlib,json,struct
SCRIPT=Path(__file__).resolve().parent
REPO=next(p for p in SCRIPT.parents if (p/'Cargo.toml').is_file() and (p/'crates/io').is_dir())
p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output-dir',type=Path,default=REPO/'target/cdx-literal/reproduced');args=p.parse_args();OUT=args.output_dir;OUT.mkdir(parents=True,exist_ok=True)
HEADER=b'VjCD0100\x04\x03\x02\x01'+bytes(16)
def prop(tag,payload,extended=False):
    return struct.pack('<H',tag)+(struct.pack('<HI',65535,len(payload)) if extended else struct.pack('<H',len(payload)))+payload
def obj(tag,ident,*parts):return struct.pack('<HI',tag,ident)+b''.join(parts)+bytes(2)
def atom(ident,element=6,position=(96.25,160.5),isotope=None,charge=None,radical=None,h=None,extended=False,reverse=False,default=False):
    fields=[] if default else [prop(0x0400,struct.pack('<h',1)),prop(0x0402,struct.pack('<h',element))]
    fields.append(prop(0x0200,struct.pack('<ii',int(position[1]*65536),int(position[0]*65536))))
    for tag,value,fmt in [(0x0420,isotope,'<h'),(0x0421,charge,'<b'),(0x0422,radical,'<B'),(0x042b,h,'<H')]:
        if value is not None:fields.append(prop(tag,value if isinstance(value,bytes) else struct.pack(fmt,value),extended))
    if reverse:fields.reverse()
    fields.append(prop(0x4003,b'original atom\0\xff'+struct.pack('<I',ident)))
    return obj(0x8004,ident,*fields)
def scene(atoms,bond=True,envelope='explicit'):
    pieces=[prop(0x4002,b'original fragment\0\x80'),*atoms]
    if bond:pieces.append(obj(0x8005,900,prop(0x0605,struct.pack('<I',149)),prop(0x0604,struct.pack('<I',73)),prop(0x0600,struct.pack('<H',1)),prop(0x4004,b'original bond\0\xfe')))
    fragment=obj(0x8003,51,*pieces)
    custom=obj(0xc123,400000001,prop(0x4005,b'original opaque\0\xfd'),obj(0xc124,400000002))
    page=obj(0x8001,90,prop(0x4001,b'original page\0\xfc'),fragment,custom)
    body=prop(0x4000,b'original root\0\xfb')+page
    if envelope=='implied':return HEADER+body+bytes(2)
    if envelope=='embedded':return HEADER[:22]+obj(0x8000,0,body)+bytes(2)
    assert envelope=='explicit';return HEADER+obj(0x8000,0,body)
def ammonium(extended=False,reverse=False):return [atom(73,6,isotope=13,charge=0,radical=0,h=3,extended=extended,reverse=reverse),atom(149,7,(144.25,160.5),isotope=0,charge=1,radical=0,h=3,extended=extended,reverse=reverse)]
sources={
 'absent_implied':scene([atom(73),atom(149,7,(144.25,160.5))],envelope='implied'),
 'scalar_zero_implied':scene([atom(73,isotope=0,charge=0,radical=0),atom(149,7,(144.25,160.5))],envelope='implied'),
 'zero_present_explicit':scene([atom(73,isotope=0,charge=0,radical=0,h=0),atom(149,7,(144.25,160.5))]),
 'methylammonium_explicit':scene(ammonium()),
 'methylammonium_extended':scene(ammonium(True),envelope='implied'),
 'methylammonium_embedded':scene(ammonium(),envelope='embedded'),
 'methyl_doublet':scene([atom(73,isotope=13,charge=0,radical=2,h=3)],False),
 'singlet':scene([atom(73,isotope=0,charge=0,radical=1,h=2)],False),
 'triplet':scene([atom(73,isotope=0,charge=0,radical=3,h=2)],False),
 'extrema':scene([atom(73,isotope=1,charge=-128,radical=0,h=0),atom(149,7,(144.25,160.5),isotope=32767,charge=127,radical=3,h=65535)]),
 'reordered':scene(ammonium(reverse=True)),
 'default_carbon_fields':scene([atom(73,isotope=13,charge=0,radical=2,h=3,default=True)],False,envelope='implied'),
 'deuteron':scene([atom(73,1,isotope=2,charge=1,radical=0,h=0)],False),
}
collisions={'charge4_low_one':'01000000','charge4_high_one':'00000001','charge4_high_minus_one':'000000ff','charge4_low_noise_max':'aabbcc7f','charge4_min':'00000080'}
for name,payload in collisions.items():sources[name]=scene([atom(73,7,isotope=15,charge=bytes.fromhex(payload),radical=0,h=4)],False)
manifest={'base':'42ccefb','contract':'docs/research/cdx-literal-atom-metadata.md','method':'Original Python struct only; no product parser/writer/output','cases':[]}
for name,raw in sources.items():
    (OUT/(name+'.cdx')).write_bytes(raw);manifest['cases'].append({'name':name,'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()})
(OUT/'literal-vectors.json').write_text(json.dumps(manifest,indent=2)+'\n')
print(json.dumps({'sources':len(sources),'manifest_sha256':hashlib.sha256((OUT/'literal-vectors.json').read_bytes()).hexdigest()}))
