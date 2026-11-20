#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Independent consumer of the real browser pointer-drawn 120-point ethane source."""
from decimal import Decimal
from hashlib import sha256
import json
from pathlib import Path
import sys
from xml.parsers import expat

import rdkit
from rdkit import Chem
from rdkit.Chem import rdMolDescriptors


def main():
    assert rdkit.__version__ == "2026.09.2", rdkit.__version__
    source = Path(sys.argv[1]).read_bytes()
    parser = expat.ParserCreate()
    elements = []
    parser.StartElementHandler = lambda tag, attributes: elements.append((tag, attributes))
    parser.Parse(source, True)
    page, = (a for tag, a in elements if tag == "page")
    nodes = [a for tag, a in elements if tag == "n"]
    bond, = (a for tag, a in elements if tag == "b")
    assert len(nodes) == 2
    assert len({a["id"] for _, a in elements if "id" in a}) == 5
    assert page["DrawingSpace"] == "Poster"
    width, height = Decimal(page["Width"]), Decimal(page["Height"])
    assert abs(width - Decimal(210)*72/Decimal("25.4")) < Decimal("1e-11")
    assert abs(height - Decimal(297)*72/Decimal("25.4")) < Decimal("1e-11")
    positions = [tuple(map(Decimal, node["p"].split())) for node in nodes]
    assert positions[1][0] - positions[0][0] == 120
    assert positions[1][1] == positions[0][1]
    assert all(0 < x < width and 0 < y < height for x, y in positions)
    assert all((node["Element"], node["NumHydrogens"], node["Charge"], node["Radical"])
               == ("6", "3", "0", "None") for node in nodes)
    assert (bond["B"], bond["E"], bond["Order"]) == (nodes[0]["id"], nodes[1]["id"], "1")
    molecule, = Chem.MolsFromCDXML(source.decode("utf-8"))
    assert molecule.GetNumAtoms() == 2 and molecule.GetNumBonds() == 1
    assert [atom.GetTotalNumHs() for atom in molecule.GetAtoms()] == [3, 3]
    assert Chem.MolToSmiles(molecule) == "CC"
    assert rdMolDescriptors.CalcMolFormula(molecule) == "C2H6"
    print(json.dumps({"sha256": sha256(source).hexdigest(), "bytes": len(source),
                      "rdkit": rdkit.__version__, "formula": "C2H6", "smiles": "CC",
                      "bond_length_pt": 120, "page": "A4", "consumers": ["Expat", "Decimal", "RDKit"]}))


if __name__ == "__main__":
    main()
