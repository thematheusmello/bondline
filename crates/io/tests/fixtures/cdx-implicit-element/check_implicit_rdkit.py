# SPDX-License-Identifier: MIT OR Apache-2.0
"""Verify each implicit-Element fixture with external binary CDX readback."""

from copy import deepcopy
from hashlib import sha256
import json
from pathlib import Path
import sys

from rdkit import Chem, rdBase

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "cdx-label-edit"))
from check_rdkit import ORDERS, check

FORMULAS = {
    "cyanide_c_to_si": (["CHN"], ["HNSi"]),
    "boxed_c_h_only": (["CH2", "Cl"], ["CH3", "Cl"]),
    "boxed_c_to_noncarbon": (["CH2", "Cl"], ["H2N", "Cl"]),
}


def main():
    assert rdBase.rdkitVersion == "2026.09.2" and Chem.HasChemDrawCDXSupport()
    ORDERS["triple"] = Chem.BondType.TRIPLE
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

    retained = manifest["retained"]["cyanide_missing_h"]
    raw = (HERE / retained["file"]).read_bytes()
    assert sha256(raw).hexdigest() == retained["sha256"]
    model = deepcopy(retained["model"])
    carbon = model["pages"][0]["fragments"][0]["atoms"][1]
    assert carbon["explicit_hydrogens"] is None
    carbon["explicit_hydrogens"] = 0  # RDKit reports zero after reading this omitted field.
    assert check(raw, model, params) == ["CHN"]
    print("cyanide_missing_h  CHN, identical directed single and triple bonds")


if __name__ == "__main__":
    main()
