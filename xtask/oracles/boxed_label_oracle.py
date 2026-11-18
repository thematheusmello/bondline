"""Original Expat/Decimal reader. Numeric RDKit formula is not label/valence authority."""
import json
import sys
import xml.parsers.expat as expat
from decimal import Decimal
from rdkit import Chem, rdBase
from rdkit.Chem import rdMolDescriptors
assert rdBase.rdkitVersion == '2026.09.2', 'use the pinned external interpreter'

raw=sys.stdin.buffer.read(1048577)
assert 0<len(raw)<=1048576
fonts={};colors=[];stack=[];scopes=[set()];node=None;text=None;run=None;nodes=[]
def number(raw):
 d=Decimal(raw);assert d.is_finite();return float(d)
def integer(raw):
 d=Decimal(raw);assert d.is_finite() and d==d.to_integral_value();return int(d)
def start(name,a):
 global node,text,run
 assert '|' not in name
 if 'id' in a and name!='font':
  key=integer(a['id']);assert key>0 and key not in scopes[-1];scopes[-1].add(key)
 stack.append(name);scopes.append(set())
 if name=='font':
  assert set(a)=={'id','name'};key=integer(a['id']);assert 0<=key<=65535 and key not in fonts;fonts[key]=a['name']
 elif name=='color':
  assert set(a)=={'r','g','b'};rgb=[number(a[k]) for k in ['r','g','b']];assert all(0<=x<=1 for x in rgb);colors.append(rgb)
 elif name=='page':assert a['DrawingSpace']=='Poster' and number(a['Width'])==512 and number(a['Height'])==512
 elif name=='n':
  assert set(a)=={'id','NodeType','Element','Charge','Radical','NumHydrogens','p'}
  assert a['NodeType']=='Element' and integer(a['Charge'])==0 and a['Radical']=='None'
  point=[number(s) for s in a['p'].split()];assert len(point)==2
  node={'element':integer(a['Element']),'hydrogens':integer(a['NumHydrogens']),'point':point};nodes.append(node)
 elif name=='t':
  assert node is not None and stack[-2]=='n' and set(a)=={'id','BoundingBox','Justification','LabelJustification'}
  assert a['Justification']==a['LabelJustification']=='Left'
  box=[number(s) for s in a['BoundingBox'].split()];assert len(box)==4 and box[0]<box[2] and box[1]<box[3]
  node['bounding_box']=box;text=node
 elif name=='s':
  assert text is not None and 'text' not in text and set(a)=={'font','size','face','color'}
  color=integer(a['color']);assert 2<=color<len(colors)+2
  text.update(text='',font=fonts[integer(a['font'])],size=number(a['size']),face=integer(a['face']),rgb=colors[color-2]);run=text
 elif name not in ['CDXML','fonttable','colortable','fragment']:raise AssertionError(name)
def chars(s):
 if run is not None:run['text']+=s
 else:assert not s.strip(' \t\r\n')
def end(name):
 global run,text,node
 assert stack.pop()==name;scopes.pop()
 if name=='s':run=None
 if name=='t':text=None
 if name=='n':node=None
p=expat.ParserCreate(namespace_separator='|');p.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
p.ExternalEntityRefHandler=lambda *unused:0
p.StartElementHandler=start;p.EndElementHandler=end;p.CharacterDataHandler=chars;p.Parse(raw,True)
assert len(nodes)==1
node=nodes[0];z=node['element'];h=node['hydrogens'];assert 1<=z<=118 and 0<=h<=65535
symbol=Chem.GetPeriodicTable().GetElementSymbol(z)
expected=symbol+('H'+(str(h) if h>1 else '') if h else '')
assert node['text']==expected and (z!=1 or h==0)
assert node['font']=='Roboto' and node['size']==12 and node['face']==0 and node['rgb']==[.2,.4,.8]
# No sanitize/valence or implicit H: independently count exactly the source numeric fields.
atom=Chem.Atom(z);atom.SetNoImplicit(True);atom.SetNumExplicitHs(h)
m=Chem.RWMol();m.AddAtom(atom);mol=m.GetMol();mol.UpdatePropertyCache(strict=False)
node['numeric_formula']=rdMolDescriptors.CalcMolFormula(mol)
print(json.dumps(node,ensure_ascii=False,separators=(',',':')))
