# Parser fuzzing

This isolated test package is never a shipped dependency. cargo-fuzz 0.13.2 drives
libFuzzer 0.4.13 with AddressSanitizer on nightly-2026-04-01. The C++ libFuzzer runtime
has NCSA terms in addition to its Rust wrapper's MIT/Apache terms; this is test tooling,
not an expansion of the native runtime dependency allowlist.

## Literal chain JSON requests

`chain_path_request` wraps up to16KiB of raw parameter bytes in the real command
envelope and exercises decoding plus immediate preparation. It never completes file/
render work or uses GUI/GPU. Errors preserve authored content/stamp/history; accepted
free paths have the bounded literal C/H/flag graph and exact whole-content Undo/Redo.
Default source has no attachment targets, so successful attachments and complete
saved-file/chemical/native behavior remain independent integration tests.

Use a fresh ignored directory; never mutate committed fixtures:

```sh
python3 - <<'PY'
import json
from pathlib import Path
root = Path('.cache/fuzz/chain-path-b4c6ae1')
root.mkdir(parents=True, exist_ok=False)
base = {'expected_revision': 0, 'page_id': 1,
        'hydrogen_policy': 'rectify_neutral_carbon',
        'placement': {'mode': 'free', 'points': []}}
for count in (2, 5, 64, 65):
    value = json.loads(json.dumps(base))
    value['placement']['points'] = [{'x_pt': i, 'y_pt': i % 2} for i in range(count)]
    (root / f'original-{count}').write_text(json.dumps(value), encoding='utf-8')
wrong = json.loads(json.dumps(base))
wrong['placement']['points'] = [[0, 0], [48, 0]]
(root / 'positional-points').write_text(json.dumps(wrong), encoding='utf-8')
duplicate = json.loads(json.dumps(base))
duplicate['placement']['points'] = [{'x_pt': 0, 'y_pt': 0}] * 2
(root / 'duplicate').write_text(json.dumps(duplicate), encoding='utf-8')
(root / 'truncated').write_bytes(b'{"placement":')
(root / 'invalid-utf8').write_bytes(b'\xff')
PY
CARGO_TARGET_DIR="$PWD/target" CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_RELEASE_DEBUG=0 RUSTFLAGS="-Cdebuginfo=0" PATH="$PWD/target/tools/bin:$PATH" cargo +nightly-2026-04-01 fuzz run --target-dir "$PWD/target" --sanitizer address chain_path_request .cache/fuzz/chain-path-b4c6ae1 -- -runs=10000 -seed=274810359 -max_len=16384 -timeout=5
```

These original vectors are tests of this API, not reference-product samples. The bound
does not exercise the full1MiB wire budget or establish a general JSON security audit.

## Capture PNG envelope

`capture_image` exercises the public `PngCapture::png_bytes` preflight, not a
general PNG/pixel decoder. The original fuzz framing is one MIME-selector byte
(even=image/png, odd=image/jpeg), widthLEu32, heightLEu32, then body bytes. Every
input tests canonical base64 encoding of the binary body. Valid UTF-8 bodies
also test verbatim raw/noncanonical base64; invalid UTF-8 is not normalized into
a different String. Missing header words become zero dimensions.

Accepted output must preserve the canonical-path payload exactly, satisfy an
independent complete chunk walk/IHDR/dimension-specific budget, repeat
deterministically and leave all capture metadata unchanged. The harness never
inflates DEFLATE, validates CRC or invokes GPU/GUI. Those remain separate
decoder/unit-test evidence. The test-only commands/base64 edges add no shipped
runtime dependency. Existing isolated lock package identities stay pinned.

A small reproducible original starting corpus can be generated with the already
pinned external interpreter; choose a new ignored directory each time:

```sh
"$BONDLINE_PYTHON" -B - <<'PY'
from pathlib import Path
import base64, io, struct
from PIL import Image
root = Path('.cache/fuzz/capture-image-local')
root.mkdir(parents=True, exist_ok=False)
image = Image.new('RGBA', (1, 1), (255, 0, 0, 255))
output = io.BytesIO()
image.save(output, format='PNG')
png = output.getvalue()
prefix = b'\0' + struct.pack('<II', 1, 1)
(root / 'original-pixel-bytes').write_bytes(prefix + png)
(root / 'original-pixel-base64').write_bytes(prefix + base64.b64encode(png))
(root / 'truncated').write_bytes(prefix + png[:7])
(root / 'noncanonical-pad').write_bytes(prefix + b'AB==')
(root / 'predecode-budget').write_bytes(prefix + b'!' * 100)
(root / 'zero').write_bytes(b'')
PY
CARGO_TARGET_DIR="$PWD/target" CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_RELEASE_DEBUG=0 RUSTFLAGS="-Cdebuginfo=0" PATH="$PWD/target/tools/bin:$PATH" cargo +nightly-2026-04-01 fuzz run --target-dir "$PWD/target" --sanitizer address capture_image .cache/fuzz/capture-image-local -- -runs=10000 -seed=418593271 -max_len=131072 -timeout=5
```

For a worktree without its own tools binary, select the existing pinned root
`target/tools/bin` read-only through PATH. Do not install another version or
share compiled target directories. `-Cdebuginfo=0` is the effective final Rust
flag despite cargo-fuzz's earlier line-table profile default; sanitizer,
coverage and debug assertions remain enabled.

The minimal Pillow recipe above is an independent own-profile seed recipe.
The recorded acceptance run uses a separate fresh 23-file corpus, including
six actual own CPU/PNG-producer byte/base64 seeds, exact stored-block boundary
bytes, and explicit hostile header/framing cases; see
[verification-capture-image-fuzz.md](../docs/verification-capture-image-fuzz.md)
for its generator/seed identities, actual outcome and limits. A max_len setting
does not prove the full 16.8MiB image bound was exercised. No hosted workflow,
native cache/window or security-audit claim follows from this local smoke.

## SMILES files

The complete-file parser has a separate target from the unchanged single-notation parser.
Start with a fresh ignored writable directory, keeping committed fixtures immutable:

```sh
mkdir -p .cache/fuzz/smiles-file
cp crates/io/tests/fixtures/smiles/*.smi .cache/fuzz/smiles-file/
python3 - <<'PY'
from pathlib import Path
root = Path('.cache/fuzz/smiles-file')
(root / 'multi.smi').write_bytes(b'\n \xffignored\r\nCC\t \xff\0name\t42\r\n\tCO ignored\nC#N id\nO last')
(root / 'later-error.smi').write_bytes(b'CC first\nC1 invalid\nCO last\n')
PY
PATH="$PWD/target/tools/bin:$PATH" cargo +nightly-2026-04-01 fuzz run --sanitizer address smiles_file_parse .cache/fuzz/smiles-file -- -runs=10000 -seed=143812957 -max_len=65536 -timeout=5
```

These two additional seeds are original inline vectors, not proprietary sample files.
Accepted inputs must preserve every byte and share source/index allocations on Clone.
The harness walks physical byte boundaries independently of the parser's iterator,
requires every meaningful line to be indexed, checks record ordinals/physical lines,
notation/separator/opaque-data/end spans, and reparses each bounded notation to compare
literal atom/bond counts. Missing ordinals cannot return a fallback record. Rejections
must give in-bounds absolute offsets. Retained-byte accounting must deduplicate aliases.

This checks internal invariants under mutation. Independent raw framing and external
per-record chemistry expectations live in the IO tests. Large file/line/record quotas
and the 65,536-byte notation versus larger framed input boundary have separate unit
tests; a 65,536-byte fuzz ceiling does not exercise those larger bounds. It is a smoke
check, not a security audit, chemical validity proof or native application workflow.
Actions remains disabled during development; this command runs locally.

## SMILES notation

```sh
mkdir -p .cache/fuzz/smiles
cp crates/io/tests/fixtures/smiles/*.smi .cache/fuzz/smiles/
PATH="$PWD/target/tools/bin:$PATH" cargo +nightly-2026-04-01 fuzz run --sanitizer address smiles_parse .cache/fuzz/smiles -- -runs=10000 -seed=703592681 -max_len=65536 -timeout=5
```

The three original17/21/21-byte and three aromatic9/4/18-byte seeds retain their
independent topology/metadata and SHA identities in their fixture README. Copy them into a fresh ignored writable corpus;
never point libFuzzer at committed fixtures. The normal full IO dependency graph is used,
without a reduced parser manifest, new dependency/tool pin or GPU initialization.

Accepted input must retain all source bytes and share source/atom/bond allocations on
Clone. The harness independently checks source/token bounds, implicit versus bracket H,
literal isotope/H ranges, supported bond tokens, distinct valid endpoints/unique edges,
both original numeric ring declarations and dot-chain resource counts. Rejected
input has a typed error and in-bounds byte offset. Fixed-size/depth limits also have
explicit unit cases; a max_len ceiling alone does not establish large-input coverage.

The initial checkpoint below was nonaromatic/source-only. Current checks also cover
literal aromatic markers and the named normalization/export invariants below, not general
chemical validity, live inferred H, radicals, stereo/aromaticity perception, canonicalization,
application Open/Save or native UI evidence. Independent published-rule/literal/external RDKit tests
remain separate; not every accepted mutation is sent to a chemistry consumer.

First normal-graph local ASan smoke exits0:10000 runs with the settings above,
cov534/features1205,148inputs/1602B/RSS67MiB,1second. Largest exercised input21B;
large/deep/dense unit boundaries are separate evidence. Exact logs/settings/hashes and
the corrected pre-build clippy command are recorded in docs/verification-smiles.md.
Hosted execution of the new seventh target is still pending at this checkpoint.

## CDXML syntax

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
The standalone syntax checks do not infer chemistry or resolve references. The current
target also exercises limited IO snapshots and new-tree construction as described below;
it performs no filesystem IO. Independent raw/RDKit expectations remain separate tests.
The original explicit ethane seed is 65750 bytes, including a 65535-byte opaque extended
property. The former 65536-byte run limit could not represent that complete scene; the
current 131072-byte bound deliberately includes it. Both original fixture identities and
their generator/source provenance are recorded in the fixture README. An accepted binary
mutation is syntax/source evidence, not independent chemical interoperability for that mutation.
The CI sanitizer job runs all seven parser targets; these bounded runs are not exhaustive input
coverage or proof of application CDX support.

### Canonical separate file ends

The current target classifies encoded roots with `Framing::has_encoded_document()`.
Before reconstruction it counts every source property independently of the body filter;
only genuine root openings/ends and the separate FileEnd may be omitted. This protects
the implicit envelope's first0805 property. New Writer output is canonical explicit-root
syntax, never an imported-file Save. Limited IO projection retains complete source and
authored unknown H; changed snapshots and lossy export remain refused.

Reproduce the ten original/source-derived seed declarations in a fresh ignored directory:

```sh
"$BONDLINE_PYTHON" -B - <<'PY'
from pathlib import Path
root = Path('.cache/fuzz/cdx-file-end-local')
root.mkdir(parents=True, exist_ok=False)
fixtures = Path('crates/cdx/tests/fixtures')
for name in ('unknown-implicit.cdx', 'original-ethane-explicit.cdx', 'original-ethane-header-document.cdx'):
    (root / name).write_bytes((fixtures / name).read_bytes())
external = (fixtures / 'external-ethane-indigo-1.47.0.cdx').read_bytes()
unknown = (fixtures / 'unknown-implicit.cdx').read_bytes()
canonical = unknown[:12] + bytes(16) + unknown[28:] + bytes(2)
seeds = {
    'external-original.cdx': external,
    'canonical-unknown-file-end.cdx': canonical,
    'canonical-explicit-file-end.cdx': canonical[:28] + bytes.fromhex('008000000000') + canonical[28:],
    'extra-file-end.cdx': external + bytes(2),
    'missing-file-end-legacy.cdx': external[:-2],
    'partial-file-end.cdx': external[:-1],
    'nonzero-file-end.cdx': external[:-2] + bytes((1, 0)),
}
for name, data in seeds.items():
    (root / name).write_bytes(data)
PY
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_RELEASE_DEBUG=0 RUSTFLAGS="-Cdebuginfo=0" PATH="$PWD/target/tools/bin:$PATH" cargo +nightly-2026-04-01 fuzz run --target-dir "$PWD/target/cdx-file-end/fuzz-target" --sanitizer address cdx_parse .cache/fuzz/cdx-file-end-local -- -runs=10000 -seed=721533879 -max_len=131072 -timeout=5
```

The missing-final-word seed is deliberately valid legacy syntax. Canonical FileEnd
acceptance does not make every legacy prefix a truncation error. Original UNKNOWN's
nonzero reserved bytes and appended-end negatives stay unchanged.

The exact external seed first reproduces11versus12 properties in the old harness
classifier; this is an assertion defect, not an ASan memory error or mutation discovery.
After the semantic fix, fixed-input execution and a separate actual10000-mutation ASan
run pass: ten initial files/133005B, coverage1054/features2322,95 retained inputs/reported
209KiB/RSS221MiB,1second. Largest exercised input65750B;64MiB/record/depth boundaries
have separate unit evidence. [Integration evidence](../docs/verification-cdx-file-end-integration.md)
pins the sources, logs, oracle qualifications and native scope. No lock, old fixture,
runtime dependency or fetched corpus change; no security-audit claim.

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


### Fixed-H normalization and explicit-H export

The existing SMILES target now also checks successful normalization directly against
accepted source IR: every dense ID, element, original positive isotope, neutral charge,
literal bracket H or independently counted implicit H, and each endpoint/order. Source
bytes/IR remain unchanged. Typed unsupported normalization is allowed; it is not a general
chemical-validity oracle. Successful export may refuse only a documented resource budget.

A separate cloned test graph assigns unique positive isotope markers from dense atom IDs.
Generated notation must parse, preserve every marked atom/literal H and every undirected
edge/order, and repeat deterministically. Markers are original test metadata, not claims
that those nuclides physically exist. They permit complete endpoint comparison without
assuming the writer preserves source traversal order. External original-vector consumers
remain separate; this internal invariant is not independent chemical interoperability.
All prior parser/source/token assertions stay byte-for-byte recoverable.

Final local normal-graph ASAN45229 passes10000 at seed703592681/max_len65536/timeout5.
Fresh ignored corpus: three unchanged licensed seeds, original4096-carbon chain and small
multiring notation; five files/4176B, largest4097B. Finalcov1867/features6734,246retained
inputs/reported309KiB/RSS474MiB,6seconds. Configured65536 is a ceiling, not exercised
large-input proof. Standard manifest/unchanged fuzz lock, cargo-fuzz0.13.2/libFuzzer0.4.13/
nightly-2026-04-01, own absolute fuzz/target, incremental0/release-debug0. Full-graph stable
all-target clippy-Dwarnings and both rustfmt checks pass. CI keeps its three original seeds;
that hosted corpus is distinct from this five-seed local run. Exact logs/review are in
verification-smiles-export-fuzz.md. No new target/runtime dependency/model or app behavior.


### Aromatic notation and explicit ring tokens

Current source preserves lowercase atom markers and Aromatic orders. Omitted bonds default
to Aromatic only when both source markers are aromatic. An explicit ordinary or ring token
wins; Bond.bond_range=None alone does not prove omission because ring declarations retain
their own ranges. An initially incorrect fuzz assertion was independently caught, reproduced
on C:1CCC1, then fixed and retested. See verification-smiles-aromatic.md for actual RED/GREEN;
this was a harness false positive, not a parser/memory defect.

All prior source/H/ring/normalization/export assertions remain. Normalization success must
contain no aromatic marker/bond; generated restricted export must never invent one. CI now
copies all six committed seeds (original three plus aromatic cycle, uppercase explicit
colon and mixed explicit-ring roles). Fetched corpus still pins only the original three.
Local fresh9-seed normal-ASAN64957 includes those six plus original4096-atomC andc chains
and multiring data:8304B initially, max4097B.10000runs, seed703592681/max_len65536/timeout5,
exit0, cov1979/features7021,331retained inputs/reported406KiB/RSS422MiB,8seconds. Largest
exercised input4097B; the64KiB ceiling is not coverage. Same unchanged full fuzz manifest/
lock, pinned tools/nightly, own absolute fuzz/target/incremental0/release-debug0. Stable
all-target clippy/fmt pass. This bounded smoke is not exhaustive chemistry or a security audit.


## Source-aware MOL display coverage

The existing mol_parse and mol_v3000_parse targets also exercise the real
`bondline_io::display::drawing_page_scene` after their existing strict-projection
gate. A successful StoredDocument construction supplies its actual A4 Page/IDs
and the unchanged identity frame (1pt/unit, origin0,0). A 1024² scene may succeed
or honestly refuse; both outcomes must preserve the full owner, authored model
and exact original MOL bytes. Pre-cancellation must return cancelled and leave
the same data intact. Every prior parser/span/resource/projection/writer check
and early writer refusal remains in place. This adds no target or dependency.

Fresh ignored source seeds must be separate from mutation corpora. This recipe
recreates all eight recorded initial seeds, including two original MIT OR
Apache-2.0 interior vectors. It keeps the ordinary raw MOL spans unchanged:

```sh
"$BONDLINE_PYTHON" -B - <<'PYSEED'
from pathlib import Path
from hashlib import sha256
import shutil
base=Path('.cache/fuzz/mol-display-example')
source=Path('crates/io/tests/fixtures/sdf-preview/valence-absent-mixed.sdf').read_bytes()
assert sha256(source).hexdigest()=='e5cbdc14329c59bad55148c5de37058de90f651af10966c30cf8b89b3b5efd39'
inputs={
    'mol_parse': [
        ('rdkit-CH3.mol',Path('crates/io/tests/fixtures/mol/rdkit-CH3.mol').read_bytes()),
        ('ethane-v2000.mol',Path('crates/io/tests/fixtures/mol/ethane-v2000.mol').read_bytes()),
        ('ordinary-CN-v2000.mol',source[410:713]),
        ('interior-CN-original-v2000.mol',b'''Original interior CN fuzz seed
  Bondline original
Independent identity-frame coordinates
  2  1  0  0  0  0            999 V2000
  100.0000 -200.0000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0
  136.0000 -200.0000    0.0000 N   0  0  0  0  0  0  0  0  0  0  0  0
  1  2  1  0  0  0  0
M  END
'''),
    ],
    'mol_v3000_parse': [
        ('noncontiguous-cc.mol',Path('crates/io/tests/fixtures/v3000/noncontiguous-cc.mol').read_bytes()),
        ('continuations-and-extensions.mol',Path('crates/io/tests/fixtures/v3000/continuations-and-extensions.mol').read_bytes()),
        ('ordinary-CCO-v3000.mol',source[0:371]),
        ('interior-CCO-original-v3000.mol',b'''Original interior CCO fuzz seed
  Bondline original
Independent identity-frame coordinates
  0  0  0  0  0  0            999 V3000
M  V30 BEGIN CTAB
M  V30 COUNTS 3 2 0 0 0
M  V30 BEGIN ATOM
M  V30 10 C 100 -200 0 0
M  V30 40 C 136 -200 0 0
M  V30 90 O 136 -164 0 0
M  V30 END ATOM
M  V30 BEGIN BOND
M  V30 7 1 10 40
M  V30 15 1 40 90
M  V30 END BOND
M  V30 END CTAB
M  END
'''),
    ],
}
for target,files in inputs.items():
    seeds=base/target/'seeds';corpus=base/target/'corpus'
    seeds.mkdir(parents=True,exist_ok=False);corpus.mkdir()
    for name,data in files:
        (seeds/name).write_bytes(data)
        shutil.copyfile(seeds/name,corpus/name)
PYSEED
```

These exact raw origins mostly reach paper/metadata refusal under the fixed
identity frame. Do not transform them and call them the unchanged source.
The separately authored interior CN/CCO seeds above reach successful scenes,
with literal coordinates and unchanged external consumer checks. Initial source
hashes match the recorded run; final mutation corpora are not promised identical.
The original generator, scene classifications and mutation corpus remain ignored
evidence; see
[verification-mol-display-fuzz.md](../docs/verification-mol-display-fuzz.md).

Use the existing installed pinned cargo-fuzz path read-only if this checkout has
no target/tools/bin, and this worktree's own absolute target directory:

```sh
CARGO_TARGET_DIR="$PWD/target" CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_RELEASE_DEBUG=0 RUSTFLAGS="-Cdebuginfo=0" PATH="$PWD/target/tools/bin:$PATH" cargo +nightly-2026-04-01 fuzz run --target-dir "$PWD/target" --sanitizer address mol_parse .cache/fuzz/mol-display-example/mol_parse/corpus -- -runs=10000 -seed=619734821 -max_len=65536 -timeout=5
CARGO_TARGET_DIR="$PWD/target" CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_RELEASE_DEBUG=0 RUSTFLAGS="-Cdebuginfo=0" PATH="$PWD/target/tools/bin:$PATH" cargo +nightly-2026-04-01 fuzz run --target-dir "$PWD/target" --sanitizer address mol_v3000_parse .cache/fuzz/mol-display-example/mol_v3000_parse/corpus -- -runs=10000 -seed=937155923 -max_len=65536 -timeout=5
```

The final rustc debug0 flag retains sanitizer/coverage/debug assertions. This is
bounded scene-construction and source-preservation smoke, not GPU/PNG/native
rendering, whole-chemistry correctness, maximum-input coverage or a security audit.
The unsupported V3000 extension remains a syntax seed, with separately recorded
strict-projection and external-consumer refusal. No committed fixture, oracle,
runtime source, root/isolated lock or workflow changes.


## Centered multiple-bond MOL seeds

The existing two MOL targets now reach centered Double/Triple scenes without changing
parser/writer invariants. The following original MIT OR Apache-2.0 recipe reproduces
the twelve exact initial sources from [the bounded run](../docs/verification-multiple-commands.md):
four successful identity-frame scenes and two honest scene refusals per version.
Source copies stay separate from writable mutation corpora. Run once from the repository
root; the fresh destination deliberately refuses overwrite.

```sh
"$BONDLINE_PYTHON" -B - <<'PYSEED'
# SPDX-License-Identifier: MIT OR Apache-2.0
# Original literal seeds; run from the repository root, never a committed fixture directory.
from pathlib import Path
from hashlib import sha256

root = Path('.cache/fuzz/multiple-bond-example')
root.mkdir(parents=True, exist_ok=False)
specs = [
    ('double-horizontal', 'positive', 'CC', [(120,-200,0),(168,-200,0)], [2]),
    ('triple-horizontal', 'positive', 'CC', [(120,-260,0),(168,-260,0)], [3]),
    ('mixed-terminal', 'positive', 'CCC', [(120,-320,0),(168,-320,0),(192,-368,0)], [1,2]),
    ('triple-diagonal-phase', 'positive', 'CC', [(240.25,-400.125,0),(288.25,-464.125,0)], [3]),
    ('coincident-axis', 'negative', 'CC', [(120,-520,0),(120,-520,0)], [2]),
    ('unresolved-phosphorus', 'negative', 'CP', [(120,-580,0),(168,-580,0)], [1]),
]
for version, target in [(2,'mol_parse'), (3,'mol_v3000_parse')]:
    for name, group, symbols, positions, weights in specs:
        lines = ['Bondline original '+name, '  Bondline          2D',
                 'Original independent multiple-bond fuzz source']
        n, m = len(positions), len(weights)
        if version == 2:
            lines.append(f'{n:3d}{m:3d}  0  0  0  0            999 V2000')
            for symbol, (x,y,z) in zip(symbols,positions):
                lines.append(f'{x:10.4f}{y:10.4f}{z:10.4f} {symbol:<3} 0  0  0  0  0  0  0  0  0  0  0  0')
            for i, weight in enumerate(weights):
                lines.append(f'{i+1:3d}{i+2:3d}{weight:3d}  0  0  0  0')
        else:
            ids = [17,41,73]
            lines += ['  0  0  0  0  0  0            999 V3000', 'M  V30 BEGIN CTAB',
                      f'M  V30 COUNTS {n} {m} 0 0 0', 'M  V30 BEGIN ATOM']
            for i, (symbol,(x,y,z)) in enumerate(zip(symbols,positions)):
                lines.append(f'M  V30 {ids[i]} {symbol} {x:.15g} {y:.15g} {z:.15g} 0')
            lines += ['M  V30 END ATOM','M  V30 BEGIN BOND']
            for i, weight in enumerate(weights):
                lines.append(f'M  V30 {99+i*32} {weight} {ids[i]} {ids[i+1]}')
            lines += ['M  V30 END BOND','M  V30 END CTAB']
        data = ('\n'.join(lines+['M  END'])+'\n').encode('ascii')
        source = root/target/'source'/group/(name+'.mol')
        writable = root/target/'corpus'/(name+'.mol')
        for path in [source,writable]:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
        print(target,group,name,len(data),sha256(data).hexdigest())
PYSEED
```

Use the already pinned cargo-fuzz0.13.2/nightly2026-04-01 and an absolute isolated
target. Existing source/oracle/fixture/golden/locks remain unchanged:

```sh
CARGO_TARGET_DIR="$PWD/fuzz/target" CARGO_INCREMENTAL=0 CARGO_PROFILE_RELEASE_DEBUG=0 RUSTFLAGS="-Cdebuginfo=0" PATH="$PWD/target/tools/bin:$PATH" cargo +nightly-2026-04-01 fuzz run --target-dir "$PWD/fuzz/target" --sanitizer address mol_parse .cache/fuzz/multiple-bond-example/mol_parse/corpus -- -runs=10000 -seed=4411 -max_len=65536 -timeout=10 -rss_limit_mb=2048 -max_total_time=600
CARGO_TARGET_DIR="$PWD/fuzz/target" CARGO_INCREMENTAL=0 CARGO_PROFILE_RELEASE_DEBUG=0 RUSTFLAGS="-Cdebuginfo=0" PATH="$PWD/target/tools/bin:$PATH" cargo +nightly-2026-04-01 fuzz run --target-dir "$PWD/fuzz/target" --sanitizer address mol_v3000_parse .cache/fuzz/multiple-bond-example/mol_v3000_parse/corpus -- -runs=10000 -seed=4411 -max_len=65536 -timeout=10 -rss_limit_mb=2048 -max_total_time=600
```

The recorded ASan runs each execute10000 iterations; observed maxima are404/388bytes,
not the65536 ceiling. Twelve fixed replays and independent source/scene classifications
are separate checks. This is bounded parser/display smoke, not GPU/native raster, broad
chemistry, exhaustive coverage or a security audit. Fresh mutation results may differ.
