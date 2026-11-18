# SPDX-License-Identifier: MIT OR Apache-2.0
"""Strict independent actual-file creation reader; Expat/Decimal/RDKit, no product imports."""
import json,math,sys,xml.parsers.expat as expat
from decimal import Decimal
from rdkit import Chem,rdBase
from rdkit.Chem import rdMolDescriptors
assert rdBase.rdkitVersion=='2026.09.2'
raw=sys.stdin.buffer.read(65537);assert 0<len(raw)<=65536
fonts={};colors=[];nodes=[];bonds=[];captions=[];objects={};style={};style_raw={};stack=[];node=None;label=None;run=False;page_id=None;fragment_id=None
flag_names=['ShowTerminalCarbonLabels','ShowNonTerminalCarbonLabels']
def nums(s,n):
    result=[float(Decimal(x)) for x in s.split()];assert len(result)==n and all(math.isfinite(x) for x in result);return result
def color(index):
    assert 0<=index<=65535
    if index==0:return [0.0,0.0,0.0]
    if index==1:return [1.0,1.0,1.0]
    return colors[index-2]
def start(name,a):
    global node,label,run,page_id,fragment_id
    parent=stack[-1] if stack else None
    assert '|' not in name and not any('|' in k for k in a)
    if 'id' in a and name!='font':
        ident=int(a['id']);assert 1<=ident<=4294967295 and ident not in objects;objects[ident]=name
    if name=='CDXML':
        assert parent is None and set(a)=={'CreationProgram','LineWidth','MarginWidth','BondLength','BondSpacing'}
        style_raw.update({k:a[k] for k in ['LineWidth','MarginWidth','BondLength','BondSpacing']});style.update({k:nums(v,1)[0] for k,v in style_raw.items()})
        assert style=={'LineWidth':1.25,'MarginWidth':2.0,'BondLength':48.0,'BondSpacing':18.0}
    elif name in ('fonttable','colortable'):assert parent=='CDXML' and not a
    elif name=='font':
        assert parent=='fonttable' and set(a)=={'id','name'};fid=int(a['id']);assert 0<=fid<=65535 and fid not in fonts;fonts[fid]=a['name']
    elif name=='color':
        assert parent=='colortable' and set(a)=={'r','g','b'};colors.append([nums(a[k],1)[0] for k in ('r','g','b')])
    elif name=='page':
        assert parent=='CDXML' and set(a)=={'id','DrawingSpace','Width','Height'} and a['DrawingSpace']=='Poster';assert nums(a['Width'],1)==nums(a['Height'],1)==[512.0];page_id=int(a['id'])
    elif name=='fragment':assert parent=='page' and set(a)=={'id'};fragment_id=int(a['id'])
    elif name=='n':
        assert parent=='fragment' and set(a)-set(flag_names)=={'id','NodeType','Element','Charge','Radical','NumHydrogens','p'}
        assert a['NodeType']=='Element' and a['Charge']=='0' and a['Radical']=='None';assert all(a[k] in ('yes','no') for k in flag_names if k in a)
        node={'id':int(a['id']),'model_id':len(nodes)+1,'element':int(a['Element']),'hydrogens':int(a['NumHydrogens']),'charge':0,'isotope':None,'radical':'None',
              'point':nums(a['p'],2),'flags':{k:a[k]=='yes' for k in flag_names if k in a},'label':None,'comments':[]};assert 0<=node['hydrogens']<=65535;nodes.append(node)
    elif name=='t':
        assert parent in ('n','page')
        if parent=='n':
            assert node is not None and node['label'] is None;assert set(a)=={'id','BoundingBox','Justification','LabelJustification'} and a['Justification']==a['LabelJustification']=='Left'
            label={'id':int(a['id']),'bounding_box':nums(a['BoundingBox'],4),'text':''};node['label']=label
        else:
            assert set(a)=={'id','p','Justification'} and a['Justification']=='Left';label={'id':int(a['id']),'model_id':len(captions)+1,'point':nums(a['p'],2),'text':''};captions.append(label)
    elif name=='s':
        assert parent=='t' and label is not None and 'font_id' not in label and set(a)=={'font','size','face','color'}
        fid,cid=int(a['font']),int(a['color']);size=nums(a['size'],1)[0];assert fonts[fid]=='Roboto' and 0<size<=100000 and a['face']=='0';rgb=color(cid);assert all(0<=v<=1 for v in rgb)
        label.update(font_id=fid,font=fonts[fid],size=size,face=0,color_id=cid,rgb=rgb);run=True
    elif name=='b':
        assert parent=='fragment' and set(a)=={'id','B','E','Order','Display','Display2','color'} and a['Display']==a['Display2']=='Solid' and color(int(a['color']))==[0,0,0]
        bonds.append({'id':int(a['id']),'model_id':len(bonds)+1,'begin':int(a['B']),'end':int(a['E']),'order':int(a['Order']),'color_id':int(a['color'])})
    else:raise AssertionError('unsupported source object '+name)
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
def comment(s):
    if node is not None:node['comments'].append(s)
def dtd(*_):raise AssertionError('no DTD/default inference')
p=expat.ParserCreate(namespace_separator='|');p.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
p.StartElementHandler=start;p.EndElementHandler=end;p.CharacterDataHandler=text;p.CommentHandler=comment;p.StartDoctypeDeclHandler=dtd;p.ExternalEntityRefHandler=lambda *_:0;p.Parse(raw,True)
assert not stack and colors==[[1,1,1],[0,0,0],[0.2,0.4,0.8]] and set(fonts.values())=={'Roboto'}
adj={n['id']:[] for n in nodes};by_id={n['id']:n for n in nodes};mol=Chem.RWMol();indexes={}
for n in nodes:
    a=Chem.Atom(n['element']);a.SetNoImplicit(True);a.SetNumExplicitHs(n['hydrogens']);indexes[n['id']]=mol.AddAtom(a)
    if n['label'] is not None:
        h=n['hydrogens'];expected=a.GetSymbol()+('H'+(str(h) if h>1 else '') if h else '');assert n['label']['text']==expected
    else:assert n['element']==6
for b in bonds:
    assert b['order'] in (1,2,3) and b['begin']!=b['end'];adj[b['begin']].append((b['end'],b['order']));adj[b['end']].append((b['begin'],b['order']))
    mol.AddBond(indexes[b['begin']],indexes[b['end']],{1:Chem.BondType.SINGLE,2:Chem.BondType.DOUBLE,3:Chem.BondType.TRIPLE}[b['order']])
hidden=[];mandatory=[];flagged=[]
for n in nodes:
    if n['label'] is not None:
        if n['flags']:flagged.append(n['model_id'])
        continue
    neighbours=adj[n['id']];degree=len(neighbours);active=flag_names[0 if degree<=1 else 1]
    assert active in n['flags'] and n['flags'][active] is False;assert sum(order for _,order in neighbours)+n['hydrogens']==4
    isolated=degree==0 or degree==1 and by_id[neighbours[0][0]]['element']==6 and len(adj[neighbours[0][0]])==1
    must=False
    if degree==2 and neighbours[0][1]==neighbours[1][1]:
        if neighbours[0][1]==2:must=True
        else:
            rays=[[by_id[other]['point'][i]-n['point'][i] for i in (0,1)] for other,_ in neighbours];lengths=[math.hypot(*v) for v in rays];assert all(v>0 for v in lengths)
            must=sum(rays[0][i]/lengths[0]*rays[1][i]/lengths[1] for i in (0,1))<=-0.984807753012208
    (mandatory if isolated or must else hidden).append(n['model_id'])
mol=mol.GetMol();mol.UpdatePropertyCache(strict=False);formula=rdMolDescriptors.CalcMolFormula(mol);smiles=Chem.MolToSmiles(mol)
direct=Chem.MolsFromCDXML(raw.decode(),sanitize=False,removeHs=False);assert len(direct)==1;direct[0].UpdatePropertyCache(strict=False)
assert direct[0].GetNumAtoms()==len(nodes) and direct[0].GetNumBonds()==len(bonds) and rdMolDescriptors.CalcMolFormula(direct[0])==formula and Chem.MolToSmiles(direct[0])==smiles
result={'source_page_id':page_id,'source_fragment_id':fragment_id,'style':style,'style_raw':style_raw,'resources':{'fonts':fonts,'colors':colors},'nodes':nodes,'bonds':bonds,'captions':captions,
        'object_ids':{str(k):v for k,v in sorted(objects.items())},'source_to_model_ids':{str(n['id']):n['model_id'] for n in nodes},'hidden_model_atoms':hidden,'flagged_printed_atoms':flagged,'mandatory_without_text':mandatory,
        'literal_formula':formula,'rdkit_literal_smiles':smiles,'flagged_expected':bool(flagged and bonds and not mandatory)}
print(json.dumps(result,separators=(',',':')))
