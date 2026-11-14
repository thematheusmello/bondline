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

## Real CI orchestration, 2026-11-13

Before implementation `cargo xtask ci` failed with the usage error. It now executes fmt,
clippy, locked workspace tests, layers, licenses/assets and the actual corpus gate. The
full command passes on this Mac with 60 tests. An intentionally unformatted helper in the
owned parser source made it fail at fmt before invoking clippy; source bytes were restored
and the complete gate rerun. No failing source was committed.

The committed native matrix and sanitizer workflow configuration uses researched runner labels,
pinned action commits and no write credentials. Its YAML parsed with existing PyYAML 6.0.3;
actionlint/Go were absent. Hosted jobs, their GPU availability and Windows/Linux native
launches have NOT run. See ci.md for limits and the distinction from source cross-checks.

## Public control inventory, 2026-11-13

Documentation-only change: no artificial unit tests. Downloaded the public vendor-authored
guide from its recorded university URL into ignored .cache/research; exact byte length,
SHA-256, metadata and 439-page count are recorded in sources.md. Printed page numbers are
one less than one-based PDF page numbers, equal to zero-based PDF indices. Poppler rendered the shortcut table pages; their column
layout, continued rows and literal keys were visually checked. Explicit menu-path references
were collected across all pages (288 lines on 165 pages) and manually screened for host
application/OS context. Official 26.0/26.1 desktop deltas were reviewed separately from cloud.

parity-checklist.md now has 573 stable-ID detailed rows: 569 missing, two wired CTRL history
bindings, and two page-only Mac history menu workflows verified by prior evidence. IDs,
Markdown row widths, state values and source-page bounds were checked. Labels inferred
only as functional actions are distinguished from explicit paths. Source conflicts and
remaining modern/palette/context-menu gaps stay open; row count does not certify M0 parity.
No guide, help prose, screenshot, sample library, installed XML configuration or binary is
redistributed or inspected beyond the public PDF. `cargo xtask ci` passed all 60 existing
tests and the formatting, lint, layer, license/asset and current corpus gates.

## XML oracle locale portability, 2026-11-13

RED: `PYTHONIOENCODING=ascii cargo test -p xtask
independent_xml_comparison_detects_a_semantic_disagreement --locked` failed with an external
UnicodeEncodeError for the original Greek-text fixture. The helper had disabled JSON's
ASCII escapes. GREEN: the helper now emits ordinary escaped JSON and explicitly runs with
ASCII stdout, so the existing Unicode/mismatch test continuously exercises this boundary.
Decoded JSON semantics are unchanged; input XML stays raw UTF-8. This fixes developer-tool
portability and does not establish native Windows application execution.
`PYTHONIOENCODING=ascii cargo xtask ci` passes all 60 tests and the real gates.

## Source-preserving CDXML attribute replacement, 2026-11-13

RED: `cargo test -p bondline-cdxml --test attributes --locked` failed with E0599 for the
absent replace_attribute method and edit errors. GREEN: five new tests pass. The original
atom coordinate edit has an independently constructed exact expected byte string; unrelated
source bytes stay unchanged. External Expat verifies both quote styles, namespaced values,
all escaped delimiters, numeric tab/LF/CR references and XML character-range boundaries.
No-op entity spelling, empty values, BOM/CRLF, a 70000-character name and over 255 bytes of
equals-whitespace are covered. Oracle expectations go through length-prefixed stdin rather
than potentially oversized platform command arguments; its stdout is explicitly ASCII.
Foreign/non-element targets, missing attributes, namespace-declaration edits, forbidden
characters and an escaped-size overflow are rejected with the original source intact.

The isolated ASan harness now exercises accepted parse/edit/reparse inputs. A 10000-run
smoke passed with seed 3641879538, max_len 65536, timeout 5, final edge coverage 1957 and
384 retained corpus entries (146 KiB). Log: ignored target/cdxml-attributes-fuzz.log.
This is bounded fuzz evidence, not exhaustive conformance or a security audit.
`cargo xtask ci` passes all 65 workspace tests and existing real gates. Windows/Linux
whole-workspace all-targets source checks pass; their native execution remains unverified.
No UI change, application file-open/save, model mapping or broader format compatibility
is claimed. No fuzz or Cargo job remains running.

## Loopback session authorization, 2026-11-13

RED: `cargo test -p bondline-mcp --test control unauthenticated_clients --locked` returned
a document snapshot where the independent wire test expected unauthorized. GREEN: an
unauthenticated connection now receives the exact generic error and closes before dispatch.
No authored state/history or change callback is touched. Missing/wrong tokens, wrong types,
malformed JSON, duplicate fields and unknown fields are checked. Existing socket workflows
now perform the handshake and continue sharing edits, captures, history and bounded lifetime.
Two live servers reject each other's token; token shape/freshness are checked, not claimed
as a statistical random-source certification. Credentials are absent from rejection responses.

A further RED test exposed borrowed-str decoding rejecting an equivalent JSON string with
Unicode escapes. The bounded handshake now owns the decoded String; the escaped-token
wire test passes. The unchanged engine, CLI and stdio MCP still receive their original
command shape. The handshake is adapter metadata, not an alternative command registry.

Actual native workflow (Apple M3 Max / Metal): an independent Python socket was rejected
without credentials, then authenticated using startup stdout kept only in process memory.
The edit set 144x72 pt at revision 1. Native accessibility and a real-window screenshot
showed that page; Cmd+Z visibly restored A4. The authenticated socket read revision 2 and
595.2755905511812x841.8897637795276 pt. Native Quit closed the app; exec 81217 exited 0 and
127.0.0.1:56743 refused a new connection. The first ad hoc driver had hit its own stdin EOF/
wait timeout; the completed driver used the controlling terminal. No token was recorded.
This native run preceded the equivalent-JSON-string decoder refinement. The final built
native app then passed its real three-frame smoke launch on Metal, accepted an entirely
Unicode-escaped token string and served document.inspect before exiting 0. Startup
credentials stayed in the parent process's pipe/memory and were omitted from tool output.

Pinned additions retain the allowlist: getrandom 0.3.4 MIT/Apache and subtle 2.6.1 BSD-3-Clause.
The native declaration gate now covers 147/158/216 packages plus two assets. Full source/
release-notice audit, compromised-account isolation and hardware side-channel certification
are not established. No proprietary program or asset was inspected.
`cargo xtask ci` passes all 68 tests, formatting, clippy, layers, native licenses/assets
and current corpus checks. Final Windows/Linux whole-workspace all-targets source checks
pass; neither is native runtime evidence. No Cargo/native job remains running.

## Source-backed CDXML node geometry, 2026-11-13

RED: `cargo test -p bondline-model --test point --locked` failed for the absent Point2;
`cargo test -p bondline-io --test cdxml_geometry --locked` failed for the absent adapter/
model dependency. GREEN: two model and four io tests pass. The independent published
CDX-COORDS "72 144" vector fixes units and axis order. Negative/zero coordinates, finite
±100000 pt boundaries and strict JSON construction are checked. Missing p remains None.
The original fixture's coordinate patch has an independently constructed full expected byte
string, and external Expat confirms its new value, other node position, isotope, bond order
and namespaced opaque property. Numeric no-ops retain original spelling/character references.
Malformed/non-finite/out-of-range pairs, missing/duplicate source IDs, unsupported n layouts,
unknown targets and edits to unpositioned nodes fail explicitly without modifying the input.

This is a source-backed geometry utility, not a chemical graph/app importer or complete atom
move. Source identifiers stay decoded strings; numeric/global identity and dependent labels/
boxes are not interpreted. Unknown properties/objects remain in the original syntax bytes.
The archived ID page returned 404; the remaining mapping questions are recorded explicitly.

The actual io adapter is exercised by the isolated ASan harness, including coordinate edit/
reparse/source-preservation checks. Its larger tool-only build graph retains original parser/
libFuzzer package pins and the reviewed loader/hex patches; gpu-descriptor is pinned to 0.3.0
there as in the product. The final 10000-run smoke passed with seed 3641879538, max_len
65536, timeout 5, edge coverage 2147 and 387 retained inputs (130 KiB). No GPU is initialized
by the fuzz path; this is not renderer fuzzing, exhaustive conformance or a security audit.
Log: ignored target/cdxml-geometry-fuzz-pinned.log.

`cargo xtask ci` passes all 74 tests and the existing real gates. Runtime declaration
closure is 150/161/219 packages for macOS/Windows/Linux plus two unchanged checked assets;
the new product reachability is the existing standalone syntax crate and its two XML parsers.
Whole-workspace all-targets Windows/Linux source checks pass. No UI change or new platform
execution claim; existing native evidence is unchanged. No running Cargo/fuzz/native job.

## Exact CDXML integral properties and source IDs, 2026-11-13

RED: `cargo test -p bondline-cdxml --test numbers --locked` failed for the absent ObjectId
and parse_integer exports. GREEN: three tests pass. Published 255/255.0/255.00 and bond
ID/endpoint examples are checked. External Python Decimal.as_integer_ratio independently
confirms accepted values, including i64 boundaries. UINT32 maxima, unassigned zero, long
leading-zero strings, whitespace/sign policy and invalid syntax are covered. Tiny nonzero
fractions, nonfinite values and overflow are rejected without floating-point conversion.
Exponent notation and empty fractions are explicit unsupported grammar, not silently rounded.

The public general record/data-type sources resolve 32-bit/container-scoped identity and
zero's meaning despite the broken id.htm link and old table labels. Source IDs remain
separate from future internal identities; no reference resolver is claimed yet.

The ASan harness now exercises integral properties and ID range invariants on raw UTF-8
and XML attributes. A 10000-run smoke passed with seed 3641879538, max_len 65536, timeout 5,
edge coverage 2324 and 434 retained entries (136 KiB). Log: ignored target/cdxml-numbers-fuzz.log.
`cargo xtask ci` passes all 77 tests and the real gates; runtime closures/assets are unchanged.
Windows/Linux all-targets source checks pass, not native execution. No UI change or full
chemistry/identity compatibility claim. No live Cargo/fuzz job remains.

## Authored elemental graph model, 2026-11-13

RED: `cargo test -p bondline-model --test graph --locked` failed for the absent graph types,
fragment storage and batch insertion API. GREEN: seven graph tests pass against public
Daylight connectivity/hydrogen/ring/disconnection examples and archived atom/bond property
definitions. The 1..118 element catalog, nonzero typed IDs, optional positions/isotopes/
hydrogens, charge and radical serialization are checked. Dative begin/end order stays 2->1
instead of being normalized by numeric ID. These are authored values, not perceived chemistry.

Dangling/self/parallel bonds, per-fragment and document-wide duplicate IDs, missing target
pages, batch atomicity and 100000-item budgets are checked. The global atom-limit case
keeps the accepted fragment after rejecting an extra atom. Page-size edits retain the
same immutable fragment Arc. No graph mutation can bypass Fragment's validated constructor.

The process CLI test first failed because its old exact page expectation lacked fragments.
Its complete expected object now includes the empty array; the assertion was not weakened.
CLI/MCP/loopback still expose only the existing five commands; no chemical editing/import,
chemical rendering, chemistry calculation or source-backed graph save is claimed.
OpenSMILES hosting timed out; the public Daylight specification supplied the independent
examples. No proprietary assets, sample libraries or program binaries were used.

`cargo xtask ci` passes all 84 tests and all current gates. Windows/Linux all-targets
source checks pass; their native execution remains missing. Dependency/asset counts are
unchanged, no new UI behavior is claimed and no Cargo/native job remains running.

## Read-only source-backed CDXML elemental mapping, 2026-11-13

RED: `cargo test -p bondline-io --test cdxml_graph --locked` failed for the absent mapper/
error exports. GREEN: six tests pass for the original unknown-data fixture, documented atom
defaults/properties, local ID resolution, zero/unassigned and repeated source IDs, nested
groups, all sixteen single-term bond orders and begin/end direction. Forward references,
UINT32 maxima, decimal identity/order spelling, unsupported types/orders/layouts, duplicate
source IDs and invalid/range/fractional properties are covered. Model identities are fresh,
not copied from source. Unknown data remains in exact original bytes.

Review added an actual failing case: `cargo test -p bondline-io --test cdxml_labels --locked`
accepted a text-only N label as default carbon. The public Node guide permits text-only
writers, so this is now explicitly rejected without guessing. The regression passes with
UnsupportedLabelOnlyAtom; complete label parsing remains missing. Explicit element metadata
is projected without label reinterpretation. Two nested-if lint findings were corrected;
no warning allowance or failing source was committed.

RED: `cargo test -p xtask independent_elemental_projection_detects_mapping_disagreements`
failed for the missing comparator. GREEN: developer-only Expat/Decimal independently derives
the model projection from source events. It detects an intentionally corrupted element;
a second test checks scoped IDs, direction, metadata and defaults. The real corpus gate
now checks its pinned RDKit graph as well as source bytes/XML. Shared process setup retains
ASCII stdout and no external DTD resolution. Formula/mass/aromaticity/stereo/SMILES/InChI
calculations are explicitly printed MISSING, as are MOL decoding/chemistry. No broad M1 pass.

The isolated ASan harness checks unchanged source, global application-ID uniqueness and
fragment-local distinct endpoints. Final reviewed smoke: 10000 runs, seed 3641879538,
max_len 65536, timeout 5, coverage 2879 and 515 retained entries (151 KiB). Log: ignored
target/elemental-cdxml-reviewed-fuzz.log. This is bounded evidence, not exhaustive coverage
or a security audit. Only existing package edges changed; no new dependency versions.
`cargo xtask ci` passes all 93 tests and gates, including the new corpus projection check.
Windows/Linux all-targets source checks pass; native execution remains unverified. No UI/
application import-save/chemical calculation claim. Runtime licenses/assets remain 150/161/219
packages plus two assets. No Cargo/fuzz/native process remains running.

## Owned source-backed application history, 2026-11-13

RED: `cargo test -p bondline-io --test stored_document --locked` failed for absent
StoredDocument/DocumentError APIs. `cargo test -p bondline-commands source_tests --locked`
failed for the absent owned-snapshot engine constructor. GREEN: three io tests check original
byte ownership without a source copy, caller lifetime, explicit preview dimensions, source
metadata, Debug privacy, guarded stale-source export, exact restoration and unsupported
multiple/table pages. Empty CDXML remains byte-exact. Page dimensions are an independent
preview policy, not source paper interpretation; the published Page guide distinguishes
physical paper from graphical BoundingBox and requires no properties.

Four engine tests check whole-source undo/redo, immutable prior results, no-op/redo retention,
a source-only change with an identical model, revision-overflow atomicity, and capture refusal
for imported and native chemical data on both requested backends. Neither rejected capture
starts the GPU or changes authored state. Another fixed-seed property test runs 128 sequences
of up to 63 edits, checks per-edit whole-snapshot undo/redo, then restores original bytes and
the final snapshot. This is additional to the existing 128 native-page sequences. A test-only
variable-name compile error was corrected before GREEN; no failing source was committed.

`cargo xtask corpus` now checks exact passthrough through the owned snapshot and compares
its model with the independent Expat/Decimal oracle. One pinned CDXML is covered; MOL decoding,
CDX and chemistry calculations remain explicitly missing. No new dependency version was added.

The isolated ASan harness adds edit/refusal/restoration invariants for accepted owned
snapshots. Reviewed bounded run: 10000 runs, seed 3641879538, max_len 65536, timeout 5,
coverage 3123, 541 retained entries (153 KiB), no reported crash. Ignored log:
target/stored-document-fuzz.log. This is a smoke run, not exhaustive or security certification.

`cargo xtask ci` passes all 101 workspace tests and the formatting, all-targets clippy,
layer, license and corpus gates. Whole-workspace all-targets `cargo check --locked --target`
x86_64-pc-windows-msvc and x86_64-unknown-linux-gnu pass; neither establishes native execution.
Runtime closure remains 150/161/219 packages and two assets. No UI behavior, exposed import/
save command, editable writer, chemical rendering or new human golden is claimed. No native
app was launched for this core-only change; actual Mac UI evidence above remains historical.

## Independent native single-bond workflow and approved golden, 2026-11-13

RED: `cargo test -p bondline-render --test strokes --locked` failed for missing Primitive,
Stroke and ordered scene APIs. GREEN: exact horizontal/vertical/45-degree grids, reversed
endpoints, interleaved alpha source replacement, clipped/fractional/distant cases and mixed
primitive/resource rejection pass. An independent i128 quarter-pixel dot/cross oracle fixes
CPU coverage exactly; before implementation GPU ambiguity was restricted to 1/32 pixel of
true boundaries. Review also reproduced an overly broad oracle boundary flag beyond finite stroke caps;
the distance check was tightened with a failing regression, without changing the tolerance.
No assertion was loosened after comparison. A separate winding test failed
for the native butt outline's orientation and passed after correcting its corner order.

RED: carbon_scene failed for the missing PageViewport/chemical producer. The first line-only
expectation was then corrected after public IUPAC GR-2.1.2 review: isolated ethane requires
explicit terminal labels. It was replaced by terminal-mask/ink and exact central-line checks,
not accepted as a correct final depiction. The first producer is deliberately limited to
independent C-C fragments with three explicitly authored H per C; no general valence inference.
Invalid metadata/bond kinds/positions fail rather than being hidden. A guessed Element method
name failed compilation and was corrected against the actual pinned API before GREEN.

RED: label_tests failed for the absent bounded font module. GREEN: FontTools 4.61.1 independently
supplied Roboto glyph IDs/advances/bounds and em/ascent/descent vectors. The font is the existing
SHA-pinned Apache asset. External Pillow 12.3.0/FreeType independently draws H3C/CH3 and
subscript positions at 96 em; declared ink IoU >=0.90 produced 0.966025/0.966897. FontTools is
not a runtime or regular CI dependency. Glyph masks, scale/placement/work bounds and same-scene
CPU/GPU output are checked. There is no claim of smooth/general chemical text rendering.

RED: `cargo test -p bondline-commands --test bonds --locked` returned UnknownCommand(bond.create).
GREEN: one registered closed schema creates two C/three-H atoms and a single bond in one
history step. Tests check the complete expected model, fresh IDs, branch nonreuse, invalid
request/redo retention, identity-domain exhaustion, source-backed rejection, immutable capture
plans and CPU/GPU PNG agreement. Another 128 fixed-seed generated sequence test restores full
chemical snapshots through Undo/Redo. CLI/MCP wire tests now discover six commands and exercise
creation/errors; authenticated loopback checks shared chemical/history/capture state.

RED: the UI drawing test failed for missing gesture state. Input tests cover preview/no edit,
release/one edit, Escape, stale revisions, focus loss, outside release, jitter and multiple
batched gestures. A real Mac run then revealed a clipped render error: the RED regression
could not find visible diagnostic text after canvas allocation. Stable interaction without
cursor advancement plus a gray error background fixes it. A second RED regression lost a
complete press-move-release batch because egui clears press_origin on release. Consuming raw
events in order fixes both complete and multiple drags between frames. One collapsible-if lint
was fixed without adding allowances. These discoveries show why shape/input tests alone were
insufficient; no failing source was committed.

Actual native final build: target/dev/Bondline.app, Apple M3 Max/Metal. From empty, CUA drag
(500,400)->(620,400) created revision 1, one history step, two atoms and one bond. Authenticated
control independently confirmed the authored graph. Native Cmd+Z produced revision 2 and an
empty fragment list; Shift+Cmd+Z produced 3 and exact restoration. A deliberate duplicate
fragment at revision 4 produced an on-canvas/accessibility-visible overlap message and a
capture error. Native Undo at 5 restored exact model and PNG bytes. Final exec 93577 exited 0
and its 127.0.0.1:50110 listener was independently refused after shutdown. Earlier test builds
also exited. No actual token was printed or committed. Native cancellation variants beyond
this workflow remain input-test evidence, not complete platform execution.

The actual native document's 1024×1024 CPU and GPU PNGs were byte-identical. The user viewed
and explicitly approved the GPU reference: SHA-256
1b650231527f47db5dc7c368644939907d2b038d32f25e77578318f483521bbc (22933 bytes).
crates/render/tests/goldens records those exact bytes, authored coordinates, date and narrow
approval. External Pillow verifies the hash and every CPU/GPU RGBA byte; one-byte corruption
must fail for the pixel-mismatch reason. The older blank preview was not approved. This is
specific-scene approval, not final UI design or broader rendering approval.

Final gate: `cargo xtask ci` passes 126 tests, rustfmt, warnings-denied all-targets clippy,
layers, licenses and the existing corpus scope. Windows/Linux whole-workspace all-targets
source checks pass; native runtime/hosted jobs remain unexecuted. Runtime closure remains
150/161/219 packages and two bundled assets; the approved PNG is a test/documentation fixture.
Only dependency edges changed in the native lockfile; the isolated fuzz lock adds the same
four already-reviewed glyph/font-parser versions. Its pinned-nightly offline check passes;
no new parser behavior or new ASan run is claimed. No save/reopen, existing-atom attachment,
general label/chemistry, full UI parity or complete M0/M1 claim is made.

## Pinned source and public runtime research, 2026-11-13

Documentation-only checkpoint after 7043563; no product code/dependencies changed.
The three ignored sparse Git checkouts have the exact commits in
[reference-projects.md](research/reference-projects.md). Source permalinks were checked
against the corresponding Git objects. No reference application/test/setup script ran.
PhotoCraft's broader font/icon licenses and VoiceStudio's AGPL application remain excluded
from shipped reuse. This review is conceptual/source evidence, not their runtime validation.

Using the installed REA 6.9.0 CLI with existing Node 26.12.1 and an explicitly selected
Chrome executable, controlled scenarios navigated the official download portal and public
Walnut playlist. The portal displayed authentication; no login or agreement was attempted.
The playlist entry and first promotional guide slide were captured and visually inspected;
no chemical editing operation was observed. Exact Evidence IDs, image hashes, action
sequences, failed/incomplete probes and cleanup are in
[official-ui-evidence.md](research/official-ui-evidence.md). All owned browsers terminated.
The captures do not prove a desktop build, complete network traffic or feature correctness.
The six new screen/flow groups organize existing rows; no parity state was promoted by
research alone. Documentation checks: Git whitespace and local Markdown targets; prior
126-test aggregate remains the code checkpoint. No new test was invented for prose changes.

## Shared source/history byte accounting, 2026-11-13

Scope: model/io retained-storage queries and engine commit eviction, using the pinned
PhotoCraft shared-payload budgeting pattern as a conceptual reference. All Rust logic is
independently written. No command/schema, renderer, UI layout, dependency or parser behavior
changed. Native import/export and failed-import/save workflows remain unimplemented.

RED: `cargo test -p bondline-commands oldest_distinct_sources_are_evicted_before_128_snapshots
--locked` failed with Undo depth **3**, expected **1** after three distinct 45 MiB XML
allocations. The prior count-only limit retained all source data. Model/io `--test storage`
checks then failed with E0599 for the missing retained_bytes APIs. Disposable transcripts:
target/history-budget-red.log, target/model-storage-red.log, target/io-storage-red.log.

GREEN: the same source-history test now evicts the oldest prefix, confirms its source
allocation is released through a Weak reference, and checks byte-preserving Undo/Redo.
It also branches from Undo and confirms discarded redo source data is released before the
new branch's budget is judged. Two further engine regressions preserve all 128 page edits
of one shared 2 MiB source and check recent-suffix/current-document retention at independent
small thresholds. Model's two tests use known vector capacity differences and shared Arcs;
io's two tests distinguish shared bytes from separate equal allocations and retain modified
documents' original baselines. These are allocation/sharing expectations, not file-codec
round trips or whole-process memory measurements.

Accounting includes model/page containers, reserved vectors, distinct fragments, source
bytes and export baselines; allocation identities stay local to the query. Sums saturate.
A changed commit clears redo, applies the count cap, then uses monotonic suffix accounting
to find the smallest oldest-prefix eviction at 128 MiB. Current data is never evicted, even
if it alone exceeds that threshold. Arc/allocator headers, caches, temporary work and
externally held snapshots are excluded. No hard RSS cap or new native memory-flow claim.
Undo/Redo retain their existing O(1) payload moves; commit accounting traverses retained data.

The first aggregate run caught a test-helper type_complexity lint; a fixture type alias
resolved it without allowances. Final `cargo xtask ci` passes **133 tests**, rustfmt,
warnings-denied all-targets clippy, layer/license gates and the existing pinned corpus.
All-targets Windows/Linux source checks also pass; neither target's runtime/hosted workflow
was run. No new parser fuzz run was needed or claimed. The earlier native drawing build
is unchanged and closed; the new accounting was verified through the headless engine.

## Native elemental CDXML encoding, 2026-11-13

Scope: new-document syntax construction in the standalone crate, native elemental encoding
in io, explicit Poster extent recovery, and an independent consumer of actual bond.create
output. Six registry commands remain; no file command, GUI save action or imported structural
editing is claimed. Original source backing and stale-export rejection remain mandatory.

RED: `cargo test -p bondline-cdxml --test writer --locked` failed with the missing Writer
API. `cargo test -p bondline-io --test cdxml_write --locked` failed with missing write_document
and to_cdxml APIs. Separately, existing from_cdxml calls in `--test cdxml_page` reproduced
the behavioral gap: explicit 312.5×456.25 Poster extents yielded the supplied 72×144 preview,
and a Poster with only Width was accepted. Logs under ignored target/: cdxml-writer-red.log,
cdxml-export-red.log and cdxml-poster-red.log.

GREEN: four standalone writer tests cover independent Expat decoding, Unicode/whitespace,
invalid-operation rollback, root/stack balance, reserved closing-size accounting and exact
depth/node/attribute bounds. Four io export tests use external Expat/Decimal projection for
all sixteen model orders and elemental metadata; test sparse/maximal application IDs and
globally unique source IDs; retain borrowed exact source with modified-source refusal; and
reject unrepresentable isotopes/syntax-budget excesses without partial export. Two reader
tests cover complete, absent, default-Pages, partial, non-finite and out-of-range dimensions.
The native model encoder does not serialize general chemical text/styles/stereo display.

The first external-consumer attempt failed because system Python had no RDKit. An isolated
ignored .cache/oracles/format-python environment was created; no system environment changed.
Actual versions: CPython 3.13.0, RDKit 2026.09.2, NumPy 2.5.4, Pillow 12.3.0. requirements-dev.txt
pins those consumers; CI installs them too, but hosted execution remains unperformed. PyPI
metadata was reviewed for CPython 3.13 wheels on the three configured native platforms.
Installed MolsFromCDXML APIs were inspected before use.

The final consumer test starts with registered bond.create, calls StoredDocument::to_cdxml,
then runs external RDKit. It confirms two carbons, one single bond, three hydrogens per
carbon, canonical SMILES CC and formula C2H6, with unchanged engine state. This is actual
engine-to-format interoperability for that scene; it does not verify a Bondline formula,
SMILES, stereo or mass algorithm, general appearance, native save/reopen or all bond kinds.
The Python reference remains an external test process, not a linked/shipped dependency.

The extended pinned ASan harness completed 10000 runs with seed 3641879538, max_len 65536
and timeout 5: coverage 3402, features 9242, corpus 600/162 KiB, reported RSS 161 MiB.
It now exercises writer escaping/rollback and explicit Poster projection. Its mutation
chooses a size different from the source, so an already-72×144 Poster cannot produce a
vacuous stale-source check. Only the new dimension-related model errors were added to its
existing unsupported-page cases. Seeds and reports are ignored, committed fixtures unchanged.
Stable rustfmt formatted the isolated harness; the pinned fuzz nightly lacks cargo-fmt and
was not modified. No fuzz result establishes exhaustive conformance/security.

Final aggregate: `BONDLINE_PYTHON=.cache/oracles/format-python/bin/python cargo xtask ci`
passes **144 tests**, rustfmt, warnings-denied all-targets clippy, layers, declared native
licenses/assets and the unchanged corpus scope. Windows/Linux all-targets source checks pass;
native runtimes and hosted CI remain unexecuted. Runtime package counts remain 150/161/219
and two bundled assets; Python consumers are test-only. The isolated harness format check
also passes. No native UI/renderer code changed or new native-window evidence was claimed.

## Reopened native-profile editing, 2026-11-13

RED: the new engine reopening test failed with UnsupportedDocumentPreview on the original
408-byte native v0 vector. Io tests also failed with the missing is_fully_mapped API.
GREEN: the constructor now compares the complete original bytes with a native re-encoding,
caching only a full equality result. The profile keeps original source/baseline and supports
headless render, bond creation, encode/reopen and exact Undo/Redo; CPU/GPU PNG bytes agree
after Undo. Independent Expat verifies all revised attributes, and hashlib checks the pinned
v0 SHA-256. Creator-name-only, changed styles, captions, comments/PIs, unknown properties,
future versions and noncanonical quotes remain opaque, byte-exact and edit-guarded.

Final cargo xtask ci with the isolated BONDLINE_PYTHON passes **147 tests**, formatting,
warnings-denied clippy, layers/licenses and the unchanged corpus scope. Windows/Linux
all-targets source checks pass. The extended pinned ASan run completed 10000 runs with seed
3641879538, max_len 65536 and timeout 5: coverage 3605, features 10562, corpus 632/168 KiB,
reported RSS 171 MiB. The harness checks changed-profile encode/reopen or opaque-edit refusal.
Logs: target/reopened-profile-red.log, native-profile-red.log, native-profile-ci.log and
native-profile-fuzz.log. No native UI/file workflow or general imported editing is claimed;
the workspace guard still awaits file-workflow wiring. No dependency or asset changes.

## First hosted matrix and Linux runtime correction, 2026-11-13

User-requested public repository creation triggered run 43448214670 at product checkpoint
3d3dd44. Hosted aggregate checks pass on macos-15, windows-2025 and ubuntu-24.04, including
147 workspace tests and real GPU checks. Mac and Windows actual three-frame native-window
launches pass. The separate Linux ASan smoke completed successfully. See docs/ci.md for the
durable run link and evidence limits; Windows interactive chemical editing is not verified.

RED: Ubuntu's actual window step failed loading libxkbcommon-x11.so.0/.so from xkbcommon-dl
0.4.2, despite its aggregate checks passing. The saved ignored log is
target/github-linux-43448214670.log. Pinned source tries those exact names; Ubuntu's public
noble source-package index identifies libxkbcommon-x11-0 as a separate runtime package.
The CI apt list now includes it. No native test, assertion or product path was disabled.
PyYAML 6.0.3 validates the corrected workflow, three-platform matrix, package entry and
unchanged Linux native smoke command; git diff --check passes. This configuration-only
correction needs actual hosted GREEN; YAML validation alone is not a successful Linux
launch. Parallel file-command/UI work is excluded from this change.

GREEN: corrective run 43451706668 at 8672de0 completes successfully: all three hosted
aggregate checks, actual native-window launches and Linux ASan. Reported backends are Metal/
Apple Paravirtual device on Mac, Dx12/Microsoft Basic Render Driver on Windows, and Vulkan/
llvmpipe LLVM20.1.2 on Linux. The latter two are CPU software adapters. These checks cover
published product code 3d3dd44, not the new file-workflow WIP or interactive Windows/Linux
chemical editing. Saved ignored log: target/github-native-43451706668.log.

Development command correction: the first local WIP aggregate used a workspace-relative
BONDLINE_PYTHON and four independent CDXML tests failed to locate it from their package
working directories. The documented venv examples now use an absolute $PWD-prefixed path.
The corrected invocation passes the full local WIP gate (192 tests); no assertion changed.

## Native filesystem CDXML adapter, 2026-11-13

RED: file-io-red.log lacks the file API; cancellation at Publish incorrectly wrote bytes;
replacement of an unrelated destination during encoding was not detected; injected same-inode
chmod0644-to-0600 could republish wider access. The focused behavioral RED logs are
file-io-cancel-red.log, file-io-metadata-red.log and file-io-security-red.log under target/.
Recovery-helper RED and GREEN exercise actual original files and a competing destination.

GREEN: cargo test -p bondline-io --locked passes 44 tests on native Mac, including 18 file
cases. Tests cover source passthrough, independent expected bytes, edited native encoding,
bounded/malformed/special inputs, cancellation, atomic no-clobber, symlink/readonly handling,
injected pre/post-publication failures, ACL/quarantine/binary xattrs and recovery retention.
Actual Linux arm64 Docker on official Rust1.94.1-bookworm passes nine file unit and eight
integration tests, including raw invalid-byte filenames, FIFO and POSIX ACL bytes on native
container storage; repository bind mount was read-only. This is not APFS emulation or a GUI
check. Image digest: sha256:6ae102bdbf528294bc79ad6e1fae682f6f7c2a6e6621506ba959f9685b308a55.
Logs: target/file-io-green.log and target/file-verification-linux-io.log.

Native/Linux/Windows all-targets io clippy with -Dwarnings and formatting pass. Declared
runtime license closures are 153/164/219 packages with two assets. Platform/metadata limits,
dependency pins and missing upstream notice are explicit in native-file-io.md. Windows
native file/ACL tests remain unexecuted; compilation is not that evidence.

The isolated fuzz lock retains all 157 prior versions and adds only seven required filesystem
dependency entries. Pinned ASan completed 10000 runs with seed3641879538/max_len65536/timeout5:
coverage3656/features10813/corpus669 entries175KiB/reported RSS178MiB. Log:
target/file-io-fuzz.log. Harness and committed fixtures stayed unchanged; this checks the
existing parser build/smoke with the new graph, not filesystem behavior or exhaustive safety.

## Cancellable macOS native Quit, 2026-11-13

Native RED: on binary86a09496e218b894669135d4659e5619868bc55c6afa9db696fc71126e5cac1c,
Cmd+Q discarded a fourth unsaved fragment despite the File-menu close guard. Exact source
inspection showed winit0.30.13's native menu sends terminate: while its delegate lacks
applicationShouldTerminate:. Apple documents that delegate's termination reply; the pinned
fork now restores Winit windows and queues their existing CloseRequested events, returning
Cancel while client approval runs, or Now with no Winit windows. Source/unsafe reasons and
exact upstream archive/VCS identities are in vendor/winit/BONDLINE-PATCH.md.

Native GREEN with new file UI on binary748b1c655fa8e41575b3c36f939f54d7ccd478b3b49e4957e6c0643fdc1df031:
dirty Cmd+Q prompts; Cancel retains the document; Cmd+W prompts; Quit then Save opens the
chooser, saves and exits. Native application-menu Quit separately prompts, and Discard exits.
External Expat/RDKit checks the one-fragment saved file as CC/C2H6; its SHA-256 is
023f407a7bca28831b9a1fc6c66ea47410e7fa491fd972726d794d531465b7ea.
The native red window close also prompts and Cancel retains the dirty document. The test
used the UI agent's sole CUA session; root sent no concurrent native input. This proves the
native route with that UI build, not atomic multi-document closing or session logout/shutdown.
Hidden/Dock visibility and Touch Bar hardware remain unverified.

An archive comparison verified all217 published winit paths. Four CC-BY-SA documentation
illustrations are omitted from the vendor copy; their attribution is retained. All remaining
213 files are byte-exact except src/platform_impl/macos/app_state.rs, plus one patch record.
No runtime source or font is changed by the omission. The retained Apache LICENSE
SHA-256 is6dc0e068dcf3a5bc8e054205b85b7720e1d49265bbc64bf515d2cf79197df69a.
Cargo update changed only winit's registry-to-local source; versions/graph stayed fixed.
Local aggregate with this bridge passes193 tests, formatting, clippy-Dwarnings, layers,
licenses153/164/219 and the documented corpus subset (target/files-ci-final.log).
Current native file UI remains a separate integration commit. These checks do not prove
the outstanding Mac accessibility SetValue bridge, whose upstream selector is empty.

## Windows private-DACL native reference correction, 2026-11-13

Actual Windows job131993129533 in run43456850621 passed six of seven io unit tests,
including original DACL/Zone.Identifier preservation and recovery. The private-temporary
test failed because Windows returns D:PAI(A;;FA;;;OW), while the test expected D:P(A;;FA;;;OW).
Microsoft's security-descriptor format defines P as protected and AI as auto-inherited;
AI is a descriptor control flag, distinct from an inherited ACE's ID flag. Both exact
full strings preserve the sole unflagged owner-rights full-access ACE and protection.

The test now accepts only those two complete representations. It retains the competing
data-open refusal, protected owner-only ACE and owner read/write checks; no production
behavior changed. Cross-target clippy-Dwarnings passes. An actual hosted rerun is required
before GREEN is claimed. Evidence: target/file-io-windows-hosted-red.log and
target/file-io-windows-ai-clippy.log; source FILE-WINDOWS-DACL-FLAGS.

## Registry CDXML files and shared executor, 2026-11-13

RED: six independent acceptance tests failed for missing registered commands/state in
target/file-verification-red.log; prepared completion lacked Engine::finish; adapters still
reported six commands. GREEN:54 focused command/CLI/MCP/loopback tests, plus nine Mac file
acceptance cases, check actual files with external Expat/RDKit. Cases cover draw/save, separate
process reopen/edit/overwrite/reopen, exact opaque passthrough/refusal, missing/malformed/
multi-page/oversized imports, no-clobber, dirty/Undo/Redo/no-op and complete-state retention.
CLI, MCP tools and authenticated sockets call the same eight commands; sockets notify repaint.
Logs: target/file-verification-all-adapters-green.log, file-verification-green.log and
file-verification-clippy.log.

Deferred tests prepare real opens/saves, edit or replace the session, complete work, then
apply it. Stale/cancelled opens retain state; old-revision saves keep current dirty; older
generations cannot relabel new paths. Worker tests cover exhausted budgets, clone sharing,
already-cancelled work, dropped futures, retained permits and short-edit commit boundaries.
Storage tests count/retain saved baseline and evict nonessential history. Root reviewed these
apply/worker/publication boundaries before integration.

Actual Linux RED: valid UTF8 alias to a raw0xFF canonical parent published before returning
save_published. Canonical preflight now returns invalid_path before create/overwrite, retaining
old bytes, while native PathBuf writes work. Linux Docker test/clippy pass; source mount stayed
read-only and invalid-byte fixtures lived only on container-native /tmp. Logs:
target/file-verification-linux-red.log and linux-green.log. Mac cannot fabricate those filenames.

Full local aggregate with the native Quit bridge passes193 tests and configured gates;
Windows/Linux all-target workspace checks pass (target/files-{windows,linux}-check.log).
Native file-menu verification is separate; no generic imported CDXML writer, chemistry
implementation or full reference File-menu parity is inferred.

## Native accessibility text values, 2026-11-13

Actual native RED: app.setValue on Folder/File name did nothing even after egui input tests
passed. Exact accesskit_macos0.17.4 source showed an empty setAccessibilityValue: implementation.
The pinned bridge now dispatches supported editable string values; it validates nullable
NSObject/NSString, UTF16/UTF8 bounds and NUL, rejects malformed UTF16, and tracks/prunes
advertised action IDs across partial tree updates. No dependency version/graph was upgraded.

Three native root-lock unit tests and clippy-Dwarnings pass, including exact Unicode payload,
unsupported/readonly/disabled/numeric targets, size/NUL/surrogate errors, action withdrawal,
partial updates and removed nodes. Sources/pins/full retained notices and unsafe reasons are
in vendor/accesskit_macos/BONDLINE-PATCH.md. The root Mac CI now executes those tests/clippy.
Logs: target/accesskit-macos-{red,green,root-lock-tests,root-lock-clippy}.log.

Native GREEN on binary6a4ff7786a5a74b51710c95e85618536b69a3acee7383a236e7857b89444f891:
actual Folder+Go and Unicode filename AX SetValue change their fields and save successfully;
draw a second fragment, Cmd+Q/Save, relaunch/Open, see two fragments, draw a third, Undo to
clean two, then Quit. Independent Expat/RDKit confirms two CC/C2H6 fragments in saved
native-file-ui-AX-λ.cdxml, SHA b0272acb0013ed45ee2a09bd68989927245faff73bc9f61a7777b5cf30bab590.
The UI agent closed the app and its sole CUA session. This is two chooser controls on this
Mac build, not full VoiceOver, other controls or cross-platform accessibility evidence.

Exact final-source recheck: rebuilt binaryf331610a4689105b11495016c18513e3a2a69c04921acfeb82c3e25e93961b1d
differs after the cache refactor, so the UI agent repeated actual AX Folder/Unicode filename,
Save/Open, edit then Quit Cancel (two fragments retained dirty), then Quit Save/exit. Its
independently decoded file has the same two-fragment SHA b0272acb0013ed45ee2a09bd68989927245faff73bc9f61a7777b5cf30bab590.
App/CUA session closed. Final cargo xtask ci passes193 workspace tests plus3 native bridge
tests, clippy (including the bridge), fmt, layers, licenses and corpus. Log:
target/files-ci-integrated.log. This is not a claim of full platform/reference parity.

## Native file menu and chooser integration, 2026-11-13

RED before code: absent chooser API; hidden linked folders; reopened complete profile refused
as unmapped; stale receipt allowed close; close shortcut/Escape missing; accessible SetValue
unhandled; rejected value lost retry action; previous Decode error leaked into valid retry.
Targeted logs: target/native-file-ui-{red,symlink-red,profile-red,stale-receipt-red,
close-shortcut-red,accessibility-red,accessibility-retry-red,retry-message-red}.log.
All24 UI tests and all-targets UI clippy pass after the corresponding fixes.

Actual Mac workflows include pointer-create/Save As/edit/Save/overwrite, dirty Open Cancel
and explicit discard, close/relaunch/Open/edit/Undo, File Close Cancel/Save, native red close
Cancel, CmdW/Escape and native Quit/CmdQ Cancel/Save/Discard. Malformed input retains dirty
data after discard approval; opaque source stays guarded and saves byte-for-byte, hash
883f079b39c8bae01e82f7a926cf150ce0d579a77f4eb75462652fbf85074006. Folder Go/Up/row selection,
home initial location and final native AX Unicode filename are observed, not inferred.
Independent external XML/RDKit checks actual files; exact build/file hashes and limitations
are in native-file-ui.md. Startup geometry in the first earlier run was ambiguous, explicitly
undone to confirmed empty before authoring; no unknown geometry counts as acceptance evidence.

The UI uses the same commands; no render/parser semantics were weakened to pass. A supported
larger viewport is used for the profile test; minimum-label-scale refusal remains. Existing
human-approved golden is unchanged; these native screenshots are observations, not new approvals.

## Full hosted file-workflow GREEN and main integration, 2026-11-13

Run43463687047 at exact product ccf8c50 completed all four jobs successfully. Actual tests:
Mac196 (193 workspace+3 native bridge), Linux194 and Windows188, plus real native-window
launches on each and the bounded Linux ASan job. Full ignored log:
target/github-files-43463687047.log. This includes command/CLI/MCP/loopback file workflows;
it does not substitute for interactive Windows/Linux file-menu or full screen/flow parity.
The earlier Mac capacity failure remains a separate real infrastructure observation.
Main was fast-forwarded to these atomic feature commits after GREEN; no code changed during
merge. Documentation/research commits afterward do not change the tested product source.

## Standalone CDX syntax and checked property edits, 2026-11-13

RED preceded implementation: the actual syntax integration test failed with missing public
types (target/cdx-syntax-red.log); the property and independent targets failed with E0599
for missing replace_property (cdx-edit-red.log and cdx-independent-red.log). The first
external integration then exposed unsupported explicit-root EOF at byte65750; that failure
is retained in cdx-independent-first-integration.log. An initial all-tests invocation also
used a nonexistent interpreter path; it is an environment error, not feature RED evidence.

GREEN:13 CDX tests cover original exact spans/opaque bytes, all truncations of the small
implicit/explicit vectors, known-prefix/opaque-reserved header policy, foreign/stale handles,
short/extended length boundaries,64 MiB input/output,100000 records and256 open objects.
External Python struct independently checks record tables and every edited byte. Pinned
RDKit2026.09.2 actually reads both implicit/explicit originals as CC/C2H6 and their changed
bond order as C=C/C2H4. Four complete generated fixture hashes are asserted independently;
see xtask/oracles/cdx-README.md for sources, test-only backend provenance and consumer limits.
The public example's inconsistent bond IDs were not adopted as an oracle. No upstream
parser implementation or proprietary sample was copied into the product.

The flat parser never recurses over attacker-controlled data. Property edits retain all
unrelated bytes by checked splice construction, not an internal second parse; integration
tests and the fuzz harness reparse the output. Exact EOF is a Bondline strictness policy:
the external consumer accepts additional zero ends. CDX chemical/reference semantics,
new object-tree writing and application CDX Open/Save are still missing.

Final local cargo xtask ci passes206 workspace tests plus3 native AccessKit bridge tests,
fmt, warnings-denied clippy, layers, unchanged native runtime/asset licenses and the existing
two-file corpus. Log:target/cdx-ci-integrated.log. The corpus gate still has no CDX entry;
the new original fixture/oracle tests do not expand its fetched-corpus claim. Isolated fuzz
format checking also passes; both lockfiles only add the dependency-free original CDX crate.
Standalone CDX all-target source checks pass for Linux/Windows, and its library check passes
for wasm32-unknown-unknown (target/cdx-{linux,windows,wasm}.log). These are compilation checks,
not native execution or approval of the application's unresolved browser dependency graph.

The final frozen-source ASan check passes10000 runs, seed721533879, max_len65536, timeout5,
coverage166/features402/corpus52 entries3092 bytes/reported RSS44 MiB. It starts from the
small original seed plus prior ignored mutations, not a claimed exhaustive64 MiB corpus.
Log:target/cdx-asan-final.log. CI now seeds and runs this target separately from CDXML.
This is syntax/editing coverage, not filesystem, chemical interpretation or native UI proof.
No UI source changed; the previously recorded native app evidence remains its exact scope.

Published source checkpoint8547f4d is on feat/cdx-syntax, after the separate M0 audit48982d0.
Hosted run43475705005 is still in progress as of2026-11-13 17:49:56 UTC: its combined parser
ASan job passed, all three native jobs are running. No terminal three-platform success is
claimed for this slice, and main remains atfefab8a pending that evidence. A final independent
read-only review found no actionable issue in framing, ranges, ownership, bounded allocation
or preservation; it did not rerun tests or extend the advertised scope. STATUS.md records
the live job handles and next exact integration step.

## CDX oracle Windows newline regression, 2026-11-13

Run43475705005 is terminal: Mac/Linux native checks and both Linux ASan targets passed;
Windows job132058846940 failed at both independent record-table comparisons before RDKit
was called. Its expected LF protocol received CRLF from Python text stdout; the binary
fixture was already emitted through the binary buffer. Saved actual log:
target/cdx-windows-hosted-red.log. No Windows feature success is inferred from this run.

A new regression wraps stdout in a CRLF-translating io.TextIOWrapper on every host. It
first failed against the actual oracle, while retaining exact binary fixture output;
target/cdx-stdout-red.log. Emitting ASCII/LF record-table bytes through stdout.buffer fixes
the protocol at its producer. No assertion is relaxed and no binary content is normalized.
All14 CDX tests and package clippy now pass; logs cdx-stdout-{green,clippy}.log. Published
Python stream semantics are recorded in sources.md. The corrected isolated branch also
passes cargo xtask ci:207 workspace tests plus3 native bridge tests, fmt, clippy, layers,
unchanged licenses/assets and pinned corpus (target/cdx-stdout-ci.log). A new hosted run is required before
merging this branch; local success does not replace Windows execution.

Corrected sourcea73bed7 subsequently passed full hosted run43480423381: Mac210, Linux208,
Windows202 tests, actual native windows on all three and both Linux ASan targets. Exact
job IDs and adapter limits are in STATUS.md/ci.md. Main was fast-forwarded to this tested
product source; the integration/status commit changes no product code. No hosted CDX job
remains active. This closes that platform regression, not M0 or full M1/CDX parity.

## Runtime workspace identity for quality gates, 2026-11-13

An attempted shared-close aggregate reused a target-directory xtask binary containing the
other checkout's compile-time CARGO_MANIFEST_DIR. Its passing log names .cache/cdx-integration,
so it is explicitly excluded from shared-close evidence (target/shared-close-ci-wrong-workspace.log).
A subsequent correct-root attempt encountered incompatible cached first-party metadata;
shared-close-ci-stale-metadata.log is also not a pass. These were integration setup failures,
not missing close APIs in the actual source. Affected first-party artifacts were removed;
future worktrees must keep separate target directories.

CI, standalone asset lookup and corpus lookup now share Cargo's runtime workspace locator.
It uses frozen JSON output, validates the absolute manifest and fails without falling back
to another checkout. CI prints its resolved root. Two independent temp-workspace tests
first failed against the old compile-time selection (xtask-runtime-root-red.log), then
passed for root/member/nested invocation and invalid-manifest refusal without creating a
lockfile. All16 xtask tests and warnings-denied clippy pass; logs xtask-runtime-root-green.log
and xtask-runtime-root-clippy-green.log. The corrected integration run is recorded separately
with its actual source/test scope; neither discarded attempt contributes to that claim.

## Shared close registry, adapters and native shutdown, 2026-11-13

TDD RED preceded the new API (shared-close-core-red.log, close-adapters-red.log and
native-shared-close-red.log). Behavioral RED then caught approval surviving an actual
saved-baseline change (shared-close-baseline-red.log), repaint before response flush
(close-response-order-red.log), a lost close-Save retry (native-shared-close-retry-red.log),
and an ordinary picker accepting Save after shutdown was approved
(native-shared-close-picker-red.log). These failures were fixed without relaxing the intended
source/history/receipt/ordering assertions. An independent read-only review found the picker
ordering issue; its other adapter paths had no actionable finding.

Core47 tests include12 new close tests. They independently assert closed/default schemas,
exact source/model/history/baseline retention, monotonic epoch/atomic exhaustion, stale
revision/path/baseline invalidation without resurrection, Cancel/new Request ABA, matching
Save followed by fresh Request, immutable old authored results with current lifecycle metadata,
post-commit refusal, late Open rejection and truthful already-published Save receipts.
CLI/MCP/socket27 targeted tests cover nine-command discovery, all decisions/errors/epochs,
stdio survival, registered pre-spawn work, clone/drop/cancel lifetimes, reply flush barriers
and write/flush failure. UI28 tests retain file/drawing coverage and add shared-state adoption,
receipt/epoch continuation, busy cancellation, obsolete-picker ordering and failure retention.
Package logs:shared-close-core-final-{green,clippy}, close-adapters-{first-green,clippy},
native-shared-close-ordering-{green,clippy}.

The final aggregate was invoked from crates/commands, printed the actual root checkout and
passed233 workspace tests plus3 native AccessKit bridge tests, fmt, warnings-denied clippy,
layers, unchanged native license/assets and pinned corpus. It includes the named new close
and runtime-workspace tests, not the other worktree's old suite. Log:
target/shared-close-ci-integrated.log. The two discarded cache/setup attempts above are not
counted. No product parser/render/golden or dependency pin was changed for these passes.

Actual corrected native executable68048eaa0d11c2559d7a51dabf2b72fcc95790a54ff244eae5089248838334c3
on Mac/Apple M3 Max/Metal passed remote prompt/Cancel, stale/malformed retention, File Close,
CmdW/Escape, red Close and Quit Cancel/Save/Discard, plus actual reopened committed-edit
protection. Two simultaneous2048 CPU/GPU captures had pending replies after explicit Discard
approved; the process stayed alive, delivered both full successful replies, then exited0.
Independent Expat/ElementTree and RDKit confirm the actual493-byte saved Unicode-name file
as CC/C2H6, SHA9f72459be089ef815f5292478a80e105e4ccfef1b56f57914067f419735204eb.
Its original bytes survived later Discard. Details, earlier distinct a430 build and the
ambiguous unobserved drag/Q attempt are in native-file-ui.md. That ambiguity is not a claim
of committed-state loss or verified pending-input ordering. All controllers/apps/CUA ended;
credentials stayed memory-only. The approved golden remains unchanged.

Native interactive Windows/Linux, Dock/session/logout/multiwindow close and full reference
dialog parity remain unverified. Shared-close source72742fb now passes all four jobs in
[hosted run43488771976](https://github.com/thematheusmello/bondline/actions/runs/43488771976).
Mac236 (233 workspace plus3 bridge), Linux234 and Windows228 aggregate tests pass, as do
all three actual native window launches and both bounded Linux parser sanitizer targets.
The adapter scope remains Metal/Apple Paravirtual, Dx12/Microsoft Basic Render Driver(Cpu)
and Vulkan/llvmpipe(Cpu); no hardware-performance claim follows. Main is fast-forwarded only
after this GREEN. These jobs are terminal; no local or hosted job remains live.

## Shared delivered controls and guarded file requests, 2026-11-13

Core API RED preceded implementation (target/file-ui-core-red.log). An enabled headless
Close Window binding then failed a behavioral assertion and was fixed without disabling
the underlying document.close intent (file-ui-headless-close-{red,green}.log). A missing
local .venv interpreter was an environment error, not a behavioral RED; the corrected run
uses the existing isolated interpreter described in development.md.

Core61 tests now pass, including independent exact storage/history retention, host absence,
monotonic request/cancel identity and exhaustion, invalid combinations, stale selection,
close Cancel/new Request, baseline/path ABA and no-op preservation. Real saves exercise
same-revision baseline/path changes; an old close-Save still reports publication without
approving a newer close. Logs:file-ui-core-{all-green,clippy,fmt}.log.

Adapter API RED is ui-contract-adapter-api-red.log. All36 CLI/MCP tests pass, with exact
11-name discovery and closed schemas, headless refusal, typed native bindings, requested
versus trusted-accepted state, repaint on presentation-only changes, old operation/current
request separation, Running Cancel refusal, busy reservation before consume, origin-bound
work, stale/cancel/close/path ABA and external saved-file consumption. Logs:
ui-contract-adapters-first-green.log and ui-contract-adapters-clippy.log.

UI API RED is native-file-controls-red.log. Review then found two real behavioral failures:
Cancel from an old picker targeted a replacement, and a known-path Save failure lost its
required overwrite:true on retry. native-file-controls-regressions-red.log records both;
the fixes retain identity and an explicit fixed-path retry confirmation. A disposable
filesystem test turns the target into a directory during prepared work, checks retained
bytes, restores it and completes the exact guarded retry. UI36 tests pass, including
shifted SaveAs/Save and Redo/Undo precedence, independent physical page values and existing
close/file/drawing coverage. Native execution is recorded separately; egui tests alone do
not establish a displayed window.

The frozen-source aggregate cargo xtask ci passes265 workspace tests plus3 native AccessKit
bridge tests, fmt, warnings-denied clippy, layers, unchanged153/164/219 native declared
runtime license closures/two assets and the pinned corpus. It printed the actual root
checkout; log target/file-controls-ci-integrated.log. No dependency pin, parser, renderer,
asset or approved golden was changed. Corpus still lacks application CDX/MOL/chemistry
coverage. The new source needs its own hosted run before main integration; earlier close
run43488771976 is terminal GREEN and cannot substitute for these changes.

Actual native acceptance on the owned4a5425b0... executable passes the shared menu/shortcut/
remote picker, stale Cancel, AX SaveAs, known-path Save, malformed retention, actual Save
failure/explicit fixed-path retry, committed-edit Quit Cancel/Save, reopen and two concurrent
capture reply-drain workflows. Independent XML/RDKit reads the1125-byte three-fragment file,
SHA771fdaf88be2129d633692bfca5a81627885c6bb7022777ca5c1b346016f7347. All processes/CUA ended.
See native-file-ui.md for exact binary hash, excluded initial ambiguity, resolved stale-app
guard and platform limits. No new golden approval or interactive Windows/Linux evidence is
claimed; the existing approved scene/hash remains unchanged.

Published frozen source65b5fb8 passes all four jobs in
[run43496010615](https://github.com/thematheusmello/bondline/actions/runs/43496010615).
Mac268 (265 workspace plus3 bridge), Linux266 and Windows260 tests pass, as do all actual
native windows and both bounded Linux parser ASan targets. Source was integrated into main
only after GREEN; later research/status commits change documentation alone. Software-driver
scope and interactive Windows/Linux limits remain explicit in ci.md/native-file-ui.md.
These jobs are terminal; no local/native/CUA/research/hosted work remains live.

## Repository presentation and community documentation,2026-11-13

Documentation/configuration checks only: no artificial Rust behavior test or full Cargo
rerun was needed for this scope. SVG banners were rendered with rsvg-convert and visually
inspected. Both1160×832 JPEGs are unchanged native capture buffers, hash/MIME/dimension
checked against their originals. They show product65b5fb8 / owned binary4a5425... on Mac
Metal; no golden was regenerated. Capture scene is three original CC/C2H6 fragments,
independently consumed by RDKit. All native/controller/CUA processes exited; exact records
are in docs/media/provenance.json and SOURCES.md.

Root link/heading/UTF-8/whitespace/SVG/accessibility/JPEG/hash checks pass; log
 target/readme-captures/docs-validation.log. The community agent separately validated all
three YAML files against published SchemaStore schemas, checked duplicate keys/form IDs
and all seven new docs/templates' links; target/community-docs-validation.log records hashes.
GitHub private reporting was enabled and independently read back as true. No issue, report,
PR or external message was sent. Rendered GitHub README/form checks follow publication;
source/configuration checks alone are not a claim of their displayed result.

In parallel, user-directed M1 implementation now starts MOLV2000. Its source/tests are
separate from this documentation commit; see the next codec evidence, not these doc checks,
for any new format claim.

Published documentation91e2f73 was checked in a new isolated Chrome tab on the public
repository page: all8 README images loaded, headings/navigation were present, and actual
rendered banner, badges and native workspace image were visually inspected. GitHub also
recognized Contributing, both licenses and Security in repository navigation. The issue
chooser redirected the unauthenticated preview to sign-in; form submission/rendered fields
remain unverified, and no login/issue/report/PR was submitted. The preview tab was closed.
A requested screenshot file path was outside the browser tool's permitted roots, so inline
viewport evidence was inspected instead; no saved browser screenshot is claimed.

## MOL V2000 codec and corpus,2026-11-13

API RED preceded implementation: target/mol-v2000-red.log and mol-header-red.log. A real
unsupported-diagnostic failure is mol-diagnostic-red.log. The isotope comparison failure in
mol-precedence-red.log exposed a published-spec versus pinned-consumer disagreement; it was
not normalized or hidden. Tests now separately assert normative global override, actual
RDKit legacy-delta behavior, exact original bytes and consistent canonical-output readback.
The14 MOL tests plus44 existing IO tests pass; target/mol-io-all-green.log. Targeted
clippy/fmt pass. External RDKit checks CH3/radical/H, CC, charge/isotopes, geometry, all118
symbols and independently generated input. Decimal independently checks fixed4dp rounding.
Original fixed-column ethane fixture282bytes/hash828c1b58... supplements the licensed170-byte
CH3 fixture. Headers and unsupported data retain their stated source/export boundaries.

Corpus RED explicitly required MOL coverage1 while the old command reported0; same assertion
now passes. All18 xtask tests pass, including deliberately mismatched raw/model element,
radical/H/coordinate rejection. Both normal and offline corpus commands pass; logs
mol-corpus-{red,green,default,tests,clippy,fmt}.log. Manifest remains two exact pinned files,
now one CDXML and one MOL input. Fetched CDX and Bondline chemistry calculations still lack
coverage; the command prints those gaps.

New mol_parse ASAN target had a missing-target RED, then passed10000runs, seed619734821,
max_len65536,timeout5 on pinned nightly2026-04-01/cargo-fuzz0.13.2/libfuzzer0.4.13. Final
coverage947/features2007,163retained inputs28445bytes/RSS66MiB; largest exercised input282bytes.
This is a bounded smoke, not broad large-input coverage. Log mol-asan-first.log; fuzz target
clippy/fmt pass. Initial seeds are the reviewedCH3 and original fixed-column ethane, copied
only into ignored writable corpora. No lock/runtime dependency changed. The workflow now
runs all three parser targets; hosted execution of this new source remains to be checked.

Actual-root aggregate cargo xtask ci passes281 workspace tests plus3 native AccessKit bridge
tests, fmt, warnings-denied clippy, layers, unchanged153/164/219 native declarations/two
assets and the expanded corpus subset. Log target/mol-codec-ci-integrated.log printed this
checkout. No application MOL command, native MOL file workflow, SDF/V3000 or chemistry
algorithm is claimed by this library/parser milestone. All local jobs are terminal.

## MOL application files — local integrated checkpoint

Test-first logs `target/mol-owned-red.log`/`mol-native-red.log`,
`mol-command-red.log`, `mol-adapter-red.log` and `native-mol-ui-api-red.log` record missing
owned-source/I/O/command/adapter/UI behavior before implementation. Core's six original MOL
tests failed unknown_command. Subsequent integration compile errors were interface migration
errors, not independent behavioral RED evidence. Additional core receipt regressions were
added after the private plan markers existed; they are GREEN regression coverage, not a
claim of earlier TDD. The failed-retry filename regression has its own observed
`native-mol-ui-retry-name-red.log` before the fix.

GREEN: IO70, commands70, CLI/MCP40 and UI43 tests pass. IO tests cover retained header/frame/
source/baseline sharing, exact undo/passthrough, explicit layout/metadata refusal and the same
atomic publication/fault stages as CDXML. Core/adapter tests actually write files, reopen
through the selected decoder and run the pinned external RDKit consumer. They check declared
H/radical/formula/SMILES, default/custom coordinate mapping and bounded four-decimal precision;
these external results do not implement Bondline chemistry algorithms. Schemas expose exactly
thirteen commands, closed objects/defaults, invalid-frame errors and no source/header leak.

Stale opens retain complete state; an older published save leaves newer edits dirty. Known
Save uses the associated encoding even with a misleading extension. Same-path format changes
invalidate close/picker approval, unchanged same-format saves preserve it. Trusted file plans
carry request ID and pre-mutation currentness; format/baseline ABA cannot restore a stale
receipt. Direct saves omit those markers. UI tests refuse missing/wrong markers/format and
preserve existing close-epoch/revision/path guards. A truthful published receipt is not rolled
back because its presentation became obsolete.

Actual-root `cargo xtask ci` passed312 workspace tests plus3 native AccessKit bridge tests,
rustfmt, all-target clippy-Dwarnings, layer graph,153/164/219 declared native runtime closures,
two bundled assets and the actual CDXML/MOL corpus subset. Log
`target/mol-files-ci-integrated.log` identifies this checkout. No runtime dependency, lock,
shader, golden or platform publication algorithm changed. CDX corpus and chemistry calculation
coverage remain explicitly missing. Native MOL workflow inspection and hosted checks for
this application source are pending; do not infer either from the aggregate.

The standalone codec's first hosted run43504072080 failed Windows fixture checkout identity;
Mac/Linux/three ASAN targets passed. Correction670178b changes only pinned fixture Git
attributes and a provenance report, not parser/oracle assertions. The independent
core.autocrlf=true RED/GREEN is in research/mol-fixture-checkout.md. Corrected hosted run
43507660824 is in progress at this checkpoint; do not restart it or claim it passed yet.

## MOL application files — native and hosted acceptance

The preceding pending checks are now terminal at source
`c879f4c95964617a86cc2d1e466515c5aeb31f29`. Hosted run43508330802 passes all jobs:
Mac315 (312 workspace plus three bridge), Linux313, Windows307; all three actual windows
and CDXML/CDX/MOL bounded sanitizer targets. Corrected standalone codec run43507660824 also
passes every job (Mac284/Linux282/Windows276). See ci.md for exact IDs/adapters. First-run
Windows fixture failures remain recorded, not relabelled as successful.

Native Mac acceptance completed 2026-11-13 21:21:39 UTC on the unchanged owned binary
`eeb98144f0fe0b7aaba24479d5950ec7de622efe9da985a02995830d3b387927`, Apple M3 Max/Metal.
Actual canvas/chooser/AX/shortcut execution verified explicit/default format choice,
Unicode Save As, reopen/known Save, malformed retention, multi-fragment refusal/Undo,
three retained source headers, misleading suffixes and exact failure/overwrite retry.
MOL-to-CDXML source loss refused without publication. The CDXML regression under a.mol
name remained editable, Quit Cancel retained state and Quit Save exited successfully.
Two pending CPU/GPU capture replies completed before approved shutdown. No product source
changed after the aggregate/native freeze. Native-file-ui.md records exact file hashes and
consumer results; original evidence is under target/native-mol-ui.

All native/controllers/CUA ended; no credential was logged or committed. This verifies the
stated C–C/empty source workflow, not general MOL rendering/editing or interactive Windows/
Linux. Full M0/M1, broad chemistry, source conversion and later milestones remain incomplete.
SDF work is a separate next codec feature; no new format support follows from these results.

## SDF V2000 container codec

Primary evidence is the unchanged CTFILE-2026 PDF, chapter10 printed97–99/PDF indices100–102,
visually reviewed. The original two-ethane/ordered-field fixture is690 bytes, SHA-256
`f95838cbb55c107c649ce2eb99ac62e191f747ce6994a171e64019f8e6c53d1a`, original MIT OR Apache-2.0.
No proprietary example/template was copied. The format source and fixture README distinguish
normative framing from conservative ASCII/header/EOF/resource policies.

Actual RED: missing API in target/sdf-codec-red.log; a writer accepted standalone MOL
program/comment/property content beginning with a reserved delimiter and emitted invalid
SDF (sdf-mol-delimiter-red.log). The preflight fix refuses without escaping/changing opaque
MOL bytes. A separate sdf-header-red.log covers the documented conservative incomplete-
parenthesis metadata profile; unsupported_content is not a general malformed-standard claim.

All12 final codec tests pass in target/sdf-codec-final-green.log. They compare every ordered
record/field/raw-value/span with an independent Python scanner; direct external RDKit reads
both Molfile graphs. A named-field writer faces the full SDF supplier, and an external
RDKit writer supplies empty-padding input. Empty/multiline/spaces-only/duplicate/DT/combined
headers, prefix delimiters, CRLF, incomplete later records and hostile writer inputs have
decisive expectations. Boundaries exercise1024 records,65536 fields, one million lines,
64MiB input/output and header/value line caps; assertions were not weakened to hide failure.

Pinned RDKit2026.09.2's full supplier loses later data and the second record at this valid
DT-only header, with either exact or suffixed delimiter. Removing the DT-only item restores
two records. Its duplicate property map retains only the last value. These are separate
consumer diagnostics; ordered raw-field fidelity does not rely on that lossy map. The
independent scanner and primary literal vectors remain the data-preservation oracles.

Final frozen source SHA-256:
`68745c5cddf98d265ad7eb8ff5df5db91a55e83b03ae81a80d2ef92e2cdf57a3`.
Warnings-denied IO and fuzz-target clippy, workspace/fuzz formatting pass. New sdf_parse
had a missing-target RED, then the final fresh-seed ASAN run passed10000 iterations,
seed826403517, max_len65536, timeout5: coverage813/features2794,240 retained inputs/reported
102KiB/RSS76MiB. Largest exercised input690 bytes; this is not large-input/security proof.
Log target/sdf-asan-final.log; seed/fuzz source/locks remain separate from writable mutations.
The earlier pre-header-guard ASAN pass is not substituted for this frozen-source result.

Actual-root cargo xtask ci passes324 workspace plus three bridge tests, rustfmt, all-target
clippy, layers, unchanged native licenses/assets and existing CDXML/MOL corpus checks.
Log target/sdf-codec-ci-integrated.log. No runtime dependency/lock/UI/renderer/golden change.
The workflow adds a fourth bounded sanitizer target; hosted execution is pending publication.
Fetched SDF corpus coverage is deliberately the next feature, after a real public source
commit exists for this original fixture. No native SDF app workflow, V3000 or completed M1
is claimed. All local codec/fuzz/aggregate jobs are terminal.

## SDF fetched corpus and source acceptance

The preceding codec's hosted run43512236206 is now terminal GREEN at source
fb86832a0c9b7744fb3ef8f8115c7ed296164505: Mac327, Linux325, Windows319 tests, all three
actual windows and all four bounded sanitizer targets. See ci.md for IDs/adapters.
The subsequent narrow Git whitespace attribute preserves the intentional three-space data
value without changing any fixture byte or behavioral assertion.

Corpus now uses two mandatory version1 bundles, retaining the RDKit manifest exactly
(SHA820739ca30312745b53df2839f0238b30d1470e73446430b4c954a142cb88ca0). The original SDF
bundle pins that actual public codec commit and its MIT OR Apache-2.0 fixture. The cache was
absent before the normal command's bounded HTTPS fetch; actual downloaded690 bytes/SHA
f95838cbb55c107c649ce2eb99ac62e191f747ce6994a171e64019f8e6c53d1a matched the reviewed source.
No local fixture fallback or invented future pin exists. Fetch attestation is
target/sdf-corpus-fetch-attestation.log.

Real RED: loader rejected unknown format sdf; the old successful corpus output failed a
separate expected SDFcoverage(files,records)=(1,2) assertion; a real Unix symlink regression
showed cache directory creation outside the checkout before confinement. Logs:
target/sdf-corpus-{loader-red,coverage-red,cache-red}.log. The fix validates all bundles,
notices/global cache names before cache/network I/O and checks the nearest existing cache
ancestor before mkdir plus the canonical path afterward. This prevents the reproduced
existing-symlink case; it does not promise atomic confinement against concurrent renames.

All25 xtask tests pass, including missing/oversized/escaping manifests/notices, global entry
limits/case aliases, offline missing/corrupt cache, exact counter/record reporting and
deliberately changed duplicate/DT/order/range/model data. Every ordered span/value agrees
with the independent scanner; both separately consumed molecular projections agree. The
full supplier's one-entry result is explicitly diagnostic, preserving the known DT-only
consumer loss. Normal/offline outputs are identical. Targeted clippy/fmt pass; exact source
SHA1210a4e96ab23838b8520a092369e424a846ff7477d2d507c30f010785e62b9f.

Actual-root cargo xtask ci passes331 workspace plus three native bridge tests, rustfmt,
all-target clippy, layers, unchanged153/164/219 runtime closures/two assets and all three
pinned files from two sources. Log target/sdf-corpus-ci-integrated.log. No runtime/lock,
IO codec, UI, renderer or golden change. Hosted corpus-feature checks follow publication.
CDX corpus and Bondline chemistry calculations remain explicitly missing; SDF application
import/export, V3000 and complete M1 remain unimplemented. V3000 work is isolated in a
different checkout/target; its preliminary tests are not part of these results.

## Corpus metadata checkout correction

Fetched-SDF source504be5f passed Mac/Linux/ASAN in run43515232724, but Windows
132195408100 failed the exact original manifest hash: CRLF checkout became1067 bytes/
cb9c0ddd07eadf12ee4c1723dbb4431746e6d656833d7f83490563b9e04fec3a instead of1041 bytes/
820739ca30312745b53df2839f0238b30d1470e73446430b4c954a142cb88ca0. The source manifest's
committed bytes and semantic fields were unchanged. This run is terminal failed.

Actual isolated core.autocrlf=true Git clones reproduce that exact RED and then GREEN
after `/docs/corpus/manifest.json -text`; no normalization, altered hash/assertion or license
change. Logs/metadata are under target/sdf-manifest-autocrlf-yoq1rmg2; the research report
records the source/public rule. CDX byte preservation is also explicit for already published
original binary vectors. Configuration-only correction: relevant byte/checkout checks pass,
no artificial Rust test or Cargo rerun claimed. A new hosted source run follows publication.

## Original positive CDX source fixture

The already-published, unchanged original struct generator now has its actual explicit-root
output committed at crates/cdx/tests/fixtures/original-ethane-explicit.cdx:65750 bytes,
SHA63ef1a74133e45de915d5fd25b2ca3321a95dbe5114873a5a59f2f45217000d2. Generator SHA remains
5ef6b183d7933b4d3857d1f36b2dd5df2478b089f543f89a4d04a6c0ae7201b6; the original70-byte
fixture remains unchanged. Both are original MIT OR Apache-2.0 source vectors, not samples
from the reference application. The missing-file RED compiled then failed NotFound;
after materialization all15 CDX tests, targeted clippy/fmt pass. Logs:
target/cdx-fixture-{red,green,clippy,fmt}.log.

The actual file must match independent generator bytes and all29 struct record events,
including ownership, extended-length spelling and payloads. Existing external RDKit
verify1/explicit reads CC/C2H6. Replacing only BondOrder0x0600 with2 must match the exact
independent double-bond vector and read C=C/C2H4. Its byte guard was not weakened for the
empty70-byte fixture. No production parser or oracle changed.

This positive scene is65750 bytes, beyond the former65536 fuzz-run limit. Both original
seeds now run under a deliberate131072-byte bound. Fresh-corpus ASAN passes10000 iterations,
seed721533879, timeout5: coverage165/features447,57 retained inputs/reported70KiB/RSS136MiB;
largest exercised input65750 bytes. Log target/cdx-positive-asan.log. This is source/opaque-
editing fuzzing, not independent chemistry verification for every mutation.

Actual-root aggregate cargo xtask ci passes332 workspace plus three bridge tests, fmt,
all-target clippy, layers, unchanged runtime licenses/assets and existing CDXML/MOL/SDF
corpus checks. Log target/cdx-fixture-ci-integrated.log. The new source must be public
before a subsequent real fetched-CDX pin is added. CDX corpus remains explicitly missing
in this checkpoint; no foreign-file/general/app CDX compatibility is claimed. All local
fixture/fuzz/aggregate jobs are terminal. V3000 remains a separate checkout's source.

The corpus metadata correction c797262 now passes every job in run43518162847: Mac334,
Linux332, Windows324 and four bounded parser targets, with all actual window smokes.
Main was fast-forwarded only after GREEN. Exact IDs/adapters are in ci.md. Earlier
run43515232724 remains terminal failed; no assertion or semantic/source data was weakened.

## Fetched original positive CDX corpus,2026-11-13

Actual RED: the new manifest failed unknown-format decoding, and the old successful corpus
runner failed the independent expected-CDX-count assertion (0 instead of1). Logs:
target/cdx-corpus-{loader-red,coverage-red}.log. The new mandatory third source bundle pins
the real published original fixture at30ae30e73a4ee3b68d41069376fc5690faa99bc4. An actual
cache-missing bounded HTTPS download matched65750 bytes and
SHA63ef1a74133e45de915d5fd25b2ca3321a95dbe5114873a5a59f2f45217000d2;
target/cdx-corpus-fetch-attestation.log records the source/identity. No local substitution.

GREEN:28 xtask tests, targeted clippy/fmt and actual-root cargo xtask ci pass335 workspace
plus three bridge tests; target/cdx-corpus-ci-integrated.log. Four pinned files from three
source bundles cover one CDXML, one MOL, two SDF records and this one original CDX scene.
Normal/offline output agrees. Every29-record span/parent/length encoding agrees with the
independent Python struct table; exact source bytes and unchanged RDKit ethane guard pass.
Changed spans, opaque payload, bond order and a different valid explicit scene refuse.
The changed double bond independently passes C=C/C2H4, without weakening the original guard.
Foreign/general/application CDX and Bondline chemistry calculations remain missing.

An actual isolated core.autocrlf checkout first changed both Bondline manifest byte
identities. Exactly two -text rules restore them; existing manifest bodies/pins/notices
remain byte-identical. Logs target/cdx-corpus-checkout-{red,green}.log. The only lock change
is the existing first-party bondline-cdx xtask edge; runtime closures153/164/219 and two
assets remain unchanged. No parser, fixture, oracle, UI, renderer, golden or fuzz-lock change.
New hosted evidence must follow this feature's own source commit.

The preceding fixture source30ae30e passes every job in43520991476: Mac335/Linux333/
Windows325 tests, three actual native windows and four10000-run ASAN targets. Separately,
standalone V3000 source5caf6ce passes43519338382: Mac341/Linux339/Windows333, three native
windows and five10000-run ASAN targets. Neither run proves the uncommitted V3000 application
integration or the new fetched-CDX feature; do not substitute their results.

## 2026-11-13: standalone MOL V3000 codec

Work ran only in the isolated `bondline-mol-v3000` worktree from 2d665de, with its own
absolute CARGO_TARGET_DIR. The original/root working tree and its artifacts were not
used for builds or product edits. Source decisions follow CTFILE-2026 printed 7–13/32;
the cached PDF and original fixture hashes are recorded in sources/fixture provenance.

Actual missing-module RED: `target/v3000-red.log`. The first two literal sparse-ID,
valence/frame and continuation/quote/list tests became GREEN. Independent consumers
then exposed an attribute-only entry refused inside an opaque block
(`target/v3000-oracle-first.log`); preserving its complete syntax fixed that RED.
A separate primary-rule test caught nonzero Z=1e-9999 underflowing to apparently 2D
(`target/v3000-underflow-red.log`); the parser now refuses nonzero coordinate underflow
rather than flattening it. Final syntax review added a misplaced second main-CTAB
COUNTS vector, which failed (`target/v3000-duplicate-counts-red.log`); the parser now
refuses that known out-of-order control entry while keeping unknown-block internals
opaque. These were substantive failures, not manufactured passes.

All 14 V3000 integration tests pass in the final `target/v3000-io-regressions.log`.
The earlier focused pass is in `target/v3000-final-tests.log`. Independent
RDKit 2026.09.2 reads the original sparse-ID graph, external-generated 314-byte isotope/
eight-decimal vector, a real 1,000-atom file and canonical 1,000-atom output, and all 118
element symbols. Source bytes/ranges, doubled quotes, counted list lengths, explicit
H/VAL0 versus −1, charge, isotope and raw RAD 1 versus 3 have independent literal checks.
RDKit's spin collapse, MASS 13.5 truncation and ignored-legacy-count disagreement remain
diagnostics. Query, stereo, maps, 3D, unknown properties/blocks and invalid syntax are
refused before unsupported model regeneration. Hostile tests cover all truncated
prefixes, encoding/nonfinite/underflow/overflow, counts/references/order, physical/
logical/segment/token/list/depth/byte limits, a million physical lines and the separate
whole-file decoded-token budget. Inputs remain unchanged across errors.

The writer's precision/tolerance policy was selected before tests: shortest-roundtrip
f64 spelling, no V2000 four-decimal quantization, and frame reconstruction within
`16*EPSILON*max(1,abs(point),abs(origin),abs(raw*scale))`. A non-four-decimal literal and
an externally read long finite coordinate verify precision and physical 80-character
continuation. A smallest-subnormal transform refuses exported infinity before output.

Frozen production source SHA-256, concatenating v3000/mod.rs,parse.rs,tokens.rs,write.rs
in that order: `38bb313977f8ea002b275c02490ecd4266284c4e497ba7cc2ccc8363adb24897`.
Targeted IO all-target Clippy and rustfmt pass (`target/v3000-clippy.log`,
`target/v3000-fmt.log`). No new runtime dependency or lock changes were needed.
The isolated new fuzz target also passes warnings-denied Clippy and its workspace
format check (`target/v3000-fuzz-clippy.log`, `target/v3000-fuzz-fmt.log`).
The complete IO regression suite passes 96 tests with the pinned external interpreter
(`target/v3000-io-regressions.log`), including the existing V2000/SDF/source/atomic-file
cases. This is the isolated IO package, not a workspace aggregate or native app run.
The final fresh-corpus ASan run passes 10,000 iterations, seed 937155923, max_len 65536,
timeout 5: coverage 1254/features 2799, 169 retained inputs/reported 45 KiB/RSS 74 MiB,
largest input492 bytes, 1 second fuzz runtime (`target/v3000-asan-final.log`). Both
original fixtures seed the writable ignored corpus. Earlier provisional/pre-COUNTS
passes are not substituted for this frozen-source result. See fuzz/README.md for
reproduction.

These are standalone codec tests on macOS arm64; no V3000 application persistence,
registry/native workflow, SDF version dispatch, fetched corpus, chemistry perception,
stereo/CIP implementation or hosted execution is claimed. Root integration/aggregate
verification and a public commit remain separate steps.


Root integration review then ran the full actual-worktree `cargo xtask ci`, explicitly
setting this sibling checkout's own Cargo target and the root cached external interpreter.
It passed 338 workspace tests plus three native bridge tests, fmt/clippy/layers/native
licenses/assets and this branch's inherited two-file CDXML/MOL corpus. The tool printed
this checkout as its root; no main-checkout build artifact or later SDF corpus pass was
substituted. Log: target/v3000-worktree-ci.log. No runtime dependency or lock changed.
Hosted checks and integration with newer corpus changes remained subsequent work at that
checkpoint. Published source5caf6ce subsequently passed43519338382 as recorded above.
The root now integrates that unchanged codec with98bc59a's fetched SDF/CDX corpus. Both
verification histories, CDX positive seed/bound and the fifth V3000 sanitizer target survive
the merge. The combined source requires its own aggregate; no isolated result is substituted.

## Save ordering and current close receipts

Independent review compiled an isolated client against the V3000 application source and
reproduced a data-loss risk: publish older V2 without finish, publish/apply newer V3,
then finish older V2. The clean association becomes V2 over V3 disk; the next Auto save
rounds raw x1.1666666666666667 to1.1667. Root target/v3000-integration-review.md and its
sibling target/v3000-stale-save-review probe retain exact source and observations. The
generic ROOT regression first published real CDXML accepted by independent Expat/RDKit,
then failed because a second same-path MOL Save was accepted before the receipt applied:
target/save-serialization-red.log. A separate actual V2/V3 regression also fails in the
sibling target/v3000-save-order-red.log; it will turn GREEN after correction integration.

One private per-Engine Arc reservation moves through FilePlan and Completed to finish.
Another Save of any format/path fails busy before I/O; queries, edits, capture/Open and
independent Engines remain available. Six lifecycle tests check exact state/destination
retention, pre-publication cancellation/refusal, both dropped handles, opaque refusal/Undo,
old-generation truthful receipts and cancellation after publication. A separate MCP test
proves distinct executor budgets cannot bypass the Engine's guard. No extra library,
counter, blocking lock or persistent authoring field was introduced.

The review also identified the post-finish/pre-UI-poll gap. Actual revised UI test
target/close-post-finish-aba-ui-red.log executes exactly one test and fails: a historically
true receipt renews Approved after actual MOL→CDXML same-path/revision/clean-state ABA.
The initial overly narrow filter ran zero tests and is explicitly not RED evidence. A
private matching-completion bit is now set after the operation's own baseline update and
cleared permanently by every subsequent real context change. UI poll must consume its
current acknowledgment before renewing close; existing receipt/path/format/revision/epoch/
clean checks remain. Uncompleted/dropped work cannot acknowledge successful publication
(target/save-ui-ack-abandoned-red.log). Returned wire receipt fields remain historical.

GREEN: command package 76 tests, MCP worker 8, UI library 36, scoped clippy/fmt, exact ABA 1/1.
The full actual-root cargo xtask ci passes 356 workspace plus three bridge tests and all
gates: target/save-completion-ci-integrated.log. Existing older-baseline/ABA tests use
baseline-first Undo→prepare→Redo schedules; every stale-ID, whole-state, dirty, history,
unsaved-Open and approval assertion is retained. Former overlap is either explicitly busy
or replaced by a valid schedule. Never lower coverage to preserve an obsolete race.
The earlier merged standalone-V3000/corpus checkout separately passed 349+3 tests in
target/v3000-corpus-merged-ci.log and all hosted jobs43526538340. Native interaction for the
corrected save/close feature and its own hosted run remain pending. No parser/IO publication,
runtime dependency/lock, renderer, font or approved golden changed.

## New standalone CDX tree writer, 2026-11-14

Actual RED target/cdx-writer-red.log fails E0432 for missing Writer. First GREEN creates
the entire original65750-byte explicit-root scene, including65535-byte opaque property,
and matches its independently generated committed bytes exactly. Unchanged external
Python struct/RDKit verify guard confirms the real chemical scene. It is not an own-writer/
own-reader-only test. Original fixture, oracle and golden remain unchanged.

All20 CDX tests pass, including five new construction tests: literal empty/header/ID0 bytes,
invalid tags/balance and rollback,65534/65535 prefixes, exact256-depth/100000-record bounds
including reserved end records, and inclusive64MiB byte budget with rejected one-byte excess.
Package clippy/fmt pass. The expanded existing CDX fuzz harness checks source/property edits
first, then constructs a new explicit-root tree and compares non-root tags/IDs/payload/nesting.
Root/header/length canonicalization is new syntax evidence, never an imported-byte-loss claim.

Fresh two-seed ASAN passes10000 runs, seed721533879/max_len131072/timeout5, coverage305/
features878,59 retained inputs/reported150KiB/RSS199MiB, largest input65750bytes. Log
target/cdx-writer-asan.log. Stable pinned clippy/fmt also pass for the isolated fuzz package.
The fuzz nightly has neither fmt nor clippy installed; those attempted commands failed as
tooling-only checks, then the already installed pinned stable tools succeeded. No tool
installation, lock or dependency change was used to hide a product failure.

The combined actual-root working tree, including the following V3000 fetched-corpus slice,
passes367 workspace plus3 bridge tests and all cargo xtask ci gates. Log
target/cdx-writer-v3000-corpus-ci.log. Runtime closures153/164/219/two assets are unchanged.
The writer has no application model/file/UI integration; no native CDX or full M1 claim.

## Fetched original V3000 source, 2026-11-14

Actual RED target/v3000-corpus-format-red.log rejects the new mol_v3000 discriminator.
The new mandatory manifest pins actual public5caf6ce/344bytes, not a future source or
local substitution. Cold bounded HTTPS fetch passes and normal/offline reports are
byte-identical: five files/four source bundles, V2000count1/V3000-originalcount1. Source
and external-oracle SHA1867a590…4c4380 agree; original source IDs10/40/bond7 and dense model
IDs1/2/bond1, H3, charge/isotope/radical/raw attributes and coordinates match separate
literal/Decimal/RDKit checks. Changed raw/model fields, coordinates, version, truncation,
query extension, different valid vector, cache corruption/missing data and manifest/global
bounds all refuse. Existing three manifests/notices/fixtures/oracles stay byte-identical.

All34 xtask tests, all-target clippy/fmt, actual cold/offline checks and the actual-root
combined367workspace+3bridge cargo xtask ci pass. Logs target/v3000-corpus-handoff.md,
target/v3000-corpus-{format-red,cold-fetch,tests-final,offline}.log and
 target/cdx-writer-v3000-corpus-ci.log identify the scoped/combined results. An isolated
core.autocrlf checkout first changed885bytes to902; its own narrow -text line restored exact
identity and all four manifests. No hash normalization, assertion/guard weakening, dependency,
lock, parser, fixture, oracle, runtime or golden change. Foreign/general/application coverage
and chemistry remain separate missing work. This source needs its own hosted run after commit.

## Version-aware MOL application slice

The V2000/V3000 shared-file integration is recorded in
[the application report](verification-v3000-application.md). The earlier364+3 gate and
partial native acceptance there are historical stages. The subsequent save reservation and
trusted current-completion acknowledgment correct the independently reproduced races;
source1c53312 passes corrected372+3 local, hosted43531603511 on all three platforms and
actual Mac author/save/reopen/refusal/dirty-close workflows. Independent raw-coordinate,
source/header retention, atomic-stage and CLI/stdio/loopback/UI evidence remain separate.

## Combined M1 application integration, 2026-11-14

The merge of accepted application source1c53312/docs2a69de2 into7596c77 retains the bounded
CDX writer and five-file/four-bundle corpus. A direct diff of engine/UI/MCP/CLI and application
IO source against feat/mol-v3000-files is empty. No runtime source, lock, font or golden
changes were needed during conflict resolution. The actual root aggregate passes394 workspace
plus3 macOS bridge tests, rustfmt, all-target clippy, layers, native declared licenses/assets
and every delivered corpus check. Log target/m1-application-merged-ci.log ends with
local CI checks passed. Session7483 is terminal. This is combined local evidence; the new
merge still needs its own hosted run. Accepted native application observations are tied to
source1c53312, not a claim of a second native run or interactive Windows/Linux acceptance.

## SDF mixed molecular versions (2026-11-14)

Executed only in the persistent bondline-mol-v3000 checkout on feat/sdf-v3000 from
accepted 2a69de2; no native/reference UI or hosted/aggregate result is claimed for this
slice. Every Cargo command used that checkout's absolute CARGO_TARGET_DIR with
CARGO_INCREMENTAL=0 and development/test debug 0. BONDLINE_PYTHON selected the existing
isolated external RDKit 2026.09.2 interpreter. Original mixed source was assembled
independently before the generic writer and has777 bytes/SHA
165e6f5d5cff37dae6315a07843d84733aa42dbbc497b2920da1702bad9526a0.

Actual missing-API RED: target/sdf-v3000-red.log (sdf::write/mol_version absent).
Existing-API semantic RED: target/sdf-v3000-parse-red.log (primary-permitted mixed input
refused before implementation). Minimal generic dispatch then passed. One new negative
test initially assumed an empty-count V2000 document with opaque M V30 properties was
invalid; target/sdf-v3000-invalid-vector-diagnostic.log identifies that mistaken case.
The corrected negative uses actual V2000 atom rows falsely stamped V3000, requiring full
V3000 syntax validation. A positive test separately requires retaining unknown property
bytes under their actual validated V2000 stamp. No production or old assertion was
weakened for that test-authoring correction.

All 12 unchanged SDF tests plus 7 new tests pass in target/sdf-v3000-oracle-green.log.
Literal expectations check all mixed record/MOL/item ranges and ordered duplicate/DT/
empty/multiline/spaces data. Tests retain unknown/continued/quoted/query/stereo MOL bytes,
CRLF/empty titles/missing-final-LF framing, require legacy-writer refusal, and reject
malformed later records, bad references/counts/stamps, padding/delimiter and molecular
resource failures. The generic V2000 output equals explicit V2000 output. Existing SDF
boundaries and DT-only supplier-loss assertions remain intact.

Actual external SDWriter switching produces 667 bytes/SHA
88516961b6a197950ac83e09c441387bd2e9ed1c2da2d94accdffd1b9c359ca6;
full ordinary supplier checks pass for both output molecules, EMPTY and multiline data.
Unchanged V3000 oracle inputs verify exact8 dp coordinates/isotope and1,000 atoms through
new SDF framing and external consumers, with no conversion/rounding. Existing oracle
SHA-256 values stay 728a7f5d3adbbcb5097a357a147ae817a461004c0246796f5f39d848969278fb
(SDF) and e351585c7c15aec4a9c31d5bd6472baace6b5628988fe44fbc7520e216fee697 (V3000).
The full-supplier DT-only loss remains separately asserted, not filtered or relabeled.

Final scoped `cargo test --locked -p bondline-io` passes 114 tests, zero failures,
plus zero doctests (target/sdf-v3000-io-green.log). IO all-target warnings-denied
Clippy passes (target/sdf-v3000-clippy.log), as do workspace/fuzz rustfmt checks
(target/sdf-v3000-fmt.log and sdf-v3000-fuzz-fmt.log). The existing sdf_parse target's
warnings-denied isolated Clippy passes (target/sdf-v3000-fuzz-clippy.log). Root and
isolated fuzz lock hashes and both existing oracle hashes remain unchanged.

Actual AddressSanitizer completed 10,000 runs, seed 826403517, max_len 65536, timeout 5,
exit 0 (target/sdf-v3000-asan.log), using cargo-fuzz 0.13.2/libFuzzer 0.4.13 and
nightly-2026-04-01 in this checkout's own target. Six fresh ignored seeds include both
original SDF fixtures, quoted/continued V3000, external mixed output, 8 dp and a
49,063-byte 1,000-atom SDfile. Final coverage 1605/features 6242, 341 corpus entries/
reported 3066 KiB, maximum exercised input 49,063 bytes, reported RSS 462 MiB;
3 seconds of fuzzing after compilation. Exact source/spans/versions/ordered items and
generic/strict writer invariants are checked, not chemistry for arbitrary mutations.
This is bounded smoke evidence, not exhaustive conformance, an RSS bound or a security
audit. No committed fixture was a writable corpus. All local handles were terminal
before handoff; no native/reference/hosted/aggregate run was performed for this slice.

## Reviewed mixed-SDF aggregate, 2026-11-14

The actual feat/sdf-v3000 checkout passes379 workspace plus3 macOS bridge tests and every
cargo xtask ci gate, with its invoked root printed first. Log target/sdf-v3000-full-ci.log
SHA77da79e40b19347c86782478f29d749e9e0a339a7e4ea5a23b678ed3bf67306d; session29645 terminal0.
The inherited two-entry corpus is explicitly narrower than root's later five-file corpus.
The bounded implementation passed independent read-only review; current capability docs
now distinguish generic mixed writing from strict write_v2000. Production, old tests/oracles,
locks/dependencies/fixtures/golden are unchanged by that finalization. Hosted and receiving-
root integration checks remain separate. No native application collection claim follows.

## Receiving-root mixed-SDF integration, 2026-11-14

The actual merge of6aeb9f2 into85bf5eb passes401 workspace plus3 macOS bridge tests and
all cargo xtask ci gates. Log target/sdf-mixed-merged-ci.log ends with local CI checks passed;
session24244 terminal0. Direct comparison of the SDF container, moved stamp helper and
application document caller to accepted6aeb9f2 is empty. Root retains its five-file/four-bundle
corpus, all old pins/oracles, CDX writer/positive seed/max_len131072, save/close barriers and
V3000 target. New mixed fixture whitespace attributes preserve spaces-only values. Runtime
closures153/164/219, assets/locks/font/golden are unchanged. The new combined source still
needs its own hosted result; no native application SDF workflow is implied.

## Standalone legacy RXN codec (2026-11-14)

Executed in the isolated `bondline-rxn` worktree from checkpoint 3b1fada, Rust 1.94.1,
macOS arm64. Every Cargo invocation used this checkout's absolute `CARGO_TARGET_DIR`,
`CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`; independent
consumers used `BONDLINE_PYTHON` pointing to the existing isolated RDKit 2026.09.2
interpreter. No native application, GPU, hosted job or root aggregate claim is made.

The actual missing-API RED is `target/rxn-red.log` (unresolved `bondline_io::rxn`).
An earlier ENOSPC during dependency compilation is retained in `target/rxn-disk-full.log`
and is not TDD evidence. `target/rxn-byte-limit-red.log` caught an actual three-byte
fixed-wrapper overhead undercount that allowed a 64 MiB + 1 output. Numeric-present
count assertions also failed before the unsupported inferred blank-zero behavior was
removed (`target/rxn-blank-count-red.log`). The published RXN count table and actual
consumer rejection support numeric present fields; the exact six-byte omitted-third
consumer extension remains explicit.

`cargo test --locked -p bondline-io --test rxn` passed all nine tests
(`target/rxn-bounds-green.log`). Independent literal/raw checks cover header/counts,
full/constituent spans, role order, maps, unknown properties, LF/CRLF, marker-like
metadata, exact source retention, mixed actual MOL versions, empty reactions, strict
EOF, malformed/truncated/control/non-ASCII input and each declared resource budget.
Separately run RDKit consumes the original reaction and molecular vectors and generates
both the standard separate-agents document and a legacy wrapper with a 1,000-atom V3000
molecule. Default agent folding, permissive trailing input and V3000 role spelling
remain diagnostics rather than fidelity expectations. No reaction chemistry execution
or application-model projection was tested.

`cargo clippy --locked -p bondline-io --all-targets -- -D warnings` passed
(`target/rxn-clippy.log`). The final source hash, concatenating rxn/mod.rs, parse.rs,
write.rs in that order, is
`5144c6dabd97d048307cc954ad0459e958ba87aafb977119be3072b430477a5e`.
Original fixture hashes are pinned in their README and tests; exact-byte attributes
protect checkout line endings. Root and isolated fuzz lockfiles remain unchanged.

The sixth parser harness completed an actual AddressSanitizer run using cargo-fuzz
0.13.2, libFuzzer 0.4.13 and nightly-2026-04-01: `rxn_parse`, 10,000 runs,
seed 224867531, max_len 65536, timeout 5; exit 0 in `target/rxn-asan.log`.
Seeds were ignored copies of both original fixtures and separately generated 44,023-byte
mixed-MOL output. Final coverage 1305/features 4915, 290 retained inputs/reported
3549 KiB corpus, largest exercised input 44,023 bytes, reported RSS 464 MiB; execution
3 seconds after compilation. This bounded run checks exact bytes, spans, role/payload
versions and canonical wrapper invariants. It is not exhaustive conformance, a hard
memory-cap proof, independent chemistry for every mutation or a security audit.

The final scoped `cargo test --locked -p bondline-io` regression passed 116 tests,
zero failures, plus zero doctests (`target/rxn-io-regression.log`), including the nine
new RXN tests and existing CDXML/MOL/SDF/source/atomic-file tests. Workspace and isolated
fuzz rustfmt checks passed (`target/rxn-fmt.log`, `target/rxn-fuzz-fmt.log`).
`cargo clippy --manifest-path fuzz/Cargo.toml --locked --bin rxn_parse -- -D warnings`
also passed (`target/rxn-fuzz-clippy.log`). Ruby/Psych parsed the configured six parser
sanitizer steps and verified each has an executable run/uses entry; no hosted execution
of the new RXN step is claimed. All local handles were terminal before handoff.

## Reviewed RXN aggregate, 2026-11-14

The actual feat/rxn-v2000 checkout passes373 workspace plus3 macOS bridge tests and every
cargo xtask ci gate. Its printed invoked root is bondline-rxn. Log target/rxn-full-ci.log
SHA1b79e3116cc8e49f625e46d8b4cab6564cde40f5d3ee6a1f925f917b14bf72b4; session56720 terminal0.
The inherited two-entry corpus is narrower than root's later five-file/four-bundle gate.
Read-only independent review found no confirmed declared-profile codec defect; the summary
now names standalone RXN separately from missing SMILES/application reactions. No production,
old oracle/fixture, lock/dependency or golden changes were needed for finalization. Hosted
and receiving-root aggregate checks remain separate. No native reaction workflow is claimed.

## Receiving-root RXN and SDF integration, 2026-11-14

The actual receiving merge of4cbc89b into23a43aa passes410 workspace plus3 macOS bridge
tests and every cargo xtask ci gate. Log target/rxn-sdf-merged-ci.log ends with local CI
checks passed; session11864 terminal0. RXN source, tests/oracle/fixtures and fuzz harness
match the independently reviewed source exactly. Root retains all newer SDF/V3000/CDX
implementation, five-file/four-bundle pins, meaningful-whitespace attributes and positive
CDX seed/max_len131072. The workflow now has six bounded parser targets; the new target's
local ASAN evidence is separate from future hosted execution. Runtime closures153/164/219,
assets/locks/font/approved golden stay unchanged. No application reaction or SDF workflow
is inferred. This combined source still needs its own hosted platform result.

## Installed-reference map integrity, 2026-11-14

The first native observation/map/ticket/catalog docs were reviewed with their exact frozen
SHA identities, UTF-8 and relative links. The503 physical menu IDs are unique, every parent
exists and blank-state flags count301 enabled/202 disabled.28 formats/12 local fields/9
Settings categories are separate;178 same-role/seven candidate mappings are not passes.
All recorded artifact hashes were validated by the investigator/mapper;17 REA Evidence and
three failed probes remain ignored. Complete AX coverage failed on retained sheet gaps and
that failed check remains explicit; canonical image/source/integrity checks are separate.
No product source changed for these documentation checks; no artificial Rust RED is claimed.
The original local XML syntax pass, current CDX TrailingData, external RDKit results and
original-window restoration exception are distinguished in the source register/reports.

## Header-embedded CDX Document framing, 2026-11-14

Work began in the separate bondline-cdx-framing checkout from85bf5eb. The first
original65,746-byte fixture passed its independent generator and actual pinned
RDKit2026.09.2 checks, then Document::parse returned TrailingData. Actual intended
RED: target/cdx-header-red.log, one failed test. No parser code was changed before
that failure. Public BSD source and SDK facts are cited as
CDX-HEADER-DOCUMENT-IO/END in clean-room/sources.md; the native input was not copied
into the original fixture.

The reader now has a bounded HeaderDocument profile:22-byte prefix, encoded root,
root ObjectEnd and separate FileEnd. Successful legacy28-byte parses retain their
interpretation. Only TrailingData with the exact supported prefix/8000-at22 marker
enables the alternate profile. This does not trim arbitrary padding. Ambiguous
missing-file-end input can still be complete legacy syntax; the tests state that
limit explicitly. Writer output and both old fixture/oracle identities are unchanged.

All27 package tests pass, including seven new framing cases: exact30-event table,
external CC/C2H6→C=C/C2H4 property edit, all-property no-ops/foreign handles,
opaque short/extended replacements with reserved bytes and unrelated records exact,
legacy reserved-header ambiguity, all source prefixes and strict trailing-data
refusal, and100000-record/256-open-object boundaries counting root/FileEnd. The
same package's all-target clippy with -D warnings and workspace rustfmt check pass.
Commands used a distinct absolute CARGO_TARGET_DIR, CARGO_INCREMENTAL=0 and
DEV/TEST_DEBUG=0; BONDLINE_PYTHON selected the existing pinned external interpreter.
Logs: target/cdx-header-{first-green,tests,clippy,fmt}.log.

The unmodified ignored local native1196-byte research file, SHA
7b2b71338595ad12bc34f46459f8389ec6a63cb453b1cf29b51543717139a73c,
now parses as HeaderDocument. All99 events match independent stdlib decoding,
including root22..28, root end1194 and file end1196. The record-table SHA is
203749413c964b233172bf290ab7364ced97113008b12317d075fabae4b5bdc7.
Exact source passthrough/no-op and one independently constructed opaque-property
splice pass; that changed property is syntax-only, not semantic editing evidence.
Direct pinned RDKit reads the unchanged file as two carbons/one single bond,
CC/C2H6. The vendor metadata and changed research output remain ignored, and the
source hash is unchanged. Evidence: target/cdx-native-passthrough-evidence.json.

Fresh ASAN passes10000 runs with three original seeds, seed721533879,
max_len131072/timeout5, coverage319/features841,53 retained inputs/reported303KiB,
RSS283MiB. The executable links the nightly AddressSanitizer runtime. Disk-constrained
local execution used the approved ignored minimal manifest and exact pinned lock
subset, compiling the unchanged committed cdx_parse harness without the unrelated
IO/GPU graph. Log target/cdx-header-asan.log. Scoped stable fuzz-harness clippy and
normal fuzz-workspace rustfmt checks pass in cdx-header-fuzz-{clippy,fmt}.log.
This is not a normal full-graph fuzz build, aggregate gate, hosted platform result
or exhaustive safety proof. No GUI was launched for this standalone parser change.
CDX model mapping/application Open/Save and broader foreign/general coverage remain
missing; the fetched original corpus pins/counts are unchanged by this new fixture.
The CI seed step now includes that third original vector; the existing two seeds,
sanitizer command and max_len131072 remain unchanged. This configuration change
has not yet run on hosted CI.

## Receiving-root header-embedded CDX gate, 2026-11-14

The actual merge ofe9bffd3 into291c3a6/c351bb8 passes417 workspace plus3 macOS bridge tests
and every cargo xtask ci gate. Log target/cdx-header-merged-ci.log ends with local CI checks
passed; session39909 terminal0. Source/tests/oracle/corpus formatter/fuzz target match the
reviewed feature exactly. Later mixed SDF/RXN, six sanitizer targets, all four manifest
attributes/pins and approved golden remain. Native closures153/164/219/two assets unchanged.

Normal full fuzz-workspace clippy passes for cdx_parse with warnings denied, and all-target
fuzz rustfmt passes. A fresh three-original-seed corpus (70/65750/65746bytes) runs the exact
normal manifest/harness on pinnednightly2026-04-01/cargo-fuzz0.13.2/libfuzzer0.4.13, ASAN10000,
seed721533879/max_len131072/timeout5. Session6194 terminal0, coverage319/features841,53inputs/
reported303KiB/RSS283MiB, maximum exercised65750bytes, zero seconds after compilation. Log
 target/cdx-header-normal-fuzz-asan.log SHA6cdcc0e47428f465e0c44e871a1da6814a32bee9207b7947c9da5010a22ae9f8;
clippy target/cdx-header-normal-fuzz-clippy.log. Root/fuzz locks remain byte-identical. This
is additional normal-graph evidence, separate from the worker's minimal-manifest run.
No native GUI changes or new screenshots/goldens are needed for this syntax-only slice.
The reviewed native1196B exact-source/property probe uses this same std-only parser source;
no raw vendor metadata is committed. Application CDX/model/general foreign coverage remain
missing, and the new integrated source still needs its own hosted platform result.

## Main palette map documentation checks, 2026-11-14

Both frozen report/catalog hashes, UTF-8, relative links and36 unique orderedIT-001–IT-036
IDs pass. The investigator validates exact button/command/source references,23 toggle/13
subtoolbar declarations, five successful canonical REA records, one preserved unsupported
NIB probe, PNG identities and before/after cleanup. XML fallback and inferred orientation
remain separate from runtime; all tooltip/AX/action/enabled/hit-bound fields are null.
No Rust/product source or asset changed; this is relevant documentation validation, not
a fabricated RED or a working36-tool claim.

## Owned CDX integration, 2026-11-14

The source-backed elemental adapter and owned snapshot/export/refusal/accounting slice
passes438workspace+3macOSbridge and the actual full aggregate. Its first RED,7 snapshot
tests/144IO tests, independent reviews, internal-only lock edge/154-165-220 license closure
and normal full-graph ASAN10000 are recorded in
[verification-cdx-owned-snapshots.md](verification-cdx-owned-snapshots.md). No registry/UI/
file-worker behavior, old fixture/pin/font/golden or source-export proof was weakened.
The new source still needs matching hosted execution; no native CDX app workflow is claimed.

## CDX filesystem library, 2026-11-14

The shared native adapter now reads accepted CDX snapshots and atomically saves unchanged
complete backing. Actual missing-API/dispatch REDs precede149 IO and443workspace+3bridge
GREEN. Both original framing variants' actual saved bytes face their unchanged independent
struct/RDKit consumers. Four filesystem cases and seven injected stages verify refusal,
old-data retention, cleanup and truthful postpublication errors. All aggregate gates pass;
see [verification-cdx-native-io.md](verification-cdx-native-io.md). No command/native chooser,
new encoder, license/runtime dependency, fixture/pin or approved golden is added.

## Windows terminated-socket assertion, 2026-11-14

Actual hosted43549658341/Windows132308919222 fails only excess_connections in its15-case
control binary: a terminated read returns10053/ConnectionAborted, excluded by the test.
The observed failure remains in ci.md; no failed run is relabeled GREEN. Published Microsoft/
Rust semantics justify admitting that closed outcome in all three closed-read assertions.
The same16 authenticated held clients, positive-byte/live-read/timeouts rejection and
state/authentication checks remain. No production networking or timeout code changes.
Local control tests pass15, target/socket-closed-windows-kind-green.log, session61439 exit0.
The new receiving-root full gate passes443workspace+3bridge, target/socket-closed-kind-ci.log,
session98879 exit0, with the unchanged154/165/220 native closures/corpus/assets. Its Windows
execution remains pending a new matching source run; this local pass does not prove it.

## Carbon graph receiving integration, 2026-11-14

One bounded producer now serves supported positioned neutral-carbon/Single graphs, literal
authored H, CPU/GPU capture and typed atom/bond hit regions. Source completeness/export
guards remain strict. Literal independent geometry/pixel/FreeType cases, graph/error/limit
tests and the unchanged approved golden pass. Native baseline stale hover observations
produced7 additional runtime REDs; the sticky freshness fix passes61 UI tests. Receiving
root aggregate passes467workspace+3bridge with all gates. Exact handoffs/hashes, the earlier
failed observations and pending corrected native/hosted acceptance are in
[verification-carbon-graph.md](verification-carbon-graph.md). This is not selection,
connected drawing, other-atom/stereo/style rendering or completed M2-02.

Final source adds the actual7 ordering/press-position RED regressions and their minimal
shared eligibility/foreground fix,70 UI tests and476workspace+3bridge full GREEN. Actual
ROOT binary2963b444…1629 now passes visible C–C/UndoRedo/Unicode save/reopen, independent
actual-byte consumers and bounded1x/2x hover/reopen/Escape-held/focus/context/chooser cases.
Every owned app/controller/helper exited; Escape released/CUA completed once after cleanup.
Exact full hashes,136-file evidence index and rejected probes are in the linked report.
No new golden, arbitrary chemical rendering, same-frame native timing or Windows/Linux
interactive claim follows. Its new hosted source run remains pending.

## Source-only CDX metadata receiving integration, 2026-11-14

Selected Z/bounds/U0-N1 CIP/opaque AtomID/unstyled-warning fields now face strict source-
only type/owner tests and original323/319B stdlib/RDKit vectors. Existing snapshot/export/
source/identity guards remain. Receiving486workspace+3bridge passes all gates; normal full
fuzz-graph clippy and six-original-seed ASAN10000 pass, cov1131/features2728/RSS255MiB.
Exact independent review, logs/hashes/settings and the unchanged ignored1196B native
diagnostic are in [verification-cdx-native-metadata.md](verification-cdx-native-metadata.md).
The native file/prose is not committed; chemistry/encoder/app coverage is not inferred.

## External font-oracle portability, 2026-11-14

Actual Mac/Windows hosted failures at983ffee reproduce locally with BASIC whole-pixel
advances. Independent OpenType design metrics restore layout without changing production,
font/golden bytes, the0.90 threshold or negative cases. Five focused label tests and31
whole-render tests pass. Receiving-root `cargo xtask ci` passes487workspace+3bridge,
session31715 exit0, log SHA1b2d0e0e1af11134bd977bf71cddd3184c57ba4f32690d7f0780f4949260d0ee.
[The exact source/probe record](verification-label-oracle-layout.md) distinguishes the
reproduced mechanism from unattested hosted build flags. New hosted verification is pending.

## Guarded CDX application integration, 2026-11-14

The15-command registry now carries supported CDX Open/unchanged Save through CLI/stdio/
authenticated loopback and native file workers. Four original fixture variants' actual saved
bytes face independent raw/RDKit consumers. Source/dirty/history/reservation/receipt/close
guards and exact-format retries remain. Receiving Root passes499workspace+3bridge,502 total,
session95800 exit0, target/cdx-application-integrated-ci.log SHA
7c5a36e4b5dcaa5ec9db510056876f9e5e92581aaf9a611d24522aa6e264c99e. Runtime graph/input/
capture/golden are unchanged. [Application evidence](verification-cdx-application.md)
keeps actual native chooser acceptance and hosted execution explicitly pending.


## Terminal-carbon engine integration, 2026-11-14

Required-policy bond.extend, atomic validated fragment replacement and global revision/
ID/source/history/close guards pass21 new independent/property/headless tests. Actual
published propane/butane CDXML faces mandatory Expat/Decimal/RDKit consumers, not only
our own round trip. In isolated bondline-rxn, the16-command merge with CDX application
source passes520workspace+3bridge,523 total,54630 exit0; log SHA
 dfb35244cdbdf58d69d40a9fa98d89d278054930292a2f2e561290d00b274e8d.
[Exact evidence](verification-terminal-carbon-extension.md) records the original REDs,
source/ID limits and paper-expectation test correction. Native attachment and this source's
hosted run remain pending; no chemistry or complete connected-drawing claim follows.


## Accepted native CDX and hosted engine checkpoints, 2026-11-14

Frozen source450962c/binarye9491a72…5749 passes actual bounded Mac chooser/Open/Known Save/
Unicode Save As/reopen/retry/dirty-close refusal/Cancel/Undo and exact saved-byte consumers.
Four original vectors and one ignored1196B native own-drawing input retain every byte.
Root independently rechecked379 source hashes and83 finalized evidence files plus binary/
cleanup before changing checkout. [The exact report](verification-cdx-application.md)
qualifies unavailable preview/editing, excluded probes and nearest-integer cursor restore.
All owned app/controller/helper handles are terminal; CUA completed once after cleanup.
Its own hosted43559974203 passes502/500/492 tests/windows and six ASAN targets.

The separate16-command terminal engine source501a315 passes hosted43560603081:
523/521/513 tests, all three actual windows, six ASAN targets. Read-only peer review finds
no confirmed defect; its scope/report hash are in verification-terminal-carbon-extension.md.
Native attachment is being implemented separately; neither result verifies that gesture.
The documentation-only acceptance update changes no production, fixture or configuration.


## Terminal-carbon attachment receiving integration, 2026-11-14

Fresh per-event target/anchor/release/context routing uses the same required-policy
bond.extend registry. Actual5 initial REDs and2 inherited InvalidViewport cancellation
REDs precede minimal fixes. All73 prior UI cases and17 new cases pass; actual workspace
edit/save output faces independent Expat/Decimal/RDKit. One reviewed error branch now
clears held gesture state. No golden, model, encoder, command or dependency change.
Receiving Root537workspace+3bridge=540 passes all gates,57320 exit0, log SHA
 e140e7f400d948944f7157db40c3cea44993a7341d5df6573fd6a5df95b67e0e.
[Exact source/review/limits](verification-bond-attachment-ui.md) retain both failure
checkpoints. New-source native acceptance and hosted checks remain pending.


## Immutable whole-SDF source receiving integration, 2026-11-14

Nine new independent literal/consumer/boundary/identity/accounting tests preserve both
original690/777B vectors, every record/field/version and source-only differences. Existing
parser/writer bodies/fixtures/oracles/pins are unchanged. Root94645 passes546workspace+
3bridge=549; log SHAfe1e23fee5df7a1310243fd9e1b034ad6a25a898cf8aad4b073e591c50c987b8.
Normal full-graph fuzz clippy and10000-run ASAN24955 pass with original copied seeds;
exact hashes/settings and independent review are in verification-sdf-owned-source.md.
No Engine/native SDF or data-editing claim follows. Attachment sourcef0910ca separately
passes hosted checks after failed-only Linux infrastructure retry, but native extension
was not attempted due unowned input; the linked UI verification records that partial pass.


## Immutable SDF field replacement receiving integration, 2026-11-14

Exact second-DUP and DT/record2 edits, CRLF/no-op sharing,1m-line EOF-aware quotas and
malformed/missing target cases preserve every unrelated byte. Actual edited files face
independent raw scans/splice literals and per-MOL RDKit; old consumers remain unchanged.
Root71692 passes556workspace+3bridge=559, logSHA
6adf07911f3563a00c6b2a80763e2b4a6c67ff75b94ce0c50da913352551c267. Normal SDF fuzz clippy/fmt
and ASAN10000 pass; exact logs/settings and qualified read-only review are in
verification-sdf-field-value.md. No native/session/command integration is implied.

## SMILES notation syntax/source ownership, 2026-11-14

[verification-smiles.md](verification-smiles.md) records the bounded iterative
nonaromatic library parser, immutable full source/token ranges and original graph
expectations. Missing-API, topology and query-category REDs precede their fixes;
the exact final IO gate passes186 tests, warnings-denied all-target IO clippy and
workspace formatting. Whole accepted LF/CRLF source faces the pinned external parser
with chemical inference/H removal/name/CX parsing disabled. Actual consumer binding
and dialect failures remain recorded, including unavailable skipCleanup and numeric
ring/dot-branch/HH/closure differences. No old consumer or assertion was weakened.
Peer review, Root full aggregation/fuzz/hosted gates and model/application work remain
separate. No native, generated writer or chemical-perception success is implied.


## Native whole-SDF files and short-chain drawing, 2026-11-14

[Exact native evidence](verification-native-sdf-and-extension.md) identifies sourcebb6d5b9,
396 input files, the actual18235928-byte executable and all101 rehashed artifacts. Native
C3/C4 literal-H authoring/history/Unicode CDXML reopen and mixed777-byte SDF Open/Save/
Unicode.sd Save As/refusals retain independent actual-byte graph/raw/per-MOL evidence.
Input stopped on observed foreign foreground ownership;690-byte/additional refusal checks
are controller-only. All owned native/controller/helpers closed before CUA completion.
Root later restored the exact CLI test-profile cache and verified the original index again.
The observed Open-help omission is fixed by one string change; fmt and warnings-denied
all-target UI clippy pass (56549), but new wording is not natively rechecked. No new golden,
chemical preview, general chemistry, special input/DPI or Windows/Linux workflow claim.

The same source passed hosted43575973213: Mac590 (587workspace+3bridge), Linux588,
Windows580, all three native-window smokes and seven10000-run Linux ASAN targets. Main
advanced tobb6d5b9 only after those terminal checks; earlier8e0e92b results are separate.
