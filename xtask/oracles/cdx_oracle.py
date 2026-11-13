#!/usr/bin/env python3
"""Original CDX fixtures and independent struct/external-RDKit test oracles.

Required tests use the repository's pinned external RDKit interpreter, without network.
No downloaded source, binary, or fixture is included in Bondline's shipped dependency graph.
"""

import hashlib
import struct
import sys

LIMIT = 8 * 1024 * 1024
HEADER = b"VjCD0100\x04\x03\x02\x01" + bytes(16)


def property_bytes(tag, payload, force_extended=False):
    length = len(payload)
    size = struct.pack("<HI", 0xFFFF, length) if force_extended or length >= 65535 else struct.pack("<H", length)
    return struct.pack("<H", tag) + size + payload


def object_bytes(tag, identifier, *children):
    return struct.pack("<HI", tag, identifier) + b"".join(children) + b"\0\0"


def fixture(order=1, explicit=False):
    """Original author-owned scene, not the SDK example or a downloaded sample.

    Numeric facts: public IUPAC FAIRSpec SDK IntroCDX.htm, Page.htm, Fragment.htm,
    Node.htm, Bond.htm, properties/{2DPosition,Node_Element,Bond_Order,Bond_Begin,
    Bond_End}.htm and DataType/CDXCoordinates.htm. Header-implied document root;
    coordinates are signed 16.16 points, stored y then x. Bond order is a bit mask.
    Unknown/user-defined tags carry original opaque bytes at distinct owners.
    """
    atom1 = object_bytes(0x8004, 101,
        property_bytes(0x0200, struct.pack("<ii", -229376, 671744)),
        property_bytes(0x0402, struct.pack("<H", 6)),
        property_bytes(0x4003, b"atom\0\xff"))
    atom2 = object_bytes(0x8004, 102,
        property_bytes(0x0200, struct.pack("<ii", 557056, 1720320)),
        property_bytes(0x0402, struct.pack("<H", 6)))
    bond = object_bytes(0x8005, 103,
        property_bytes(0x0604, struct.pack("<I", 101)),
        property_bytes(0x0605, struct.pack("<I", 102)),
        property_bytes(0x0600, struct.pack("<H", order)),
        property_bytes(0x4004, b"bond\0\xfe"))
    fragment = object_bytes(0x8003, 100, atom1, atom2, bond)
    unknown = object_bytes(0xC123, 77,
        property_bytes(0x4005, b"user object\0\xfd"), object_bytes(0xC124, 78))
    page = object_bytes(0x8001, 42,
        property_bytes(0x4001, b"page\0\xfc"), fragment, unknown)
    opaque = bytes((index * 37 + 11) % 256 for index in range(65535))
    body = property_bytes(0x4000, b"root\0\xfb") + property_bytes(0x4002, opaque) + page
    return HEADER + (object_bytes(0x8000, 0, body) if explicit else body + b"\0\0")


def records(source):
    """Iterative standard-library decoder, independent of every Bondline crate."""
    if len(source) > LIMIT or len(source) < 30 or source[:12] != HEADER[:12]:
        raise ValueError("invalid header/size")
    offset, opened, result = 28, [], []
    explicit = struct.unpack_from("<H", source, offset)[0] == 0x8000
    while offset < len(source):
        if len(result) >= 100000 or offset + 2 > len(source):
            raise ValueError("record limit/truncated tag")
        start = offset
        tag = struct.unpack_from("<H", source, offset)[0]
        offset += 2
        if tag == 0:
            matched = opened.pop() if opened else None
            result.append({"kind": "end", "matched": matched,
                "parent": opened[-1] if opened else None, "start": start, "end": offset})
            if not opened and (explicit or matched is None):
                if offset != len(source):
                    raise ValueError("trailing bytes")
                return result
        elif tag & 0x8000:
            if offset + 4 > len(source) or len(opened) >= 256:
                raise ValueError("object depth/truncated id")
            identifier = struct.unpack_from("<I", source, offset)[0]
            offset += 4
            result.append({"kind": "start", "tag": tag, "id": identifier,
                "parent": opened[-1] if opened else None, "start": start, "end": offset})
            opened.append(len(result) - 1)
        else:
            if offset + 2 > len(source):
                raise ValueError("truncated short length")
            length = struct.unpack_from("<H", source, offset)[0]
            offset += 2
            encoding = "short"
            if length == 65535:
                if offset + 4 > len(source):
                    raise ValueError("truncated extended length")
                length = struct.unpack_from("<I", source, offset)[0]
                offset += 4
                encoding = "extended"
            payload_start = offset
            offset += length
            if offset > len(source):
                raise ValueError("truncated payload")
            result.append({"kind": "prop", "tag": tag, "encoding": encoding,
                "payload_start": payload_start, "payload_end": offset,
                "parent": opened[-1] if opened else None, "start": start, "end": offset})
    raise ValueError("missing document terminator")


def describe(source):
    lines = []
    for record in records(source):
        parent = "-" if record["parent"] is None else record["parent"]
        fields = [record["kind"]]
        if record["kind"] == "start":
            fields += [record["tag"], record["id"]]
        elif record["kind"] == "prop":
            fields += [record["tag"], record["encoding"], record["payload_end"] - record["payload_start"], record["payload_start"], record["payload_end"]]
        else:
            fields += ["-" if record["matched"] is None else record["matched"]]
        lines.append(" ".join(map(str, fields + [parent, record["start"], record["end"]])))
    return "\n".join(lines) + "\n"


def verify_rdkit(source, order, explicit=False):
    from rdkit import Chem, rdBase
    from rdkit.Chem import rdMolDescriptors

    assert rdBase.rdkitVersion == "2026.09.2", "use the pinned BONDLINE_PYTHON interpreter"
    assert Chem.HasChemDrawCDXSupport(), "external RDKit build lacks binary CDX support"
    assert source == fixture(order, explicit), "unexpected bytes outside intended original fixture/property edit"
    parsed = records(source)
    coordinates = [struct.unpack_from("<ii", source, record["payload_start"]) for record in parsed if record.get("tag") == 0x0200]
    assert [(x / 65536, y / 65536) for y, x in coordinates] == [(10.25, -3.5), (26.25, 8.5)]
    parameters = Chem.CDXMLParserParams()
    parameters.format = Chem.CDXMLFormat.CDX
    molecules = Chem.MolsFromCDXML(source, parameters)
    assert len(molecules) == 1
    molecule = molecules[0]
    assert molecule.GetNumAtoms() == 2
    assert [atom.GetAtomicNum() for atom in molecule.GetAtoms()] == [6, 6]
    assert molecule.GetNumBonds() == 1
    bond = molecule.GetBondWithIdx(0)
    assert (bond.GetBeginAtomIdx(), bond.GetEndAtomIdx(), bond.GetBondTypeAsDouble()) == (0, 1, order)
    assert Chem.MolToSmiles(molecule) == ("CC" if order == 1 else "C=C")
    assert rdMolDescriptors.CalcMolFormula(molecule) == ("C2H6" if order == 1 else "C2H4")
    assert molecule.GetNumConformers() == 1
    # Observed RDKit CDX reader retains raw 16.16 values and reverses y; it does
    # not return document points here. The independent struct check above does.
    assert [tuple(molecule.GetConformer().GetAtomPosition(index)) for index in range(2)] == [
        (671744.0, 229376.0, 0.0), (1720320.0, -557056.0, 0.0)]
    print("independent CDX RDKit connectivity/order/formula/SMILES and raw-coordinate mapping verified")


def verify_opaque_edit(source, length):
    original = fixture()
    target = next(record for record in records(original) if record.get("tag") == 0x4002)
    expected = original[:target["start"]] + property_bytes(0x4002, b"\xa7" * length) + original[target["end"]:]
    assert source == expected, "changed header, ownership, order, opaque bytes, or unrelated record"
    edited = next(record for record in records(source) if record.get("tag") == 0x4002)
    assert edited["encoding"] == ("short" if length < 65535 else "extended")
    assert edited["parent"] is None
    print("independent CDX length/ownership/full-byte edit verified")


def self_test():
    hashes = {
        (1, False): "6e3228ecc2791366c1f23be8e4dd411f7f1fb4c3250b239fdfb812aa698df1f0",
        (2, False): "88b2ac7f0a4028323224647d16eadafca4c022d043010e5da94c46b2ad28a1f1",
        (1, True): "63ef1a74133e45de915d5fd25b2ca3321a95dbe5114873a5a59f2f45217000d2",
        (2, True): "1a183a24e21619084c16c4a39c7a803ad1c73dd3e6ac067dd355cd9122fd4774",
    }
    for (order, explicit), expected in hashes.items():
        assert hashlib.sha256(fixture(order, explicit)).hexdigest() == expected, "original fixture identity changed"
    source = fixture()
    result = records(source)
    assert len([record for record in result if record["kind"] == "start"]) == 7
    assert len([record for record in result if record["kind"] == "end"]) == 8
    assert source[28:30] == struct.pack("<H", 0x4000)
    for invalid in [b"", HEADER, source[:-1], source[:-2], source + b"\0\0",
            HEADER + struct.pack("<HHI", 0x4001, 65535, 0xFFFFFFFF) + b"\0\0",
            HEADER + struct.pack("<H", 0x8001), HEADER + struct.pack("<HH", 0x4001, 1)]:
        try:
            records(invalid)
        except ValueError:
            pass
        else:
            raise AssertionError("accepted malformed self-test vector")
    assert records(HEADER + property_bytes(0x4001, b"x", True) + b"\0\0")[0]["encoding"] == "extended"
    print("independent CDX oracle self-test passed")


def main():
    command = sys.argv[1] if len(sys.argv) > 1 else "self-test"
    if command == "fixture":
        sys.stdout.buffer.write(fixture(explicit=len(sys.argv) > 2 and sys.argv[2] == "explicit"))
    elif command == "self-test":
        self_test()
    else:
        source = sys.stdin.buffer.read(LIMIT + 1)
        if command == "describe":
            print(describe(source), end="")
        elif command == "verify":
            verify_rdkit(source, int(sys.argv[2]), explicit=len(sys.argv) > 3 and sys.argv[3] == "explicit")
        elif command == "verify-opaque-edit":
            verify_opaque_edit(source, int(sys.argv[2]))
        else:
            raise ValueError("unknown oracle command")


if __name__ == "__main__":
    main()
