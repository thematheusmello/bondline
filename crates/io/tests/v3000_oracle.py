"""Original MIT OR Apache-2.0 external V3000 oracle; never linked or shipped.

BIOVIA Chemistry 2026 CTfile Formats, printed pages 7-13 and 32:
https://www.3ds.com/assets/invest/2026-01/biovia_chemistry_2026_ctfile_formats.pdf
RDKit describes a supported graph; it does not preserve unknown syntax or spin
multiplicity. Raw hashes describe stdin bytes, not a consumer round trip.
"""

import hashlib
import io
import json
import math
import re
import sys

from rdkit import Chem, rdBase

PINNED_RDKIT = "2026.09.2"
MAX_BYTES = 64 * 1024 * 1024
MAX_ITEMS = 100_000
MAX_LINES = 1_000_000
MAX_LINE_BYTES = 80
MAX_LOGICAL_BYTES = 65536
MAX_SEGMENTS = 1024
MAX_TOKEN_VALUES = 8192
MAX_DEPTH = 256


def read_source(stream):
    source = stream.read(MAX_BYTES + 1)
    assert len(source) <= MAX_BYTES, "input exceeds 64 MiB"
    return source


def bounded_source(source):
    """Bound work before the external parser; this is not a preservation parser."""
    assert 0 < len(source) <= MAX_BYTES, "input byte budget"
    text = source.decode("ascii")
    assert all(character in "\r\n" or " " <= character <= "~" for character in text), "printable ASCII subset"
    logical, segments, depth = "", 0, 0
    for number, raw in enumerate(io.StringIO(text), 1):
        assert number <= MAX_LINES, "physical line budget"
        line = raw.removesuffix("\n").removesuffix("\r")
        assert "\r" not in line and len(line) <= MAX_LINE_BYTES, "physical line/encoding bound"
        if number <= 4:
            if number == 4:
                assert line[33:39] == " V3000", "V3000 oracle subset"
            continue
        if logical or segments:
            assert line.startswith("M  V30 "), "continuation prefix"
        elif not line.startswith("M  V30 "):
            continue
        segment = line[7:]
        continued = segment.endswith("-")
        logical += segment[:-1] if continued else segment
        segments += 1
        assert segments <= MAX_SEGMENTS and len(logical) <= MAX_LOGICAL_BYTES, "logical entry budget"
        if continued:
            continue
        # Ignore quoted values when checking allocation-sensitive numeric counts.
        unquoted = re.sub(r'"(?:""|[^"])*"', '""', logical)
        tokens = unquoted.split()
        assert len(tokens) <= MAX_TOKEN_VALUES, "token budget"
        if tokens and tokens[0] == "COUNTS":
            assert len(tokens) >= 6, "truncated counts"
            for count in tokens[1:5]:
                assert len(count) <= 6 and count.isdecimal() and int(count) <= MAX_ITEMS, "object count budget"
        for match in re.finditer(r"=\(\s*([+-]?[0-9]+)", unquoted):
            count = match.group(1)
            assert len(count) <= 6 and 0 <= int(count) <= MAX_TOKEN_VALUES, "list count budget"
        if tokens and tokens[0] in ("BEGIN", "END"):
            depth += 1 if tokens[0] == "BEGIN" else -1
            assert 0 <= depth <= MAX_DEPTH, "block depth budget"
        logical, segments = "", 0
    assert number >= 4 and not segments and depth == 0, "truncated header/entry/block"
    return text


def describe(source):
    text = bounded_source(source)
    molecule = Chem.MolFromMolBlock(text, sanitize=False, removeHs=False, strictParsing=True)
    assert molecule is not None, "external consumer rejected V3000"
    assert molecule.GetNumAtoms() <= MAX_ITEMS and molecule.GetNumBonds() <= MAX_ITEMS
    atoms = []
    for atom in molecule.GetAtoms():
        position = list(molecule.GetConformer().GetAtomPosition(atom.GetIdx()))
        assert all(math.isfinite(value) for value in position), "non-finite coordinate"
        atoms.append({
            "element": atom.GetAtomicNum(), "isotope": atom.GetIsotope(),
            "charge": atom.GetFormalCharge(), "explicit_h": atom.GetNumExplicitHs(),
            "query": atom.DescribeQuery() if atom.HasQuery() else None,
            "radical_electrons": atom.GetNumRadicalElectrons(), "position": position,
            "no_implicit": atom.GetNoImplicit(), "map": atom.GetAtomMapNum(),
        })
    return {
        "rdkit": rdBase.rdkitVersion, "source_sha256": hashlib.sha256(source).hexdigest(),
        "source_bytes": len(source), "atoms": atoms,
        "bonds": [{
            "begin": bond.GetBeginAtomIdx(), "end": bond.GetEndAtomIdx(),
            "order": bond.GetBondTypeAsDouble(), "direction": str(bond.GetBondDir()),
            "stereo": str(bond.GetStereo()),
        } for bond in molecule.GetBonds()],
    }


def emit(large):
    molecule = Chem.RWMol()
    for index in range(1000 if large else 2):
        atom = Chem.Atom(6)
        atom.SetNoImplicit(True)
        if not large:
            atom.SetNumExplicitHs(3)
            atom.SetIsotope(13 if index == 0 else 0)
        molecule.AddAtom(atom)
    if not large:
        molecule.AddBond(0, 1, Chem.BondType.SINGLE)
    molecule = molecule.GetMol()
    molecule.UpdatePropertyCache(strict=False)
    if large:
        text = Chem.MolToMolBlock(molecule, includeStereo=False, kekulize=False)
    else:
        conformer = Chem.Conformer(2)
        conformer.Set3D(False)
        conformer.SetAtomPosition(0, (-1.23456789, 2.34567891, 0.0))
        conformer.SetAtomPosition(1, (0.26543211, 2.34567891, 0.0))
        molecule.AddConformer(conformer)
        parameters = Chem.MolWriterParams()
        parameters.includeStereo = False
        parameters.kekulize = False
        parameters.forceV3000 = True
        parameters.precision = 8
        text = Chem.MolToV3KMolBlock(molecule, parameters)
    # Binary stdout avoids Windows text-mode LF translation.
    return text.encode("ascii")


def original_block(atoms, bonds=()):
    """Original test vectors, not copied vendor examples or generated fixtures."""
    records = [
        "Bondline original V3000 probe",
        "  Bondline          2D",
        "Original specification-derived vector",
        "  0  0  0  0  0  0            999 V3000",
        "BEGIN CTAB",
        f"COUNTS {len(atoms)} {len(bonds)} 0 0 0",
        "BEGIN ATOM",
        *atoms,
        "END ATOM",
    ]
    if bonds:
        records.extend(["BEGIN BOND", *bonds, "END BOND"])
    records.extend(["END CTAB", "M  END"])
    return ("\n".join(
        line if index < 4 or line == "M  END" else "M  V30 " + line
        for index, line in enumerate(records)
    ) + "\n").encode("ascii")


def self_test():
    source = original_block(
        ["10 C -1.25 0.5 0 0 VAL=4", "40 C 0.25 0.5 0 0 VAL=4"],
        ["7 1 10 40"],
    )
    graph = describe(source)
    assert graph["source_sha256"] == "1867a5904e742030431b523a31167951ff998007a89b9c3d94ec88cf7d4c4380"
    assert graph["source_bytes"] == len(source)
    assert [atom["explicit_h"] for atom in graph["atoms"]] == [3, 3]
    assert [atom["position"] for atom in graph["atoms"]] == [[-1.25, 0.5, 0.0], [0.25, 0.5, 0.0]]
    assert graph["bonds"] == [{
        "begin": 0, "end": 1, "order": 1.0,
        "direction": "NONE", "stereo": "STEREONONE",
    }]
    radical = describe(original_block(["10 C 0 0 0 0 MA-\nM  V30 SS=13 RAD=2 VAL=3"]))["atoms"][0]
    assert (radical["element"], radical["isotope"], radical["radical_electrons"], radical["explicit_h"]) == (6, 13, 1, 3)
    ammonium = describe(original_block(["10 N 0 0 0 0 CHG=1 VAL=4"]))["atoms"][0]
    assert (ammonium["charge"], ammonium["explicit_h"]) == (1, 4)
    assert describe(original_block([]))["atoms"] == []
    emitted = emit(False)
    assert len(emitted) == 314
    assert hashlib.sha256(emitted).hexdigest() == "c5260d67d8bf6e416c6d3be94e2dd4019516300a4fb4e36ba6c3f01193c50e38"
    assert b"\r" not in emitted
    generated = describe(emitted)
    assert generated["atoms"][0]["position"] == [-1.23456789, 2.34567891, 0.0]
    assert generated["atoms"][0]["isotope"] == 13
    assert generated["atoms"][0]["explicit_h"] == 0  # RDKit omits normal VAL on write.
    large = emit(True)
    assert len(large) == 49058
    assert hashlib.sha256(large).hexdigest() == "3e7d06cca8a916618536ee915ac7b52dac10ba7f657398496ccc76578a64f27c"
    assert len(describe(large)["atoms"]) == 1000
    assert b"\r" not in large

    # Consumer limitations are diagnostics, not expectations imposed on Bondline.
    query = describe(original_block(["10 C 0 0 0 0 HCOUNT=3"]))["atoms"][0]
    assert query["query"] is not None and query["explicit_h"] == 0
    spins = [describe(original_block([f"10 C 0 0 0 0 RAD={spin} VAL=-1"]))["atoms"][0]["radical_electrons"] for spin in (1, 3)]
    assert spins == [2, 2]
    assert describe(original_block(["10 C 0 0 0 0 MASS=13.5 VAL=-1"]))["atoms"][0]["isotope"] == 13
    assert describe(original_block(["10 N 0 0 0 0 CHG=16 VAL=-1"]))["atoms"][0]["charge"] == 16
    unknown = original_block(['10 C 0 0 0 0 VAL=-1 XNOTE="two ""quoted"" words"'])
    parsed = Chem.MolFromMolBlock(unknown.decode("ascii"), sanitize=False, removeHs=False, strictParsing=True)
    assert parsed is not None
    assert not parsed.GetAtomWithIdx(0).HasProp("XNOTE")
    parsed.UpdatePropertyCache(strict=False)
    assert "XNOTE" not in Chem.MolToV3KMolBlock(parsed, includeStereo=False, kekulize=False)
    legacy = source.replace(b"  0  0  0  0  0  0            999 V3000", b"  9  9  0  0  0  0            999 V3000", 1)
    assert Chem.MolFromMolBlock(legacy.decode("ascii"), sanitize=False, removeHs=False, strictParsing=True) is None

    def rejected(source):
        try:
            bounded_source(source)
        except (AssertionError, UnicodeDecodeError, ValueError):
            return
        raise AssertionError("hostile input passed the oracle resource preflight")

    rejected(b"A" * (MAX_BYTES + 1))
    rejected(source.replace(b"COUNTS 2 1", b"COUNTS 999999999 1"))
    rejected(source.replace(b"VAL=4", b"VAL=4 ITEMS=(999999999 1)", 1))
    rejected(source.replace(b"BEGIN ATOM", b"BEGIN " + b"X" * MAX_LINE_BYTES, 1))
    rejected(source.replace(b"BEGIN ATOM", b"BEGIN X\nM  V30 " * (MAX_DEPTH + 1) + b"BEGIN ATOM", 1))
    rejected(source.replace(b"VAL=4", b"VAL=4\x00", 1))
    rejected(source.replace(b"999 V3000", b"999 V2000", 1))
    # Bounded stream input preserves all source bytes.
    assert read_source(io.BytesIO(source)) == source
    return {"rdkit": rdBase.rdkitVersion, "self_test": "passed", "diagnostics": {
        "hcount_is_query": True, "singlet_triplet_radical_electrons": spins,
        "unknown_properties_preserved": False, "fractional_mass_truncated": True,
        "charge_16_raw_accepted": True, "nonzero_legacy_counts_strictly_rejected": True,
    }}


if __name__ == "__main__":
    assert rdBase.rdkitVersion == PINNED_RDKIT, "use pinned BONDLINE_PYTHON"
    mode = sys.argv[1] if len(sys.argv) > 1 else "describe"
    assert len(sys.argv) <= 2, "one mode argument only"
    if mode == "self-test":
        result = self_test()
    elif mode == "describe":
        result = describe(read_source(sys.stdin.buffer))
    elif mode in ("emit", "emit1000"):
        sys.stdout.buffer.write(emit(mode == "emit1000"))
        raise SystemExit(0)
    else:
        raise AssertionError("usage: v3000_oracle.py [describe|emit|emit1000|self-test]")
    sys.stdout.buffer.write((json.dumps(result, separators=(",", ":"), allow_nan=False) + "\n").encode("ascii"))
