# SPDX-License-Identifier: MIT OR Apache-2.0
"""Original creation vectors, FontTools geometry and Expat lexical edits, no product output."""
from pathlib import Path
from decimal import Decimal
import argparse,hashlib,importlib.util,json,re,xml.parsers.expat as expat
SCRIPT_DIR=Path(__file__).resolve().parent
REPO=next(p for p in SCRIPT_DIR.parents if (p/'assets/fonts/Roboto-Regular.ttf').is_file())
parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output-dir',type=Path,default=REPO/'target/label-creation/reproduced')
ROOT=parser.parse_args().output_dir;ROOT.mkdir(parents=True,exist_ok=True)
spec=importlib.util.spec_from_file_location('font_oracle',REPO/'xtask/oracles/clearance_oracle.py')
font=importlib.util.module_from_spec(spec);spec.loader.exec_module(font)
BASE=(REPO/'crates/io/tests/fixtures/mixed-labels/amino_alcohol.cdxml').read_text()
BLACK={'font_family':'Roboto','size_pt':12.0,'face':0,'rgb':[0.0,0.0,0.0]}
BLUE={'font_family':'Roboto','size_pt':14.0,'face':0,'rgb':[0.2,0.4,0.8]}
def num(x):
    value=str(float(x));return value[:-2] if value.endswith('.0') else value
def scan(raw):
    data=raw.encode();objects=set();fonts={};nodes={};stack=[]
    def start(name,a):
        if 'id' in a and name!='font':objects.add(int(a['id']))
        if name=='font':fonts[int(a['id'])]=a['name']
        if name=='n':
            at=p.CurrentByteIndex;end=at;quote=None
            while True:
                c=data[end]
                if quote is not None:
                    if c==quote:quote=None
                elif c in (34,39):quote=c
                elif c==62:break
                end+=1
            nodes[int(a['id'])]={'attrs':a,'start':at,'tag_end':end,'close':None,'text':False,'empty':data[end-1:end]==b'/'}
            stack.append(int(a['id']))
        elif name=='t' and stack:nodes[stack[-1]]['text']=True
    def end(name):
        if name=='n':nodes[stack.pop()]['close']=p.CurrentByteIndex
    p=expat.ParserCreate();p.StartElementHandler=start;p.EndElementHandler=end;p.Parse(data,True)
    return nodes,objects,fonts
def reservations(baseline):
    nodes,used,_=scan(baseline);result={};n=1
    for key,node in nodes.items():
        if node['text']:continue
        while n in used:n+=1
        result[key]=n;used.add(n)
    return result
def field(raw,source_id,name,value):
    data=raw.encode();n=scan(raw)[0][source_id];part=data[n['start']:n['tag_end']+1]
    m=re.search(rb'\b'+name.encode()+rb'\s*=\s*([\'"])(.*?)\1',part);assert m
    a,b=n['start']+m.start(2),n['start']+m.end(2)
    return (data[:a]+value.encode()+data[b:]).decode()
def body(symbol,h):return symbol+('H'+(str(h) if h>1 else '') if h else '')
def glyph_box(z,h,size,point):
    symbol={6:'C',7:'N',8:'O'}[z];text,gs=font.authored(symbol,h,size,point)
    return text,font.placed_box(gs)
def create(raw,baseline,source_id,z=6,h=2,style=BLACK):
    nodes,_,fonts=scan(raw);n=nodes[source_id];assert not n['text'];a=n['attrs']
    fid=min(key for key,value in fonts.items() if value==style['font_family'])
    cid=0 if style['rgb']==[0,0,0] else 4
    point=[float(Decimal(x)) for x in a['p'].split()];text,box=glyph_box(z,h,style['size_pt'],point)
    child=f'<t id="{reservations(baseline)[source_id]}" BoundingBox="{" ".join(num(x) for x in box)}" Justification="Left" LabelJustification="Left"><s font="{fid}" size="{num(style["size_pt"])}" face="0" color="{cid}">{text}</s></t>'
    raw=field(raw,source_id,'Element',str(z));raw=field(raw,source_id,'NumHydrogens',str(h));n=scan(raw)[0][source_id];data=raw.encode()
    if n['empty']:
        at=n['tag_end']-1;raw=(data[:at]+b'>'+child.encode()+b'</n>'+data[n['tag_end']+1:]).decode()
    else:
        at=n['close'];raw=(data[:at]+child.encode()+data[at:]).decode()
    return raw
def edit_label(raw,source_id,z,h):
    n=scan(raw)[0][source_id];assert n['text'];a=n['attrs'];point=[float(Decimal(x)) for x in a['p'].split()]
    inner=raw[n['tag_end']+1:n['close']];size=float(re.search(r'\bsize=([\'"])(.*?)\1',inner)[2])
    text,box=glyph_box(z,h,size,point)
    inner=re.sub(r'\bBoundingBox=([\'"])(.*?)\1',lambda m:'BoundingBox='+m[1]+' '.join(num(x) for x in box)+m[1],inner,count=1)
    inner=re.sub(r'(<s\b[^>]*>).*?(</s>)',lambda m:m[1]+text+m[2],inner,count=1,flags=re.S)
    raw=raw[:n['tag_end']+1]+inner+raw[n['close']:]
    return field(field(raw,source_id,'Element',str(z)),source_id,'NumHydrogens',str(h))
def moved(raw,selected,dx,dy):
    for source_id in selected:
        n=scan(raw)[0][source_id];a=n['attrs'];point=[Decimal(x) for x in a['p'].split()]
        raw=field(raw,source_id,'p',' '.join(num(v+Decimal(str(dx if i==0 else dy))) for i,v in enumerate(point)))
        if n['text']:
            n=scan(raw)[0][source_id];inner=raw[n['tag_end']+1:n['close']]
            inner=re.sub(r'\bBoundingBox=([\'"])(.*?)\1',lambda m:'BoundingBox='+m[1]+' '.join(num(Decimal(v)+Decimal(str(dx if i%2==0 else dy))) for i,v in enumerate(m[2].split()))+m[1],inner,count=1)
            raw=raw[:n['tag_end']+1]+inner+raw[n['close']:]
    return raw
def dense(raw):
    mapping={90:1,51:2,73:3,97:4,109:5,137:6,149:7,900:8,901:9,902:10,903:11,21:12,25:13}
    raw=re.sub(r'\bid=\'([0-9]+)\'',lambda m:"id='"+str(14 if int(m[1])==42 else mapping[int(m[1])])+"'",raw)
    raw=re.sub(r'\b(B|E)=\'([0-9]+)\'',lambda m:m[1]+"='"+str(mapping[int(m[2])])+"'",raw)
    return raw.replace("font='42'","font='14'")
paired=BASE.replace("p='240 160' ShowTerminalCarbonLabels='yes' ShowNonTerminalCarbonLabels='no'/>","p='240 160' ShowTerminalCarbonLabels='yes' ShowNonTerminalCarbonLabels='no'><!-- retain original node trivia -->\n</n>")
dense_base=dense(BASE)
caption_base=BASE.replace('</page>',"<t id='1' p='64 64' Justification='Left'><s font='42' size='12' face='0' color='4'>Original source caption</s></t></page>")
c3=create(BASE,BASE,109);c32=create(c3,BASE,97);all_=create(c32,BASE,137)
c2=create(BASE,BASE,97);c23=create(c2,BASE,109)
assert c23==c32 and create(c23,BASE,137)==all_
assert create(c3,c3,97)==c32
d3=create(dense_base,dense_base,5);d32=create(d3,dense_base,4);dall=create(d32,dense_base,6)
sources={
 'base':BASE,'created_3_black':c3,'created_3_2_black':c32,'created_all_black':all_,'created_2_black':c2,
 'created_3_blue14':create(BASE,BASE,109,style=BLUE),
 'created_3_nh':create(BASE,BASE,109,z=7,h=1),
 'created_3_old48_nh':edit_label(c3,109,7,1),
 'endpoint_edited_created_3':create(edit_label(BASE,73,7,2),BASE,109),
 'created_3_all_p24_m12':moved(c3,[73,97,109,137,149],24,-12),
 'created_3_selected_p12_5_m7_25':moved(c3,[109,137],12.5,-7.25),
 'created_all_p24_m12':moved(all_,[73,97,109,137,149],24,-12),
 'paired_base':paired,'paired_created_3':create(paired,paired,109),
 'dense_base':dense_base,'dense_created_3':d3,'dense_created_3_2':d32,'dense_created_all':dall,
 'caption_base':caption_base,'caption_created_3':create(caption_base,caption_base,109),
}
assert sources['created_3_nh']==sources['created_3_old48_nh']
manifest={'base':'7113237','contract':'126f99e','method':'Original frozen Mixed source plus independent Expat/Decimal append/splices and FontTools outline boxes; no Bondline output',
 'font_sha256':font.FONT_SHA,'default_explicit_style':BLACK,'alternate_explicit_style':BLUE,
 'reserved_main':reservations(BASE),'reserved_dense':reservations(dense_base),'cases':[]}
for name,raw in sources.items():
    data=raw.encode();(ROOT/(name+'.cdxml')).write_bytes(data)
    manifest['cases'].append({'name':name,'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest(),
                              'flagged_expected':name not in ('base','paired_base','dense_base','caption_base')})
(ROOT/'creation-vectors.json').write_text(json.dumps(manifest,indent=2)+'\n')
old=(REPO/'crates/cli/tests/support/connected-label-old48.json').read_bytes()
assert len(old)==45486 and hashlib.sha256(old).hexdigest()=='c861b31ce690ec5808e6262884dc7195e4e7e796f0e2ec6dcf47e082de9ad1da'
print(json.dumps({'sources':len(sources),'manifest_sha256':hashlib.sha256((ROOT/'creation-vectors.json').read_bytes()).hexdigest()}))
