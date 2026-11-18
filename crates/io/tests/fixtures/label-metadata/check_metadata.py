# SPDX-License-Identifier: MIT OR Apache-2.0
"""Independent typed/lexical owner and actual-run geometry checks, no product execution."""
from pathlib import Path
import argparse,hashlib,importlib.util,json,os,re,subprocess
SCRIPT=Path(__file__).resolve().parent
REPO=next(p for p in SCRIPT.parents if (p/'assets/fonts/Roboto-Regular.ttf').is_file())
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--input-dir',type=Path,default=SCRIPT)
parser.add_argument('--output-dir',type=Path,default=REPO/'target/effective-labels/reproduced-checks')
parser.add_argument('--python',default=os.environ.get('BONDLINE_PYTHON',str(REPO/'.cache/oracles/format-python/bin/python')))
args=parser.parse_args();ROOT=args.input_dir;OUT=args.output_dir;OUT.mkdir(parents=True,exist_ok=True);PYTHON=args.python
spec=importlib.util.spec_from_file_location('font_oracle',REPO/'xtask/oracles/clearance_oracle.py');font=importlib.util.module_from_spec(spec);spec.loader.exec_module(font)
def read(raw,valid=True):
    p=subprocess.run([str(PYTHON),'-B',str(SCRIPT/'label_metadata_oracle.py')],input=raw,capture_output=True)
    assert (p.returncode==0 if valid else p.returncode!=0),p.stderr.decode()
    return json.loads(p.stdout) if valid else None
def metadata(raw):
    return [{k:v for k,v in r.items() if k not in ('owner_byte_offset','value_byte_span')} for r in raw['metadata_records']]
def actual(raw):
    return {k:raw[k] for k in ['nodes','bonds','captions','style','source_to_model_ids','literal_formula','rdkit_literal_smiles']}
data=json.loads((ROOT/'metadata-vectors.json').read_text());facts={};geometry=[]
for case in data['cases']:
    raw=(ROOT/(case['name']+'.cdxml')).read_bytes();assert len(raw)==case['bytes'] and hashlib.sha256(raw).hexdigest()==case['sha256']
    values=read(raw);facts[case['name']]=values;assert values['metadata_present'] and values['all_actual_styles_explicit'] and values['current_hidden_valid']
    for n in values['nodes']:
        if n['label'] is None:continue
        label=n['label'];text,gs=font.authored({6:'C',7:'N',8:'O'}[n['element']],n['hydrogens'],label['size'],n['point']);bounds=font.placed_box(gs)
        assert text==label['text'] and label['font']=='Roboto' and label['size']==12 and label['face']==0
        error=max(abs(a-b) for a,b in zip(bounds,label['bounding_box']));assert error<=1e-12
        geometry.append({'case':case['name'],'source_node':n['id'],'model_atom':n['model_id'],'actual_size':12,'expected_font_box':bounds,'max_point_error':error})
assert actual(facts['mixed_a'])==actual(facts['mixed_b']) and metadata(facts['mixed_a'])!=metadata(facts['mixed_b'])
assert facts['mixed_a']['resources']['fonts']['7']=='Unused serif'
lookup=next(x for x in facts['mixed_a']['default_lookup'] if x['target_name']=='t' and x['target_id']=='21')
assert {k:v['value'] for k,v in lookup['selected'].items()}=={'font':7,'size_pt':16,'face':0}
assert lookup['selected']['font']['owner_name']=='CDXML' and lookup['selected']['size_pt']['owner_name']=='t'
root_font=next(x for x in facts['mixed_a']['metadata_records'] if x['owner_name']=='CDXML' and x['property']=='LabelFont')
assert root_font['raw_value']==' +&#55;.000 ' and root_font['decoded_value']==' +7.000 ' and root_font['value']==7
assert any(x['owner_name']=='b' and x['property']=='LabelSize' and x['value']==1e308 for x in facts['mixed_a']['metadata_records'])
assert any(x['owner_name']=='b' and x['property']=='LabelFace' and x['value']==32767 for x in facts['mixed_a']['metadata_records'])
assert any(x['owner_name']=='t' and x['owner_id']=='1' and x['property']=='LabelFace' and x['value']==96 for x in facts['caption_owners']['metadata_records'])
assert all(n['label'] is None or n['label']['font_id']==65535 for n in facts['unsigned_actual_font']['nodes'])
for name in ['edited_endpoint','created_3','all_printed','mixed_moved','compound','all_moved']:
    assert metadata(facts[name])==metadata(facts['mixed_a']),name
assert facts['compound']['literal_formula']=='C3H10N2'
assert facts['all_printed']['hidden_model_atoms']==facts['all_moved']['hidden_model_atoms']==[]
a=(ROOT/'mixed_a.cdxml').read_text();unsigned=(ROOT/'unsigned_actual_font.cdxml').read_text();caption=(ROOT/'caption_owners.cdxml').read_text()
controls={
 **{f'missing_actual_{k}':a.replace(f" {k}='{v}'",'',1) for k,v in [('font','42'),('size','12'),('face','0'),('color','4')]},
 'scalar_minus_one_not_font65535':unsigned.replace("LabelFont=' +&#55;.000 '","LabelFont='-1'",1),
 'scalar_font65535_not_unsigned':unsigned.replace("LabelFont=' +&#55;.000 '","LabelFont='65535'",1),
 'missing_scalar_font_resource':a.replace("LabelFont=' +&#55;.000 '","LabelFont='8'",1),
 'shadowed_bad_size':a.replace("LabelSize='10.00'","LabelSize='0'",1),
 'negative_face':a.replace("LabelFace='96'","LabelFace='-1'",1),
 'unsupported_page_owner':a.replace("<page id='90'","<page LabelSize='12' id='90'",1),
 'unsupported_s_owner':a.replace("<s font='42'","<s LabelSize='14' font='42'",1),
 'unsupported_caption_defaults':a.replace('<CDXML ',"<CDXML CaptionFont='42' ",1),
 'unused_namespace':a.replace('<CDXML ',"<CDXML xmlns:x='urn:unused' ",1),
}
at=caption.index("<t id='1'");controls['missing_caption_actual_size']=caption[:at]+caption[at:].replace(" size='12'",'',1)
for name,raw in controls.items():assert raw!=a or name.startswith('scalar') or name.startswith('missing_caption'),name;read(raw.encode(),False)
wrong=a
_,gs=font.authored('O',1,16,[96,160]);box=font.placed_box(gs)
wrong=re.sub(r"BoundingBox='[^']*'",lambda _:"BoundingBox='"+' '.join(str(v) for v in box)+"'",wrong,count=1)
wrong_facts=read(wrong.encode());assert wrong_facts['nodes'][0]['label']['size']==12 and wrong_facts['nodes'][0]['label']['bounding_box']!=facts['mixed_a']['nodes'][0]['label']['bounding_box']
mandatory=a.replace("p='240 160'","p='240 208'",1);invalid=read(mandatory.encode());assert not invalid['current_hidden_valid'] and invalid['mandatory_without_text']==[3]
(OUT/'metadata-facts.json').write_text(json.dumps(facts,indent=2)+'\n')
out={'status':'passed','cases':len(facts),'font_boxes':len(geometry),'fonttools':font.fontTools.__version__,'font_sha256':font.FONT_SHA,'rdkit':'2026.09.2',
 'geometry':geometry,'reader_refusals':list(controls),'wrong16pt_box_for12pt_detected':True,'current_mandatory_hidden_detected':True,'equal_model_different_metadata':True,
 'omitted_run_inheritance':False,'product_execution':False}
(OUT/'independent-checks.json').write_text(json.dumps(out,indent=2)+'\n')
print(json.dumps({'status':'passed','sources':len(facts),'font_boxes':len(geometry),'reader_refusals':len(controls),'facts_sha256':hashlib.sha256((OUT/'metadata-facts.json').read_bytes()).hexdigest()}))
