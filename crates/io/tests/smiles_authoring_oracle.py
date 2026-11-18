"""Original external chemistry consumer of ACTUAL serialized authoring Fragment data.

This validates integer graph/H/isotopes against independent notation interpretation,
not geometry, source validity in every dialect or physical aromaticity. Source lowercase
markers are deliberately distinct from the reported RDKit-perceived aromatic flags.
"""
import json
import math
import sys

from rdkit import Chem, rdBase
from rdkit.Chem import rdMolDescriptors

assert rdBase.rdkitVersion == "2026.09.2", "use the literal pinned external interpreter"
assert len(sys.argv) in (2, 3)
source = sys.argv[1].encode("ascii")
assert 0 < len(source) <= 65_536
notation = source[:-2] if source.endswith(b"\r\n") else source[:-1] if source.endswith(b"\n") else source
assert notation and all(33 <= byte <= 126 for byte in notation), "whole notation, no names or extra records"
parameters = Chem.SmilesParserParams()
parameters.sanitize, parameters.removeHs = True, False
parameters.parseName, parameters.allowCXSMILES = False, False
reference = Chem.MolFromSmiles(notation.decode("ascii"), parameters)
assert reference is not None and 1 <= reference.GetNumAtoms() <= 64
assert all(atom.GetAtomicNum() in (1, 6, 7, 8, 9, 17, 35, 53)
           and atom.GetFormalCharge() == atom.GetNumRadicalElectrons() == 0
           and atom.GetChiralTag() == Chem.ChiralType.CHI_UNSPECIFIED
           for atom in reference.GetAtoms())
assert all(bond.GetStereo() == Chem.BondStereo.STEREONONE for bond in reference.GetBonds()), "source stereo must not be erased"
parameters.sanitize = False
raw_reference = Chem.MolFromSmiles(notation.decode("ascii"), parameters)
assert raw_reference is not None and raw_reference.GetNumAtoms() == reference.GetNumAtoms()
expected = {
    "atoms": [[atom.GetAtomicNum(), atom.GetIsotope() or None, atom.GetTotalNumHs()]
              for atom in reference.GetAtoms()],
    "formula": rdMolDescriptors.CalcMolFormula(reference),
    "canonical": Chem.MolToSmiles(reference, isomericSmiles=True),
    "components": [list(index + 1 for index in group) for group in Chem.GetMolFrags(reference)],
    "perceived_aromatic_atoms": [atom.GetIsAromatic() for atom in reference.GetAtoms()],
}
options = json.loads(sys.argv[2]) if len(sys.argv) == 3 else {}
assert isinstance(options, dict)
assert set(options) <= {"atoms", "bonds", "formula", "canonical", "components",
                        "perceived_aromatic_atoms", "perceived_aromatic_bonds"}
for name in expected:
    if name in options:
        assert options[name] == expected[name], "literal expectation must match independent source interpretation"
expected.update(options)
raw = sys.stdin.buffer.read(1_048_577)
assert 0 < len(raw) <= 1_048_576, "bounded test consumer input"
fragment = json.loads(raw)
assert isinstance(fragment, dict) and set(fragment) == {"id", "atoms", "bonds"}


def identity(value):
    assert type(value) is int and 1 <= value <= 0xffffffff
    return value


identity(fragment["id"])
atoms, bonds = fragment["atoms"], fragment["bonds"]
assert isinstance(atoms, list) and isinstance(bonds, list)
assert 1 <= len(atoms) <= 64 and len(bonds) <= 128
assert len(atoms) == reference.GetNumAtoms() and len(bonds) == reference.GetNumBonds()
atom_ids, bond_ids = {}, set()
molecule = Chem.RWMol()
actual_atoms = []
for index, atom in enumerate(atoms):
    assert isinstance(atom, dict)
    assert set(atom) == {"id", "element", "position", "isotope", "charge", "radical", "explicit_hydrogens"}
    key = identity(atom["id"])
    assert key not in atom_ids
    atom_ids[key] = index
    z, isotope, h = atom["element"], atom["isotope"], atom["explicit_hydrogens"]
    assert type(z) is int and z in (1, 6, 7, 8, 9, 17, 35, 53)
    assert isotope is None or type(isotope) is int and 1 <= isotope <= 65535
    assert type(h) is int and 0 <= h <= 65535, "fixed literal H is required"
    assert type(atom["charge"]) is int and atom["charge"] == 0 and atom["radical"] == "none"
    position = atom["position"]
    if position is not None:
        assert isinstance(position, dict) and set(position) == {"x_pt", "y_pt"}
        assert all(type(value) in (int, float) and math.isfinite(value) and abs(value) <= 100_000
                   for value in position.values()), "only a finite model-coordinate check, not layout acceptance"
    actual_atoms.append([z, isotope, h])
    external_atom = Chem.Atom(z)
    if isotope is not None:
        external_atom.SetIsotope(isotope)
    external_atom.SetNoImplicit(True)
    external_atom.SetNumExplicitHs(h)
    molecule.AddAtom(external_atom)
assert actual_atoms == expected["atoms"]
actual_edges, pairs = [], set()
for bond in bonds:
    assert isinstance(bond, dict) and set(bond) == {"id", "begin", "end", "order"}
    key = identity(bond["id"])
    assert key not in bond_ids
    bond_ids.add(key)
    a, b = atom_ids[identity(bond["begin"])], atom_ids[identity(bond["end"])]
    assert a != b and tuple(sorted((a, b))) not in pairs
    pairs.add(tuple(sorted((a, b))))
    assert bond["order"] in ("single", "double", "triple"), "integer authoring graph only"
    order = {"single": 1, "double": 2, "triple": 3}[bond["order"]]
    source_bond = raw_reference.GetBondBetweenAtoms(a, b)
    assert source_bond is not None, "actual connectivity must match source"
    source_order = source_bond.GetBondTypeAsDouble()
    assert source_order == 1.5 or source_order == order, "fixed source Single/Double/Triple must remain exact"
    molecule.AddBond(a, b, {1: Chem.BondType.SINGLE, 2: Chem.BondType.DOUBLE, 3: Chem.BondType.TRIPLE}[order])
    actual_edges.append([a + 1, b + 1, order])
assert pairs == {tuple(sorted((bond.GetBeginAtomIdx(), bond.GetEndAtomIdx())))
                 for bond in raw_reference.GetBonds()}
if "bonds" in expected:
    assert actual_edges == expected["bonds"], "literal directed source-order integer assignment"
molecule = molecule.GetMol()
Chem.SanitizeMol(molecule)
assert [[atom.GetAtomicNum(), atom.GetIsotope() or None, atom.GetTotalNumHs()]
        for atom in molecule.GetAtoms()] == expected["atoms"]
assert all(atom.GetFormalCharge() == atom.GetNumRadicalElectrons() == 0
           and atom.GetChiralTag() == Chem.ChiralType.CHI_UNSPECIFIED
           for atom in molecule.GetAtoms())
assert all(bond.GetStereo() == Chem.BondStereo.STEREONONE for bond in molecule.GetBonds())
perceived_atoms = [atom.GetIsAromatic() for atom in molecule.GetAtoms()]
perceived_bonds = [bond.GetIsAromatic() for bond in molecule.GetBonds()]
assert perceived_atoms == expected["perceived_aromatic_atoms"]
assert perceived_bonds == [reference.GetBondBetweenAtoms(a - 1, b - 1).GetIsAromatic()
                           for a, b, _ in actual_edges]
if "perceived_aromatic_bonds" in expected:
    assert perceived_bonds == expected["perceived_aromatic_bonds"]
components = [list(index + 1 for index in group) for group in Chem.GetMolFrags(molecule)]
assert components == expected["components"]
formula = rdMolDescriptors.CalcMolFormula(molecule)
canonical = Chem.MolToSmiles(molecule, isomericSmiles=True)
assert formula == expected["formula"] and canonical == expected["canonical"]
print(json.dumps({"atoms": len(atoms), "bonds": len(bonds), "components": len(components),
                  "formula": formula, "canonical": canonical,
                  "perceived_aromatic_atoms": perceived_atoms,
                  "perceived_aromatic_bonds": perceived_bonds,
                  "no_assigned_stereo": True, "no_radicals": True,
                  "raw_graph": {"atoms": actual_atoms, "bonds": actual_edges,
                                "components": components}}))
