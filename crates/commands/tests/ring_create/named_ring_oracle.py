"""Original test-only literal CDXML and independently sanitized named-ring checks.

Raw directed integer bonds, H, labels and DoublePosition are authoritative data.
RDKit sanitization is a separate chemical interpretation: aromatic 1.5 after that
operation is neither a raw-order allowance nor a source rewrite.
"""
import json
import sys
import xml.parsers.expat as expat
from decimal import Decimal

from rdkit import Chem, rdBase
from rdkit.Chem import rdMolDescriptors

assert rdBase.rdkitVersion == "2026.09.2", "use the pinned external interpreter"
assert len(sys.argv) == 2
expected = json.loads(sys.argv[1])
assert set(expected) == {"graph", "smiles", "formula", "flags", "double_positions"}
assert len(expected["graph"]) == 1
graph = expected["graph"][0]
atoms, bonds = graph["atoms"], graph["bonds"]
assert 1 <= len(atoms) <= 64 and 1 <= len(bonds) <= 128
assert len(expected["flags"]) == len(atoms)
assert len(expected["double_positions"]) == len(bonds)
raw = sys.stdin.buffer.read(1_048_577)
assert 0 < len(raw) <= 1_048_576
nodes, edges, fragments = [], [], []
stack = []


def start(name, attributes):
    if name == "fragment":
        assert not fragments and stack[-1] == "page"
        fragments.append(dict(attributes))
    elif name == "n":
        assert stack[-1] == "fragment"
        nodes.append(dict(attributes))
    elif name == "b":
        assert stack[-1] == "fragment"
        edges.append(dict(attributes))
    stack.append(name)


def end(name):
    assert stack.pop() == name


parser = expat.ParserCreate()
parser.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
parser.StartElementHandler, parser.EndElementHandler = start, end
parser.Parse(raw, True)
assert len(fragments) == 1 and len(nodes) == len(atoms) and len(edges) == len(bonds)
source_ids = {}
for index, (node, atom, flag) in enumerate(zip(nodes, atoms, expected["flags"])):
    assert node["id"] not in source_ids
    source_ids[node["id"]] = index + 1
    assert atom["id"] == index + 1 and atom["element"] == 6
    assert node.get("NodeType", "Element") == "Element"
    assert Decimal(node.get("Element", "6")) == atom["element"]
    assert Decimal(node.get("Charge", "0")) == atom["charge"] == 0
    assert node.get("Radical", "None").lower() == atom["radical"] == "none"
    assert Decimal(node.get("Isotope", "0")) == 0 and atom["isotope"] is None
    assert Decimal(node["NumHydrogens"]) == atom["explicit_hydrogens"]
    assert flag is None or type(flag) is bool
    assert node.get("ShowNonTerminalCarbonLabels") == ({False: "no", True: "yes"}[flag] if flag is not None else None)
    actual_point = tuple(map(Decimal, node["p"].split()))
    assert len(actual_point) == 2
    for value, axis in zip(actual_point, ("x_pt", "y_pt")):
        assert value.is_finite()
        assert abs(value - Decimal(str(atom["position"][axis]))) <= Decimal("0.000000001")

literal = []
bond_ids = set()
for index, (edge, bond, side) in enumerate(zip(edges, bonds, expected["double_positions"])):
    assert edge["id"] not in bond_ids
    bond_ids.add(edge["id"])
    assert bond["id"] == index + 1
    assert bond["order"] in ("single", "double")
    weight = {"single": 1, "double": 2}[bond["order"]]
    assert Decimal(edge.get("Order", "1")) == weight, "literal integer bond order"
    assert [source_ids[edge["B"]], source_ids[edge["E"]]] == [bond["begin"], bond["end"]], "literal B/E direction"
    assert side in (None, "left", "right", "center")
    assert edge.get("DoublePosition") == (side.capitalize() if side is not None else None)
    assert "Display" not in edge, "no unrequested wedge or other bond display"
    literal.append((min(bond["begin"], bond["end"]), max(bond["begin"], bond["end"]), weight))

# The unsanitized external reader must retain the exact integer assignment. Its
# endpoint convention is not used to relax the independently checked raw B/E data.
integer_molecules = Chem.MolsFromCDXML(raw.decode("utf-8"), sanitize=False, removeHs=False)
assert len(integer_molecules) == 1
integer = integer_molecules[0]
assert integer.GetNumAtoms() == len(atoms) and integer.GetNumBonds() == len(bonds)
integer_edges = []
for bond in integer.GetBonds():
    weight = bond.GetBondTypeAsDouble()
    assert weight in (1.0, 2.0), "unsanitized CDXML must not substitute aromatic bonds"
    a, b = bond.GetBeginAtomIdx() + 1, bond.GetEndAtomIdx() + 1
    integer_edges.append((min(a, b), max(a, b), int(weight)))
assert sorted(integer_edges) == sorted(literal)
integer.UpdatePropertyCache(strict=True)
assert [[a.GetAtomicNum(), a.GetTotalNumHs()] for a in integer.GetAtoms()] == [
    [a["element"], a["explicit_hydrogens"]] for a in atoms]

reference = Chem.MolFromSmiles(expected["smiles"])
assert reference is not None
assert rdMolDescriptors.CalcMolFormula(reference) == expected["formula"], "manual reference formula"
sanitized = Chem.MolsFromCDXML(raw.decode("utf-8"), sanitize=True, removeHs=False)
assert len(sanitized) == 1
molecule = sanitized[0]
assert molecule.GetNumAtoms() == len(atoms) and molecule.GetNumBonds() == len(bonds)
assert [[a.GetAtomicNum(), a.GetTotalNumHs()] for a in molecule.GetAtoms()] == [
    [a["element"], a["explicit_hydrogens"]] for a in atoms]
assert all(a.GetFormalCharge() == a.GetIsotope() == a.GetNumRadicalElectrons() == 0
           and a.GetChiralTag() == Chem.ChiralType.CHI_UNSPECIFIED for a in molecule.GetAtoms())
assert all(b.GetStereo() == Chem.BondStereo.STEREONONE for b in molecule.GetBonds())
assert Chem.MolToSmiles(molecule, isomericSmiles=True) == Chem.MolToSmiles(reference, isomericSmiles=True)
assert rdMolDescriptors.CalcMolFormula(molecule) == expected["formula"]
print(json.dumps({"atoms": len(atoms), "bonds": len(bonds), "formula": expected["formula"],
                  "literal_integer_orders": True, "literal_direction": True,
                  "literal_flags_and_double_positions": True, "no_assigned_stereo": True}))
