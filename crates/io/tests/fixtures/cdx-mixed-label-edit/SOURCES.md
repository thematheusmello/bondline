# Existing chemical labels beside CDX Page captions

These original binary fixtures and their generator are MIT OR Apache-2.0. The generator uses the [existing Python record and FontTools label oracle](../cdx-label-edit/generate_edits.py). It rechecks each non-target record byte for byte, including length words, style runs, direct Page Text and its persistent ID marker. The [source contract](../../../../../docs/research/cdx-mixed-label-edits.md) cites the public format documentation.

| Starting file | Operation and independent witness |
| --- | --- |
| `cdx-chemical-caption-create/chemical_first.cdx` | First existing caption beside chemical ink. The three `chemical_first_*` outputs change NH2→OH, Cl→Br, or both. |
| `cdx-chemical-caption-create/mixed_first.cdx` | Raw 73 belongs to each of two Node children and the Page caption with live ID 19. `mixed_first_oh` and `mixed_first_both` change the first or both chemical labels. |
| `cdx-mixed-caption/overlap_before.cdx` | `overlap_before_oh` retains the front Page Text recorded before the two Fragment objects. |
| `cdx-mixed-caption-edit/extended_before.cdx` and `after_prefix.cdx` | `extended_br` preserves the original extended Page string length. Editing that run afterward produces the same `prefix_br` as changing it before the Node label. |
| `cdx-mixed-caption/water_after.cdx` | `water_after_nh` retains both directed bonds, the Page caption and its maximum UINT32 raw ID. |
| `depth_max_before.cdx` | The foreground caption is already at signed depth 32767. `depth_max_oh` replaces the atom label while retaining the exhausted depth. |
| `edge_cl_before.cdx` | The Cl label lies within the paper. The modeled Mg edit in `edge_mg_off_paper.cdx` would have a negative left bound. It is a binary refusal witness. |

Replay the checked-in files with the pinned FontTools 4.61.1 environment:

```sh
python3 -B crates/io/tests/fixtures/cdx-mixed-label-edit/generate.py
```

Pass `--output-dir DIR` to regenerate. Pass `--rdkit` with external RDKit 2026.09.2 and FontTools available to verify the atomic numbers, attached H counts, and directed bonds in all fixtures, including the off-paper graph. Rust tests independently compare exact published bytes and complete re-imported models. No RDKit or Python code is linked into Bondline.
