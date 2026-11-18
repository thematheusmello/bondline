# Neutral SMILES authoring chemistry

2026-11-15. The [source contract](research/aromatic-authoring-contract.md) is implemented
in model-only bondline-chem plus the additive IO normalization adapter. This is a real
integer-bond graph prerequisite for editable paste, not aromaticity perception or a
delivered clipboard workflow. Old normalization and all four converter policies stay exact.

Before implementation, the first2251-byte independent IO test fails with one E0599.
The expanded six-test declaration fails with exactly six E0599 for the missing method.
Its35043 bytes remain SHA256
`7613e06fc8263da84f6debadd81a7d9c35189d69e00a821def4a4aea09ff80cb`.
Nine independently declared direct-core tests fail with exactly one E0432 after adding
real source-semantics types and the crate manifest; no placeholder function was used.
Those10804 bytes remain SHA256
`3897a41e5a26d05ddc3b303e530c66550e4af7f81cc40fa9afe3c891afa22767`.

The new7602-byte actual-Fragment consumer remains SHA256
`bcf4fdd9bef3fb0d63fef2772821a102ef92d3a9ec1ae11547b06768fa81d997`.
Before Rust implementation,33 manual controls pass:20 valid independent graphs and13
deliberate faults. RDKit2026.09.2 checks actual emitted integer graph, exact fixed bonds,
explicit/physical H, isotopes, components, formula, canonical structure, radicals and
stereo. Source aromatic flags are separately recorded from sanitized perception; C4/C8
are not forced through an invented4n+2 rule. Oracle qualification needed no correction.

Root31347 exits0 on the first implemented candidate:9 core and6 IO tests pass. Eighteen
literal IO vectors include benzene/fixed Kekulé, pyridine/furan/pyrrole and backtracking,
physical isotope-H, N substitution, mixed notation, fused/disconnected graphs, fixed
biaryl Single and exocyclic carbonyl. Direct-core tests independently cover nondense IDs,
directions, positions, positive isotopes, exact sidecar/H correspondence, invalid states,
64atom/128bond boundaries and entry/mid-work cancellation. Input owners/IR stay unchanged.
The sp² induced-cycle test avoids incorrectly admitting saturated donor cycles.

Root45008 exits0: warnings-denied core clippy; all43 existing IO integration targets plus
the new IO target and IO library tests total323 passes; warnings-denied matching IO clippy.
This deliberately excludes the separately frozen, still-unimplemented layout declaration.
Rust1.94.1; CARGO_INCREMENTAL=0, BUILD_JOBS=2, DEV/TEST_DEBUG=0; literal pinned
`.cache/oracles/format-python/bin/python` supplied through BONDLINE_PYTHON.

```sh
cargo test -p bondline-chem --test kekule --locked
cargo test -p bondline-io --test smiles_authoring --locked
cargo clippy -p bondline-chem --all-targets --locked -- -D warnings
```

Ignored logs/receipts: target/aromatic-authoring/. Core first-GREEN log1096B SHA256
`495b7fe06bf96e6f0d57cdaafb28807a25215794922ac313b5d3da7ed586e8e0`;
IO first-GREEN1040B `abd205b656372e2eb528c86ecf52b002de4628e06ccf96752a0071b71f4c1619`;
IO regression37332B `de5e5da1236626ba42e666fe856550cbbcacabcc8985cbbdeeb4307358fc877c`.
Lock changes add only the first-party chem package and IO edge; no external version,
font, fixture, corpus pin, old normalizer/oracle, parser, SDK or golden changes.

The new chemistry caps and deterministic matching are original bounded authoring policy.
Full aromatic chemistry, charge/stereo, general layout/import and native clipboard remain
required. No parser/fuzz, native or other-platform execution is claimed. The earlier full
aggregate's MCP replacement-bind failure is not fixed by these targeted passes.
