# SPDX-License-Identifier: MIT OR Apache-2.0
from pathlib import Path
from rdkit import Chem,rdBase
from rdkit.Chem import rdMolDescriptors

assert rdBase.rdkitVersion=="2026.09.2" and Chem.HasChemDrawCDXSupport()
out=Path(__file__).resolve().parent
params=Chem.CDXMLParserParams()
params.format=Chem.CDXMLFormat.CDX
params.sanitize=False
params.removeHs=False
results={}
for name in ("propane","dormant_true","dormant_nonterminal","terminal_visible","interior_visible","missing_h","all_h_missing","ethane"):
    mol,=Chem.MolsFromCDXML((out/(name+".cdx")).read_bytes(),params)
    mol.UpdatePropertyCache(strict=False)
    results[name]=dict(elements=[a.GetAtomicNum() for a in mol.GetAtoms()],
        attached_h=[a.GetNumExplicitHs() for a in mol.GetAtoms()],
        sum_h=[a.GetTotalNumHs() for a in mol.GetAtoms()],
        edges=[[b.GetBeginAtomIdx(),b.GetEndAtomIdx(),b.GetBondTypeAsDouble()] for b in mol.GetBonds()],
        formula=rdMolDescriptors.CalcMolFormula(mol),smiles=Chem.MolToSmiles(mol))
for name in ("propane","dormant_true","dormant_nonterminal","terminal_visible","interior_visible"):
    assert results[name]["elements"]==[6,6,6] and results[name]["attached_h"]==[3,2,3]
    assert results[name]["edges"]==[[0,1,1],[1,2,1]]
    assert results[name]["formula"]=="C3H8" and results[name]["smiles"]=="CCC"
assert results["all_h_missing"]["sum_h"]==[3,2,3]
assert results["missing_h"]["attached_h"] == [3, 0, 3]
assert results["all_h_missing"]["attached_h"] == [0, 0, 0]
assert results["missing_h"]["sum_h"] == [3, 2, 3]
assert results["ethane"]["smiles"] == "CC"
print(f"RDKit {rdBase.rdkitVersion}: {len(results)} binary graphs, attached and resolved H, formula and SMILES")
