# Independent multiple-bond command acceptance

2026-11-14. Independent owner in the isolated `bondline-multiple-tests` checkout,
initial base `a279bce`. This slice tests actual shared commands, files and offscreen
CPU/GPU replies. It supplies no native-window evidence or general multiple-bond parity.
GitHub Actions stays disabled; no desktop input, activation or installation is used.

## Independent expectations

`crates/commands/tests/multiple_bonds.rs` authors original fixed-column V2000 and V3000
C2 records directly, without a product writer: C at `(0,0,0)` and `(2,0,0)`, one order2
or3 bond, absent VAL/HCOUNT. V3000 uses nonsequential source IDs17/41/99. Open must retain
the entire source/version and authored H=None at exact model points `(72,72)/(120,72)`.
The unchanged external `io/tests/mol_oracle.py`, with pinned RDKit2026.09.2, independently
requires `C=C/C2H4/H[2,2]` or `C#C/C2H2/H[1,1]`, ordered bonds and exact raw positions.

The original adopted centered3w spacing is a Bondline policy, not a reference-product or
SDK default. The reference image starts with the protected literal Single font/paper/mask
scene and replaces its sole stroke with independently declared parallel strips. The new
original `multiple_pixels.py` independently calculates A4 viewport geometry and tests each
central pixel against the exact strip/gap inequalities. Full CPU/GPU RGBA must agree with
the reference. Collapsed-segment, wrong order and missing-H controls must differ. A real
GPU reply must identify its adapter. The literal producer is a shared font reference, not
an independent rasterizer; old FreeType evidence and the approved Single golden retain
their separate scope. Neither fonts nor existing oracles/goldens/fixtures are changed.

[IUPAC GR1.6](https://iupac.qmul.ac.uk/drawing/drawing.html#GR16) supports nearby parallel
segments; [GR1.10](https://iupac.qmul.ac.uk/drawing/drawing.html#GR110) distinguishes centered
and side-offset/context-sensitive behavior. [The research record](research/multiple-bond-plan.md)
contains the SDK pins and unresolved style questions. None of these establishes26.1 pixels.

## Authored chemistry and guards

Explicit JSON order tests require C2 H2/Double or H1/Triple and attached C3H6/C3H4 through
both actual saved CDXML and MOL. A new test-only inline Expat/RDKit probe checks every
CDXML node's literal H/position/charge and endpoint/order graph; the protected MOL consumer
checks actual saved MOL bytes. Both files reopen to the exact authored graph. Formula,
SMILES and ordered atom/bond/H expectations are literal, not generated from our model.
Independent reference-only RDKit checks confirmed canonical `C=CC`, `C#CC`, `C=C=C` before
the backend arrived; those checks are not successful Bondline workflow evidence.

Root clarified the planned extension scope before draft freeze: a terminal C with one
Single/Double/Triple and literal H=`4-existingWeight` may consume a requested weight no
larger than that H. Required `rectify_terminal_carbon` is retained; no inferred or arbitrary
override is repaired. Tests cover H2/H1 terminal→Single, Double→Double allene, insufficient-H
refusals and a valid Single/H3 terminal in a fragment containing a supported Double elsewhere.
The earlier unfrozen assumption that any Double elsewhere blocked extension was replaced
before execution under that explicit refinement. Committed invalid Double/H3, metadata,
unknown-H, degree, source, ID, geometry and history guards must remain meaningful.

Additional independent checks preserve explicit/default Single equality, closed order JSON
(including rejecting explicit null), exact failure state/redo, monotonic IDs on Undo branches,
MOL source authority, unknown CDXML spacing/position retention, complete historical capture
ownership across history/new Open/source removal, early/late cancellation and absent pages.
One JSON tool-mode coupling test checks transient selection/controls and ordinary Inspect
privacy while omitted headless order remains Single and explicit order remains authoritative.
It adds no duplicate gesture or worker framework.

## Actual initial RED

Before receiving any implementation, this command ran against unchanged production:

```sh
CARGO_TARGET_DIR=/absolute/path/bondline-multiple-tests/target \
CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
BONDLINE_PYTHON=/absolute/path/bondline/.cache/oracles/format-python/bin/python \
cargo test --locked -p bondline-commands --test multiple_bonds baseline_ -- --nocapture
```

Handle4207 exited101 with the two intended runtime failures:

- V2000 order2's unchanged external chemistry, exact unknown-H/coordinate/source checks
  and independent pixel-oracle/control self-check passed. Actual `render.capture` then
  returned `unsupported_content`, where the test requires a complete multiple-bond image.
- Explicit `bond.create.order="double"` returned `invalid_params`, where authoring is required.

Later Triple/V3000/GPU/Save/extension steps did not execute in this first failing loop.
The run compiled; warnings came from then-drafted unused helpers/imports, not a feature
compiler failure. Their uses are supplied by the additive acceptance cases.

`target/multiple-baseline-red.log`:6,107bytes, SHA-256
`f988289317c70f87b30168f8f58632b761d4069ea8c3dc9e7ffdf665df46347d`.
Initial Rust source:14,435bytes, SHA-256
`88c175e6fc60b986973d46343a23309589be99ceb310a95fc9306baf1c07b6e9`,
retained byte-exact in `target/multiple-red-source.rs`.
Initial pixel oracle:1,939bytes, SHA-256
`539b5d5bb0d7aa3754985a82e5fe46c12cafe49bcec604995315129618541b0d`,
also retained in `target/multiple-red-pixels.py`.

## Draft checkpoint

Six tests are drafted. GREEN execution awaits Root's real command/render/tool-mode backend;
there is no stub or missing future Rust API in these tests. The protected baseline records
281 old source/test/fixture/oracle/asset/manifest identities in
`target/multiple-protected-baseline.json`, SHA-256
`170473fd977cb87461eba66cdb74b4f6c0aadc944874fa3af1d358c9464cf88d`.
Receiving production and deliberately upgraded old rendering tests must be classified
separately; no old input or assertion was edited by this owner. No new full-workspace,
Windows/Linux, hosted or native-window result is claimed. No worker job remains live.

## Independent CLI/stdio follow-on

Two new adapter files add actual CLI command discovery/ordered creation and extension,
CDXML Save/reopen, exact graph/H/history checks and independently consumed actual MOLs;
the stdio workflow also checks tool discovery, transient selection/selected controls,
repeat-selection no-op, ordinary Inspect privacy, typed-null failure retention and
Single continuation after newly authored Double/Triple. It reuses the existing private
stdio `Client` as a child module of `tests/mcp.rs`; no new client framework is copied.
Root owns the two-line module bridge and subsequent old discovery-fixture migrations.

The new adapter source ran against baseline production before backend import:

- CLI handle86193 exited101 on real `bond.create.order="double"`; the executable's error
  JSON reported `invalid_params` / unknown field `order`. Log `target/multiple-cli-red.log`,
  3,004bytes, SHA-256 `148524db43a8ed7cd2a883f87d146b12f63a2b8eae3db233d57dc7fa29362cad`.
- Stdio handle74807 exited101 after actual initialization and Inspect; `tools/call` for
  `ui.bond_tool` returned JSON-RPC error-32601, `unknown Bondline tool`. Log
  `target/multiple-stdio-red.log`,1,083bytes, SHA-256
  `2ecb1e9a64501f5d532d3f79a45311d4a0998ab3ad32ce759d4e697841a653e7`.

Root then supplied six actual command/MCP backend files, recorded separately in
`target/multiple-command-backend-import.json`; no renderer was imported. CLI handle77451
exited0 with both new adapter tests passing; stdio handle73394 exited0 with the new child
test passing. All three executed unchanged from their RED sources. Old full discovery
tests were deliberately not run against this intermediate import because their25-name
fixtures had not yet received Root's migration. The six command/image acceptance tests
and pixel oracle remain byte-exact and still await the real renderer before GREEN.

CLI all-target warnings-denied clippy, handle36822, passed; the two owned adapter files'
rustfmt check also passed. Workspace rustfmt, handle9920, exited1 only for received Root
formatting in the MCP module bridge and command bond/bond_tool/controls files. Its exact
diff remains in `target/multiple-adapter-fmt.log` (SHA-256
`9d6ffcac263d587647ff10a1f8ba158617b28e83a28e59e1c88dcbbb90df1186`).
No owned assertion or source was changed to obtain these adapter passes. Formatting of
those received files and a full integration gate remain Root's subsequent work.

## Final actual backend and GREEN

Root supplied the real renderer, command, MCP and UI implementation as thirty exact
inputs in `target/multiple-final-backend-import.json` (SHA-256
`3d548b87bef2be560a8912a3e22d5d0294be562e8265b32ff6a5b33bc51e12d2`).
All four independently frozen acceptance/oracle files remain byte-exact. Against those
actual inputs, the following completed in this checkout's separate absolute target:

```sh
cargo test --locked -p bondline-commands --test multiple_bonds -- --nocapture
cargo test --locked -p bondline-cli --test multiple_bond_adapters -- --nocapture
cargo test --locked -p bondline-cli --test mcp multiple_bonds:: -- --nocapture
cargo clippy --locked -p bondline-commands -p bondline-cli --all-targets -- -D warnings
cargo fmt --all -- --check
```

Handles96130/12466/58687 exited0 with6/2/1 tests passing, respectively; no ignored tests.
The command tests exercised all four original V2000/V3000 Double/Triple CPU/GPU cases,
external chemistry/raw-source checks, authored files, history and refusal paths. Handles
31250/36939 exited0 for all-target clippy/workspace rustfmt. Logs are
`target/multiple-final-{command,cli,stdio}-tests.log`, `multiple-final-clippy.log` and
`multiple-final-fmt.log`. The earlier intermediate formatting failure remains historical;
Root's final supplied source resolves it without changing any owned assertion.

The281-file original baseline now has261 byte-exact files and20 explicitly received
Root replacements; all20 are covered by the final import receipt. New runtime/test inputs
are classified separately from the old baseline. This owner neither ran a full workspace
integration gate nor changed the two old unsupported-MOL/SDF test expectations awaiting
Root's stronger replacement coverage. No native-window or hosted result is supplied.

## Original fuzz seeds and actual scene reachability

Twelve independently authored raw MOL files, six per molecular version, live only under
ignored `target/multiple-fuzz-seeds/original-v1/`. Four positive geometries per version
cover48pt horizontal Double/Triple, a mixed Single/Double terminal and an80pt3:4 diagonal
Triple with fractional phase. Two separate valid-chemistry negatives cover coincident
points and unresolved phosphorus. The unchanged raw MOL/RDKit consumer accepts every
source with the literal molecular/coordinate/H expectations; a fresh regeneration matches
all26 original source/consumer/manifest/license files byte-for-byte. No product writer
constructs these sources. Their original code/data license is MIT OR Apache-2.0.

A small ignored inspector, compiled against the actual frozen libraries, uses the exact
existing fuzz frame:1pt/raw unit, origin0, y flip, A4 paper,1024square. It observed8 complete
multiple-bond GraphScenes and4 expected `unsupported_content` refusals, with exact points,
authored H=None, sidecar/source ownership and pre-cancellation checks. This inspector and
the parser fuzz targets construct scenes/glyph geometry; they do not rasterize mutated
inputs through CPU/GPU or prove native pixels. The separate command tests above supply
their declared CPU/GPU evidence. Scene receipt v2 is
`target/multiple-fuzz-seeds/scene-handoff-v2.json`; v1 and the exact transient ignored README
snapshot remain retained. A note appended to the frozen ignored README was moved into
`SCENE-OBSERVATIONS.md`, restoring the original README bytes; no seed/manifest changed.

## Bounded ASan and separate fixed-input replay

Only the existing `mol_parse` and `mol_v3000_parse` targets ran, each from a separate
writable copy of its six original sources. The pinned toolchain is nightly2026-04-01
(`rustc1.96.0-nightly 48cc71ee8`), cargo-fuzz0.13.2 and libfuzzer-sys0.4.13. Supported flags
were checked against the pinned tool's help/flag definitions before execution. Each run
used AddressSanitizer, seed4411, `-runs=10000 -max_len=65536 -max_total_time=600 -timeout=10
-rss_limit_mb=2048`, an additional600second outer bound, its own artifact directory and
the absolute isolated target. Fuzz sources/configuration/dependencies/locks stayed exact.

- `mol_parse`, handle63611: exit0,10,000 executions, final coverage3603/features6961,
  corpus184 files, current mutation limit/maximum retained404bytes, reported RSS77Mb.
  Build plus execution took46.650seconds; libFuzzer reported1second fuzzing.
- `mol_v3000_parse`, handle85780: exit0,10,000 executions, final coverage3188/features6517,
  corpus156 files, limit/maximum retained388bytes, RSS78Mb. Build plus execution took
  5.467seconds; libFuzzer reported1second fuzzing.

Both finished by run count, with no timeout or reported failure artifact. The64KiB setting
is a cap, not evidence that these short runs reached64KiB inputs. No universal safety,
chemistry, native-platform or CPU/GPU agreement claim follows from these runs.

Separate handle1556 replayed each already-built ASan binary on its six immutable original
files using seed4411 and `-runs=1`; both exited0, reporting six inputs once each and
explicitly "fuzzing was not performed." These12 executions are distinct from the20,000
fuzz executions. Each source and every old retained tool input remains byte-exact; both
owned and default artifact directories contain no failure files. Commands, binary hashes,
logs, corpus hashes and exact settings are in `target/multiple-fuzz-seeds/asan-*/` and
`replay-*/`. No job remains live.

A small pure-stdlib generator proposal under ignored
`target/multiple-fuzz-seeds/readme-recipe-proposal.py` reproduces all twelve sources and
their twelve writable copies exactly without host paths, product writers or a new framework.
Root must review it before any committed reproducibility recipe is added.

## Exact seed recipe receiving

Root adopts the original recipe in [fuzz/README.md](../fuzz/README.md). Its code is exactly
2486bytes/SHA-256`ad3eaeec476395b157cf486696d887e44fc17b9b67b41bf541067b7caf935a9f`.
Running that actual Markdown code block in a fresh ignored destination recreates all24
source/writable files; each matches the frozen original12-seed manifest's length/SHA.
Receipt: `target/multiple-fuzz-recipe-receiving.json`. This verifies reproducible initial
inputs, not a new fuzz run or identical mutation corpus. No extra script, fixture, target,
dependency, lock, oracle or workflow is introduced.
