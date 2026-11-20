"""Check user-facing MOL Analysis and exact Save/reopen against independent RDKit reads."""
import argparse
import json
import re
from pathlib import Path
import subprocess
import tempfile

from rdkit import Chem, rdBase
from rdkit.Chem import Descriptors, rdMolDescriptors


def dispatch(cli, commands):
    args = [str(cli), "run"]
    for name, params in commands:
        args.extend(("--cmd", json.dumps({"command": name, "params": params})))
    return json.loads(subprocess.check_output(args))


def verify(cli, path, expected_formula, selected=None, mass_reference=None):
    operations = [("document.open_mol", {"path": str(path)}),
                  ("ui.analysis", {"expected_revision": 1, "visible": True})]
    if selected:
        operations.append(("ui.selection", {"action": "set", "expected_revision": 1,
                                            "page_id": 1, "atom_ids": [selected]}))
    query = ("analysis.formula", {"expected_revision": 1, "expected_selection_epoch": int(bool(selected))})
    result = dispatch(cli, operations + [query])
    analysis = result["analysis"]
    assert analysis["status"] == "available" and analysis["formula"] == expected_formula, (path.name, analysis, expected_formula)
    atoms = result["document"]["pages"][0]["fragments"][0]["atoms"]
    omitted = sum(a["explicit_hydrogens"] is None for a in atoms if not selected or a["id"] == selected)
    assert analysis.get("defaulted_hydrogen_atoms", 0) == omitted, (path.name, analysis, omitted)
    assert not result["file"]["dirty"] and result["undo_depth"] == result["redo_depth"] == 0
    if mass_reference is not None:
        detailed = dispatch(cli, operations + [("analysis.properties", query[1])])
        full_report = detailed["analysis"].copy()
        masses = full_report.pop("masses")
        assert full_report == analysis and detailed["document"] == result["document"]
        weight, exact = mass_reference
        # RDKit uses different editions for some standard weights. Fixed absolute
        # tolerances for these small molecules were set before the comparison.
        assert abs(masses["molecular_weight"] - weight) <= 0.010, (path, masses, weight)
        assert abs(masses["exact_mass_da"] - exact) <= 0.000001, (path, masses, exact)
    return omitted


def exercise(cli, directory, name, source, selected_oxygen=None):
    path = directory / (name + ".mol")
    path.write_bytes(source)
    molecule = Chem.MolFromMolBlock(source.decode("ascii"), sanitize=True, removeHs=False, strictParsing=True)
    assert molecule is not None, name
    expected = rdMolDescriptors.CalcMolFormula(molecule, separateIsotopes=True, abbreviateHIsotopes=False)
    # RDKit places H before B here. Carbon-free Hill order puts B before H.
    if name.startswith("borane-"):
        boron, = molecule.GetAtoms()
        assert expected == "H3B" and boron.GetSymbol() == "B" and boron.GetTotalNumHs() == 3
        expected = "BH3"
    mass_reference = Descriptors.MolWt(molecule), rdMolDescriptors.CalcExactMolWt(molecule)
    counts = verify(cli, path, expected, mass_reference=mass_reference)
    if selected_oxygen:
        atom = molecule.GetAtomWithIdx(selected_oxygen - 1)
        assert atom.GetSymbol() == "O" and atom.GetTotalNumHs() == 1
        verify(cli, path, "HO", selected_oxygen)
    destination = directory / ("保存-" + name + ".mol")
    dispatch(cli, [("document.open_mol", {"path": str(path)}),
                   ("document.save_mol", {"path": str(destination)})])
    assert destination.read_bytes() == path.read_bytes() == source
    assert verify(cli, destination, expected, mass_reference=mass_reference) == counts
    print(f"{name}: {expected}, {counts} omitted counts", flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("cli", type=Path)
    cli = parser.parse_args().cli.resolve()
    assert rdBase.rdkitVersion == "2026.09.2"
    fixtures = Path(__file__).resolve().parents[2] / "crates/io/tests/fixtures"
    sdf = (fixtures / "sdf-preview/valence-absent-mixed.sdf").read_bytes()
    with tempfile.TemporaryDirectory(prefix="bondline-mol-formula-") as tmp:
        directory = Path(tmp)
        for i, record in enumerate(sdf.split(b"$$$$\n")[:2]):
            exercise(cli, directory, ["ethanol-v3000", "methylamine-v2000"][i],
                     record[:record.index(b"M  END\n") + 7], 3 if i == 0 else None)
        for relative in ("mol/rdkit-CH3.mol", "v3000/noncontiguous-cc.mol"):
            fixture = fixtures / relative
            exercise(cli, directory, "corpus-" + fixture.stem, fixture.read_bytes())
        ordered = (fixtures / "sdf/ordered-fields-v2000.sdf").read_bytes()
        records = re.split(rb"(?m)^[$]{4}[^\r\n]*\r?\n", ordered)
        assert len(records) == 3 and not records[-1]
        for i, raw in enumerate(records[:-1]):
            end = raw.index(b"M  END") + len(b"M  END")
            eol = b"\r\n" if raw[end:].startswith(b"\r\n") else b"\n"
            exercise(cli, directory, f"corpus-sdf-record-{i+1}", raw[:end] + eol)
        for name, smiles in [("acetylene", "C#C"), ("carbonyl", "C=O"), ("cyclopropane", "C1CC1"),
                             ("silane", "[SiH4]"), ("thiol", "CS"), ("sulfoxide", "CS(=O)C"),
                             ("phosphine", "P"), ("borane", "B"), ("ammonium", "[NH4+]"),
                             ("hydroxide", "[OH-]"), ("chloride", "[Cl-]"),
                             ("isotope", "[13CH3]CO"), ("methyl-radical", "[CH3]"),
                             ("physical-hydrogen", "[2H]O[2H]")]:
            molecule = Chem.MolFromSmiles(smiles)
            assert molecule is not None
            for v3000 in (False, True):
                block = Chem.MolToMolBlock(molecule, forceV3000=v3000).encode("ascii")
                exercise(cli, directory, f"{name}-v{3000 if v3000 else 2000}", block)
        carbon = Chem.MolToMolBlock(Chem.MolFromSmiles("[CH3]"), forceV3000=True)
        assert "RAD=2" in carbon and "VAL=3" in carbon
        exercise(cli, directory, "unmarked-doublet-v3000", carbon.replace(" VAL=3", "").encode("ascii"))
        isolated = carbon.replace(" RAD=2", "").replace("VAL=3", "VAL=-1")
        exercise(cli, directory, "explicit-zero-carbon-v3000", isolated.encode("ascii"))
        coincident = sdf.split(b"M  END\n", 1)[0] + b"M  END\n"
        coincident = coincident.replace(b"0.25 0.5", b"-1.25 0.5").replace(b"0.25 2", b"-1.25 0.5")
        exercise(cli, directory, "coincident-ethanol-v3000", coincident)
        mixed = directory / "ethanol-and-iron.mol"
        mixed.write_text(Chem.MolToMolBlock(Chem.MolFromSmiles("CCO.[Fe]"), forceV3000=True))
        # A metal with no declared valence is ambiguous across CTfile conventions.
        # Construct the unmarked source even if a future RDKit writer emits VAL=-1.
        text = mixed.read_text()
        mixed.write_text("\n".join(line.split(" VAL=")[0] if " Fe " in line else line
                                   for line in text.split("\n")))
        report = dispatch(cli, [("document.open_mol", {"path": str(mixed)}),
                                ("analysis.formula", {"expected_revision": 1, "expected_selection_epoch": 0})])
        assert report["analysis"] == {"scope":"document","status":"unresolved_hydrogens","atom_id":4}, report["analysis"]
        assert verify(cli, mixed, "HO", 3) == 1


if __name__ == "__main__":
    main()
