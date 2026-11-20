# SPDX-License-Identifier: MIT OR Apache-2.0
"""Compare individual V2000/V3000 records in the actual Bondline CLI and RDKit."""

import io
from pathlib import Path
import sys
import tempfile

from rdkit import Chem, rdBase
from rdkit.Chem import rdMolDescriptors

from analysis_elemental import check
from elemental_mol import run


def main(cli):
    assert rdBase.rdkitVersion == "2026.09.2"
    fixtures = Path(__file__).resolve().parents[2] / "crates/io/tests/fixtures"
    original = (fixtures / "sdf-preview/valence-absent-mixed.sdf").read_bytes()
    # This data field is deliberately unrelated to the connection table.
    original = original.replace(b"> <NOTE>", b"> <ReportedFormula>\nXeF8\n\n> <NOTE>")
    for smiles, v3000 in [("C=C", True), ("[13CH4]", False), ("[NH4+]", True)]:
        molecule = Chem.MolFromSmiles(smiles)
        block = Chem.MolToMolBlock(molecule, forceV3000=v3000)
        original += block.encode("ascii") + b"> <ReportedFormula>\nXeF8\n\n$$$$\n"
    molecules = list(Chem.ForwardSDMolSupplier(io.BytesIO(original), removeHs=False))
    assert len(molecules) == 5 and all(m is not None for m in molecules)
    with tempfile.TemporaryDirectory(prefix="bondline-sdf-analysis-") as tmp:
        source = Path(tmp) / "mixture.sdf"
        source.write_bytes(original)
        for record, molecule in enumerate(molecules, 1):
            saved = Path(tmp) / f"原本-{record}-β.sdf"
            query = (
                "analysis.properties",
                {
                    "expected_revision": 1,
                    "expected_selection_epoch": 0,
                    "sdf_record_id": record,
                    "include_elemental_analysis": True,
                },
            )
            ops = [("document.open_sdf", {"path": str(source)}), query]
            result = run(cli, ops)
            report = result["analysis"]
            assert report["record_id"] == record and report["scope"] == "sdf_record"
            assert report["formula"] == rdMolDescriptors.CalcMolFormula(
                molecule, separateIsotopes=True, abbreviateHIsotopes=False
            )
            assert (
                abs(
                    report["masses"]["exact_mass_da"]
                    - rdMolDescriptors.CalcExactMolWt(molecule)
                )
                < 1e-6
            )
            check(molecule, report)
            assert not result["file"]["dirty"] and result["undo_depth"] == 0
            run(cli, [*ops, ("document.save_sdf", {"path": str(saved)})])
            assert saved.read_bytes() == source.read_bytes() == original
            assert (
                run(cli, [("document.open_sdf", {"path": str(saved)}), query])[
                    "analysis"
                ]
                == report
            )
            print(record, report["formula"], report["masses"]["exact_mass_da"])

        # Isolate a valid, oversized CTAB from its tiny neighboring molecule.
        header = original.split(b"M  V30 COUNTS", 1)[0]
        count = 35000
        atoms = b"".join(
            f"M  V30 {n} C {n}.125000000000000 0 0 0\n".encode()
            for n in range(1, count + 1)
        )
        large = (
            header
            + f"M  V30 COUNTS {count} 0 0 0 0\nM  V30 BEGIN ATOM\n".encode()
            + atoms
            + b"M  V30 END ATOM\nM  V30 END CTAB\nM  END\n"
        )
        assert len(large) > 1024 * 1024
        source.write_bytes(large + b"$$$$\n" + original)
        open_source = ("document.open_sdf", {"path": str(source)})
        params = {"expected_revision": 1, "expected_selection_epoch": 0}
        oversized = run(
            cli, [open_source, ("analysis.properties", {**params, "sdf_record_id": 1})]
        )
        assert oversized["analysis"] == {
            "scope": "sdf_record",
            "record_id": 1,
            "status": "record_byte_limit",
            "limit": 1024 * 1024,
        }
        neighbor = run(
            cli, [open_source, ("analysis.properties", {**params, "sdf_record_id": 2})]
        )
        assert neighbor["analysis"]["formula"] == "C2H6O"


if __name__ == "__main__":
    main(sys.argv[1])
