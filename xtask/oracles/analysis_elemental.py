"""Independently compute per-element masses using RDKit's explicit-H molecular graph."""
from collections import defaultdict
from decimal import Decimal
import json
import sys

from rdkit import Chem

def check(molecule, report, selected=None, *, natural_weights=None):
    initial_atoms = molecule.GetNumAtoms()
    expanded = Chem.AddHs(molecule)
    weights = defaultdict(Decimal)
    for atom in expanded.GetAtoms():
        # Added H nodes belong to the selected original parent; pre-existing
        # isotope-labelled H is an independent atom in the source selection.
        if selected is not None:
            origin = atom.GetIdx() if atom.GetIdx() < initial_atoms else atom.GetNeighbors()[0].GetIdx()
            if origin not in selected:
                continue
        weight = Decimal(str(atom.GetMass()))
        if natural_weights and not atom.GetIsotope():
            weight = natural_weights.get(atom.GetSymbol(), weight)
        weights[(atom.GetAtomicNum(), atom.GetSymbol())] += weight
    total = sum(weights.values())
    rows = report["elemental_analysis"]["elements"]
    assert len(rows) == len(weights)
    for row, ((atomic_number, symbol), weight) in zip(rows, sorted(weights.items())):
        assert (row["atomic_number"], row["symbol"]) == (atomic_number, symbol)
        assert abs(Decimal(str(row["weight_contribution"])) - weight) < Decimal("1e-8")
        fraction = 100 * weight / total
        assert abs(Decimal(str(row["percent_by_weight"])) - fraction) < Decimal("1e-7")
    assert abs(Decimal(str(report["masses"]["molecular_weight"])) - total) < Decimal("1e-8")


if __name__ == "__main__":
    check(Chem.MolFromSmiles(sys.argv[1]), json.load(sys.stdin))
