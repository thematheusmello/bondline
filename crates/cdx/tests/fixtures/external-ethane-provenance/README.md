# Original ethane through an external CDX writer

`../external-ethane-indigo-1.47.0.cdx` is the exact original positioned ethane
output from the2026-11-15 Bondline external-writer investigation. It is258bytes,
SHA256 `856482269dbd85640b87c11f608104dba72ebd308a686f1333a893965ed560ba`.
Both final zero words remain: implicit Document end254..256 and the distinct
file-end candidate256..258. No trimming, re-encoding or new producer run was used.
The existing flat `fixtures/*.cdx -text` attribute protects its checkout bytes.

The input graphs, generators and authored output are original Bondline test work,
licensed MIT OR Apache-2.0; exact original LICENSE-MIT/LICENSE-APACHE are retained
here. No upstream drawing, template or sample file is used. The two original
generators and complete input-graphs.json are byte-exact copies. Only the stated
ethane output is included, not every graph/output mentioned by those scripts.

Producer: external Apache-2.0 `epam.indigo`1.47.0, recorded macOSarm64 release,
unsigned tag `indigo-1.47.0` at source
`cc2ea85187433c253c0233a13f1859a6a5298d97`. `pins.json` records exact wheel,
metadata, source, input, generator and output identities/URLs. The released
Indigo LICENSE is retained as producer attribution; no producer source/native
library/wheel is shipped or linked. Its font table contains original font-name
strings, not font files. No comprehensive binary dependency audit or reproducible
build attestation is claimed.

`generate_positioned.py` first invokes public layout only to initialize coordinate
presence, then overwrites EVERY atom coordinate with the original literals and
checks public float32 readback. Input positions(0,0,0)/(1.5,0,0) are Indigo units;
the emitted signed16.16 document positions are(0,0)/(45,0)pt. Their relation is
recorded producer behavior, not a normative unit conversion. The pinned external
RDKit reader returns conformer x0/2.25; raw words and reader coordinates have
separate independent checks.

For reproduction, copy these exact scripts/input into a NEW ignored investigation
directory, verify the pinned wheel, create only that directory's separate venv
with `--copies`, and install the verified local wheel with no-index/no-deps and
hash checking. Run the unchanged generators there, never in this fixture folder.
The positioned script requires its separate venv and refuses an existing output
directory. Do not install Indigo into Root's pinned consumer environment or
regenerate this fixture to pass a test. Historical repeats were exact on the
recorded platform; other platforms/versions are unverified.

`original-raw-scan.json` is the untouched original full24-record investigation
table. The NEW `../../external_ethane_oracle.py` independently inventories all
tags, owners, offsets and opaque payloads using stdlib struct, then independently
checks literal CC/C2H6/neutralH3,3/Single/stereo/coordinates with RDKit2026.09.2.
The older scene-specific oracles and all older fixtures remain unchanged.
This fixture is not a fetched corpus admission, application/model compatibility
claim or permission for arbitrary trailing zeros. The declared canonical-header
profile and successful-legacy priority are separately tested.

See [the investigation and access limits](../../../../../docs/research/cdx-external-writer-probe.md)
for the original recipe, recorded package/source evidence and provenance limits.
