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
