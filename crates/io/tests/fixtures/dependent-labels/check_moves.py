# SPDX-License-Identifier: MIT OR Apache-2.0
"""Independent original source facts and font geometry, no product renderer or writer."""
from pathlib import Path
import hashlib,importlib.util,json,subprocess
ROOT=Path(__file__).resolve().parent
REPO=next(p for p in ROOT.parents if (p/'xtask/oracles/clearance_oracle.py').is_file())
OUTPUT=REPO/'target/dependent-labels/independent'
OUTPUT.mkdir(parents=True,exist_ok=True)
spec=importlib.util.spec_from_file_location('independent_font',REPO/'xtask/oracles/clearance_oracle.py')
font=importlib.util.module_from_spec(spec);spec.loader.exec_module(font)
PYTHON=REPO/'.cache/oracles/format-python/bin/python'
facts={};geometry=[]
for case in json.loads((ROOT/'move-vectors.json').read_text())['cases']:
    raw=(ROOT/(case['name']+'.cdxml')).read_bytes()
    result=subprocess.run([str(PYTHON),'-B',str(ROOT/'dependent_label_oracle.py')],input=raw,capture_output=True,check=True)
    values=json.loads(result.stdout);facts[case['name']]=values
    for node in values['nodes']:
        symbol={1:'H',6:'C',7:'N',8:'O'}[node['element']]
        body,glyphs=font.authored(symbol,node['hydrogens'],12,node['point'])
        expected=font.placed_box(glyphs);actual=node['label']['bounding_box']
        assert body==node['label']['text']
        error=max(abs(a-b) for a,b in zip(actual,expected))
        assert error<=1/65536+1e-10
        geometry.append({'case':case['name'],'source_node':node['id'],'expected_font_box':expected,'actual_raw_box':actual,'max_point_error':error})
for before,after,ids,dx,dy in [
 ('boxed','boxed_sub_p12_5_m7_25',{73},12.5,-7.25),
 ('boxed_tolerated_raw','boxed_tolerated_sub_p12_5_m7_25',{73},12.5,-7.25),
 ('boxed_pair','boxed_pair_all_p24_m12',{73,97},24,-12),
 ('connected','connected_sub_p12_5_m7_25',{73},12.5,-7.25),
 ('connected','connected_all_p24_m12',{73,97,109},24,-12),
 ('connected_nh','connected_nh_sub_p12_5_m7_25',{73},12.5,-7.25),
 ('connected_component','connected_component_p24_m12',{73,97,109},24,-12),
]:
    a,b=facts[before],facts[after];assert a['style']==b['style'] and a['bonds']==b['bonds'] and a['literal_formula']==b['literal_formula']
    for old,new in zip(a['nodes'],b['nodes']):
        expected=json.loads(json.dumps(old))
        if old['id'] in ids:
            expected['point']=[old['point'][0]+dx,old['point'][1]+dy]
            expected['label']['bounding_box']=[v+(dx if i%2==0 else dy) for i,v in enumerate(old['label']['bounding_box'])]
        assert expected==new
assert 128.0+1e-14==128.0
assert facts['boxed_rounding_boundary']['nodes'][0]['label']['bounding_box'][0]+1e-14!=facts['boxed_rounding_boundary']['nodes'][0]['label']['bounding_box'][0]
(OUTPUT/'dependent-label-facts.json').write_text(json.dumps(facts,indent=2)+'\n')
out={'cases':len(facts),'fonttools':font.fontTools.__version__,'font_sha256':font.FONT_SHA,'geometry':geometry,'source_writer':'independent Expat/Decimal splices','product_execution':False,'status':'passed'}
(OUTPUT/'independent-checks.json').write_text(json.dumps(out,indent=2)+'\n')
print(json.dumps({'status':'passed','sources':len(facts),'font_boxes':len(geometry),'facts_sha256':hashlib.sha256((OUTPUT/'dependent-label-facts.json').read_bytes()).hexdigest()}))
