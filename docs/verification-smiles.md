# SMILES notation syntax and exact source ownership

2026-11-14. Original Rust library feature in the isolated `bondline-mol-v3000`
checkout, supplied branch `feat/smiles-syntax`, supplied base `15ecbc7`. No Git,
install, GUI, model, Engine, application command, filesystem adapter or generated
writer work was performed. Original implementation/tests remain MIT OR Apache-2.0.

## Delivered contract

`io::smiles::SmilesDocument::parse(Arc<[u8]>)` accepts one nonempty bounded ASCII
notation with at most one terminal LF or CRLF. It retains the entire original Arc
and returns those exact bytes from `as_bytes()`. No name, CX suffix, extra line or
unparsed trailing token is silently accepted. `notation_range()` excludes only the
optional terminal line ending. Clone shares source/atom/bond allocations; equality
includes every source byte. Debug discloses byte/atom/bond counts only.

The immutable notation graph retains zero-based lexical atom indices, original atom
and symbol/isotope/H/charge ranges, explicit bond-token ranges, and both original
ring-label/bond declarations. Bare organic-subset atoms carry `Hydrogens::Implicit`,
without resolving a count. Bracket H omission is `Bracket(0)` with no H-token range;
an explicit H0 token has the same count but its own range. Physical `[H]` nodes stay
separate, and an H-count on a hydrogen atom, including H0, refuses. Optional isotope
retains `None` versus `Some(0)` and leading-zero source spelling. No atom is assigned
model hydrogen, radical, valence, charge validity, geometry or application identity.

The accepted initial syntax has bare B/C/N/O/P/S/F/Cl/Br/I, bracket elements from the
existing IUPAC catalog, isotope numbers0..65535, bracket H0..H9, charge-15..15
(including deprecated ++/-- spellings), Single/Double/Triple bonds, branches, dots
and numeric ring labels0..99. Larger isotope/charge values are explicit supported-
profile refusals. Existing118-element IUPAC2022 names include Nh/Mc/Ts/Og; these
are a documented present-catalog extension beyond the pinned2016 symbol listing.
No symbol data is copied: two visibility-only lines expose the existing catalog
internally, and lib.rs adds one module export.

Branches/rings are parsed iteratively. Ring labels compare numerically:0 and%00,
or1 and%01, can match; %73 and%74 remain distinct. `%123` means%12 followed by3.
Both endpoint bond declarations must agree, and unmatched labels, self links and
duplicate links refuse. A closed label can be reused. Dot breaks a notation chain,
while a later ring can reconnect it; chain count is never called graph component
count. The pinned grammar permits a branch to begin with a dot and requires an
atom's ring labels before its branches.

Original caps are65536 complete input bytes,4096 atoms,8192 bonds,128 simultaneously
open branches,1024 notation-chain starts (initial plus dots, including dot branches),
and100 open ring labels. Bounds are checked before graph insertion/allocation;
offset/numeric accumulation and counter changes are checked. The ring table has
exactly100 slots, so a101st distinct open label has no spelling in this profile.
Tests fill all100 simultaneously; extended label syntax refuses. These bounds are
not an allocator/RSS guarantee. Aromatic atoms/bonds, stereo, classes/maps,
wildcards, query/extended/reaction features and non-ASCII return typed unsupported
errors. Malformed topology/grammar/strict EOF return syntax errors. Quotas have a
separate resource code; every error carries a byte offset.

## Primary sources and independent expectations

The publisher's [OpenSMILES v1.0 source](https://github.com/opensmiles/OpenSMILES/blob/9240a2887158699503605bc838e1de9a9f7883f4/opensmiles.asciidoc)
is pinned to9240a2887158699503605bc838e1de9a9f7883f4. This implementation task
downloaded/read the exact86816-byte source, SHA256
`00f0423da612de913770133aa13e456d0cebc562dd50166534a8a7f9a834bd21`,
in ignored `target/smiles-opensmiles.asciidoc`. Grammar/bracket/isotope/charge/H,
branch, ring and dot rules inform original code; no GFDL prose/grammar/example
collection or upstream parser implementation is copied into the product. Public
reference caching remains separate from shipped assets. The existing IUPAC-ELEMENTS
source supplies current symbol facts; no periodic-table image/weights are bundled.

Original literal tests specify full source, atom spans and graph adjacency for
branched carbonyl, triple bonds, cycles, cross-dot connections, reused labels,
two distinct%73/%74 closures, numeric aliases and `%123`. Bracket cases independently
specify elements/isotopes/H/charge and raw token presence, including isotope0,
H omission/H0/H1, charged salt and physical isotope-H neighbors. Actual65536-byte,
4096-atom,8192-bond,128-branch and1024-chain boundaries succeed; the next size refuses.
The maximum-atom chain and nested branches exercise iterative construction. The
dense8192-bond graph is a notation-boundary case, not chemical valence evidence.

Root's separate read-only external vector report, `target/smiles-independent-vectors.md`
in the Root checkout, SHA256
`cdfc504b996c80a58347a000bd20ffb5bfc45d907ebf3dea1079c36c8f77c10f`,
records27 independently authored atom/graph expectations,13 rejects and qualified
consumer disagreements. Its external calls are not this codec's Rust GREEN.

## Actual RED and GREEN evidence

All Cargo work selected this checkout's absolute target, `CARGO_INCREMENTAL=0`,
`CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0`, and the existing Root
`BONDLINE_PYTHON` interpreter. No package/toolchain/lock/dependency was changed.

- `target/smiles-red.log`, session72678 exit101: E0432, missing io::smiles module.
  The first ownership/literal implicit-versus-bracket test failed before production
  implementation. Minimal linear/bracket implementation then passed that test,
  session55901 exit0, `target/smiles-first-green.log`.
- Expanded original branch/ring/dot/limit/error tests failed on the missing behavior,
  session67809 exit101, `target/smiles-topology-red.log` (1 pass/6 failures). After the
  iterative stack/table implementation, all7 passed, session42721 exit0,
  `target/smiles-topology-green.log`.
- The independent-consumer test initially failed with its absent script, session27074
  exit101, `target/smiles-consumer-red.log`. Subsequent real diagnostics retained in
  `smiles-consumer-green.log`, `smiles-consumer-corrected-green.log` and
  `smiles-consumer-final.log` are failures despite their early filenames: unavailable
  Python skipCleanup, then numeric-label and dot-branch consumer differences. They
  led to explicit diagnostic assertions, never relaxed parser behavior.
- The first complete8-test GREEN is session64777 exit0,
  `target/smiles-final-green.log`. Later SMARTS primitive `[#6]` coverage failed with
  the wrong error category, session39404 exit101, `target/smiles-query-red.log`.
  Adding that primitive to the existing unsupported branch fixed the category.
- Final exact-source `cargo test -p bondline-io --locked`, session46150 exit0,
  `target/smiles-io-green.log`:186 tests,0 failures/ignored, plus0 doctests. This
  includes all8 SMILES tests and every existing IO test.
- Final all-target IO warnings-denied clippy passes, session27896 exit0,
  `target/smiles-clippy-final.log`; the earlier identical-if-block warning remains
  in `smiles-clippy.log`, fixed by one shared condition. Workspace rustfmt check
  passes, `target/smiles-fmt-check.log`.

## External-consumer boundary

New original external-only `tests/smiles_oracle.py` requires actual RDKit2026.09.2
and passes the **whole accepted ASCII source**, including LF/CRLF, directly to its
SMILES parser. It explicitly sets sanitize/removeHs/parseName/allowCXSMILES false.
It checks actual source bytes/length, literal element/isotope/charge/H attributes,
physical H retention, and independently parsed graph edges. Missing consumer/version
fails, never skips. The official release parser API is pinned to
[f86b7dc9b31e2531200bbd1f54589bcbd968bb05](https://github.com/rdkit/rdkit/blob/f86b7dc9b31e2531200bbd1f54589bcbd968bb05/Code/GraphMol/SmilesParse/SmilesParse.h).

The Python binding exposes no skipCleanup property; an actual setter attempt raised
AttributeError. The final oracle asserts/reports its absence rather than pretending
to disable the underlying C++ default cleanup. No SanitizeMol/UpdatePropertyCache,
H addition/removal, canonical output or layout is called. Pending property caches
and zero radical getters are unperceived consumer defaults, not a Radical::None or
implicit-H assignment. Isotope omission/0 and bracket H omission/H0 collapse in its
numeric getters, so literal source/token tests remain decisive for those distinctions.

Pinned RDKit accepts `[HH]`, `[HH0]` and contradictory closure `C=1CC-1`; the codec
retains normative refusal. It accepts empty notation, which this nonempty profile
refuses. It rejects numeric%01/%00 aliases and a leading-dot branch that this pinned
grammar accepts. These are asserted dialect diagnostics, not automatic validity
oracles. Its acceptance of aromatic/maps/stereo/wildcard cases is separate from the
codec's explicit unsupported result. No full OpenSMILES conformance is claimed.

## Frozen source and continuation

| File | SHA256 |
| --- | --- |
| `crates/io/src/smiles.rs` | `c9d8e2c810e9b9a1d1c7ca1c26ecdb441a214d63531ba67efaa4d6e6d15f9ff3` |
| `crates/io/tests/smiles.rs` | `7401a2945ff9890bb932c019fac7033b4de5278b3ce3129c41e34974355d3810` |
| `crates/io/tests/smiles_oracle.py` | `a13b0de3d1c9ea24fc2b53427c51049b832595a86c2db25af7717100236f6982` |

Independent peer source review, Root integration/fuzz/full aggregate and hosted
acceptance are subsequent gates. No full-workspace or native SMILES run belongs to
this handoff, and all own command handles are terminal. Root owns commits; no Git
operation was run here. No old fixture/test/oracle, runtime package, lock, font or
approved golden was changed. Model projection/generated output, hydrogen/perception
decisions, immutable application backing/accounting/history, commands/FS/clipboard,
layout/native workflows and M1 completion remain separate work. No placeholder
successful conversion/export API is exposed.
