# Original empty CDX Poster caption fixtures

The generator and fixtures are original Bondline material licensed MIT OR Apache-2.0.
Python `struct` constructs the bytes. The repository's independent resource oracle,
SHA-256 `a5a2df85a9fc1a2df9ba57b1a7b5538c812a6f724e18d25a42a16d98d6ad5db2`, reads back
the exact framing, owners, field widths and resources. `cases.json` records the pinned
file hashes and complete model expectations generated directly from this specification.

Each source has one empty direct Page 90 with explicit Poster mode and positive signed
16.16 dimensions 512.25 × 384.5 pt. The Height-first field order and extended Height and
FontTable lengths exercise byte preservation. The public Page format permits zero
children and direct Text. An omitted DrawingSpace chooses Pages. The source keeps the
explicit dimensions that define the admitted physical paper. [Page object](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Page.htm), [DrawingSpace](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/DrawingSpaceType.htm), [Width](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Width.htm), [Height](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Height.htm), [coordinate encoding](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/DataType/CDXCoordinates.htm)

The five main cases use one UTF-8 Roboto entry with matching selector and ordinal zero.
The alternate case selects Roboto at ordinal and ID one. Its root palette occurs after
the Page and has a single white background entry. The other five receive the documented
white background default from the absent palette. The caption always writes reserved
black index zero. Origin platform 1 and coincident selectors are Bondline's established
UTF-8 profile. [Font table](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/DataType/CDXFontTable.htm), [styled string](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/DataType/CDXString.htm), [code page 65001](https://learn.microsoft.com/en-us/windows/win32/intl/code-page-identifiers), [color table](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/ColorTable.htm)

The new Text uses raw ID 91, explicit signed Z zero, 12 pt Roboto Regular, and text
`Phase β & q` at baseline (144.25, 216.75) pt. The SDK uses zero for omitted Z and permits
omission where nothing overlaps. Bondline writes explicit zero as its first-object rule.
The persistent ObjectTag gives live ID 1 for the first creation and ID 2 for another
creation after Undo in the same session. Undo itself returns the exact `before` bytes.
Deleting the inserted 112-byte interval restores all root, Page and file bytes.
[ZOrder](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/ZOrder.htm), [ObjectTag](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/ObjectTag.htm), [existing caption creation contract](../../../../../docs/research/cdx-caption-creation.md)

| Witness | Before bytes | Page end offset | Each created file |
| --- | ---: | ---: | ---: |
| `implicit` | 87 | 83 | 199 |
| `explicit` | 93 | 89 | 205 |
| `header` | 89 | 83 | 201 |
| `implicit_file_end` | 89 | 83 | 201 |
| `explicit_file_end` | 95 | 89 | 207 |
| `ordinal_one_white` | 110 | 94 | 222 |

All controls parse in the standalone oracle. They cover missing, empty, misowned,
duplicate or incompatible fonts, a missing or nonwhite palette entry, missing or
invalid dimensions, absent Poster mode, opaque payloads and missing or extra Pages.
`expected_open` records the legacy elemental data fallback for these single-source
shapes and the old Open refusal for two Pages. Runtime admission, capture, history and
Save evidence come from the receiving application tests.

Reproduce and check the vectors in place from the repository root.

```sh
python3 -B crates/io/tests/fixtures/cdx-caption-empty/generate.py
```
