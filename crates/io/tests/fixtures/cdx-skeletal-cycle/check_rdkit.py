# SPDX-License-Identifier: MIT OR Apache-2.0
"""Independent consumer for a saved positive ordinary CDX cycle."""
from pathlib import Path
import sys
from rdkit import Chem, rdBase
from rdkit.Chem import rdMolDescriptors

assert rdBase.rdkitVersion == '2026.09.2' and Chem.HasChemDrawCDXSupport()
params = Chem.CDXMLParserParams()
params.format = Chem.CDXMLFormat.CDX
params.sanitize = True
params.removeHs = False
mol, = Chem.MolsFromCDXML(Path(sys.argv[1]).read_bytes(), params)
assert [atom.GetAtomicNum() for atom in mol.GetAtoms()] == [6]*4
assert [atom.GetNumExplicitHs() for atom in mol.GetAtoms()] == [0]*4
assert [atom.GetTotalNumHs() for atom in mol.GetAtoms()] == [2]*4
assert [(bond.GetBeginAtomIdx(), bond.GetEndAtomIdx(), bond.GetBondTypeAsDouble())
        for bond in mol.GetBonds()] == [(0, 1, 1), (1, 2, 1), (2, 3, 1), (3, 0, 1)]
assert Chem.MolToSmiles(mol) == 'C1CCC1'
assert rdMolDescriptors.CalcMolFormula(mol) == 'C4H8'
print('RDKit 2026.09.2: saved cyclobutane, exact topology, omitted/total H and C4H8')
