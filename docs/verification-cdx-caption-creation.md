# CDX caption creation verification

Base `492b6ae`, 2026-11-19. The [source contract](research/cdx-caption-creation.md)
specifies the direct Page Text, explicit font resources and persistent ObjectTag.
The [fixture register](../crates/io/tests/fixtures/cdx-caption-create/SOURCES.md)
pins six independently checked binary files.

## Persistent ID receiving

The identity runtime RED failed four valid marked inputs at the existing
unsupported Text record. Both legacy and invalid-marker controls passed.
With the identity reader, all six pass. The same run passed 17 existing
caption preview, styled-run and edit tests.

The tests independently check raw-order legacy IDs and sparse IDs with
explicit and implicit persistence. They move a marked Text ahead of the
old raw ID and still recover the original live IDs. Both old and marked
run edits match exact byte splices, including Unicode Save and reopen.
Malformed identities still refuse application Open. The standalone CDX
reader retains their exact bytes, and the Open attempt leaves the file
intact.

Primary worktree receipts are under `target/cdx-caption-creation`.
The identity RED log SHA-256 is
`32a166b73416f59cdca10e4f0b73a6ea5e7d500f584374e9cebd2e1e9c341dd2`.
The passing log SHA-256 is
`af254ec993e56ca3561598e5e84838e02cb5ffc52c1188a2d18d80cbb80e46fc`.

## Binary append

The writer tests first failed to compile at the missing
`StoredDocument::with_cdx_caption` API. Five focused tests now pass.
They verify exact first and sparse-ID bytes, the signed fixed-point
half-tie, all five binary framings, append after an existing edit, and
atomic refusal at the paper, style, raw-ID and paint-depth limits.
The backing bytes and model identities survive every refused edit.

The writer checks the already certified source and appends the caption
and its tag using two owned splices. A reparse between splices supplies
the second handle. The final certification compares the complete
projected model with the expected appended caption and runs the
existing full-paper check. The original snapshot keeps its unchanged
bytes and model.

## Shared command and Text tool

The command runtime RED refused caption creation on the supported CDX
source. The App runtime RED never opened the New caption draft on paper.
Both pass through the same `text.create` authoring path. The command test
checks sparse IDs, exact current bytes and refusal after detaching the
model. App tests at 1× and 2× also check Cancel, an over-wide draft, Undo,
Redo, Unicode Save As, reopen and edit targeting for both captions. A
separate App case checks stale paper presses and drafts when the view
changes and comes back to its original position.

The actual CLI creates and branches history, saves the exact sparse-ID
fixture, reopens it and compares CPU and Metal PNGs. Its first test run
used an incorrect type assertion for the GPU adapter field. The adapter
is an object with a `name`. The corrected test passes without changes to
rendering. Registry 51 keeps the first 50 serialized entries byte-exact.
Only the description of `text.create` changes, and the tool catalogue
remains 17 delivered and 123 planned.

An actual stdio MCP session completes each Open and authoring request
before the next one. It creates, undoes, branches, saves to a Unicode
path and reopens an exact match for the Undo-gap fixture. The first
probe pipelined async file requests and reached stale-revision and busy
responses before Open completed. The serial client passed without
changing MCP. Replies are under `target/cdx-caption-creation/mcp-verified`.

The sidebar and CDX Save description now explain the supported creation
and editing paths. An additional assertion in the App workflow first fails
on the old sidebar string. The rebuilt App passes that assertion and the
new Save description check at both scales.

## macOS workflow

An isolated signed review of the authoring build loaded the 156-byte
fixture using File Open. Native `t`, a blank-paper click, Apply and Cancel
exercised the same editor. Undo and Redo restored the first edit with live
ID 2. Creating again after Undo allocated ID 3 and retired the redo branch.
Save As wrote a 268-byte Unicode-named CDX file. An independent raw reader
located the new baseline and checked its signed fixed-point words
`9482774, 14224161` using decimal rounding. With that actual pointer
baseline the entire file matches the frozen Undo-gap fixture. The input
bytes remained intact. Saved SHA-256 is
`4f323a35cf9eead583311dc2677316142efa4940bf748a67c58649a495156a4d`.

Reopen recovered IDs 1 and 3. Both captions opened their exact Text
drafts, and the new caption accepted a run edit that Undo restored.
The reopened 512×512 CPU and Apple M3 Max/Metal PNGs match byte-for-byte.
The review quit cleanly. File and probe receipts are under
`target/cdx-caption-creation/native`. The two help-string changes were
verified separately in the App harness.

## Sanitizer and platform scope

AddressSanitizer ran `cdx_parse` for 10,000 iterations from 66 unchanged
seeds, using `721533879`. The append oracle checks original-byte retention,
one inserted interval, unchanged framing, complete reopened model and the
saved live ID. The largest observed new mutation was 532 bytes. The largest
initial seed was 45,177 bytes. Log SHA-256 is
`7c9bc3ac55ec6d29ba551a26e818d0c18c534a75958b71c50ae6814233ede84b`.

The command crate type-checks on Windows MSVC, Linux GNU and WebAssembly.
The WebAssembly check emits eight warnings for platform-specific work
paths. New desktop execution is limited to macOS. Blank and mixed CDX
pages, general typography, other-platform native acceptance and external
editor resave remain open.

## Local gate receipts

The full authoring gate in `ci.log` passed 2,320 workspace tests with
one ignored, including the completed CDX creation workflow. The three
AccessKit tests and all format, Clippy, layer, license, asset and pinned
corpus checks passed. The log SHA-256 is
`4dce4f1bd05886d5a2f0859a63d24cacde795985d028514f03d8960ca106dbf0`.

The next aggregate included the help-string improvements. Its format,
Clippy and earlier workspace stages passed, then the unchanged MCP
`shutdown_without_clients_releases_the_listener_address` test failed
at `replacement_bind` on port 63204. The focused guided App test
passed before that aggregate. `ci-final.log` records the failure at
SHA-256 `636a5e4aa05b35fce919eecf278d3b54a1b82c99f8e44ccba17080565d2ae986`.

The unchanged retry on the final application code at `6c0689a` passed
2,320 workspace tests with one ignored across 347 reports, all three
AccessKit tests and every other local gate. The listener code and its
test remained the same. `ci-retry.log` SHA-256 is
`b732c519dc37e475ba35ec4711d0fee3fdfa41ff35117b8682a1f1b319c90e03`.
