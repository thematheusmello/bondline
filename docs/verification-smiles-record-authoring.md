# Complete-source SMILES record authoring

2026-11-16. The [contract](research/smiles-record-authoring-contract.md) separates
notation-span replacement, retained drawing projection and command/native integration.
Public OpenSMILES file semantics and pinned PhotoCraft ownership/job distinctions are
sources; selection, geometry, identity and lifecycle policies are original Bondline work.

## Notation-only file transaction

`SmilesFileDocument::with_record_notation` replaces exactly one existing nonempty token.
It preserves every other byte, including duplicate records, opaque NUL/non-UTF8 data,
ignored lines, separators and LF/CRLF/final EOF. The complete candidate is reparsed under
the existing limits before return. An exact no-op shares source and index. Cancellation
wins at entry; missing records, invalid notation and resource limits keep typed errors
and absolute source context. Standalone notation's optional terminal EOL is deliberately
refused here so a replacement cannot insert a record or absorb application data.

The first independent 1187-byte declaration was frozen as SHA256
`2f31b5a2f153fb8aed8e70f0295a825dadea5dab013d9f70aac1d0980b2c21cc`.
The initial combined IO run exits 101 with the intended missing setter, selection and
constructor API errors only; see `target/smiles-record-authoring/io-initial-red.log`.
The six-case expansion is 14276 bytes, SHA256
`43bf783a8206398d93b4eab91829da465317da0d5ceaea14e0810c4dbcfdba4b`.
First receiving 4425 exits 0: six tests pass, including literal spans and unchanged raw
scanner/RDKit consumers over actual modified bytes. An in-progress mapped-writer field
warns as unused; this isolated run is not a warnings-denied or full-workspace pass.

One test-helper portability correction uses the existing Python override/default
convention; it changes no input, assertion, script or pinned-version check. Its 14336-byte
declaration is SHA256 `6a9bdfa78ca04f11c95b9a699b4033bc5710f081d281adf5559bbe1856061eea`.
Receiving 80221 passes the same six cases plus seven projection cases without warnings.
Final receiving rustfmt is mechanical; the original declarations remain archived under
ignored `target/smiles-record-authoring/` with the source and helper-diff receipts.

```sh
cargo test -p bondline-io --test smiles_file_edit --locked
```

Boundary cases cover 65536/65537 token bytes, a complete 64 MiB file whose replacement
would exceed the file cap, record ordinal 65536, invalid or missing targets, cancellation
before no-op and exact source/index identity. Existing parsers, limits and independent
consumers remain unchanged. This file transaction alone grants no Drawing, model-edit,
native selection or Save acknowledgment authority.

## Retained drawing projection

`StoredDocument::from_smiles_records` accepts an explicit record or all records and owns
the complete original/current file, original/current model baselines and source maps.
One aggregate chemistry pass and component layout precede splitting back into one Fragment
per record. The shared 64-atom/128-bond and geometry/ink budgets apply to the whole selection.
Unselected accepted records may be unprojectable; selected failures refuse atomically.

The first 1313-byte constructor declaration, SHA256
`d37cb1e1da5b811401917665dbe77c1534d272811938991a7fae8acad83531a7`, reaches the intended
missing selection/constructor errors before implementation. The seven-case expansion is
14626 bytes, SHA256 `d31f58b4ee9e5b8416b3c4cea37d6013549f549d790e56d5d5a1bec0f7101900`.
First receiving 80221 passes all seven unchanged plus the six codec cases without warnings.

The literal 53-byte three-record case changes only its middle `C1.N.C1` to
`[CH2]=[CH2].[NH3]`, producing 63 bytes. Model atom order remains C3/N4/C5 and the bond
remains directed 3→5, while emitted token order is 3/5/4. Absolute later ranges move by
ten bytes; original metadata, geometry, identities and unrelated Fragment Arcs survive.
The unchanged raw scanner and external Fragment consumer verify actual output. Further
cases cover exact no-op sharing, all/selected/empty/missing cases, 64/65 atoms and 129 bonds,
entry cancellation, detached geometry and non-H/order replacement refusals.

`is_fully_mapped` stays false. Display and complete-source Save have a separate validated
current-projection capability; detached `model_mut` changes cannot acquire it. Only H and
integer order changes can synchronize a record. Cross-format output and unencoded geometry
remain refused. The old byte-only writer output and normalizer profiles remain exact.
One accountant deduplicates original/current files, maps, models and raw source owners.

```sh
cargo test -p bondline-io --test smiles_record_projection --locked
```

The seven IO product pins remain exact from the implementation handoff through integrated
receiving. Five independent test declarations reproduce their committed form using only
rustfmt; no expected values or assertions changed. This library slice does not deliver
native record selection, drawing navigation or editing controls.
