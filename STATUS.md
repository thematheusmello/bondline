# Status

2026-11-14: persistent M0–M5 goal active; no milestone is complete.
Public repository: https://github.com/thematheusmello/bondline .

This checkout is feat/sdf-v3000, based on accepted application docs2a69de2/source1c53312.
It adds source-preserving mixed V2000/V3000 SDF library records, actual per-record versions
and a generic preserving writer. write_v2000 remains strict; no application collection or
first-record-only import is exposed. Seven new independent tests, literal/raw-span and
external RDKit/version-switched writer checks pass; existing V2000 oracles stay unchanged.
DT-only supplier loss and duplicate property-map limits remain documented separately.

TDD, scoped clippy/fmt and fresh ASAN10000 are recorded in docs/verification.md. Read-only
review confirmed no declared-profile code defect; current capability docs were corrected.
The actual own-checkout cargo xtask ci passes379 workspace plus3 macOS bridge tests, fmt,
all-target clippy, layers, native license/assets and the inherited two-entry corpus. Log
 target/sdf-v3000-full-ci.log SHA77da79e40b19347c86782478f29d749e9e0a339a7e4ea5a23b678ed3bf67306d.
This is not root's newer five-file/four-bundle corpus or a hosted SDF feature result.
Production concat eded429d55b8dd4ab0b520f6e432b859a33269aceaf6ce4aa40595f8dce9f75d;
both locks and runtime dependency closures stay unchanged. No native UI change or run.

Next: publish this atomic source, observe its own hosted jobs and integrate into root85bf5eb,
retaining root's CDX writer/corpus/fuzz rules. Run the actual combined aggregate before main.
Root alone manages Git. All local SDF jobs ended; no live handles. Keep this checkout's
absolute Cargo target and CARGO_INCREMENTAL=0. Source-reviewed raw REA/native research remains
in ignored root target/reference-ui, outside this feature. Read root STATUS for its live
research agents, installed-app ownership and full project continuation. M1 application
SDF/CDX/RXN/SMILES, broader mappings, M0 inventory/dialogs and M2–M5 remain unfinished.
The approved single-bond golden and all original fixtures/pins are unchanged.
