# Independent CDX verification

These original test helpers are MIT OR Apache-2.0. They are not product dependencies and
do not implement application CDX import/export or a chemistry engine.

Use the existing isolated interpreter described in `docs/development.md`:

```sh
export BONDLINE_PYTHON="$PWD/.cache/oracles/format-python/bin/python"
"$BONDLINE_PYTHON" -B xtask/oracles/cdx_oracle.py self-test
cargo test -p bondline-cdx --test independent --locked
```

Tests require the external `rdkit==2026.9.2` already pinned in `requirements-dev.txt`.
They assert the actual `rdBase.rdkitVersion == "2026.09.2"` and
`Chem.HasChemDrawCDXSupport()` instead of silently skipping an unavailable reader. They
run offline and need no Java or newly downloaded artifact. `-B` prevents test bytecode caches.

## Original fixture and independent expectations

`cdx_oracle.py fixture` emits an original 65,744-byte implicit-root document. Adding
`explicit` emits its 65,750-byte explicit-document equivalent. Neither is copied from a
vendor sample, downloaded corpus, or another reader's writer. The Python authoring logic
and fixture bytes are project-original under MIT OR Apache-2.0.

The scene contains page 42, fragment 100, carbon nodes 101/102, and bond 103 referencing
those nodes. Coordinates in document points are (10.25, −3.5) and (26.25, 8.5). The bond
order starts at the single-bond bit `0x0001`; the property edit replaces it with double-bond
bit `0x0002`. Independent chemical expectations are CC/C2H6 and C=C/C2H4, two carbons,
one bond, and the same endpoints. Unknown binary properties exist on root, page, node and
bond. Nested user-defined objects and a 65,535-byte opaque extended property exercise
ownership, unknown preservation, and record boundaries.

Numeric format sources are the public archived SDK:

- [Binary header, tagged records and lengths](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/IntroCDX.htm),
  [simple framing example](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/IntroExampleSimple.htm),
  and the [object inventory](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/AllCDXObjects.htm).
- [Coordinate representation](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/DataType/CDXCoordinates.htm)
  and [2D position](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/2DPosition.htm):
  signed 16.16 points, binary y before x.
- [Element](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Node_Element.htm),
  [bond begin](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Bond_Begin.htm),
  [bond end](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Bond_End.htm), and
  [bond-order bit mask](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Bond_Order.htm).

The SDK simple example has inconsistent atom/bond IDs; its hex data was not adopted as a
chemical oracle. The project-original scene has consistent references. All four complete
identities are asserted in the oracle self-test:

| Framing / order | Bytes | SHA-256 |
|---|---:|---|
| Implicit / single | 65744 | `6e3228ecc2791366c1f23be8e4dd411f7f1fb4c3250b239fdfb812aa698df1f0` |
| Implicit / double | 65744 | `88b2ac7f0a4028323224647d16eadafca4c022d043010e5da94c46b2ad28a1f1` |
| Explicit / single | 65750 | `63ef1a74133e45de915d5fd25b2ca3321a95dbe5114873a5a59f2f45217000d2` |
| Explicit / double | 65750 | `1a183a24e21619084c16c4a39c7a803ad1c73dd3e6ac067dd355cd9122fd4774` |

The standard-library `struct` oracle decodes raw records independently of Rust. It checks
all event ranges, payload ranges, short/extended encodings, parent ownership and matching
ends. Edit tests compare every output byte to an independent splice of the fixed original;
no unrelated data may change. Payload sizes 0, 1, 32767, 32768, 65534, 65535 and 65536
exercise both sides of the length sentinel. Identical and foreign record-handle tests
cover no-op fidelity and document ownership. Its 8 MiB input, 100,000-event and 256-depth
limits are test-oracle limits, not assertions about all valid CDX.

## External reader provenance and limits

The installed RDKit API accepts binary bytes through `MolsFromCDXML` with
`CDXMLParserParams.format = CDXMLFormat.CDX`. This is a separate format-support path.
The public [RDKit 2026.09.2 build recipe](https://github.com/rdkit/rdkit/blob/Release_2026_09_2/External/ChemDraw/CMakeLists.txt)
selects the public Glysade file-format parser v1.0.14. Its tag resolves to commit
`5dd743230d75ecacacdf8dcd0b0e022f13173b9c`.
The [README](https://github.com/Glysade/chemdraw/blob/5dd743230d75ecacacdf8dcd0b0e022f13173b9c/README.md)
describes the publicly released parser and modifications; the
[actual parser license](https://github.com/Glysade/chemdraw/blob/5dd743230d75ecacacdf8dcd0b0e022f13173b9c/chemdraw/license.txt)
is BSD-3-Clause, copyright 1986–2025 Revvity, CambridgeSoft and others. Exact downloaded
README/license SHA-256 values are respectively
`68d5720c7b6a61e5144a1e4f1eafb892867b6962058e6490f1e15c62f5064200` and
`20060cab135e99992dc5adc0aec48f2d8183f39191e7be4666510c6aba228941`.
These metadata files remain ignored; no upstream parser implementation was copied into
Bondline. The release recipe is not attestation that an installed wheel used that exact
source revision or a complete audit of the wheel's native dependency closure.

The locally observed environment is Python 3.13, macOS arm64, RDKit 2026.09.2;
`rdmolfiles.so` is 1,056,208 bytes, SHA-256
`d60f9cf61091f69387d8393fc179b11119c3914078c77648da966902eab0e95a`.
This identifies that extension only, not the complete wheel or delegated parser library.
Other platforms must run the same version/API assertions; they are not represented by this
Mac extension hash.

Observed limitations are explicit:

- RDKit conformers retain the CDX raw 16.16 integer magnitude and reverse y for these
  fixtures. The test asserts that exact external-reader mapping. Document-point positions
  are established separately with `struct`; no implicit Angstrom/point scaling is claimed.
- RDKit accepts both single-terminator implicit and explicit documents. A missing implicit
  root terminator raises a parsing error. It also accepts extra trailing zero terminators
  in both forms. Bondline's exact-EOF rejection is its documented strictness policy, not
  something this permissive consumer proves necessary.
- Optional OpenChemLib 2026.9.0 was inspected externally and is not required by these
  tests. Its POM says BSD-3-Clause, but its CDXML parser carries Jmol LGPL-2.1-or-later
  notices. It starts decoding at byte 22, mishandles signed short lengths 32768–65534, and
  rounds/normalizes coordinates. Downloaded Java archives/diagnostics stay ignored and are
  never shipped, linked into the application, copied, or required in CI.

This verifies two original structures, their opaque binary backing and one bond property
edit. It does not verify all properties, chemistry/stereochemistry, arbitrary scoped IDs,
reactions, text/fonts, native application interoperability, full corpus coverage, or an
application CDX Open/Save workflow.

## RED / GREEN evidence

The independent integration target first failed compilation with E0599 because the
promised `Document::replace_property` API did not exist. Exact command above; saved log:
`target/cdx-independent-red.log`. The independent oracle self-test and direct external
ethane/ethene and framing probes passed separately before the implementation.
After property editing landed, the same integration target still failed the explicit-root
fixture with `Truncated { offset: 65750 }`; the opaque-length/ownership test passed. That
framing RED is retained in `target/cdx-independent-first-integration.log`.

After explicit framing landed, both integration tests passed, including all four external
RDKit reads and seven binary-edit boundary cases. Log: `target/cdx-independent-green.log`.
`cargo clippy -p bondline-cdx --all-targets --locked -- -D warnings` passed in
`target/cdx-independent-clippy.log`; targeted rustfmt and the pinned oracle self-test also
passed. This was local macOS arm64 execution. Hosted Windows/Linux consumer behavior and
full-workspace gates are separate integration checks, not implied by this result. No
parser production source, root manifest, CI file, or GUI was changed by this independent
verification task. No Git operation was performed.

## Windows text-output correction

Hosted run43475705005 passed Mac/Linux and both Linux sanitizer targets, but Windows
failed both independent tests at record-table comparison before the external chemical
reader ran. Python text stdout translated LF to CRLF. The describe response now writes
ASCII bytes directly through sys.stdout.buffer; binary fixtures and CDX assertions stay
byte-exact. A regression injects a translating CRLF TextIOWrapper on any host, asserts
that fixture bytes are unchanged and compares the entire canonical LF record table. It
failed before the fix (target/cdx-stdout-red.log) and now passes with all14 CDX tests
(cdx-stdout-green.log); package clippy also passes (cdx-stdout-clippy.log). This correction
does not establish the new Windows hosted result; that rerun must be observed separately.

## Header-embedded Document framing, 2026-11-14

`cdx_header_oracle.py` reuses only Bondline's existing original scene generator and
field constants. The old oracle, hashes and strict scene guards are unchanged. The
new helper creates a 22-byte prefix followed by Document8000/ID0, its object end and
one separate file end. Its independent bounded `struct` decoder starts at22 and
distinguishes that final file event; binary stdout retains the exact LF protocol.
The source facts and fixture identities are in the
[fixture provenance](../../crates/cdx/tests/fixtures/README.md).

Pinned public `CDXIO.cpp` is25,597 bytes, SHA-256
`71a2942f44dd6b875aee72d97c1101b874dc743d5f8f121f61f9877f3000a30b`;
`CDXDocument.cpp` is53,887 bytes, SHA-256
`dceab68450961a57f6099ba3f981dc4ce18390211ea23930754f550a2f1ec004`.
Both are at the Glysade commit already qualified above and carry BSD-3-Clause
notices. Downloads are ignored research evidence; no upstream implementation is
shipped, linked, translated or copied into Bondline's original Rust parser.

The helper pins the65,746-byte single and double vectors by SHA, checks every event
and tests strict missing/extra-end refusal independently of Rust. Direct pinned
RDKit reads verify the same specific graph/formula/order/raw-coordinate expectations
as the earlier scenes. That external consumer's tolerant EOF does not establish
Bondline's stricter framing policy. A successful legacy28-byte interpretation takes
precedence in Rust: an otherwise complete legacy tree whose reserved bytes happen
to contain Document8000 remains legacy, even when it could also be a truncated
native envelope. No universal identification of such ambiguous files is claimed.

The first Rust test reached `TrailingData` only after actual independent fixture and
RDKit checks passed (`target/cdx-header-red.log`). Seven framing tests then passed;
the package and bounded sanitizer results are recorded in docs/verification.md.
The local native1196-byte observation stays ignored and distinct from this original
redistributable vector. Its successful syntax passthrough is not application CDX I/O.
