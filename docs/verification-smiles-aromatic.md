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


## Receiving Root and actual coupling RED/GREEN

Root strictly verified all8 candidate/10evidence/21preserved records and exact file set,
then committed178aee9. Every protected entry was also checked against actual3295e8e.
Read-only core reviewtarget/smiles-aromatic-review.md10346B/SHA
e38cc45aba98fe58401e8dd6c81703e0c9239b5b8a910e03a5468f72b7a95807 found no confirmed defect;
no reviewer execution. Root receives this on28ac0b5's newer corpus/20-command/native-media
foundation. Parsere3f2d969…c58c and normalizere75bfa03…5c4e9e remain exact. Root preserves
its extra ninth original-seed test and three whole-source consumer entries; all older
assertions remain except deliberate affirmative migration of now-supported aromatic cases.

Actual receiving compiler REDs46315/73456 reject missing exhaustive Aromatic cases in
corpus/fuzz. Runtime corpus99285 failsNull-versus-false: markers were not serialized into
literal/actual views. Explicit Aromatic dispatch, actual source flags, false seed literals
and wrong-flag negatives fix that gap; scoped72251 passes. Fixture-discovery22016 fails
before original aromatic seed files exist, then48182 passes actual bytes/literal marker/
edge/EOF/refusal and unchanged external-consumer checks. New dual-licensed9/4/18-byte
fixtures are documented in their README; CI's existing glob now copies six total seeds.
The fetched bundle still pins the original three; its eight-file/five-bundle count is unchanged.

Independent receiving review caught a real harness error: Bond.bond_range=None also occurs
for explicit ring tokens stored in RingClosure, so endpoint-default assertion falsely
rejected valid C:1CCC1 and c-1ccccc1. Root reproduced C:1CCC1 with normal ASAN fixed-input
87810 (wrapper1/internal77, assertion only), retained the failure, then restricted default
checking to cases with no ordinary or explicit ring token. All old token checks remain.
Fixed-input41788 exits0. Mixed explicit-ring orders are now an original committed seed and
face literal complete10-atom/10-bond topology plus the unchanged external consumer.
This is a test false positive, not a parser or memory defect.

Final receiving reviewtarget/smiles-aromatic-integration-review.md10357B/SHA
9b8781c7f4c544a7b6f8ba0b13b4265b705536f0a285efc54858af5f40787a55 retains finding/resolution;
no further confirmed issue or reviewer execution. Final harness13044B/SHA
8b65205cd152e888cd868cd93c7e894ed3de9e95401386c7410dfbd59a4d7d36;
checker9778B/SHA4d06bc0c48e0340c7087ce8db411d194e8875179f00712edfdf21e6a2285310b.
A later provenance-table edit removes one empty line only; recordtarget/smiles-aromatic-readme-format.json.

## Final receiving gates

Root cargo xtask ci99138 exits0:648workspace+3bridge=651,0failed/ignored; fmt/all-target
clippy/layers/unchanged licenses/assets and actual corpus8files/5bundles/3SMILES pass.
Correct Root/absolute target/incremental0/default debug/pinned external Python. Log
 target/smiles-aromatic-integrated-ci-final.log78298B/SHA
 39d98735e3bb783f57255419e6e97d670cc85426f4c974bb1d57946ceaaf1c2c.
The earlier16503 full651 pass preceded the stronger mixed-ring seed assertion; it is
retained separately, not substituted for this final gate.

Normal full-graph stable all-target fuzz clippy exits0, log319B/SHA
 ee7c32acf168efff04a39b1c26b78b101ed97504bcaddb14c9455af65da32365;
workspace/isolated fmt pass. Final ASAN64957 exits0:10000, seed703592681/max_len65536/
timeout5, fresh9seeds/8304B (six committed plus original4096C/4096c chains/multiring),
max starting/exercised4097B, cov1979/features7021,331retained/reported406KiB/RSS422MiB,
8seconds after compilation. Unchanged normal fuzz manifest/lock/pinned tools/nightly,
own absolute fuzz/target/incremental0/release-debug0; no minimal alternate graph. Log
 target/smiles-aromatic-fuzz-asan.log54932B/SHA
 aca959bfb1c0727f734ca7b608b258d1310c24a7daf0e26ced4427fe0d25b86c.
Configured64KiB is a ceiling, not exercised coverage. This is not a security audit or
aromaticity/chemistry verification. New-source hosted checks remain pending. No native
SMILES, perception, layout/model aromatic conversion, golden or M1 completion claim.
