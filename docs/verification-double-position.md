# Persistent DoublePosition data and CDXML

2026-11-16. Data/format prerequisite after185476d, following the locked
[contract](research/sided-double-prerequisite.md). This does not implement inset strokes,
skeletal labels, an alignment command, benzene/cyclopentadiene tools or aromaticity.
GitHub Actions remains off; no native window/reference binary is used.

## Independent declarations and RED/GREEN

Root alone runs Cargo/Git and changes live files. Three workers provide source research,
immutable declarations and bounded reviews under ignored target/. Public SDK names and
IUPAC depiction rules are versioned evidence; original scope/default choices are explicit.
All Cargo commands use Rust1.94.1, incremental off/two jobs/debug info off and the literal
pinned external `.cache/oracles/format-python/bin/python` interpreter.

Before product edits:

- Model JSON1493B SHA256
  `e930221dc7ca504aca339371121e78a437dddedfc09d302f73751c097050c6a9`
  reaches runtime0/1: unknown_field(double_position).
- IO XML1179B SHA256
  `f868919862a889e997f2763e97a35a2d5072025fba8b654106daa909b2caae40`
  reaches runtime0/1: raw bytes remain exact, typed Bond JSON omits Right.
- Separate public API741B SHA256
  `8f95cb7afdfcb760bec50e4c6f1074558f08492f4e9032ba2c95485823b4142d`
  fails E0432 for missing DoublePosition. This is not a runtime behavioral RED.

The default-omitted model field follows those REDs. Compiler diagnostics drive exactly
63 old Bond initializers:9 production,54 test sites. Each fresh constructor explicitly
uses Unspecified, with CDXML later replaced by its real property reader. Source IR Bond
and HitTarget/RingPlacement variants are excluded. A migration-helper qualification
initially uses crate:: in two model integration tests; that means the test crate, not
bondline_model. Those qualifiers are corrected, with no value/input change. The helper
refuses the non-E0063 diagnostics before writing; a redundant compile of that setup is
not behavioral evidence. Final compiler check7 passes all workspace targets.

Expanded model JSON5787B SHA256
`043b77276e50c6018cf7334d0c4c6d7f40992438ea0db6d746853d06d66d1b23`
retains its entire1493B prefix. Model52241 passes JSON4/API1: exact four values, omission,
directed/non-dense IDs, dormant data, invalid/closed input and unchanged Fragment guards.
The expansion follows the first model draft; it is not claimed to predate that draft.

Before mapping/writer/loss/render guard implementation:

- Expanded IO18466B SHA256
  `22893345a0d0cd5cc8cbc42f672ce1d12dda7e69657fddadf05a39e6d4b229c8`
  retains1179B prefix.72175 reaches2/7. Positive controls are the original independent
  reader qualification and raw SDF/RXN/CDX preservation. Failures expose missing XML
  typed names/conditional output/invalid-name handling, silent MOL loss, atomic-file
  encoding loss and unauthorized retained-SMILES placement replacement.
- Renderer6308B SHA256
  `84cfbf3df4bd00204f93e2ca8b005ab9f72256d36e5a2704d027ebd1dc5341af`
  reaches2/2 in38740: Center/legacy and cancellation pass; sided Double and specified
  Single/Triple silently render old geometry. This occurs after model support, so the
  renderer assertions are reached rather than masked by a missing-field setup error.
- Registry8389B SHA256
  `08528bf3b9a95d5dbc25331a8658389996fef58c577f7dd467ef1b1248de4f42`
  reaches0/1 in42243. The independent actual-file Expat/Decimal check sees no Right.

After those REDs, CDXML reads/writes the three exact names; MOL model writers refuse
specified metadata; retained-SMILES replacement permits only its existing H/order changes;
the shared renderer accepts Unspecified or explicit Center Double and refuses unsupported
placement. No inset success, default auto positioning or source-authority override is faked.
IO29831 passes9/0, renderer88874 passes4/0 and registry82967 passes1/0 unchanged.

## What those tests establish

Native XML preserves presence, direction, dormant metadata and old v0 bytes. Source
completeness remains exact-byte re-encoding, never a producer-name/whole-map override.
Invalid property names reject typed mapping while the standalone syntax retains all
bytes. A newly original Expat/Decimal reader is qualified by three manually authored
positive documents and eight semantic mutations before checking complete writer output.
Actual Unicode files also face the unchanged graph consumer and pinned RDKit, then real
Open/Save/order/no-op/history operations. Failed MOL output preserves files, Engine state,
source/Arc ownership, association and prior work; no partial publication occurs.

Both V2000/V3000 model writers reject any specified position, including Center/dormant.
Delivered SDF/RXN writers wrap existing MOL bytes; no model-taking API or invented test
closure is claimed. Actual raw wrapper and CDX automatic/manual property controls remain
exact. Graph-only explicit SMILES export already excludes depiction/style; it is not a
complete-document Save. Retained-SMILES H/order replacement rejects a position difference;
existing complete-model equality rejects direct model_mut drift without another override.

Explicit Center Double preserves the sealed legacy scene, literal strokes/gaps/hits,
full CPU/GPU frame and input data. Left/Right and specified non-Double data refuse before
exposing a scene; cancellation remains first. Old Single golden, font and pixel predicates
are unchanged. New display choices do not erase authored H or imply skeletal presentation.

## Review, bounds and remaining dependencies

Source-only review independently counts all63 initializers and confirms every old field/
value survives.48 of49 constructor files recover the exact baseline after removing only
the inserted default; CDXML has only its intentional mapping/import/helper changes.
Chemistry/layout/order edits copy whole bonds and preserve placement. Accounting uses
actual size_of::<Bond>(), with no new heap owner or quota change. Standalone parser/codecs,
old fixtures/corpus/oracles/golden and dependency/asset policy remain unchanged.

The existing cdxml_parse fuzz harness already exercises ElementalGraph and native source
proof/round trips. A fresh ignored15-seed corpus adds three native positions, absence,
dormant/unknown-root, invalid values, truncation and zero input; its manifest is under
`target/sided-double/root-receiving/`. The unchanged harness's bounded ASan run57919
passes10,000 executions, seed492176381, max_len65536 and timeout5. LibFuzzer consumes14
nonempty seeds (408..450B,5995B total); the empty seed is ignored. Final cov2982/ft7862,
335 entries/90KiB, adaptive lim450 and RSS168MiB are observed. Its two-second fuzz phase
excludes compilation. No sanitizer failure occurs; lim450 does not exercise max_len65536
or prove the full file-size bound. Log61337B SHA256
`b68d1afa1f7291b0dc8e71511d8dff5fccbc578cf78b256e9969143760cfc09f`.

The isolated fuzz lock was stale: Cargo adds only the already existing local
bondline-chem0.1.0 package and its bondline-io edge. A full parsed-lock comparison confirms
all external packages/versions remain exact. Root Cargo.lock, both workspace/fuzz manifests,
the harness and shipped dependencies stay unchanged. This nine-line test-only alignment
is committed explicitly, not hidden as an unchanged lock or a new runtime dependency.

## Formatting and aggregate receiving

Root99879 rustfmt passes. All five live declaration units initially reproduce exact
`rustfmt --edition 2024 --emit stdout` output from their sealed raw bytes; hashes are in
the ignored formatted-declarations.json. Aggregate54264 then stops at one new-test
needless_borrows_for_generic_args lint. Removing the redundant borrow in current_bond
passes the same Copy Bond to serde_json; no expected value, input, assertion or tolerance
changes. This is a declaration lint qualification, not a runtime product RED. The failed
aggregate remains recorded separately from the final gate.

Final aggregate49615 passes fmt and warnings-denied workspace/all-target clippy, then
fails909/1 across137 result lines at the unchanged MCP test
`shutdown_without_clients_releases_the_listener_address`. Replacement bind
127.0.0.1:57773 reports AddrInUse/OS48; no failure-time owner identity is captured.
Log111119B SHA256
`3cae96e504cfb3c66d53807f50fbd51902bcd198e372d09ed417e966972ed9ab`.
MCP control.rs remains SHA256
`fd96b257cd6c00b52e97544d652ed69fdbaaaa454e1f3a9360d77f3292218fb0`.
This is a failed aggregate, not a passing full gate or a newly attributed socket cause.
The consumed controlled probes/20 diagnostic contexts are not repeated. Root84887 runs
only the eight not-yet-started MCP integration binaries, remaining packages/doc-tests,
macOS vendor tests/clippy, layers/licenses/corpus and native/CLI build. It never reruns
or skips assertions inside the failed control binary; separate results follow below.

Root84887 finishes all nine remaining commands with exit0:521 passed,0 failed,1 ignored
across65 result lines, plus vendor clippy/layers/licenses/corpus/native-and-CLI build.
The ignored test remains the existing optional GUI capture; no ignore is added. These
are separate not-started-stage results, not a green aggregate or a socket fix. The exact
command/log ledger is remaining-checks/results.json,4305B SHA256
`c85a0014cd8ce2493e4650a62d847a1a91e105dea3b6d252b6b36db9b0d1a5bd`.

Built native executable22673808B SHA256
`322089e18aa19504e4e345ed140b09e79056c9488e8ad61364f69ccc38b70b1c`;
CLI20696304B SHA256
`72ef57442f0d8d4038d8c39f0b730c7c234c50b2ba41abbc598a732b9ff1b927`.
No new window is launched for this data-only slice; native interaction, DPI and
Windows/Linux execution are unverified. Existing offscreen/golden evidence is unchanged.

The final bounded source review checks46 prior inputs: ten changed Rust files differ
only in formatting, with no token/literal/comment-content change. Its report4001B
SHA256`5a8fc7e88606ffe432290e37b74ab068d27db0ad56bda2b9dfa3addc6e180737`
and handoff15918B SHA256
`27b508f9c15beb0d9644f84cf93000297d16ed10ab614e529d204d2b3a42963a`
remain under ignored target/sided-double/final-formatted-review/. All168 external fuzz
package records/checksums/dependency arrays remain exact. Root additionally removes only
the54 added default fields from all41 old test files and rustfmts: each recovers its exact
HEAD baseline. New expectations remain frozen, with only the separately qualified
one-byte borrow cleanup. No fixture, old assertion, consumer, golden, shader, font,
resource limit, deadline, dependency policy or socket behavior is relaxed.

The first metadata-only ID audit excludes13 R26 rows because its selector permits only
letters. Including digits in the prefix recovers all582 IDs in the same baseline/current
order, without changing inventory or assertions. Relative Markdown links and staged
whitespace are checked separately; this audit setup is not a product behavioral RED.

All agents and execution handles are terminal, including fmt99879, aggregates54264/
49615, fuzz57919 and remaining84887. Actions remains off; nothing is pushed/published.

Next: shared inset components with real shortened-span corridors/gaps/hits and pixel
proofs, then a separate skeletal-label policy preserving literal H, then unsaturated
ring candidate/phase/weighted-valence rules and real native tools. XML names cannot
reconstruct the CDX automatic/manual bit; no new CDX model authority is claimed. Public
GUIDE25 does not resolve shared-edge phase/order precedence. These unknowns remain in
the roadmap rather than being inferred from installed proprietary software.
