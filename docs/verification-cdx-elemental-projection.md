# Read-only CDX elemental projection

Implemented in the isolated `feat/cdx-elemental-projection` checkout based on `c351bb8`,
2026-11-14. This is the first bounded IO projection, not application CDX Open/Save,
native editing, complete chemical interpretation, or general scene rendering.

## Delivered API and source authority

`bondline_io::cdx::ElementalGraph::parse` retains the standalone borrowed CDX `Document`.
`source()` exposes its complete record indices, ranges, parent identities and opaque source;
`as_bytes()` borrows the original input and `to_bytes()` copies that unchanged source.
There is no model-to-CDX encoder or editing method. `fragments()` exposes validated model
fragments with fresh dense fragment/atom/bond IDs, independent of source IDs and scoped
source-ID reuse. Source object hierarchy remains in the backing, not in a regenerated tree.

The public `MappingError::code()` distinguishes `resource_limit`, `unsupported_content`
and `invalid_document`. Relevant property/object/reference errors include source record
indices; callers can inspect corresponding ranges in the standalone source. Codes are
Bondline's adapter policy, not vendor behavior. No raw bytes are added to Debug output.

The dependency change is one internal edge, `bondline-io -> bondline-cdx`; it adds no
external package/version or asset. Standalone parsers still depend on no model/GUI crate.
Only IO's module export and root lock's existing-package edge change. The integrating owner
reconciles the isolated fuzz lock; no external versions are updated to make tests pass.

## Exact supported profile

| Area | Supported / refused |
|---|---|
| Layout | Zero or one direct document page; fragments under that page/nested groups; nodes/bonds directly in fragments. Other standard object types, additional pages and unsupported containment refuse |
| Identity | Nonzero UINT32 source IDs unique per containing object; repeated zero IDs allowed when unreferenced; endpoints cannot reference zero. Dense application IDs are deterministic per supported traversal |
| Node type / element | Absent type defaults to Element, explicit type 1 only. Absent element defaults to carbon; exact signed INT16 values must fit the existing 1..118 model catalog |
| Geometry | Optional property 0200, exactly 8 bytes, signed INT32 y then x divided by 65536. No rounding, axis reversal, inferred origin or automatic cleanup |
| Bond | Exact UINT32 begin/end references to atoms in the same fragment; forward references and authored direction preserved. Absent order or explicit UINT16 bitmask 1 only |
| Chemical metadata | Missing H stays `None`; no valence inference. Charge/isotope/radical/H properties, labels, queries, stereo and all other standard node/bond/fragment properties refuse, even when a supplied value is zero |
| Source extras | Root/page/group metadata and user-defined properties/custom objects stay opaque in source. Known chemical objects cannot hide under custom/unsupported containers; mapped properties on wrong owners refuse |
| Model invariants | Existing self-bond/parallel-connection and graph-size policies apply. These are model restrictions, not a claim that every valid CDX graph fits this subset |

The native 1196-byte observation is not an admitted fixture or verified projection. Its
fragment 000A/0204 and node 000A/0437/044D properties are outside this initial whitelist;
the adapter must refuse rather than silently remove their meaning. A subsequent sourced
metadata slice may admit proven nonchemical properties or known safe values. It must not
broaden this decoder merely to get that one file through. Header/document/file-end repair
is a separate standalone-parser workstream; this adapter uses record ownership rather than
branching on framing enum values or interpreting terminators itself.

No command, CLI/MCP binding, StoredDocument variant, filesystem function or UI was added.
The immediate owned-snapshot follow-up will add its private preview helper alongside its
real caller, with caller-supplied page size and no paper inference from bounding boxes.
It must always mark this source incomplete, preserve source/baseline accounting, and refuse
stale model export or lossy cross-format conversion until complete mapping is proven.

## Independent expectations and fixtures

The unchanged original explicit-root fixture is
`crates/cdx/tests/fixtures/original-ethane-explicit.cdx`, 65750 bytes, SHA-256
`63ef1a74133e45de915d5fd25b2ca3321a95dbe5114873a5a59f2f45217000d2`.
Its independently authored Python generator/stdlib record decoder and external RDKit
preexist this adapter. Literal expected points are `(10.25,-3.5)` and `(26.25,8.5)`, with
two carbon nodes and one directed single-bond connection. The 65KiB extended opaque property
and custom objects remain byte-exact. The separate original opaque-header fixture is reused
unchanged. No downloaded or proprietary runtime metadata bytes were committed.

Tests construct additional small literal records from published field widths, not from a
Bondline writer. They cover absent defaults, explicit Element type, reversed/forward refs,
signed 32-bit limits and the smallest fixed-point quantum, source IDs 0/UINT32_MAX, duplicate/
cross-fragment/invalid references, duplicate properties, invalid sizes/elements, unsupported
metadata/containers, noncanonical extended lengths, and inherited graph validation.

`cdx_projection_oracle.py` reuses the existing independent stdlib CDX decoder and invokes
pinned external RDKit 2026.09.2 on the unchanged original and absent-default vector. The
stdlib decoder checks point units/order independently of Rust. RDKit verifies the molecule,
two carbons/one single bond, external `CC` and `C2H6`; these are external consumer results,
not Bondline formula/valence implementation or an explicit hydrogen count.

The exact public SDK property/default/type sources and the inaccessible separate
CDXPoint2D link are recorded in [the source register](clean-room/sources.md#read-only-cdx-elemental-projection--2026-11-14).
The coordinate definition itself is available in the public coordinate-types page.

## Actual TDD and checks

Commands ran from this checkout with its absolute `target/`, `CARGO_INCREMENTAL=0`,
dev/test debug info 0, and existing root `.cache/oracles/format-python/bin/python` selected
through the absolute `BONDLINE_PYTHON` path. It reports RDKit 2026.09.2; no environment or
dependency installation was performed.

| Step | Actual result / ignored log |
|---|---|
| First fixture test before API/path dependency | RED, exit 101: unresolved `bondline_io::cdx`; `target/cdx-projection-red.log` |
| Minimal projection and internal edge | GREEN, original fixture test passes; `target/cdx-projection-first-green.log` |
| Wrong-container chemical-property regression | RED: root/page/group core properties were silently ignored; `target/cdx-projection-scope-red.log` |
| Shared owner validation | GREEN for fixture/regression; `target/cdx-projection-scope-green.log` |
| Public error-code contract before method | RED, exit 101: missing `MappingError::code`; `target/cdx-projection-codes-red.log` |
| Final CDX integration tests | GREEN, 14 tests including mandatory external consumers; `target/cdx-projection-final-green.log` |
| `cargo clippy -p bondline-io --all-targets --locked -- -D warnings` | GREEN; `target/cdx-projection-clippy.log` |
| Final `cargo test -p bondline-io --locked` | GREEN, 137 tests; `target/cdx-projection-io-final.log` |
| `cargo fmt -p bondline-io -- --check` | GREEN, no formatting differences |

The integrating owner runs the receiving-root aggregate, layers/licenses, hosted platform
checks, adds this mapping invocation to the existing CDX fuzz target and observes the bounded
fuzz result. No new full fuzz build was attempted under the low-disk constraint. GUI and
full native-file projection/editing remain unverified; this report cannot complete M1.
