# Pinned public corpus

`cargo xtask corpus` fetches only reviewed files from two mandatory version1 source bundles:

- [manifest.json](manifest.json) retains the two RDKit documents and their original pinned
  revision unchanged. Its checked-in BSD-3-Clause notice applies to those files.
- [bondline-manifest.json](bondline-manifest.json) pins one original MIT OR Apache-2.0 SDF
  vector from published Bondline commit `fb86832a0c9b7744fb3ef8f8115c7ed296164505`.
  `LICENSE-MIT` supplies the applicable MIT notice; both original project licenses remain.

Each entry records source path, exact byte length, SHA-256 and provenance. Proprietary
sample/template libraries, application binaries, fonts and icons are excluded.

Both manifests and their confined, nonempty notices are validated before cache or network
I/O. Neither bundle is optional. The reviewed manifest path list is explicit in xtask;
each manifest is capped at1MiB and all bundles together at1024 files. The existing8MiB
per-file limit remains. Flat cache names must be unique across bundles, including ASCII
case aliases on case-insensitive filesystems. Adding a source does not change another
bundle's pins, license declarations or existing cached names.

Downloads use an external Python HTTPS process with a 15-second socket timeout and a
read bound of expected length + 1. Rust validates the exact size/hash before staging the
bytes under ignored .cache/corpus. The same verification applies on every cached read;
corrupt cache entries fail rather than being accepted or overwritten. Filenames/source
paths cannot traverse directories; notice/cache paths are checked for symlink escapes.
The existing cache ancestor is confined before creating directories, then the completed
cache path is checked again.
This is developer tooling. The application does not invoke Python or fetch documents.

`cargo xtask corpus --offline` requires a populated valid cache and never fetches. Use
BONDLINE_PYTHON as described in [development.md](../development.md). Fetched copies stay
in the ignored cache and are never silently substituted when a source is inaccessible.
The reviewed MOL vector is also a checked-in regression fixture with
[its source and notice recorded](../../crates/io/tests/fixtures/mol/README.md).
Interrupted .part files can be inspected and removed from this disposable cache before retrying.

Current coverage is deliberately separate:

| Entry | Integrity | Syntax/semantic check | Chemistry |
|---|---|---|---|
| rdkit-chain1.cdxml | Exact 727 bytes and SHA-256 | Byte-preserving syntax and owned-snapshot passthrough; full XML view agrees with Expat; stored model projection agrees with independent Expat/Decimal | Calculations missing |
| rdkit-methyl-radical.mol | Exact 170 bytes and SHA-256 | V2000 parse and exact source passthrough; strict elemental projection agrees with external RDKit/Decimal for one methyl radical | Calculations missing |
| bondline-ordered-fields-v2000.sdf | Exact 690 bytes and SHA-256 | Exact source; two separate MOL records; every ordered field/raw span agrees with an independent scanner; each elemental projection agrees with a direct external RDKit read | Calculations missing |
| CDX | No reviewed entry fetched yet | Corpus coverage missing; standalone syntax/edit tests are separate | Missing |

The command succeeds for the implemented CDXML, single-input MOL and single-input SDF V2000 subsets.
It explicitly prints missing CDX corpus and chemistry checks; it does not certify M1,
general MOL compatibility or all-format round trips. A manifest without any implemented
parser coverage is an error. Extending the corpus requires source/license review, pinned bytes and the matching parser/oracle
checks. Root repository licensing does not authorize copying third-party asset libraries.

The CDXML elemental oracle is original developer-only Python under xtask/oracles. It independently
derives fresh IDs, scoped endpoint references, atom metadata and authored bond kinds from
Expat events and Decimal arithmetic. A deliberate model-property mutation is rejected by
its verifier test; another test checks scopes, direction and explicit metadata. It does
not use our parser/writer or compute chemistry. Source bytes are syntax-validated and
bounded before either external oracle runs; neither fetches DTD/entity resources.
The gate now also prints the missing formula/mass/aromaticity/stereo/SMILES/InChI checks.
Its owned-snapshot count covers unchanged-source export, not edited-file compatibility.

The separate MOL oracle uses pinned external RDKit 2026.09.2 for the raw elemental fields,
explicit H count and one radical electron. Python Decimal independently transforms the
fixed-column coordinates with the test's explicit 24 points/unit, origin (0, 0) and inverted
y axis. This is a chosen test transform, not an inferred file unit or application default.
The resulting carbon is a doublet with three authored hydrogens at (-29.7, -51.6216) points.
Metadata is compared exactly; transformed coordinates permit at most 1e-10 point error for
binary64 rounding. Deliberately changed raw/model coordinates, element, radical and H count
are rejected by verifier tests. Input is parsed, bounded and projected before the external
process; the helper also caps its read and accepts only the current one-atom/no-bond corpus
shape. Adding a different MOL entry requires extending its independent evidence.

MOL checks cover original-byte passthrough and read-only model projection. They do not test
the writer, edited-source export, application import/export, V3000, SDF or chemistry
calculations. The RDKit manifest remains the same two reviewed files. CDX standalone tests do
not substitute for a fetched and independently verified CDX corpus entry.

The original SDF vector has two ethane records with an empty second title and seven ordered
items: empty/multiline values, combined and DT-only identities, duplicate names and literal
spaces. The independent external [SDF oracle](../../crates/io/tests/sdf_oracle.py) checks
complete byte ranges, raw headers, digit spelling, values and order. The corpus also compares
each record's supported elemental projection at24points/unit, origin(0,0), to a separate
RDKit MOL read. This vector has explicitly authored H and no radical-spin ambiguity; it
does not establish general inferred chemistry or all SDF projection cases.

Pinned RDKit2026.09.2's full SDF supplier loses the valid DT-only item and later data/record
in this vector. Its entry count is reported only as a diagnostic, separate from the two
individually consumed molecular blocks and the raw ordered-data proof. No supplier map,
zip or filtering step substitutes for missing fields or records. See the
[original fixture provenance](../../crates/io/tests/fixtures/sdf/README.md). Deliberately
changed duplicate values, dropped/reordered fields, raw ranges and model metadata are rejected
by corpus verifier tests. Exact source retention does not certify edited-file export,
application SDF import/export, V3000, general SDF interoperability or M1 completion.
