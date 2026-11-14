"""Original metadata field vectors; independent struct and external RDKit oracles."""

import hashlib
import json
from pathlib import Path
import struct
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[3] / "xtask/oracles"))
import cdx_oracle as legacy
import cdx_header_oracle as header


def prop(tag, data):
    return struct.pack("<HH", tag, len(data)) + data


def obj(tag, identifier, *children):
    return struct.pack("<HI", tag, identifier) + b"".join(children) + bytes(2)


def fixture(framing):
    """Author-owned values; no native metadata, font, style library or warning copied."""
    bounds = struct.pack("<iiii", -524288, 262144, 786432, 2097152)
    first = obj(0x8004, 101,
        prop(0x0200, struct.pack("<ii", -229376, 671744)),
        prop(0x000A, struct.pack("<h", -2)), prop(0x0437, b"\x01"),
        prop(0x044D, struct.pack("<I", 70001)), prop(0x4003, b"node opaque\0\xff"))
    second = obj(0x8004, 102,
        prop(0x0200, struct.pack("<ii", 557056, 1720320)),
        prop(0x000A, struct.pack("<h", 5)), prop(0x0437, b"\x01"),
        prop(0x044D, struct.pack("<I", 90005)))
    bond = obj(0x8005, 103, prop(0x000A, struct.pack("<h", 7)),
        prop(0x0010, b"\0\0Original metadata advisory"),
        prop(0x0604, struct.pack("<I", 101)), prop(0x0605, struct.pack("<I", 102)),
        prop(0x060A, b"\x01"), prop(0x4004, b"bond opaque\0\xfe"))
    fragment = obj(0x8003, 100, prop(0x0204, bounds),
        prop(0x000A, struct.pack("<h", 12)), first, second, bond)
    custom = obj(0xC123, 77, prop(0x4005, b"custom opaque\0\xfd"))
    page = obj(0x8001, 42, prop(0x0204, bounds), fragment, custom)
    document = obj(0x8000, 0, prop(0x4000, b"metadata root\0\xfb"), page)
    if framing == "explicit":
        return legacy.HEADER + document
    if framing == "header":
        return legacy.HEADER[:22] + document + bytes(2)
    raise ValueError("unknown original framing")


def verify(source, framing):
    from rdkit import Chem, rdBase
    from rdkit.Chem import rdMolDescriptors

    assert source == fixture(framing), "original metadata fixture changed"
    table = (legacy.records if framing == "explicit" else header.records)(source)
    fields = {}
    for row in table:
        if row["kind"] != "prop":
            continue
        owner = table[row["parent"]]
        key = (owner["tag"], owner["id"], row["tag"])
        assert key not in fields
        fields[key] = source[row["payload_start"]:row["payload_end"]]
    expected = {
        (0x8003, 100, 0x0204): ("<iiii", (-524288, 262144, 786432, 2097152)),
        (0x8003, 100, 0x000A): ("<h", (12,)),
        (0x8004, 101, 0x0200): ("<ii", (-229376, 671744)),
        (0x8004, 102, 0x0200): ("<ii", (557056, 1720320)),
        (0x8004, 101, 0x000A): ("<h", (-2,)),
        (0x8004, 102, 0x000A): ("<h", (5,)),
        (0x8004, 101, 0x0437): ("<b", (1,)),
        (0x8004, 102, 0x0437): ("<b", (1,)),
        (0x8004, 101, 0x044D): ("<I", (70001,)),
        (0x8004, 102, 0x044D): ("<I", (90005,)),
        (0x8005, 103, 0x000A): ("<h", (7,)),
        (0x8005, 103, 0x0604): ("<I", (101,)),
        (0x8005, 103, 0x0605): ("<I", (102,)),
        (0x8005, 103, 0x060A): ("<b", (1,)),
    }
    for key, (fmt, values) in expected.items():
        assert len(fields[key]) == struct.calcsize(fmt)
        assert struct.unpack(fmt, fields[key]) == values
    assert fields[(0x8005, 103, 0x0010)] == b"\0\0Original metadata advisory"
    assert fields[(0x8004, 101, 0x4003)] == b"node opaque\0\xff"
    assert fields[(0x8005, 103, 0x4004)] == b"bond opaque\0\xfe"
    assert fields[(0xC123, 77, 0x4005)] == b"custom opaque\0\xfd"
    assert rdBase.rdkitVersion == "2026.09.2", "use pinned BONDLINE_PYTHON"
    assert Chem.HasChemDrawCDXSupport()
    params = Chem.CDXMLParserParams()
    params.format = Chem.CDXMLFormat.CDX
    molecules = Chem.MolsFromCDXML(source, params)
    assert len(molecules) == 1
    molecule = molecules[0]
    assert [atom.GetAtomicNum() for atom in molecule.GetAtoms()] == [6, 6]
    assert molecule.GetNumBonds() == 1
    bond = molecule.GetBondWithIdx(0)
    assert (bond.GetBeginAtomIdx(), bond.GetEndAtomIdx(), bond.GetBondTypeAsDouble()) == (0, 1, 1)
    assert Chem.MolToSmiles(molecule) == "CC"
    assert rdMolDescriptors.CalcMolFormula(molecule) == "C2H6"
    print(json.dumps({"bytes": len(source), "sha256": hashlib.sha256(source).hexdigest(),
                      "external_smiles": "CC", "atoms": 2, "bonds": 1}, ensure_ascii=True))


if __name__ == "__main__":
    mode = sys.argv[1]
    framing = sys.argv[2]
    if mode == "fixture":
        sys.stdout.buffer.write(fixture(framing))
    elif mode == "verify":
        verify(sys.stdin.buffer.read(legacy.LIMIT + 1), framing)
    else:
        raise ValueError("unknown oracle operation")
