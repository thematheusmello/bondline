# Named-ring command receiving — 2026-11-16

The full M0–M5 goal remains active. This increment extends the existing ring.create
command; it does not deliver named native tools, general aromaticity or a completed
milestone. The user's running older desktop bundle and unsaved drawing stay intact.
Actions remains off; no push, publication or new platform execution.

## Contract and implementation

[The preimplementation contract](research/unsaturated-ring-authoring.md), committed
105b220, records the public GUIDE25/IUPAC/SDK basis and original phase/preservation
policies. Sized3–8 requests and public RingCreate stay unchanged. A typed request union
adds closed named benzene/cyclopentadiene objects with optional non-null string phase.
Registry38 and native controls29/20/14/10 remain. Six typed old callers only gain .into().

One actual builder retains the old geometry, precision, source and install path.
Bounded2/5 pattern rotations fit literal target H once proved from actual old incidence.
Only new atoms receive explicit false flags and new Doubles directed inward sides.
Existing metadata, arrays, shared edge direction/order/style and unrelated Arcs remain.
IDs publish only with the ordinary complete-source commit. Unsupported phases/sources/
metadata/precision/budgets fail atomically; no neighbor correction or resonance rephase.

An independent test also exposed a pre-existing registry mismatch: known command
structs could deserialize positional arrays despite object schemas. The shared parse
boundary now requires an object for a known name, retaining unknown-name precedence.
This is the documented Bondline command contract, not a general JSON-RPC restriction.

## Test-first evidence and qualifications

Logs are disposable under target/unsaturated-rings/root-receiving/. Root owns every
execution and adoption; workers supplied declarations/candidates/read-only review.

| Observation | Actual result |
| --- | --- |
| First declaration71885, existing JSON API | RED0/1: named kind is unknown to the old size-only parser. |
| Object-boundary32415 | RED1/2: empty params array and full sized-ring struct sequence accepted. Shared fix9881 passes3/0. |
| Expanded core66902 | Runtime RED0/7 at old named parsing after the separate test borrow correction. |
| Request JSON/schema runtime | RED0/2: missing named request and schema alternatives. |
| New public Rust API79217 | Missing-export E0432 compile qualification, not runtime behavior evidence. |
| Actual CLI/initialized stdio/authenticated loopback47769 | RED0/3 at old named parsing. |
| First builder23146 | E0282 closure argument qualification; explicitly typing usize→Result<u16,Error> changes no behavior/assertions. |
| Integrated13624 | Parameters3/0; core18/1. New five-file case incorrectly reused the old Single-only sanitized RDKit helper. |
| Corrected new consumer66740 | Parameters3/0, core19/0, public types1/0. |
| Actual wire GREEN |3/0; all three delivered adapters and actual Unicode files. |

The first expanded declaration had E0382 from moving a test Vec inside a two-side loop.
Changing only zip(weights) to zip(weights.iter().copied()) preserves inputs/assertions;
5636/21698 compile failures are not runtime REDs. Original raw source SHA256
1c9db35c9b52512be6a16a79409704d092c9eb9f76ac1fdba3afc83c0d3db437 becomes
c038ca0aee0251dddccea8bf0c5714f73629eac761d8cf7e6bf89c88919444ae before the consumer correction.
Positive schema upgrades keep every old sized-property/required/bounds/placement assertion,
resolving that branch from the new wrapper. The generic schema test adds local-ref/anyOf
closed-branch checks; Result diagnostics replace newly proposed panic diagnostics.

## Independent file consumer scope

The old parent consume_file and old graph/chemistry consumers remain byte-exact. Its
sanitized-only Single/Double assertion is correct for its original cases but unsuitable
for benzene: RDKit legitimately aromatizes integer Kekulé input. The observed failure
does not establish a product or literal-file defect.

Only the new child composition changes. It retains the unchanged external_graph and
compare_graph_positions, then checks raw Expat/Decimal coordinates, exact literal1/2
orders, B/E direction, H, flag presence and DoublePosition. Unsanitized pinned RDKit
must retain the integer graph/H; separately sanitized RDKit must match manually declared
SMILES/formula, H and neutral/natural/nonradical/unassigned-stereo semantics. No raw1.5
allowance or broad sanitized-order relaxation is added.

Before any generated-file use, Root12288 qualifies five original manually authored
CDXML positives and twelve negatives: raw aromatic order, equivalent rephase, reversed
edge, wrong H/side/flag/coordinate/charge/display, dropped bond, wrong chemistry/formula.
All17 match expected acceptance. Original integer-grid controls use no Bondline writer/
parser/model. The checker b7539256ecde6801c692a07a27e9e1deec13f836767db1e10b19d9d107135767
and corrected raw child6172af2710994f9cf239c8e1bd7d33830824aac085c0b0ca56b19b371457dcef
are sealed before adoption; inverse composition recovers the borrow-corrected declaration.
The first2284-byte test, every model/phase/source/schema/history assertion and old parent
consumer remain exact before rustfmt. No product logic changes for this setup repair.

## Reproduction and remaining scope

Use the pinned external interpreter from docs/development.md, without resolving its
virtualenv symlink, Rust1.94.1 and the root Cargo.lock:

```sh
export BONDLINE_PYTHON="$PWD/.cache/oracles/format-python/bin/python"
cargo test -p bondline-commands --test ring_create --test ring_request_types --test parameter_objects --locked
cargo test -p bondline-cli --test sdf_capture_adapters --locked ring_construction_wire::unsaturated_wire::
cargo xtask ci
```

Full local gate18337 exits0:1510 passed/0 failed/1 ignored across214 result lines,
including rustfmt, warnings-denied workspace/vendor clippy/tests, layers/licenses/corpus.
Its186029-byte log has SHA256
18c24e4ae36954323417c2a9a7e48abe2dd9c01ab1272dbf7633475ba2c3e339.
Native/CLI build75233 exits0. Sixteen cases are new: seven core, two JSON/schema,
one public type, three object-boundary and three real adapter cases. Existing legacy
tests remain, including the unchanged MCP control test SHA256
fd96b257cd6c00b52e97544d652ed69fdbaaaa454e1f3a9360d77f3292218fb0.
The root lock, dependency/workflow definitions, old consumers, fixtures and approved
golden remain unchanged; the gate checks155/166/221 declared runtime packages and2 assets.

Independent bounded source review finds no material issue. Its7681-byte report SHA256
4973ddd33b5942610289e0c6f12f3e8f767d66038a8ea604b57290df1cd72103 separately qualifies
the exact final rustfmt deltas and five unchanged legacy helpers. No worker executes
tests or claims runtime/native evidence from source inspection.

Actual built CLI additionally creates both named rings in a fresh original scene,
saves2303-byte CDXML, and captures1024×768 CPU and Metal/Apple M3 Max/IntegratedGpu
frames. Independent Expat checks literal weights/H/flags/sides; RDKit checks C6H6/C5H6
and manual canonical references. Complete RGBA agrees. The actual GPU PNG is visually
inspected:21506B, SHA256d4d1a4ac4d0d53aa1016b97d4ce88de8f90ef01de08316b63210d26ab5cc46a2.
Source SHA256c3dc4ac98fd715bfad2f39504dd65553d931cb9c8812ee300ec7465277a0cc5b;
CLI SHA256cab34aeeb31133e0beff412b347c1e82e7c05fde065e416eb632886cb7bac6de.
Disposable script/source/PNGs/receipt are under target/unsaturated-rings/visual/.
This is actual command authoring/rendering, not native gesture evidence or a human-
approved golden. The existing user executable/draft is never replaced or mutated.

At this command checkpoint, named native controls/preview/Shift were missing. Their
later [native integration](verification-native-unsaturated-rings.md) now has actual-App/
adapter/file checks. Real OS/monitor and Windows/Linux interaction remain unverified.
Existing Single-ring tool behavior stays unchanged.
Automatic fused-face restyling, general aromaticity/valence/stereo, chair/sprout/ring
systems, default styles and broader M0–M5 scope remain required. A passing gate would
not identify or fix the earlier natural MCP replacement-bind AddrInUse failures.
