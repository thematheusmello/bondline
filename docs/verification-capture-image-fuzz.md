# Capture PNG envelope fuzz smoke

2026-11-14. Additional isolated parser smoke for the public
`PngCapture::png_bytes` contract in
[verification-preview-capture-contract.md](verification-preview-capture-contract.md).
No production/core/UI code or old target is changed. This is not native preview,
CRC/pixel-decoder, broad security or maximum-image coverage evidence.

Root's receiving check verified all five files,292 protected identities, ten logs,
seed-maker/binary/input receipts and the isolated lock delta. The only Root-side
difference in protected code is the separately verified UI test-module declaration;
it does not enter this fuzz dependency graph. All165 old isolated package identities
are retained,164 full package records are exact and only the fuzz package gains its
two edges;12 required test-only records are added. The root runtime lock is unchanged.

The ninth future sanitizer step is configured in the disabled workflow. YAML parses
locally; its exact standard-library Python seed recipe (only destination changed)
generates six original inputs. Independent pinned Pillow verifies the70-byte1×1 red
RGBA PNG, and the actual ASan binary replays all six successfully. This smaller future
recipe is separate from the recorded23-seed/10000-run corpus below. No workflow was
enabled/dispatched/run. The recipe follows public
[PNG Third Edition IHDR/IDAT framing](https://www.w3.org/TR/2025/REC-png-3-20250624/),
not proprietary sample bytes. The source/chunk preflight still does not validate CRC.

## Target contract and independent seed expectations

The original test input prefix was defined before implementation:

```text
byte0       MIME selector: even=image/png, odd=image/jpeg
bytes1..5   width, little-endian u32
bytes5..9   height, little-endian u32
bytes9..    payload
```

Every input canonical-base64-encodes its binary payload and calls the public
reader twice. A valid UTF-8 payload also becomes the verbatim raw/noncanonical
base64 String and is called twice. Invalid UTF-8 is not lossily rewritten; the
binary path still runs. Missing prefix words resolve to zero dimensions. No
image decoder, Engine, GPU, account or UI is invoked per fuzz input.

Accepted canonical bytes must equal the exact original payload. Accepted output
faces independent signature/header assertions, a complete variable-length
chunk walk, RGBA8/noninterlace facts and the unchanged dimension-specific bound.
It must have exactly IHDR13, one nonempty IDAT and IEND0, without extra bytes.
Both success and typed error must repeat deterministically; MIME, dimensions,
body, backend/adapter and presentation metadata remain unchanged.

These are internal invariants, not an external compatibility oracle for every
mutation. In particular, the harness deliberately does not check CRC, DEFLATE
or decoded scanlines. A valid envelope with corrupt compressed data can be
accepted unchanged by this preflight; a real image decoder still has work to do.

The fresh ignored source seeds were generated using actual original CPU registry
captures at 1×1 and 8×8, and the unchanged own RGBA8 PNG writer at 64×255 with
original fixed-seed random pixels. Each is present as binary payload and raw
base64, giving six valid transport seeds. The 64×255 image has exactly 65,535
filtered bytes and an actual 65,603-byte stored-block output. Its base64 input
plus prefix is 87,481 bytes, below the chosen 131,072-byte smoke ceiling.

The existing pinned Pillow 12.3.0 independently verified each PNG's CRC, RGBA mode,
dimensions and framing. The 8×8 registry capture matches every byte of an original
literal page grid; Python independently reproduces every random RGBA byte for
the 64×255 writer output. Literal source/header expectations do not come from
the fuzz target's returned ranges.

Seventeen further original seeds cover empty input, missing prefix words,
zero/maximum-u32 dimensions, wrong MIME, truncated signature/tail, canonical-pad
and alphabet failures, a pre-decode size failure, forged IHDR dimension/depth,
maximum chunk length, an auxiliary chunk and trailing data. A corrupt-IDAT seed
has independently confirmed Pillow failure while retaining acceptable envelope
framing, preserving the decoder-role distinction.

Initial source seeds: 23 files/155,800 bytes; maximum 87,481 bytes. LibFuzzer reports
22 nonempty initial files because the zero-byte seed is excluded from its file
count. The exact original names, bytes, expected classifications, source hashes,
Pillow observations and separately writable corpus are recorded in ignored
`target/capture-image-fuzz-seeds.json` (8,628 bytes, SHA-256
`d4810a8f64c5338b6fb1e78b5692504bf623c862e87e715a4eba9750cfab5434`).
Source seeds stay immutable; only their fresh ignored corpus copies are mutated.

The actual seed maker is a disposable full-dependency-graph Rust tool under
target/capture-seed-maker, using existing commands/IO/render APIs and explicit
CPU capture. Its source, manifest, lock, binary and build log are pinned in the
handoff. This adds no committed helper crate or shipped dependency. The smaller
independent Pillow recipe in fuzz/README.md is a reproducible starting recipe,
not a claim that its corpus is identical to this recorded run.

## Lock and source preservation

Before changing the real isolated lock, an ignored full-graph manifest copy
resolved the commands and direct base64 test-only edges offline. The real
manifest then passed locked/offline metadata resolution. All 165 old package
identities, versions, sources, checksums and dependency records remain exact,
except the fuzz package's two declared new edges. No old package moved version;
the root Cargo.lock is unchanged.

Exactly 12 required package nodes are added to the isolated test lock:

| Package | Version |
|---|---|
| base64 |0.22.1|
| bondline-commands |0.1.0|
| dyn-clone |1.0.20|
| itoa |1.0.19|
| pollster |0.3.0|
| ref-cast / ref-cast-impl |1.0.27 each|
| schemars / schemars_derive |1.2.3 each|
| serde_derive_internals |0.30.0|
| serde_json |1.0.154|
| zmij |1.0.23|

The direct base64 test edge adds no node beyond commands' required transitive
dependency and avoids implementing another codec in the harness. The package
is publish=false and excluded from the application workspace. No shipped
runtime dependency, dependency allowlist, installer, font, fixture or asset
changes. Existing eight fuzz-target bodies and all protected production bytes
stay exact. The new target is ninth; Root handles the workflow definition later
while Actions remains disabled.

Probe delta receipt: target/capture-lock-probe-delta.json, 4,193 bytes, SHA-256
`2bfbbe718f529bedd25d660fb66c49c9266ac946c9528ce7614d931c3bd7a169`.
The original isolated manifest/lock/README and 295-file protected-source manifest
are retained under target for independent comparison.

## Actual RED/GREEN and bounded run

Pinned tools: existing cargo-fuzz 0.13.2, libfuzzer-sys 0.4.13,
nightly-2026-04-01 (rustc 1.96.0-nightly 48cc71ee8), stable checks 1.94.1,
existing Root pinned Python/Pillow. A bare initial cargo-fuzz lookup failed
because the pinned binary was not on the global PATH; selecting the existing
Root target/tools/bin corrected setup. Nothing was installed.

The intended missing-target RED exits 1 before implementation:
`cargo +nightly-2026-04-01 fuzz build --sanitizer address capture_image` reports
no bin target named capture_image and lists the eight old targets. The complete
output is target/capture-image-fuzz-red.log.

ASan build 66182 exits 0 using the real full graph. Seed-maker 5009 exits 0; it uses
normal stable compilation and CPU/PNG code only. Actual ASan run 12179 exits 0:

```sh
CARGO_TARGET_DIR=/Users/sarenaberg/Workspace/doozle/open-source-world/bondline-preview-fuzz/target CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_RELEASE_DEBUG=0 RUSTFLAGS="-Cdebuginfo=0" PATH="/Users/sarenaberg/Workspace/doozle/open-source-world/bondline/target/tools/bin:$PATH" cargo +nightly-2026-04-01 fuzz run --target-dir /Users/sarenaberg/Workspace/doozle/open-source-world/bondline-preview-fuzz/target --sanitizer address capture_image /Users/sarenaberg/Workspace/doozle/open-source-world/bondline-preview-fuzz/.cache/fuzz/capture-envelope-20261114/corpus -- -runs=10000 -seed=418593271 -max_len=131072 -timeout=5 -artifact_prefix=/Users/sarenaberg/Workspace/doozle/open-source-world/bondline-preview-fuzz/target/capture-image-fuzz-artifacts/
```

Result: 10,000 runs, coverage 488/features 1538, 88 retained inputs/reported 715KiB,
largest exercised input 87,481 bytes, reported RSS 452MiB, 2 seconds after build.
No crash, timeout, sanitizer report, assertion failure or failure artifact.
The 131,072 setting is a ceiling; neither it nor the full 16.8MiB image boundary
is claimed as exercised by this smoke. The protected core unit tests separately
cover maximum image/dimension and exact envelope budgets.

Verbose build output records -Zsanitizer=address, coverage instrumentation and
debug assertions. Cargo-fuzz passes an earlier line-tables-only profile flag;
the final Rust -Cdebuginfo=0 override is present on the actual rustc invocations.
Thus Cargo's summary still says optimized+debuginfo while effective Rust debug
information is 0. No sanitizer/coverage/assertion or parser quota was disabled.

Scoped stable commands also pass:

```sh
cargo +1.94.1 check --locked --manifest-path fuzz/Cargo.toml --all-targets
cargo +1.94.1 clippy --locked --manifest-path fuzz/Cargo.toml --all-targets -- -D warnings
cargo +1.94.1 fmt --manifest-path fuzz/Cargo.toml --all -- --check
```

Check 28723, clippy and fmt exit 0 across all nine isolated targets, without source
correction after the first ASan pass. Exact logs, tool/binary/seed/lock-delta/
source hashes and terminal handles are frozen in the ignored handoff. No old
target smoke, broad workspace gate, GPU, GUI/input/activation, reference runtime,
Actions workflow, Git operation or shared-artifact cleanup ran in this slice.

A separate ASan fixed-input replay 67216 exits 0 for all 23 original files,
explicitly including the zero-byte input. Its log correctly states that no
mutation fuzzing occurred during that replay; these 23 executions are not added
to the 10,000-run result. The original source-seed hashes stay unchanged after
both runs. The two artifact directories contain no failure files.
