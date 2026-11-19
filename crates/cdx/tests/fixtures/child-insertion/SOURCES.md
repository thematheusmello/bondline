# Original CDX child insertion vectors

Bondline authored these binary fixtures and their generator under MIT OR Apache-2.0.
The generator uses Python `struct`. The existing independent resource oracle reads the
bytes back and verifies the framing, offsets, owners, and matched ends. The SDK defines
the little-endian fields, tag classes, short and extended lengths, object terminators,
separate file end, scoped IDs, and unassigned ID zero. It recommends document-wide
uniqueness for writing references. These vectors use no object references. The five
envelopes match the parser's existing compatibility profiles. [Public CDX format SDK](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/IntroCDX.htm)

The original tree has Page 55, a Group 19 containing custom child `C131` with ID 74,
and direct Graphic 73. A root property and a second Page 56 follow Page 55. The originals
have opaque payloads that include zero pairs and small properties using extended lengths.
The edited tree adds one direct child `C132` with ID 74 as the last child of Page 55. Its
two properties have payloads `0012` and empty, respectively. Removing the 18-byte addition
returns the exact original. The relative scope of each ID 74 is different.

| Envelope | Page 55 insertion offset | Before bytes | After bytes |
| --- | ---: | ---: | ---: |
| Implicit Document | 87 | 110 | 128 |
| Explicit Document | 93 | 116 | 134 |
| Header-embedded Document | 87 | 112 | 130 |
| Implicit Document with file end | 87 | 112 | 130 |
| Explicit Document with file end | 93 | 118 | 136 |

`duplicate_direct_id.cdx` has the new child with ID 73, shared with a direct sibling.
`after_page_end.cdx` has the ID 74 addition two bytes later, as a root child. Both
files parse under the independent syntax reader. Their records establish the ownership
errors the insertion operation must prevent. Rust tests also construct exact binary
controls for foreign handles, anonymous siblings, tag classes, caller property order,
the length threshold, and the depth, record, and byte limits.

Run `python3 -B crates/cdx/tests/fixtures/child-insertion/generate.py` from the repository
root to recreate the vectors in memory, compare every byte, and print the checksums.
Pass `--output-dir` with a scratch directory to reproduce the files there.

| File | SHA-256 |
| --- | --- |
| `implicit_before.cdx` | `e4e2aa5d4896b40e517be1100fc09f2a8e83c8311119e5fb6e7b14772dee21e6` |
| `implicit_after.cdx` | `d3d3b3f8708bb1a62c4e22b7045c271b2c4dd9b8b8babeb3ea06998cf937092b` |
| `explicit_before.cdx` | `b8bfe5e46d2b8bc814abf3da1055707a976c44e1fcab17431a2e1a1cd2ae235f` |
| `explicit_after.cdx` | `0e4407649526d3e37fecd3aa3aea3e3451c41072dee95c40366b9d009228d188` |
| `header_before.cdx` | `01a6a3bd0b2d71191b603ae652048a1d39204c27aec0342f295bfd109e088e55` |
| `header_after.cdx` | `da9f3b718927bbdfeed6d01a207068d826682f5f48d135e727101e5003f3bbf5` |
| `implicit_file_end_before.cdx` | `9a47118c06f46f2f724e194b2eb3a406968626d4dbed89edd27b408a6245fd58` |
| `implicit_file_end_after.cdx` | `bcc106bd1af9ebd35e35490670cdd84493a69c3abf42d002acd9838b0bb8e30b` |
| `explicit_file_end_before.cdx` | `528f218ccbc2ce51e3b77539c2583d716869c8ee2f550757d3b1d103b79cbd72` |
| `explicit_file_end_after.cdx` | `015baacb227a31fc3a4a4f14deac346087b503708f31ce691813c37f6f3f72f2` |
| `duplicate_direct_id.cdx` | `3413122bf71017c77c7ab8e4978d4125f38e4f14d2b9266f3e6129ba5f97d6d3` |
| `after_page_end.cdx` | `17e03d923e87d0ba6f06048a6bea29986fe3846eb647428888c63eed8cde5263` |
