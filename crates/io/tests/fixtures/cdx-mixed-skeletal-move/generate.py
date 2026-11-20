# SPDX-License-Identifier: MIT OR Apache-2.0
"""Expected binary edits from source owner paths and independently moved XML coordinates."""

import argparse
from copy import deepcopy
from decimal import Decimal
from fractions import Fraction
from pathlib import Path
import runpy
import struct
from xml.etree import ElementTree as ET

HERE = Path(__file__).resolve().parent
FIX = HERE.parent
RECORDS = runpy.run_path(str(HERE.parents[4] / "xtask/oracles/cdx_oracle.py"))[
    "records"
]
CASES = [
    ("hidden", "ethanol", [(51, 73)], "8.25", "-5.125"),
    ("printed", "methanol", [(51, 97)], "-5.125", "3.25"),
    ("combined", "amino_alcohol", [(51, 73), (51, 109), (51, 149)], "8.25", "-5.125"),
    ("scoped", "twins", [(51, 97), (62, 73)], "-5.125", "3.25"),
]


def geometry(raw):
    records = RECORDS(raw)
    for r in records:
        if r["kind"] != "prop" or r["tag"] not in (0x0200, 0x0204):
            continue
        owner = records[r["parent"]]
        if r["tag"] == 0x0204:
            assert owner["tag"] == 0x8006
            owner = records[owner["parent"]]
        frag = records[owner["parent"]]
        assert owner["tag"] == 0x8004 and frag["tag"] == 0x8003
        yield (frag["id"], owner["id"]), r


def move(raw, owners, dx, dy):
    result = bytearray(raw)
    changed = set()
    delta = [round(Fraction(dy) * 65536), round(Fraction(dx) * 65536)]
    for key, r in geometry(raw):
        if key not in owners:
            continue
        changed.add(key)
        words = range(r["payload_start"], r["payload_end"], 4)
        assert len(words) == {0x0200: 2, 0x0204: 4}[r["tag"]]
        for i, at in enumerate(words):
            (old,) = struct.unpack_from("<i", raw, at)
            struct.pack_into("<i", result, at, old + delta[i % 2])
    assert changed == set(owners)
    return bytes(result)


def twins(raw):
    # Place a child Text before its Node position and force long size encodings.
    pos = next(r for k, r in geometry(raw) if k == (51, 97) and r["tag"] == 0x0200)
    records = RECORDS(raw)
    end = next(r["start"] for r in records if r.get("matched") == pos["parent"])
    raw = (
        raw[: pos["start"]]
        + raw[pos["end"] : end]
        + raw[pos["start"] : pos["end"]]
        + raw[end:]
    )
    for key, r in reversed(list(geometry(raw))):
        if r["encoding"] == "short":
            size = r["end"] - r["payload_start"]
            raw = (
                raw[: r["start"] + 2]
                + struct.pack("<HI", 0xFFFF, size)
                + raw[r["payload_start"] :]
            )
    shifted = move(raw, [(51, 73), (51, 97)], "32", "144")
    records = RECORDS(raw)
    i, frag = next((i, r) for i, r in enumerate(records) if r.get("tag") == 0x8003)
    end = next(r["end"] for r in records if r.get("matched") == i)
    peer = bytearray(shifted[frag["start"] : end])
    struct.pack_into("<I", peer, 2, 62)
    return raw[:end] + peer + raw[end:]


def shift_xml(frag, selected, dx, dy):
    vector = [Decimal(dx), Decimal(dy)]
    for n in frag.findall("n"):
        if int(n.get("id")) not in selected:
            continue
        for owner, field in [(n, "p"), (n.find("t"), "BoundingBox")]:
            if owner is not None:
                owner.set(
                    field,
                    " ".join(
                        str(Decimal(v) + vector[i % 2])
                        for i, v in enumerate(owner.get(field).split())
                    ),
                )


def xml(name, owners, dx, dy):
    path = (
        FIX / "cdx-mixed-skeleton/ethanol.cdxml"
        if name == "ethanol"
        else FIX
        / "mixed-labels"
        / (("methanol" if name == "twins" else name) + ".cdxml")
    )
    root = ET.parse(path).getroot()
    page = root.find("page")
    frag = page.find("fragment")
    if name == "twins":
        peer = deepcopy(frag)
        peer.set("id", "62")
        shift_xml(peer, {73, 97}, "32", "144")
        page.append(peer)
    for f in page.findall("fragment"):
        shift_xml(f, {n for k, n in owners if int(f.get("id")) == k}, dx, dy)
        # CDXML identifiers are globally unique, binary identities can be scoped.
        if f.get("id") == "62":
            for item in f.iter():
                for key in ("id", "B", "E"):
                    if item.get(key):
                        item.set(key, str(1000 + int(item.get(key))))
    return ET.tostring(root, encoding="utf-8")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--output-dir", type=Path)
    dest = parser.parse_args().output_dir
    if dest:
        dest.mkdir(parents=True, exist_ok=True)
    sources = {
        name: (FIX / "cdx-mixed-skeleton" / (name + ".cdx")).read_bytes()
        for name in ("methanol", "ethanol", "amino_alcohol")
    }
    sources["twins"] = twins(sources["methanol"])
    files = {"twins.cdx": sources["twins"]}
    for case, name, selected, dx, dy in CASES:
        files[case + ".cdx"] = move(sources[name], selected, dx, dy)
        files[case + ".cdxml"] = xml(name, selected, dx, dy)
    for name, data in files.items():
        p = (dest or HERE) / name
        if dest:
            p.write_bytes(data)
        else:
            assert p.read_bytes() == data, p
    print(f"{len(files)} exact binary and XML controls")


if __name__ == "__main__":
    main()
