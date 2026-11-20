"""Compare the real CLI's formula with the independent RDKit CDX reader (test only)."""
import argparse
from collections import Counter
import json
from pathlib import Path
import subprocess
import tempfile

from rdkit import Chem
from rdkit.Chem import rdMolDescriptors


def dispatch(cli, operations):
    args = [str(cli), "run"]
    for command, params in operations:
        args.extend(["--cmd", json.dumps({"command": command, "params": params})])
    return json.loads(subprocess.check_output(args))


def verify_formula(cli, path, query, expected, omitted):
    state = dispatch(cli, [("document.open_cdx", {"path": str(path)}), query])
    report = state["analysis"]
    assert report["status"] == "available" and report["formula"] == expected, (path, report)
    assert report.get("defaulted_hydrogen_atoms", 0) == omitted, (path, report)
    counts = [
        atom["explicit_hydrogens"]
        for page in state["document"]["pages"]
        for fragment in page["fragments"]
        for atom in fragment["atoms"]
    ]
    assert counts.count(None) == omitted
    assert not state["file"]["dirty"] and state["undo_depth"] == 0


def verify_ethanol_selection(cli, path, options):
    mol, = Chem.MolsFromCDXML(path.read_bytes(), options)
    mol.UpdatePropertyCache(strict=True)
    # The independent fixture orders the atoms as C–C–O, which fixes the live IDs.
    atoms = list(mol.GetAtoms())
    assert [atom.GetSymbol() for atom in atoms] == ["C", "C", "O"]
    for ids in [[3], [2], [1], [2, 3], [1, 3], [1, 2, 3]]:
        selected = [atoms[i - 1] for i in ids]
        counts = Counter(atom.GetSymbol() for atom in selected)
        counts["H"] = sum(atom.GetTotalNumHs() for atom in selected)
        formula = "".join(symbol + (str(counts[symbol]) if counts[symbol] > 1 else "")
                          for symbol in ["C", "H", "O"] if counts[symbol])
        state = dispatch(cli, [
            ("document.open_cdx", {"path": str(path)}),
            ("ui.selection", {"action": "set", "expected_revision": 1, "page_id": 1, "atom_ids": ids}),
            ("analysis.formula", {"expected_revision": 1, "expected_selection_epoch": 1}),
        ])
        expected = {"scope": "selection", "status": "available", "atom_count": len(ids),
                    "fragment_count": 1, "partial_fragments": len(ids) < 3, "formula": formula}
        if counts["C"]:
            expected["defaulted_hydrogen_atoms"] = counts["C"]
        assert state["analysis"] == expected, (path, ids, state["analysis"])
        assert not state["file"]["dirty"] and state["undo_depth"] == 0


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("cli", type=Path)
    cli = parser.parse_args().cli.resolve()
    fixtures = Path(__file__).resolve().parents[2] / "crates/io/tests/fixtures"
    options = Chem.CDXMLParserParams()
    options.format = Chem.CDXMLFormat.CDX
    options.sanitize = options.removeHs = False
    with tempfile.TemporaryDirectory(prefix="bondline-cdx-formula-") as tmp:
        for name, omitted in [
            ("cdx-skeletal/propane", 0),
            ("cdx-skeletal/missing_h", 1),
            ("cdx-skeletal/all_h_missing", 3),
            ("cdx-skeletal-cycle/cycle", 4),
            ("cdx-mixed-skeleton/literal_h", 0),
            ("cdx-mixed-skeleton/methanol", 1),
            ("cdx-mixed-skeleton/ethanol", 2),
            ("cdx-mixed-skeleton/amino_alcohol", 3),
        ]:
            source = (fixtures / name).with_suffix(".cdx")
            original = source.read_bytes()
            mol, = Chem.MolsFromCDXML(original, options)
            mol.UpdatePropertyCache(strict=True)
            expected = rdMolDescriptors.CalcMolFormula(mol)
            saved = Path(tmp) / ("保存-" + source.name)
            query = ("analysis.formula", {"expected_revision": 1, "expected_selection_epoch": 0})
            verify_formula(cli, source, query, expected, omitted)
            dispatch(cli, [
                ("document.open_cdx", {"path": str(source)}),
                query,
                ("document.save_cdx", {"path": str(saved)}),
            ])
            assert saved.read_bytes() == original == source.read_bytes()
            verify_formula(cli, saved, query, expected, omitted)
            if name == "cdx-mixed-skeleton/ethanol":
                verify_ethanol_selection(cli, source, options)
                verify_ethanol_selection(cli, saved, options)
                moved = Path(tmp) / "移動-ethanol.cdx"
                dispatch(cli, [
                    ("document.open_cdx", {"path": str(saved)}),
                    ("ui.selection", {"action": "set", "expected_revision": 1, "page_id": 1, "atom_ids": [2]}),
                    ("selection.translate", {"expected_revision": 1, "expected_selection_epoch": 1,
                                             "delta_x_pt": 8.25, "delta_y_pt": -5.125}),
                    ("history.undo", {}), ("history.redo", {}),
                    ("document.save_cdx", {"path": str(moved)}),
                ])
                assert moved.read_bytes() == (fixtures / "cdx-mixed-skeletal-move/hidden.cdx").read_bytes()
                verify_ethanol_selection(cli, moved, options)
            print(f"{name}: {expected}, {omitted} defaulted atom(s)")


if __name__ == "__main__":
    main()
