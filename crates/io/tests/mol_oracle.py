"""Independent external RDKit consumer; developer-only, never linked or shipped."""
import json
import sys
import hashlib
from decimal import Decimal, ROUND_HALF_EVEN, localcontext
from rdkit import Chem, rdBase
from rdkit.Chem import rdMolDescriptors

assert rdBase.rdkitVersion == "2026.09.2", "use pinned BONDLINE_PYTHON"
mode=sys.argv[1] if len(sys.argv)>1 else "describe"
if mode=="emit":
    molecule=Chem.MolFromSmiles("[13CH3][O-]")
    assert molecule is not None
    molecule.SetProp("_Name","independent RDKit-generated vector")
    conformer=Chem.Conformer(2)
    conformer.Set3D(False)
    conformer.SetAtomPosition(0,(-1.25,2.5,0.0))
    conformer.SetAtomPosition(1,(0.25,2.5,0.0))
    molecule.AddConformer(conformer)
    sys.stdout.buffer.write(Chem.MolToMolBlock(molecule,forceV3000=False).encode("ascii"))
    raise SystemExit(0)
source = sys.stdin.buffer.read(64 * 1024 * 1024 + 1)
assert len(source) <= 64 * 1024 * 1024
if mode=="quantization":
    with localcontext() as context:
        context.prec=50
        expected=[str((Decimal(value)/Decimal(24)).quantize(Decimal("0.0001"),rounding=ROUND_HALF_EVEN)) for value in [100,25]]
    print(json.dumps({"expected":expected}))
    raise SystemExit(0)
molecule = Chem.MolFromMolBlock(source.decode("ascii"), sanitize=mode!="raw", removeHs=False, strictParsing=True)
assert molecule is not None, "external consumer rejected MOL output"
if mode=="raw":
    molecule.UpdatePropertyCache(strict=False)
conformer = molecule.GetConformer()
print(json.dumps({
    "source_sha256":hashlib.sha256(source).hexdigest(),
    "smiles": Chem.MolToSmiles(molecule) if mode!="raw" else None,
    "formula": rdMolDescriptors.CalcMolFormula(molecule) if mode!="raw" else None,
    "atoms": [{"element": atom.GetAtomicNum(), "charge": atom.GetFormalCharge(),
               "isotope": atom.GetIsotope(), "hydrogens": atom.GetTotalNumHs(),
               "radical_electrons": atom.GetNumRadicalElectrons(),
               "position": list(conformer.GetAtomPosition(atom.GetIdx()))}
              for atom in molecule.GetAtoms()],
    "bonds": [{"begin": bond.GetBeginAtomIdx(), "end": bond.GetEndAtomIdx(),
               "order": bond.GetBondTypeAsDouble()} for bond in molecule.GetBonds()],
}, separators=(",", ":")))
