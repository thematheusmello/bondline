# SPDX-License-Identifier: MIT OR Apache-2.0
"""Original connected CDXML fixtures from independent FontTools, never Bondline rendering."""
from pathlib import Path
import json,hashlib
import fontTools
from fontTools.ttLib import TTFont
from fontTools.pens.boundsPen import BoundsPen
OUT=Path(__file__).resolve().parent
WORKSPACE=OUT.parents[4]
assert fontTools.__version__=='4.61.1'
font_path=WORKSPACE/'assets/fonts/Roboto-Regular.ttf'
assert hashlib.sha256(font_path.read_bytes()).hexdigest()=='56a45233d29f11b4dfb86d248e921939d115778f87325e7ae8cc108383d6664d'
font=TTFont(font_path);glyphs=font.getGlyphSet();cmap=font.getBestCmap();upem=font['head'].unitsPerEm
gpos=font['GPOS'].table
lang=next(r.Script.DefaultLangSys for r in gpos.ScriptList.ScriptRecord if r.ScriptTag=='DFLT')
lookups=[n for i in lang.FeatureIndex if gpos.FeatureList.FeatureRecord[i].FeatureTag=='kern' for n in gpos.FeatureList.FeatureRecord[i].Feature.LookupListIndex]
def kern(a,b):
 names=(cmap[ord(a)],cmap[ord(b)]);amount=0
 for index in lookups:
  lookup=gpos.LookupList.Lookup[index];assert lookup.LookupType==2 and lookup.LookupFlag==0
  for table in lookup.SubTable:
   if names[0] not in table.Coverage.glyphs:continue
   if table.Format==1:
    record=next((r for r in table.PairSet[table.Coverage.glyphs.index(names[0])].PairValueRecord if r.SecondGlyph==names[1]),None)
    if record is None:continue
    amount+=record.Value1.XAdvance
   else:
    assert table.Format==2
    amount+=table.Class1Record[table.ClassDef1.classDefs.get(names[0],0)].Class2Record[table.ClassDef2.classDefs.get(names[1],0)].Value1.XAdvance
   break
 return amount
symbols={1:'H',6:'C',7:'N',8:'O'}
def label(element,h,point,size=12):
 symbol=symbols[element];body=symbol+('H'+(str(h) if h>1 else '') if h else '')
 x=0;boxes=[];parts=[]
 for i,ch in enumerate(body):
  if i:x+=kern(body[i-1],ch)
  pen=BoundsPen(glyphs);name=cmap[ord(ch)];glyphs[name].draw(pen);a,b,c,d=pen.bounds
  box=[x+a,b,x+c,d];boxes.append(box);parts.append({'character':ch,'outline_bounds':list(pen.bounds),'pen_x_units':x})
  x+=font['hmtx'][name][0]
 union=lambda bs:[min(b[0] for b in bs),min(b[1] for b in bs),max(b[2] for b in bs),max(b[3] for b in bs)]
 full=union(boxes);token=union(boxes[:len(symbol)]);f=size/upem
 base=[point[0]-(token[0]+token[2])*f/2,point[1]+(token[1]+token[3])*f/2]
 bounds=[base[0]+full[0]*f,base[1]-full[3]*f,base[0]+full[2]*f,base[1]-full[1]*f]
 return {'element':element,'h':h,'text':body,'point':point,'size_pt':size,'bounding_box':bounds,'baseline':base,'glyphs':parts}
def num(value):return format(value,'.17g')
def make(name,entries,edges,multiple=False):
 attrs=" LineWidth='.60' MarginWidth='1.600'"+(" BondLength='48.000' BondSpacing='12.0'" if multiple else '')
 nodes=[];facts=[]
 for ident,element,h,point in entries:
  v=label(element,h,point);facts.append({'id':ident,**v})
  bounds=' '.join(map(num,v['bounding_box']));p=' '.join(map(num,point))
  nodes.append(f"<n id='{ident}' NodeType='Element' Element='{element}' Charge='0' Radical='None' NumHydrogens='{h}' p='{p}'><t id='21' BoundingBox='{bounds}' Justification='Left' LabelJustification='Left'><s font='42' size='12' face='0' color='4'>{v['text']}</s></t></n>")
 bonds=[]
 for ident,a,b,order in edges:
  pos=" DoublePosition='Center'" if order==2 else ''
  bonds.append(f"<b id='{ident}' B='{a}' E='{b}' Order='{order}'{pos}/>")
 source=f"<CDXML CreationProgram='Original independent connected label vectors'{attrs}><fonttable><font id='42' name='Roboto'/></fonttable><colortable><color r='1' g='1' b='1'/><color r='0' g='0' b='0'/><color r='.2' g='.4' b='.8'/></colortable><page id='91' DrawingSpace='Poster' Width='512' Height='512'><fragment id='901'>"+''.join(nodes+bonds)+"</fragment></page></CDXML>"
 return {'name':name,'source':source,'nodes':facts,'bonds':edges}
cases=[make('water',[(71,1,0,[80,100]),(207,8,0,[128,100]),(13,1,0,[176,100])],[(700,71,207,1),(702,207,13,1)]),make('carbon-dioxide',[(23,8,0,[80,180]),(88,6,0,[128,180]),(75,8,0,[176,180])],[(40,23,88,2),(92,88,75,2)],True),make('hydrogen-cyanide',[(6,1,0,[80,260]),(99,6,0,[128,260]),(42,7,0,[176,260])],[(55,6,99,1),(56,99,42,3)],True)]
out={'scope':'Original source/FontTools vectors. No Bondline renderer or reference exporter used.','fonttools':fontTools.__version__,'font_sha256':hashlib.sha256(font_path.read_bytes()).hexdigest(),'units_per_em':upem,'gpos_NH':kern('N','H'),'cases':cases,'ammonia_middle':label(7,1,[128,100])}
(OUT/'vectors.json').write_text(json.dumps(out,indent=2)+'\n')
print(json.dumps({'cases':len(cases),'gpos_NH':out['gpos_NH'],'nh_box':out['ammonia_middle']['bounding_box'],'fonttools':fontTools.__version__}))
