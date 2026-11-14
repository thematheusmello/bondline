"""Original MIT OR Apache-2.0 external SMILES consumer; never linked or shipped.

Pinned RDKit parser settings deliberately retain physical H and avoid chemical
inference. RDKit acceptance alone is not an OpenSMILES or raw-token oracle.
https://github.com/rdkit/rdkit/blob/f86b7dc9b31e2531200bbd1f54589bcbd968bb05/Code/GraphMol/SmilesParse/SmilesParse.h
"""

import hashlib
import json
import sys

from rdkit import Chem, rdBase

MAX_BYTES = 65_536


def describe(source):
    assert len(source) <= MAX_BYTES, "external input byte budget"
    text = source.decode("ascii")
    parameters = Chem.SmilesParserParams()
    parameters.sanitize = False
    parameters.removeHs = False
    parameters.parseName = False
    parameters.allowCXSMILES = False
    assert not hasattr(parameters, "skipCleanup"), "qualify the pinned Python API before changing cleanup"
    molecule = Chem.MolFromSmiles(text, parameters)
    result = {
        "rdkit": rdBase.rdkitVersion,
        "source_ascii": text,
        "source_bytes": len(source),
        "source_sha256": hashlib.sha256(source).hexdigest(),
        "parameters": {"sanitize": parameters.sanitize,
                       "removeHs": parameters.removeHs,
                       "parseName": parameters.parseName,
                       "allowCXSMILES": parameters.allowCXSMILES},
        "skip_cleanup_exposed": False,
        "accepted": molecule is not None,
    }
    if molecule is not None:
        result["atoms"] = [{
            "element": atom.GetAtomicNum(), "isotope": atom.GetIsotope(),
            "charge": atom.GetFormalCharge(), "explicit_h": atom.GetNumExplicitHs(),
            "no_implicit": atom.GetNoImplicit(),
            "radical_electrons": atom.GetNumRadicalElectrons(),
            "needs_update_cache": atom.NeedsUpdatePropertyCache(),
        } for atom in molecule.GetAtoms()]
        result["bonds"] = sorted([
            min(bond.GetBeginAtomIdx(), bond.GetEndAtomIdx()),
            max(bond.GetBeginAtomIdx(), bond.GetEndAtomIdx()),
            str(bond.GetBondType()),
        ] for bond in molecule.GetBonds())
    return result


if __name__ == "__main__":
    assert rdBase.rdkitVersion == "2026.09.2", "use pinned BONDLINE_PYTHON"
    assert len(sys.argv) == 1
    source = sys.stdin.buffer.read(MAX_BYTES + 1)
    sys.stdout.buffer.write(json.dumps(describe(source), ensure_ascii=True,
                                      allow_nan=False, separators=(",", ":")).encode("ascii") + b"\n")
