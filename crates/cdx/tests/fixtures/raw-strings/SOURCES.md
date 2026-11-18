# Original CDXString fixtures

Original Bondline test data and stdlib Python scripts, MIT OR Apache-2.0. The
[public CDXString layout](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/DataType/CDXString.htm)
defines the count and ten-byte runs. The
[binary framing specification](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/IntroCDX.htm)
defines little-endian words and property lengths. See the
[raw string contract](../../../../../docs/research/cdx-string.md) for decisions
about opaque fields and separately owned records.

`generate_strings.py` builds original files with Python struct.
`string_oracle.py` independently reads file and property boundaries, run words and
text bytes. The checker qualifies that reader on fixed literal hex and a
one-byte-short run. It checks source SHA-256, all 131,070 entries across both
maximum-count cases and every query error. The run-start fields stay literal.
The UTF-8 byte witness distinguishes eight bytes, four scalars and five UTF-16
code units without assigning those units to the CDX start field.

`data/string-vectors.json` pins 32 originals, including 28 small binary files
totaling 3,689 bytes and four generated at runtime. `data/string-facts.json` freezes
complete source framing, property and query facts. It uses boundary samples and
full span digests for large run blocks. `data/handle-controls.json` records foreign
and nonproperty handle refusals. `data/independent-checks.json` gives the counts
and reader qualification. The Rust tests replay the same original data with the
independent reader and check all run words, ranges and borrowing.

From the repository root, reproduce with Python 3.10 or later:

```sh
fixtures=crates/cdx/tests/fixtures/raw-strings
output=$(mktemp -d)
python3 -B "$fixtures/generate_strings.py" --output "$output"
python3 -B "$fixtures/check_strings.py" --source-dir "$output"
diff -qr "$fixtures/data" "$output"
```

The manifest SHA-256 is
`1d742fb1d73512ea01c52712576171c79a306b2a3f1bf6638089f67eaf81bd26`.
The facts SHA-256 is
`236018a8a5cdbca903ca243748f3048adf8e92f935371bfb405b203d2242bf94`.
