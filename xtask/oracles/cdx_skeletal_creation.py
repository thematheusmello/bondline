# SPDX-License-Identifier: MIT OR Apache-2.0
"""Independent binary witnesses for a new label on a previously hidden CDX atom."""
import argparse
from pathlib import Path
import struct

from cdx_oracle import records

FIXTURES = Path(__file__).resolve().parents[2] / "crates/io/tests/fixtures"
CASES = [
    ("ethanol", 61, 1, "N", 7, 2, "NCO", "CH5NO", 12, 0),
    ("amino_alcohol", 97, 2, "O", 8, 0, "NCCOO", "C2H7NO2", 12, 0),
    ("literal_h", 73, 1, "N", 7, 2, "NO", "H3NO", 14, 4),
    ("propane", 7, 2, "O", 8, 0, "COC", "C2H6O", 12, 0),
]


def before(name):
    directory = "cdx-skeletal" if name == "propane" else "cdx-mixed-skeleton"
    return FIXTURES / directory / (name + ".cdx")


def prop(tag, value):
    return struct.pack("<HH", tag, len(value)) + value


def expected(name, node_id, symbol, element, hydrogens, size, ink):
    from clearance_oracle import authored, placed_box

    raw = before(name).read_bytes()
    rows = records(raw)
    node, = [i for i, r in enumerate(rows) if r.get("tag") == 0x8004 and r["id"] == node_id]
    pos, = [r for r in rows if r.get("parent") == node and r.get("tag") == 0x0200]
    end, = [r for r in rows if r.get("kind") == "end" and r["matched"] == node]
    assert not any(r.get("parent") == node and r.get("tag") == 0x8006 for r in rows)
    y, x = struct.unpack_from("<ii", raw, pos["payload_start"])
    body, glyphs = authored(symbol, hydrogens, size, (x / 65536, y / 65536))
    left, top, right, bottom = placed_box(glyphs)
    box = struct.pack("<4i", *(round(v * 65536) for v in (top, left, bottom, right)))
    run = struct.pack("<6H", 1, 0, 0, 0, size * 20, ink) + body.encode("ascii")
    fresh = 1 + max(r["id"] for r in rows if r["kind"] == "start")
    child = (struct.pack("<HI", 0x8006, fresh) + prop(0x0204, box)
             + prop(0x0700, run) + prop(0x0701, bytes(1)) + prop(0x0823, bytes(1)) + bytes(2))
    fields = b""
    changes = [(end["start"], end["start"], child)]
    for tag, old, value in [(0x0402, 6, element), (0x042B, 2 if name == "propane" else 3, hydrogens)]:
        matches = [r for r in rows if r.get("parent") == node and r.get("tag") == tag]
        payload = struct.pack("<H", value)
        if matches:
            record, = matches
            assert struct.unpack_from("<H", raw, record["payload_start"]) == (old,)
            changes.append((record["payload_start"], record["payload_end"], payload))
        else:
            fields += prop(tag, payload)
    if fields:
        changes.append((pos["start"], pos["start"], fields))
    for start, stop, replacement in sorted(changes, reverse=True):
        raw = raw[:start] + replacement + raw[stop:]
    return raw


def verify(cli, verify_sources_only):
    from tempfile import TemporaryDirectory

    from cdx_formula import dispatch
    from rdkit import Chem, rdBase
    from rdkit.Chem import rdMolDescriptors

    assert rdBase.rdkitVersion == "2026.09.2"
    params = Chem.CDXMLParserParams()
    params.format = Chem.CDXMLFormat.CDX
    params.sanitize = params.removeHs = False
    with TemporaryDirectory(prefix="bondline-cdx-creation-") as directory:
        for name, _, atom, _, element, h, smiles, formula, size, ink in CASES:
            style = {"font_family": "Roboto", "size_pt": size, "face": 0,
                     "rgb": {0: [0, 0, 0], 4: [0.2, 0.4, 0.8]}[ink]}
            filename = name + ".cdx"
            original = before(name)
            witness = FIXTURES / "cdx-skeletal-label-creation" / filename
            mol, = Chem.MolsFromCDXML(witness.read_bytes(), params)
            mol.UpdatePropertyCache(strict=True)
            assert (Chem.MolToSmiles(mol), rdMolDescriptors.CalcMolFormula(mol)) == (smiles, formula)
            query = ("analysis.formula", {"expected_revision": 1, "expected_selection_epoch": 0})
            width = 768
            capture = ("render.capture", {"page_id": 1, "width_px": width, "height_px": width, "backend": "cpu"})
            opening = ("document.open_cdx", {"path": str(witness)})
            assert dispatch(cli, [opening, query])["analysis"]["formula"] == formula
            assert dispatch(cli, [opening, capture])["capture"]
            print(name, smiles, formula, len(witness.read_bytes()), flush=True)
            if verify_sources_only:
                continue
            paths = [Path(directory) / (name + tail) for tail in ("-変.cdx", "-undo.cdx", "-redo.cdx")]
            change = ("atom.create_authored_label", {
                "page_id": 1, "fragment_id": 1, "atom_id": atom, "expected_revision": 1,
                "element": element, "explicit_hydrogens": h, "style": style,
            })
            def save(path):
                return ("document.save_cdx", {"path": str(path)})
            state = dispatch(cli, [
                ("document.open_cdx", {"path": str(original)}), change, save(paths[0]),
                ("history.undo", {}), save(paths[1]), ("history.redo", {}), save(paths[2]),
            ])
            assert state["undo_depth"] == 1 and not state["file"]["dirty"]
            assert paths[0].read_bytes() == paths[2].read_bytes() == witness.read_bytes()
            assert paths[1].read_bytes() == original.read_bytes()
            saved = ("document.open_cdx", {"path": str(paths[0])})
            assert dispatch(cli, [saved, query])["analysis"]["formula"] == formula
            assert dispatch(cli, [saved, capture])["capture"] == dispatch(cli, [opening, capture])["capture"]


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    task = parser.add_mutually_exclusive_group(required=True)
    task.add_argument("--generate", type=Path)
    task.add_argument("--verify", type=Path)
    task.add_argument("--check-witnesses", type=Path)
    args = parser.parse_args()
    if args.generate:
        args.generate.mkdir(exist_ok=True, parents=True)
        for name, node, _, symbol, element, h, _, _, size, ink in CASES:
            (args.generate / (name + ".cdx")).write_bytes(expected(name, node, symbol, element, h, size, ink))
    else:
        verify(args.verify or args.check_witnesses, args.check_witnesses is not None)
