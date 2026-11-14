"""Original MIT OR Apache-2.0 actual-file graph/negative-control acceptance."""
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "io" / "tests"))
from rdkit import rdBase
from smiles_write_oracle import generated

assert rdBase.rdkitVersion == "2026.09.2"
assert len(sys.argv) == 2


def graph(record):
    assert 0 < len(record) <= 65_537 and record.endswith(b"\n")
    notation = record[:-1]
    assert notation and all(33 <= byte <= 126 for byte in notation)
    result = generated(notation)  # unchanged pinned unsanitized + sanitized consumer
    atoms = result["notation"]["atoms"]
    labels = [atom["isotope"] for atom in atoms]
    assert len(set(labels)) == len(labels), "fixture isotope keys must remain unique"
    nodes = sorted((atom["isotope"], atom["element"], atom["explicit_h"],
                    atom["charge"], atom["no_implicit"]) for atom in atoms)
    edges = sorted((min(labels[left], labels[right]), max(labels[left], labels[right]), order)
                   for left, right, order in result["notation"]["bonds"])
    assert result["meaning"]["conformers"] == 0
    assert result["meaning"]["radicals"] == [0] * len(atoms)
    return (nodes, edges), result["meaning"]["formula"]


nodes = sorted([(mass, 6, 1, 0, True) for mass in range(11, 19)] + [
    (0, 1, 0, 0, True), (2, 1, 0, 0, True), (23, 6, 2, 0, True),
    (41, 6, 1, 0, True), (42, 6, 1, 0, True),
    (51, 6, 1, 0, True), (52, 6, 3, 0, True),
    (53, 8, 1, 0, True), (54, 9, 0, 0, True),
])
edges = sorted([
    (11, 12, "DOUBLE"), (12, 13, "SINGLE"), (13, 14, "DOUBLE"),
    (14, 15, "SINGLE"), (15, 16, "DOUBLE"), (16, 17, "SINGLE"),
    (17, 18, "DOUBLE"), (11, 18, "SINGLE"),
    (0, 23, "SINGLE"), (2, 23, "SINGLE"), (41, 42, "TRIPLE"),
    (51, 52, "SINGLE"), (51, 53, "SINGLE"), (51, 54, "SINGLE"),
])
expected = (nodes, edges)
ring = b"[11CH1]1=[12CH1][13CH1]=[14CH1][15CH1]=[16CH1][17CH1]=[18CH1]1"
tail = b".[23CH2]([H])[2H].[41CH1]#[42CH1].[51CH1]([52CH3])([53OH1])[54FH0]"
reference, formula = graph(ring + tail + b"\n")
assert reference == expected
with Path(sys.argv[1]).open("rb") as file:
    actual = file.read(65_538)
assert graph(actual) == (expected, formula), "actual published graph differs"

wrong_rings = (b"[11CH1]1=[12CH1][13CH1]=[14CH1]1."
               b"[15CH1]1=[16CH1][17CH1]=[18CH1]1")
wrong_orders = b"[11CH1]1[12CH1]=[13CH1][14CH1]=[15CH1][16CH1]=[17CH1][18CH1]=1"
for wrong in [wrong_rings, wrong_orders]:
    counter, counter_formula = graph(wrong + tail + b"\n")
    assert counter[0] == nodes and len(counter[1]) == len(edges)
    assert sorted(edge[2] for edge in counter[1]) == sorted(edge[2] for edge in edges)
    assert counter_formula == formula
    assert counter != expected, "same-count/formula graph control was not detected"
collapsed = tail.replace(b"[23CH2]([H])[2H]", b"[23CH3][2H]")
counter, counter_formula = graph(ring + collapsed + b"\n")
assert counter_formula == formula and counter != expected
assert graph(ring + tail.replace(b"[41CH1]", b"[43CH1]") + b"\n")[0] != expected
assert graph(b"[99CH4]\n")[0] != expected
print(json.dumps({"atoms": 17, "bonds": 14, "semantic_controls": 5,
                  "same_count_formula_controls": 2, "actual_file": True}))
