# SPDX-License-Identifier: MIT OR Apache-2.0
"""Independent actual-file/FontTools checks, no product output or product execution."""
from pathlib import Path
import hashlib,importlib.util,json,os,re,subprocess
ROOT=Path(__file__).resolve().parent
REPO=next(p for p in ROOT.parents if (p/'xtask/oracles/clearance_oracle.py').is_file())
OUTPUT=Path(os.environ.get('BONDLINE_ORACLE_OUTPUT',REPO/'target/mixed-labels/independent'))
OUTPUT.mkdir(parents=True,exist_ok=True)
PYTHON=Path(os.environ.get('BONDLINE_PYTHON',REPO/'.cache/oracles/format-python/bin/python'))
spec=importlib.util.spec_from_file_location('independent_font',REPO/'xtask/oracles/clearance_oracle.py')
font=importlib.util.module_from_spec(spec);spec.loader.exec_module(font)
def consume(raw,valid=True):
    p=subprocess.run([str(PYTHON),'-B',str(ROOT/'mixed_label_oracle.py')],input=raw,capture_output=True)
    assert p.returncode==0 if valid else p.returncode!=0,p.stderr.decode()
    return json.loads(p.stdout) if valid else None
manifest=json.loads((ROOT/'mixed-vectors.json').read_text());facts={};geometry=[]
for case in manifest['cases']:
    raw=(ROOT/(case['name']+'.cdxml')).read_bytes()
    assert len(raw)==case['bytes'] and hashlib.sha256(raw).hexdigest()==case['sha256']
    values=consume(raw);facts[case['name']]=values
    assert values['mixed_expected']==case['mixed_expected']
    for node in values['nodes']:
        if node['label'] is None:continue
        symbol={6:'C',7:'N',8:'O'}[node['element']]
        body,glyphs=font.authored(symbol,node['hydrogens'],12,node['point']);box=font.placed_box(glyphs)
        assert body==node['label']['text']
        error=max(abs(a-b) for a,b in zip(node['label']['bounding_box'],box));assert error<=1e-12
        geometry.append({'case':case['name'],'source_node':node['id'],'model_atom':node['model_id'],'font_box':box,'max_point_error':error})
for name,formula,hidden,mandatory in [
 ('amino_alcohol','C3H9NO',[2,3,4],[]),('amino_diamine','C3H10N2',[2,3,4],[]),
 ('methanol','CH4O',[1],[]),('methanol_bad_c2','C2H6',[],[1]),('mandatory_collinear','C3H9NO',[2,4],[3]),
]:
    x=facts[name];assert x['literal_formula']==formula and x['hidden_model_atoms']==hidden and x['mandatory_without_text']==mandatory
base=(ROOT/'amino_alcohol.cdxml').read_text()
def node_attr(raw,name,value):
    return re.sub(r"(<n id='97'[^>]*\b"+name+r"=')[^']*'",lambda m:m[1]+value+"'",raw,count=1)
controls={
 'active_flag_yes':node_attr(base,'ShowNonTerminalCarbonLabels','yes'),
 'drop_active_flag':base.replace(" ShowNonTerminalCarbonLabels='no'",'',1),
 'invalid_C4':node_attr(base,'NumHydrogens','1'),
 'missing_H':re.sub(r"(<n id='97'[^>]*?) NumHydrogens='2'",r'\1',base,count=1),
 'noncarbon_no_text':node_attr(base,'Element','7'),
 'charge':node_attr(base,'Charge','1'),
 'radical':node_attr(base,'Radical','Doublet'),
 'isotope':base.replace("<n id='97' ","<n id='97' Isotope='13' ",1),
 'unsupported_hidden_child':base.replace(" ShowNonTerminalCarbonLabels='no'/>"," ShowNonTerminalCarbonLabels='no'><t unsupported='retain'/></n>",1),
 'conflicting_printed_body':base.replace('>OH</s>','>N</s>',1),
 'missing_ink':base.replace(" LineWidth='1.2500'",'',1),
}
for key,raw in controls.items():assert raw!=base,key;consume(raw.encode(),False)
dormant=consume(base.replace("ShowTerminalCarbonLabels='yes'","ShowTerminalCarbonLabels='no'",1).encode())
assert dormant!=facts['amino_alcohol'] and dormant['hidden_model_atoms']==[2,3,4]
for before,after,selected,dx,dy in [
 ('amino_alcohol','hidden_middle_p12_5_m7_25',{3},12.5,-7.25),
 ('amino_alcohol','all_p24_m12',{1,2,3,4,5},24,-12),
 ('amino_alcohol','subset_1_3_5_p12_5_m7_25',{1,3,5},12.5,-7.25),
 ('amino_diamine','edited_all_p24_m12',{1,2,3,4,5},24,-12),
]:
    a,b=facts[before],facts[after]
    assert a['style']==b['style'] and a['style_raw']==b['style_raw'] and a['resources']==b['resources'] and a['bonds']==b['bonds']
    for old,new in zip(a['nodes'],b['nodes']):
        expected=json.loads(json.dumps(old))
        if old['model_id'] in selected:
            expected['point']=[old['point'][0]+dx,old['point'][1]+dy]
            if old['label'] is not None:expected['label']['bounding_box']=[v+(dx if i%2==0 else dy) for i,v in enumerate(old['label']['bounding_box'])]
        assert expected==new
(OUTPUT/'mixed-label-facts.json').write_text(json.dumps(facts,indent=2)+'\n')
out={'status':'passed','cases':len(facts),'fonttools':font.fontTools.__version__,'font_sha256':font.FONT_SHA,'rdkit':'2026.09.2',
     'formula_claim':'literal original fields only; no general valence/inference certification','font_geometry':geometry,
     'reader_refusal_controls':list(controls),'dormant_flag_change_detected':True,'product_execution':False}
(OUTPUT/'independent-checks.json').write_text(json.dumps(out,indent=2)+'\n')
print(json.dumps({'status':'passed','sources':len(facts),'font_boxes':len(geometry),'reader_refusals':len(controls),'facts_sha256':hashlib.sha256((OUTPUT/'mixed-label-facts.json').read_bytes()).hexdigest()}))
