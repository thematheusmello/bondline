"""External consumer for the native C-C/three-H case; not a Bondline chemistry engine."""
import sys
import rdkit
from rdkit import Chem
from rdkit.Chem import rdMolDescriptors

assert rdkit.__version__ == "2026.09.2", rdkit.__version__
molecules = Chem.MolsFromCDXML(sys.stdin.buffer.read().decode("utf-8"))
assert len(molecules) == 1
mol = molecules[0]
assert mol.GetNumAtoms() == 2 and mol.GetNumBonds() == 1
assert [a.GetAtomicNum() for a in mol.GetAtoms()] == [6, 6]
assert [a.GetFormalCharge() for a in mol.GetAtoms()] == [0, 0]
assert [a.GetTotalNumHs() for a in mol.GetAtoms()] == [3, 3]
assert mol.GetBondWithIdx(0).GetBondTypeAsDouble() == 1.0
assert Chem.MolToSmiles(mol) == "CC"
assert rdMolDescriptors.CalcMolFormula(mol) == "C2H6"
