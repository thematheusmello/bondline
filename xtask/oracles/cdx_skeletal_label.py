# SPDX-License-Identifier: MIT OR Apache-2.0
"""Check edited CDX bytes with a struct reader, FontTools glyph bounds and RDKit."""

import argparse
from pathlib import Path
import struct
import tempfile

from cdx_oracle import records

FIXTURES = Path(__file__).resolve().parents[2] / "crates/io/tests/fixtures"
WITNESSES = FIXTURES / "cdx-skeletal-label-edit"


def expected_bytes(source, fragment_id, node_id, element, hydrogen_count):
    from clearance_oracle import authored, placed_box

    rows = records(source)

    def child(parent, tag):
        (index,) = (
            i
            for i, row in enumerate(rows)
            if row.get("parent") == parent and row.get("tag") == tag
        )
        return index

    def payload(index):
        row = rows[index]
        return source[row["payload_start"] : row["payload_end"]]

    (fragment,) = (
        i
        for i, r in enumerate(rows)
        if r.get("tag") == 0x8003 and r["id"] == fragment_id
    )
    (node,) = (
        i
        for i, r in enumerate(rows)
        if r.get("tag") == 0x8004 and r["parent"] == fragment and r["id"] == node_id
    )
    label = child(node, 0x8006)
    text = child(label, 0x0700)
    y, x = struct.unpack("<ii", payload(child(node, 0x0200)))
    original = payload(text)
    assert struct.unpack_from("<HH", original) == (1, 0)
    (size,) = struct.unpack_from("<H", original, 8)
    symbol = {7: "N", 17: "Cl"}[element]
    body, glyphs = authored(symbol, hydrogen_count, size / 20, (x / 65536, y / 65536))
    left, top, right, bottom = placed_box(glyphs)
    edits = {
        child(node, 0x0402): struct.pack("<h", element),
        child(node, 0x042B): struct.pack("<H", hydrogen_count),
        child(label, 0x0204): struct.pack(
            "<4i", *(round(v * 65536) for v in (top, left, bottom, right))
        ),
        text: original[:12] + body.encode("ascii"),
    }
    result = source
    for index in sorted(edits, reverse=True):
        row, value = rows[index], edits[index]
        header = (
            struct.pack("<HI", 65535, len(value))
            if row["encoding"] == "extended"
            else struct.pack("<H", len(value))
        )
        result = result[: row["start"] + 2] + header + value + result[row["end"] :]
    return result


def verify_chem(raw, expected):
    from rdkit import Chem, rdBase
    from rdkit.Chem import rdMolDescriptors

    assert rdBase.rdkitVersion == "2026.09.2"
    params = Chem.CDXMLParserParams()
    params.format = Chem.CDXMLFormat.CDX
    params.sanitize = params.removeHs = False
    formulas = []
    for mol in Chem.MolsFromCDXML(raw, params):
        mol.UpdatePropertyCache(strict=True)
        formulas.append((Chem.MolToSmiles(mol), rdMolDescriptors.CalcMolFormula(mol)))
    assert sorted(formulas) == sorted(expected), formulas


def exercise(cli, out, name, target, element, h, chemistry, formula):
    from cdx_formula import dispatch

    source = FIXTURES / (name + ".cdx")
    raw = source.read_bytes()
    expected = (WITNESSES / (source.stem + ".cdx")).read_bytes()
    verify_chem(expected, chemistry)
    filename = name.replace("/", "-")
    saved, undone, redone = [
        out / (filename + "-" + stage + "-保存.cdx")
        for stage in ("edit", "undo", "redo")
    ]
    command = (
        "atom.set_authored_label",
        {
            "page_id": 1,
            "fragment_id": target[0],
            "atom_id": target[1],
            "expected_revision": 1,
            "element": element,
            "explicit_hydrogens": h,
        },
    )
    state = dispatch(
        cli,
        [
            ("document.open_cdx", {"path": str(source)}),
            command,
            ("document.save_cdx", {"path": str(saved)}),
            ("history.undo", {}),
            ("document.save_cdx", {"path": str(undone)}),
            ("history.redo", {}),
            ("document.save_cdx", {"path": str(redone)}),
        ],
    )
    assert (
        state["undo_depth"] == 1
        and state["redo_depth"] == 0
        and not state["file"]["dirty"]
    )
    assert (
        saved.read_bytes() == redone.read_bytes() == expected
        and undone.read_bytes() == source.read_bytes() == raw
    )
    reopened = dispatch(
        cli,
        [
            ("document.open_cdx", {"path": str(saved)}),
            (
                "analysis.formula",
                {"expected_revision": 1, "expected_selection_epoch": 0},
            ),
        ],
    )
    (atom,) = (
        atom
        for frag in reopened["document"]["pages"][0]["fragments"]
        for atom in frag["atoms"]
        if atom["id"] == target[1]
    )
    assert atom["element"] == element and atom["explicit_hydrogens"] == h
    assert (
        reopened["analysis"]["status"] == "available"
        and reopened["analysis"]["formula"] == formula
    )
    verify_chem(saved.read_bytes(), chemistry)
    print(
        f"{filename} -> {chemistry}: {len(expected)} exact bytes; {reopened['analysis']['formula']}",
        flush=True,
    )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("cli", type=Path, nargs="?")
    parser.add_argument("--generate", type=Path)
    args = parser.parse_args()
    if not args.generate and not args.cli:
        parser.error("give a CLI or a generation directory")
    with tempfile.TemporaryDirectory(prefix="bondline-mixed-label-") as tmp:
        for case in [
            (
                "cdx-mixed-skeleton/ethanol",
                (51, 97),
                (1, 3),
                7,
                2,
                [("CCN", "C2H7N")],
                "C2H7N",
            ),
            (
                "cdx-mixed-skeleton/methanol",
                (51, 97),
                (1, 2),
                17,
                0,
                [("CCl", "CH3Cl")],
                "CH3Cl",
            ),
            (
                "cdx-mixed-skeletal-move/printed",
                (51, 97),
                (1, 2),
                7,
                2,
                [("CN", "CH5N")],
                "CH5N",
            ),
            (
                "cdx-mixed-skeletal-move/twins",
                (62, 97),
                (2, 4),
                7,
                2,
                [("CO", "CH4O"), ("CN", "CH5N")],
                "C2H9NO",
            ),
        ]:
            name, scope, target, element, h, chemistry, formula = case
            if args.generate:
                src = FIXTURES / (name + ".cdx")
                args.generate.mkdir(parents=True, exist_ok=True)
                (args.generate / src.name).write_bytes(
                    expected_bytes(src.read_bytes(), *scope, element, h)
                )
            else:
                exercise(
                    args.cli.resolve(),
                    Path(tmp),
                    name,
                    target,
                    element,
                    h,
                    chemistry,
                    formula,
                )


if __name__ == "__main__":
    main()
