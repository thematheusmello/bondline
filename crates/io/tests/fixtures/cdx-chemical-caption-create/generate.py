# SPDX-License-Identifier: MIT OR Apache-2.0
"""Build a direct Page Text at the exact matched Page end using an independent CDX reader."""
import argparse
from hashlib import sha256
from pathlib import Path
import runpy
import struct

HERE = Path(__file__).resolve().parent
# Reuse only the independent binary specification helpers from the caption-only oracle.
helpers = runpy.run_path(str(HERE.parent / "cdx-caption-create/generate.py"))
obj, prop, raw_records, children = (helpers[k] for k in ("obj", "prop", "raw_records", "children"))
PAGE, TEXT, MARKER = 0x8001, 0x8006, 0x8011
SOURCES = {
    "chemical": ("cdx-label-preview/boxed.cdx",
                 "b7d89d7d117b449e25cfcb61a9221f2a75d1cc7cf05b74073a3f921af162f2ee", 1, 1),
    "mixed": ("cdx-mixed-caption-edit/before.cdx",
              "b6be2d319bde7d41d8835a6fa4f6d5fde65cd568f52afd29b6447deca04ac469", 8, 20),
}


def vectors():
    for kind, (path, digest, depth, first_id) in SOURCES.items():
        baseline = (HERE.parent / path).read_bytes()
        assert sha256(baseline).hexdigest() == digest
        framing, header, old = raw_records(baseline)
        page, = (i for i, r in enumerate(old) if r.get("tag") == PAGE)
        assert old[page]["id"] == 51
        at, = (r["range"][0] for r in old if r.get("matched") == page)
        assert max(r.get("id", 0) for r in old) == 902
        aliases = [old[r["parent"]]["tag"] for r in old if r.get("tag") == TEXT and r["id"] == 73]
        assert aliases == [0x8004, 0x8004] + ([PAGE] if kind == "mixed" else [])
        for stage, live_id in (("first", first_id), ("after_undo", first_id + 1)):
            marker = {0x0008: b"org.bondline.caption-id", 0x0d00: struct.pack("<h", 3),
                      0x0d04: bytes([1]), 0x0d05: f"1:{live_id}".encode("ascii")}
            fields = {
                0x0200: struct.pack("<ii", 867 * 16384, 577 * 16384),  # y, x at signed 16.16
                0x080c: bytes(1),
                0x000a: struct.pack("<h", depth),
                0x0700: struct.pack("<6H", 1, 0, 0, 0, 240, 0) + "Phase β & q".encode("utf-8"),
            }
            child = obj(TEXT, 903, *(prop(k, v) for k, v in fields.items()),
                        obj(MARKER, 0, *(prop(k, v) for k, v in marker.items())))
            output = baseline[:at] + child + baseline[at:]
            mode, offset, records = raw_records(output)
            assert (mode, offset) == (framing, header)
            owner, = (i for i, r in enumerate(records) if r.get("id") == 903)
            annotation, = (i for i, r in enumerate(records)
                           if r.get("tag") == MARKER and r["parent"] == owner)
            assert records[owner]["parent"] == page and records[annotation]["id"] == 0
            assert children(output, records, owner) == fields and children(output, records, annotation) == marker
            end, = (r["range"][1] for r in records if r.get("matched") == owner)
            assert records[owner]["range"][0] == at and output[:at] + output[end:] == baseline
            yield f"{kind}_{stage}.cdx", output


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", type=Path)
    destination = parser.parse_args().output_dir
    for name, data in vectors():
        if destination is None:
            assert data == (HERE / name).read_bytes(), name
        else:
            destination.mkdir(exist_ok=True, parents=True)
            (destination / name).write_bytes(data)
        print(sha256(data).hexdigest(), len(data), name)
