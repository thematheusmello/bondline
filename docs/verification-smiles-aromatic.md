# Literal aromatic SMILES notation

2026-11-14. Original Rust library slice in the supplied isolated
`bondline-mol-v3000` checkout, branch `feat/smiles-aromatic-syntax`, base `3295e8e`.
No Git operation, GUI, installation, model, Engine, filesystem, runtime dependency,
lock, native encoder, renderer, layout or application command was changed here.

## Delivered boundary

`smiles::Atom.aromatic` records a lowercase source symbol; notation
`smiles::BondOrder::Aromatic` records explicit `:` or an omitted bond between two
lowercase-marked endpoints. Bare b/c/n/o/p/s and bracketed variants plus as/se map
through the existing catalog. Uppercase As/Se retain false markers. Ranges always
refer to the original bytes, including two-character symbols and isotope/H/charge
tokens. Bare implicit H, bracket omission/H0/H1 and physical H remain distinct.

An omitted mixed-endpoint bond is Single. Explicit Single/Double/Triple/Aromatic
declarations override omission, including closures; two explicit closure orders must
match. Both original declarations stay retained. Reused/numeric ring labels and
cross-dot closure behavior remain unchanged. Dots still count notation-chain starts,
not connected components. Branch/ring parsing remains iterative and bounded.

The parser indexes notation, including literal ringless lowercase chains or `C:C`.
It does not verify an aromatic ring, assign electrons or hybridization, Kekulize,
compute aromaticity, infer H, perceive radicals, certify valence/chemical validity or
canonicalize. In `C:C`, both atom markers stay false. This is intentionally less than
the complete OpenSMILES semantic requirements. Unsupported lowercase symbols such
as te/si remain typed refusals even if a consumer accepts them. Stereo/maps/queries,
wildcards, wider charges/isotopes, strict ASCII/EOF and all old budgets retain their
own explicit boundaries.

`normalize_explicit_h_fragment` preflights every aromatic atom marker and bond before
integer bond sums or fixed-H mapping. Refusals point to the lowercase symbol or
explicit colon, including either ring declaration. No model Fragment is returned;
source/IR allocations remain shared and exact. The prior model writer and its guards
are byte-for-byte unchanged; no aromatic model conversion or generated aromatic
export is delivered.

## Primary source and independent checks

The existing [OpenSMILES v1.0 publisher source](https://github.com/opensmiles/OpenSMILES/blob/9240a2887158699503605bc838e1de9a9f7883f4/opensmiles.asciidoc)
was rehashed/read at the same immutable commit9240a2887158699503605bc838e1de9a9f7883f4:
86816 bytes, SHA256
`00f0423da612de913770133aa13e456d0cebc562dd50166534a8a7f9a834bd21`.
Its formal syntax versus semantic distinction, aromatic spellings, bond rules and
ring-declaration rules inform original logic and literal expectations. The separate
aromaticity discussion includes proposed electron rules explicitly marked under
revision; this feature implements no such algorithm. Cached GFDL reference remains
ignored in target/. No upstream source, grammar/prose, example corpus, asset or new
dependency is copied into product code.

Eight new tests independently specify literal edges, source markers/ranges, H and
isotope presence (including zero), charge, mixed branches, both closure declarations,
numeric aliases, distinct73/74 labels, reuse and cross-dot connections. Source/IR
pointer sharing/equality and count-only Debug remain checked. Aromatic byte/atom/
branch/chain boundaries and the old dense8192-bond/100-open-label tests cover the
unchanged quotas. Existing aromatic negatives were explicitly replaced by positive
source/graph cases and normalization refusals; unrelated negatives remain asserted.

Actual complete bytes, including terminal CRLF, face the unchanged
`tests/smiles_oracle.py` external-only RDKit2026.09.2 consumer. Independent literal
element/isotope/charge/explicit-H/no-implicit/edge expectations check15 positive
sources. Sanitization, H removal, name parsing and CX parsing are false. The pinned
Python API still lacks skipCleanup; all inspected atoms still need property-cache
updates. Default radical zeros and implicit-H getters are not chemical evidence.
No sanitization, cache update, Kekulization, canonical writer or layout operation is
used by this aromatic consumer test.

New diagnostic cases explicitly retain RDKit's te/si acceptance and contradictory
colon/single closure acceptance as consumer differences. Numeric `%01` alias refusal
also remains qualified; no consumer result rewrites source or weakens the profile.
An ignored25-source probe additionally recorded raw atom aromatic flags with the same
four false parser settings; that probe is separate from the unchanged mandatory
consumer and makes no sanitized/perceived-aromaticity claim.

## Actual RED and GREEN

Every Cargo command uses this checkout's absolute target, `CARGO_INCREMENTAL=0`,
`CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0`, and the existing Root
`BONDLINE_PYTHON` interpreter. Jobs run sequentially; no workspace/GPU/fuzz build or
install is duplicated here.

The test command is `cargo test -p bondline-io --test smiles_aromatic --locked` for
the focused RED/GREEN steps, then `cargo test -p bondline-io --locked` for the IO
suite. The absolute target is
`/Users/sarenaberg/Workspace/doozle/open-source-world/bondline-mol-v3000/target`;
the interpreter is Root's `.cache/oracles/format-python/bin/python`.

- Runtime RED49280 exit101, `target/smiles-aromatic-runtime-red.log`: the initial
  literal six-cycle test refused aromatic source at byte0 before implementation.
- Missing-API RED53356 exit101, `target/smiles-aromatic-api-red.log`: E0609 for
  `Atom.aromatic`, E0599 for `BondOrder::Aromatic`.
- Minimal GREEN70092 exit0, `target/smiles-aromatic-first-green.log`:1 test passed.
- Profile RED77298 exit101, `target/smiles-aromatic-profile-red.log`:6 passed/2 failed;
  `[si]` incorrectly received syntax instead of the intended typed unsupported code.
  The bracket-symbol boundary was then fixed without broadening the symbol profile.
- Profile GREEN1545 exit0, `target/smiles-aromatic-profile-green.log`:all8 new tests
  passed, including the unchanged external consumer and explicit dialect differences.
- Scoped IO GREEN53401 exit0, `target/smiles-aromatic-io-green.log`:208 tests passed,
  including the old parser, writer, normalization and other format assertions; no
  failures/ignored tests. The log is24380 bytes, SHA256
  `5e321ef26d3497ffba4835f03364469c5d3558a01ee43bec9733e37a17138df8`.
- `cargo clippy -p bondline-io --all-targets --locked -- -D warnings`,95947 exit0,
  `target/smiles-aromatic-clippy.log`. `cargo fmt --all -- --check` exits0,
  `target/smiles-aromatic-fmt-check.log`. No job started by this slice remains live.

Hash checks retain the exact prior writer and writer tests, both external oracle
scripts, model graph, native CDXML encoder and v0 fixture. No dependency/catalog data
or approved golden changed. The ignored handoff manifest records exact candidate,
preserved-surface and evidence hashes. All authored source/test jobs are terminal.

This is library notation/source support. Parent owns independent source review,
integration, complete gates, fuzz/hosted evidence and the project status. No application,
native workflow, model aromaticity or M1 completion follows from these checks.
