"""Independent test process only; never a linked/shipped chemistry dependency."""
import collections
import json
import re
import sys

from rdkit import Chem, rdBase
from rdkit.Chem import rdMolDescriptors

assert len(sys.argv) in (4, 6), "notation, raw RDKit formulas and optionally both Hill outputs"
raw = sys.stdin.buffer.read(65537)
assert len(raw) <= 65536, "test payload bound"
actual = json.loads(raw)
assert isinstance(actual, dict) and set(actual) == {
    "element_counts", "isotope_counts", "total_charge"
}, "closed composition object"
for key, size in [("element_counts", 2), ("isotope_counts", 3)]:
    assert isinstance(actual[key], list)
    for row in actual[key]:
        assert isinstance(row, list) and len(row) == size
        assert type(row[0]) is int and 1 <= row[0] <= 118
        if size == 3:
            assert row[1] is None or (type(row[1]) is int and 1 <= row[1] <= 65535)
        assert type(row[-1]) is int and 1 <= row[-1] <= 2**64 - 1
assert type(actual["total_charge"]) is int
assert -(2**63) <= actual["total_charge"] < 2**63

params = Chem.SmilesParserParams()
params.removeHs = False
reference = Chem.MolFromSmiles(sys.argv[1], params)
assert reference is not None
# These reference vectors specify every H; no toolkit valence filling is used.
assert all(atom.GetNumImplicitHs() == 0 for atom in reference.GetAtoms())
formula = rdMolDescriptors.CalcMolFormula(reference)
isotope_formula = rdMolDescriptors.CalcMolFormula(
    reference, separateIsotopes=True, abbreviateHIsotopes=False
)
assert formula == sys.argv[2], (formula, sys.argv[2])
assert isotope_formula == sys.argv[3], (isotope_formula, sys.argv[3])
elements, isotopes = collections.Counter(), collections.Counter()
for atom in reference.GetAtoms():
    z, isotope = atom.GetAtomicNum(), atom.GetIsotope() or None
    elements[z] += 1
    isotopes[(z, isotope)] += 1
    attached = atom.GetNumExplicitHs()
    if attached:
        elements[1] += attached
        isotopes[(1, None)] += attached
expected = {
    "element_counts": [[z, count] for z, count in sorted(elements.items())],
    "isotope_counts": [[z, isotope, count] for (z, isotope), count in sorted(
        isotopes.items(), key=lambda item: (item[0][0], item[0][1] or 0))],
    "total_charge": Chem.GetFormalCharge(reference),
}
assert actual == expected, (actual, expected)

if len(sys.argv) == 6:
    def strict_hill(rdkit_formula):
        if 6 in elements:
            return rdkit_formula
        # RDKit 2026.09.2 places H first even without carbon. Keep its complete
        # isotope/count tokens, moving whole element groups into Hill's alphabetic order.
        body, charge = re.fullmatch(r"(.*?)([+-][0-9]*)?", rdkit_formula).groups()
        tokens = re.findall(r"(?:\[[0-9]+[A-Z][a-z]?\]|[A-Z][a-z]?)[0-9]*", body)
        assert "".join(tokens) == body
        return "".join(sorted(tokens, key=lambda t: re.search(r"[A-Z][a-z]?", t)[0])) + (charge or "")

    for produced, raw_formula in zip(sys.argv[4:], (formula, isotope_formula), strict=True):
        assert produced == strict_hill(raw_formula), (produced, raw_formula, strict_hill(raw_formula))
print(json.dumps({"rdkit": rdBase.rdkitVersion, "formula": formula,
                  "isotope_formula": isotope_formula, **expected}, separators=(",", ":")))
