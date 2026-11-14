# Status

2026-11-14: persistent M0–M5 goal ACTIVE; no milestone is complete.
This isolated feat/cdx-elemental-projection source is based onc351bb8. It adds a bounded
read-only elemental CDX view retaining complete borrowed source, dense application IDs,
fragment-scoped references and exact fixed-point geometry. Initial chemical support is
Element/default-carbon nodes and Single/default-order bonds without other standard node/
fragment/bond metadata. Source labels, H/charge/isotope/stereo and native Z/Bounds/CIP/AtomID
remain explicitly unsupported; no native/application compatibility claim follows.

Actual missing-API, wrong-owner-property and error-code RED/GREEN evidence is in
 docs/verification-cdx-elemental-projection.md. Fourteen new CDX tests, full137 IO tests,
all-target IO clippy-Dwarnings and fmt pass. Independent raw struct/RDKit and existing
original fixture hashes stay intact. The only dependency change is the internal IO→CDX
edge in manifest/rootlock. Public MappingError::code is ready. No private preview helper,
StoredDocument/native I/O/registry/UI, fuzzlock or fuzz-graph change was made in this slice.
No GUI/build job remains; own warm target, incremental0/debug0. No caches were deleted.

Root target/cdx-elemental-projection-handoff.json contains8 frozen file/log hashes.
Next: independent read-only review; integrate with current header-framing source, add
immediate owned snapshot/caller and private preview helper, normal fuzz invocation/lock edge,
then receiving aggregate/hosted checks. Root owns Git and that integration. Root's current
Snapshot tests are RED outside this checkout; do not claim current main has application
CDX import. Native metadata and broader mapping remain follow-up format features.
