"""Independently read command-saved C-C fragments; test tool, never shipped."""
import json
import sys
import xml.parsers.expat

import rdkit
from rdkit import Chem
from rdkit.Chem import rdMolDescriptors

assert rdkit.__version__ == "2026.09.2", rdkit.__version__
expected = int(sys.argv[1])
source = sys.stdin.buffer.read()
ids = set()
fragments = []
stack = []
parser = xml.parsers.expat.ParserCreate()
parser.SetParamEntityParsing(xml.parsers.expat.XML_PARAM_ENTITY_PARSING_NEVER)


def start(name, attributes):
    if not stack:
        assert name == "CDXML"
    if "id" in attributes:
        assert attributes["id"] not in ids
        ids.add(attributes["id"])
    if name == "fragment":
        assert stack[-1] == "page"
        fragments.append({"nodes": {}, "bonds": []})
    elif name == "n":
        assert stack[-1] == "fragment"
        assert attributes["Element"] == "6"
        assert attributes["NumHydrogens"] == "3"
        assert attributes["Charge"] == "0"
        position = tuple(float(value) for value in attributes["p"].split())
        fragments[-1]["nodes"][attributes["id"]] = position
    elif name == "b":
        assert stack[-1] == "fragment"
        assert attributes["Order"] == "1"
        fragments[-1]["bonds"].append((attributes["B"], attributes["E"]))
    stack.append(name)


def end(name):
    assert stack.pop() == name


parser.StartElementHandler = start
parser.EndElementHandler = end
parser.Parse(source, True)
assert len(fragments) == expected
for index, fragment in enumerate(fragments):
    assert set(fragment["nodes"].values()) == {
        (100.0, 200.0 + 100.0 * index),
        (140.0, 200.0 + 100.0 * index),
    }
    assert len(fragment["nodes"]) == 2 and len(fragment["bonds"]) == 1
    begin, end = fragment["bonds"][0]
    assert begin != end and {begin, end} == set(fragment["nodes"])

molecules = Chem.MolsFromCDXML(source.decode("utf-8"))
assert len(molecules) == expected
for molecule in molecules:
    assert molecule.GetNumAtoms() == 2 and molecule.GetNumBonds() == 1
    assert [atom.GetAtomicNum() for atom in molecule.GetAtoms()] == [6, 6]
    assert [atom.GetFormalCharge() for atom in molecule.GetAtoms()] == [0, 0]
    assert [atom.GetTotalNumHs() for atom in molecule.GetAtoms()] == [3, 3]
    assert molecule.GetBondWithIdx(0).GetBondTypeAsDouble() == 1.0
    assert Chem.MolToSmiles(molecule) == "CC"
    assert rdMolDescriptors.CalcMolFormula(molecule) == "C2H6"
print(json.dumps({"fragments": len(fragments), "molecules": len(molecules)}))
