# CDX caption run edits

The [source contract](research/cdx-caption-edits.md) reuses the qualified
caption reader and the existing `text.set_run` command. Only that
command's description changes. Actual CLI discovery matches the
historical 49-entry file with that description change, and stdio MCP
reports the same description and input schema. Historical snapshots
stay byte identical.

## Exact independent witnesses

The [original binary and FontTools checks](../crates/io/tests/fixtures/cdx-caption-edit/SOURCES.md)
reproduce all four edit pairs. They cover single-run growth, early-run
growth and shrinkage, and a supported UTF-8 tail. The ordered witness
contains two anonymous Texts. Its encounter IDs [4, 1, 3, 2] differ
from paint order [1, 3, 4, 2]. Editing Text 2 changes only its property.
Growth shifts starts from [0, 2, 3] to [0, 4, 5] and preserves the
original style words and siblings. The accepted file is 361 bytes.

The isolated encoder and IO compiler REDs precede their APIs. The
command, actual CLI and App runtime REDs report the former CDXML-only
guard. The encoder now checks the exact 65,535 start and its overflow.
IO passes five cases, including no-op byte and Arc sharing, detached
source refusal, and rejection of ambiguous offsets, unsupported joins,
glyphs, and paper overflow. Atom-label cases pass after sharing the
whole-source CDX edit certificate.

## Repaint, history and files

The command, actual CLI, authenticated loopback and initialized stdio
MCP pass existing run edits with exact no-op, Undo, Redo and Unicode
Save/reopen. The App harness checks actual paint changes, history and
its file controls. The original canvas gestures remain gated.

The actual file from stdio reopened with byte-identical CPU and Metal
PNGs. Both blue and orange ink extents agree with the independent
FontTools model within the declared three-pixel solid-colour allowance.
The orange left edge shifts from x322 to x376. The initial ink script
compared the white fill with the paper's outer rectangle. The corrected
check verifies both the one-pixel border and interior under the same
ink tolerance.

The signed macOS review was prepared while hardware input was active.
Live desktop input, a native caption editor, other runtime platforms
and new human golden approval remain open.
The IO crate type-checks on Windows MSVC, Linux GNU and WebAssembly.

## Sanitizer and receipts

AddressSanitizer completed 10,000 runs in one second with 211 unchanged
input files. The largest recorded new or reduced mutation was 1,648
bytes under a 65,536-byte limit. The harness checks exact ownership,
unchanged surrounding records and whole-model reopen.

Receipts are in `target/cdx-resources/next-text/caption-edits`.
`saved-caption-ink.json` SHA-256 is
`8ed9341ef82cae78a184649392b892703ae9e669cd790503455820682d606218`.
`fuzz.log` SHA-256 is
`a8a6c6a02ab73d20f576a9e9877df82a2c94a1c05e39447d7b6e84f8cc339f49`.
The actual CLI registry SHA-256 is
`89b1b1b8416e4da29ed39e1f0e07099bf3788337d7af7480cfe8982c3aeb8c4f`.
The first full gate caught duplicate test helper modules. The qualified
run uses one module per test root and preserves the exact comparisons.
