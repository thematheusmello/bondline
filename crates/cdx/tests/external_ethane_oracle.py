"""Original test-only raw/CDX-reader oracle for the externally authored ethane.

No Bondline decoder or old scene-specific oracle is imported. Raw framing here is
the declared canonical28-byte document-end/file-end profile, not a general reader.
"""

import json
import math
import struct
import sys

from rdkit import Chem, rdBase
from rdkit.Chem import rdMolDescriptors

assert rdBase.rdkitVersion == "2026.09.2", "use literal pinned BONDLINE_PYTHON"
HEADER = bytes.fromhex("566a4344303130300403020100000000000000000000000000000000")
EXPECTED = [
    ("prop", 2053, None, 28, 36, 32, 36, "00001e00"),
    ("prop", 2074, None, 36, 42, 40, 42, "0300"),
    ("prop", 2075, None, 42, 48, 46, 48, "0400"),
    ("prop", 256, None, 48, 88, 52, 88, "000002000200e9fd0500417269616c0300e9fd0f0054696d6573204e657720526f6d616e"),
    ("prop", 768, None, 88, 142, 92, 142, "0800ffffffffffff000000000000ffff00000000ffffffff00000000ffff00000000ffffffff00000000ffffffff0000ffff"),
    ("start", 32769, 0, None, 142, 148),
    ("prop", 2064, 5, 148, 154, 152, 154, "0100"),
    ("prop", 2063, 5, 154, 160, 158, 160, "0100"),
    ("prop", 516, 5, 160, 180, 164, 180, "00000000000000000000000000002d00"),
    ("start", 32771, 4, 5, 180, 186),
    ("start", 32772, 5, 9, 186, 192),
    ("prop", 512, 10, 192, 204, 196, 204, "0000000000000000"),
    ("end", 10, 9, 204, 206),
    ("start", 32772, 6, 9, 206, 212),
    ("prop", 512, 13, 212, 224, 216, 224, "0000000000002d00"),
    ("end", 13, 9, 224, 226),
    ("start", 32773, 7, 9, 226, 232),
    ("prop", 1540, 16, 232, 240, 236, 240, "05000000"),
    ("prop", 1541, 16, 240, 248, 244, 248, "06000000"),
    ("end", 16, 9, 248, 250),
    ("end", 9, 5, 250, 252),
    ("end", 5, None, 252, 254),
    ("end", None, None, 254, 256),
    ("file_end", None, 256, 258),
]


def records(raw):
    assert len(raw) <= 1_048_576 and raw[:28] == HEADER, "canonical28-byte envelope"
    cursor, stack, result = 28, [], []
    explicit = raw[28:30] == b"\x00\x80"
    ended = False
    while not ended:
        assert cursor + 2 <= len(raw), "missing document end"
        start = cursor
        tag = struct.unpack_from("<H", raw, cursor)[0]
        cursor += 2
        parent = stack[-1] if stack else None
        if tag == 0:
            owner = stack.pop() if stack else None
            result.append(("end", owner, stack[-1] if stack else None, start, cursor))
            ended = not stack if explicit else owner is None
        elif tag & 0x8000:
            assert cursor + 4 <= len(raw), "truncated object id"
            identity = struct.unpack_from("<I", raw, cursor)[0]
            cursor += 4
            result.append(("start", tag, identity, parent, start, cursor))
            stack.append(len(result) - 1)
        else:
            assert cursor + 2 <= len(raw), "truncated property length"
            length = struct.unpack_from("<H", raw, cursor)[0]
            cursor += 2
            if length == 0xFFFF:
                assert cursor + 4 <= len(raw), "truncated extended length"
                length = struct.unpack_from("<I", raw, cursor)[0]
                cursor += 4
            end = cursor + length
            assert end <= len(raw), "truncated property payload"
            result.append(("prop", tag, parent, start, end, cursor, end, raw[cursor:end].hex()))
            cursor = end
        assert len(result) <= 4096, "bounded test oracle records"
    assert not stack and raw[cursor:cursor + 2] == b"\0\0", "one separate file end"
    result.append(("file_end", None, cursor, cursor + 2))
    assert cursor + 2 == len(raw), "exact file-end EOF"
    return result


def raw_ethane(raw):
    assert records(raw) == EXPECTED, "literal full record/payload/owner table"
    positions = [struct.unpack_from("<ii", raw, start) for start in (196, 216)]
    assert positions == [(0, 0), (0, 2_949_120)], "literal y/x16.16 words"
    assert [tuple(value / 65_536 for value in (x, y)) for y, x in positions] == [(0, 0), (45, 0)]
    return positions


def chemistry(raw):
    parameters = Chem.CDXMLParserParams()
    parameters.format = Chem.CDXMLFormat.CDX
    parameters.sanitize = True
    parameters.removeHs = False
    molecules = Chem.MolsFromCDXML(raw, parameters)
    assert len(molecules) == 1, "one actual external molecule"
    molecule = molecules[0]
    assert molecule.GetNumAtoms() == 2 and molecule.GetNumBonds() == 1
    assert [atom.GetAtomicNum() for atom in molecule.GetAtoms()] == [6, 6]
    assert [atom.GetTotalNumHs() for atom in molecule.GetAtoms()] == [3, 3]
    assert all(atom.GetFormalCharge() == atom.GetIsotope() == atom.GetNumRadicalElectrons() == 0
               and atom.GetChiralTag() == Chem.ChiralType.CHI_UNSPECIFIED
               for atom in molecule.GetAtoms())
    assert [(bond.GetBeginAtomIdx(), bond.GetEndAtomIdx(), bond.GetBondTypeAsDouble())
            for bond in molecule.GetBonds()] == [(0, 1, 1.0)]
    assert all(bond.GetStereo() == Chem.BondStereo.STEREONONE for bond in molecule.GetBonds())
    assert Chem.MolToSmiles(molecule, isomericSmiles=True) == "CC"
    assert rdMolDescriptors.CalcMolFormula(molecule) == "C2H6"
    assert molecule.GetNumConformers() == 1
    points = [list(molecule.GetConformer().GetAtomPosition(i)) for i in range(2)]
    assert all(math.isfinite(value) for point in points for value in point)
    assert points == [[0.0, 0.0, 0.0], [2.25, 0.0, 0.0]], "pinned consumer coordinates, not an input-unit claim"
    return {"atoms": 2, "bonds": 1, "formula": "C2H6", "smiles": "CC", "hydrogens": [3, 3]}


def verify(raw):
    positions = raw_ethane(raw)
    result = chemistry(raw)
    result.update(byte_length=len(raw), record_count=len(EXPECTED), header_bytes=28,
                  document_end=[254, 256], file_end=[256, 258], raw_y_x16_16=positions)
    return result


def describe(raw):
    def index(value):
        return "-" if value is None else str(value)

    lines = []
    for row in records(raw):
        if row[0] == "start":
            _, tag, identity, parent, start, end = row
            lines.append(f"start {tag} {identity} {index(parent)} {start} {end}")
        elif row[0] == "prop":
            _, tag, parent, start, end, payload_start, payload_end, payload_hex = row
            encoding = "extended" if raw[start + 2:start + 4] == b"\xff\xff" else "short"
            lines.append(f"prop {tag} {encoding} {payload_end - payload_start} {payload_start} {payload_end} {index(parent)} {start} {end} {payload_hex}")
        elif row[0] == "end":
            _, owner, parent, start, end = row
            lines.append(f"end {index(owner)} {index(parent)} {start} {end}")
        else:
            _, parent, start, end = row
            lines.append(f"file_end {index(parent)} {start} {end}")
    return ("\n".join(lines) + "\n").encode("ascii")


if __name__ == "__main__":
    assert len(sys.argv) in (1, 2)
    source = sys.stdin.buffer.read(1_048_577)
    if len(sys.argv) == 2:
        assert sys.argv[1] == "describe"
        sys.stdout.buffer.write(describe(source))
    else:
        sys.stdout.buffer.write((json.dumps(verify(source), separators=(",", ":")) + "\n").encode("ascii"))
