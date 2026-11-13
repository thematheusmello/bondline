# Verification log

2026-11-13, bootstrap documentation: no application checks yet. Starting workspace was empty.
Rust 1.94.1 is installed. Public specifications and reference license practices were inspected.
Future behavioral entries must identify the RED command/failure, GREEN command/result,
independent reference, relevant environment and remaining limitations. No invented passes.

## Graphics license boundary (in progress)

- RED `cargo test -p xtask`: missing allowlist implementation. GREEN: SPDX OR/AND, legacy
  spelling and target-tree parsing tests pass. RED `cargo xtask licenses`: real forbidden
  native dependencies. Selected-target Cargo trees replaced metadata's feature overapproximation.
- RED `cargo test -p hexf-parse -p libloading`: missing parser/loader entry points. GREEN:
  IEEE-754 bit vectors, inexact/overflow/underflow rejection, system PID vs std::process::id,
  missing library/symbol and invalid type/name tests pass. Raw-symbol ownership test also
  observed RED before implementation.
- RED `cargo test -p clipboard-win`: missing UTF-16/CF_HTML validators. GREEN: known Unicode,
  malformed UTF-16, termination, exact fragment and out-of-range tests pass on macOS.
  This verifies portable parsing, not native Windows clipboard execution.
- Native declared license closure passes all three desktop targets after rejecting gpu-descriptor
  0.3.1 (still foldhash), then pinning 0.3.0. The hypothetical browser graph is not approved.
- Native macOS GPU readback matched independently specified RGBA bytes: Apple M3 Max,
  Metal, IntegratedGpu. This tests clear/readback, not chemical rendering or complete parity.
- UI startup configuration and font/page paint tests observed RED for missing functions,
  then passed. Native app launch and visual inspection are still pending at this entry.

Follow-up evidence:
- Native `cargo run -p bondline-ui --bin bondline -- --smoke-test` exited 0 after the real
  window rendered three frames; adapter JSON: Apple M3 Max, Metal, IntegratedGpu.
- Actual macOS development bundle was opened through native computer use. Observed title,
  readable bundled font, white portrait page and Page 1/A4 status. This was AI visual
  inspection of a read-only prototype; no human-approved golden or drawing workflow claim.
- GUI-free render crate: CPU expected pixels, invalid dimensions and GPU readback agree
  exactly for the tested empty canvas. No chemical primitives are implemented yet.
- Shader parser 64 KiB bound observed RED (oversized zero literal accepted), then GREEN.
  Expanded independent Python float.hex/struct vectors: 128 binary32 and 128 binary64 pass.
- Asset gate observed RED for a missing verifier, then GREEN for known SHA-256, changed bytes,
  disallowed licenses and confined paths. Real font hash and notice presence pass.
## Graphics checkpoint follow-up, 2026-11-13

Workspace tests passed (18 tests, including the 256-case IEEE vector test and actual Metal
readback). Native runtime/asset license gate passed: macOS 109, Windows 120, Linux 187
runtime packages. Whole-workspace Linux source check passed after extracting the render
crate. The DX12-enabled Windows check exposed the safe `open_already_loaded` API mismatch
(E0133); the adapter now owns an existing module reference directly, and adds a Windows-only
PID/absent-module test. This test is source-checked here, not executed on Windows.

GREEN: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D
warnings`, `cargo test --workspace --locked`, Windows all-targets check and clippy, Linux
workspace check. Latest macOS real-window smoke also exited 0 with Metal. No confirmed
background Cargo jobs remain after polling these checks.

## Layer gate, 2026-11-13

RED: `cargo test -p xtask layers --locked` failed for the absent graph validator. GREEN:
three tests reject equal/upward edges, misclassified/new packages, format-to-workspace
dependencies and a transitive GUI dependency; valid lower-layer edges pass. The actual
all-features Cargo resolution passes `cargo xtask layers`. Focused xtask clippy passes
after collapsing the nested condition required by the pinned compiler.

## Page model, 2026-11-13

RED: `cargo test -p bondline-model` failed for missing PageId/PageSize/Document APIs. GREEN:
three tests pass against independent physical-unit expectations (72 points per inch, A4
210 by 297 mm), invalid dimensions including non-finite values, checked deserialization,
nonzero IDs and failed-edit atomicity. Focused model clippy and actual layer gate pass.

## External artifact inspection, 2026-11-13

REA 6.9.0 ran as an external development tool under an already installed Node 26.12.1.
`rea audit-release-exposure target/dev/Bondline.app --json` completed with exit 0. This
selected the earlier read-only development bundle, not any proprietary program. Evidence
ID `ev_a770a1af9d192bd3602b3081084097908158b625f86fbdcf35ff496f2a904c99`, directory
digest `b691b23c9b8b3f131dc99480ffe78828adc22e9ce00ba78dad1b4560ff8d86d9`. No source-map,
credential-marker or debug-companion condition was observed within the reported bounds.
Go/Rust metadata remained unknown; hardening, hardened runtime and Android were not assessed.
Raw report: ignored build output `target/rea-graphics-audit.json`. This is neither a license
audit nor a security pass. No REA code is bundled with the app.

## Page command and snapshot history, 2026-11-13

RED: `cargo test -p bondline-commands` failed for the missing registry/engine. GREEN: typed
closed schemas, published physical page dimensions, malformed/oversized requests, failed-edit
atomicity, read-only inspection, redo preservation and branch replacement pass. Proptest runs
128 fixed-seed sequences of up to 63 edits; each edit has independently asserted dimensions,
exact undo/redo restoration, and full-sequence restoration. Boundary tests additionally check
the declared 128-snapshot cap and atomic rejection at revision overflow. No desktop or transport
verification is implied by these headless tests.

## CLI adapter, 2026-11-13

RED: `cargo test -p bondline-cli` failed because the executable emitted no discovery/state
payload and returned success for invalid commands. GREEN: both executable-level tests pass:
registered schema discovery, physical Letter page dimensions after edit/undo/redo, consistent
errors on invalid commands and usage, and help. A manual CLI page edit also returned the
expected 612 by 792 point page. No file persistence is advertised.

The license product-discovery test first failed for the missing selector, then passed along
with all nine xtask tests. The actual declared runtime audit now covers UI and CLI: macOS
116 packages, Windows 127, Linux 193, plus two verified bundled assets. Layer and focused
CLI/tooling clippy checks pass.

## Native page controls, 2026-11-13

RED: `cargo test -p bondline-ui` failed because the workspace did not accept engine state.
GREEN: three UI tests pass, including keyboard events through the shared history engine and
page geometry against physical aspect ratios. The actual newly built macOS development app
was opened: Page -> Letter portrait displayed 612.0 x 792.0 pt, Cmd+Z restored 595.3 x 841.9,
and Shift+Cmd+Z restored Letter. Accessibility and a real-window screenshot confirmed the
result. This is AI workflow inspection, not a human-approved golden. No chemical editing or
Windows/Linux runtime claim. The prior REA report remains historical and bound to its old digest.

Integrated page-control checkpoint: 34 workspace tests pass, including actual Metal readback,
CLI subprocess checks and generated history cases. Workspace rustfmt/clippy pass. Windows
and Linux all-targets source cross-checks pass; both product roots retain allowed declared
runtime licenses (116/127/193 packages for macOS/Windows/Linux respectively). Native
Windows/Linux execution, CI runs and human golden approval are still missing.

## Stdio MCP, 2026-11-13

RED: `cargo test -p bondline-cli --test mcp --locked` failed because the executable did not
provide MCP. GREEN: an independent line-oriented JSON-RPC client, using public 2025-11-25
messages, verifies negotiated protocol, tool discovery/schemas, Letter edit, undo, structured
invalid-parameter errors, unknown-tool protocol error, recovery after an oversized line, and
clean EOF shutdown. The executable is the real CLI, not a mock server. CLI regression tests
also pass. The actual layer gate passes; native runtime declaration checks pass with the SDK:
macOS 143, Windows 154, Linux 213 packages. Two bundled asset hashes/notices remain verified.

MCP checkpoint: 35 workspace tests, workspace/all-targets clippy, rustfmt, native declared
license/asset gates and layer gate pass. Windows/Linux all-targets source checks also pass
after boxing the SDK initialization error to satisfy clippy. No native Windows/Linux,
three-platform CI or browser execution is claimed.

## Native loopback control, 2026-11-13

RED: `cargo test -p bondline-mcp --test control --locked` failed for the absent server.
The launch-argument test then failed for the missing parser. GREEN: four independent std TCP
client tests cover shared state/history, repaint notifications, domain errors, the 1 MiB
bound, truncated/HTTP rejection, the 16-client cap, idle-client cancellation and port release.
The native launch parser rejects missing/invalid/duplicate flags. Engine guards do not cross
async waits; the compiler caught that boundary before the owned-result refactor.

Actual Mac integration: launched the real Metal app with `--control-port 0`. Startup/UI
reported 127.0.0.1:50967. An independent Python socket client set 792 x 612 pt; accessibility
and the canvas showed landscape paper. Native Cmd+Z restored A4; another socket query
confirmed revision 2 and redo depth 1. App quit exited 0 and the process/listener were
independently confirmed closed. Computer-use AX observation can relaunch a closed app;
the incidental default relaunch was also closed. No development listener is left open.

Integrated gates pass: 40 workspace tests, rustfmt, workspace/all-targets clippy, layers,
native declared licenses (143/154/213 packages), two asset hashes/notices and Windows/Linux
all-targets source checks. This is not native Windows/Linux execution or a human golden.

## Shared CPU/GPU page scene, 2026-11-13

RED: `cargo test -p bondline-render --test page_scene` failed for absent scene APIs.
GREEN: both CPU and real GPU agree with a hand-authored 8x8 reference grid, explicit layered
RGBA bytes (including alpha source replacement), clipping, and checked hostile geometry.
Additional cases cover one-pixel/odd-pitch readbacks, maximum capture width and the declared
4096-primitive bound. Native GPU is Metal on the previously identified Apple M3 Max.

Before changing the UI, the new shared-border/pixel-grid geometry assertion failed with
"shared scene border was not painted". The native adapter now consumes the render scene.
The integer rounding invariant and exact one-unit interior inset are documented in
rendering.md; this replaces the prior continuous-layout assertion rather than claiming
continuous precision for rounded dimensions. Keyboard undo/redo and physical dimensions
still have their independent checks. All relevant tests pass.

The actual new development bundle was opened and Page -> Rotate page displayed 841.9 x
595.3 pt with landscape paper, border and shadow visible. It was closed and no development
process remains. This is AI native inspection, not human golden approval. Integrated gates
pass: 44 tests, rustfmt, workspace/all-targets clippy, layers, native declared licenses/assets,
and Windows/Linux all-targets source checks. No native Windows/Linux or browser pass.

## PNG interchange backend, 2026-11-13

RED: `cargo test -p bondline-io` failed for missing encode_png. GREEN: an independent external
Pillow 12.3.0 process verifies PNG integrity, RGBA mode, 3x2 dimensions and all 24 expected
bytes, including alpha 128 and alpha 0 with nonzero RGB. Malformed byte counts, zero/oversized
and overflowing dimensions fail before encoding. PNG output is finalized with IEND; no
filesystem operation or application export workflow is claimed. The native app does not
need Python; the test interpreter/oracle setup is documented in development.md.

Integrated checkpoint: 46 tests, rustfmt, workspace/all-targets clippy, actual layers and
native license/asset gates pass. Windows/Linux all-targets source checks pass. The PNG crate
is already part of the audited graphics dependency graph; Pillow runs externally only.
No new native runtime, full M0, chemical compatibility or human-golden claim.

## Cooperative render cancellation, 2026-11-13

RED: `cargo test -p bondline-render --test cancellation --locked` failed for the missing
cancellation API and error. GREEN: a shared pre-cancelled flag rejects both paths before
allocation/GPU initialization; all seven render tests pass, including real Metal pixel
comparisons. GPU readback now polls without the previous unbounded device wait, observes
cancellation and retains a ten-second readback deadline. Driver calls themselves remain
non-preemptible. Adapter shutdown and in-flight job cancellation need their own integration
evidence when the capture command lands.

## Shared offscreen capture command, 2026-11-13

RED: `cargo test -p bondline-commands --test capture --locked` failed for the missing
preparation/capture API. GREEN: external Python base64/Pillow checks every PNG byte against
the independent 8x8 page grid after another edit has changed the live document. The query
keeps its original revision and history. Explicit GPU output matches CPU PNG output and
reports its real adapter. Invalid dimensions, page IDs, backends and unknown fields fail
atomically; pre-cancellation rejects GPU work before initialization.

RED: `cargo test -p bondline-mcp --lib --locked` failed for the missing worker implementation.
GREEN: two-worker budgeting does not block short authoring commands. Explicit cancellation
and aborting the awaiting future signal an instrumented running worker; its permit remains
held until actual completion. Real CLI, MCP and loopback wire tests discover and execute
capture through the same registry; read-only captures do not notify native repaint or alter
history. Engine guards do not cross rendering/encoding or asynchronous waits.

Actual Mac app (Apple M3 Max / Metal): independent socket captures at 512x512 decoded with
Pillow and matched CPU/GPU pixels. A concurrent 2048x2048 GPU capture returned input revision
1 while a page edit reached revision 2. That edit's measured round trip was 0.301 ms in this
single observation, not a throughput benchmark. Native Cmd+Z visibly restored 72x144 pt;
the next capture at revision 3 exactly matched the original PNG. Exec session 56032 exited
0 and 127.0.0.1:53031 was independently confirmed closed. Candidate PNGs and observations are
ignored outputs under target/captures; human approval was requested and remains pending.

Integrated gates: 54 workspace tests, rustfmt, all-targets clippy, layers, declared native
licenses (145/156/215 packages for macOS/Windows/Linux), two asset hashes/notices and
Windows/Linux all-targets source checks pass. Native Windows/Linux, human goldens, full M0
and chemical rendering/interchange remain unverified or missing.

## Initial CDXML syntax and parser fuzzing, 2026-11-13

RED: `cargo test -p bondline-cdxml` failed for the absent parser/API. An external-doctype
fixture then failed with DtdDetected under the tree parser's default configuration. The
streaming preflight now rejects internal subsets before enabling external-doctype syntax
with no resolver. No external bytes are fetched or expanded. GREEN: original fixture
round trip is byte-exact and external Python Expat independently checks element order,
attributes, numeric spelling, namespaced extensions, Unicode/character references, comments,
CDATA and processing instructions. UTF-8 BOM and CRLF remain byte-exact. Invalid syntax,
characters, duplicate attributes, roots, encodings/versions, nesting, input and node caps fail.

RED: the attribute-budget test failed for the missing bound/error. GREEN: 256 attributes
are accepted; excess per-element and total attributes fail before building the tree.
A bracket inside a quoted external identifier is preserved rather than mistaken for a subset.
All 57 workspace tests, rustfmt/clippy, layers, native declared-license/assets and Windows/
Linux source checks pass. No application open/save or chemical interpretation is claimed.

Fuzz setup: installed external cargo-fuzz 0.13.2 under ignored target/tools and added
nightly-2026-04-01 (rustc 1.96.0-nightly 48cc71ee8) without changing the project's 1.94.1 pin.
The isolated fuzz/Cargo.lock pins libfuzzer-sys 0.4.13; its C++/NCSA test runtime is never
shipped. `cargo +nightly-2026-04-01 fuzz run cdxml_parse .cache/fuzz/cdxml -- -runs=10000
-max_len=65536 -timeout=5` passed with AddressSanitizer, seed 3641879538, final edge coverage
1696 and 326 retained inputs. No failure was reported. This bounded smoke run is not
exhaustive conformance, parser coverage or a security audit. No live fuzz/install job remains.

## Licensed corpus gate, 2026-11-13

RED: `cargo test -p xtask --offline` failed for the absent manifest/validation API. GREEN:
manifest checks reject unpinned commits, traversal, missing provenance, bad hashes and an
outside-policy license without widening the existing policy. Exact size/SHA checks use the
independent NIST abc vector. A separate verifier test intentionally changes a bond attribute
and confirms that the Expat comparison detects the disagreement.

Actual `cargo xtask corpus` fetched the pinned RDKit chain1.cdxml (727 bytes) and CH3.mol
(170 bytes); hashes matched their reviewed manifest. CDXML byte round trip and all element/
attribute/text values agreed with external Expat. `cargo xtask corpus --offline` also passed.
A same-length mutation in the actual cache was rejected for SHA-256 mismatch; original
bytes were restored. The MOL decoder/chemistry checks are explicitly printed MISSING.
No CDX corpus entry or complete M1/all-format round-trip claim. The BSD notice is retained.
