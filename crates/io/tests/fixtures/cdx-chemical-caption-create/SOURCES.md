# Adding a caption beside existing chemistry

These four original fixtures and their generator are available under MIT OR Apache-2.0. Run `python3 -B crates/io/tests/fixtures/cdx-chemical-caption-create/generate.py` to verify the checked-in files. Pass `--output-dir` with a directory to regenerate them.

| Starting source | File | Live ID | Raw ID | Signed depth | Bytes |
| --- | --- | ---: | ---: | ---: | ---: |
| `cdx-label-preview/boxed.cdx` | `chemical_first.cdx` | 1 | 903 | 1 | 432 |
| Same starting source after Undo | `chemical_after_undo.cdx` | 2 | 903 | 1 | 432 |
| `cdx-mixed-caption-edit/before.cdx` | `mixed_first.cdx` | 20 | 903 | 8 | 600 |
| Same starting source after Undo | `mixed_after_undo.cdx` | 21 | 903 | 8 | 600 |

Each source has two fragments with separately scoped, Node-owned Text ID 73. The mixed source also has a Page-owned Text ID 73 with live ID 19 at depth 7 and a peer at depth 2. The independently encoded new child carries UTF-8 text “Phase β & q” at baseline (144.25, 216.75) pt. It uses the existing black, 12 pt Roboto style and a persistent Bondline identity tag. The Python reader checks the child and marker owners, records and values and matches the original bytes exactly before and after insertion. [Binary framing and containment](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/IntroCDX.htm), [ZOrder](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/ZOrder.htm), [Text](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Text.htm), [ObjectTag](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/ObjectTag.htm).
