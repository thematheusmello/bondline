# Roadmap

The complete target is an offline desktop and WebAssembly chemical editor with familiar
documented workflows. The initial reference baseline is Professional 26.1; older public
guides are versioned evidence, not proof of complete modern parity.

## M0: foundation — in progress

- [x] Resolve declared runtime/asset license closure for native egui/wgpu on all three platforms.
      Actual Mac Metal drawing/readback verified; hosted Mac/Windows window smoke passes.
      Linux native launch needs the runtime-package correction verified below.
- [x] Cargo workspace, pinned toolchain, lints, build-time layer and license gates.
- [ ] Pure document model, named typed JSON commands, transactions and undo/redo tests.
      Page controls/history are delivered; headless elemental graphs now have structural/
      identity/budget tests. Independent C-C creation and transactional drag/history now work;
      connected editing, broader models and a general transaction/job engine remain.
- [x] CLI, stdio MCP and opt-in loopback control using that registry.
- [ ] Native UI and CPU/GPU reference canvas with actual launch/render evidence.
      Native Mac drawing/history/recovery and one human-approved golden now pass; native
      Windows window smoke passes; interactive Windows/Linux workflows and broader rendering
      remain unverified.
- [ ] macOS/Windows/Linux CI, licensed corpus tooling, parser fuzz infrastructure.
      First hosted run passes aggregate checks on all three platforms and Linux ASan; Mac/
      Windows window launches pass. Linux window launch needs its dependency fix rerun.
- [ ] Complete public-document menu/tool/shortcut inventory; continuation instructions.
      573 stable-ID source-cited rows now exist; modern menus, unnamed glyphs and remaining
      context menus still need evidence. Six versioned screen/flow groups and pinned public
      research now guide implementation; the download center requires login and observed
      Signals demo states are not desktop evidence. Do not infer completeness from row counts.

## M1: files — in progress

Standalone CDXML source preservation and checked existing-attribute replacement have
original-fixture/Expat and ASan smoke evidence plus one pinned unchanged RDKit corpus
document. A source-backed direct-fragment node geometry view now uses validated model
points; missing coordinates remain absent. Model atom/bond/fragment storage is validated.
Read-only elemental mapping now assigns fresh application IDs, resolves fragment-local
references and retains source backing; an external Expat/Decimal oracle checks the corpus.
Owned application snapshots/history retain that backing; unsynchronized edits cannot
export stale XML. New native elemental CDXML encoding now preserves modeled metadata,
connectivity and explicit Poster canvas extents; an external RDKit consumer opens the
actual single-bond command's output. Imported structural edits and file workflows remain.
Native independent C-C/three-H rendering is delivered, with explicit terminal labels.
Source/history now has shared-allocation accounting and oldest-first byte eviction.
A proven generated-file profile now reopens/edits/renders headlessly, retaining source and
exact history; extended/noncanonical files remain guarded. Next: native workspace wiring,
general source-preserving edits and registry open/save with failed-import/save retention;
broader query/label/style mapping, lossless CDX records, MOL/SDF V2000
and V3000, RXN, SMILES and explicit data-loss diagnostics. Every fixture needs a pinned
source, license and SHA-256. Never silently treat unsupported formats as passing.

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
