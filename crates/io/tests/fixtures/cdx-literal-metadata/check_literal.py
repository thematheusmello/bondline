# SPDX-License-Identifier: MIT OR Apache-2.0
"""Verify original CDX bytes with independent struct and pinned external RDKit."""
from pathlib import Path
import argparse,hashlib,importlib.util,json,struct
from rdkit import Chem,rdBase
from rdkit.Chem import rdMolDescriptors
SCRIPT=Path(__file__).resolve().parent
REPO=next(p for p in SCRIPT.parents if (p/'Cargo.toml').is_file() and (p/'crates/io').is_dir())
p=argparse.ArgumentParser(description=__doc__);p.add_argument('--input-dir',type=Path,default=SCRIPT);p.add_argument('--output-dir',type=Path,default=REPO/'target/cdx-literal/reproduced-checks');args=p.parse_args();ROOT=args.input_dir;OUT=args.output_dir;OUT.mkdir(parents=True,exist_ok=True)
s=importlib.util.spec_from_file_location('literal_oracle',SCRIPT/'literal_oracle.py');oracle=importlib.util.module_from_spec(s);s.loader.exec_module(oracle)
assert rdBase.rdkitVersion=='2026.09.2' and Chem.HasChemDrawCDXSupport()
manifest=json.loads((ROOT/'literal-vectors.json').read_text());facts={};sources={}
for case in manifest['cases']:
 raw=(ROOT/(case['name']+'.cdx')).read_bytes();assert len(raw)==case['bytes'] and hashlib.sha256(raw).hexdigest()==case['sha256'];sources[case['name']]=raw;facts[case['name']]=oracle.read(raw)
 assert all(r['payload_hex']=='0100' for r in facts[case['name']]['records'] if r.get('tag')==0x0400)
def model_atoms(name):return [{k:a[k] for k in ['source_id','model_id','source_fragment_id','element','position','isotope','charge','radical','explicit_hydrogens']} for a in facts[name]['atoms']]
assert model_atoms('absent_implied')==model_atoms('scalar_zero_implied') and sources['absent_implied']!=sources['scalar_zero_implied']
assert facts['absent_implied']['atoms'][0]['explicit_hydrogens'] is None and facts['zero_present_explicit']['atoms'][0]['explicit_hydrogens']==0
assert model_atoms('methylammonium_explicit')==model_atoms('methylammonium_extended')==model_atoms('methylammonium_embedded')==model_atoms('reordered')
assert facts['methylammonium_extended']['envelope']=='implied' and all(a['properties'][k]['encoding']=='extended' for a in facts['methylammonium_extended']['atoms'] for k in oracle.NODE_FIELDS.values())
assert facts['extrema']['atoms'][0]['isotope']==1 and facts['extrema']['atoms'][1]['isotope']==32767
assert [a['charge'] for a in facts['extrema']['atoms']]==[-128,127] and [a['explicit_hydrogens'] for a in facts['extrema']['atoms']]==[0,65535]
assert facts['singlet']['atoms'][0]['radical']=='singlet' and facts['triplet']['atoms'][0]['radical']=='triplet'
collisions={'charge4_low_one':0,'charge4_high_one':1,'charge4_high_minus_one':-1,'charge4_low_noise_max':127,'charge4_min':-128}
for name,charge in collisions.items():assert facts[name]['atoms'][0]['charge']==charge and facts[name]['atoms'][0]['properties']['charge']['width']==4
# Literal field mutations are declared independently of product error handling.
base=sources['methylammonium_explicit'];records=facts['methylammonium_explicit']['records'];controls={}
def replacement(record,payload):
 return base[:record['start']]+struct.pack('<HH',record['tag'],len(payload))+payload+base[record['end']:]
for tag,widths in [(0x0420,[0,1,3,4]),(0x0421,[0,2,3,5]),(0x0422,[0,2,3]),(0x042b,[0,1,3,4])]:
 record=next(r for r in records if r.get('tag')==tag)
 for width in widths:controls[f'width_{tag:04x}_{width}']=replacement(record,bytes(width))
 controls[f'duplicate_{tag:04x}']=base[:record['end']]+base[record['start']:record['end']]+base[record['end']:]
for value in [-32768,-1]:controls[f'isotope_domain_{value}']=replacement(next(r for r in records if r.get('tag')==0x0420),struct.pack('<h',value))
for value in [4,255]:controls[f'radical_domain_{value}']=replacement(next(r for r in records if r.get('tag')==0x0422),bytes([value]))
controls['node_unspecified_not_element']=replacement(next(r for r in records if r.get('tag')==0x0400),bytes(2))
for owner in [0x8000,0x8001,0x8003,0x8005,0xc123]:
 position=next(r['end'] for r in records if r.get('tag')==owner)
 controls[f'wrong_owner_{owner:04x}']=base[:position]+struct.pack('<HHh',0x0420,2,13)+base[position:]
for count in [1,2,3]:controls[f'truncated_eof_{count}']=base[:-count]
extended=sources['methylammonium_extended'];er=next(r for r in facts['methylammonium_extended']['records'] if r.get('tag')==0x0420)
for count in [1,3,7,9]:controls[f'truncated_extended_{count}']=extended[:er['start']+count]
for name,raw in controls.items():
 assert raw!=base,name
 try:oracle.read(raw)
 except (ValueError,struct.error,KeyError):pass
 else:raise AssertionError('independent reader accepted '+name)
observed={}
for name in ['methylammonium_explicit','methylammonium_extended','methylammonium_embedded','methyl_doublet','singlet','triplet','default_carbon_fields','deuteron',*collisions]:
 params=Chem.CDXMLParserParams();params.format=Chem.CDXMLFormat.CDX;params.removeHs=False;params.sanitize=name not in collisions
 molecules=Chem.MolsFromCDXML(sources[name],params);assert len(molecules)==1;mol=molecules[0];mol.UpdatePropertyCache(strict=False)
 atoms=[{'element':a.GetAtomicNum(),'isotope':a.GetIsotope(),'charge':a.GetFormalCharge(),'explicit_hydrogens':a.GetNumExplicitHs(),'total_hydrogens':a.GetTotalNumHs(),'radical_electrons':a.GetNumRadicalElectrons()} for a in mol.GetAtoms()]
 observed[name]={'sanitize':params.sanitize,'atoms':atoms,'bonds':[b.GetBondTypeAsDouble() for b in mol.GetBonds()],'smiles':Chem.MolToSmiles(mol),'formula':rdMolDescriptors.CalcMolFormula(mol)}
 if name not in collisions:
  for a,b in zip(facts[name]['atoms'],atoms):
   assert (a['element'],a['isotope'] or 0,a['charge'],a['explicit_hydrogens'])==(b['element'],b['isotope'],b['charge'],b['explicit_hydrogens'])
   if a['radical'] not in ('singlet','triplet'):assert b['radical_electrons']==(1 if a['radical']=='doublet' else 0)
assert observed['methylammonium_explicit']['smiles']=='[13CH3][NH3+]' and observed['methyl_doublet']['smiles']=='[13CH3]' and observed['deuteron']['smiles']=='[2H+]'
assert observed['singlet']['atoms'][0]['radical_electrons']==observed['triplet']['atoms'][0]['radical_electrons']==2
assert {name:observed[name]['atoms'][0]['charge'] for name in collisions}=={'charge4_low_one':1,'charge4_high_one':1,'charge4_high_minus_one':-1,'charge4_low_noise_max':-86,'charge4_min':-128}
(OUT/'literal-facts.json').write_text(json.dumps(facts,indent=2)+'\n')
(OUT/'rdkit-observed.json').write_text(json.dumps({'version':rdBase.rdkitVersion,'binary_cdx_support':True,'cases':observed},indent=2)+'\n')
checks={'status':'passed','sources':len(facts),'reader_refusals':list(controls),'rdkit':'2026.09.2','binary_cdx_support':True,'valid_literal_agreement_cases':8,'charge4_normative':collisions,'charge4_rdkit_differences':{'charge4_low_one':{'normative':0,'rdkit':1},'charge4_low_noise_max':{'normative':127,'rdkit':-86}},'singlet_triplet_distinct_struct_only':True,'absent_h_unresolved':True,'equal_model_distinct_scalar_presence':True,'prefreeze_qualification':'Draft explicit NodeType0 was corrected to public Element1; no product guard changed','product_execution':False}
(OUT/'independent-checks.json').write_text(json.dumps(checks,indent=2)+'\n')
print(json.dumps({'status':'passed','sources':len(facts),'reader_refusals':len(controls),'facts_sha256':hashlib.sha256((OUT/'literal-facts.json').read_bytes()).hexdigest(),'rdkit_observation_sha256':hashlib.sha256((OUT/'rdkit-observed.json').read_bytes()).hexdigest()}))
