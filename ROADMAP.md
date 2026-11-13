# Roadmap

The complete target is an offline desktop and WebAssembly chemical editor with familiar
documented workflows. The initial reference baseline is Professional 26.1; older public
guides are versioned evidence, not proof of complete modern parity.

## M0: foundation — in progress

- [x] Resolve declared runtime/asset license closure for native egui/wgpu on all three platforms.
      Actual macOS Metal launch/readback verified; Windows/Linux runtime checks remain below.
- [x] Cargo workspace, pinned toolchain, lints, build-time layer and license gates.
- [ ] Pure document model, named typed JSON commands, transactions and undo/redo tests.
- [x] CLI, stdio MCP and opt-in loopback control using that registry.
- [ ] Native UI and CPU/GPU reference canvas with actual launch/render evidence.
- [ ] macOS/Windows/Linux CI, licensed corpus tooling, parser fuzz infrastructure.
- [ ] Complete public-document menu/tool/shortcut inventory; continuation instructions.

## M1: files — in progress

Standalone CDXML unchanged source preservation has original-fixture/Expat and ASan smoke
evidence; corpus coverage is next. Then model mapping retaining source backing, lossless CDX records, MOL/SDF V2000
and V3000, RXN, SMILES and explicit data-loss diagnostics. Every fixture needs a pinned
source, license and SHA-256. Never silently treat unsupported formats as passing.

## M2: drawing — planned

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
