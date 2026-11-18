# SPDX-License-Identifier: MIT OR Apache-2.0
"""Independent original CDX bond data/negative and external RDKit checks."""
from pathlib import Path
import argparse,hashlib,importlib.util,json,struct
from rdkit import Chem,rdBase
from rdkit.Chem import rdMolDescriptors
SCRIPT=Path(__file__).resolve().parent
REPO=next(p for p in SCRIPT.parents if (p/'Cargo.toml').is_file() and (p/'crates/io').is_dir())
p=argparse.ArgumentParser(description=__doc__);p.add_argument('--input-dir',type=Path,default=SCRIPT);p.add_argument('--output-dir',type=Path,default=REPO/'target/cdx-bonds/reproduced-checks');args=p.parse_args();ROOT=args.input_dir;OUT=args.output_dir;OUT.mkdir(parents=True,exist_ok=True)
s=importlib.util.spec_from_file_location('original_bond_oracle',SCRIPT/'bond_oracle.py');oracle=importlib.util.module_from_spec(s);s.loader.exec_module(oracle)
assert rdBase.rdkitVersion=='2026.09.2' and Chem.HasChemDrawCDXSupport()
manifest=json.loads((ROOT/'bond-vectors.json').read_text());facts={};sources={}
for case in manifest['cases']:
 raw=(ROOT/(case['name']+'.cdx')).read_bytes();assert len(raw)==case['bytes'] and hashlib.sha256(raw).hexdigest()==case['sha256'];sources[case['name']]=raw;facts[case['name']]=oracle.read(raw)
 assert all(r['payload_hex']=='0100' for r in facts[case['name']]['records'] if r.get('tag')==0x0400)
assert facts['single_absent']['model']==facts['single_solid']['model'] and sources['single_absent']!=sources['single_solid']
for side in ['center','right','left']:
 auto,manual=facts['auto_'+side],facts['manual_'+side]
 assert auto['model']==manual['model'] and auto['bonds'][0]['position_manual'] is False and manual['bonds'][0]['position_manual'] is True
 assert auto['bonds'][0]['double_position']==manual['bonds'][0]['double_position']==side
assert facts['ethene']['bonds'][0]['double_position']=='unspecified' and facts['auto_center']['bonds'][0]['double_position']=='center'
assert 'double_position' not in facts['ethene']['model']['pages'][0]['fragments'][0]['bonds'][0]
assert facts['reversed_right']['bonds'][0]['begin_source']==149 and facts['reversed_right']['bonds'][0]['end_source']==73
assert facts['reversed_right']['bonds'][0]['begin']==2 and facts['reversed_right']['bonds'][0]['end']==1 and facts['reversed_right']['bonds'][0]['double_position']=='right'
for name in ['reordered_right','extended_right','embedded_right','implied_right']:assert facts[name]['model']==facts['manual_right']['model']
assert all(p['encoding']=='extended' and p['width']==2 for p in facts['extended_right']['bonds'][0]['properties'].values())
assert [a['explicit_hydrogens'] for a in facts['ethene']['atoms']]==[2,2] and [a['explicit_hydrogens'] for a in facts['ethyne']['atoms']]==[1,1]
assert all(a['explicit_hydrogens'] is None for a in facts['ethene_unknown_h']['atoms'])
n=importlib.util.spec_from_file_location('original_negative_bonds',SCRIPT/'negative_bonds.py');negative=importlib.util.module_from_spec(n);n.loader.exec_module(negative)
negative_cases=negative.negative_sources(sources,facts)
controls={name:raw for name,(raw,_) in negative_cases.items()};expected={name:error for name,(_,error) in negative_cases.items()}
for name,raw in controls.items():
 try:oracle.read(raw)
 except (ValueError,struct.error,KeyError):pass
 else:raise AssertionError('independent reader accepted '+name)
observed={}
for name,source in sources.items():
 params=Chem.CDXMLParserParams();params.format=Chem.CDXMLFormat.CDX;params.removeHs=False
 molecules=Chem.MolsFromCDXML(source,params);assert len(molecules)==1;mol=molecules[0];assert mol.GetNumAtoms()==2 and mol.GetNumBonds()==1;bond=mol.GetBondWithIdx(0)
 order={'single':1,'double':2,'triple':3}[facts[name]['bonds'][0]['order']]
 assert bond.GetBondTypeAsDouble()==order
 assert (bond.GetBeginAtomIdx(),bond.GetEndAtomIdx())==(facts[name]['bonds'][0]['begin']-1,facts[name]['bonds'][0]['end']-1)
 assert Chem.MolToSmiles(mol)=={1:'CC',2:'C=C',3:'C#C'}[order] and rdMolDescriptors.CalcMolFormula(mol)=={1:'C2H6',2:'C2H4',3:'C2H2'}[order]
 explicit=[a.GetNumExplicitHs() for a in mol.GetAtoms()]
 for a,h in zip(facts[name]['atoms'],explicit):
  assert a['element']==6
  if a['explicit_hydrogens'] is not None:assert h==a['explicit_hydrogens']
 positions=[list(mol.GetConformer().GetAtomPosition(i)) for i in range(2)]
 assert positions==[[6307840.0,-10518528.0,0.0],[9453568.0,-10518528.0,0.0]]
 observed[name]={'elements':[a.GetAtomicNum() for a in mol.GetAtoms()],'explicit_hydrogens':explicit,'begin_index':bond.GetBeginAtomIdx(),'end_index':bond.GetEndAtomIdx(),'order':bond.GetBondTypeAsDouble(),'formula':rdMolDescriptors.CalcMolFormula(mol),'smiles':Chem.MolToSmiles(mol),'raw_coordinate_convention':positions}
(OUT/'bond-facts.json').write_text(json.dumps(facts,indent=2)+'\n')
(OUT/'rdkit-observed.json').write_text(json.dumps({'version':rdBase.rdkitVersion,'binary_cdx_support':True,'cases':observed,'scope':'graph/order/direction only; no side, manual-state, rendering or spin inference'},indent=2)+'\n')
checks={'status':'passed','sources':len(facts),'reader_refusals':[{'name':name,'product_error_contract':expected[name]} for name in controls],'rdkit':'2026.09.2','binary_cdx_support':True,'valid_graph_cases':len(observed),'side_presence_struct_only':True,'non_double_fields_refused_including_zero':True,'absent_h_unresolved':True,'same_model_different_source_pairs':['single_absent/single_solid','auto_center/manual_center','auto_right/manual_right','auto_left/manual_left'],'product_execution':False}
(OUT/'independent-checks.json').write_text(json.dumps(checks,indent=2)+'\n')
print(json.dumps({'status':'passed','sources':len(facts),'reader_refusals':len(controls),'facts_sha256':hashlib.sha256((OUT/'bond-facts.json').read_bytes()).hexdigest(),'observation_sha256':hashlib.sha256((OUT/'rdkit-observed.json').read_bytes()).hexdigest()}))
