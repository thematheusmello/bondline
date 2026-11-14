"""Original MIT OR Apache-2.0 RXN test oracle; external only, never shipped.

BIOVIA Chemistry2026 CTfile Formats printed91-92 establishes the fixed wrapper.
Raw framing is checked independently; RDKit does not preserve every unknown field.
"""
import hashlib
import json
import re
import sys
from rdkit import Chem, rdBase
from rdkit.Chem import rdChemReactions as reactions

MAX_BYTES = 64 * 1024 * 1024

def scan(source):
    assert len(source) <= MAX_BYTES
    assert all(byte in (9, 10, 13) or 32 <= byte <= 126 for byte in source)
    spans = [match.span() for match in re.finditer(rb"[^\n]*\n|[^\n]+$", source)]
    assert len(spans) <= 1_000_000 and len(spans) >= 5
    def line(index):
        start, end = spans[index]
        text = source[start:end].removesuffix(b"\n").removesuffix(b"\r")
        assert b"\r" not in text and len(text) <= 4096
        return text
    assert line(0).rstrip(b" ") == b"$RXN"
    count_line = line(4)
    assert len(count_line) >= 6
    if len(count_line) < 9:
        assert len(count_line) == 6
        count_line = count_line + b"  0"
    assert count_line[9:].strip(b" ") == b""
    counts = []
    for offset in (0, 3, 6):
        text = count_line[offset:offset+3].strip(b" ")
        assert text and all(48 <= byte <= 57 for byte in text)
        counts.append(int(text))
    assert sum(counts) <= 1024
    components, index, atoms, bonds = [], 5, 0, 0
    for role, count in zip(("reactant", "product", "reagent"), counts):
        for _ in range(count):
            assert index < len(spans) and line(index) == b"$MOL"
            marker = index
            index += 1
            start = index
            assert index + 4 <= len(spans)
            version = line(index + 3)[33:39]
            assert version in (b" V2000", b" V3000")
            index += 4
            while index < len(spans) and line(index).rstrip(b" \t") != b"M  END":
                index += 1
            assert index < len(spans)
            end = spans[index][1]
            payload = source[spans[start][0]:end]
            if version == b" V2000":
                declared = line(start + 3)
                na, nb = int(declared[:3].strip() or b"0"), int(declared[3:6].strip() or b"0")
                assert 0 <= na <= 999 and 0 <= nb <= 999
            else:
                match = re.search(rb"^M  V30 COUNTS ([0-9]+) ([0-9]+) ", payload, re.M)
                assert match, "oracle's generated/literal V3000 count subset"
                na, nb = map(int, match.groups())
                assert na <= 100_000 and nb <= 100_000
            atoms += na
            bonds += nb
            assert atoms <= 100_000 and bonds <= 100_000
            molecule = Chem.MolFromMolBlock(payload.decode("ascii"), sanitize=False,
                                           removeHs=False, strictParsing=True)
            assert molecule is not None
            components.append({"role": role, "version": version.strip().decode().lower(),
                "range": [spans[marker][0], end], "mol_range": [spans[start][0], end],
                "mol_sha256": hashlib.sha256(payload).hexdigest(),
                "elements": [atom.GetAtomicNum() for atom in molecule.GetAtoms()],
                "maps": [atom.GetAtomMapNum() for atom in molecule.GetAtoms()],
                "positions": [list(molecule.GetConformer().GetAtomPosition(i))
                              for i in range(molecule.GetNumAtoms())],
                "bonds": [[bond.GetBeginAtomIdx(), bond.GetEndAtomIdx(), bond.GetBondTypeAsDouble()]
                          for bond in molecule.GetBonds()]})
            index += 1
    assert index == len(spans), "uncounted/trailing content"
    parsed = reactions.ReactionFromRxnBlock(source.decode("ascii"), sanitize=False,
                                           removeHs=False, strictParsing=True)
    assert parsed is not None
    observed = [parsed.GetNumReactantTemplates(), parsed.GetNumProductTemplates(), parsed.GetNumAgentTemplates()]
    assert observed == counts
    return {"source_sha256": hashlib.sha256(source).hexdigest(), "counts": counts,
            "header": [line(i).decode("ascii") for i in (1, 2, 3)],
            "header_ranges": [list(spans[i]) for i in (1, 2, 3)],
            "counts_range": list(spans[4]), "components": components,
            "consumer_roles": observed}

def reaction():
    result = reactions.ChemicalReaction()
    for smiles in ("C", "O"):
        result.AddReactantTemplate(Chem.MolFromSmiles(smiles))
    result.AddProductTemplate(Chem.MolFromSmiles("CO"))
    result.AddAgentTemplate(Chem.MolFromSmiles("N"))
    return result

def emit():
    return reactions.ReactionToRxnBlock(reaction(), separateAgents=True).encode("ascii")

def emit_mixed():
    molecule = Chem.RWMol()
    for _ in range(1000): molecule.AddAtom(Chem.Atom(6))
    molecule = molecule.GetMol()
    molecule.UpdatePropertyCache(strict=False)
    result = reactions.ChemicalReaction()
    result.AddReactantTemplate(molecule)
    result.AddProductTemplate(Chem.MolFromSmiles("C"))
    return reactions.ReactionToRxnBlock(result, separateAgents=True, forceV3000=False).encode("ascii")

def self_test():
    ordinary, mixed = emit(), emit_mixed()
    assert len(ordinary) == 700 and hashlib.sha256(ordinary).hexdigest() == "8c373afa75fb12fed0a76906d5316794e1453cce6193ed75eb225a4fa25ff8a3"
    assert len(mixed) == 44023 and hashlib.sha256(mixed).hexdigest() == "86720c4025a8f66acbcb4fdbf592d6ba37a453fa78d1df86ec6e99920f0394b7"
    assert scan(ordinary)["counts"] == [2, 1, 1]
    assert [component["version"] for component in scan(mixed)["components"]] == ["v3000", "v2000"]
    folded = reactions.ReactionToRxnBlock(reaction())
    parsed = reactions.ReactionFromRxnBlock(folded, sanitize=False, removeHs=False, strictParsing=True)
    assert [parsed.GetNumReactantTemplates(), parsed.GetNumProductTemplates(), parsed.GetNumAgentTemplates()] == [3, 1, 0]
    # The external consumer ignores trailing garbage; our raw scanner must not.
    assert reactions.ReactionFromRxnBlock(ordinary.decode()+"trailing data\n") is not None
    try: scan(ordinary + b"trailing data\n")
    except AssertionError: pass
    else: raise AssertionError("raw oracle silently dropped trailing data")
    return {"self_test": "passed", "default_folded_roles": [3, 1, 0]}

if __name__ == "__main__":
    assert rdBase.rdkitVersion == "2026.09.2", "use the pinned external interpreter"
    mode = sys.argv[1] if len(sys.argv) == 2 else "describe"
    assert len(sys.argv) <= 2 and mode in ("describe", "emit", "emit_mixed", "self-test")
    if mode in ("emit", "emit_mixed"):
        sys.stdout.buffer.write(emit() if mode == "emit" else emit_mixed())
    else:
        result = self_test() if mode == "self-test" else scan(sys.stdin.buffer.read(MAX_BYTES+1))
        sys.stdout.buffer.write(json.dumps(result, allow_nan=False, separators=(",", ":")).encode("ascii")+b"\n")
