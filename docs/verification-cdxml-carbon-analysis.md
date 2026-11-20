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
