# Parser fuzzing

This isolated test package is never a shipped dependency. cargo-fuzz 0.13.2 drives
libFuzzer 0.4.13 with AddressSanitizer on nightly-2026-04-01. The C++ libFuzzer runtime
has NCSA terms in addition to its Rust wrapper's MIT/Apache terms; this is test tooling,
not an expansion of the native runtime dependency allowlist.

```sh
rustup toolchain install nightly-2026-04-01 --profile minimal
cargo install --locked cargo-fuzz --version 0.13.2 --root target/tools
mkdir -p .cache/fuzz/cdxml
cp crates/cdxml/tests/fixtures/unknown.cdxml .cache/fuzz/cdxml/
PATH="$PWD/target/tools/bin:$PATH" cargo +nightly-2026-04-01 fuzz run cdxml_parse .cache/fuzz/cdxml -- -runs=10000 -max_len=65536 -timeout=5
```

Do not use the committed fixture directory as the writable mutation corpus. Keep generated
inputs/artifacts ignored; minimize and add any failure as an intentional regression test.
The harness checks bounded parser execution and exact source preservation on accepted input.
When an ordinary unnamespaced attribute exists, it also replaces its value with Unicode,
markup delimiters and whitespace, reparses, verifies the decoded value and checks the
original stayed unchanged. Inputs too close to the output size bound may reject the edit.
The same actual io crate also maps accepted direct-fragment geometry and patches one
explicit position to the published 72/144 point vector, reparses and checks the original.
This pulls the io/render dependencies into the isolated build, but does not initialize a
GPU or fuzz renderer behavior. The isolated workspace uses the same original loader/hex
compatibility patches and pins gpu-descriptor 0.3.0 in its own lockfile. Existing parser/
libFuzzer package pins remain unchanged. This extra build graph is test-only.
The renderer now uses the already-reviewed ab_glyph/font-parser pins; the isolated lockfile
includes those four packages too. Checking this graph does not fuzz glyphs or rendering.
Raw UTF-8 inputs and parsed attribute values also exercise exact integral-property and
UINT32 source-ID decoding. Successful conversions must agree with integer range checks
and retain their exact value after canonical decimal formatting. Independent Decimal
expectations live in the numeric integration tests.
The read-only elemental mapper also checks unchanged source plus unique application IDs
and fragment-local, distinct bond endpoints on accepted inputs. Its local reference maps
are exercised with the original fixture seed. Chemistry perception and a complete editable
file workflow remain outside this harness's claims. Accepted owned StoredDocument values
must export byte-exact source. The harness edits preview dimensions, requires stale-source
export to fail, restores the model and requires exact passthrough again. Unsupported page
layouts may reject the owned projection without rejecting the standalone syntax.
Explicit Poster dimensions are now read by the owned projection. The mutation always
chooses dimensions different from the source; missing/invalid explicit Poster dimensions
may reject the model while syntax remains intact. UTF-8 inputs also exercise the new
writer's attribute escaping and rollback: a successful page attribute decodes exactly;
rejected values leave no partial child. Original Poster and incomplete-Poster seeds are
placed only in the ignored mutation corpus, never the committed fixture directory.
The original native v0 seed also exercises complete-profile classification. Its changed
model may encode and reopen exactly; other source-backed edits must remain refused. Original
backing and borrowed-only stale-source guards still apply to both cases.
Independent XML semantic assertions live in the external Expat tests. A short smoke run is
not exhaustive parser conformance or a security audit.

## CDX binary syntax

```sh
mkdir -p .cache/fuzz/cdx
cp crates/cdx/tests/fixtures/unknown-implicit.cdx crates/cdx/tests/fixtures/original-ethane-explicit.cdx .cache/fuzz/cdx/
PATH="$PWD/target/tools/bin:$PATH" cargo +nightly-2026-04-01 fuzz run cdx_parse .cache/fuzz/cdx -- -runs=10000 -seed=721533879 -max_len=131072 -timeout=5
```

This separate target exercises bounded binary framing, exact original-byte preservation,
contiguous record spans and checked opaque property replacement. Accepted edits must
reparse, preserve record count/framing and retain all bytes outside the changed property;
replacing a payload with itself must be byte-exact. The original document stays unchanged.
The harness does not map CDX chemistry, resolve references, write new object trees or access
files. Independent Python struct/RDKit expectations live in the crate's integration tests.
The original explicit ethane seed is 65750 bytes, including a 65535-byte opaque extended
property. The former 65536-byte run limit could not represent that complete scene; the
current 131072-byte bound deliberately includes it. Both original fixture identities and
their generator/source provenance are recorded in the fixture README. An accepted binary
mutation is syntax/source evidence, not independent chemical interoperability for that mutation.
The CI sanitizer job runs all six parser targets; these bounded runs are not exhaustive input
coverage or proof of application CDX support.

## MOL V2000 syntax and elemental graph

```sh
mkdir -p .cache/fuzz/mol
cp crates/io/tests/fixtures/mol/rdkit-CH3.mol crates/io/tests/fixtures/mol/ethane-v2000.mol .cache/fuzz/mol/
PATH="$PWD/target/tools/bin:$PATH" cargo +nightly-2026-04-01 fuzz run mol_parse .cache/fuzz/mol -- -runs=10000 -seed=619734821 -max_len=65536 -timeout=5
```

The seed is the pinned BSD-3-Clause RDKit methyl-radical document; its source, SHA-256
and retained notice are recorded in `crates/io/tests/fixtures/mol/README.md`. The original
fixed-column two-carbon vector from `crates/io/tests/mol.rs` may also be copied into this
ignored corpus. Never use the committed fixtures as a writable mutation corpus.

Accepted input must preserve every original byte and respect the published parser's
line/count bounds. Strict elemental projection uses a validated identity scale/origin;
unsupported syntax/model features may be refused explicitly. Successful V2000 export
must parse and preserve graph identities/connectivity, elements, explicit H, isotopes,
charges and radicals. Coordinates may change by half of the format's four-decimal step
plus a bounded floating-point representation term. Canonical output is not claimed to be
byte-identical to original input; original source remains unchanged throughout.

This target does not infer chemistry, compare every mutation with RDKit, import into an
application session, or exercise native file I/O. Independent published-field/RDKit
expectations remain in the IO integration tests. The bounded sanitizer run is not complete
format conformance or a security audit. It adds no shipped dependency or GPU initialization.

## SDF V2000 records and ordered data

```sh
mkdir -p .cache/fuzz/sdf
cp crates/io/tests/fixtures/sdf/ordered-fields-v2000.sdf .cache/fuzz/sdf/
PATH="$PWD/target/tools/bin:$PATH" cargo +nightly-2026-04-01 fuzz run --sanitizer address sdf_parse .cache/fuzz/sdf -- -runs=10000 -seed=826403517 -max_len=65536 -timeout=5
```

The original seed has ordered/duplicate/empty data fields; see its fixture provenance.
Accepted input must retain exact bytes and bounded record/item/header/value spans, including
raw Molfile bytes and record delimiters. Ordered data fields remain distinct. The harness
also writes each accepted record canonically and checks the complete Molfile and ordered
headers/value lines; new wrapper framing, delimiter tails, padding and line endings may normalize. Source stays
unchanged. Near-budget canonical output may refuse resource_limit explicitly.

This is parser/codec fuzzing. It does not establish chemistry correctness, application SDF
import/export, native file workflows or independent compatibility for every mutation. The
original fixed-field vectors and separately run RDKit consumer remain the independent tests.
Committed seeds are never writable mutation corpora. No lock/runtime dependency changes.

## MOL V3000 syntax and strict graph projection

```sh
mkdir -p .cache/fuzz/mol-v3000
cp crates/io/tests/fixtures/v3000/*.mol .cache/fuzz/mol-v3000/
CARGO_TARGET_DIR="$PWD/target" PATH="$PWD/target/tools/bin:$PATH" cargo +nightly-2026-04-01 fuzz run --sanitizer address mol_v3000_parse .cache/fuzz/mol-v3000 -- -runs=10000 -seed=937155923 -max_len=65536 -timeout=5
```

Use this checkout's own target directory; separate worktrees must never share compiled
artifacts. cargo-fuzz 0.13.2/libFuzzer 0.4.13 and nightly-2026-04-01 retain the pinned
test-tool versions above. The original sparse-ID and continuation/quote/list/collection
seeds are MIT OR Apache-2.0; see their fixture README. Mutations stay in ignored cache,
never the committed fixture directory. CI seeds and runs this fifth parser target.

Accepted input must preserve exact bytes, bounded source spans and flat block parents,
positive unique atom source IDs and valid bond endpoints. A strict supported graph is
written with full f64 precision, reparsed and compared for authored fields/coordinates.
Unknown/query/stereo/3D data must refuse projection; near-output budgets may refuse
canonical writing with resource_limit. This does not exercise version-aware application
files, a native window/GPU, or external chemistry for every mutation.

The fresh final local ASan run passed 10,000 iterations with the exact seed/max_len/timeout
above: coverage 1254/features 2799, 169 retained inputs/reported 45 KiB/RSS 74 MiB, 1 second of
fuzzing. Its largest exercised input was 492 bytes, so max_len 65536 is a ceiling rather
than proof of large-input coverage. Explicit resource/large-graph tests and external
RDKit consumers provide separate evidence. This bounded pass is not a security audit.

The CDX target now additionally constructs new explicit-root syntax through Writer and
checks every non-root object tag/ID, opaque property payload and nesting event after parsing.
Canonical rootID0, zero reserved header and length prefixes are deliberately new syntax,
not imported-source passthrough. The original exact-source/property checks remain intact.
Near-boundary implicit-root inputs may refuse new explicit framing on byte/record/depth
budgets. Invalid semantic references/types are not inferred as chemistry. The fresh
2026-11-14 two-seed pass ran10000 iterations at seed721533879/max_len131072/timeout5:
coverage305/features878,59 retained inputs/reported150KiB/RSS199MiB; largest65750bytes.
Use pinned stable cargo fmt/clippy for source checks; the isolated nightly is for ASAN.

## SDF mixed molecular versions

The existing `sdf_parse` target now checks actual per-record V2000/V3000 encodings
and uses the generic preserving writer. Its explicit write_v2000 assertions still
require byte-identical output for V2000 and unsupported_content for V3000. It does
not convert or project a molecule. Ordered raw headers, DT digits, duplicate/empty/
multiline values, source ranges and all MOL bytes remain checked. The old V2000
seed and independent oracle assertions stay intact. This extends the fourth target,
not the number of parser targets.

```sh
mkdir -p .cache/fuzz/sdf-mixed
cp crates/io/tests/fixtures/sdf/*.sdf .cache/fuzz/sdf-mixed/
CARGO_TARGET_DIR="$PWD/target" CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 PATH="$PWD/target/tools/bin:$PATH" cargo +nightly-2026-04-01 fuzz run --target-dir "$PWD/target" --sanitizer address sdf_parse .cache/fuzz/sdf-mixed -- -runs=10000 -seed=826403517 -max_len=65536 -timeout=5
```

Keep this worktree's own absolute target; the local command may select the existing
main-checkout pinned tool binary read-only via PATH. Extra ignored seeds may wrap the
original continued/quoted V3000 MOL plus data, or use externally generated8 dp/1,000-atom
MOL bytes and a complete SDF delimiter. Never normalize input or mutate fixture files.
No new tool version, lock, native/runtime dependency or application workflow is added.
Actual bounded-run evidence and input sizes belong in docs/verification.md; max_len
alone establishes no large-input coverage or security guarantee.

The actual fresh mixed-SDF ASan run passed 10,000 iterations at the exact settings
above: coverage 1605/features 6242, 341 retained entries/reported 3066 KiB, maximum
exercised input 49,063 bytes, reported RSS 462 MiB, 3 seconds after compilation.
The six source seeds and hashes are recorded in ignored target/sdf-v3000-fuzz-seeds.json;
source fixtures remain unchanged. Scope and independent-consumer limits remain as above.

## Legacy RXN containers and molecule payloads

```sh
mkdir -p .cache/fuzz/rxn
cp crates/io/tests/fixtures/rxn/*.rxn .cache/fuzz/rxn/
# Optional independent large seed, using the pinned external test interpreter:
"$BONDLINE_PYTHON" -B crates/io/tests/rxn_oracle.py emit_mixed > .cache/fuzz/rxn/rdkit-mixed-1000.rxn
CARGO_TARGET_DIR="$PWD/target" CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 PATH="$PWD/target/tools/bin:$PATH" cargo +nightly-2026-04-01 fuzz run --target-dir "$PWD/target" --sanitizer address rxn_parse .cache/fuzz/rxn -- -runs=10000 -seed=224867531 -max_len=65536 -timeout=5
```

Use the worktree's own absolute target path for every Cargo call. The local RXN run
used the existing pinned cargo-fuzz executable from the main checkout read-only via
PATH; it did not install another copy or share build output. Keep mutation inputs in
ignored cache and never clean another active worktree's target. Root/fuzz lockfiles
and dependency pins did not change. CI seeds both original RXN vectors and runs this
sixth target with the exact seed/iteration/length/timeout bounds above.

Accepted source must remain byte-exact with contiguous component/whole-MOL spans,
correct role counts/order, bounded payloads and distinct wrapper/MOL versions. New
wrapper output must reparse and preserve each complete MOL payload, except a missing
final LF may be appended for framing. Headers and roles must survive. Near-budget
canonical writing may refuse resource_limit; it may not reorder roles or regenerate
unknown molecular content. The harness does not project a reaction, execute chemistry,
exercise application/native files or run external consumers for arbitrary mutations.

The actual local final ASan pass completed 10,000 runs, seed 224867531, max_len 65536,
timeout 5, exit 0: coverage 1305/features 4915, 290 retained corpus entries/reported
3549 KiB, maximum exercised input 44,023 bytes, reported RSS 464 MiB, 3 seconds of
fuzzing after compilation. The original 696/739-byte seeds plus a separately generated
44,023-byte mixed-version/1,000-atom RXN were copied only into ignored cache. This is
bounded smoke evidence, not universal format compatibility, complete resource-bound
coverage, chemistry verification or a security audit. The IO tests independently
exercise explicit limits and raw/RDKit expectations.

### Header-embedded CDX root

Add the new original framing seed to the ignored mutation corpus; keep all committed
fixtures and old seed identities unchanged:

```sh
cp crates/cdx/tests/fixtures/original-ethane-header-document.cdx .cache/fuzz/cdx/
```

The same CDX harness now treats HeaderDocument's encoded root as structural framing
and excludes its separate FileEnd only when constructing a new canonical Writer tree.
Source passthrough and property edits still preserve every original framing byte and
record event, including FileEnd. New Writer output retains its earlier28-byte header
and explicit root; it never reconstructs source-backed input implicitly.

The fresh2026-11-14 three-original-seed ASAN pass completed10000 runs at
seed721533879/max_len131072/timeout5: coverage319/features841,53 retained inputs,
reported303KiB/RSS283MiB, largest65750bytes. Because disk was constrained, this local
run used an ignored minimal manifest containing only path bondline-cdx and pinned
libfuzzer-sys0.4.13, with the exact committed cdx_parse source and all transitive
versions/checksums copied from the unchanged fuzz lock. It did not rebuild the full
IO/GPU fuzz dependency graph. Stable clippy passed for that same scoped harness and
the normal fuzz workspace's rustfmt check passed. The normal full-graph/hosted checks
remain separate integration evidence; this bounded run is not a security audit.

### CDX elemental and owned-source guards

The existing CDX target also invokes the limited IO projection. Successful views retain
exact source, fresh dense IDs and fragment-local endpoints with no inferred H. The same
input creates an owned snapshot; unchanged export is exact, a page edit refuses stale/
cross-format output and restoring its snapshot restores all original bytes. Source guards
remain separate from new-tree canonical construction. No new target or external dependency.
The normal full-graph four-seed2026-11-14 run passes10000 iterations at seed721533879,
max_len131072/timeout5: cov1019/features2161,81inputs/reported394KiB/RSS303MiB, largest65750B.

The source-only metadata extension reuses that target unchanged. A fresh normal full-graph
2026-11-14 run adds the original323/319B metadata vectors to the four original seeds:
10000 iterations, same seed721533879/max_len131072/timeout5, cov1131/features2728,
97inputs/reported273KiB/RSS255MiB,3seconds, largest65750B. Normal-graph clippy passes.
Exact hashes/settings are in verification-cdx-native-metadata.md. This exercises accepted
metadata/source/snapshot guards without introducing H inference, stereo computation,
styled-warning support or new targets. Copy seeds to ignored cache before mutation.
CI now copies both committed original metadata vectors alongside its three existing CDX
seeds. Its five-file initial CDX corpus is separate from the local six-file run, which
also includes the independently packed92B default/no-geometry seed in ignored storage.
The extra92B independently authored default-C/single/no-position vector is ignored, not a
new fetched corpus entry. See verification-cdx-owned-snapshots.md for exact logs and limits.


### Owned whole-SDF snapshots

The same SDF target now retains each accepted complete file in SdfDocument, checks exact
source, positive ordinal record IDs/ranges/versions/field counts and borrowed reread,
shared source/index pointers after Clone, full equality and deduplicated snapshot bytes.
All existing span/wrapper/molecular assertions remain. This adds no target/dependency.
Fresh ignored seeds are unchanged original690/777B vectors, not committed mutation output.
Normal full-graph all-target clippy passes; AddressSanitizer10000 runs at seed826403517,
max_len65536/timeout5 exits0: cov1633/features5009,294retained inputs/reported154KiB,
RSS91MiB,1second, largest seed/observed length777B. The configured bound is not claimed
as exercised coverage; explicit large/boundary cases remain in IO tests. Exact logs/hashes
are in verification-sdf-owned-source.md. No data edit, file/Engine/native workflow or
external chemistry comparison for arbitrary mutations is established by this smoke.


The owned snapshot also performs an existing-field candidate edit in the same target:
identical logical values must share old bytes/index; a fixed valid new value must preserve
exact prefix/suffix, all molecular blocks, versions/IDs/field counts and old snapshots.
Quota refusal is permitted; malformed value validation has independent static tests.
Fresh original690/777B-seed ASAN17018 passes10000 runs at seed826403517/max_len65536/
timeout5: cov1688/features5390,306inputs/reported158KiB/RSS59MiB,5seconds, largest exercised
777B. Full-graph all-target clippy/fmt pass; see verification-sdf-field-value.md for exact
logs/hashes and scope. No Engine command, native workflow or arbitrary-mutation external
chemical equivalence is claimed from these harness assertions.
