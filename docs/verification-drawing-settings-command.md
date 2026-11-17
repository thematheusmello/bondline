# Atomic drawing-settings command: receiving

2026-11-16–17, data base b6497dd, contract dede5ee and separate UI test qualification
e516dd9. Full M0–M5 remains active/incomplete. Actions OFF; no push/publication.

## Delivered boundary

One appended `document.set_drawing_settings`/`DrawingSettingsSet` requires exactly three
keys: expected revision and two number-or-null values. Explicit null clears; missing keys
refuse. Runtime model validation remains finite length>0..100000pt, angle1..179°. Source
authority precedes domain/equality: source-free or complete native CDXML only, including
no-ops. Existing source/completeness and whole-MOL loss guards remain authoritative.

Validated no-ops retain source/model Arcs, redo and all contexts. Real edits use one normal
immediate commit, preserving pages/H/geometry, tool, camera, view focus, generation and ID
floors; normal selection/close/file invalidation applies. No worker/Save lease/receipt,
native form/binding, inheritance/default or tool application is added. Registry41 leaves
controls31/20/14/10 and catalogue12 delivered/128 disabled. Native chain controls stay grey.

[The contract](research/drawing-settings-command.md) and
[source register](clean-room/sources.md) distinguish public creation-default roles from
these original protocol/transaction policies. Installed serde1.0.229 and schemars1.2.3
source informs required-present nullable fields: ordinary Option schemas plus an explicit
root required list, without field-level `schemars(required)` removing nullability.

## Actual TDD and independent cases

Root owns all executions/integration. Workers provide ignored declarations/reviews only;
none of their source reading is counted as runtime/native evidence.

First existing-JSON test is1261B SHA256
`06e5d40f59c32eb57a03c1475316aa8c371b699921f1af95c9c67a1f4b1adbd9`.
On the pre-producer engine it creates an old bond, then expects one literal24.5/120.5 edit
and exact old pages. Job35719 exits101, **UnknownCommand,0/1**, before producer/types.
Producer18321 then passes the same expectation **1/0**. Later formatting preserves its
behavior; first bytes/log remain archived. New-type cases are separate API qualification,
not a fabricated missing-type RED.

| Root execution | Actual result and coverage |
|---|---|
| 79634 core targets | 9/0: first1, independent behavioral6, typed1, property1. Required/null/closed schema, ranges, source classes including equal refusals, stale/source/domain order, no-op sharing/redo/selection/close/file intent, exact scene/pages/history/IDs, Unicode original/current/saved distinction and immediate/late-cancel/focus policy. |
| 6151 private counter case | 1/0: actual revision/selection epoch MAX, valid no-op, refused real edit, unchanged current/saved/model/stamps/focus/IDs. |
| 48981 captured Save case | 1/0: edited/no-op arms, lease through publication/finish, exact old bytes and truthful old input revision, same-generation path/old baseline association while newer current stays dirty, no UI ACK, clear-to-old baseline and dropped-work nonpublication. |
| 1536 actual adapters | 3/0: CLI, initialized stdio MCP, authenticated loopback. Schema/discovery, strict/stale/source/no-op/history, settings callbacks only real edits, actual Unicode native files; both registered MOL versions have positive unstyled controls before existing/absent destination refusals and null-clear success. |

These initial fourteen cases pass on first receiving. A later serialization regression7219
gives actual RED0/1: typed Some(NaN) becomes JSON null, a valid clear request. A command-local
field serializer rejects nonfinite Some while preserving explicit None/null and finite
values;46704 gives GREEN1/0. Incoming validation, source/commit and schema stay unchanged.
This is a real caller-boundary safety fix, distinct from the numeric-separator lint below.

Fifteen new cases pass. The property case runs64 generated supported optional pairs,
checking unchanged geometry and exact snapshot Undo/Redo; receiving sets
`PROPTEST_RNG_SEED=671308429`. Expanded declarations are frozen without reading producer
and honestly recorded as post-first-producer acceptance.

Existing independent Expat/Decimal root-field checks precede unchanged raw directed bond/
flag/geometry and external RDKit consumers. They verify present/removed root properties;
old chemical readers alone cannot prove metadata preservation. No old reader, fixture,
approved golden, parser or shipped dependency is changed. Format fuzz harnesses remain as
verified at b6497dd; this command adds no format parser and claims no new fuzz/security audit.

Reproduce with the pinned oracle venv (retain its symlink path):

```sh
export BONDLINE_PYTHON="$PWD/.cache/oracles/format-python/bin/python"
export CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0
export PROPTEST_RNG_SEED=671308429
cargo test -p bondline-commands --test drawing_settings_set --test drawing_settings_behavior --test drawing_settings_types --test drawing_settings_history --test drawing_settings_save --test drawing_settings_serialization --locked
cargo test -p bondline-commands --lib drawing_settings_tests --locked
cargo test -p bondline-cli --test sdf_capture_adapters settings_wire --locked
```

## Inventory and bounded source review

The independent inventory maps46 cardinality assertions in37 files plus16 exact-name
comparisons. Root positively updates them across44 files:40→41 and one added name, retaining
old names, schema/behavior predicates, OLD_* constants and request ID40. An initial adoption
check catches a CSV multiline line anchor before any file writes; corrected exact spans
apply the same intended assertions. No old behavioral predicate is weakened.

Producer review checks exact inverse reconstruction of the old lib.rs and all44 before
inputs. Its subsequent qualification confirms producer/lib bytes unchanged after formatting;
only test layout, an additive63B child-module bridge and separator-only
`100_000.0001`→`100_000.000_1` differ. The tested out-of-range literal is identical.
Review SHA256`ce45e64af6a3069904299bce367037b7d80854cc55b62bafb8ab523d6f77f720`;
qualification`b1eb177da1c81cbdfddbc0a68c711b5662dff0809b2ec85fcd67a0235fc7c042`.
Exact61/66-input pins and copies are ignored under the producer-review directory. These
reviews prove source continuity within scope, not independent runtime or native parity.
They precede the later serialization guard; its separate bounded review finds no material
issue and reconstructs the original source/domain/history code exactly. Serialization
review SHA256`ac398e1c5c767299b6edaa534b0effc3e602db3142f15428d342db92e42d805e`.

Actual newly built CLI discovery independently compares the complete old40 records,
schemas/descriptions/flags/order against preserved prior CLI output: every record equals
the new prefix and only the final command is added. Old33798B SHA256
`dcb9d6cfead7f9da962cadaeec6946383ff20ef0d7a4223cd0e4c4324a6217ef`;
new34619B`fd857d3f9643a6cb8f7957ee89419f2655ea56acb5e7f1c8d82d2992aff8879a`.

## Failed aggregates and separately completed checks

First `cargo xtask ci`79447 stops at warnings-denied clippy's numeric grouping lint in a
new test, before workspace tests. The separator-only correction preserves the value.
Corrected full gate37811 passes rustfmt/workspace clippy, then exits1 at the unchanged MCP
`shutdown_without_clients_releases_the_listener_address`: replacement bind51300,
AddrInUse/OS48, **1012/1/0 across161 result lines**. No unchanged retry, owner identity,
socket/assertion/deadline/concurrency change or exhausted probe is introduced.

Only not-started stages run in24093; its **486/1/1 across50 lines** includes the separate
existing SMILES UI readiness failure. [The test-only qualification](verification-smiles-order-readiness.md)
preserves that failure and the uncaptured schedule uncertainty. The helper gains actual
owner retirement plus timeout diagnostics; old assertions, ten-second deadline, two warm
frames and product busy fences stay. Full modified UI library passes268/0/1. Subsequent
`--bins --tests` also repeats that library and adds50 binary/integration passes; no extra
unique coverage is inferred. That85282 script then exits1 on invalid xtask `--lib` selection,
before any xtask test. Its runtime sum is586/0/2 across9 lines.

Using the actual bin-only xtask selection, then-unstarted74317 passes55/0/0 across18 lines:
xtask, compatibility targets, workspace doctests and vendor tests. Rustfmt, warnings-denied
UI/vendor clippy, layers, seven-asset licenses, unchanged155/166/221 runtime closures and
the pinned corpus pass. Both failed test runs remain failed; this is **not a green aggregate**.

Ignored receiving logs under `target/drawing-settings-command/root-receiving`:

| Log | Bytes | SHA256 |
|---|---:|---|
| first-red.log | 994 | `36f501e57c7c898431b2a40ef552c0f4fd8ebb16751663bf9c5eab490e22efb4` |
| ci-lint-first.log | 1389 | `df1e338d74be05d95dcdb57841c8805e2971c2fe144e14058358c56e6de0d9cb` |
| ci.log | 125470 | `336de6dbcb9b5aa05dcc5971bdbb38c02beac29c1a2832ee76a2bfa942910f85` |
| remaining-stages.log | 62631 | `8ed05d3ea3422fa42d74a8949cc8b937d4e2a5b2d5eced1ec3b82a4876f531d2` |
| qualified-remaining.log | 68993 | `99bb3cec3621aa736ddac1546892984b42b9519a11488fa0d2148c238e6f414a` |
| then-unstarted.log | 10550 | `8b4ff4807c49feb5b80c265941cb2e7e5a873a6e43cf8d39c285a0f3278cd640` |

## Serialization boundary and storage recovery

The later7219/46704 regression rejects typed Some(NaN/±Infinity) during serialization,
before JSON could reinterpret them as explicit-null clears. None/finite values retain
their exact payload; incoming/domain validation is unchanged. Serialization RED1231B
SHA256`4bde959019c5e3063edcede83bfc733bf01020c16987b484e4d23dc8e862af51`;
GREEN491B`1b2eced5160b503826e7fead16e541345bbade6e8bec3db99d360946c389b24b`.

Current fmt/workspace clippy pass in97163. Its command/CLI runtime sweep then stops at
smiles_files: six explicit OS28/StorageFull errors, after482 passes/6 failures across80
result lines. These are preserved as environmental failures, not attributed to the
serializer. The volume is observed100% full,432MiB reported available. Scoped
`cargo clean --target-dir target/debug-pre-index-recovery/deps` removes7084 old rebuildable
dependency artifacts/4.4GiB; current target/debug, protected app bundles, source, fixtures
and all logs/pins remain. The volume subsequently reports9.0GiB available; concurrent
volume changes are not attributed solely to this cleanup.

Only the failed runtime target and not-started command targets (derived from actual Cargo
metadata), commands/CLI doctests, gates and native/CLI build resume in42816. It passes
51/0/0 across11 result lines; the earlier failed target's one passing case is naturally
repeated and is not extra unique coverage. No assertions, deadlines, client policy or
protected MCP tests change. Storage log64964B SHA256
`8fd764405155bb0ddf784630503774b6513a31863284fec2fcd42222224daf03`;
recovery9730B`9273b5122017ad2995e3f244d869ac54bd06cf1b7617f3ddfca38c194a8bc9ea`.
Actual final CLI discovery is identical across all41 records to pre-serializer output,
including the new nullable/required schema. No full green aggregate follows from this
targeted recovery; the earlier MCP failure remains unproven and unchanged.

## Actual artifacts and continuation

Initial native/CLI build89296 exits0. Final build within42816 includes the serializer
guard: native SHA256`4170ce64bc3dedc97b6901176ef8c5fffeb7dba337a9b284e19cb87f221e3d08`;
CLI`0fd9a2cc767356a2501ae92eb406c84de23ab1d0e4a74df42a4f995162c4e092`.
No native launch or copied-bundle refresh.

The initial additional CLI smoke opens original `原-ethane-λ.cdxml`, sets24.5/120.5, changes
the bond to Double, clears length/sets137.5°, Undo/Redo, saves to `保存-ethene-設定.cdxml`,
performs an exact no-op and reopens. Saved revision6/undo3/redo0/dirtyfalse and
source.modifiedtrue remain distinct from reopened clean baseline/history. Independent
root Expat/Decimal proves only ChainAngle137.5; unchanged graph reader proves two H2
carbons at(100,200)/(140,200) and directed Double, and RDKit2026.09.2 reads C2H4/C=C.
The input stays exact. Actual output427B SHA256
`2cef9002cadaa9d7cc0004845844848d9df98752a14149968cf732d9ea225d7b`;
identity2788B`a123a313a6cff5dc73bc584cbacc47ad359c4a5521a92592c346bd9c8ae0ce17`.
Its ordinary JSON success path precedes the final serialization guard; the actual CLI/
stdio/socket acceptance cases also pass on guarded code in the later integration sweep.

All managed execution jobs are terminal. User apps/copied bundles/drafts remain untouched;
current drawings are not re-inspected. No native settings workflow, new platform/DPI,
human-approved golden, hosted run or publication is claimed. Next implement ordered native
modifier events and the complete settings/chain forms, drag/count/repeat/poses/context/
real-ink workflow with actual native acceptance. Full M0–M5 remains required, including
the unproven MCP failure's proper identity-based investigation.
