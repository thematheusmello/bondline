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
one less than PDF page indices. Poppler rendered the shortcut table pages; their column
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
