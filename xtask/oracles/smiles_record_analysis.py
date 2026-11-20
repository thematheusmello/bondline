# SPDX-License-Identifier: MIT OR Apache-2.0
"""Compare actual raw-record chemistry before and after a byte-exact Unicode Save."""
from decimal import Decimal
import json
from pathlib import Path
import subprocess
import sys
import tempfile

from rdkit import Chem, rdBase
from rdkit.Chem import rdMolDescriptors

from analysis_elemental import check
from corpus_formula import reference


def run(cli, ops, *, success=True):
    args = [str(cli), "run"]
    for command, params in ops:
        args += ["--cmd", json.dumps({"command": command, "params": params})]
    proc = subprocess.run(args, capture_output=True)
    assert (proc.returncode == 0) == success, (ops[-1][0], proc.stderr.decode())
    return json.loads(proc.stdout if success else proc.stderr)


def main(cli):
    assert rdBase.rdkitVersion == "2026.09.2"
    # RDKit 2026.09.2 uses different natural atomic weights for Na and Cl.
    table = Chem.GetPeriodicTable()
    assert table.GetAtomicWeight(11) == 22.990 and table.GetAtomicWeight(17) == 35.453
    # Independent 2024 CIAAW full Na value and abridged Cl value, as used in Bondline.
    conventional = {"Na": Decimal("22.98976928"), "Cl": Decimal("35.45")}
    fixtures = Path(__file__).resolve().parents[2] / "crates/io/tests/fixtures/smiles"
    cases = [(fixtures.joinpath(name + ".smi").read_text().strip(), defaults)
             for name, defaults in [("isotopic-chain", 0), ("two-ring-labels", 7), ("salt-branch", 3)]]
    cases += [("[2H][NH3+]", 0), ("[H+]", 0), ("Cl", 1), ("C1.C1", 2)]
    params = Chem.SmilesParserParams()
    params.removeHs = False
    params.allowCXSMILES = False
    params.parseName = False
    revisions = {"expected_revision": 1, "expected_selection_epoch": 0}
    query = lambda **extra: ("analysis.properties", revisions | extra)
    with tempfile.TemporaryDirectory() as directory:
        source = Path(directory) / "Mélange.smi"
        saved = Path(directory) / "résultat-保存.smi"
        raw = b"\r\n" + b"".join(s.encode() + b"\topaque application data\r\n" for s, _ in cases)
        unavailable = ["[0C]", "C:C", "c1ccccc1"]
        raw += b"".join(s.encode() + b" opaque\r\n" for s in unavailable)
        source.write_bytes(raw)
        for path in (source, saved):
            opened = ("document.open_smiles", {"path": str(path)})
            baseline = run(cli, [opened, query()])
            assert baseline.pop("analysis") == {"scope": "document", "status": "unsupported_source"}
            for record, (notation, defaults) in enumerate(cases, 1):
                state = run(cli, [opened, query(smiles_record_id=record, include_elemental_analysis=True)])
                report = state.pop("analysis")
                assert state == baseline and not state["file"]["dirty"] and state["undo_depth"] == 0
                mol = Chem.MolFromSmiles(notation, params)
                ref = reference(mol)
                assert report["scope"] == "smiles_record" and report["record_id"] == record
                assert report["formula"] == ref["formula"] and report["atom_count"] == ref["atoms"]
                assert report["fragment_count"] == len(Chem.GetMolFrags(mol))
                assert report.get("defaulted_hydrogen_atoms", 0) == defaults
                assert abs(report["masses"]["exact_mass_da"] - rdMolDescriptors.CalcExactMolWt(mol)) < 1e-6
                check(mol, report, natural_weights=conventional)
            for record in range(len(cases) + 1, len(cases) + len(unavailable) + 1):
                result = run(cli, [opened, query(smiles_record_id=record)])
                answer = result.pop("analysis")
                assert result == baseline
                assert answer == {"scope": "smiles_record", "record_id": record, "status": "unsupported_source"}
            for selector, reason in [
                ({"smiles_record_id": 0}, "invalid_params"),
                ({"smiles_record_id": len(cases) + len(unavailable) + 1}, "not_found"),
                ({"sdf_record_id": 1}, "unsupported_content"),
                ({"sdf_record_id": 1, "smiles_record_id": 1}, "invalid_params"),
                ({"smiles_record_id": 1, "expected_revision": 0}, "stale_revision"),
                ({"smiles_record_id": 1, "expected_selection_epoch": 1}, "stale_selection"),
            ]:
                assert run(cli, [opened, query(**selector)], success=False)["error"]["code"] == reason
            # A layout projection keeps a file backing, but its active owner is a drawing.
            project = ("document.import_smiles_records", {
                "expected_revision": 1, "selection": {"mode": "record", "record_id": 7},
                "bond_length_pt": 48, "origin_x_pt": 72, "origin_y_pt": 72,
                "page_width_pt": 595, "page_height_pt": 842,
            })
            drawing = run(cli, [opened, project])
            assert drawing["source"]["format"] == "smiles" and drawing["document"]["pages"][0]["fragments"]
            denied = run(cli, [opened, project, query(smiles_record_id=1, expected_revision=drawing["revision"])], success=False)
            assert denied["error"]["code"] == "unsupported_content"
            if path == source:
                run(cli, [opened, ("document.save_smiles", {"path": str(saved)})])
            assert source.read_bytes() == saved.read_bytes() == raw
    print("7 molecular graphs matched RDKit on original and reopened bytes; Na/Cl percentages use CIAAW weights")


if __name__ == "__main__":
    main(sys.argv[1])
