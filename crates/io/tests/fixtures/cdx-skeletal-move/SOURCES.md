# Skeletal atom translation witnesses

These are original Bondline fixtures under MIT OR Apache-2.0. The Python standard library reader obtains each Node's directly owned position payload. Fraction arithmetic predicts both signed words and retains complete input framing. Regenerate the files with the following command from the repository root.

```sh
python3 -B crates/io/tests/fixtures/cdx-skeletal-move/generate.py
```

The five `*_center.cdx` cases move only live atom 2 (+8.25, −5.125 pt). Each has the same byte length as the matching input in `../cdx-skeletal/`. Their proportions of omitted explicit H and raw presence of dormant terminal/nonterminal flags differ. The 425-byte `aliased_extended.cdx` duplicates a translated Fragment with raw Fragment IDs 901 and 902, child Node IDs 81, 7 and 91 in each, plus two deliberately long-form 8-byte position records. The expected `aliased_moved.cdx` moves live atoms 2 and 4 (−5.125, +3.25 pt). The owning source identities are (901,7) and (902,81). [Public coordinate and movement decisions](../../../../../docs/research/cdx-skeletal-movement.md)

The `odd_position` source offsets one of those targets by exactly one raw grid unit. Quantizing (+1.5, −1.5) raw units gives a common (+2, −2) vector, yielding `odd_position_halves.cdx`. A half-unit displacement alone rounds to zero even at the odd coordinate.
