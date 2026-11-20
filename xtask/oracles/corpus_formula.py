"""Read reference molecules directly from the original file bytes, in an external RDKit process."""
import json
import re
import sys

from rdkit import Chem, rdBase
from rdkit.Chem import rdMolDescriptors


def reference(molecule):
    assert molecule is not None and molecule.GetNumAtoms(), "RDKit did not read a molecule"
    raw = rdMolDescriptors.CalcMolFormula(molecule, separateIsotopes=True, abbreviateHIsotopes=False)
    formula = raw
    if not any(atom.GetAtomicNum() == 6 for atom in molecule.GetAtoms()):
        # RDKit writes hydrogen first for some carbon-free formulas. Hill orders all symbols.
        body, charge = re.fullmatch(r"(.*?)([+-][0-9]*)?", raw).groups()
        tokens = re.findall(r"(?:\[[0-9]+[A-Z][a-z]?\]|[A-Z][a-z]?)[0-9]*", body)
        assert "".join(tokens) == body
        formula = "".join(sorted(tokens, key=lambda t: re.search(r"[A-Z][a-z]?", t)[0])) + (charge or "")
    return {"formula": formula, "atoms": molecule.GetNumAtoms()}


def records(kind, source):
    if kind in ("cdx", "cdxml"):
        params = Chem.CDXMLParserParams()
        params.format = Chem.CDXMLFormat.CDX if kind == "cdx" else Chem.CDXMLFormat.CDXML
        params.removeHs = False
        molecules = Chem.MolsFromCDXML(source, params)
        assert molecules
        whole = molecules[0]
        for part in molecules[1:]:
            whole = Chem.CombineMols(whole, part)
        return [reference(whole)]
    if kind == "mol":
        return [reference(Chem.MolFromMolBlock(source.decode("ascii"), removeHs=False))]
    if kind == "sdf":
        # DT-only properties can make the full supplier skip a record. Read each
        # original mol block independently; the corpus also tests the supplier itself.
        results, block = [], []
        for line in source.splitlines(keepends=True):
            if line.startswith(b"$$$$"):
                stop = next(i for i, text in enumerate(block) if text.rstrip(b"\r\n") == b"M  END")
                results.extend(records("mol", b"".join(block[:stop + 1])))
                block = []
            else:
                block.append(line)
        assert results and not block
        return results
    assert kind == "smiles"
    params = Chem.SmilesParserParams()
    params.removeHs = False
    return [reference(Chem.MolFromSmiles(line.split()[0].decode("ascii"), params))
            for line in source.splitlines() if line.strip()]


if __name__ == "__main__":
    assert rdBase.rdkitVersion == "2026.09.2", "use the pinned BONDLINE_PYTHON interpreter"
    print(json.dumps(records(sys.argv[1], sys.stdin.buffer.read())))
