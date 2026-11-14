# Status

2026-11-14: persistent M0–M5 goal active; no milestone is complete.
Public repository: https://github.com/thematheusmello/bondline .

This isolated checkout is feat/rxn-v2000 from3b1fada. It adds a bounded source-preserving
legacy $RXN container codec, explicit ordered reactant/product/reagent roles and a preserving
new-wrapper writer. Embedded MOL payloads are independently validated V2000 or V3000 syntax;
reaction-model editing, native/registry Open/Save and V3000 reaction wrappers remain absent.
Mixed MOL payloads and six-column zero-reagent counts are observed consumer extensions,
separate from the published CTfile legacy layout. Complete bytes, raw maps/metadata and
source spans remain authoritative; external RDKit reaction semantics are not raw fidelity.

Nine new independent tests plus all107 existing IO tests pass. RED/GREEN, boundary tests,
independent generated inputs, role/count/EOF checks and fresh ASAN10000 are recorded in
 docs/verification.md. Read-only review found no confirmed defect in the declared profile.
The actual own-checkout cargo xtask ci passes373 workspace plus3 macOS bridge tests, fmt,
all-target clippy, layers, native license/assets and its inherited two-entry corpus. Log
 target/rxn-full-ci.log SHA1b79e3116cc8e49f625e46d8b4cab6564cde40f5d3ee6a1f925f917b14bf72b4.
Session56720 is terminal0. This is not root's later five-file corpus or hosted feature proof.
Production concat5144c6dabd97d048307cc954ad0459e958ba87aafb977119be3072b430477a5e;
locks/runtime dependencies and approved golden are unchanged. No UI/native run was performed.

Next: publish the atomic codec, observe its own hosted checks and integrate into current
root after the mixed-SDF merge, preserving newer four-manifest/CDX-positive fuzz rules.
Run the actual combined root gate before main advances; do not replace entire older config
files. Root alone manages Git. No local job remains. Keep own absolute Cargo target and
CARGO_INCREMENTAL=0. Read root STATUS for actual installed-reference GUI ownership and
research maps. M0 inventory/dialogs, application SDF/CDX/RXN, SMILES, chemistry and M2–M5
remain unfinished. No vendor binaries/assets/code were copied into the product.
