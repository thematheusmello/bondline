# Bondline agent instructions

## Goal and orientation

Implement an independent offline Rust chemical structure and reaction editor, native on
macOS, Windows and Linux, eventually also WebAssembly. Original code is MIT OR Apache-2.0.
The reference is publicly documented Professional 26.1 behavior, not proprietary code.
Read STATUS.md, ROADMAP.md, docs/architecture.md, docs/parity-checklist.md and
docs/clean-room/sources.md before selecting work. Continue existing work; do not restart.
The persistent project goal is active. M0's native license blocker is resolved; this does
not complete M0. See docs/control-protocol.md for the delivered command/adapter contract and
docs/verification.md for exact RED/GREEN and platform evidence.

## Research, provenance and implementation

- Target one-to-one Professional desktop screens, control roles, icon roles, workflows and
  functionality, with Bondline branding and the existing offline/open-source commitments.
  Use public documentation, screenshots, videos and authorized artifact/runtime inspection.
  The user's 2026-11-13 clarification supersedes the earlier program-binary inspection ban.
- Use REA for artifact/runtime questions and ordinary source tools for open-source repositories.
  Record exact source/build/version/platform, URLs or artifact hashes, video timestamps,
  observations, inferences and unknowns. A screenshot or static string alone proves no flow.
- Obtain research artifacts from official/public or otherwise authorized sources. Use existing
  entitlements for licensed runtime access; do not bypass activation/access controls. Record
  actual access blockers and continue independent public research. Keep reference downloads,
  screenshots, installers and REA reports in ignored .cache/ or target/, never shipped by default.
- Implement original Rust logic and original redistributable assets matching documented roles
  and appearance. Do not transplant proprietary source, decompiled code, fonts, templates,
  sample libraries, help prose or icon files into Bondline without applicable redistribution
  rights and the asset/dependency policy. Reference caching is distinct from product reuse.
- Never use proprietary product marks in Bondline's UI or executable names. README may
  identify compatibility and must state non-affiliation. Parity/source records may name references.
- Shipped dependencies and bundled assets must permit use under MIT, Apache-2.0 or BSD.
  Do not broaden this allowlist or change license labels to pass a check. Preserve notices.
  Build-only tools are distinct from shipped runtime dependencies. GPL/LGPL/AGPL software
  may be run only as external test programs, never linked or copied into the shipped application.
- Record sources and behavior decisions in docs/clean-room/sources.md (historical path).
  Unresolved behavior goes in docs/clean-room/open-questions.md; do not guess from memory.
  The updated method does not claim strict isolation from runtime/artifact observations.

## Architecture

The intended crate graph is model/cdxml/cdx -> chem/render -> io -> commands -> mcp -> cli/ui.
Packages use bondline- prefixes. cdxml and cdx have no workspace dependencies; formats
preserve unknown data and must never silently drop it. Native I/O stays out of model/parsers.
The UI, CLI, loopback channel and MCP call one typed JSON command registry.
StoredContent now distinguishes Drawing(StoredDocument) from complete Sdf{original,current};
Engine current/saved/history/file plans retain that whole variant. Drawing JSON is unchanged;
SDF has document:null plus collection metadata, never a first/empty drawing. document() and
stored_document() return typed Results; preserve wrong-kind refusals and clear stale UI data.
StoredDocument owns drawing source/model/baseline. Keep that backing
when wiring import; a cloned elemental projection alone loses unknown data. Require failed-
import/source-retention tests before exposing imports. Retained payload/container accounting
now enforces 128 prior snapshots and a 128 MiB threshold, deduplicating shared source/model/
fragment allocations and including baselines/reserved vectors. Keep current data even when
it alone exceeds the threshold; this excludes allocator headers, caches and external copies.
Update accounting/tests whenever a new model field owns heap data; do not claim a hard RSS cap.
Unchanged-source export is guarded; never write stale XML after a model edit. StoredDocument::
to_cdxml borrows unchanged source or encodes new native elemental data. Do not bypass it by
regenerating stored.model(): that projection omits unknown data. A generated profile is fully
mapped only if re-encoding reproduces every input byte. Its cached classification now permits
headless editing/rendering; unknown/extended/noncanonical files remain guarded and preserved.
Keep the pinned v0 fixture editable when changing the encoder/version; add migrations instead
of refreshing it. Native file wiring now uses those complete snapshots. Poster Width/Height
are drawing extents, not inferred paper/printer settings.
xtask checks layers and licenses, fetches a pinned SHA-256 corpus, and orchestrates CI.
No GUI dependencies below ui; wgpu belongs in render. Add crates when useful behavior lands,
not empty successful-looking API stubs. Document any isolated OS FFI exception explicitly.
Loopback connections require the fresh session token emitted only in opt-in startup stdout.
Never commit or expose actual tokens in logs, screenshots or verification records. Stdio MCP
is separate and opens no listener. Keep file access bounded and explicitly authorized.

Current map: model (L0, pages, Point2 and validated elemental atom/bond/fragment graphs);
cdxml (L0, source-preserving syntax, integral/source-ID decoding, checked attribute
replacement and a bounded new-syntax writer);
cdx (L0, bounded flat binary syntax, exact source passthrough and checked opaque property
replacement and new-tree Writer; model mapping and application commands live above this crate);
render (L1, shared neutral-carbon/Single GraphScene, bounded hit regions, literal authored-H
labels, CPU/GPU rectangles/strokes and approved golden); io (L2,
source-backed CDXML/MOL and limited CDX projections, elemental encoding, owned snapshots, PNG encoding,
bounded native CDXML/CDX/MOL/SDF filesystem libraries and source-preserving SDF/RXN container records;
bounded SMILES notation/source ownership including literal aromatic markers/bonds,
opt-in fixed-H normalization and
restricted noncanonical Fragment export, not application SMILES or chemistry;
see docs/native-file-io.md for publication/access limits);
commands (L3, registry, complete stored-snapshot Arc history and transient close policy); mcp (L4, stdio/native loopback, registered work and reply publication barriers); cli/ui (L5,
headless and native adapters). Other planned crates do not exist yet. `compat/libloading`
and `compat/clipboard-win` opt into reviewed OS unsafe; `compat/hexf-parse` forbids
unsafe. These three original compatibility packages retain upstream package names solely
for Cargo patch routing. The eframe/winit source forks and Apache font retain upstream notices.
The pinned winit macOS application-quit callback is the other reviewed OS-boundary exception:
two standard main-thread AppKit visibility actions, with exact source/pin/native evidence in
vendor/winit/BONDLINE-PATCH.md. No unsafe enters Bondline's UI/model/command/format crates.
The pinned accesskit_macos string-value bridge is another reviewed native boundary; its
exact source/notices, nullable NSString checks, bounds and tests are in that vendor's
BONDLINE-PATCH.md. cargo xtask ci includes its native tests/clippy on macOS from the root lock.

## Test first, then implement

For each behavior: write one test using independent expectations, observe the intended
RED failure, implement minimally, observe GREEN, then refactor. Record commands/results
in docs/verification.md. A round trip through only our own reader and writer is insufficient.
Use independent parsers, published reference vectors, externally run chemistry tools,
analytic cases, approved goldens and property tests as appropriate.
Documentation/configuration-only work needs relevant checks, not artificial tests.
Keep invalid input, resource bounds, error handling, accessibility and data preservation.
No production panic/unwrap/expect/todo/unimplemented and no unsafe outside an explicitly
reviewed OS boundary. Borrow where practical; use typed Result errors in libraries.

To add a command: first add its independent behavioral/error/history tests, register its
typed closed parameter object once in command_registry!, implement it below the UI, wire
delivered adapters, update CLI/MCP wire tests as needed, then update provenance and parity.
For UI changes, launch the actual native app and inspect the workflow. Eguis's input tests
and source cross-checks do not substitute for native platform execution.

Every native save must use the shared FilePlan/Completed reservation. One outstanding Save
per Engine is allowed from prepare through finish/drop, including after publication. Never
release it at the worker/publication boundary or replace it with an adapter-only guard:
reversed completions reproduced a clean wrong encoding and a later Auto precision downgrade.
Preserve busy-before-I/O, release on errors/cancellation/drop, independent Engines and
truthful old-generation/committed receipts. Successful receipts must be finished, not discarded.

MOLV2000/V3000 now have source-preserving codecs, owned snapshots, atomic I/O and shared commands.
Its transform is explicit, headers/raw source stay authoritative, and unsupported projection
must fail rather than erase data. Associated file.format is independent of source.format and
filename extension. Open defaults24pt/unit and origin(72,72)pt; source-backed Save keeps its
captured frame/headers. Multiple fragments, changed page layout and source metadata loss
remain refused. Do not call this complete M1 support. Pinned MOL fixtures have -text Git
attributes: preserve exact hashes on Windows, never normalize them in an oracle/parser.
Declared valence yields exact authored H by the published finite arithmetic; do not infer
periodic-table valence. Preserve the documented normative M ISO/RDKit discrepancy, and the
original/independent test vectors. The corpus now actually checks one MOL input, not only
its hash. An original explicit-root CDX scene has fetched byte/struct/RDKit checks;
foreign/general/application CDX and Bondline chemistry calculations remain missing.

Open selects the actual MOL header version and fully validates that codec; no suffix-based
version fallback. Save's optional version is auto/v2000/v3000, captured during prepare.
Auto keeps the associated MOL version, else original MOL version, else V2000. Source info
still says mol; file state/receipts say mol_v2000/mol_v3000. Original V3000→V2000 refuses;
V2000→V3000 also requires strict raw-coordinate preservation, never the V2000 rounding
allowance. The chooser has one MOL Open family and two concrete Save encodings. Preserve
same-path version-change invalidation, exact failed retry and trusted close-Save receipts.
Native/hosted evidence must be separate from engine/egui tests; see
docs/verification-v3000-application.md. No runtime dependency or codec source changed here.

SDF is a mixed V2000/V3000 source container and complete application-content variant.
Shared Open/Save retain every record. sdf.record.inspect exposes bounded ordered fields;
sdf.field.set_value edits one existing occurrence through immutable deferred candidates.
Chemical preview, field-edit UI and record operations remain missing. Preserve off-lock
work, exact no-op sharing, stale revision/generation/Arc checks, saved/original distinctions
and the same worker/reply/close barriers. Explicit data results are separate from automatic
metadata-only state/Debug; see control-protocol.md for input/output budgets.
Its generic writer preserves actual molecular versions; write_v2000 must refuse V3000. Preserve exact
record/MOL/item spans, duplicate order, raw headers/DT digits, empty/multiline values and
opaque embedded MOL bytes. Never rebuild these from RDKit's last-value property map: its
full supplier loses records/data at the original valid DT-only vector. Independent raw
checks and per-record molecular consumers have different jobs. The writer must refuse
reserved delimiter prefixes in embedded MOL blocks before output, not normalize the source.
Strict EOF/header/ASCII bounds and the documented post-item empty-padding accommodation
must stay explicit. See file-formats.md. Corpus now has five mandatory pinned source bundles, including original SMILES vectors;
validate all licenses/notices/global cache names before cache/network I/O. The original
SDF fixture is fetched from its actual public source commit; never invent a future pin or
rewrite existing RDKit metadata. Cache ancestry must be confined before directory creation.

RXN is a standalone legacy container. Keep wrapper/MOL versions distinct, ordered
reactant/product/reagent counts, complete molecular payloads/maps/unknown data and full EOF.
Do not flatten it into an elemental fragment or call the library a reaction editor.
Mixed molecular versions and omitted zero-reagent counts are observed consumer extensions;
V3000 wrapper AGENT versus published REAGENT remains an explicit unresolved conflict.

The current twenty-command registry includes complete SDF Open/Save and bond.create for independent C-C fragments with
three explicitly authored H per carbon, plus CDXML/CDX/MOL open/save. bond.extend is a
headless terminal-carbon operation with required explicit H3-to-H2 policy and expected
revision; only source-free or byte-proven native CDXML is allowed. Actual MOL/CDX backing
refuses. Native attachment is wired with ordered fresh hit/anchor/release/context tests;
actual native verification is separate. Document::replace_fragment atomically retains
fragment/old IDs and unrelated Arcs; global validation excludes only the old fragment. Save complete StoredDocument data; file.dirty uses the saved snapshot, while
source.modified describes original backing. Prepare/complete/finish separates I/O from
revision-checked apply. Count current+saved snapshots and never call a publication error rollback.
Identity floors must survive Undo branches and advance only after successful commits.
Native gestures consume ordered events; press_origin alone loses a drag completed between
frames. Canvas interaction must not advance layout beyond visible error text.
Use Engine::controls() / ui.controls as the authoritative delivered File/Edit/Page bindings;
do not duplicate labels, shortcut tables, preset parameters or enabled rules in the UI.
ui.file is presentation intent, distinct from actual document.open_cdxml/save_cdxml or MOL I/O.
Trusted file plans bind ui_request_id; ui_request_current is captured before their own save
changes state. This returned marker is historical. Native close-Save also requires the
current Engine finish_ui_file success acknowledgment; later format/baseline changes clear
it permanently even after values return. Never acknowledge uncompleted work or release
close from an old worker result. Keep both receipt markers and format/revision/path/epoch
checks. A direct save never fabricates trusted completion state.
Headless engines have no host. Keep request IDs and context guards through native selection,
worker preparation and retry. Reserve/prepare under one engine lock before spawning; only
the matching current request may restore a failed picker. A host acknowledgement does not
prove visible pixels or saved bytes. Keep unaddressed dialog fields/job controls explicit.
Close intent/epoch never belongs in authored history or serialized CDXML. Request/Discard/
Cancel must use document.close; successful actions advance close_epoch without changing
authored revision. A close-specific Save must retain its captured epoch/revision and matching
current receipt; Cancel/new Request must not reuse it. Native complete_close is a trusted
host acknowledgement under the shared Engine mutex, after executor activity, UI file jobs
and pending file actions are absent. Register deferred work/replies under that same mutex;
release response activity after write/flush. Never restore UI-private close approval or let
an accepted file action start after Committed. Headless close intent does not kill stdio.
The user approved crates/render/tests/goldens/single-bond.png for its exact scene/hash only.
Its JSON and README record provenance; external Pillow compares every CPU/GPU RGBA byte.
Do not regenerate that golden without renewed human approval. Other images remain unapproved.
requirements-dev.txt now pins external RDKit/NumPy as well as Pillow. Select an isolated
interpreter with BONDLINE_PYTHON as described in docs/development.md; missing consumers fail
instead of skipping. The RDKit test verifies engine-created CDXML interoperability for
ethane only, not an implemented chemistry engine or complete differential corpus coverage.

## Pick and finish the next task

Take the highest-priority unblocked item in ROADMAP.md and the parity checklist. Read its
source, callers, tests, fixtures, config and exports. Implement the smallest complete slice
through the shared registry and delivered adapters. Check meaningful success/error cases.
Track missing, wired and verified separately; wired is not evidence of correct behavior.
Never call all M0 complete while desktop/GPU, cross-platform or inventory evidence is missing.
Do not spawn subagents unless the user or other applicable instructions explicitly authorize it.

## Commands and commits

When the workspace exists: cargo fmt --all -- --check;
cargo clippy --workspace --all-targets --locked -- -D warnings;
cargo test --workspace --locked; cargo xtask layers; cargo xtask licenses;
cargo xtask corpus; cargo xtask ci. Until implemented, a documented future command is not a pass.
Keep Cargo target directories separate for each worktree. Shared artifacts caused a real
wrong-checkout CI run and stale metadata; runtime workspace discovery does not make shared
binary outputs safe. xtask locates the invoked workspace through Cargo, and ci prints its
root. Verify that root before counting results. Use local Git commits. Small conventional
commits: one command or format feature, with its
tests, provenance and relevant docs. Keep main green; record red evidence without committing
failing main. Commit all intentional project files, never caches, secrets or build outputs.
Do not push or publish without an explicit request. Update STATUS.md with verified state,
actual failures, remaining work, next executable steps and live job handles before stopping.
Currently implemented xtask gates: layers, licenses, corpus and ci. Corpus checks the delivered
CDXML syntax/elemental projection, MOL, ordered SDF and one original explicit CDX scene,
plus pinned file integrity; its output identifies missing foreign/general/application CDX,
chemistry and broader format coverage. New source needs its own hosted evidence; see STATUS.md.
Hosted native-window smoke passes on all three platforms, plus Linux ASan, after the
Linux runtime-package correction. These are window/API checks, not interactive Windows/
Linux chemical-workflow evidence; see docs/ci.md.
Development bundles
and REA reports under target/ are disposable build outputs and must not be committed.
Parser fuzzing is isolated under fuzz/; read its README for pinned toolchain/tool versions,
seed setup and bounded runs. Do not mutate committed fixture directories as fuzz corpora.

## New CDX syntax

CDX Writer creates only new canonical explicit-root documents with ID0 and zero reserved
header bytes. Its raw property payloads and object sequence do not validate chemistry,
property types, references/unique IDs or object containment. Never use it to regenerate an
imported source projection; Document::to_bytes and checked property replacement retain their
separate exact-source contract. Include pending terminators in byte/record/depth budgets
before mutation. Independent original bytes/RDKit, literal boundary tests and fuzz construction
cover this builder; they do not deliver application CDX files.

## CDX header-embedded framing

Default parsing preserves successful28-byte legacy interpretations. Only legacy TrailingData
and an8000 tag at byte22 may select HeaderDocument's22-byte prefix, encoded root and exactly
one distinct FileEnd. Do not turn this into arbitrary trailing-zero acceptance, scanning or
normalization. FileEnd counts toward the record budget and the encoded root toward depth.
header() is22 bytes for that profile; property offsets/ownership use its actual length.
Writer's canonical28-byte profile and all old fixtures/pins stay unchanged. Full-byte source
retention/property edits and original independent/native probes are separate from model/UI
compatibility. A complete legacy tree is inherently indistinguishable from a native file
missing only its file end; keep this ambiguity explicit. Fuzz construction omits root and
FileEnd intentionally to build new canonical syntax, never to reserialize an import.

## Limited CDX model backing

IO's ElementalGraph retains complete borrowed source while mapping only its closed initial
Element/default-C, Single/default-order, fixed16.16-point subset. Its fresh IDs are not
container-scoped source IDs. Selected Z/bounds/CIP0U-or1N/opaque AtomID/unstyled bond-warning
fields now remain source-only with strict widths/owners. FragmentZ is a qualified public
compatibility case. AtomID is never a reference/app-ID alias; its normative uniqueness is
unknown. Other standard metadata refuses; no stereo/H/charge perception or general compatibility.
StoredDocument::from_cdx owns raw Arc plus baseline; is_fully_mapped is always false here.
Export only exact unchanged backing. Changed models, source-free CDX encoding and cross-
format metadata loss must fail; do not regenerate a projection with syntax Writer.
Snapshot equality/accounting includes opaque bytes and baseline. Preview paper is caller
policy, not inferred from BoundingBox. CDX filesystem functions reuse the existing bounded
read/atomic save path. document.open_cdx/save_cdx and the native chooser are now wired
through the shared registry; see verification-cdx-application.md for exact local versus
native evidence. Never bypass shared FilePlan/Completed reservations in those adapters.
Normal CDX fuzzing now exercises projection/snapshot guards; both lock edges add only the
existing internal CDX crate. New native declared closures154/165/220 include that crate.

## Carbon graph display and transient hover

graph_page_scene and registered captures share a bounded GraphScene. Positioned neutral
carbon/Single only; missing H is never inferred, connected unknown-H may be skeletal,
isolated unknown-H/other elements/metadata/orders or unsupported geometry refuse. Authored
0/1/N H renders literally. Source completeness/export guards remain independent and strict.
Atom/label priority, visible clipped ink, five logical canvas units and stable-ID ties are
original policies, not full selection or reference-product default claims. Hover state is
UI-only and excluded from Scene/history/source: revision/Escape/focus/context invalidation
stays clear until a fresh ordered positional event. Context includes available origin,
public native origin and effective DPI. Keep multi-idle-frame regressions and native checks;
do not rearm from cached hover_pos, key release or focus gain alone. No OS pointer polling.


## Owned whole-SDF prerequisite

io::sdf::SdfDocument owns validated complete source and compact immutable record metadata.
Clone shares bytes/index, equality includes every byte and Debug omits content. IDs are
local positive ordinals, not persisted/source chemical IDs or guaranteed identities across
other sources/future edits. read() reparses under existing limits; retained_bytes excludes
borrowed views/allocator headers. StoredContent supplies the separate Engine integration. Preserve every
record/field/MOL/version and never pass a first-record model to a file Save. The normal
SDF fuzz target now checks owned retention/metadata/sharing/accounting as well as old spans.


SdfDocument::with_field_value returns an immutable candidate for an existing ordinal
record/zero-based field occurrence; never find duplicate fields by a property map/name.
Logical no-op shares old bytes/index; authored changed lines use LF, preserving original
terminator/header/DT/padding/other records/MOL bytes. Typed missing targets use not_found;
existing parser/writer error codes and quotas remain. It is not an Engine edit command;
Open/Save already retain whole current/saved/history source. Future record operations
must not make ordinal IDs promises across record deletion/reorder without an identity design.


## SMILES source markers

Aromatic atom/bond markers are notation, not aromaticity or valence perception. The named
fixed-H normalizer must reject any aromatic atom OR bond before integer counts; uppercase
C:C has false atom markers and an explicit Aromatic bond. Keep the writer/model guard intact.
Match the notation BondOrder exhaustively in corpus/fuzz. Ring explicit tokens live in
RingClosure.open_bond_range/close_bond_range even when Bond.bond_range is None; endpoint
aromatic defaults apply only when every applicable explicit token is absent. Original
explicit-ring regression seeds protect this distinction. Fetched corpus remains three
original nonaromatic vectors; six committed fuzz seeds are a different dataset. Never
repin old manifests or drop Root's additive seed/consumer tests when receiving codec work.
