"""Independent raw-field and external consumer checks for original authored C chains."""
import sys
from decimal import Decimal
from xml.parsers import expat
from rdkit import Chem, rdBase
from rdkit.Chem import rdMolDescriptors

count = int(sys.argv[1])
assert count in (2, 3, 4)
paper = sys.argv[2]
assert paper in ("a4", "letter")
raw = sys.stdin.buffer.read(64 * 1024 * 1024 + 1)
assert len(raw) <= 64 * 1024 * 1024
stack, pages, fragments, atoms, bonds = [], [], [], [], []


def start(name, attributes):
    if name == "page":
        assert stack == ["CDXML"]
        pages.append(attributes)
    elif name == "fragment":
        assert stack == ["CDXML", "page"]
        fragments.append(attributes)
    elif name == "n":
        assert stack == ["CDXML", "page", "fragment"]
        atoms.append(attributes)
    elif name == "b":
        assert stack == ["CDXML", "page", "fragment"]
        bonds.append(attributes)
    else:
        assert name == "CDXML" and not stack
    stack.append(name)


parser = expat.ParserCreate()
parser.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
parser.StartElementHandler = start
parser.EndElementHandler = lambda name: stack.pop()
parser.Parse(raw, True)
assert not stack and len(pages) == len(fragments) == 1
assert pages[0]["DrawingSpace"] == "Poster"
if paper == "letter":
    assert (Decimal(pages[0]["Width"]), Decimal(pages[0]["Height"])) == (612, 792)
else:
    assert abs(Decimal(pages[0]["Width"]) - Decimal(210) * 72 / Decimal("25.4")) < Decimal("1e-10")
    assert abs(Decimal(pages[0]["Height"]) - Decimal(297) * 72 / Decimal("25.4")) < Decimal("1e-10")
assert len(atoms) == count and len(bonds) == count - 1
ids = [int(atom["id"]) for atom in atoms]
assert len(set(ids)) == count and min(ids) > 0
positions = [(100, 200), (140, 200), (180, 240), (220, 280)][:count]
hydrogens = [3] + [2] * (count - 2) + [3]
for atom, position, hydrogens_at_atom in zip(atoms, positions, hydrogens):
    assert atom["NodeType"] == "Element" and atom["Element"] == "6"
    assert atom["Charge"] == "0" and atom["Radical"] == "None"
    assert "Isotope" not in atom
    assert int(atom["NumHydrogens"]) == hydrogens_at_atom
    assert tuple(Decimal(value) for value in atom["p"].split()) == position
for bond, begin, end in zip(bonds, ids, ids[1:]):
    assert (int(bond["B"]), int(bond["E"]), bond["Order"]) == (begin, end, "1")
all_ids = [int(item["id"]) for item in pages + fragments + atoms + bonds]
assert len(set(all_ids)) == len(all_ids)
assert rdBase.rdkitVersion == "2026.09.2", "use pinned BONDLINE_PYTHON"
molecules = Chem.MolsFromCDXML(raw.decode("utf-8"))
assert len(molecules) == 1
molecule = molecules[0]
assert [atom.GetAtomicNum() for atom in molecule.GetAtoms()] == [6] * count
assert [atom.GetFormalCharge() for atom in molecule.GetAtoms()] == [0] * count
assert [atom.GetTotalNumHs() for atom in molecule.GetAtoms()] == hydrogens
assert [(bond.GetBeginAtomIdx(), bond.GetEndAtomIdx(), bond.GetBondTypeAsDouble()) for bond in molecule.GetBonds()] == [(index, index + 1, 1.0) for index in range(count - 1)]
assert Chem.MolToSmiles(molecule) == "C" * count
assert rdMolDescriptors.CalcMolFormula(molecule) == f"C{count}H{2 * count + 2}"
