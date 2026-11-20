"""Exercise actual V2000/V3000 open, scope, Save As and reopened elemental weights."""
import json
from pathlib import Path
import subprocess
import sys
import tempfile

from rdkit import Chem
from analysis_elemental import check


def run(cli, commands):
    args = [cli, "run"]
    for name, params in commands:
        args.extend(["--cmd", json.dumps({"command": name, "params": params})])
    return json.loads(subprocess.check_output(args))


def verify(cli, path, original, selected=None):
    ops = [("document.open_mol", {"path": str(path)})]
    if selected is not None:
        ops.append(("ui.selection", {"action": "set", "expected_revision": 1, "page_id": 1,
                                     "atom_ids": [i+1 for i in selected]}))
    ops.append(("analysis.properties", {"expected_revision": 1, "expected_selection_epoch": int(selected is not None),
                                        "include_elemental_analysis": True}))
    result = run(cli, ops)
    report = result["analysis"]
    assert report["status"] == "available", (path.name, report)
    assert report["partial_fragments"] == (selected is not None)
    assert not result["file"]["dirty"] and result["undo_depth"] == result["redo_depth"] == 0
    check(original, report, selected)
    return report


def main(cli):
    with tempfile.TemporaryDirectory(prefix="bondline-elemental-") as tmp:
        directory = Path(tmp)
        for name, notation in [("ethanol", "CCO"), ("numbered-carbon", "[13CH3]CO"), ("ammonium", "[NH4+]")]:
            original = Chem.MolFromSmiles(notation)
            for version in (2000, 3000):
                path = directory / f"{name}-{version}.mol"
                source = Chem.MolToMolBlock(original, forceV3000=version == 3000).encode("ascii")
                path.write_bytes(source)
                full = verify(cli, path, original)
                if name == "ethanol":
                    verify(cli, path, original, {1, 2})
                saved = directory / ("元素-" + path.name)
                run(cli, [("document.open_mol", {"path": str(path)}), ("document.save_mol", {"path": str(saved)})])
                assert saved.read_bytes() == source
                assert verify(cli, saved, original) == full


if __name__ == "__main__":
    main(sys.argv[1])
