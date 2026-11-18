# SPDX-License-Identifier: MIT OR Apache-2.0
"""Independent actual-file/FontTools qualification; no product execution or expected output."""
from pathlib import Path
import argparse,hashlib,importlib.util,json,os,re,subprocess
SCRIPT_DIR=Path(__file__).resolve().parent
REPO=next(p for p in SCRIPT_DIR.parents if (p/'assets/fonts/Roboto-Regular.ttf').is_file())
parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--input-dir',type=Path,default=SCRIPT_DIR);parser.add_argument('--output-dir',type=Path,default=REPO/'target/label-creation/reproduced-checks');parser.add_argument('--python',type=Path,default=Path(os.environ.get('BONDLINE_PYTHON',REPO/'.cache/oracles/format-python/bin/python')))
args=parser.parse_args();ROOT=args.input_dir;OUT=args.output_dir;OUT.mkdir(parents=True,exist_ok=True);PYTHON=args.python
spec=importlib.util.spec_from_file_location('font_oracle',REPO/'xtask/oracles/clearance_oracle.py');font=importlib.util.module_from_spec(spec);spec.loader.exec_module(font)
def read(raw,valid=True):
    r=subprocess.run([str(PYTHON),'-B',str(SCRIPT_DIR/'created_label_oracle.py')],input=raw,capture_output=True)
    assert (r.returncode==0 if valid else r.returncode!=0),r.stderr.decode()
    return json.loads(r.stdout) if valid else None
manifest=json.loads((ROOT/'creation-vectors.json').read_text());facts={};geometry=[]
for case in manifest['cases']:
    raw=(ROOT/(case['name']+'.cdxml')).read_bytes();assert len(raw)==case['bytes'] and hashlib.sha256(raw).hexdigest()==case['sha256']
    v=read(raw);facts[case['name']]=v;assert v['flagged_expected']==case['flagged_expected']
    for n in v['nodes']:
        if n['label'] is None:continue
        text,gs=font.authored({6:'C',7:'N',8:'O'}[n['element']],n['hydrogens'],n['label']['size'],n['point']);box=font.placed_box(gs)
        assert text==n['label']['text'];error=max(abs(a-b) for a,b in zip(box,n['label']['bounding_box']));assert error<=1e-12
        geometry.append({'case':case['name'],'source_node':n['id'],'model_atom':n['model_id'],'body':text,'font_box':box,'max_point_error':error})
    assert all(n['charge']==0 and n['isotope'] is None and n['radical']=='None' for n in v['nodes'])
    assert v['literal_formula']==('C2H8N2O' if case['name'] in ('created_3_nh','created_3_old48_nh') else 'C3H10N2' if case['name']=='endpoint_edited_created_3' else 'C3H9NO')
for name,v in facts.items():
    expected=[{'ShowNonTerminalCarbonLabels':False},{'ShowTerminalCarbonLabels':True,'ShowNonTerminalCarbonLabels':False},{'ShowTerminalCarbonLabels':False,'ShowNonTerminalCarbonLabels':False}]
    assert [n['flags'] for n in v['nodes'][1:4]]==expected
assert facts['created_all_black']['hidden_model_atoms']==[] and facts['created_all_black']['flagged_printed_atoms']==[2,3,4]
assert facts['created_3_black']['hidden_model_atoms']==[2,4]
assert facts['dense_created_3']['nodes'][2]['label']['id']==15
assert facts['dense_created_3_2']['nodes'][1]['label']['id']==14 and facts['dense_created_3_2']['nodes'][2]['label']['id']==15
assert facts['dense_created_all']['nodes'][3]['label']['id']==16 and '14' in facts['dense_created_all']['resources']['fonts']
assert facts['caption_created_3']['captions'][0]['id']==1 and facts['caption_created_3']['nodes'][2]['label']['id']==3
assert facts['paired_created_3']['nodes'][2]['comments']==[' retain original node trivia ']
assert facts['created_3_black']['nodes'][2]['label']['color_id']==0
assert facts['created_3_blue14']['nodes'][2]['label']['color_id']==4 and facts['created_3_blue14']['nodes'][2]['label']['size']==14
for before,after,ids,dx,dy in [('created_3_black','created_3_all_p24_m12',{1,2,3,4,5},24,-12),('created_3_black','created_3_selected_p12_5_m7_25',{3,4},12.5,-7.25),('created_all_black','created_all_p24_m12',{1,2,3,4,5},24,-12)]:
    a,b=facts[before],facts[after];assert a['bonds']==b['bonds'] and a['resources']==b['resources'] and a['style_raw']==b['style_raw']
    for old,new in zip(a['nodes'],b['nodes']):
        expected=json.loads(json.dumps(old))
        if old['model_id'] in ids:
            expected['point']=[old['point'][0]+dx,old['point'][1]+dy]
            if old['label'] is not None:expected['label']['bounding_box']=[v+(dx if i%2==0 else dy) for i,v in enumerate(old['label']['bounding_box'])]
        assert expected==new
created=(ROOT/'created_3_black.cdxml').read_text()
controls={
 'object_id_collision':created.replace('<t id="2"','<t id="21"',1),
 'missing_font':created.replace('font="42"','font="43"',1),
 'unsupported_face':created.replace('face="0"','face="1"',1),
 'bad_body':created.replace('>CH2</s>','>N</s>',1),
 'unknown_child':created.replace('<t id="2"','<t future="retain" id="2"',1),
 'missing_current_H':created.replace("NumHydrogens='2' p='168 208'","p='168 208'",1),
 'charge':created.replace("Element='6' Charge='0'","Element='6' Charge='1'",1),
 'missing_ink':created.replace(" LineWidth='1.2500'",'',1),
}
for name,raw in controls.items():assert raw!=created,name;read(raw.encode(),False)
dropped=read(created.replace("ShowTerminalCarbonLabels='yes' ",'',1).encode());assert dropped!=facts['created_3_black']
(OUT/'creation-facts.json').write_text(json.dumps(facts,indent=2)+'\n')
result={'status':'passed','cases':len(facts),'font_boxes':len(geometry),'fonttools':font.fontTools.__version__,'font_sha256':font.FONT_SHA,'rdkit':'2026.09.2','product_execution':False,
        'geometry':geometry,'strict_reader_refusals':list(controls),'changed_retained_flags_detected':True,'formula_scope':'Literal independently parsed fields, no implicit H or general chemistry certification'}
(OUT/'independent-checks.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps({'status':'passed','sources':len(facts),'font_boxes':len(geometry),'reader_refusals':len(controls),'facts_sha256':hashlib.sha256((OUT/'creation-facts.json').read_bytes()).hexdigest()}))
