# Source-aware elemental CDXML profile — 2026-11-17

Base `03d5205`, following the batch primitive `1176039` and
[locked contract](research/source-preserving-cdxml-profile.md). The full M0–M5 goal stays
active. This is a closed Bondline-v0 elemental profile; general foreign styles/defaults,
labels/annotations, full format compatibility and native/platform acceptance remain open.

## Implemented behavior and boundary

Private Canonical/Preserved/Opaque states replace only the old CDXML completeness bit.
Canonical still requires every-byte equality to the native writer and keeps its old
writer path. Preserved proves every node/key/known value and complete typed projection,
positive scoped IDs and plain lexical envelope. It preserves quote/whitespace/comment,
ID/reference, numeric-lexeme and object-order variations. Opaque remains exact unchanged
passthrough with authoring refusal. Producer text alone is never trusted as provenance.

Four syntax facts collected during existing bounded preflight expose DTD, PI, CDATA and
namespace declarations, including redundant xmlns:xml; ordinary roxmltree attributes/
nodes cannot establish all four. Parser admission/resource/entity/UTF8 rules stay the
same. New independent Expat callbacks verify the facts, quoted/comment lookalikes,
mutation immutability and old rejection cases.

Current Preserved eligibility and export share a typed prefix/domain validator. Old
fragment/atom/bond order, membership, identities and directed endpoints cannot drift.
Known scalar/optional fields may change and objects may append. The writer rebuilds
original bindings, patches only changed typed fields, coalesces container appends and
reuses the existing native attribute helpers for new data. Full syntax, expected source
IDs and semantic reprojection are certified before bytes return. Sparse/burned model IDs
are explicitly rebound. No persistent map, mutable cache, registry/schema or dependency
is introduced. Lowest-unused generated IDs reserve all originals, including u32::MAX;
generated-only IDs remain per-snapshot, without a stronger persistence promise.

Whole-MOL export stays canonical/source-free only; Preserved completeness cannot drop
its source context. Explicit fragment SMILES export retains its existing graph-only
scope and geometry/document_context loss report, without saving or changing the session.

## TDD and independent receiving

Root owns all execution and tracked adoption. Agents supply frozen independent cases,
an unexecuted producer candidate and a separate source review. Exact inputs and logs are
ignored under `target/noncanonical-cdxml/io-profile-*` and related proposal directories.

| Stage | Actual evidence |
| --- | --- |
| First existing-API IO34952 | Opens/borrows the original464B file, then a typed Double/H2 update fails UnsupportedEdit. Runtime RED0/1, not a future API. |
| Syntax facts | Missing SyntaxFeatures/features is API compile RED. First producer assumes older byte QName API; pinned quick-xml0.42 uses str, so two literals are corrected. A new root xmlns='' positive fixture hits the old InvalidRoot policy; it stays as a negative while an accepted nested reset supplies the positive. All37 cdxml cases then pass, with independent Expat callbacks; no admission is relaxed. |
| Empty/update/appends | New empty-page RED0/1, scalar updates RED0/2 and append RED0/3 fail at old completeness guard. Initial empty receiving before QName correction is a compile qualification only. |
| Remaining9105 | Guard/snapshot/removal receipt1/5: all31 opaque-source vectors pass before producer, while new complete-source controls fail. All12 valid-model drift fixtures are subsequently exercised on the enabled profile. |
| Engine12991 | Normal registered Open succeeds, bond.set_order fails UnsupportedBondOrderSource. Root's temporary document wiring draft is saved/restored byte-exact around this legacy-boundary run. |
| First producer83440 | All13 independent IO cases pass unchanged: exact bytes/Expat/Decimal, every scalar/optional field, both empty-container forms, scoped repeats/u32MAX/sparse appends, opaque/drift/MOL loss guards and source/model/JSON sharing. |
| Engine46488/43323 | First history/file case passes, then all3 expanded cases pass. The unchanged Expat/Decimal/RDKit reader first accepts both independent literal manual vectors before checking actual output. CC/C2H6 becomes C=C/C2H4 with exactly three byte changes; IDs, direction, geometry and other bytes stay. |
| Adapters9214 | Actual CLI, initialized stdio MCP and authenticated loopback pass both cases; Unicode real files, history/reopen, unknown refusal, sentinel retention and canonical-only MOL controls are exercised. Existing helpers, handshake, deadlines and listener policy stay unchanged. |
| Post-producer families16109 | Nine cases pass: four Engine/history/export/consumer cases and five paired canonical/Preserved family cases. Page/settings/noop/clear, labels, selection/translation, free/extend/order bonds, sized/named/chair/sprout rings, fixed/path chains and paste match full typed state after each request. Public CPU captures are byte-identical and non-authoring. These are headless/public API checks, not native input evidence. |
| IO all85217 | 388 pass/2 fail. Both failures are old quote-only/numeric-lexeme refusal expectations. The exact vectors move to explicit positive preservation cases; all other invalid/partial/nested/opaque vectors stay unchanged. Qualified native_profile/drawing_settings pass8/0. |
| Clippy32866/58817 | One new private helper's needless explicit lifetime fails warnings-denied clippy; elision changes no body or lifetime relation. Fresh IO all-target clippy exits0. |
| First full75528 | Fmt/clippy pass; workspace test stops at an old whitespace-only settings refusal. Its unchanged lexical vector gains a positive noop/sharing/settings/Undo test, including invalid-domain rejection. A whitespace+Private variant retains the opaque precedence checks; all old other-format and semantic-negative assertions stay. Focused settings tests pass7/0 before the additive invalid-domain assertion, then enter the fresh full gate. |

The independent IO declarations are12 cases in six targets; Root's original first vector
adds one. Their final manifest is4,968B/SHA256
`7314828601ccbdbba57f9eb539c871315f38df170b33ea5f920b662e33cfad36`.
The unexecuted producer is18,248B/SHA256
`c19c38b2ef34188935525d4ec8043a1561162994b6f1d3243205d87f0eef17d5` before formatting.
Frozen independent production review finds no material issue; report7,626B/SHA256
`976e5870d001ef83cec537dab5311d97c353f54ff36fdbfdfd524f59852e9fd7`.
The later helper lifetime elision is the only producer delta after that readpoint.

## Bounded sanitizer and native scope

New `cdxml_profile` uses only existing isolated fuzz dependencies. Original20 seeds
include complete/opaque/rejected shapes, structured selector extremes and16KiB boundary.
Fixed small candidates exercise every scalar/optional field and sparse appends without
broadly masking errors; arbitrary accepted sources must preserve exact source or refuse
unsupported edits. Reopen checks use explicit correspondence, not dense ID equality.

ASan69021 exits0: **10,000 runs/33s**, actual NEW/REDUCE length16,384B, coverage4382/
features8459, final RSS461MiB. Log72,987B/SHA256
`5af1b4569143f0f044edecfd69c87e06919e06725cc2c3dfaab44655f75bac4b`.
The unchanged `cdxml_parse` target on fresh same-declaration seeds exits0 as84686:
**10,000 runs/2s**, observed NEW/REDUCE maximum15,502B of the configured16KiB, coverage4265/
features12871, RSS206MiB. Log66,641B/SHA256
`2cc899d478da3247170f88dd2a2e78e014fa4fb9dd2238b3169e37497bfe4ec7`.
These are bounded smoke runs, not general64MiB-input/large-document/security coverage.

Fresh native inventory reports native-pipe startup failure and browser access host
unavailable. No native app is launched/selected or user window/drawing inspected. The
new native Open/edit/history/Save/reopen workflow is **unverified**. Existing prior Mac
evidence retains its old binary/scope; public CPU capture is separate. New Windows/Linux
execution is also unverified. Actions remains OFF; no hosted run, push or publication.

Qualified fresh full local `cargo xtask ci`48986 exits0: **1,766 passed/0 failed/1 ignored
across263 result lines**. Fmt, warnings-denied workspace/vendor checks, layering, strict
runtime license closures, seven bundled assets/notices and pinned corpus pass. The
222,486B log SHA256 is
`5436dbb7b99972b071208f0968dce919d51cdfe7ea318b88bbc704809777226b`.
The first failed aggregate75528 remains369/1 across49 result lines, not a hidden pass;
the qualified aggregate includes the strengthened whitespace positive/domain case.

Actual rebuilt CLI discovery is byte-identical to the earlier45 records/schema/order:
SHA256 `5dbb59f2688083a37d88876c5063326b4874b3e06381b4688b6592d858a23099`.
Controls39/22/16/12 and catalogue16/124 stay. Cargo/fuzz locks, workflow, protected MCP
test/listener policy, vendor/font/assets/corpus/golden and both protected user binaries
retain their earlier hashes. Natural passing socket tests do not explain the historical
AddrInUse failure. No owned execution/agent jobs remain at this feature checkpoint.

Verified scope is the headless closed-v0 profile, file/history/adapters and stated CPU
capture/consumer cases. Its native path is wired through existing commands, not newly
verified. Full foreign CDXML styles/defaults, general labels/objects, stable generated
IDs across snapshots, full CDX/chemistry/format parity and M0–M5 completion stay open.
