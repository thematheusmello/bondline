"""Original MIT OR Apache-2.0 external SDF consumer/writer, never shipped.

This deliberately does not scan raw spans or preserve duplicate properties. Published
CTfile literals and unchanged molecular/SDF oracles provide those separate checks.
"""
import hashlib
import io
import json
import sys
from rdkit import Chem, rdBase

MAX_BYTES = 64 * 1024 * 1024


def describe(source):
    assert 0 < len(source) <= MAX_BYTES
    assert all(byte in (10, 13) or 32 <= byte <= 126 for byte in source)
    lines = source.splitlines()
    assert len(lines) <= 1_000_000
    assert all(len(line) <= 4096 for line in lines)
    assert sum(line.startswith(b"$$$$") for line in lines) <= 1024
    supplier = Chem.ForwardSDMolSupplier(io.BytesIO(source), sanitize=False,
                                        removeHs=False, strictParsing=True)
    supplier.SetProcessPropertyLists(False)
    records = []
    for molecule in supplier:
        assert len(records) < 1024
        if molecule is None:
            records.append(None)
            continue
        assert molecule.GetNumAtoms() <= 100_000 and molecule.GetNumBonds() <= 100_000
        conformer = molecule.GetConformer()
        records.append({
            "atoms": [{"element": atom.GetAtomicNum(), "charge": atom.GetFormalCharge(),
                       "isotope": atom.GetIsotope(), "explicit_h": atom.GetNumExplicitHs(),
                       "map": atom.GetAtomMapNum(),
                       "position": list(conformer.GetAtomPosition(atom.GetIdx()))}
                      for atom in molecule.GetAtoms()],
            "bonds": [{"begin": bond.GetBeginAtomIdx(), "end": bond.GetEndAtomIdx(),
                       "order": bond.GetBondTypeAsDouble()} for bond in molecule.GetBonds()],
            "properties": {key: molecule.GetProp(key) for key in
                           molecule.GetPropNames(includePrivate=False, includeComputed=False)},
        })
    return {"rdkit": rdBase.rdkitVersion,
            "source_sha256": hashlib.sha256(source).hexdigest(), "records": records}


def emit():
    output = io.StringIO()
    writer = Chem.SDWriter(output)
    writer.SetProps(["EMPTY", "MULTI", "ORDINARY"])
    for index, smiles in enumerate(("CO", "CN")):
        molecule = Chem.MolFromSmiles(smiles)
        assert molecule is not None
        conformer = Chem.Conformer(molecule.GetNumAtoms())
        conformer.SetAtomPosition(0, (-1.25, 0.5, 0))
        conformer.SetAtomPosition(1, (0.25, 0.5, 0))
        molecule.AddConformer(conformer)
        molecule.SetProp("EMPTY", "")
        molecule.SetProp("MULTI", "one\ntwo")
        molecule.SetProp("ORDINARY", "v3" if index == 0 else "v2")
        writer.SetForceV3000(index == 0)
        writer.write(molecule)
    writer.flush()
    result = output.getvalue().encode("ascii")
    assert b" V3000\n" in result and b" V2000\n" in result
    return result


if __name__ == "__main__":
    assert rdBase.rdkitVersion == "2026.09.2"
    assert len(sys.argv) <= 2
    mode = sys.argv[1] if len(sys.argv) == 2 else "describe"
    if mode == "describe":
        source = sys.stdin.buffer.read(MAX_BYTES + 1)
        result = describe(source)
    elif mode == "emit":
        sys.stdout.buffer.write(emit())
        raise SystemExit(0)
    elif mode == "self-test":
        source = emit()
        decoded = describe(source)
        assert len(decoded["records"]) == 2
        assert decoded["records"][0]["atoms"][1]["element"] == 8
        assert decoded["records"][1]["atoms"][1]["element"] == 7
        assert all(record["properties"]["EMPTY"] == "" for record in decoded["records"])
        assert all(record["properties"]["MULTI"] == "one\ntwo" for record in decoded["records"])
        result = {"self_test": "passed", "source_bytes": len(source),
                  "source_sha256": hashlib.sha256(source).hexdigest()}
    else:
        raise AssertionError("describe, emit or self-test required")
    sys.stdout.buffer.write(json.dumps(result, sort_keys=True).encode("ascii") + b"\n")
