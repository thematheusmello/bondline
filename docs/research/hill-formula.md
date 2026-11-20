# Molecular formula in Hill order

`Composition::hill_formula(FormulaIsotopes)` formats the resolved elemental and isotope counts of one fragment as plain text. It includes every physical atom, its known attached H, and the net formal charge. Disconnected components sum into the same composition. The empty composition gives an empty string. The analysis entry point still reports an unresolved attached-H count; the formatter performs no valence or isotope-abundance inference. [Numeric composition contract](formula-composition.md).

## Ordering and spelling

Hill's 1900 rule places carbon first and hydrogen second when carbon occurs. All remaining element symbols follow alphabetically. Without carbon, all symbols, including H, follow alphabetically. Compare symbols case-sensitively as whole units. Keep the full count, omit a subscript of one and never reduce a molecular formula to lowest terms. For example, hydrogen bromide sorts as `BrH`; ethanol and dimethyl ether both give `C2H6O`. [Hill, JACS 22, 478–494, especially p. 484](https://doi.org/10.1021/ja02046a005), [NIST indexing rules](https://srd.nist.gov/NSRDS/NSRDS-NBS37.pdf), [NIST hydrogen bromide formula index](https://webbook.nist.gov/cgi/formula/BrH).

The caller chooses how to display the existing isotope bins. `FormulaIsotopes::Combined` combines all mass numbers under the element symbol. `Separate` emits the unspecified isotope first, then numbered isotopes in increasing numeric order as `[massSymbol]`. An explicit mass number of one remains visible. Hydrogen-2 and hydrogen-3 use `[2H]` and `[3H]`. Unspecified and explicitly numbered 12C stay distinct. These bracket spellings are a plain-text convention exercised with RDKit's `separateIsotopes=True, abbreviateHIsotopes=False`. IUPAC uses a left superscript for a nuclide mass number in typeset text. [RDKit molecular formula API](https://rdkit.org/new_docs/cppapi/namespaceRDKit_1_1MolOps.html), [IUPAC Red Book, IR-3.2](https://old.iupac.org/publications/books/rbook/Red_Book_2005.pdf).

A neutral composition has no suffix. Charges of +1 or −1 use `+` or `-`. A larger magnitude follows the ASCII sign so that atom counts and charge magnitude have an explicit boundary, for example `O4S-2` and `Fe+3`. This matches the tested RDKit text. IUPAC places the magnitude before the sign in a right superscript in typeset formulas. The text API does not assign oxidation states or distinguish charge distributions with equal total charge. [IUPAC Red Book, IR-3.2](https://old.iupac.org/publications/books/rbook/Red_Book_2005.pdf), [RDKit descriptor API](https://rdkit.org/docs/source/rdkit.Chem.rdMolDescriptors.html#rdkit.Chem.rdMolDescriptors.CalcMolFormula).

| Input | Combined | Separate | Raw RDKit difference |
| --- | --- | --- | --- |
| Ethanol | `C2H6O` | `C2H6O` | Same |
| Three isotope bins of methane carbon | `C4H16` | `C[12C][13C]2H16` | Same |
| Hydrogen bromide with deuterium | `BrH` | `Br[2H]` | `HBr` and `[2H]Br` |
| Ammonium chloride, both ions | `ClH4N` | `ClH4N` | `H4ClN` |
| Sulfate ion | `O4S-2` | `O4S-2` | Same |

## Verification

The tests first failed to compile because the formatter and its options did not yet exist. After implementation, all 20 tests in the chemistry crate passed, including the 20-structure external differential test. Python 3.13.0 ran RDKit 2026.09.2 as a separate process. Each structure is independently specified as SMILES and as a validated fragment. The reference parser retains explicit physical H. Each test verifies RDKit's unmodified formula strings and atom/isotope counts. For carbon-free structures, the checker reorders RDKit's complete tokens by symbol before comparing the strict Hill output. This qualification is observable in the hydrogen bromide and ammonium chloride cases above.

The arithmetic controls cover the full 100,000-atom model bound, 6,553,600,000 H, net charges of +12,700,000 and −12,800,000, and the model's maximum positive u16 mass number. Numbered mass text carries no isotope-existence claim. The formatter sorts at most 118 element symbols and visits the composition's already ordered isotope bins. No runtime dependency was added. The command registry and the native Analysis window are separate receiving work.

Run with the pinned external Python in `BONDLINE_PYTHON`.

```sh
cargo test --locked -p bondline-chem
cargo clippy --locked -p bondline-chem --all-targets -- -D warnings
cargo fmt --all --check
```
