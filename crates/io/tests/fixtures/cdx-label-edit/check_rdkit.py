# SPDX-License-Identifier: MIT OR Apache-2.0
"""Check the original before/after CDX graphs with an external binary CDX reader."""

from hashlib import sha256
import json
from pathlib import Path

from rdkit import Chem, rdBase
from rdkit.Chem import rdMolDescriptors

HERE = Path(__file__).resolve().parent
ORDERS = {"single": Chem.BondType.SINGLE}
FORMULAS = {
    "water_to_nh": (["H2O"], ["H3N"]),
    "boxed_nh2_to_oh": (["H2N", "Cl"], ["HO", "Cl"]),
    "boxed_cl_to_br": (["H2N", "Cl"], ["H2N", "Br"]),
}


def check(raw, model, params):
    molecules = Chem.MolsFromCDXML(raw, params)
    formulas = []
    for mol, fragment in zip(molecules, model["pages"][0]["fragments"], strict=True):
        mol.UpdatePropertyCache(strict=False)
        assert [(a.GetAtomicNum(), a.GetNumExplicitHs()) for a in mol.GetAtoms()] == [
            (a["element"], a["explicit_hydrogens"]) for a in fragment["atoms"]
        ]
        atom_indices = {atom["id"]: i for i, atom in enumerate(fragment["atoms"])}
        bonds = [(b.GetBeginAtomIdx(), b.GetEndAtomIdx(), b.GetBondType()) for b in mol.GetBonds()]
        assert bonds == [(atom_indices[b["begin"]], atom_indices[b["end"]], ORDERS[b["order"]])
                         for b in fragment["bonds"]]
        formulas.append(rdMolDescriptors.CalcMolFormula(mol))
    return formulas


def main():
    assert rdBase.rdkitVersion == "2026.09.2" and Chem.HasChemDrawCDXSupport()
    params = Chem.CDXMLParserParams()
    params.format = Chem.CDXMLFormat.CDX
    params.sanitize = False
    params.removeHs = False
    manifest = json.loads((HERE / "cases.json").read_bytes())
    assert manifest["cases"].keys() == FORMULAS.keys()
    for name, case in manifest["cases"].items():
        for side, expected in zip(("before", "after"), FORMULAS[name], strict=True):
            raw = (HERE / case[side]).read_bytes()
            assert sha256(raw).hexdigest() == case["source"][side + "_sha256"]
            assert check(raw, case[side + "_model"], params) == expected, (name, side)
        before, after = FORMULAS[name]
        print(f"{name}  {', '.join(before)} -> {', '.join(after)}")


if __name__ == "__main__":
    main()
