# Canonical CDX file-end integration

The original external258-byte ethane now parses and retains every byte through owned
IO, shared commands, CLI, stdio MCP, authenticated loopback and bounded Mac Open/Save
As/reopen. Focused checks and **the final full local gate1172/0/1 pass**. An earlier
intermittent MCP shutdown failure remains unexplained and is preserved below.
Faithful CDX display, chemical editing, cross-format export and general compatibility
remain outside this increment. M0/M1–M5 are not complete.

## Contract and source authority

The [source-reviewed plan](research/cdx-file-end-plan.md) preserves every successful
legacy parse first. Only legacy `TrailingData` may select the new28-byte envelope:
the published12-byte prefix plus exactly16 zero reserved bytes, one Document end,
one separate FileEnd and exactEOF. Root8000 at28 selects
`ExplicitDocumentFileEnd`; otherwise `ImplicitDocumentFileEnd` applies. The existing
22-byte HeaderDocument branch retains priority. No scanning, trimming, arbitrary
padding, producer/property heuristic or fallback after resource/truncation errors is added.

FileEnd has parentNone and consumes a record; only encoded roots consume depth.
`Framing::has_encoded_document()` exhaustively distinguishes both implicit profiles
from the three encoded-root profiles. A canonicalized unknown body is supported,
while the original nonzero-header UNKNOWN tails remain refused. Removing the final
word from the258-byte source leaves a valid256-byte legacy document; that inherent
ambiguity is retained, not hidden by a blanket truncation assertion. Writer defaults
and source/property ownership are unchanged.

IO keeps complete source/baseline and the limited projection: two C nodes at(0,0)/
(45,0)pt, one Single bond, authored HNone and `is_fully_mapped=false`. The external
reader's H3,3/formula are test expectations, not newly inferred model chemistry.
Faithful capture/authoring and CDXML/MOL export remain guarded. A model/page edit
cannot save stale CDX; exact Undo restores export. The existing Save lease lasts
through publication/finish, and a late receipt cannot reassociate a new session.

## Test-first and independent checks

- Independent48668 and Root59054 first reproduce actual `TrailingData` on the
  untouched fixture. The frozen first1,667-byte test precedes implementation.
- The expanded declaration initially has six missing-variant E0599 errors and an
  E0282 cascade. The same15,231-byte source subsequently compiles and passes all
  **eight syntax cases69681**: full table/owners/payloads, both root types, legacy
  priority, unknown preservation, EOF/header negatives, exact property splices,
  offset/foreign-handle checks and record/depth/64MiB boundaries.
- A separate NEW application-test E0308 used `Arc::ptr_eq` on the public
  `&StoredContent`. The reviewed correction uses complete value equality plus
  full capture-stamp identity, acquired under the originating Engine guard for
  wire checks. No old assertion or production authority was weakened. Originals
  remain in `target/cdx-file-end/root-test-type-correction-before/`; the exact
  correction receipt/inverse is in `independent-app-handoff/`.
- Root76198 passes **one IO, three Engine and two actual-wire cases**. They check
  actual Unicode files/raw+RDKit/reopen, unsupported source/preview/export/edit
  refusals, failed-import retention, exact history and publication/new-session leases.
- The new independent stdlib full24-record/payload oracle and pinned external
  RDKit2026.09.2 pass **18 controls**. Metadata/order mutations still passing
  chemistry fail raw preservation; coordinate and separate actual ethene controls
  exercise chemistry checks independently. Old scene-specific oracles stay exact.

The original [fixture and recipe](../crates/cdx/tests/fixtures/external-ethane-provenance/README.md)
retain both final words, original graph/generators/licenses and external Indigo1.47.0
pins. No producer execution/regeneration is used for acceptance. A separate probe
passes syntax/exact passthrough for all **ten distinct original outputs** (five
graphs, positioned/unpositioned variants). This is not ten-file model, native or
faithful-rendering evidence and admits no fetched corpus bundle.

## Fuzz conservation and bounded ASan

The new property-conservation assertion exposes the old harness classifier on the
exact fixed seed: **11 properties versus12**. It mistook the new implicit profile
for an encoded root and skipped the first0x0805 document property. This is a harness
assertion failure, not product source loss or an ASan memory defect. The final
classifier uses the semantic Framing helper. Fixed-input86996 passes; that execution
alone performs no mutations.

Actual ASan/libFuzzer45946 then completes **10,000 mutation runs**, seed721533879,
max_len131072, timeout5, from ten separately copied original/source-derived seeds.
Final reported `cov:1054 ft:2322 corp:95/209Kb rss:221Mb`; the largest
initial seed is65,750bytes. Pinned cargo-fuzz0.13.2/libFuzzer0.4.13/nightly2026-04-01
are external development tools. No committed fixture is a mutable corpus. This
bounded smoke is not maximum-input, exhaustive, performance or security proof.
Root separately executes the documented seed recipe once in a fresh ignored cache:
all ten byte/SHA identities match the frozen declaration. Receipt
`documented-seed-recipe-receipt.json` has SHA256
`52cb0843edd465dfea4bd39ad3813f7df89ba6b95b40ab399aadd48ac2f6edbf`.
That setup validation is not another mutation run.

## Bounded native Mac receiving

The old binary first refuses the exact external source, retaining the Saved7/7
ring after chooser Cancel. On refreshed binary22,936,488bytes/SHA256
`c052d758e209e1c0e53d016b6c02d93da427bc2b077b7272de04803a87285ad9`,
Root observes Open→Saved/CDX/Original source preserved, Unicode Save As to
`target/cdx-file-end/native/roundtrip-λ.cdx`, and explicit reopen with the same
faithful-preview refusal. **The incomplete-source native UI exposes no2/1 counts**;
those counts come from IO and the external consumer only.

The actual native output is258bytes and source-exact. The unchanged raw/RDKit
consumer emits the exact206-byte expected JSON without stderr. The previous
1254-byte edited ring remains exact and is restored Saved7/7/Fit78.79% in the one
current Bondline window. No native CDX chemical edit, cross-format save, race,
Windows/Linux or faithful chemical-preview acceptance is claimed. Inline CUA
observations are not a screenshot artifact or approved golden; Root completed its
CUA summary. Native receipt:2,737bytes/SHA256
`2718e89bfb8a6ea3d7ebad2dce2539dce38c691d214d95d06b9f2f79bced5c8b`.

## Full receiving gate and unexplained earlier failure

Full CI97729 passes formatting/clippy, then stops during workspace tests at
**754 passed/1 failed/0 ignored reached**. The old
`shutdown_without_clients_releases_the_listener_address` reports
`replacement_bind 127.0.0.1:51670`, AddrInUse/Os48. This is not an aggregate pass.
Exactly one separately declared receiving aggregate81324 subsequently exits0:
**1172 passed/0 failed/1 ignored**,169 result lines. It reaches the groups skipped by
97729 and passes formatting, warnings-denied all-target workspace/vendor clippy/tests,
layers, licenses and corpus. Original MCP source, operations, assertions, deadlines and
default concurrency remain unchanged. Three declared product/test source pins match
before and after this run. This is full receiving verification, not a causal fix for
the earlier failure; no further repeat is needed without a new change or concern.

Environment: Rust1.94.1; literal absolute `.cache/oracles/format-python/bin/python`
(RDKit2026.09.2/Pillow12.3.0/NumPy2.5.4), CARGO_INCREMENTAL=0, CARGO_BUILD_JOBS=2.
Declared runtime license closures remain154/165/220 packages plus2 bundled assets.
The existing eight-file/five-bundle corpus retains its explicit foreign/general CDX and
chemistry omissions; this new fixture is not a fetched corpus admission.

Separately declared diagnostic64662 runs exactly20 default-parallel control
contexts,25 tests each: all500 pass. No retry-until-pass, thread override, timeout
increase, added wait, socket reuse change or production fix is counted. The
failure-only selected-port collector is never invoked, so no ownership snapshot
identifies the historical failure. The fixed bound is exhausted. These runs do
not establish a cause, rule out a lifetime defect or fix CI97729.

The one separately authorized controlled outbound-owner probe98404 provides
counterevidence: after shutdown, a non-reusing bound outbound connection and its
endpoint/challenge checks succeed, but the replacement std listener bind ALSO
succeeds. The probe exits1 because its expected-collision hypothesis is not
reproduced. It is not a product failure, an AddrInUse reproduction, a21st natural
context or an explanation of the historical CI failure. No collector snapshot is
produced, and no rerun/second experiment is authorized by this record.

Diagnostic evidence is under the research worktree's ignored
`target/loopback-shutdown-port-investigation/`: `results20.json`11,042bytes/SHA256
`02f6915728ff27ed5d7be4472dbebfa40f87b8f1af16d2e2b97e9c5c028ddbf2`,
with all20 per-log identities verified. The [separate diagnostic record](verification-loopback-shutdown-observations.md)
preserves the source facts, contradicted hypothesis and remaining uncertainty. Broader
CDX workflows remain open. Actions stays disabled; no hosted or milestone pass follows.
The one-trial `counterexample-first-result.json` is437bytes/SHA256
`dd8d029e765731e7161992220ef16c5751b09afe44c0ad02abbcb953b2370ea6`;
its249-byte log is
`52073a0fb017c5a8ac6371ff9b622be521a6bfc85c048da4447e2df2c0f85e7f`.

## Exact inputs and receipts

| Input | Bytes | SHA256 |
|---|---:|---|
| `crates/cdx/src/lib.rs` |15,134|`c3f27f4645372fc70e75f93fa9168f4f6d4bf74c1bcdf1e631500213e79f81cd`|
| `fuzz/fuzz_targets/cdx_parse.rs` |6,405|`9f2597f0eaf04dfdbdfdfe2540aca0475e7e01ffd3a69ab6949606f404c1bfeb`|
| Original external fixture/native round trip |258|`856482269dbd85640b87c11f608104dba72ebd308a686f1333a893965ed560ba`|
| Unchanged `crates/cdx/src/write.rs` |4,229|`9e9c2a112d85cbc1ba87a45ea54b011ec81de2f2d89330d4098249485880a93d`|
| Frozen eight-case syntax test |15,231|`eb8aa20380801067a384dcfb9b944f5839e3242e9ff1af3b9156726a0c5981a3`|
| New external oracle |7,151|`1e9e84ac0b392d26b247b6ca1fb26679f59d14fa80505c687db5a7aa5b7b38a0`|

The independent22-input baseline differs only at the authorized parser source;
its other21 old source/test/fixture/oracle/lock/attribute identities remain exact.
No new shipped dependency, writer API, old fixture/oracle, font, golden or corpus pin changes.
The initial final-source snapshot's pending fuzz wording records its earlier stage;
the actual10000-run log below is the later execution evidence.

Logs below are under ignored `target/cdx-file-end/`:

| Evidence | Bytes | SHA256 |
|---|---:|---|
| `independent-syntax-handoff/first-runtime.log` |773|`95ffe683ed2d17c34b3f63a99f57be2f847d7a906659377a84413e72647e6e3c`|
| `root-first-syntax-red.log` |675|`cac253b7dcd68735af66c87e1e8df28cd20ca45b47846693f782dd695072b163`|
| `root-first-application-red.log` |1,123|`2204eb0e81c1aabe447461d7f6ccb119e1a98cb4984b0e6801d5bc0a07cdae20`|
| `root-first-syntax-green.log` |1,049|`e907bd23c69e0860ce06edacd42d150f15686cf24a623ad7a26e7ecb21759e80`|
| `root-io-green.log` |478|`384c071a02b30c6cb94bcb9f0ae3cfc30b511f9ccb40b9adc79add3f39de6a13`|
| `root-commands-green.log` |753|`824e74c4397fbd7c8d0c035ae538013317314f4abe0fef57b2d1a1e403427da8`|
| `root-wire-green.log` |944|`f17608d91d45324f4070cebe0664ccfd59c649d3e11bd88b819a3e552cace637`|
| `independent-syntax-handoff/oracle-qualification.json` |2,940|`5e8c4418c189432f57c7fc4dfaa52a8a536e3ca46aea57a6392bb14ef0319823`|
| `root-fuzz-classifier-red.log` |8,491|`2ca72c27e714c756cfb92dc1470e547aa0827fd5178904484d915b2493a03018`|
| `root-fuzz-classifier-green.log` |1,361|`2de91c1b77eca92860e14236f74e4e0a2bc32a1decc826a6260364a96cb9b037`|
| `root-fuzz-10000.log` |25,486|`08f752c69544a3f895fb6181f841084c32b3255ff4079c7a0b893084c5cc2895`|
| `ten-original-probe.tsv` |522|`be1d43204188f12b7b1987ad7857f2be7ad0d5df2e63d37cc16613275411159b`|
| `root-final-ci.log` |91,080|`78153a7443ff803009751be506278666a9b200a823b07f755cf8d40edc1e4a88`|
| `root-receiving-ci.log` |141,627|`a59098b56b4293de7e7da3f81b26ff1f2e61c6137d6ceea256cc6509b2e8ded6`|
| `native/consumer-output.json` |206|`d304cc211443f7953e4b85a86b3a8f2f0b4f0f56888f4b56ea48ffab668f5e93`|
