# SPDX-License-Identifier: MIT OR Apache-2.0
"""Independent literal source/geometry reader. No implicit H, valence or source authority."""
import json,sys,xml.parsers.expat as expat
from decimal import Decimal
from rdkit import Chem,rdBase
from rdkit.Chem import rdMolDescriptors
assert rdBase.rdkitVersion=='2026.09.2'
raw=sys.stdin.buffer.read(65537);assert 0<len(raw)<=65536
fonts={};colors=[];nodes=[];bonds=[];style={};node=None;label=None;run=False
def start(name,a):
 global node,label,run
 assert '|' not in name
 if name=='CDXML':style.update({k:float(Decimal(a[k])) for k in ['LineWidth','MarginWidth','BondLength','BondSpacing'] if k in a})
 if name=='font':fonts[int(a['id'])]=a['name']
 if name=='color':colors.append([float(Decimal(a[k])) for k in ['r','g','b']])
 if name=='n':
  node={'id':int(a['id']),'element':int(a['Element']),'hydrogens':int(a['NumHydrogens']),'point':[float(Decimal(x)) for x in a['p'].split()]};nodes.append(node)
 if name=='t' and node is not None:
  assert 'p' not in a and 'LabelAlignment' not in a and a['Justification']==a['LabelJustification']=='Left'
  label={'id':int(a['id']),'bounding_box':[float(Decimal(x)) for x in a['BoundingBox'].split()],'text':''};node['label']=label
 if name=='s' and label is not None:
  run=True;label.update(font=fonts[int(a['font'])],size=float(Decimal(a['size'])),face=int(a['face']),rgb=colors[int(a['color'])-2])
 if name=='b':bonds.append({'id':int(a['id']),'begin':int(a['B']),'end':int(a['E']),'order':int(a['Order'])})
def end(name):
 global node,label,run
 if name=='n':node=None
 if name=='t':label=None
 if name=='s':run=False
def text(s):
 if run:label['text']+=s
p=expat.ParserCreate(namespace_separator='|');p.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
p.StartElementHandler=start;p.EndElementHandler=end;p.CharacterDataHandler=text;p.ExternalEntityRefHandler=lambda *_:0
p.Parse(raw,True)
assert colors[0]==[1,1,1] and colors[1]==[0,0,0]
mol=Chem.RWMol();indexes={}
for n in nodes:
 a=Chem.Atom(n['element']);a.SetNoImplicit(True);a.SetNumExplicitHs(n['hydrogens']);indexes[n['id']]=mol.AddAtom(a)
 symbol=a.GetSymbol();h=n['hydrogens'];body=symbol+('H'+(str(h) if h>1 else '') if h else '')
 assert n['label']['text']==body and n['label']['font']=='Roboto' and n['label']['face']==0 and n['label']['size']==12
for b in bonds:mol.AddBond(indexes[b['begin']],indexes[b['end']],{1:Chem.BondType.SINGLE,2:Chem.BondType.DOUBLE,3:Chem.BondType.TRIPLE}[b['order']])
mol=mol.GetMol();mol.UpdatePropertyCache(strict=False)
result={'style':style,'nodes':nodes,'bonds':bonds,'literal_formula':rdMolDescriptors.CalcMolFormula(mol)}
direct=Chem.MolsFromCDXML(raw.decode(),sanitize=False,removeHs=False)
assert len(direct)==1
direct[0].UpdatePropertyCache(strict=False);assert rdMolDescriptors.CalcMolFormula(direct[0])==result['literal_formula']
print(json.dumps(result,separators=(',',':')))
