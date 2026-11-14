"""Original MIT OR Apache-2.0 raw XML/chemical oracle for conversion plus extension."""
from decimal import Decimal as D
import sys
from xml.parsers import expat

from rdkit import Chem, rdBase
from rdkit.Chem import rdMolDescriptors

assert rdBase.rdkitVersion == "2026.09.2"
assert len(sys.argv) == 2
count = int(sys.argv[1])
assert count in (2, 3)
source = sys.stdin.buffer.read(65_537)
assert len(source) <= 65_536
atoms, bonds = [], []
parser = expat.ParserCreate()
parser.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)


def start(name, attributes):
    if name == "n":
        atoms.append(attributes)
    elif name == "b":
        bonds.append(attributes)


parser.StartElementHandler = start
parser.Parse(source, True)
assert len(atoms) == count and len(bonds) == count - 1
points = [[D(x), D(y)] for x, y in ((72, 72), (120, 72), (160, 112))[:count]]
assert [[D(v) for v in a["p"].split()] for a in atoms] == points
assert [int(a["NumHydrogens"]) for a in atoms] == ([3, 3] if count == 2 else [3, 2, 3])
assert all(int(a["Element"]) == 6 and int(a["Charge"]) == 0
           and a["Radical"] == "None" and "Isotope" not in a for a in atoms)
ids = [int(a["id"]) for a in atoms]
assert len(set(ids)) == count and all(identity > 0 for identity in ids)
assert [(int(b["B"]), int(b["E"])) for b in bonds] == list(zip(ids, ids[1:]))
assert all(D(b["Order"]) == 1 for b in bonds)
molecules = Chem.MolsFromCDXML(source.decode("utf-8"))
assert len(molecules) == 1
molecule = molecules[0]
assert molecule.GetNumAtoms() == count and molecule.GetNumBonds() == count - 1
assert Chem.MolToSmiles(molecule) == "C" * count
assert rdMolDescriptors.CalcMolFormula(molecule) == f"C{count}H{2 * count + 2}"
assert [atom.GetTotalNumHs() for atom in molecule.GetAtoms()] == ([3, 3] if count == 2 else [3, 2, 3])
