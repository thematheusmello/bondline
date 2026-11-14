"""Original MIT OR Apache-2.0 generated-SMILES consumer; external only."""

import json
import sys

from rdkit import Chem, rdBase
from rdkit.Chem import rdMolDescriptors
from smiles_oracle import MAX_BYTES, describe


def generated(source):
    notation = describe(source)
    assert notation["accepted"], "unsanitized external consumer rejected generated notation"
    parameters = Chem.SmilesParserParams()
    parameters.sanitize = True
    parameters.removeHs = False
    parameters.parseName = False
    parameters.allowCXSMILES = False
    molecule = Chem.MolFromSmiles(source.decode("ascii"), parameters)
    assert molecule is not None, "separate sanitized consumer rejected supported structure"
    return {"notation": notation, "meaning": {
        "smiles": Chem.MolToSmiles(molecule),
        "formula": rdMolDescriptors.CalcMolFormula(molecule),
        "radicals": [atom.GetNumRadicalElectrons() for atom in molecule.GetAtoms()],
        "conformers": molecule.GetNumConformers(),
    }}


if __name__ == "__main__":
    assert rdBase.rdkitVersion == "2026.09.2", "use pinned BONDLINE_PYTHON"
    assert len(sys.argv) == 1
    source = sys.stdin.buffer.read(MAX_BYTES + 1)
    sys.stdout.buffer.write(json.dumps(generated(source), ensure_ascii=True,
                                      allow_nan=False, separators=(",", ":")).encode("ascii") + b"\n")
