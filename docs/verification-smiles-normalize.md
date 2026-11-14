# Opt-in SMILES fixed-H normalization

2026-11-14. Original library-only slice on supplied `feat/smiles-normalization`,
base685a1c8, in `bondline-mol-v3000`. The accepted design report is Root's ignored
`target/smiles-hydrogen-model-next.md`, SHA256
`7583cf06d931d7f4c76c77d4b8b3c7458676d0ea34f6370116c8b5d35ae1311c`.
This deliberately creates fixed counts; it adds no model H-origin/live-inference field.

## Delivered contract

`SmilesDocument::normalize_explicit_h_fragment(&self)` returns a fresh validated
Fragment with local dense IDs and position=None. It neither consumes nor mutates the
original complete source/notation graph. Bare C/N/O/halogens resolve only within the
declared neutral C4/N3/O2/halogen1 profile, and only when integer bond sum does not
exceed the target. Bracket counts remain exact and must satisfy that finite identity.
Physical H requires bracket H0 and one Single bond; its bond contributes to its heavy
neighbor's sum, without being added again to that atom's literal count. Positive u16
isotopes remain exact; isotope0 refuses instead of becoming unspecified mass.

The result materializes Some(n) fixed model counts, just as the existing MOL mapper
materializes counts from explicit VAL. It does not assert that the source contained an
H token. Original raw/IR distinctions remain in the owner: bare/bracket H, omitted/H0,
isotope omission/0, token spans and original line endings. Model None/Some(0), JSON,
MOL VAL, native-v0 encoding/classification, literal-label rendering, bond.extend and
retained-storage semantics stay unchanged. Known count, source constraint and visible
label remain separate concerns; this helper sets no drawing/display policy.

Charged, isotope0, B/P/S/other-element, broader N5, over-target and undercoordinated
bracket contexts return typed unsupported errors. The latter are radical-ambiguous;
the helper does not guess a radical or fill missing bracket H. Unsupported/valence
errors include the source token offset and original H mode. Resource/model errors
remain typed. Radical::None is constructed only within this declared finite projection
profile; this is not chemical-validity, physical-isotope or spin-state certification.
Syntax-valid inputs outside the projection stay preserved, not reclassified as malformed.

The mapper iterates bounded atom/bond slices and uses checked sums, degrees and IDs.
The immutable source owner already enforces64KiB/4096 atoms/8192 bonds and structural
limits. Candidate model data is discarded on any failure; source Arcs/IR remain exact.
One private finite-target helper now serves normalization and the existing writer;
there is no duplicated valence/symbol table or general chemistry framework. Existing
writer guards/messages remain exact after reversing only that helper factoring.

There is no StoredDocument backing/Engine/history/FS/UI/clipboard/import command,
source-backed Save, canonicalizer, or coordinate/layout work here. A returned Fragment
does not own the original interchange backing or authorize discarding it. Future
application import must retain the whole source/IR/baseline and account for it. The
existing generated writer remains a separate explicit export with its own output budgets.

## Original expectations and actual checks

Before implementation, `target/smiles-normalize-red.log`, session74860 exit101,
reported missing NormalizeError/E0599 normalize_explicit_h_fragment (plus consequent
inference errors). All6 original contract tests were present before production edits.
The initial filesystem write had failed during a disk shortage and created no file;
it is not counted as RED. Cargo was held while Root reclaimed inactive artifacts.

Minimal implementation passed all6 tests, session41174 exit0,
`target/smiles-normalize-first-green.log`. Intermediate full IO session73570 passed200
tests. A readability refactor made the isotope0 refusal an explicit match, retaining
the same behavior. Exact final evidence:

- `cargo test -p bondline-io --locked`: session83383 exit0,200 tests,0 failed/ignored,
  plus0 doctests; `target/smiles-normalize-io-final.log`.
- Warnings-denied all-target IO clippy: session55392 exit0,
  `target/smiles-normalize-clippy-final.log`.
- Workspace rustfmt check: session73841 exit0,
  `target/smiles-normalize-fmt-check.log`.

Each Cargo command used the absolute own target with CARGO_INCREMENTAL=0 and dev/test
debug0. BONDLINE_PYTHON selected the existing pinned Root interpreter; no installation,
dependency, lock, Git, GUI or full-workspace execution was performed. No Cargo job was
shared with another checkout. All own handles are terminal.

Literal tests independently specify C→H4, CC→3/3, CCC→3/2/3, C=C→2/2, C#N→C1/N0,
O→2 and N→3, dense IDs/None positions and exact bonds. Bare CRLF ethane and bracketed
ethane yield the same fixed model while their original source/IR/pointers remain distinct.
Other vectors cover positive isotope/physical-H nodes, literal H2 beside two physical
H atoms, H-H, halogen H0 and cross-dot ring reconnection as one graph. Missing/ambiguous
bracket counts, charges, isotope0, broader valences and unsupported elements refuse
without changing bytes or shared allocations. Existing syntax refusals stay typed.

The4096-atom chain and128 nested branches normalize without recursion. A complete
65536-byte leading-zero positive-isotope token normalizes to numeric isotope1 while
every original byte remains exact. The existing parser's4097-atom refusal stays intact.
The full200-test run includes all194 prior tests/assertions, including MOL VAL,
native-v0, source guards and the unchanged parser/writer tests and consumers.

Actual normalized graphs are generated through the existing writer and sent to the
unchanged external `smiles_write_oracle.py`. Its pinned unsanitized parser checks the
literal generated H values/physical nodes/isotopes/edges; a separate sanitized molecule
checks simple chemical meaning/formulas and perceived radical counts. No unsanitized
implicit-H/radical-zero getter is used to establish the normalized count. No new oracle
or weakened old assertion was needed. The old Python cleanup/dialect qualifications
remain; these are external expectations, not Bondline chemical perception.

## Primary sources and frozen files

The same pinned [OpenSMILES v1.0 publisher source](https://github.com/opensmiles/OpenSMILES/blob/9240a2887158699503605bc838e1de9a9f7883f4/opensmiles.asciidoc)
(86816 bytes, SHA256`00f0423da612de913770133aa13e456d0cebc562dd50166534a8a7f9a834bd21`)
defines bare/bracket H distinctions. The first projection deliberately omits broader
normal-valence cases. The existing [CTfile2026 VAL fields](https://www.3ds.com/assets/invest/2026-01/biovia_chemistry_2026_ctfile_formats.pdf)
and public [NumHydrogens](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Atom_NumHydrogens.htm)
and [display-property table](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/TableOfProperties.htm)
support the distinction between source constraints, fixed attached counts and depiction.
The accepted report records exact source pins/limitations. No prose, grammar, parser
code or assets are copied; original Rust remains MIT OR Apache-2.0.

| File | Bytes | SHA256 |
| --- | --- | --- |
| `crates/io/src/smiles/normalize.rs` |6678|`0efeb86ce526d3196a0a70f7b0d9937ed2fdfef3b6f9bf5cb2325c4d2e277f76`|
| `crates/io/tests/smiles_normalize.rs` |13139|`1cf6e24b2ad2edaf4fd72e93ce57fb0f3c574ddca3ffff13d22a8133fde0145f`|
| `crates/io/src/smiles/write.rs` |16411|`c6ae25315749b3f32cf3ad5eb339263163d9f551f6444b695177218d549603c7`|

Static reconstruction recovers exact pre-slice parent hash4f2e8ac8…466fd and old writer
hash77e0fc89…58be2 after removing only routing/comment and helper factoring. Original
parser tests/oracle remain7401a294…3810 / a13b0de3…6982; writer tests/oracle remain
76b8fae0…6742d / fae51a48…e5f2b. Model, encoders, fixtures, golden and both locks are
unchanged. Independent peer review and Root receive/full/fuzz/hosted gates remain
separate. No native or completed M1 claim follows.


## Receiving Root verification, 2026-11-14

Root independently verified all36 handoff entries and exact seven-file intentional set,
then received3295e8e on its newer SDF/native/writer foundation. Production parent/writer/
normalizer/test hashes remain4af679f1…f0ca7/c6ae2531…603c7/0efeb86c…77f76/
1cf6e24b…145f. Source/format appends retain the later SDF/native evidence. Independent
read-only reporttarget/smiles-normalize-review.md4936B/SHA
696c0d813feb08d194ccc6aee06da67faa07e8b13132e61e150d012cfee53f92 found no confirmed defect;
no reviewer execution. Root cargo xtask ci34271 exits0:601workspace+3bridge=604, all gates
GREEN, correct absolute Root target/incremental0/default debug/pinned external Python.
Logtarget/smiles-normalize-integrated-ci.log74302B/SHA
ecb50024a97fe4074180d5410591c05ea44aba7990216d2db5d27606b8c377f8.
Normal combined normalization/export fuzz and exact-source hosted checks remain separate.
This opt-in fixed-count library operation does not change model None, source IR, live
inference, application import/layout, backing/Save authority or chemical validity claims.
