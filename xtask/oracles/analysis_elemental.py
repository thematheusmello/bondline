"""Independently compute per-element masses using RDKit's explicit-H molecular graph."""
from collections import defaultdict
from decimal import Decimal
import json
import sys

from rdkit import Chem

molecule = Chem.AddHs(Chem.MolFromSmiles(sys.argv[1]))
weights = defaultdict(Decimal)
for atom in molecule.GetAtoms():
    weights[(atom.GetAtomicNum(), atom.GetSymbol())] += Decimal(str(atom.GetMass()))
total = sum(weights.values())
report = json.load(sys.stdin)
rows = report["elemental_analysis"]["elements"]
assert len(rows) == len(weights)
for row, ((atomic_number, symbol), weight) in zip(rows, sorted(weights.items())):
    assert (row["atomic_number"], row["symbol"]) == (atomic_number, symbol)
    assert abs(Decimal(str(row["weight_contribution"])) - weight) < Decimal("1e-8")
    fraction = 100 * weight / total
    assert abs(Decimal(str(row["percent_by_weight"])) - fraction) < Decimal("1e-7")
assert abs(Decimal(str(report["masses"]["molecular_weight"])) - total) < Decimal("1e-8")
