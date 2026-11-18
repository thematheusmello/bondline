# SPDX-License-Identifier: MIT OR Apache-2.0
"""Original immutable metadata overlays on independently frozen explicit-label sources."""
from pathlib import Path
from decimal import Decimal
import argparse,hashlib,json,re,xml.parsers.expat as expat
SCRIPT=Path(__file__).resolve().parent
REPO=next(p for p in SCRIPT.parents if (p/'assets/fonts/Roboto-Regular.ttf').is_file())
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--output-dir',type=Path,default=REPO/'target/effective-labels/reproduced')
args=parser.parse_args();ROOT=args.output_dir;ROOT.mkdir(parents=True,exist_ok=True)
CREATED=REPO/'crates/io/tests/fixtures/created-labels';MIXED=REPO/'crates/io/tests/fixtures/mixed-labels'
def insert(raw,kind,identity,attributes):
    data=raw.encode();target=[]
    def start(name,a):
        if name==kind and (identity is None or a.get('id')==str(identity)):
            at=p.CurrentByteIndex;quote=None
            while True:
                c=data[at]
                if quote is not None:
                    if c==quote:quote=None
                elif c in (34,39):quote=c
                elif c==62:break
                at+=1
            target.append(at-1 if data[at-1:at]==b'/' else at)
    p=expat.ParserCreate();p.StartElementHandler=start;p.Parse(data,True);assert len(target)==1
    at=target[0];return (data[:at]+(' '+attributes).encode()+data[at:]).decode()
def overlay(raw,variant='a'):
    raw=raw.replace('</fonttable>',"<font id='7' name='Unused serif'/></fonttable>")
    raw=insert(raw,'CDXML',None,"LabelFont=' +&#55;.000 ' LabelSize='10.00' LabelFace='96'" if variant=='a' else 'LabelFont="7" LabelSize="11.000" LabelFace="96"')
    raw=insert(raw,'n',73,"LabelSize='14.00'")
    raw=insert(raw,'t',21,"LabelSize='16.0' LabelFace='0'")
    if "<b id='900'" in raw:raw=insert(raw,'b',900,"LabelFont='42' LabelSize='1e308' LabelFace='32767'")
    return raw
def number(value):
    s=str(float(value));return s[:-2] if s.endswith('.0') else s
def move_all(raw,dx,dy):
    data=raw.encode();edits=[]
    def start(name,a):
        key='p' if name=='n' else 'BoundingBox' if name=='t' and 'BoundingBox' in a else None
        if key is None:return
        at=p.CurrentByteIndex;end=data.index(b'>',at);part=data[at:end]
        m=re.search(rb'\b'+key.encode()+rb'=([\'"])(.*?)\1',part);assert m
        fields=a[key].split();new=' '.join(number(Decimal(x)+Decimal(str(dx if i%2==0 else dy))) for i,x in enumerate(fields))
        edits.append((at+m.start(2),at+m.end(2),new.encode()))
    p=expat.ParserCreate();p.StartElementHandler=start;p.Parse(data,True)
    for a,b,value in sorted(edits,reverse=True):data=data[:a]+value+data[b:]
    return data.decode()
base=(CREATED/'base.cdxml').read_text();a=overlay(base)
caption=overlay((CREATED/'caption_base.cdxml').read_text());caption=insert(caption,'t',1,"LabelFont='7' LabelSize='20.000' LabelFace='96'")
unsigned=a.replace("</fonttable>","<font id='65535' name='Roboto'/></fonttable>").replace("<s font='42'","<s font='65535'")
boxed_code=(REPO/'xtask/oracles/boxed_label_support.rs').read_text();boxed=re.search(r'pub const SOURCE: &str = r#"(.*?)"#;',boxed_code,re.S).group(1)
boxed=boxed.replace('</fonttable>',"<font id='7' name='Unused serif'/></fonttable>")
boxed=insert(boxed,'CDXML',None,"LabelFont='7' LabelSize='10' LabelFace='96'")
boxed=insert(boxed,'n',73,"LabelSize='14'");boxed=insert(boxed,'t',21,"LabelSize='16' LabelFace='0'")
sources={
 'mixed_a':a,'mixed_b':overlay(base,'b'),
 'edited_endpoint':overlay((MIXED/'amino_diamine.cdxml').read_text()),
 'created_3':overlay((CREATED/'created_3_black.cdxml').read_text()),
 'all_printed':overlay((CREATED/'created_all_black.cdxml').read_text()),
 'mixed_moved':overlay((MIXED/'all_p24_m12.cdxml').read_text()),
 'compound':overlay(move_all((CREATED/'endpoint_edited_created_3.cdxml').read_text(),24,-12)),
 'all_moved':overlay((CREATED/'created_all_p24_m12.cdxml').read_text()),
 'caption_owners':caption,'unsigned_actual_font':unsigned,
 'partial':insert(base,'CDXML',None,"LabelSize='.125'"),'boxed':boxed,
}
manifest={'base':'0aa97cd','contract':'8c2b608','method':'Original Expat/Decimal metadata overlays on prior independent sources; no Bondline expected output',
 'scope':'Metadata only; every actual style-run font/size/face/color explicit; no omitted-run inheritance','cases':[]}
for name,raw in sources.items():
    data=raw.encode();(ROOT/(name+'.cdxml')).write_bytes(data)
    manifest['cases'].append({'name':name,'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest()})
(ROOT/'metadata-vectors.json').write_text(json.dumps(manifest,indent=2)+'\n')
# Actual49 stdout was frozen before producer changes; reproduction copies that receipt.
listed=(SCRIPT/'old49.json').read_bytes()
assert hashlib.sha256(listed).hexdigest()=='3cfd51a6e72891137cf0cc56fd756ff33652cfc685e5aaf877634b259cc41e4f'
rows=json.loads(listed);assert len(rows)==49 and rows[-1]['name']=='atom.create_authored_label'
old48=(REPO/'crates/cli/tests/support/connected-label-old48.json').read_text().strip();assert listed.startswith(old48[:-1].encode()+b',')
(ROOT/'old49.json').write_bytes(listed)
print(json.dumps({'sources':len(sources),'manifest_sha256':hashlib.sha256((ROOT/'metadata-vectors.json').read_bytes()).hexdigest(),'metadata49_bytes':len(listed),'metadata49_sha256':hashlib.sha256(listed).hexdigest()}))
