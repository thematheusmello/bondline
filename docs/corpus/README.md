# Pinned public corpus

`cargo xtask corpus` fetches only the reviewed files in manifest.json from their pinned
GitHub commit. The checked-in RDKit BSD-3-Clause notice applies to those test documents.
Each entry records source path, exact byte length, SHA-256 and provenance. Proprietary
sample/template libraries, application binaries, fonts and icons are excluded.

Downloads use an external Python HTTPS process with a 15-second socket timeout and a
read bound of expected length + 1. Rust validates the exact size/hash before staging the
bytes under ignored .cache/corpus. The same verification applies on every cached read;
corrupt cache entries fail rather than being accepted or overwritten. Filenames/source
paths cannot traverse directories; notice/cache paths are checked for symlink escapes.
This is developer tooling. The application does not invoke Python or fetch documents.

`cargo xtask corpus --offline` requires a populated valid cache and never fetches. Use
BONDLINE_PYTHON as described in development.md. No corpus bytes are committed or silently
substituted when a source is inaccessible. Interrupted .part files can be inspected and
removed from this disposable cache before retrying.

Current coverage is deliberately separate:

| Entry | Integrity | Syntax/semantic check | Chemistry |
|---|---|---|---|
| rdkit-chain1.cdxml | Exact 727 bytes and SHA-256 | Byte-preserving round trip; full XML view agrees with Expat; typed elemental projection agrees with independent Expat/Decimal | Calculations missing |
| rdkit-methyl-radical.mol | Exact 170 bytes and SHA-256 | Missing MOL parser; printed as integrity-only | Missing |
| CDX | No reviewed entry fetched yet | Missing parser | Missing |

The command succeeds for this foundation's implemented CDXML coverage and integrity
checks. It explicitly prints missing MOL checks; it does not certify M1 or all-format
round trips. A manifest without any implemented parser coverage is an error. Extending
the corpus requires source/license review, pinned bytes and the matching parser/oracle
checks. Root repository licensing does not authorize copying third-party asset libraries.

The elemental oracle is original developer-only Python under xtask/oracles. It independently
derives fresh IDs, scoped endpoint references, atom metadata and authored bond kinds from
Expat events and Decimal arithmetic. A deliberate model-property mutation is rejected by
its verifier test; another test checks scopes, direction and explicit metadata. It does
not use our parser/writer or compute chemistry. Source bytes are syntax-validated and
bounded before either external oracle runs; neither fetches DTD/entity resources.
The gate now also prints the missing formula/mass/aromaticity/stereo/SMILES/InChI checks.
