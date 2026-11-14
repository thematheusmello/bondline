# Roadmap

The complete target is an offline desktop and WebAssembly chemical editor with familiar
documented workflows. The initial reference baseline is Professional 26.1; older public
guides are versioned evidence, not proof of complete modern parity.

## M0: foundation — in progress

- [x] Resolve declared runtime/asset license closure for native egui/wgpu on all three platforms.
      Actual Mac Metal drawing/readback verified; hosted Mac/Windows window smoke passes.
      Linux native launch passes after the runtime-package correction below.
- [x] Cargo workspace, pinned toolchain, lints, build-time layer and license gates.
- [ ] Pure document model, named typed JSON commands, transactions and undo/redo tests.
      Page controls/history are delivered; headless elemental graphs now have structural/
      identity/budget tests. Independent C-C creation and transactional drag/history now work;
      connected editing, broader models and a general transaction/job engine remain.
      Conditional close intent now joins the shared registry, with epoch-bound Save and
      native worker/reply shutdown barriers. Shared delivered File/Edit/Page bindings and guarded
      file-presentation requests now exist; broader dialog discovery remains incomplete;
      see docs/research/m0-command-ui-audit.md for the remaining distinction.
- [x] CLI, stdio MCP and opt-in loopback control using that registry.
- [ ] Native UI and CPU/GPU reference canvas with actual launch/render evidence.
      Native Mac drawing/history/recovery and one human-approved golden now pass.
      All three hosted window smokes pass; interactive Windows/Linux workflows and broader rendering
      remain unverified.
- [x] macOS/Windows/Linux CI, licensed corpus tooling, parser fuzz infrastructure.
      Corrected hosted run 43451706668 passes aggregate checks and actual window launches
      on all three platforms plus Linux ASan. Corpus remains its documented CDXML subset;
      passing infrastructure does not establish missing MOL/CDX/chemistry coverage.
- [ ] Complete public-document menu/tool/shortcut inventory; continuation instructions.
      576 stable-ID source-cited control rows now exist, including three provisional
      unversioned Help roles; modern menus, unnamed glyphs and remaining
      context menus still need evidence. Six versioned screen/flow groups and pinned public
      research now guide implementation; the download center requires login and observed
      Signals demo states are not desktop evidence. Do not infer completeness from row counts.

## M1: files — in progress

The V3000 application slice now retains versioned source/header/frame/history and exposes
both versions through the existing two shared MOL commands. Auto Save keeps its associated
version; Save As chooses V2000/V3000 explicitly; Open validates the actual family member.
Independent IO/engine/CLI/stdio/loopback and UI tests plus the corrected local372+3 gate pass.
Actual Mac author/save/reopen/refusal/close and CPU/GPU reply draining now pass at1c53312.
This feature's own hosted checks pass in43531603511 on all three platforms; see
docs/verification-v3000-application.md. Conservative conversion/layout/source-loss refusals
remain. Application SDF workflows and broader mappings still follow; this is not M1 completion.

Current user-directed priority: finish M1, then continue M2–M5. MOLV2000 now has a source-
preserving codec, owned backing/history, atomic filesystem I/O and two shared commands;
engine/CLI/MCP tests use published BIOVIA2026 fields and external RDKit consumers. The native
format choice/Save/reopen/close workflow is verified on Mac; all three hosted aggregates and
native window smokes pass at c879f4c. One CTAB/fragment and layout/metadata guards
are partial interoperability, not completed M1. SDF V2000 now has a source-preserving
container library, ordered duplicate/empty/DT data, a wrapper writer and independent
raw-field/molecular consumers. A fetched original SDF corpus now checks two records and
every ordered field without a lossy property map. Application SDF workflows and wider mappings remain next work. Standalone V3000 parsing,
writing and the version-aware MOL application slice are now integrated; mixed-version SDF
container and legacy RXN slices are being reviewed separately.
Full M0
inventory/dialog/addressability gaps remain tracked while this format work proceeds.


Standalone CDXML source preservation and checked existing-attribute replacement have
original-fixture/Expat and ASan smoke evidence plus one pinned unchanged RDKit corpus
document. A source-backed direct-fragment node geometry view now uses validated model
points; missing coordinates remain absent. Model atom/bond/fragment storage is validated.
Read-only elemental mapping now assigns fresh application IDs, resolves fragment-local
references and retains source backing; an external Expat/Decimal oracle checks the corpus.
Owned application snapshots/history retain that backing; unsynchronized edits cannot
export stale XML. New native elemental CDXML encoding now preserves modeled metadata,
connectivity and explicit Poster canvas extents; an external RDKit consumer opens the
actual single-bond command's output. Shared registry/native CDXML Open/Save/Save As and
Mac safe-close/reopen/edit workflows now have independent file-consumer evidence. Unknown
source passthrough/refusal, bounded atomic saves and background/stale-result guards pass.
The original single-document chooser is partial File coverage; broader imported editing remains.
Native independent C-C/three-H rendering is delivered, with explicit terminal labels.
Source/history now has shared-allocation accounting and oldest-first byte eviction.
A proven generated-file profile now reopens/edits/renders headlessly, retaining source and
exact history; extended/noncanonical files remain guarded. Next: general source-preserving
structural edits, broader native file parity and retention/recovery scenarios;
broader query/label/style mapping, application CDX mapping/editing, MOL/SDF V2000
and V3000, RXN, SMILES and explicit data-loss diagnostics. Every fixture needs a pinned
source, license and SHA-256. Never silently treat unsupported formats as passing.

The first standalone CDX slice now indexes bounded little-endian records, preserves unknown
source bytes and checks opaque property replacements. Original vectors face independent
Python struct and external RDKit consumers. Implicit/explicit roots have specific probe
evidence. A bounded new-tree syntax writer now independently reproduces an original chemical
scene and passes literal lengths/resource checks and external RDKit. General file compatibility
and application import/export remain missing. One original explicit chemical scene is fetched from its real public commit,
SHA-verified and checked against independent struct/RDKit consumers. Foreign/general CDX
corpus and reference/chemical mapping remain M1 work.

## M2: drawing — in progress

The first command creates an independent single C-C fragment with three authored H per
carbon. Native drag creation, exact history, automation and bounded rendering have evidence.
This is partial solid-bond coverage: click creation, existing-atom attachment, connected
chains, snapping, order changes and general atom labels remain missing.

Bonds, rings, chains, labels/abbreviations, charges, isotopes, radicals, wedge/hash bonds,
selection, transforms, hotkeys, undo/redo and style sheets from public journal guidelines.

## M3: chemistry — planned

Valence, aromaticity, stereo/CIP, checking, cleanup, formula/mass, analysis, reactions,
curved arrows, brackets and chemically formatted captions. Compare against independently
run RDKit, reference InChI and OPSIN; document conventions and tolerances before testing.

## M4: publishing — planned

SVG/PDF/PNG, physical-scale output, documented clipboard/Office workflows, atomic saving,
recovery and independent export consumers. Editable embedding needs public evidence.

## M5: depth and distribution — planned

Independent name/structure conversion, original templates, biopolymers/HELM, WebAssembly,
packaging and signed installers. Keep documented advanced capabilities (including naming,
predictions, annotations and file types) visible in the parity backlog until supported.

No milestone claims full commercial parity. Account/cloud/licensing services are not required
for Bondline; unsupported reference integrations remain identified rather than disappearing.

The [May2026 capability audit](docs/research/capability-chart-audit.md) refines existing
milestones with explicit placement-aware paste, dark drawing styles, axial/M/P stereo,
chirality-flag interpretation, hydrogen-bond relationships, named descriptors, additional
format directions and monomer cases. All remain missing with source/platform qualifications.
Optional scientific-service connectors must not become offline creation dependencies.
The [CDX candidate review](docs/research/cdx-corpus-candidate.md) adds a concrete unsupported
framing/provenance question; it does not add a passing corpus file or justify relaxed parsing.
