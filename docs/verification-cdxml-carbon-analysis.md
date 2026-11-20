# CDXML carbon analysis without coordinates

The public RDKit four-carbon chain and an original reordered drawing both give
C4H10. RDKit 2026.09.2 independently reads each complete XML file, counts four
drawn atoms and calculates molecular weight 58.124 and monoisotopic mass
58.07825032 Da. Bondline retains all original hydrogen-field absences and the
absence of every atom position. The original draws one explicit attached-H count
and the public file draws none.

The source gate requires a complete elemental graph, explicit supported bond
orders and chemically irrelevant checked resources. The command tests include
double and triple bonds, inconsistent hydrogen counts, query and stereo fields,
opaque children, source drift and a refused conversion. The first command run
returned unsupported_source for the positive drawing. The first App-frame run
showed the CDX hydrogen explanation on this CDXML document. Both regressions
now pass. [Source profile](research/cdxml-carbon-analysis.md).

## Replay

Set `BONDLINE_PYTHON` to the RDKit oracle interpreter and fetch the pinned
corpus. Run the following from the repository root.

```sh
cargo test --locked -p bondline-commands --test analysis_cdxml_carbon
cargo test --locked -p bondline-ui --lib analysis_
cargo test --locked -p bondline-cli --test analysis_cdx
cargo build --locked -p bondline-cli
"$BONDLINE_PYTHON" -B xtask/oracles/cdxml_carbon.py target/debug/bondline-cli --pinned .cache/corpus/rdkit-chain1.cdxml
```

The CLI compares a new Unicode-named save and its reopened source against the
original bytes and both independent mass estimates. Requests for atom selection
and CPU canvas capture report `unsupported_content`.

On macOS the authenticated loopback opened both files in an isolated background
native app. The actual Analysis panel's accessibility tree reported C4H10,
58.124, 58.078250 Da and three or four CDXML valence defaults as appropriate,
both before and after Unicode Save As and reopen. An opaque stereo field produced
the unsupported-source message. The host approved Close and its process exited.
The test used no desktop input. Receipt SHA-256 was
`e32947ab757563774ed6f7d5ee281d75594e91a194d10012efeeb73ce75f43a1`.
Interactive Windows, Linux and web panel execution for this increment remains pending.

Initialized stdio MCP independently opened the pinned file, queried the complete
properties, checked both refused operations and repeated them on a byte-identical
Unicode-named save. The result SHA-256 was
`4238447c953ca2b021e0b58ed31c34bf239d5c33e15d76305500a205687d37fd`.
The corpus gate verified three matching complete drawings and five unsupported
sources. Two individual SDF mol-block checks matched, and three SMILES record
attempts remained unavailable. All 44 xtask tests and its warnings-denied
Clippy check passed. [Corpus inventory](corpus/README.md).

The existing CDXML parser fuzz target checks carbon valence conservation,
written/omitted H identity and refusal of defaults after source-model
detachment. An AddressSanitizer run with the original, the pinned file,
double/triple and omitted-H variants, and stereo/DTD/valence negatives
completed 10,000 inputs in eight seconds (seed 20261124). The eight immutable
seed digests survived. The run reached a 727-byte mutation limit and 461 MiB
reported RSS. Log SHA-256 was
`2b96f6b29abd7a9afec83466d257eb3681adb73bf83ffb85e82af44dc5d86a2d`.

With the elemental analysis table enabled, an independent RDKit explicit-H
read calculates H 17.34% and C 82.66% by weight. The CLI compares unrounded
values, using both the pinned file and original fixture before and after
Unicode Save. The egui frame and background macOS accessibility tree contain
the two formatted rows; initialized MCP returned the same contributions and
percentages. The integrated native receipt SHA-256 is
`457387829e10bbc4c72e5d82e9a28e2d86de5e01aee3938b58e2ac3b9d1883ac`,
and the MCP receipt SHA-256 is
`04c3c72cc1e3f4f0f6e9af04f2239545f90ab5a9bfa90850daaf4abe204a4a25`.
