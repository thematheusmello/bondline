# SPDX-License-Identifier: MIT OR Apache-2.0
"""Original mixed-label sources: FontTools boxes and Expat/Decimal lexical splices only."""
from pathlib import Path
from decimal import Decimal
import hashlib, importlib.util, json, os, re, xml.parsers.expat as expat

REPO = next(p for p in Path(__file__).resolve().parents if (p/'xtask/oracles/clearance_oracle.py').is_file())
ROOT = Path(os.environ.get('BONDLINE_ORACLE_OUTPUT', REPO/'target/mixed-labels/independent'))
ROOT.mkdir(parents=True, exist_ok=True)
spec = importlib.util.spec_from_file_location('independent_font', REPO/'xtask/oracles/clearance_oracle.py')
font = importlib.util.module_from_spec(spec)
spec.loader.exec_module(font)

def num(value):
    s = str(float(value))
    return s[:-2] if s.endswith('.0') else s

def exact(value):
    s = format(Decimal(str(value)), 'f')
    return s.rstrip('0').rstrip('.') if '.' in s else s

def label(symbol, h, point, identity):
    text, glyphs = font.authored(symbol, h, 12, point)
    box = ' '.join(exact(v) for v in font.placed_box(glyphs))
    return f"<t id='{identity}' BoundingBox='{box}' Justification='Left' LabelJustification='Left'><s font='42' size='12' face='0' color='4'>{text}</s></t>"

def node(identity, z, h, point, body=None, terminal=None, nonterminal=None):
    attrs = f"id='{identity}' NodeType='Element' Element='{z}' Charge='0' Radical='None' NumHydrogens='{h}' p='{' '.join(exact(v) for v in point)}'"
    if terminal is not None: attrs += f" ShowTerminalCarbonLabels='{terminal}'"
    if nonterminal is not None: attrs += f" ShowNonTerminalCarbonLabels='{nonterminal}'"
    return f'<n {attrs}>{body}</n>' if body is not None else f'<n {attrs}/>'

def bond(identity, a, b):
    return f"<b id='{identity}' B='{a}' E='{b}' Order='1' Display='Solid' Display2='Solid' color='3'/>"

def document(nodes, bonds):
    return "<CDXML CreationProgram='Original mixed skeletal labels' LineWidth='1.2500' MarginWidth='2.000' BondLength='48.00' BondSpacing='18.0'><fonttable><font id='42' name='Roboto'/></fonttable><colortable><color r='1' g='1' b='1'/><color r='0' g='0' b='0'/><color r='.2' g='.4' b='.8'/></colortable><page id='90' DrawingSpace='Poster' Width='512' Height='512'><fragment id='51'>" + ''.join(nodes+bonds) + '</fragment></page></CDXML>'

def splice(raw, selected, dx=0, dy=0, edit=None):
    data = raw.encode(); edits=[]; own=None; text_owner=None; inside_run=False
    dx,dy = Decimal(str(dx)),Decimal(str(dy))
    def attribute(start,name,value):
        end=start; quote=None
        while True:
            c=data[end]
            if quote is not None:
                if c==quote: quote=None
            elif c in (34,39): quote=c
            elif c==62: break
            end+=1
        m=re.search(rb'\b'+name.encode()+rb'\s*=\s*([\'"])(.*?)\1',data[start:end+1])
        assert m is not None
        edits.append((start+m.start(2),start+m.end(2),value.encode()))
    def start(name,a):
        nonlocal own,text_owner,inside_run
        if name=='n':
            own=int(a['id'])
            if own in selected:
                x,y=map(Decimal,a['p'].split())
                attribute(parser.CurrentByteIndex,'p',f'{num(x+dx)} {num(y+dy)}')
            if edit is not None and own==edit['id']:
                attribute(parser.CurrentByteIndex,'Element',str(edit['element']))
                attribute(parser.CurrentByteIndex,'NumHydrogens',str(edit['h']))
        if name=='t' and own is not None:
            text_owner=own
            if own in selected:
                box=list(map(Decimal,a['BoundingBox'].split()))
                attribute(parser.CurrentByteIndex,'BoundingBox',' '.join(num(v+(dx if i%2==0 else dy)) for i,v in enumerate(box)))
            if edit is not None and own==edit['id']:
                _,gs=font.authored(edit['symbol'],edit['h'],12,edit['point'])
                attribute(parser.CurrentByteIndex,'BoundingBox',' '.join(num(v) for v in font.placed_box(gs)))
        if name=='s' and edit is not None and text_owner==edit['id']:
            inside_run=True
            begin=data.index(b'>',parser.CurrentByteIndex)+1
            end=data.index(b'</s>',begin)
            body=edit['symbol']+('H'+(str(edit['h']) if edit['h']>1 else '') if edit['h'] else '')
            edits.append((begin,end,body.encode()))
    def end(name):
        nonlocal own,text_owner,inside_run
        if name=='n': own=None
        if name=='t': text_owner=None
        if name=='s': inside_run=False
    parser=expat.ParserCreate(namespace_separator='|');parser.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
    parser.StartElementHandler=start;parser.EndElementHandler=end;parser.ExternalEntityRefHandler=lambda *_:0
    parser.Parse(data,True)
    for a,b,value in sorted(edits,reverse=True): data=data[:a]+value+data[b:]
    return data.decode()

points=[[96,160],[168,208],[240,160],[312,208],[384,160]]
ids=[73,97,109,137,149]
main=document([
    node(73,8,1,points[0],label('O',1,points[0],21)),
    node(97,6,2,points[1],nonterminal='no'),
    node(109,6,2,points[2],terminal='yes',nonterminal='no'),
    node(137,6,2,points[3],terminal='no',nonterminal='no'),
    node(149,7,2,points[4],label('N',2,points[4],25)),
],[bond(900+i,a,b) for i,(a,b) in enumerate(zip(ids,ids[1:]))])
edited=splice(main,set(),edit={'id':73,'element':7,'symbol':'N','h':2,'point':points[0]})
methanol=document([
    node(73,6,3,[128,192],terminal='no',nonterminal='yes'),
    node(97,8,1,[208,144],label('O',1,[208,144],21)),
],[bond(900,73,97)])
sources={
    'amino_alcohol':main,
    'amino_diamine':edited,
    'hidden_middle_p12_5_m7_25':splice(main,{109},'12.5','-7.25'),
    'all_p24_m12':splice(main,set(ids),24,-12),
    'subset_1_3_5_p12_5_m7_25':splice(main,{73,109,149},'12.5','-7.25'),
    'edited_all_p24_m12':splice(edited,set(ids),24,-12),
    'methanol':methanol,
    'methanol_bad_c2':splice(methanol,set(),edit={'id':97,'element':6,'symbol':'C','h':3,'point':[208,144]}),
    'mandatory_collinear':splice(main,{109},0,48),
}
manifest={'base':'f16b79f','contract':'b30f0d6','method':'Original FontTools boxes plus Expat/Decimal lexical source splices; no Bondline output',
          'font_sha256':font.FONT_SHA,'source_to_model_ids':{'73':1,'97':2,'109':3,'137':4,'149':5},'cases':[]}
for name,raw in sources.items():
    data=raw.encode();(ROOT/(name+'.cdxml')).write_bytes(data)
    manifest['cases'].append({'name':name,'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest(),
        'mixed_expected':name not in ('methanol_bad_c2','mandatory_collinear')})
(ROOT/'mixed-vectors.json').write_text(json.dumps(manifest,indent=2)+'\n')
metadata=(REPO/'crates/cli/tests/support/connected-label-old48.json').read_bytes()
(ROOT/'mixed-old48.json').write_bytes(metadata)
assert len(metadata)==45486 and hashlib.sha256(metadata).hexdigest()=='c861b31ce690ec5808e6262884dc7195e4e7e796f0e2ec6dcf47e082de9ad1da'
print(json.dumps({'cases':len(sources),'manifest_sha256':hashlib.sha256((ROOT/'mixed-vectors.json').read_bytes()).hexdigest()}))
