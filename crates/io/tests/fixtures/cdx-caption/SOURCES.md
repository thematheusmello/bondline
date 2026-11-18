# Original CDX caption witnesses

Original Bondline fixture files and Python scripts, MIT OR Apache-2.0. The
[preview contract](../../../../../docs/research/cdx-caption-preview.md) records
the public field definitions and the bounded profile for these originals.

The stdlib struct generator declares 12 proposed positive models, including
five framings, a reversed-field pair with equal models, fractional font size,
Greek text, raw RGB16, and a selected font at matching ID and ordinal one.
The independent reader checks actual framing, original resource records,
all field owners and widths, and complete expected models. The 63 valid-raw
refusals cover missing, duplicate, ambiguous and extra data, including 28
countermodels that would equal the base if unsupported fields were ignored.
The checker also exercises six invalid outer structures and nine changed
models beside an equal clone.

The reader reuses the original resource oracle from commit dce5e0f, pinned to
SHA-256 `a5a2df85a9fc1a2df9ba57b1a7b5538c812a6f724e18d25a42a16d98d6ad5db2`.
The `nominal_model` on a refusal represents a derived counterexample.
The `model` on a proposed positive is the complete expected projection.

`legacy-open.json` separately records the actual pre-preview CLI result for
each source. All 75 were refused with `unsupported_content`. The input hashes
matched the independent manifest before and after every Open, and the fixture
set records the precise old diagnostics for the 63 retained refusals.

Formatting preserved all 75 binaries, the manifest, model facts and drift
witnesses. The check receipt changed only the generator and reader hashes.
Run the generator and checker from the repository root:

```sh
fixtures=crates/io/tests/fixtures/cdx-caption
output=$(mktemp -d)
python3 -B "$fixtures/generate_captions.py" --output-dir "$output/sources"
python3 -B "$fixtures/check_captions.py" --input-dir "$output/sources" --output-dir "$output/facts"
```

Manifest SHA-256 is `a9a7d7b429312ebe488f7efcc1b0345e4c0cf656c1bb180446014ff4b7aa6fc0`.
Fact SHA-256 is `e7ab8cbfcff49f2b6397371984889321337a3e8c875641ce25e3b58dbe8a6577`.
The legacy baseline SHA-256 is `55862eaea4c6e50cfbe4aea815233ba48f16d87805a4d0755b3c6faba4a45910`.
