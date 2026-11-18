# SPDX-License-Identifier: MIT OR Apache-2.0
"""Strict actual-file mixed source reader; Expat/Decimal/RDKit, no Bondline imports."""
import json, math, sys, xml.parsers.expat as expat
from decimal import Decimal
from rdkit import Chem, rdBase
from rdkit.Chem import rdMolDescriptors

assert rdBase.rdkitVersion == '2026.09.2'
raw=sys.stdin.buffer.read(65537); assert 0<len(raw)<=65536
fonts={};colors=[];nodes=[];bonds=[];style={};style_raw={};stack=[];node=None;label=None;run=False
flags=['ShowTerminalCarbonLabels','ShowNonTerminalCarbonLabels']
def numbers(s,count):
    value=[float(Decimal(x)) for x in s.split()]
    assert len(value)==count and all(math.isfinite(x) for x in value)
    return value
def start(name,a):
    global node,label,run
    parent=stack[-1] if stack else None
    assert '|' not in name and not any('|' in k for k in a)
    if name=='CDXML':
        assert parent is None and set(a)=={'CreationProgram','LineWidth','MarginWidth','BondLength','BondSpacing'}
        style_raw.update({k:a[k] for k in ['LineWidth','MarginWidth','BondLength','BondSpacing']})
        style.update({k:numbers(a[k],1)[0] for k in style_raw})
        assert style=={'LineWidth':1.25,'MarginWidth':2.0,'BondLength':48.0,'BondSpacing':18.0}
    elif name in ('fonttable','colortable'):
        assert parent=='CDXML' and not a
    elif name=='font':
        assert parent=='fonttable' and set(a)=={'id','name'} and int(a['id']) not in fonts
        fonts[int(a['id'])]=a['name']
    elif name=='color':
        assert parent=='colortable' and set(a)=={'r','g','b'}
        colors.append([numbers(a[k],1)[0] for k in ['r','g','b']])
    elif name=='page':
        assert parent=='CDXML' and set(a)=={'id','DrawingSpace','Width','Height'} and a['DrawingSpace']=='Poster'
        assert numbers(a['Width'],1)==numbers(a['Height'],1)==[512.0]
    elif name=='fragment':
        assert parent=='page' and set(a)=={'id'}
    elif name=='n':
        assert parent=='fragment' and set(a)-set(flags)=={'id','NodeType','Element','Charge','Radical','NumHydrogens','p'}
        assert a['NodeType']=='Element' and a['Charge']=='0' and a['Radical']=='None'
        fs={k:a[k]=='yes' for k in flags if k in a};assert all(a[k] in ('no','yes') for k in fs)
        node={'id':int(a['id']),'model_id':len(nodes)+1,'element':int(a['Element']),'hydrogens':int(a['NumHydrogens']),
              'charge':int(a['Charge']),'radical':a['Radical'],'isotope':None,
              'point':numbers(a['p'],2),'flags':fs,'label':None}
        assert node['id']>0 and node['id'] not in [n['id'] for n in nodes]
        assert 0<=node['hydrogens']<=65535
        nodes.append(node)
    elif name=='t':
        assert parent=='n' and node is not None and node['label'] is None and not node['flags']
        assert set(a)=={'id','BoundingBox','Justification','LabelJustification'} and a['Justification']==a['LabelJustification']=='Left'
        label={'id':int(a['id']),'bounding_box':numbers(a['BoundingBox'],4),'text':''};node['label']=label
    elif name=='s':
        assert parent=='t' and label is not None and 'font' not in label
        assert set(a)=={'font','size','face','color'}
        fid,cid=int(a['font']),int(a['color'])
        assert fonts[fid]=='Roboto' and numbers(a['size'],1)==[12.0] and a['face']=='0'
        assert colors[cid-2]==[0.2,0.4,0.8]
        label.update(font_id=fid,font=fonts[fid],size=12.0,face=0,color_id=cid,rgb=colors[cid-2]);run=True
    elif name=='b':
        assert parent=='fragment' and set(a)=={'id','B','E','Order','Display','Display2','color'}
        assert a['Display']==a['Display2']=='Solid' and int(a['color'])==3 and colors[1]==[0,0,0]
        bonds.append({'id':int(a['id']),'model_id':len(bonds)+1,'begin':int(a['B']),'end':int(a['E']),'order':int(a['Order']),
                      'display':a['Display'],'display2':a['Display2'],'color_id':int(a['color'])})
    else: raise AssertionError('unsupported source child '+name)
    stack.append(name)
def end(name):
    global node,label,run
    assert stack.pop()==name
    if name=='n':node=None
    if name=='t':label=None
    if name=='s':run=False
def text(s):
    if run:label['text']+=s
    else:assert not s.strip()
def doctype(*_): raise AssertionError('no DTD/default expansion')
p=expat.ParserCreate(namespace_separator='|');p.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
p.StartElementHandler=start;p.EndElementHandler=end;p.CharacterDataHandler=text;p.StartDoctypeDeclHandler=doctype
p.ExternalEntityRefHandler=lambda *_:0;p.Parse(raw,True)
assert not stack and fonts=={42:'Roboto'} and colors==[[1,1,1],[0,0,0],[0.2,0.4,0.8]]
adj={n['id']:[] for n in nodes};by_id={n['id']:n for n in nodes};model=Chem.RWMol();indexes={}
for n in nodes:
    a=Chem.Atom(n['element']);a.SetNoImplicit(True);a.SetNumExplicitHs(n['hydrogens']);indexes[n['id']]=model.AddAtom(a)
    if n['label'] is not None:
        h=n['hydrogens'];body=a.GetSymbol()+('H'+(str(h) if h>1 else '') if h else '')
        assert n['label']['text']==body
    else:
        assert n['element']==6
for b in bonds:
    assert b['begin']!=b['end'] and b['order'] in (1,2,3)
    adj[b['begin']].append((b['end'],b['order']));adj[b['end']].append((b['begin'],b['order']))
    model.AddBond(indexes[b['begin']],indexes[b['end']],{1:Chem.BondType.SINGLE,2:Chem.BondType.DOUBLE,3:Chem.BondType.TRIPLE}[b['order']])
hidden=[];mandatory=[]
for n in nodes:
    if n['label'] is not None: continue
    neighbours=adj[n['id']];degree=len(neighbours);active=flags[0 if degree<=1 else 1]
    assert active in n['flags'] and n['flags'][active] is False
    assert sum(order for _,order in neighbours)+n['hydrogens']==4
    isolated=degree==0 or degree==1 and by_id[neighbours[0][0]]['element']==6 and len(adj[neighbours[0][0]])==1
    must=False
    if degree==2 and neighbours[0][1]==neighbours[1][1]:
        if neighbours[0][1]==2:must=True
        else:
            vectors=[[by_id[other]['point'][i]-n['point'][i] for i in (0,1)] for other,_ in neighbours]
            lengths=[math.hypot(*v) for v in vectors];assert all(length>0 and math.isfinite(length) for length in lengths)
            dot=sum(vectors[0][i]/lengths[0]*vectors[1][i]/lengths[1] for i in (0,1))
            must=dot<=-0.984807753012208
    (mandatory if isolated or must else hidden).append(n['model_id'])
model=model.GetMol();model.UpdatePropertyCache(strict=False)
formula=rdMolDescriptors.CalcMolFormula(model);smiles=Chem.MolToSmiles(model)
direct=Chem.MolsFromCDXML(raw.decode(),sanitize=False,removeHs=False);assert len(direct)==1
direct[0].UpdatePropertyCache(strict=False)
assert direct[0].GetNumAtoms()==len(nodes) and direct[0].GetNumBonds()==len(bonds)
assert rdMolDescriptors.CalcMolFormula(direct[0])==formula and Chem.MolToSmiles(direct[0])==smiles
result={'style':style,'style_raw':style_raw,'resources':{'fonts':fonts,'colors':colors},'nodes':nodes,'bonds':bonds,
        'source_to_model_ids':{str(n['id']):n['model_id'] for n in nodes},'literal_formula':formula,'rdkit_literal_smiles':smiles,
        'hidden_model_atoms':hidden,'mandatory_without_text':mandatory,
        'mixed_expected':bool(hidden and bonds and any(n['label'] is not None for n in nodes) and not mandatory)}
print(json.dumps(result,separators=(',',':')))
