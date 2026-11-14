"""Original MIT OR Apache-2.0 actual-CDXML Decimal/chemical consumer; external only."""
from decimal import Decimal as D, localcontext
from xml.parsers import expat
import json
import sys
from rdkit import Chem, rdBase
from rdkit.Chem import rdMolDescriptors

assert rdBase.rdkitVersion == "2026.09.2", "use pinned BONDLINE_PYTHON"
assert len(sys.argv) == 2
expected = json.loads(sys.argv[1])
assert 1 <= len(expected) <= 12
source = sys.stdin.buffer.read(65_537)
assert len(source) <= 65_536
points = []
parser = expat.ParserCreate()
parser.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
def start(name, attributes):
    if name == "n":
        points.append([D(value) for value in attributes["p"].split()])
parser.StartElementHandler = start
parser.Parse(source, True)
assert points == [[D(x), D(y)] for x, y in expected], "raw XML must retain every authored digit"
# Independent high-precision analytic vector for this fixture's L48/origin72 policy.
with localcontext() as context:
    context.prec = 40
    n = len(points)
    a = D(3).sqrt() / 2
    cx, cy = D(n - 1) * a, D("0.5") if n % 2 == 0 else D(0)
    r = (cx * cx + cy * cy).sqrt()
    cosine, sine = (cx / r, cy / r) if n > 1 else (D(1), D(0))
    for i, (x, y) in enumerate(points):
        u, v = D(i) * 48 * a, D(24) if i % 2 else D(0)
        assert abs(x - (72 + u * cosine + v * sine)) < D("1e-9")
        assert abs(y - (72 + v * cosine - u * sine)) < D("1e-9")
molecules = Chem.MolsFromCDXML(source.decode("utf-8"))
assert len(molecules) == 1
molecule = molecules[0]
assert all(atom.GetAtomicNum() == 6 and atom.GetFormalCharge() == 0
           and atom.GetIsotope() == 0 and atom.GetNumRadicalElectrons() == 0
           for atom in molecule.GetAtoms())
assert all(bond.GetBondTypeAsDouble() == 1 for bond in molecule.GetBonds())
print(json.dumps({"smiles": Chem.MolToSmiles(molecule),
                  "formula": rdMolDescriptors.CalcMolFormula(molecule),
                  "atoms": molecule.GetNumAtoms(), "bonds": molecule.GetNumBonds()}))
