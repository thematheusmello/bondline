# CDX labels with an omitted Element

The [source contract](research/cdx-implicit-element.md) uses the documented
carbon default and retains the requirement for an explicit H count. The
[standalone insertion batch](verification-cdx-property-insertions.md)
supports the same exact owner, framing and total output checks as the
original replacement path.

## Independent binaries and history

The [original witnesses](../crates/io/tests/fixtures/cdx-implicit-element/SOURCES.md)
reproduce byte for byte with the independent struct and FontTools generator.
The 496-byte connected case inserts a signed Si record and becomes 503
bytes. Its Single and Triple bonds keep their directed endpoints. The
boxed cases cover a hydrogen-only edit that leaves Element omitted and a
nitrogen insertion with scoped Node and Text IDs reused in another
fragment. RDKit verifies all six unique files. The missing-H file keeps
its exact bytes and data view with unresolved H and without edit
authority.

Before the producer, the IO run passed five cases and rejected the two
new admission cases. The command RED reported the old explicit Element
requirement. The App RED could not open the form under the old gate.
After the change, the seven IO cases and shared command check pass.
An unchanged C/H request shares the original snapshot without adding
bytes. An H-only edit retains omitted Element. A subsequent edit after Si
insertion can author explicit carbon, while Undo restores the original
omission and Redo restores exact Si bytes.

## Desktop workflow and adapters

The App harness drives the existing Alt-click form, unchanged Apply,
real change, keyboard history and Unicode Save/reopening. CLI and
authenticated loopback both compare full models, saved bytes and
reopened captures against the independent witness. Initialized stdio
MCP verifies omission and history, then saves the 503-byte CDX.
RDKit reads that actual file with formula HNSi. CPU and Metal produce
matching 1024×1024 captures with SHA-256
`d26756d3eaff09f5d593012a37020fd27569af8c72e00124d105318bc5b00455`.

Physical macOS input remains unverified. The signed review was prepared
while the machine was receiving hardware input. Synthetic App density
and independent render evidence are scoped separately from live platform
interaction and human golden approval.

The IO crate type-checks on Windows MSVC, Linux GNU and WebAssembly.
Those runtime workflows remain open.

## Sanitizer and receipts

The parser and binary/label edit harness completed 10,000 ASan runs in
one second with seed 130705204, limit 65,536 and 205 unchanged originals.
The largest recorded new or reduced mutation was 870 bytes. Independent
standalone tests check exact 64 MiB and record boundaries.

Receipts are in `target/cdx-resources/next-text/implicit-element`.
`fuzz.log` SHA-256 is
`00182057717dea1383d28e6abe52ee507075e113ef15a5bce0447f18f0669dab`.
`stdio-actual.json` SHA-256 is
`ba036053b737a152047dfc2aab1d6df11e0f331973d0f7c44011e8cde1fe3b93`.
The actual CLI's 49 registry entries retain SHA-256
`3cfd51a6e72891137cf0cc56fd756ff33652cfc685e5aaf877634b259cc41e4f`.

## Final local gate

The exact source and tests at `20f3644` passed `cargo xtask ci` with the
pinned `BONDLINE_PYTHON`. The workspace passed **2,254 tests** with no
failures and one existing ignored case across 334 reports. AccessKit
passed three tests. Fmt, warnings-denied Clippy, layers, runtime license
closure, seven assets and the pinned eight-file corpus passed.
The historical MCP listener failure remains unexplained.

The 284,300-byte `ci.log` SHA-256 is
`932aa7ff0895516937f3b36c0708ce9c43e01e4e7e5d834e832a30bb9eb113e0`.
All owned execution jobs exited. The prepared desktop review was never
launched. GitHub Actions remained disabled.
