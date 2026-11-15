# Immutable SDF record removal

This library slice adds `SdfDocument::without_record(RecordId)` to the existing
complete-source owner. It removes the selected validated record's full byte range,
including its original delimiter and line ending, then invokes the unchanged parser.
Every retained byte stays exact. The new index uses positive source-local ordinal IDs;
an old ordinal is not an identity across removal. No model projection or writer is used.

The method resolves the target before checking the last-record rule. An absent ordinal
returns the existing `MissingRecord`/`not_found`; removing a valid sole record returns
the existing `Empty`/`invalid_sdf`. Neither changes the original source or index. The new
source and index are independently owned; clones of the result share those allocations.
Existing retained-byte accounting needs no new field or accounting rule.

## Public basis and scope

The [official BIOVIA Chemistry 2026 CTfile Formats](https://www.3ds.com/assets/invest/2026-01/biovia_chemistry_2026_ctfile_formats.pdf),
chapter 10, printed 97–99 (PDF indexes 100–102), defines molecular records and data framing.
Printed 97 explicitly describes one or more compounds and permits V2000, V3000 or a
combination. That supplies the format basis for refusing the final removal. The cached
PDF was rehashed at 1,580,820 bytes, SHA-256
`959f2bbf5487e00fa7535ea7ee36f7ce04f39ea72920844641952276de146c94`;
its existing extracted text was reread. No vendor example, code or asset was copied.
The [existing source register](clean-room/sources.md#mixed-sdfile-molecular-versions-2026-11-14)
records the same pin and parser-profile decisions.

Exact byte splicing, immutable candidates, regenerated source-local ordinals and target
error priority are original Bondline API choices. The existing ASCII, EOF, delimiter
prefix, padding and resource profiles remain unchanged. This API is synchronous bounded
codec work; it adds no cancellation API, registry command, filesystem publication,
history entry, native control or record-removal UI. Those integration steps remain work.

## Independent expectations

The seven tests in `crates/io/tests/sdf_record_remove.rs` use the unchanged original
MIT OR Apache-2.0 fixtures, with literal byte spans established independently of the
reader. They cover:

- The exact 315-byte suffix after deleting the first record of the 777-byte mixed source.
  The unchanged external raw scanner reads the resulting ranges/ordered data, and the
  unchanged per-MOL RDKit consumer reads actual retained bytes as ethane, C2H6, with the
  literal single-bond graph.
- First/middle/final removal from three original records, exact retained order and
  reindexed spans; duplicate occurrences, raw DT digits, empty/multiline/spaces values,
  padding, the suffixed delimiter and the blank molecular title remain exact. The
  unchanged raw scanner and its per-MOL consumer inspect each actual resulting source.
- Removal of the final mixed record, retaining every V3000/data byte and independently
  checking the actual 344-byte MOL graph and source coordinates with the unchanged
  V3000 consumer.
- CRLF retention, suffixed delimiter and final delimiter at EOF without a line ending.
- Missing/zero-ID behavior, missing-before-last error priority, sole-record refusal,
  immutable source/index pointers, clone sharing, retained-byte accounting and release
  of the old source allocation after its owner drops.
- The existing 1024-record and 4096-byte delimiter-line boundaries without source growth.

The full SDF supplier is not used as a raw field oracle: its already documented DT-only
and duplicate-property limitations remain visible in the existing tests. No fixture,
oracle, parser, writer, dependency, lockfile, font or golden was changed by this slice.
This is not broader chemical perception or complete format/workflow parity.

## RED, setup corrections and GREEN

The initial 3,129-byte test, SHA
`633521f5627a0689e294349597f2b9a4ac92de8b6e58749d1cb6f8a003437b01`,
was frozen before production changes. The first command exited 101 with E0599: no method
named `without_record`. This is a missing-API compile RED, not runtime behavior evidence.

Two new-test setup corrections are preserved separately from that RED:

- Before runtime, direct fixture inspection corrected its expected header name from
  NOTE to the actual ORDINARY. The original version remains archived. No oracle changed.
- The expanded seven first ran 6/1 because a new CRLF MOL-end expectation was 291.
  Literal `M  END\r\n` scanning and the unchanged external scanner both establish 290;
  the 282-byte source has 8 LF. Only 291→290 changed, with the 15,825-byte failing source,
  failed log and independent proof retained. No production or other assertion changed.

The minimal production method then passed the first actual independent case. The final
seven pass 7/0. The complete IO package passes 291/0/0 across 41 result lines;
all-target IO clippy with warnings denied and workspace formatting also exit 0. Their
exact logs and terminal sessions are recorded in the ignored final receipt. All old
`document.rs` bytes are identical when the single additive method is removed; protected
fixtures/oracles/locks/golden identities are checked against the pre-edit manifest.

Reproduce with the pinned development interpreter selected as `BONDLINE_PYTHON`:

```sh
CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
  cargo +1.94.1 test -p bondline-io --test sdf_record_remove --locked
CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
  cargo +1.94.1 test -p bondline-io --locked
CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
  cargo +1.94.1 clippy -p bondline-io --all-targets --locked -- -D warnings
cargo +1.94.1 fmt --all -- --check
```

Use a separate absolute `CARGO_TARGET_DIR` for each checkout. These checks ran on the
available macOS host; no new Windows/Linux execution, native UI or hosted Actions run
is claimed. GitHub Actions stayed disabled and no Git mutation was performed by the
codec worker. The ignored `target/sdf-record-codec/` receipt preserves exact source,
protected-input and log hashes, commands, sessions and terminal exit results.
