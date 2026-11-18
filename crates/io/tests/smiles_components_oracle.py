"""Original external actual-CDXML consumer for the bounded neutral-components profile.

Literal graph expectations are checked against the original notation by pinned RDKit;
raw XML/Decimal checks separately verify directed IDs, geometry and shelf packing.
The input is one complete Fragment, irrespective of the external reader's Mol count.
"""

from decimal import Decimal as D, localcontext
import json
import math
import sys
from xml.parsers import expat

from rdkit import Chem, rdBase
from rdkit.Chem import rdMolDescriptors

assert rdBase.rdkitVersion == "2026.09.2", "use the pinned external interpreter"
assert len(sys.argv) in (2, 3)
original = sys.argv[1]
assert 0 < len(original.encode("ascii")) <= 65_536
parameters = Chem.SmilesParserParams()
parameters.sanitize, parameters.removeHs = True, False
parameters.parseName, parameters.allowCXSMILES = False, False
reference = Chem.MolFromSmiles(original, parameters)
assert reference is not None and 1 <= reference.GetNumAtoms() <= 64
assert all(not atom.GetIsAromatic() and atom.GetAtomicNum() in (1, 6, 7, 8, 9, 17, 35, 53)
           and atom.GetFormalCharge() == atom.GetIsotope() == atom.GetNumRadicalElectrons() == 0
           and atom.GetChiralTag() == Chem.ChiralType.CHI_UNSPECIFIED
           for atom in reference.GetAtoms())
assert all(bond.GetBondTypeAsDouble() in (1, 2, 3)
           and bond.GetStereo() == Chem.BondStereo.STEREONONE
           for bond in reference.GetBonds())
components = sorted([sorted(index + 1 for index in group)
                     for group in Chem.GetMolFrags(reference)], key=lambda group: group[0])
expected = {
    "atoms": [[atom.GetAtomicNum(), atom.GetTotalNumHs()] for atom in reference.GetAtoms()],
    "components": components,
    "formula": rdMolDescriptors.CalcMolFormula(reference),
    "length": "48", "origin": ["72", "72"],
    "page": [str(D(210) * 72 / D("25.4")), str(D(297) * 72 / D("25.4"))],
}
options = json.loads(sys.argv[2]) if len(sys.argv) == 3 else {}
assert isinstance(options, dict)
assert set(options) <= {"atoms", "bonds", "components", "formula", "length", "origin", "page"}
for name in ("atoms", "components", "formula"):
    if name in options:
        assert options[name] == expected[name], "literal expectation differs from original graph"
expected.update(options)
source_rings = [tuple(ring) for ring in Chem.GetSymmSSSR(reference)]
for group in components:
    members = {index - 1 for index in group}
    bonds = [bond for bond in reference.GetBonds()
             if bond.GetBeginAtomIdx() in members and bond.GetEndAtomIdx() in members]
    rings = [ring for ring in source_rings if set(ring) <= members]
    if len(bonds) == len(group) - 1:
        assert not rings, "tree component must have no ring"
    else:
        assert len(bonds) == len(group) and len(rings) == 1 and 3 <= len(rings[0]) <= 8
        ring = rings[0]
        assert all(reference.GetBondBetweenAtoms(ring[i], ring[(i + 1) % len(ring)])
                   .GetBondType() == Chem.BondType.SINGLE for i in range(len(ring)))

raw = sys.stdin.buffer.read(1_048_577)
assert 0 < len(raw) <= 1_048_576, "bounded actual XML input"
nodes, edges, pages, fragments, stack = [], [], [], [], []
ids, references = set(), {}


def start(name, attributes):
    assert name in ("CDXML", "page", "fragment", "n", "b"), "unexpected drawing content"
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
        assert set(attributes) == {"id", "NodeType", "Element", "Charge", "Radical", "NumHydrogens", "p"}
        assert attributes["NodeType"] == "Element"
        assert attributes["Charge"] == "0" and attributes["Radical"] == "None"
        point = tuple(D(part) for part in attributes["p"].split())
        assert len(point) == 2 and all(value.is_finite() for value in point)
        assert attributes["id"] not in references
        references[attributes["id"]] = len(nodes) + 1
        nodes.append((int(attributes["Element"]), int(attributes["NumHydrogens"]), point))
    elif name == "b":
        assert stack == ["CDXML", "page", "fragment"]
        assert set(attributes) == {"id", "B", "E", "Order"}
        edges.append((attributes["B"], attributes["E"], int(attributes["Order"])))
    stack.append(name)


def end(name):
    assert stack.pop() == name


def refuse_doctype(*_):
    raise AssertionError("no document type/entities in this generated profile")


parser = expat.ParserCreate()
parser.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
parser.StartElementHandler, parser.EndElementHandler = start, end
parser.StartDoctypeDeclHandler = refuse_doctype
parser.Parse(raw, True)
assert not stack and len(pages) == len(fragments) == 1, "one complete Fragment"
assert pages[0]["DrawingSpace"] == "Poster"
assert [[z, h] for z, h, _ in nodes] == expected["atoms"]
actual_edges = [(references[a], references[b], order) for a, b, order in edges]
reference_edges = sorted((min(bond.GetBeginAtomIdx(), bond.GetEndAtomIdx()) + 1,
                          max(bond.GetBeginAtomIdx(), bond.GetEndAtomIdx()) + 1,
                          int(bond.GetBondTypeAsDouble())) for bond in reference.GetBonds())
assert sorted((min(a, b), max(a, b), order) for a, b, order in actual_edges) == reference_edges
assert len({tuple(sorted((a, b))) for a, b, _ in actual_edges}) == len(actual_edges)
if "bonds" in expected:
    assert [list(edge) for edge in actual_edges] == expected["bonds"], "directed source-order graph"
adjacency = {index: set() for index in range(1, len(nodes) + 1)}
for a, b, _ in actual_edges:
    adjacency[a].add(b)
    adjacency[b].add(a)
actual_components, unseen = [], set(adjacency)
while unseen:
    pending, members = [min(unseen)], set()
    while pending:
        at = pending.pop()
        if at not in members:
            members.add(at)
            pending.extend(adjacency[at] - members)
    unseen -= members
    actual_components.append(sorted(members))
assert actual_components == expected["components"], "actual edges, not dot tokens"
points = [point for _, _, point in nodes]
length = D(str(expected["length"]))
origin = [D(str(value)) for value in expected["origin"]]
width, height = [D(str(value)) for value in expected["page"]]
assert len(origin) == 2 and length.is_finite() and length > 0
assert all(value.is_finite() for value in (*origin, width, height)) and width > 0 and height > 0
epsilon = max(length, D(1)) * D("1e-9")
assert all(abs(D(pages[0][name]) - value) < D("1e-9")
           for name, value in (("Width", width), ("Height", height)))
assert abs(min(point[0] for point in points) - origin[0]) <= epsilon
assert abs(min(point[1] for point in points) - origin[1]) <= epsilon
assert all(0 <= x <= width and 0 <= y <= height for x, y in points)


def cross(a, b, point):
    return (b[0] - a[0]) * (point[1] - a[1]) - (b[1] - a[1]) * (point[0] - a[0])


def on_segment(a, b, point):
    return abs(cross(a, b, point)) <= epsilon * max(length, D(1)) and all(
        min(a[axis], b[axis]) - epsilon <= point[axis] <= max(a[axis], b[axis]) + epsilon
        for axis in (0, 1))


with localcontext() as context:
    context.prec = 40
    for a, b, _ in actual_edges:
        u, v = points[a - 1], points[b - 1]
        distance = sum((u[i] - v[i]) ** 2 for i in (0, 1)).sqrt()
        assert abs(distance - length) <= epsilon, "actual raw Decimal bond length"
    for i, point in enumerate(points):
        for other in points[i + 1:]:
            assert sum((point[axis] - other[axis]) ** 2 for axis in (0, 1)).sqrt() > epsilon
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
    cursor_x, row_y, row_height, occupied = *origin, D(0), False
    for group in actual_components:
        values = [points[index - 1] for index in group]
        minima = [min(point[axis] for point in values) for axis in (0, 1)]
        maxima = [max(point[axis] for point in values) for axis in (0, 1)]
        box_width, box_height = [maxima[axis] - minima[axis] for axis in (0, 1)]
        if occupied and cursor_x + box_width > width + epsilon:
            cursor_x, row_y, row_height, occupied = origin[0], row_y + row_height + length, D(0), False
        assert abs(minima[0] - cursor_x) <= epsilon and abs(minima[1] - row_y) <= epsilon, "source-ordered center-box shelf"
        assert maxima[0] <= width and maxima[1] <= height
        cursor_x, row_height, occupied = cursor_x + box_width + length, max(row_height, box_height), True
    for ring in source_rings:
        n = len(ring)
        center = [sum(points[i][axis] for i in ring) / D(n) for axis in (0, 1)]
        radius = length / (2 * D(str(math.sin(math.pi / n))))
        target_cosine = -D(str(math.cos(2 * math.pi / n)))
        turns = []
        ring_edges = [(ring[i], ring[(i + 1) % n]) for i in range(n)]
        for i, at in enumerate(ring):
            before, after = ring[(i - 1) % n], ring[(i + 1) % n]
            distance = sum((points[at][axis] - center[axis]) ** 2 for axis in (0, 1)).sqrt()
            assert abs(distance - radius) <= epsilon, "regular ring radius"
            u = [points[before][axis] - points[at][axis] for axis in (0, 1)]
            v = [points[after][axis] - points[at][axis] for axis in (0, 1)]
            assert abs(sum(u[axis] * v[axis] for axis in (0, 1)) / length ** 2 - target_cosine) <= D("1e-9")
            turn = cross(points[before], points[at], points[after])
            assert abs(turn) > epsilon * max(length, D(1))
            turns.append(turn > 0)
        assert all(turn == turns[0] for turn in turns)
        axis = 0 if n == 6 else 1
        extreme = (min if n == 6 else max)(points[i][axis] for i in ring)
        assert any(abs(points[a][axis] - extreme) <= epsilon and abs(points[b][axis] - extreme) <= epsilon
                   for a, b in ring_edges), "preferred ring edge orientation"

molecules = Chem.MolsFromCDXML(raw.decode("ascii"), sanitize=True, removeHs=False)
assert molecules, "independent molecular reader must accept actual bytes"
molecule = molecules[0]
for other in molecules[1:]:
    molecule = Chem.CombineMols(molecule, other)
assert molecule.GetNumAtoms() == len(nodes) and molecule.GetNumBonds() == len(edges)
assert [[atom.GetAtomicNum(), atom.GetTotalNumHs()] for atom in molecule.GetAtoms()] == expected["atoms"]
assert all(atom.GetFormalCharge() == atom.GetIsotope() == atom.GetNumRadicalElectrons() == 0
           and atom.GetChiralTag() == Chem.ChiralType.CHI_UNSPECIFIED
           for atom in molecule.GetAtoms())
assert sorted(sorted(index + 1 for index in group) for group in Chem.GetMolFrags(molecule)) == components
for original_bond in reference.GetBonds():
    a, b = original_bond.GetBeginAtomIdx(), original_bond.GetEndAtomIdx()
    actual = molecule.GetBondBetweenAtoms(a, b)
    assert actual is not None and actual.GetBondType() == original_bond.GetBondType()
    other_ends = []
    if original_bond.GetBondType() == Chem.BondType.DOUBLE:
        for at, partner in ((a, b), (b, a)):
            atom = reference.GetAtomWithIdx(at)
            others = [bond for bond in atom.GetBonds() if bond.GetOtherAtomIdx(at) != partner]
            if atom.GetDegree() != 2 or len(others) != 1 or others[0].GetBondType() != Chem.BondType.SINGLE:
                break
            other_ends.append(others[0].GetOtherAtomIdx(at))
    if len(other_ends) == 2:
        with localcontext() as context:
            context.prec = 40
            for at, partner, other in ((a, b, other_ends[0]), (b, a, other_ends[1])):
                axis = [points[partner][i] - points[at][i] for i in (0, 1)]
                arm = [points[other][i] - points[at][i] for i in (0, 1)]
                assert abs(axis[0] * arm[1] - axis[1] * arm[0]) <= epsilon * max(length, D(1)), "unknown Double requires raw collinearity"
                assert sum(axis[i] * arm[i] for i in (0, 1)) < 0, "unknown Double must be antiparallel"
        assert actual.GetStereo() in (Chem.BondStereo.STEREONONE, Chem.BondStereo.STEREOANY)
    else:
        assert actual.GetStereo() == Chem.BondStereo.STEREONONE, "assigned or unqualified unknown bond stereo"
assert Chem.MolToSmiles(molecule, isomericSmiles=True) == Chem.MolToSmiles(reference, isomericSmiles=True)
assert rdMolDescriptors.CalcMolFormula(molecule) == expected["formula"]
print(json.dumps({"atoms": len(nodes), "bonds": len(edges), "components": len(components),
                  "formula": expected["formula"],
                  "stereo_none": all(bond.GetStereo() == Chem.BondStereo.STEREONONE for bond in molecule.GetBonds()),
                  "no_assigned_stereo": True,
                  "bond_stereo": [str(bond.GetStereo()) for bond in molecule.GetBonds()],
                  "physical_h": sum(z == 1 for z, _, _ in nodes),
                  "raw_graph": {"atoms": [[z, h] for z, h, _ in nodes],
                                "bonds": [list(edge) for edge in actual_edges],
                                "components": actual_components}}))
