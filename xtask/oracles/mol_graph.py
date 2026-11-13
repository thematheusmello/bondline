"""External RDKit/Decimal oracle for the one pinned V2000 methyl-radical input.

Original MIT OR Apache-2.0 developer-only code; no Bondline parser imports.
Numeric fields follow BIOVIA Chemistry 2026 CTfile Formats, pp. 62, 66-70, 90.
This deliberately rejects other graph shapes; extend its evidence with new corpus entries.
"""
from decimal import Decimal
import json
import sys
from rdkit import Chem, rdBase

assert rdBase.rdkitVersion == "2026.09.2", "use the pinned BONDLINE_PYTHON interpreter"
source = sys.stdin.buffer.read(8 * 1024 * 1024 + 1)
assert len(source) <= 8 * 1024 * 1024, "corpus MOL exceeds the read bound"
lines = source.decode("ascii").splitlines()
assert len(lines) >= 5 and lines[3][33:39] == " V2000"
assert int(lines[3][:3]) == 1 and int(lines[3][3:6]) == 0, "single-atom corpus subset only"
molecule = Chem.MolFromMolBlock(source.decode("ascii"), sanitize=True, removeHs=False, strictParsing=True)
assert molecule is not None and molecule.GetNumAtoms() == 1 and molecule.GetNumBonds() == 0
atom = molecule.GetAtomWithIdx(0)
assert atom.GetNumRadicalElectrons() == 1, "corpus input is a doublet"
assert atom.GetNumImplicitHs() == 0, "corpus hydrogens must be explicitly authored"
declared_valence = int(lines[4][48:51])
assert declared_valence == 3 and atom.GetNumExplicitHs() == 3
coordinates = [Decimal(lines[4][start:start + 10]) for start in (0, 10, 20)]
assert all(value.is_finite() for value in coordinates) and coordinates[2] == 0
position = list(molecule.GetConformer().GetAtomPosition(0))
assert position == [float(value) for value in coordinates], "external raw coordinate disagreement"
isotope = atom.GetIsotope() or None
json.dump({
    "raw": {
        "header": lines[:3],
        "atoms": [{
            "symbol": atom.GetSymbol(), "element": atom.GetAtomicNum(), "position": position,
            "mass_difference": int(lines[4][34:36]), "isotope": isotope,
            "charge": atom.GetFormalCharge(), "radical": "doublet", "declared_valence": declared_valence,
        }],
        "bonds": [],
    },
    "fragment": {
        "id": 1,
        "atoms": [{
            "id": 1, "element": atom.GetAtomicNum(),
            "position": {"x_pt": float(coordinates[0] * Decimal(24)), "y_pt": float(-coordinates[1] * Decimal(24))},
            "isotope": isotope, "charge": atom.GetFormalCharge(), "radical": "doublet",
            "explicit_hydrogens": atom.GetNumExplicitHs(),
        }],
        "bonds": [],
    },
}, sys.stdout, ensure_ascii=True, allow_nan=False, separators=(",", ":"))
