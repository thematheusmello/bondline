# Binary skeletal-carbon fixtures

These original fixtures, generator and oracle are MIT OR Apache-2.0. The positive case is a three-carbon chain on 512 × 512 pt paper with 4 pt black lines. Each bond measures 80 pt. The middle angle is approximately 106.26°. Source IDs 81, 7 and 91 map in encounter order to model atoms 1, 2 and 3. Both endpoints omit Element. No Node owns a label. [Public sources and admission contract](../../../../../docs/research/cdx-skeletal.md).

| Files | What they distinguish |
| --- | --- |
| `propane`, `missing_h`, `all_h_missing` | Identical geometry with literal attached counts 3/2/3, an omitted central H, or all H omitted. The portions absent in CDX stay absent in the stored document. |
| `dormant_true`, `dormant_nonterminal` | Zero-length true terminal flag on the middle atom, or true nonterminal flag on an endpoint. The other degree class still controls visible ink. |
| `terminal_visible`, `interior_visible`, `malformed_false`, `duplicate_flag`, `malformed_dormant`, `duplicate_dormant` | Active true, a byte pretending to encode false, and duplicate present properties. The last two fail even though the flag is dormant. |
| `root_flag`, `custom_root`, `custom_node`, `child_text`, `missing_width` | Source ownership, opaque data, an authored child or unknown physical ink width. |
| `charge`, `isotope`, `radical`, `heteroatom`, `inconsistent_h`, `collinear`, `ethane`, `off_paper` | Chemistry, mandatory visible glyphs, and paper-boundary controls. |

Run the byte generator and the independent chemical reader from the repository root.

```sh
python3 -B crates/io/tests/fixtures/cdx-skeletal/generate.py
.cache/oracles/format-python/bin/python -B crates/io/tests/fixtures/cdx-skeletal/check_rdkit.py
```

Pass `--output-dir DIR` to regenerate. RDKit 2026.09.2 reads eight binary witnesses; for the all-omitted source it reports zero explicitly attached H on all three atoms and totals of 3/2/3, with formula C3H8 and SMILES CCC. RDKit is an external test tool. The separate optical control is a same-geometry CDXML source with both flags set explicitly to no. At centered 100% and 512 × 512, pixel cells (191,216), (255,215), (175,228) and (272,228) lie wholly inside the two 4 pt bond strips. The original 283-byte `propane.cdx` has SHA-256 `d000b90042e8e9d4aff3f291cb062fae560b6447f0cd60bbece2135050e31ce0`.
