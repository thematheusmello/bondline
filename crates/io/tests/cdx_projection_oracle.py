"""Independent stdlib field decoding and pinned external molecular-consumer checks."""

import json
from pathlib import Path
import runpy
import struct
import sys

from rdkit import Chem, rdBase
from rdkit.Chem import rdMolDescriptors

assert rdBase.rdkitVersion == "2026.09.2", "use pinned BONDLINE_PYTHON"
assert Chem.HasChemDrawCDXSupport(), "external consumer lacks CDX support"
original_oracle = runpy.run_path(
    str(Path(__file__).resolve().parents[3] / "xtask/oracles/cdx_oracle.py")
)
source = sys.stdin.buffer.read(original_oracle["LIMIT"] + 1)
records = original_oracle["records"](source)
coordinates = [
    struct.unpack_from("<ii", source, row["payload_start"])
    for row in records
    if row.get("tag") == 0x0200
]
positions = [[x / 65536, y / 65536] for y, x in coordinates]
assert positions == [[10.25, -3.5], [26.25, 8.5]]
parameters = Chem.CDXMLParserParams()
parameters.format = Chem.CDXMLFormat.CDX
molecules = Chem.MolsFromCDXML(source, parameters)
assert len(molecules) == 1
molecule = molecules[0]
assert [atom.GetAtomicNum() for atom in molecule.GetAtoms()] == [6, 6]
assert molecule.GetNumBonds() == 1
bond = molecule.GetBondWithIdx(0)
assert (bond.GetBeginAtomIdx(), bond.GetEndAtomIdx(), bond.GetBondTypeAsDouble()) == (0, 1, 1)
assert Chem.MolToSmiles(molecule) == "CC"
assert rdMolDescriptors.CalcMolFormula(molecule) == "C2H6"
print(json.dumps({"positions_pt": positions, "external_smiles": "CC"}, ensure_ascii=True))
