"""Original bounded simple-ring actual-CDXML/Decimal/RDKit consumer, external only.

Expected atoms, directed edges, H counts and formula are literal test vectors, not
output from Bondline's normalizer. Geometry is checked independently of its layout.
"""

from decimal import Decimal as D, localcontext
import json
import math
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
assert reference.GetNumBonds() == reference.GetNumAtoms(), "one connected unicyclic graph"
source_rings = [tuple(ring) for ring in Chem.GetSymmSSSR(reference)]
assert len(source_rings) == 1 and 3 <= len(source_rings[0]) <= 8
cycle = source_rings[0]
assert len(set(cycle)) == len(cycle)
cycle_edges = [(cycle[i], cycle[(i + 1) % len(cycle)]) for i in range(len(cycle))]
assert all(reference.GetBondBetweenAtoms(a, b).GetBondType() == Chem.BondType.SINGLE
           for a, b in cycle_edges), "cycle edges must remain Single"
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

# Independent source-ring perception and analytic raw-coordinate polygon checks.
# This is diagram geometry, never a 3D conformation/strain/chemistry certificate.
with localcontext() as context:
    context.prec = 40
    n = len(cycle)
    center = [sum(points[i][axis] for i in cycle) / D(n) for axis in (0, 1)]
    radius = length / (2 * D(str(math.sin(math.pi / n))))
    expected_cosine = -D(str(math.cos(2 * math.pi / n)))
    turns = []
    for i, current in enumerate(cycle):
        previous, following = cycle[(i - 1) % n], cycle[(i + 1) % n]
        at, before, after = points[current], points[previous], points[following]
        distance = sum((at[axis] - center[axis]) ** 2 for axis in (0, 1)).sqrt()
        assert abs(distance - radius) <= epsilon, "regular ring radius"
        u = [before[axis] - at[axis] for axis in (0, 1)]
        v = [after[axis] - at[axis] for axis in (0, 1)]
        assert abs(sum(u[axis] * v[axis] for axis in (0, 1)) / (length * length)
                   - expected_cosine) <= D("1e-9"), "regular ring interior angle"
        turn = cross(before, at, after)
        assert abs(turn) > epsilon * max(length, D(1))
        turns.append(turn > 0)
    assert all(turn == turns[0] for turn in turns), "ring must be convex and ordered"
    if n == 6:
        extreme = min(points[i][0] for i in cycle)
        assert any(abs(points[a][0] - extreme) <= epsilon
                   and abs(points[b][0] - extreme) <= epsilon
                   and abs(abs(points[a][1] - points[b][1]) - length) <= epsilon
                   for a, b in cycle_edges), "hexagon preferred vertical-left edge"
    else:
        extreme = max(points[i][1] for i in cycle)
        assert any(abs(points[a][1] - extreme) <= epsilon
                   and abs(points[b][1] - extreme) <= epsilon
                   and abs(abs(points[a][0] - points[b][0]) - length) <= epsilon
                   for a, b in cycle_edges), "preferred horizontal-bottom edge"

    def direction(start, end):
        vector = [points[end][axis] - points[start][axis] for axis in (0, 1)]
        norm = sum(value * value for value in vector).sqrt()
        assert norm > 0
        return [value / norm for value in vector]

    def cosine(a, b):
        return sum(a[axis] * b[axis] for axis in (0, 1))

    cycle_set = set(cycle)
    for at in cycle:
        outside = [atom.GetIdx() for atom in reference.GetAtomWithIdx(at).GetNeighbors()
                   if atom.GetIdx() not in cycle_set]
        assert len(outside) <= 2
        radial = [points[at][axis] - center[axis] for axis in (0, 1)]
        norm = sum(value * value for value in radial).sqrt()
        radial = [value / norm for value in radial]
        vectors = [direction(at, other) for other in outside]
        if len(outside) == 1:
            angle = math.atan2(float(radial[1]), float(radial[0]))
            snap = round(math.degrees(angle) / 30) * 30
            if abs(snap - math.degrees(angle)) <= 5 + 1e-9:
                angle = math.radians(snap)
            target = [D(str(math.cos(angle))), D(str(math.sin(angle)))]
            assert abs(cosine(vectors[0], target) - 1) <= D("1e-9"), "radial substituent with source-qualified snap"
        elif len(outside) == 2:
            degrees = [reference.GetAtomWithIdx(other).GetDegree() for other in outside]
            pair = cosine(vectors[0], vectors[1])
            assert all(cosine(vector, radial) > 0 for vector in vectors), "geminal substituents point outward"
            if all(degree >= 3 for degree in degrees):
                expected_angle = (180 + 360 / n) / 3
                assert abs(pair - D(str(math.cos(math.radians(expected_angle))))) <= D("1e-9")
                bisector = [vectors[0][axis] + vectors[1][axis] for axis in (0, 1)]
                norm = sum(value * value for value in bisector).sqrt()
                assert abs(cosine([value / norm for value in bisector], radial) - 1) <= D("1e-9")
            elif n == 6 and (degrees[0] == 1) != (degrees[1] == 1):
                larger = vectors[0] if degrees[0] > 1 else vectors[1]
                assert abs(cosine(larger, radial) - 1) <= D("1e-9")
                assert abs(pair - D("0.5")) <= D("1e-9")
            else:
                assert abs(pair - D("0.5")) <= D("1e-9") or (n == 4 and abs(pair) <= D("1e-9"))

molecules = Chem.MolsFromCDXML(raw.decode("ascii"), sanitize=True, removeHs=False)
assert len(molecules) == 1
molecule = molecules[0]
actual_rings = [tuple(ring) for ring in Chem.GetSymmSSSR(molecule)]
assert len(actual_rings) == 1 and set(actual_rings[0]) == set(cycle)
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
                  "physical_h": sum(a[0] == 1 for a in expected["atoms"]),
                  "ring_size": len(cycle)}))
