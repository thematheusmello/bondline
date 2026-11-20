# Pinned public corpus

## Molecular formula evidence

The full local integration gate at `018cf0b` passed 2,407 workspace tests and three additional macOS AccessKit tests. One workspace test was ignored. Source and record counts below were observed in that gate.

The checker opens each verified source through Bondline's command engine and reads `analysis.formula`. A separate RDKit 2026.09.2 process reads the same file bytes and calculates the reference formula. Each positive comparison checks the complete split-isotope Hill formula, charge and number of drawn atoms. The oracle retains physical hydrogen atoms, spells numbered H explicitly, and reorders carbon-free element groups alphabetically for strict Hill notation. The tests exercise isotope loss, hydrogen-first ordering in hydrogen bromide and a wrong atom count. [Reference reader](../../xtask/oracles/corpus_formula.py) · [Registry comparison](../../xtask/src/corpus/formula.rs) · [Hill convention](../research/hill-formula.md).

| Pinned file | Bondline result on the complete source | Independent formula and per-record evidence |
| --- | --- | --- |
| `rdkit-chain1.cdxml` | Unsupported source | C4H10 |
| `rdkit-methyl-radical.mol` | CH3 matched | CH3, one drawn atom |
| `bondline-ordered-fields-v2000.sdf` | Unsupported collection | The open collection returned C2H6 for records 1 and 2, two drawn atoms each. Both isolated MOL blocks independently reproduced the same results. The separate whole-supplier diagnostic returned one entry. |
| `bondline-original-ethane-explicit.cdx` | Unsupported source | C2H6 |
| `bondline-original-noncontiguous-v3000.mol` | C2H6 matched | C2H6, two drawn atoms |
| `bondline-original-isotopic-chain.smi` | Unsupported collection | The record import refuses isotopic drawing labels. RDKit gives C[13C]H6O. |
| `bondline-original-two-ring-labels.smi` | Unsupported collection | The record imports, and Analysis reports unsupported source. RDKit gives the combined C7H14. |
| `bondline-original-salt-branch.smi` | Unsupported collection | The record import refuses this charged structure. RDKit gives the combined C4H9NaO. |

Two of eight complete files returned a matching formula. Two of five record checks matched and three were unavailable. The SDF record count uses the result of querying the intact, SHA-256-verified collection through `analysis.properties`. Each extracted MOL block is checked separately against the same RDKit result. The two paths count each record once. The raw collection has its own reported outcome. SMILES component groups stay together within their original record. The printed reference for an unavailable result comes solely from RDKit. The corpus still needs independent comparisons for mass, aromaticity, stereo labels, canonical SMILES, InChI and names.

Run `cargo xtask corpus --offline` after fetching the pinned files and setting `BONDLINE_PYTHON` to the RDKit interpreter. [Official molecular readers](https://www.rdkit.org/docs/source/rdkit.Chem.rdmolfiles.html) · [RDKit formula descriptor](https://www.rdkit.org/docs/source/rdkit.Chem.rdMolDescriptors.html#rdkit.Chem.rdMolDescriptors.CalcMolFormula).

## Source bundles

`cargo xtask corpus` checks eight reviewed files from five mandatory version1 source bundles:

- [manifest.json](manifest.json) retains the two RDKit documents and their original pinned
  revision unchanged. Its checked-in BSD-3-Clause notice applies to those files.
- [bondline-manifest.json](bondline-manifest.json) pins one original MIT OR Apache-2.0 SDF
  vector from published Bondline commit `fb86832a0c9b7744fb3ef8f8115c7ed296164505`.
  `LICENSE-MIT` supplies the applicable MIT notice; both original project licenses remain.
- [bondline-cdx-manifest.json](bondline-cdx-manifest.json) pins the original explicit-root
  chemical CDX vector from published Bondline commit `30ae30e73a4ee3b68d41069376fc5690faa99bc4`.
  It retains the original MIT OR Apache-2.0 declaration and the applicable MIT notice.
  A separate revision bundle leaves both earlier source pins unchanged.
- [bondline-v3000-manifest.json](bondline-v3000-manifest.json) pins the original344-byte
  V3000 vector from published Bondline commit `5caf6ce0ecf40badf2666e36dcb73dc8c4337b51`.
  Its explicit `mol_v3000` format tag selects the standalone V3000 codec; existing `mol`
  entries continue to select V2000. The original MIT OR Apache-2.0 declaration and MIT
  notice remain. Multiple bundles may come from the same repository.
- [bondline-smiles-manifest.json](bondline-smiles-manifest.json) pins three original
  MIT OR Apache-2.0 notation seeds from published Bondline commit
  `eabe02f267985b2788496b58f96415d7fa4d1a4d`. Exact17/21/21-byte ASCII/LF data was fetched
  and verified; the retained MIT notice matches its previously fetched public blob. Explicit `smiles`
  format tags select the bounded notation/source checker; all four older pins remain exact.

Each entry records source path, exact byte length, SHA-256 and provenance. Proprietary
sample/template libraries, application binaries, fonts and icons are excluded.

All manifests and their confined, nonempty notices are validated before cache or network
I/O. Every bundle is mandatory. The reviewed manifest path list is explicit in xtask;
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
| bondline-original-ethane-explicit.cdx | Exact 65,750 bytes and SHA-256 | Original explicit scene; byte-exact parser passthrough and all29 events match independent struct; external RDKit verifies ethane | Bondline calculations missing |
| bondline-original-noncontiguous-v3000.mol | Exact344 bytes and SHA-256 | Original V3000 vector; byte-exact source, raw sparse IDs/fields and dense elemental projection agree with independent literal/Decimal expectations and external RDKit | Bondline calculations missing |
| bondline-original-isotopic-chain.smi | Exact17 bytes and SHA-256 | Source passthrough; literal C/C/O adjacency, isotope13 and bracket H3/H2/H1 agree with qualified external RDKit | Calculations/canonicalization missing |
| bondline-original-two-ring-labels.smi | Exact21 bytes and SHA-256 | Source passthrough; two separate carbon rings, distinct73/74 declarations/ranges and unresolved bare-H tokens agree with literal graph and external reader | Calculations/canonicalization missing |
| bondline-original-salt-branch.smi | Exact21 bytes and SHA-256 | Source passthrough; Na+1/O-1/bracket CH2/bare-C branch and both chains agree with literal graph and external reader | Calculations/canonicalization missing |

The command succeeds for the implemented CDXML, single-input MOL/SDF V2000, original
MOL V3000/CDX field-vector subsets and the three original nonaromatic SMILES notations.
It explicitly prints missing foreign/general/application CDX
compatibility and Bondline chemistry checks; it does not certify M1, general MOL
compatibility or all-format round trips. A manifest without any implemented
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

The V2000 MOL check covers original-byte passthrough and read-only model projection.
It does not test the writer, edited-source export, application import/export or chemistry
calculations. SDF and V3000 coverage have their separate rows and limits. The RDKit manifest
remains the same two reviewed files.

The fetched V3000 input is the original two-carbon vector with source atom IDs10/40 and
bond ID7. Its source remains byte-exact; an independent bounded Python literal/Decimal
reader checks the header, coordinates, VAL=4, source IDs/endpoints, and dense model
IDs1/2/bond1. A separately invoked, unchanged [external V3000 oracle](../../crates/io/tests/v3000_oracle.py)
requires RDKit2026.09.2 and checks the raw two-carbon/single-bond graph, explicit H3,
charges, isotopes, query/stereo absence and coordinates. Neither comparison uses our writer.
The authored H value is the published declared-valence arithmetic4-1, not perceived normal valence.

The explicit corpus transform is24points/unit, origin(0,0), inverted Y; positions are
(-30,-12) and(6,-12) points. These vector values are exactly representable in binary64;
literal/model comparison is exact. No physical file unit or application default is inferred.
Negative checks reject changed raw source IDs/endpoints/VAL/coordinates and model
IDs/element/H/radical/bond/coordinates, malformed/extended input, a different valid bond graph,
and the wrong declared source version. A V3000-only test must also count as real parser
coverage; missing or corrupt offline V3000 bytes fail. This is one original vector, not
general V3000, edited-file/writer or application compatibility, chemistry calculations,
or M1 completion. Additional V3000 entries require their own independent expectations.

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

The fetched CDX input is the original 65,750-byte explicit-root ethane scene, not a foreign
file or reference-application output. Both Rust source views and reserialized bytes must
equal the downloaded bytes. The existing independent Python `struct` decoder checks every
record range, payload span, short/extended length, parent and matched end. Its separate,
unchanged RDKit `verify 1 explicit` guard requires the exact original scene, including opaque
payloads, then checks CC/C2H6, two carbons, one single bond and the known raw-coordinate
mapping. It never treats an empty molecular result as a chemical pass. Negative tests reject
altered record spans, opaque payloads, a valid double-bond graph and a different explicit
scene without weakening that guard.

RDKit's CDX conformer values retain raw16.16 magnitudes and reverse y for this vector;
document-point coordinates are independently checked with `struct`. No inferred physical
units, Bondline chemistry algorithm, application CDX Open/Save or general CDX compatibility
is established. See [fixture provenance](../../crates/cdx/tests/fixtures/README.md) and
[external-reader provenance/limits](../../xtask/oracles/cdx-README.md). The original70-byte
empty syntax fixture and generated-scene edits remain separate standalone regressions.

The fifth bundle supplies source-retention and notation evidence. Its checker compares
every atom's element/isotope/charge and bracket-versus-implicit H, sorted edges/orders,
numeric ring identities/ranges, notation end and chain starts against independent literal
expectations for the exact three original inputs. Bare H stays unresolved; notation-chain
starts are not substituted for a general connected-component algorithm. Changed valid
vectors, LF/CRLF substitutions, wrong metadata/edges and oversized/unsupported input refuse.
Adding a different SMILES source requires its own independent expectations.

The unchanged [external SMILES consumer](../../crates/io/tests/smiles_oracle.py) receives
the complete accepted bytes and independently returns source text/length/SHA-256 plus
atom/bond data. RDKit2026.09.2 explicitly has sanitize/removeHs/parseName/allowCXSMILES
false. Its Python binding exposes no skipCleanup switch, and numeric getters collapse
some source distinctions; literal source/H metadata remains the raw oracle. No implicit
H, radicals, chemical validity, canonical output or coordinates are inferred. The printed
missing chemistry/SMILES line concerns calculations and generated/canonical chemistry,
separate from this source/notation count. See
[fifth-bundle verification](../verification-smiles-corpus.md).

All helpers supply five mandatory bundles. The default fetched/offline gate reports8 files
and5 bundles. A deliberately CDXML-free offline unit retains its narrower scope and reports7
files/5 bundles; a separate all-SMILES synthetic manifest case proves that real SMILES
parsing counts toward implemented coverage even when other formats have zero checks. Every
existing manifest/license/notice/name/count/confinement/hash negative remains in force.
