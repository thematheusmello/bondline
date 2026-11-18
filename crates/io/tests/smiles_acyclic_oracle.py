"""Original bounded actual-CDXML graph/Decimal/RDKit consumer, external tests only.

Expected atoms, directed edges, H counts and formula are literal test vectors, not
output from Bondline's normalizer. Geometry is checked independently of its layout.
"""

from decimal import Decimal as D, localcontext
import json
import sys
from xml.parsers import expat

from rdkit import Chem, rdBase
from rdkit.Chem import rdMolDescriptors

assert rdBase.rdkitVersion == "2026.09.2", "use the literal pinned BONDLINE_PYTHON"
assert len(sys.argv) in (2, 3)
original = sys.argv[1]
assert len(original.encode("ascii")) <= 65_536
parameters = Chem.SmilesParserParams()
parameters.sanitize, parameters.removeHs = True, False
parameters.parseName, parameters.allowCXSMILES = False, False
reference = Chem.MolFromSmiles(original, parameters)
assert reference is not None
assert len(Chem.GetMolFrags(reference)) == 1, "one connected reference graph"
assert all(not atom.GetIsAromatic() and atom.GetAtomicNum() in (1, 6, 7, 8, 9, 17, 35, 53)
           and atom.GetFormalCharge() == atom.GetIsotope() == atom.GetNumRadicalElectrons() == 0
           for atom in reference.GetAtoms())
expected = {
    "atoms": [[atom.GetAtomicNum(), atom.GetTotalNumHs()] for atom in reference.GetAtoms()],
    "formula": rdMolDescriptors.CalcMolFormula(reference),
    "length": "48", "origin": ["72", "72"],
    "page": [str(D(210) * 72 / D("25.4")), str(D(297) * 72 / D("25.4"))],
}
options = json.loads(sys.argv[2]) if len(sys.argv) == 3 else {}
assert set(options) <= {"atoms", "bonds", "formula", "length", "origin", "page"}
for name in ("atoms", "formula"):
    if name in options:
        assert options[name] == expected[name], "independent literal expectation must match the original graph"
expected.update(options)
assert 1 <= len(expected["atoms"]) <= 64
assert reference.GetNumBonds() + 1 == reference.GetNumAtoms()
raw = sys.stdin.buffer.read(1_048_577)
assert len(raw) <= 1_048_576, "bounded test consumer input"
nodes, edges, ids, pages, fragments = [], [], set(), [], []
stack = []
references = {}


def start(name, attributes):
    if not stack:
        assert name == "CDXML"
    if "id" in attributes:
        identity = int(attributes["id"])
        assert 1 <= identity <= 0xffffffff and identity not in ids
        ids.add(identity)
    if name == "page":
        assert stack == ["CDXML"]
        pages.append(attributes)
    elif name == "fragment":
        assert stack == ["CDXML", "page"]
        fragments.append(attributes)
    elif name == "n":
        assert stack == ["CDXML", "page", "fragment"]
        assert attributes["NodeType"] == "Element"
        assert attributes["Charge"] == "0" and attributes["Radical"] == "None"
        assert "Isotope" not in attributes
        position = tuple(D(part) for part in attributes["p"].split())
        assert len(position) == 2 and all(value.is_finite() for value in position)
        references[attributes["id"]] = len(nodes) + 1
        nodes.append((int(attributes["Element"]),
                      int(attributes["NumHydrogens"]), position))
    elif name == "b":
        assert stack == ["CDXML", "page", "fragment"]
        assert set(attributes) == {"id", "B", "E", "Order"}
        edges.append((attributes["B"], attributes["E"], int(attributes["Order"])))
    stack.append(name)


def end(name):
    assert stack.pop() == name


parser = expat.ParserCreate()
parser.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
parser.StartElementHandler, parser.EndElementHandler = start, end
parser.Parse(raw, True)
assert not stack and len(pages) == len(fragments) == 1
assert pages[0]["DrawingSpace"] == "Poster"
assert all(abs(D(pages[0][name]) - D(value)) < D("1e-9")
           for name, value in zip(("Width", "Height"), expected["page"]))
assert [[element, h] for element, h, _ in nodes] == expected["atoms"]
actual_edges = [(references[a], references[b], order) for a, b, order in edges]
reference_edges = sorted((min(bond.GetBeginAtomIdx(), bond.GetEndAtomIdx()) + 1,
                          max(bond.GetBeginAtomIdx(), bond.GetEndAtomIdx()) + 1,
                          int(bond.GetBondTypeAsDouble())) for bond in reference.GetBonds())
assert all(bond.GetBondTypeAsDouble() in (1, 2, 3) for bond in reference.GetBonds())
assert sorted((min(a, b), max(a, b), order) for a, b, order in actual_edges) == reference_edges
if "bonds" in expected:
    assert [list(edge) for edge in actual_edges] == expected["bonds"], "directed source-order graph"
assert len({tuple(sorted((a, b))) for a, b, _ in actual_edges}) == len(actual_edges)
points = [position for _, _, position in nodes]
length = D(expected["length"])
origin = [D(value) for value in expected["origin"]]
assert length.is_finite() and length > 0
epsilon = max(length, D(1)) * D("1e-9")
assert abs(min(point[0] for point in points) - origin[0]) <= epsilon
assert abs(min(point[1] for point in points) - origin[1]) <= epsilon
width, height = [D(value) for value in expected["page"]]
assert all(0 <= x <= width and 0 <= y <= height for x, y in points)


def cross(a, b, p):
    return (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0])


def on_segment(a, b, p):
    return abs(cross(a, b, p)) <= epsilon * max(length, D(1)) and all(
        min(a[axis], b[axis]) - epsilon <= p[axis] <= max(a[axis], b[axis]) + epsilon
        for axis in (0, 1))


def verify_unassigned_bonds(reference, molecule, points, length, epsilon):
    """ANY is unknown, allowed only for the source's collinear two-arm Double."""
    for original_bond in reference.GetBonds():
        assert original_bond.GetStereo() == Chem.BondStereo.STEREONONE
        a, b = original_bond.GetBeginAtomIdx(), original_bond.GetEndAtomIdx()
        actual = molecule.GetBondBetweenAtoms(a, b)
        assert actual is not None
        other_ends = []
        if original_bond.GetBondType() == Chem.BondType.DOUBLE:
            for center, partner in ((a, b), (b, a)):
                atom = reference.GetAtomWithIdx(center)
                others = [edge for edge in atom.GetBonds()
                          if edge.GetOtherAtomIdx(center) != partner]
                if (atom.GetDegree() != 2 or len(others) != 1
                        or others[0].GetBondType() != Chem.BondType.SINGLE):
                    break
                other_ends.append(others[0].GetOtherAtomIdx(center))
        eligible_unknown = len(other_ends) == 2
        if eligible_unknown:
            # Check both ends even if this reader happens to return NONE instead
            # of ANY. A 2-degree reader tolerance is not our geometry tolerance.
            with localcontext() as context:
                context.prec = 40
                for center, partner, other in ((a, b, other_ends[0]),
                                                (b, a, other_ends[1])):
                    axis = [points[partner][i] - points[center][i] for i in (0, 1)]
                    arm = [points[other][i] - points[center][i] for i in (0, 1)]
                    determinant = axis[0] * arm[1] - axis[1] * arm[0]
                    assert abs(determinant) <= epsilon * max(length, D(1)), "unknown Double requires raw collinearity at both ends"
                    assert axis[0] * arm[0] + axis[1] * arm[1] < 0, "unknown Double substituent must be antiparallel"
            assert actual.GetStereo() in (Chem.BondStereo.STEREONONE,
                                          Chem.BondStereo.STEREOANY), "assigned Double configuration"
        else:
            assert actual.GetStereo() == Chem.BondStereo.STEREONONE, "ANY requires the qualified collinear source context; assigned stereo is forbidden"


with localcontext() as context:
    context.prec = 40
    for a, b, _ in actual_edges:
        u, v = points[a - 1], points[b - 1]
        distance = ((u[0] - v[0]) ** 2 + (u[1] - v[1]) ** 2).sqrt()
        assert abs(distance - length) <= epsilon, "actual raw Decimal bond length"
    for i, point in enumerate(points):
        for other in points[i + 1:]:
            assert ((point[0] - other[0]) ** 2 + (point[1] - other[1]) ** 2).sqrt() > epsilon
        for a, b, _ in actual_edges:
            if i + 1 not in (a, b):
                assert not on_segment(points[a - 1], points[b - 1], point)
    for index, (a, b, _) in enumerate(actual_edges):
        for c, d, _ in actual_edges[index + 1:]:
            if {a, b} & {c, d}:
                continue
            u, v, p, q = [points[number - 1] for number in (a, b, c, d)]
            proper = cross(u, v, p) * cross(u, v, q) < 0 and cross(p, q, u) * cross(p, q, v) < 0
            assert not (proper or on_segment(u, v, p) or on_segment(u, v, q)
                        or on_segment(p, q, u) or on_segment(p, q, v))

molecules = Chem.MolsFromCDXML(raw.decode("ascii"), sanitize=True, removeHs=False)
assert len(molecules) == 1
molecule = molecules[0]
assert molecule.GetNumAtoms() == len(nodes) and molecule.GetNumBonds() == len(edges)
assert [atom.GetAtomicNum() for atom in molecule.GetAtoms()] == [a[0] for a in expected["atoms"]]
assert [atom.GetTotalNumHs() for atom in molecule.GetAtoms()] == [a[1] for a in expected["atoms"]]
assert all(atom.GetFormalCharge() == atom.GetIsotope() == atom.GetNumRadicalElectrons() == 0
           and atom.GetChiralTag() == Chem.ChiralType.CHI_UNSPECIFIED
           for atom in molecule.GetAtoms())
verify_unassigned_bonds(reference, molecule, points, length, epsilon)
assert Chem.MolToSmiles(molecule, isomericSmiles=True) == Chem.MolToSmiles(reference, isomericSmiles=True)
assert rdMolDescriptors.CalcMolFormula(molecule) == expected["formula"]
print(json.dumps({"atoms": len(nodes), "bonds": len(edges),
                  "formula": expected["formula"],
                  "stereo_none": all(bond.GetStereo() == Chem.BondStereo.STEREONONE
                                     for bond in molecule.GetBonds()),
                  "no_assigned_stereo": True,
                  "bond_stereo": [str(bond.GetStereo()) for bond in molecule.GetBonds()],
                  "physical_h": sum(a[0] == 1 for a in expected["atoms"])}))
