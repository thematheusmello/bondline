# SPDX-License-Identifier: MIT OR Apache-2.0
"""Strict explicit-run/default-metadata reader; Expat/Decimal/RDKit, no product imports."""
import json,math,re,sys,xml.parsers.expat as expat
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
metadata_records=[];default_lookup=[];scope_stack=[];metadata_fonts=[]
METADATA={'LabelFont':'font','LabelSize':'size_pt','LabelFace':'face'}
def scalar(name,decoded):
    value=decoded.strip(' \t\r\n')
    if name=='LabelSize':
        assert re.fullmatch(r'[+-]?(?:[0-9]+(?:\.[0-9]*)?|\.[0-9]+)(?:[eE][+-]?[0-9]+)?',value)
        number=float(value);assert math.isfinite(number) and number>0;return number
    assert re.fullmatch(r'[+-]?[0-9]+(?:\.0+)?',value)
    number=int(Decimal(value));assert 0<=number<=32767;return number
def start(name,a):
    allowed=name in ('CDXML','n','b','t')
    found={k:v for k,v in a.items() if k in METADATA}
    assert not found or allowed
    path=[{'name':x['name'],'id':x['id']} for x in scope_stack]+[{'name':name,'id':a.get('id')}]
    own={};begin=p.CurrentByteIndex;end=begin;quote=None
    while True:
        char=raw[end]
        if quote is not None:
            if char==quote:quote=None
        elif char in (34,39):quote=char
        elif char==62:break
        end+=1
    for key,decoded in found.items():
        typed=scalar(key,decoded)
        if key=='LabelFont':metadata_fonts.append(typed)
        m=re.search(rb'\b'+key.encode()+rb'\s*=\s*([\'"])(.*?)\1',raw[begin:end+1]);assert m
        record={'property':key,'value':typed,'decoded_value':decoded,'raw_value':m[2].decode(),'quote':m[1].decode(),
                'owner_name':name,'owner_id':a.get('id'),'owner_path':path,'owner_byte_offset':begin,
                'value_byte_span':[begin+m.start(2),begin+m.end(2)]}
        own[key]=record;metadata_records.append(record)
    scope_stack.append({'name':name,'id':a.get('id'),'metadata':own})
    if allowed:
        selected={}
        for key,field in METADATA.items():
            selected[field]=next((x['metadata'][key] for x in reversed(scope_stack) if key in x['metadata']),None)
        default_lookup.append({'target_name':name,'target_id':a.get('id'),'target_path':path,'selected':selected})
    base_start(name,{k:v for k,v in a.items() if k not in METADATA})
def end(name):
    base_end(name);assert scope_stack.pop()['name']==name

def base_start(name,a):
    global node,label,run,page_id,fragment_id
    parent=stack[-1] if stack else None
    assert '|' not in name and not any('|' in k for k in a)
    if 'id' in a and name!='font':
        ident=int(a['id']);assert 1<=ident<=4294967295 and ident not in objects;objects[ident]=name
    if name=='CDXML':
        assert parent is None and set(a)<={'CreationProgram','LineWidth','MarginWidth','BondLength','BondSpacing'} and 'CreationProgram' in a
        style_raw.update({k:a[k] for k in ['LineWidth','MarginWidth','BondLength','BondSpacing'] if k in a});style.update({k:nums(v,1)[0] for k,v in style_raw.items()})
        assert all(value=={'LineWidth':1.25,'MarginWidth':2.0,'BondLength':48.0,'BondSpacing':18.0}[key] for key,value in style.items())
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
def base_end(name):
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
p.StartElementHandler=start;p.EndElementHandler=end;p.CharacterDataHandler=text;p.CommentHandler=comment;p.StartDoctypeDeclHandler=dtd;p.ExternalEntityRefHandler=lambda *_:0
p.StartNamespaceDeclHandler=lambda *_:(_ for _ in ()).throw(AssertionError('no namespace admission'))
p.ProcessingInstructionHandler=lambda *_:(_ for _ in ()).throw(AssertionError('no PI admission'))
p.StartCdataSectionHandler=lambda:(_ for _ in ()).throw(AssertionError('no CDATA admission'))
p.Parse(raw,True)
assert metadata_records and all(index in fonts for index in metadata_fonts)
assert not stack and colors==[[1,1,1],[0,0,0],[0.2,0.4,0.8]] and fonts and all(isinstance(name,str) and name for name in fonts.values())
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
        'metadata_records':metadata_records,'default_lookup':default_lookup,'literal_formula':formula,'rdkit_literal_smiles':smiles,'all_actual_styles_explicit':True,'current_hidden_valid':not mandatory,'metadata_present':bool(metadata_records)}
print(json.dumps(result,separators=(',',':')))
