# Owned CDX Node label verification

The [Node label contract](research/cdx-node-labels.md) adds data coverage to
the elemental CDX reader. It retains the child Text bounds, position, styles
and body alongside the separate numeric atom fields. Unchanged Save uses
the original source bytes.

## Runtime RED and GREEN

The [34 original cases](../crates/io/tests/fixtures/cdx-node-labels/cases.json)
were pinned at `fa721a0`. The pre-label binary rejected all 34. Two new
runtime tests failed at Open, including the comparison of all seven supported
models. The remaining test checked the exact diagnostics of all 27 controls
through both the graph and document readers.

All three new IO tests pass. They check a conflicting label and numeric
element, signed positions and boxes, distinct justification and alignment
codes, explicit empty text, scoped Text identities, non-Roboto styles and
UTF-8. Equal models from reordered source retain distinct original bytes.
Changing an owned label in a detached model refuses file publication until
the original model is restored. Both Unicode paths save and reopen exactly.
The existing caption, elemental graph and plain-bond tests also pass, for 42
tests in the focused run. A mistaken initial test target name ran no tests.
The first run with the correct targets lacked the pinned RDKit interpreter.
The qualified run uses the documented `BONDLINE_PYTHON` setting and retains
all earlier assertions.

The independent binary reader pins object owners, field widths, stored
values and resources. RDKit 2026.09.2 reads the connected witness as C₂H₄
with literal H2 on each atom and a directed double bond from atom index 1 to
0. The conflicting text fixture keeps every numeric field as authored.

## Command and native evidence

The real CLI and authenticated loopback adapter opened the same connected
model and saved and reopened exact files. Capture, atom editing and page
resizing refused with the same state before and after. An initial adapter
assertion expected detached resizing through a command. The existing
authored-label guard rejects that command, so the final test requires the
refusal and checks detached model changes separately at the IO boundary.

Stdio MCP opened the explicit alternate-font, coloured Unicode witness and
saved the same bytes under a Unicode filename. The first pipelined probe
allowed later requests to race the asynchronous Open. The qualified probe
waits for each response and checks the complete model, exact file, refused
capture and resize, and clean reopen. The CLI registry remains byte-identical
at 49 commands. The catalogue remains at 16 active bindings and 124 planned
entries.

On macOS the separately signed review app opened the conflicting fixture.
Cyclooctane was selected in the new file and became unavailable after Open.
The canvas showed the existing incomplete-source notice. Option-click and
drag preserved the complete reopened document state. An unsupported source
reported its retained decode error and Cancel kept the saved file active.
Known Save advanced the controlled mtime to `1795038691584531781` ns. Save As
wrote `label-再開-λ.cdx`, and actual Open recovered the complete clean model
with empty history. The original and copy are 212 bytes with SHA-256
`9c524ca8466cdbf0af158c40b118b1a0290a011cb5cb1a87a669b29eea91863e`.
The independent binary reader confirmed both outputs. The review app exited.
Some native filename labels display placeholders for CJK glyphs.

The executable SHA-256 was
`0db5e0cf7fd3f17e992e6085e43c6439773b45e5165eafcdb62bfa7533ff8fb4`.
The local receipts are under
`target/cdx-resources/next-text/node-labels.B6ZqBH`.
`native/native-evidence/verified.json` has SHA-256
`37051a87c20060fc6e19f65bdcb56e58d203e2080c96a928e82ba2c74a23922c`.

## Sanitizer and platform checks

`cdx_parse` checked each admitted atom's child owner and raw text, positions,
box coordinates, byte offsets, literal style words and resolved resources.
The graph and application models preserve the same fragments. ASan passed
10,000 runs in one second with 151 immutable originals, including 34 label
witnesses and 117 earlier captions. All input hashes passed afterward.
Largest seed was 45,177 bytes, and largest recorded new or reduced mutation
was 858 bytes. `fuzz.log` has SHA-256
`31acf7b5330c17008ce6fc89b3e6e52a199ba461d2ed8758a4e5b3be9d40fe49`.

The IO crate type-checks for Windows MSVC, Linux GNU and WebAssembly under
the pinned toolchain. Runtime work on these platforms, faithful CDX label
layout, inherited styles, chemistry agreement, conversions and editing
remain open.

## Final local gate

`cargo xtask ci` passed **2,227/0/1** workspace tests across 328 reports and
**3/0** AccessKit tests. Formatting, warnings-denied clippy, layers, runtime
licenses, seven assets and the pinned eight-file corpus passed. The
280,491-byte `ci.log` has SHA-256
`f51df6a96282c3c25c4dfcbdd5f5ce44118b59641d1e30158771438a9dedfd89`.
The historical MCP listener issue remains unexplained. All local verification
jobs for this increment have exited. GitHub Actions remained disabled.
