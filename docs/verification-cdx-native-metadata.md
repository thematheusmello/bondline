# Read-only CDX metadata verification

2026-11-14. Isolated source base `725e92d`, worktree `bondline-mol-v3000`, branch
`feat/cdx-native-metadata`. This slice extends `io::cdx::ElementalGraph`; commands,
native UI, filesystem implementation, model, renderer and dependencies are unchanged.
The source and bounds remain the complete standalone CDX document, not a re-encoded
elemental projection. Existing owned snapshots keep `fully_mapped=false` and retain
their baseline, exact export and stale/cross-format refusal contracts.

## Behavior and independent expectations

The [source registry](clean-room/sources.md#cdx-metadata-retained-by-the-read-only-elemental-view--2026-11-14)
records primary format/type/default evidence and the Fragment-Z qualification.
Only these additions are supported:

| Owner | Property | Check and retained meaning |
|---|---|---|
| Fragment | Z000A, BoundingBox0204 | Exact2/16-byte source-only graphical values; no stacking or page inference |
| Node | Z000A, CIP0437, AtomID044D | Exact2/1/4 bytes; CIP only U0/N1; AtomID remains distinct from all reference/application IDs |
| Bond | Z000A, ChemicalWarning0010, CIP060A | Exact2/1-byte fixed fields; CIP only U0/N1; warning permits only a complete zero-style CDXString prefix |

The old SDK Z owner list omits Fragment; the public graphic-base implementation and
observed native data support that specific compatibility case. This distinction is
retained in provenance. AtomID type/storage is public, but its normative uniqueness,
zero meaning and lifecycle remain unknown. Equal or extreme annotations are retained
without interpreting them as source references. No model metadata fields are added.

The original [323/319-byte fixtures](../crates/io/tests/fixtures/cdx/README.md) were
generated with Python `struct` before production changes. Their analytic positions
are (10.25,-3.5)/(26.25,8.5)pt and source IDs101/102; AtomID70001/90005 deliberately
differs from fresh application IDs1/2. Original signed Z/bounds, zero-style advisory
text, custom bytes/object and symmetric CIP1 cover complete-source retention. The two
framing variants are checked by the pre-existing independent struct decoders. Pinned
external RDKit2026.09.2 actually consumes both unchanged inputs as2C/1 single bond,
CC/C2H6. Those chemistry results are external, not Bondline calculations/authored H.

## Actual TDD evidence

All logs below are ignored files in this worktree's `target/`.

1. Before production edits, `cargo test -p bondline-io --test cdx_metadata
   original_metadata --locked` exited101. Standalone syntax and exact bytes passed,
   then the projection failed `UnsupportedProperty { tag:516, record:5 }` (0204
   Fragment BoundingBox). Log: `cdx-metadata-red.log`. This was an unsupported
   metadata RED, not a syntax/header/API failure.
2. The expanded nine-test metadata matrix also exited101 before implementation;
   log `cdx-metadata-matrix-red.log`. Existing-node Z acceptance separately failed
   `UnsupportedProperty { tag:10, record:4 }`, log `cdx-metadata-expanded-red.log`.
3. The previously unsupported valid node Z literal was moved, with explicit parent
   authorization, into `node_z_order_is_retained_as_source_only_graphical_metadata`.
   It uses the identical `property(0x000A,&1_i16.to_le_bytes())` and original literal
   containment. This is the deliberately changed capability boundary; every other
   old negative, old fixture/hash/pin and approved golden is preserved.
4. Minimal field admission, checked widths, owner guards and source-only validation
   made all15 graph and9 metadata tests pass. Log `cdx-metadata-first-green.log`.

The new matrix covers exact source/both framings, absent/U0/N1 byte distinction,
refused CIP2..12/127/128/255, wrong widths, duplicate fields, wrong semantic owners
including raw/custom/group containers, unsupported node/bond bounds, warning count/
run truncation and valid-but-unsupported styles, opaque warning bytes, AtomID0/MAX/
duplicates and failed AtomID-based references. Other stereo/chemical fields still
refuse. Source Arc lifetime/identity, retained-storage deduplication, different source
metadata with equal models, caller-supplied preview size, baseline restoration and
stale/cross-format refusals are exercised through real `StoredDocument::from_cdx`.

## Reproducible scoped checks

The shell used its own absolute `CARGO_TARGET_DIR` ending in
`bondline-mol-v3000/target`, `CARGO_INCREMENTAL=0`,
`CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0`, and the documented isolated
`BONDLINE_PYTHON` with the pinned RDKit/NumPy/Pillow environment. It did not share
Cargo artifacts with the root checkout.

```sh
cargo test -p bondline-io --test cdx_metadata --test cdx_graph --locked
cargo test -p bondline-io --locked
cargo clippy -p bondline-io --all-targets --locked -- -D warnings
cargo fmt --all -- --check
```

All passed. Full IO result:159 tests,0 failed,0 ignored, plus0 doc tests across25
reported suites (`cdx-metadata-io-tests.log`). Clippy completed without warnings
(`cdx-metadata-clippy.log`); rustfmt check passed (`cdx-metadata-fmt.log`). The two
fixture oracle processes are mandatory, not skipped when RDKit is missing.

## Bounded native-file diagnostic

An ignored headless Rust probe linked the exact own-worktree IO artifact returned
by `cargo build -p bondline-io --locked --message-format=json`. It read the original
licensed runtime save without modifying it:

`target/reference-ui/owned-original-bond-20261114.cdx` in the root checkout,
1196 bytes, SHA-256
`7b2b71338595ad12bc34f46459f8389ec6a63cb453b1cf29b51543717139a73c`.

Borrowed and owned parse produced1 fragment/2 carbons/1 Single bond with absent
authored H. Exact unchanged export was written only to ignored target storage;
both length/hash and every byte equal the input. The owned snapshot remained
unmodified and incompletely mapped. An independent stdlib decoder checked the
primitive CIP1/AtomID values and warning style count0 without reading warning prose.
External RDKit2026.09.2 consumed both original and exported bytes asCC/C2H6. Full
ignored receipt: `target/cdx-metadata-native-evidence.json` in this worktree.

This verifies one specific native metadata profile. No native file/vendor prose,
font, template, code or image is committed. It is not a native UI workflow or broad
compatibility assertion, and it does not imply that a chemical advisory was checked.

## Remaining verification and next slice

The root integration owner must run aggregate/layer/license/corpus gates, cached
normal-graph ASan fuzzing with the two original positive metadata seeds and this
feature's own hosted checks. Those results are not claimed here. No new full fuzz
graph was built in the low-disk worktree. General stereo, styled warnings, other
chemical metadata and broader scenes remain unsupported. A separate command/UI
slice must retain complete stored snapshots and verify failed imports, unchanged
native save/history and edit refusals; a successful parse is not completed M1.
No owned jobs remain running at this handoff.
