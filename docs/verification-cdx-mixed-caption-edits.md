# Editing existing captions over chemical structures in CDX

Verified on macOS on 2026-11-20. A qualified mixed Page contains two chemical
fragments and two direct captions in these witnesses. The blue caption's
persistent live ID is 19. Its raw object ID is also used by both Node-owned
labels. The intended bytes and rendered overlap come from the independent
generator and pinned font outlines. [Contract](research/cdx-mixed-caption-edits.md),
[fixture construction and byte ranges](../crates/io/tests/fixtures/cdx-mixed-caption-edit/SOURCES.md).

Before the change the actual CLI read all five sources and refused editing.
The new command, CLI and App tests also failed against the read-only version.
The App could see the ink, but a click on its frontmost blue cell opened no
caption draft. The focused receiving run passed two IO tests, two command
tests, 16 caption App tests and the real CLI Unicode save-and-reopen test.
The App tests click the blue/red overlap to select ID 19, modify both runs,
exercise Undo/Redo, open the Unicode-named result and edit again. Clicking
a red-only cell or blank paper leaves the drawing intact.
[IO](../crates/io/tests/cdx_mixed_caption_edits.rs),
[commands](../crates/commands/tests/cdx_caption_edits.rs),
[App](../crates/ui/src/native_caption_tests.rs),
[CLI](../crates/cli/tests/support/cdx_caption_wire.rs).

The edited first run shifts the second start from 2 to 4. A second edit changes
the final Unicode body. Each accepted state matches its entire independent
487–491 byte file. The tests also distinguish reversing an edit from Undo
restoring a source's original extended-length encoding. Neither atom label
changes. Missing IDs, detached models, ambiguous run starts and text outside
the paper retain their guards. Discovery is byte-identical to the previous
branch at 51 commands and 49,922 serialized bytes.

The real CLI, initialized stdio MCP and authenticated native host exercised
both edits, checked Unicode Save/reopen and compared the 100% and 200%
outputs. Separate renders of the original fixture bytes supplied an independent
file input for every image comparison. The two edited states produced distinct
6,765/23,510-byte and 6,799/23,552-byte PNGs. CPU and Apple M3 Max Metal
agreed, including the overlap samples. The app ran hidden to avoid interfering
with desktop input. Painted-cell input was exercised in the App harness.

The mixed-run fuzz oracle checks the original targeted Text property, all other
bytes and graph objects, style words, re-import and detached refusal.
AddressSanitizer completed 10,000 trials with 137 distinct seeds whose hashes
were unchanged. The largest new/reduced input was 750 bytes. Peak reported
RSS was 250 MiB. Final log SHA-256
`7c676e6985d6f674d12e247a80811e61e6106d924af9e3f95ae9dee5b6019ac4`.
Local files and receipts are under `target/cdx-mixed-run-preparation`.
The first native verification approved Close but exceeded its eight-second
process-exit bound. A fresh run with the same binary also approved Close and
exited in 0.416 s. Neither observation identifies the cause of the first timeout.

```sh
python3 -B crates/io/tests/fixtures/cdx-mixed-caption-edit/generate.py
cargo test --locked -p bondline-io --test cdx_mixed_caption_edits
cargo test --locked -p bondline-commands --test cdx_caption_edits
cargo test --locked -p bondline-ui --lib native_caption_tests
cargo test --locked -p bondline-cli --test sdf_capture_adapters mixed_unicode_run_edits
```

## Completing an externally applied draft

An App-frame regression reproduced a separate presentation bug. A successfully
applied `ui.caption` request disappeared from the engine while its already
presented local form stayed open. Even a no-op Apply could leave a pristine
field blocking Close. The fix records the submitted text on the exact transient
request, with no change to wire serialization. The form processes queued input
before checking whether it still holds the original or submitted text. A
different local draft stays available to copy and explicitly cancel.

All 18 focused caption App tests passed. Cases include a changed Apply, a
no-op Apply, Close queued before the first repaint, a matching local submission,
and divergent input entered before or during the receiving frame. The original
native timeout had no failure-time observation of the form and remains separate
from this deterministic regression. [App regression tests](../crates/ui/src/native_caption_creation_tests.rs).

The rebuilt app passed the same native Apply sequence twice. In the second
run, four requests remained current for 501–506 ms before Apply. Both runs
saved the exact file and exited on approved Close. The public host interface
reports draft ownership and text. The App regression checks widget presentation
directly. The fixed CLI and initialized MCP server passed the same file and
capture checks. The held-draft native receipt is
`native/host-iagrhp4a/verified.json`, SHA-256
`84a0540c49de98f5cf93168bad53fdb61bd9efd563777ea640b4a8d04d800650`.
The Windows MSVC, Linux GNU and wasm32 commands-crate type checks passed.
Wasm emitted eight warnings. Runtime input on those platforms remains open.

The final `cargo xtask ci` on `f93778f` passed 2,336 workspace tests with one
ignored and three additional AccessKit tests. Formatting, warnings-denied
clippy, layer, license, asset and corpus checks passed. The complete local
log is `target/cdx-mixed-run-preparation/ci-final.log`, SHA-256
`350ba35f2f0607146a0c036d43b3f0db644a9ad525b9aca1ea9768f0ef5b1d87`.
