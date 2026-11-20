"""Read an original or pinned coordinate-free carbon drawing in RDKit and Bondline."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile

from rdkit import Chem
from rdkit.Chem import Descriptors, rdMolDescriptors


def commands(cli, operations, *, success=True):
    args = [str(cli), "run"]
    for name, params in operations:
        args += ["--cmd", json.dumps({"command": name, "params": params})]
    result = subprocess.run(args, capture_output=True)
    assert (result.returncode == 0) == success, (operations[-1][0], result.stderr.decode())
    return json.loads(result.stdout or result.stderr)


def reference(data):
    params = Chem.CDXMLParserParams()
    params.format = Chem.CDXMLFormat.CDXML
    params.removeHs = False
    mol, = Chem.MolsFromCDXML(data, params)
    return rdMolDescriptors.CalcMolFormula(mol), Descriptors.MolWt(mol), Descriptors.ExactMolWt(mol)


def verify(cli, path, destination, omitted):
    original = path.read_bytes()
    expected, weight, exact = reference(original)
    path_open = lambda p: ("document.open_cdxml", {"path": str(p)})
    query = {"expected_revision": 1, "expected_selection_epoch": 0}
    commands(cli, [path_open(path), ("analysis.properties", query), ("document.save_cdxml", {"path": str(destination)})])
    for p in (path, destination):
        assert p.read_bytes() == original
        assert reference(p.read_bytes()) == (expected, weight, exact)
        state = commands(cli, [path_open(p), ("analysis.properties", query)])
        data = state["analysis"]
        assert (data["status"], data["scope"], data["formula"]) == ("available", "document", expected)
        assert data["defaulted_hydrogen_atoms"] == omitted and data["atom_count"] == 4
        # The published average weights agree to 1e-8 for C/H, isotope references to 1e-6 Da.
        assert abs(data["masses"]["molecular_weight"] - weight) < 1e-8
        assert abs(data["masses"]["exact_mass_da"] - exact) < 1e-6
        assert not state["file"]["dirty"] and state["undo_depth"] == 0
        atoms = state["document"]["pages"][0]["fragments"][0]["atoms"]
        assert all(a["position"] is None for a in atoms)
        assert sum(a["explicit_hydrogens"] is None for a in atoms) == omitted
        for name, params in [
            ("ui.selection", {"action": "set", "expected_revision": 1, "page_id": 1, "atom_ids": [1]}),
            ("render.capture", {"page_id": 1, "width_px": 128, "height_px": 128, "backend": "cpu"}),
        ]:
            result = commands(cli, [path_open(p), (name, params)], success=False)
            assert result["error"]["code"] == "unsupported_content", (name, result)
    print(f"{path.name}: {expected}; molecular weight {weight:.3f}, exact mass {exact:.6f} Da; {omitted} defaulted")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("cli", type=Path)
    parser.add_argument("--pinned", type=Path)
    opts = parser.parse_args()
    root = Path(__file__).resolve().parents[2]
    original = root / "crates/io/tests/fixtures/cdxml-carbon-analysis/butane.cdxml"
    with tempfile.TemporaryDirectory() as tmp:
        verify(opts.cli.resolve(), original, Path(tmp) / "original-保存.cdxml", 3)
        if opts.pinned:
            assert hashlib.sha256(opts.pinned.read_bytes()).hexdigest() == "fadcbe8eee438dfb55232cb2c3baca41ea6613edb0d6163af9486b6c84ff6fc9"
            verify(opts.cli.resolve(), opts.pinned.resolve(), Path(tmp) / "épreuve-保存.cdxml", 4)


if __name__ == "__main__":
    main()
