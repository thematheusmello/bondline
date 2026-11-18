# SPDX-License-Identifier: MIT OR Apache-2.0
"""Original dependent-label exact sources. Expat/Decimal, never a Bondline writer."""
from pathlib import Path
from decimal import Decimal
import hashlib,importlib.util,json,re,xml.parsers.expat as expat
REPO=next(p for p in Path(__file__).resolve().parents if (p/'xtask/oracles/clearance_oracle.py').is_file())
ROOT=REPO/'target/dependent-labels/independent'
ROOT.mkdir(parents=True,exist_ok=True)
code=(REPO/'xtask/oracles/boxed_label_support.rs').read_text()
boxed=re.search(r'pub const SOURCE: &str = r#"(.*?)"#;',code,re.S).group(1)
connected=(REPO/'crates/io/tests/fixtures/connected-labels/water.cdxml').read_text()
nh=(REPO/'crates/io/tests/fixtures/connected-labels/ammonia_after_label_edit.cdxml').read_text()
spec=importlib.util.spec_from_file_location('independent_font',REPO/'xtask/oracles/clearance_oracle.py')
font=importlib.util.module_from_spec(spec);spec.loader.exec_module(font)

def number(value):
    s=format(value,'f')
    return s.rstrip('0').rstrip('.') if '.' in s else s

def changed_number(value):
    # Independent shortest-roundtrip formatting for these exact f64 dyadic fixture values.
    s=str(float(value))
    return s[:-2] if s.endswith('.0') else s

def node(source_id,element,symbol,h,point):
    text,glyphs=font.authored(symbol,h,12,point);box=font.placed_box(glyphs)
    p=' '.join(number(Decimal(str(x))) for x in point)
    b=' '.join(number(Decimal(str(x))) for x in box)
    return f"<n id='{source_id}' NodeType='Element' Element='{element}' Charge='0' Radical='None' NumHydrogens='{h}' p='{p}'><t id='21' BoundingBox='{b}' Justification='Left' LabelJustification='Left'><s font='42' size='12' face='0' color='4'>{text}</s></t></n>"

def moved(raw,ids,dx,dy):
    data=raw.encode();edits=[];selected=False
    dx,dy=Decimal(str(dx)),Decimal(str(dy))
    def attribute(start,name,value):
        # Only the original numeric fixture attributes are targeted; preserve every other byte.
        end=start;quote=None
        while True:
            c=data[end]
            if quote is not None:
                if c==quote:quote=None
            elif c in (34,39):quote=c
            elif c==62:break
            end+=1
        part=data[start:end+1]
        m=re.search(rb'\b'+name.encode()+rb'\s*=\s*([\'"])(.*?)\1',part)
        assert m is not None
        edits.append((start+m.start(2),start+m.end(2),value.encode()))
    def start(name,a):
        nonlocal selected
        if name=='n':
            selected=int(a['id']) in ids
            if selected:
                x,y=[Decimal(v) for v in a['p'].split()]
                attribute(parser.CurrentByteIndex,'p',f'{changed_number(x+dx)} {changed_number(y+dy)}')
        if name=='t' and selected:
            l,t,r,b=[Decimal(v) for v in a['BoundingBox'].split()]
            attribute(parser.CurrentByteIndex,'BoundingBox',' '.join(map(changed_number,[l+dx,t+dy,r+dx,b+dy])))
    def end(name):
        nonlocal selected
        if name=='n':selected=False
    parser=expat.ParserCreate(namespace_separator='|');parser.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
    parser.StartElementHandler=start;parser.EndElementHandler=end;parser.Parse(data,True)
    assert len(edits)==2*len(ids)
    for a,b,value in sorted(edits,reverse=True):data=data[:a]+value+data[b:]
    return data.decode()

boxed_pair=boxed.replace('</fragment>',node(97,8,'O',1,[200,200])+'</fragment>')
boxed_rounding=re.sub(r'<n\b.*?</n>',node(73,8,'O',0,[128,128]),boxed,count=1,flags=re.S)
boxed_tolerated=boxed.replace('80.69140625 91.3515625','80.691410064697265625 91.3515625')
connected_component=connected.replace('</fragment>',node(137,7,'N',3,[360,360])+'</fragment>')
crossing=connected.replace('</fragment>',node(137,1,'H',0,[180,250])+node(149,1,'H',0,[280,250])+"<b id='902' B='137' E='149' Order='1'/></fragment>")
cases=[]
sources={
 'boxed':boxed,'boxed_pair':boxed_pair,'boxed_rounding_boundary':boxed_rounding,'boxed_tolerated_raw':boxed_tolerated,'connected':connected,'connected_nh':nh,
 'connected_component':connected_component,'connected_crossing_base':crossing,
 'boxed_sub_p12_5_m7_25':moved(boxed,{73},'12.5','-7.25'),
 'boxed_all_p24_m12':moved(boxed,{73},24,-12),
 'boxed_nudge_p25_m2':moved(boxed,{73},25,-2),
 'boxed_tolerated_sub_p12_5_m7_25':moved(boxed_tolerated,{73},'12.5','-7.25'),
 'boxed_pair_all_p24_m12':moved(boxed_pair,{73,97},24,-12),
 'connected_sub_p12_5_m7_25':moved(connected,{73},'12.5','-7.25'),
 'connected_all_p24_m12':moved(connected,{73,97,109},24,-12),
 'connected_nh_sub_p12_5_m7_25':moved(nh,{73},'12.5','-7.25'),
 'connected_drag_p24_m12':moved(connected,{73,97,109},24,-12),
 'connected_nudge_p25_m2':moved(connected,{73,97,109},25,-2),
 'connected_component_p24_m12':moved(connected_component,{73,97,109},24,-12),
}
for name,source in sources.items():
    (ROOT/(name+'.cdxml')).write_text(source)
    cases.append({'name':name,'bytes':len(source.encode()),'sha256':hashlib.sha256(source.encode()).hexdigest()})
manifest={'base':'9cf7103','scope':'Original exact numeric source splices, no product encoder/native/reference execution',
 'source_id_to_app_atom':{'73':1,'97':2,'109':3,'137':4,'149':5},
 'field_policy':'Pure translation changes Node.p and child BoundingBox only. Compound label edit then translation permits exactly Element/NumHydrogens/body/BoundingBox/Node.p; no other field.',
 'font_sha256':font.FONT_SHA,'cases':cases}
(ROOT/'move-vectors.json').write_text(json.dumps(manifest,indent=2)+'\n')
print(json.dumps({'cases':len(cases),'sha256':hashlib.sha256((ROOT/'move-vectors.json').read_bytes()).hexdigest()}))
