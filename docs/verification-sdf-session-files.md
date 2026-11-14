# Complete SDF sessions and files

This is the first whole-SDF application-content slice, based on15ecbc7 in the isolated
bondline-rxn checkout. It is partial M1, not complete SDF authoring or milestone completion.
Native/hosted evidence must be added separately after integration; no GUI was operated in
this implementation session.

## Delivered contract

The one typed registry adds document.open_sdf and document.save_sdf while retaining all16
previous names. The resulting18 commands are discovered by the CLI, stdio MCP and
authenticated loopback. Open takes path/discard_unsaved; Save takes path/overwrite. Closed
schemas reject unknown fields, transforms and fake whole-container version selections.

StoredContent distinguishes a source-backed Drawing from Sdf{original,current}. Current,
saved, Undo/Redo and deferred file plans own complete immutable content. Every accepted
record, molecular version, ordered duplicate/header/DT/value/EOL/padding/delimiter byte
remains retained. Valid opaque molecular data need not support a model projection. No
first-record or empty drawing is substituted, and no property map or wrapper writer
reconstructs imports.

Drawing JSON keeps its existing document object and omits collection. SDF has document:null
and bounded metadata for every record: positive ordinal ID, actual mol_version, complete
record/MOL lengths and field count. Raw bytes, headers and values are absent from automatic
state and Debug. Engine::content handles both variants; document/stored_document now return
typed Results. Drawing-only authoring/rendering and cross-kind saves return unsupported_content
without a fallback model or temporary file.

collection.byte_length/Save receipt lengths describe current bytes; source.byte_length
describes original bytes. Source.modified compares complete original/current; file.dirty
compares complete current/saved content. Private existing-field candidates test complete
history without exposing a new edit command. A704-byte current Save from the690-byte
original reports704, clears dirty and retains original690/modified=true. Distinct originals
with identical current files remain unequal. Accounting counts outer enum inline storage
once and deduplicates source/index/model allocations across the retained pool. This retains
the existing threshold policy and exclusions; it is not an RSS cap.

SDF uses the same bounded reader, atomic publisher, FilePlan/Completed save reservation,
off-lock worker, stale-generation/revision, trusted UI receipt, close epoch and publication
barriers as drawing formats. Cancellation/refusal/drop releases the lease; publication does
not. An old receipt reports actual saved bytes without relabeling newer content. No second
save worker/slot or unrestricted file API was introduced.

The shared chooser/known Save/retry path adds one SDF family with sdf/sd hints. The workspace
clears previous held gestures, hover hits and paper, then displays a static complete-record
summary and says chemical preview is unavailable. This is an original partial UI. It does
not implement the installed26.1 all-record molecular drawing layout or invent a reference
record browser. Native SDF interaction remains unverified.

## Independent expectations and RED

The unchanged original vectors remain690 bytes/SHA
f95838cbb55c107c649ce2eb99ac62e191f747ce6994a171e64019f8e6c53d1a and777 bytes/SHA
165e6f5d5cff37dae6315a07843d84733aa42dbbc497b2920da1702bad9526a0. Their literal ordered
record/MOL spans, versions and field counts define expected session metadata independently
of the new collection summary. The shared test support feeds actual saved files to the
unchanged raw SDF scanner and per-MOL pinned RDKit consumers. The valid DT-only full-supplier
record loss remains diagnostic; it is not used to relax raw preservation checks.

Observed failing checks before their corresponding implementation:

| Ignored RXN target log | Actual failure |
|---|---|
| sdf-session-red.log | Runtime UnknownCommand(document.open_sdf) and old16-name registry versus required18 |
| sdf-session-io-red.log | Missing whole-SDF native read/save APIs before shared publication implementation |
| sdf-session-content-red.log | Missing complete StoredContent/summary API before ownership implementation |
| sdf-session-history-red.log | Missing complete content constructor/accessors/history API before Engine migration |
| sdf-session-ui-behavior-red.log | Actual egui frame lacked the required SDF collection label after import |

Old caller migration compile failures in sdf-session-command-migration.log and
sdf-session-ui-red.log are intentional API exhaustiveness evidence, not runtime workflow
proof. The former unsupported-SDF chooser expectations were moved to positive family
checks because that capability is now delivered. All other old assertions, approved
goldens, pinned original files, codec algorithms and external oracle scripts remain intact.

## Executed checks

Every Cargo invocation uses this checkout's absolute target directory, incremental0,
dev/test debug0 and the existing pinned external BONDLINE_PYTHON interpreter. No dependency,
lock, model, renderer, fixture, golden or vendored code changes belong to this slice.

| Ignored RXN target log | Observed result before aggregate |
|---|---|
| sdf-session-io-green.log |14 native IO unit tests, including complete SDF bytes and injected pre/post-publication failures |
| sdf-session-content-green.log |2 ownership/accounting/source-sensitive identity checks |
| sdf-session-command-expanded.log |All command-package tests pass, including5 new public SDF cases and then6 private lifecycle cases |
| sdf-session-lifecycle.log |8 final private SDF lifecycle checks pass, including changed-length raw/per-MOL consumption |
| sdf-session-adapters.log |Actual CLI, stdio MCP and authenticated loopback checks pass; both original encodings saved/reopened and independently consumed |
| sdf-session-ui-green.log |93 UI checks pass: all90 prior checks plus summary/input invalidation, native-worker routing and chooser family tests |

Coverage includes all-record source retention; Drawing JSON compatibility; no raw automatic
data exposure; malformed later records/encoding retaining dirty Drawing or complete dirty
SDF plus saved/history; unknown model projection; exact unmodified file outputs; Unicode
and misleading suffixes; no-clobber/retry; cross-kind refusal before I/O; captured save
versus newer704-byte data; whole-source Undo/Redo; original/current/saved accounting;
stale/old-generation work; trusted close receipt and revision/kind/path ABA; cancellation,
drop, save-slot lifetime and truthful post-publication outcomes. Existing reply/worker/
shutdown barriers remain covered by their unchanged generic tests.

Final candidate command (with the environment above):

```sh
cargo xtask ci
```

It printed `/Users/sarenaberg/Workspace/doozle/open-source-world/bondline-rxn` and completed
with exit0:578 workspace tests plus3 macOS bridge tests =581 passed,0 failed/ignored.
That is22 additional checks against the559-check base. Workspace rustfmt/all-target clippy,
layers, all three declared runtime license closures and the pinned corpus also pass. The
separate scoped all-target clippy command passes with -D warnings. The corpus still reports
missing broad CDX/chemistry coverage; this feature does not change its existing format pins.

Ignored `target/sdf-session-ci.log` is71,140 bytes/SHA-256
b372caca79de403cba4abb8358b66215dd0b8d79c0f53a6adfe7dd42a917787e.
The frozen43 source/test files are listed and hashed in
`target/sdf-session-owned-source-freeze.json` (SHA-256
8e2aed269fd8acf36c989ec48231b754bd5fc86704d300fdeffbf639ab219230).
Final documentation is additive to that source freeze. Build outputs/evidence stay ignored;
no published/new-platform/native claim follows from these local results.

## Remaining work

Actual native and new-source hosted validation remain pending. No explicit record.inspect
or field.set_value command, UI field editor, molecular preview/layout, active-record model,
selection/delete/reorder, molecule editing or format conversion is implemented here. The
reference imported field/attachment/export behavior remains unknown. Library-only field
replacement and private test candidates are not advertised as application editing features.

## Root receiving source and next native acceptance

Root commits the exact49-file author handoff asb842e36, then receives it onto tested
SMILES integration8e0e92b. All43 frozen source/test identities survive; newer Root SMILES
exports/catalog visibility and all original tests/fixtures/oracles/locks/golden remain.
Strict validation covers49 files,13 logs and8 protected identities. Independent review
has no confirmed defect and runs no tests: target/sdf-session-source-review.md9847B/SHA
87f31040766629f905c9b56d14aa2cd71cdf31abd1b97500a0196d9abad02582.

Receiving cargo xtask ci79477 exits0:587workspace+3bridge=590,0failed/ignored, all gates.
It prints the correct Root workspace, absolute target/default dev/test debuginfo,
incremental0 and pinned external Python. Log target/sdf-session-integrated-ci.log72645B/
SHAf78935e91fe4023f93a79fdf4dc2e2824c1bc200737d6b0187a1da17dce9ea1f.
No source rule/parser/dependency changes or reduced test graph is introduced. Native
acceptance and this receiving source's hosted execution are still pending.

The prepared native plan corrects an earlier assumption: Save As retains its existing
suffix-validation contract, so Unicode SDF uses.sdf/.sd and mismatched Save As must
refuse. Misleading suffix/explicit association is tested through a supported explicit
Open/Known Save route, never by forcing the chooser. This is source-derived planning,
not an observed GUI result. Actual native pixels/actions/bytes need a new frozen binary.
