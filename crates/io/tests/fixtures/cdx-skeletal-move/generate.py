# SPDX-License-Identifier: MIT OR Apache-2.0
"""Predict exact fixed-width edits using an independent reader and rational arithmetic."""
from fractions import Fraction
from pathlib import Path
import runpy
import struct

HERE = Path(__file__).resolve().parent
FIX = HERE.parent
RECORDS = runpy.run_path(str(HERE.parents[4] / "xtask/oracles/cdx_oracle.py"))["records"]

def positions(data):
    rec = RECORDS(data)
    out = {}
    for r in rec:
        if r.get("tag") != 0x0200:
            continue
        node = rec[r["parent"]]
        fragment = rec[node["parent"]]
        assert node["tag"] == 0x8004 and fragment["tag"] == 0x8003
        assert r["payload_end"] - r["payload_start"] == 8
        owner = fragment["id"], node["id"]
        assert owner not in out
        out[owner] = r
    return out

def move(data, owners, dx, dy):
    result = bytearray(data)
    slots = positions(data)
    for owner in owners:
        start = slots[owner]["payload_start"]
        y, x = struct.unpack_from("<ii", data, start)
        struct.pack_into("<ii", result, start, y + round(Fraction(dy)*65536),
                         x + round(Fraction(dx)*65536))
    return bytes(result)

def main():
    for name in ("propane", "missing_h", "all_h_missing", "dormant_true", "dormant_nonterminal"):
        raw = (FIX / "cdx-skeletal" / (name + ".cdx")).read_bytes()
        moved = move(raw, [(901,7)], "8.25", "-5.125")
        (HERE / (name + "_center.cdx")).write_bytes(moved)
    src = (FIX / "cdx-skeletal/all_h_missing.cdx").read_bytes()
    central = positions(src)[901,7]
    # Preserve a long-form property header on the edited eight-byte field.
    raw = src[:central["start"]+2] + struct.pack("<HI",0xffff,8) + src[central["payload_start"]:]
    # A second fragment deliberately reuses every child node and bond source ID.
    records = RECORDS(raw)
    fi, root = next((i,r) for i,r in enumerate(records) if r.get("tag")==0x8003)
    end = next(r["end"] for r in records if r.get("matched")==fi)
    start = root["start"]
    peer = bytearray(raw[start:end])
    struct.pack_into("<I",peer,2,902)
    for slot in positions(raw).values():
        i=slot["payload_start"]-start
        y,x=struct.unpack_from("<ii",peer,i)
        struct.pack_into("<ii",peer,i,y+128*65536,x+32*65536)
    two=raw[:end]+peer+raw[end:]
    (HERE/"aliased_extended.cdx").write_bytes(two)
    (HERE/"aliased_moved.cdx").write_bytes(move(two,[(901,7),(902,81)],"-5.125","3.25"))
    # Opposite grid parity distinguishes rigid-delta rounding from rounding each endpoint.
    odd = move(two, [(902,81)], Fraction(1,65536), Fraction(1,65536))
    (HERE/"odd_position.cdx").write_bytes(odd)
    (HERE/"odd_position_halves.cdx").write_bytes(
        move(odd, [(901,7),(902,81)], Fraction(3,131072), Fraction(-3,131072)))


if __name__=="__main__":
    main()
