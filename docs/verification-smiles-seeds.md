# Original SMILES source seeds

2026-11-14. Three new independently authored MIT OR Apache-2.0 ASCII vectors are in
[the fixture directory](../crates/io/tests/fixtures/smiles/README.md). Its table records
exact17/21/21-byte lengths, SHA-256 identities and literal graph expectations. These are
future parser/fuzz inputs, not an application import/export or chemistry implementation.
No proprietary sample, OpenSMILES example corpus, artwork or parser code is copied.

## Independent consumer before implementation integration

Root consumed all three ACTUAL files using the existing external-only Python3.13.0 /
RDKit2026.09.2 interpreter on macOSarm64. SmilesParserParams explicitly set sanitize,
removeHs, parseName and allowCXSMILES false. The Python API has no skipCleanup field;
default cleanup cannot be disabled by that binding. No sanitization, property-cache
update, canonical string or coordinates were requested.

Independent literal checks passed atom/bond/component counts, atomic numbers and every
isotope/charge/bracket-H count: isotope-chain(3,2,1), two-rings(7,7,2), salt-branch(6,4,2).
Every source is ASCII and has exactly one final LF. Implicit hydrogens/radicals remain
unperceived; these checks do not certify chemical validity or raw-token distinctions
normalized by RDKit. Source literal assertions remain necessary alongside the consumer.

## Actual Git checkout RED/GREEN

Before the new attribute, an isolated ordinary Git repository containing exact copies
was committed and cloned without checkout. A real checkout with core.autocrlf=true
changed all three hashes to CRLF variants. The exact-original assertion failed as
intended, exit1. Git version2.54.0(AppleGit-157); no global config or source fixture changed.
The ignored RED record is target/smiles-fixture-checkout-red.json,850B/SHA
174fd5c8b54eab3855ca82e6e3e4af953bc68879de04eb1e5c86d1a053e6e810.

The narrow `/crates/io/tests/fixtures/smiles/*.smi -text` attribute then preserves all
three original hashes in a separate real clone/checkout with the same autocrlf setting.
The GREEN repository uses the actual project-relative paths and full actual attributes;
git check-attr reports text:unset for each. Exit0. Its ignored980B record SHA is
45488b8b76286e7b4da07ab3f62b7b96705a89124b9cd40ca07e849a09df4267.
Both temporary repositories are removed after their check; only our created files were
involved. This is a Git conversion reproduction on Mac, not Windows-native execution.

Fixture-only changes need relevant identity/consumer/config checks, not an artificial
parser test before that codec exists. The integrated codec must subsequently check
these actual immutable bytes and use COPY-only ignored fuzz mutation corpora. Existing
MOL/SDF/CDX/RXN pins, parser assertions, both locks and approved golden are unchanged.
No dependency/tool install, runtime/reference GUI operation, publication or new golden
approval occurred in this seed check.
