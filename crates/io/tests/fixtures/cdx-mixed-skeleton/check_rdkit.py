# SPDX-License-Identifier: MIT OR Apache-2.0
from pathlib import Path
from rdkit import Chem
from rdkit.Chem import rdMolDescriptors


def check_chem(data, formula, smiles):
    params = Chem.CDXMLParserParams()
    params.format, params.sanitize, params.removeHs = Chem.CDXMLFormat.CDX, False, False
    (mol,) = Chem.MolsFromCDXML(data, params)
    mol.UpdatePropertyCache(strict=True)
    assert (rdMolDescriptors.CalcMolFormula(mol), Chem.MolToSmiles(mol)) == (
        formula,
        smiles,
    )


facts = {
    "methanol": ("CH4O", "CO"),
    "ethanol": ("C2H6O", "CCO"),
    "amino_alcohol": ("C3H9NO", "NCCCO"),
    "literal_h": ("CH4O", "CO"),
}
for name, (formula, smiles) in facts.items():
    check_chem(Path(__file__).with_name(name + ".cdx").read_bytes(), formula, smiles)
print("Four actual CDX formulas and SMILES verified with RDKit")
