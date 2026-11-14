# MOL source-aware display fuzz coverage

2026-11-14. Coverage-only additions to the existing mol_parse and mol_v3000_parse
targets, using the accepted shared IO producer from
[verification-mol-display-model.md](verification-mol-display-model.md). The task's
short v3000_parse spelling was corrected to the actual existing target; nothing
is added or renamed. No production source or behavior is changed here.

## Added checks and preserved contract

After the existing strict elemental projection succeeds, each target attempts a
source-backed StoredDocument with its existing identity transform: 1pt/raw unit,
origin(0,0), A4. On accepted ownership, it calls the real drawing_page_scene at
1024×1024 with the actual PageId. Either a scene or an honest refusal must leave
the complete owner, original source bytes, authored model and baseline unchanged.
A second call with pre-cancellation must return cancelled and preserve the same
data. No source text is added to ordinary output, nor is source backing replaced
with a projection. StoredDocument construction may explicitly refuse input.

All existing parser/resource/source-span/unsupported-projection checks, V2000
coordinate-quantization bounds, V3000 exact graph checks and writer refusals
remain unchanged. The added display calls precede the existing writer's early
return; writer refusal cannot silently skip the new check. No second manual
parser/writer path or shared helper is introduced. The existing commands test
edge supplies the existing Cancellation type; no dependency/manifest/lock change.

The production resolver still uses its original restricted neutral default-H
profile, authoritative literal counts, source completeness, existing 4096-hit
preflight, actual coordinates/IDs and typed rendering/scale/ink refusals.
Neither target asserts a general chemistry result for mutated inputs. Existing
independent analytic/consumer/renderer checks remain separate evidence.
This constructs real GraphScene data/labels; it does not render a CPU frame/PNG,
initialize a GPU, launch a native app, edit a session or test a file publication.

## Independent initial sources and actual branch evidence

Fresh ignored immutable source directories are separate from writable corpus
copies. Each target starts with four files:

| Target | Unchanged original sources | Separate original interior seed | Initial bytes / max |
|---|---|---|---|
| mol_parse | BSD-3-Clause RDKit CH3, original ethane, raw CN span410..713 from original740B SDF | 299B CN at raw(100,-200),(136,-200) | 1054 /303 |
| mol_v3000_parse | original sparse-ID CC, original continuation/extension, raw CCO span0..371 from original740B SDF | 374B CCO at raw(100,-200),(136,-200),(136,-164) | 1581 /492 |

The shared original SDF remains SHA-256
`e5cbdc14329c59bad55148c5de37058de90f651af10966c30cf8b89b3b5efd39`.
Its raw MOL spans remain exact, with their original VAL-absent bytes, coordinates
and source IDs. Their identities are not relabeled as translated sources.
The two interior seeds are independently written literal CTfile bytes in the
ignored seed generator, not our writer output or copied reference files.
Their seed names, bytes, hashes, generator and consumer results are pinned in
`target/mol-display-fuzz-seeds.json`: 17177 bytes, SHA-256
`82b3592818d7d126cd8c41c19fd8b8bb300badc5909096ea31794d4cf96e0d05`.
Original fixture provenance/licenses remain in their unchanged fixture READMEs.

The unchanged external mol_oracle.py with pinned RDKit 2026.09.2 positively
consumes seven inputs. Literal CN/CH5N/H[3,2] and CCO/C2H6O/H[3,2,1], exact raw
coordinates and raw-source hashes are independently asserted; ethane likewise
has literal CC/C2H6/H[3,3]. No model/writer round trip supplies these expectations.
The original continuation/stereo-extension 492B syntax fixture is different:
unchanged v3000_oracle.py/RDKit rejects it with an index-range error. Bondline's
strict mapper also refuses this known unsupported syntax. It remains an exact
syntax seed, not evidence of molecular/display compatibility.

The initial seed-generator probe mistakenly required external success for that
unsupported extension as well. Its failed source/log are preserved; the final
helper explicitly records that negative diagnostic instead of claiming success.
No fixture, old oracle, fuzz assertion or production behavior is changed to
suppress it. Every projectable seed retains positive literal consumer checks.

An authorized ignored seed inspector uses the exact accepted IO producer and
unchanged dependency graph, with only absolute manifest-path relocation. Its
lock remains byte-identical to the committed isolated lock. --locked --offline
run 30629 exits 0. Actual identity-frame outcomes are two interior scene successes,
five honest source-origin/metadata scene refusals, and one strict-owner refusal.
All accepted owners also pass cancellation and exact retention checks. The
inspector emits metadata/status/counts only; input hashes are checked before and
after. It supplies branch evidence, not an independent rasterizer or chemistry
oracle. Its source/manifest/lock/binary/output hashes are retained in the handoff.
No disposable helper is committed.

## Bounded ASan execution

Existing tools: cargo-fuzz 0.13.2, libfuzzer-sys 0.4.13, nightly-2026-04-01 and
stable 1.94.1. The pinned cargo-fuzz binary is reused read-only from Root's
target/tools/bin; nothing is installed. All artifacts use this worktree's
absolute target. CARGO_INCREMENTAL=0, dev/test/release debug0 and final
RUSTFLAGS=-Cdebuginfo=0 are explicit. Verbose rustc commands retain address
sanitizer, coverage instrumentation and debug assertions. Cargo's summary still
says optimized+debuginfo because cargo-fuzz supplies an earlier line-table flag;
the final effective Rust debug-information override is 0.

Coverage tooling extends already tested behavior, so no artificial RED is
created. First real ASan builds 16479/68281 exit 0, without a target correction.
Production behavior REDs remain in the linked IO evidence.

| Target / session | Runs | Fixed seed | max_len / timeout | Coverage / features | Retained corpus / RSS | Fuzz time | Largest exercised |
|---|---|---|---|---|---|---|---|
| mol_parse /49818 |10000|619734821|65536 /5s|3416 /5932|188 /40KiB /75MiB|1s|303B|
| mol_v3000_parse /98341 |10000|937155923|65536 /5s|3050 /6033|197 /58KiB /80MiB|1s|492B|

Times exclude compilation; sizes/RSS/retained counts are libFuzzer reports.
The configured max_len is a ceiling. Neither 65536-byte nor maximum-format/graph
coverage is claimed. No crash, timeout, sanitizer report or assertion failure
occurred. Failure artifact directories remain empty. Full exact arguments/logs,
source hashes and final corpus indices are recorded under target.

Separate fixed-input ASan replays execute all four original seeds per target.
Their logs explicitly state that mutation fuzzing was not performed during
replay. These eight fixed executions are not added to either 10000-run count.
Source-seed hashes remain exact after fuzzing/replay; committed inputs are never
used as writable corpora. The worker initially appended a smaller original/raw
recipe to fuzz/README.md. Root's receiving update includes both complete original
interior literals, so the documented recipe now recreates all eight initial files.
Executing that exact block verifies all16 immutable-source/mutation-copy identities
against the recorded inputs. Recipe SHA256
`6657f10ace75b068c5294f2de69748ef3f272d6e8b8f43a4f413b57174818f50`;
receipt `target/mol-display-readme-seed-recipe-receipt.json`. Final mutated corpora
are not promised identical; no new mutation runs are counted for this doc check.

## Scoped final gates and preservation

```sh
cargo +1.94.1 check --locked --offline --manifest-path fuzz/Cargo.toml --bin mol_parse --bin mol_v3000_parse
cargo +1.94.1 clippy --locked --offline --manifest-path fuzz/Cargo.toml --bin mol_parse --bin mol_v3000_parse -- -D warnings
cargo +1.94.1 fmt --manifest-path fuzz/Cargo.toml --all -- --check
```

Relevant-target check 40692 exits 0; warnings-denied clippy and isolated rustfmt
exit 0. The Rust target bodies are unchanged after their first ASan builds. All old target source
invariants are retained; removing only the added imports/display block restores
each original target exactly. README changes are append-only. Protected manifests
cover accepted IO production, runtime/root and isolated locks, old fixtures,
oracles, seven other fuzz targets and the approved golden. The manifest contains 304 existing inputs: only the three owned target/README
files change, and the other 301 are byte-exact. Exact hashes, all logs and terminal
handles are frozen in the ignored handoff.

No Git operation, hosted Actions, workflow edit, dependency update, source/asset
transplant, GUI/input/activation/polling, native window, cleanup or broad workspace
gate ran. This is bounded parser/owned-scene smoke, not security certification,
independent chemistry for mutations, GPU/native workflow or full M1 parity.
