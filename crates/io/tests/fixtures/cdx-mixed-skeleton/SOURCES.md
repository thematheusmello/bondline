# Skeletal carbon with printed heteroatoms

The binary drawings, XML control, expected clearance intervals and scripts are original Bondline test data under MIT OR Apache-2.0. The binary generator uses Python record and resource packers. It reads two original CDXML controls and constructs a third by adding one appropriately bonded carbon to methanol. The separate CDX reader checks record boundaries and framing. FontTools 4.61.1 reads the pinned Roboto font and independently reproduces every printed box and each directed bond cut. The independent RDKit parser consumes the binary files. [Public SDK and drawing contract](../../../../../docs/research/cdx-mixed-skeleton.md) · [Original CDXML controls](../mixed-labels/SOURCES.md) · [Clearance oracle](../../../../../xtask/oracles/README-clearance.md)

| Binary source | Authored chemistry | Expected display |
| --- | --- | --- |
| methanol | Hidden C with omitted H and a dormant nonterminal flag, printed OH | CH₄O |
| literal_h | The same geometry, explicit H3 on the hidden carbon with absent flags | CH₄O |
| ethanol | Two hidden carbons with omitted H, terminal printed OH | C₂H₆O |
| amino_alcohol | OH and NH2 separated by three hidden carbons with omitted H | C₃H₉NO |

All four contain an unused UTF-8 font-table entry named Étude λ. The negative literal_utf8 source retains its actual child text Oη as data. Additional controls exercise overlapping labels, a contradictory box, coincident endpoints, an unlabelled oxygen, missing printed H, an active carbon flag and a one-byte implied flag. Only the supported four drawings have certified paper. [Test implementation](../../cdx_mixed_skeleton.rs)

To reproduce from the repository root, run the first command with FontTools 4.61.1. Run the second with the pinned RDKit interpreter.

```sh
python3 -B crates/io/tests/fixtures/cdx-mixed-skeleton/generate.py
"$BONDLINE_PYTHON" -B crates/io/tests/fixtures/cdx-mixed-skeleton/check_rdkit.py
```
