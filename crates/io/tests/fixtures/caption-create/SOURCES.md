# Original native caption creation witnesses

The fixtures and generator are MIT OR Apache-2.0. Python builds
the native byte strings and Expat verifies the Page, Fragment,
Node, Bond, Text and Style owners and values. The source lock
records the public file and interaction evidence, including the
published DTD resource order discrepancy.
[Native caption creation contract](../../../../../docs/research/native-caption-creation.md).

| Case | Input → output | Model Text IDs → XML Text IDs |
| --- | --- | --- |
| `first` | `blank.cdxml` → `first.cdxml` | 1 → 2 |
| `after_undo` | `blank.cdxml` → `after_undo.cdxml` | 2 → 3 |
| `mixed_append` | `mixed_before.cdxml` → `mixed_after.cdxml` | 1 and 2 → 6 and 7 |

The standalone results differ by the Text id alone. The mixed
output retains every original byte through the final existing
caption, inserts the new caption and retains the closing tags.
The two old runs use black 14 pt and blue 18 pt, sharing font 0
with the new black 12 pt run. The existing white, black and blue
palette remains byte exact. `mixed_noncanonical.cdxml` inserts
only a root comment and retains the expected mixed model.

RDKit 2026.09.2 reads the same directed single bond and two C/H3
atoms in all three mixed files with sanitization and H removal
disabled. Its formula for each is C2H6.
[RDKit CDXML parser](https://www.rdkit.org/docs/source/rdkit.Chem.rdmolfiles.html#rdkit.Chem.rdmolfiles.MolsFromCDXML).

FontTools 4.61.1 and the repository's independent pinned Roboto
oracle compute per-run ink and advances. The new single run has
ink bounds [145.240234375, 207.75, 206.880859375, 219.1875] pt and
advances 63.55078125 pt from the click baseline. The existing mixed
caption anchor is (80.75, 124.25) pt.
[Font and outline provenance](../cdx-caption-edit/SOURCES.md).

Reproduce from the repository root with FontTools 4.61.1.

```sh
fixtures=crates/io/tests/fixtures/caption-create
out=$(mktemp -d)
python3 -B "$fixtures/generate_creation.py" --output-dir "$out/vectors"
diff -r -x SOURCES.md -x generate_creation.py "$fixtures" "$out/vectors"
```

Manifest SHA-256 is `d7578c0fa9cd833659b3133529a36070e2649d444a58e63027ac0aac75712616`.
