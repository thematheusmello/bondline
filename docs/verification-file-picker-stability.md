# File picker scan-completion stability

2026-11-15. This bounded follow-up changes only the shared FileChooser browser layout.
The delivered restricted-conversion implementation, commands, worker lifecycle, acceptance
tests, dependencies, fixtures, consumers and golden images remain frozen. Root owns Git,
receiving aggregate and native interaction. This worker uses no native window or OS input.

## Observed defect and independent test

Root reported a browse timeout in an aggregate run, with the unchanged case passing in
isolation. Its cause is unproven. The separate deterministic regression below establishes
a real chooser defect; it does not establish the cause of that timeout.

The independent test owner froze `file_picker_layout_tests.rs` at 9,055 bytes/SHA256
`1fb4850995ee53cd5648f1ad7c851c9dda51c18f9343c2c4b89d94589141a079`.
It warms the actual chooser, obtains the enabled reachable Open/Save button's accessibility
bounds, independently verifies its caption was painted there, accepts a directory-result
state, and sends real pointer press/release at that last-painted point. It requires the
actual FileChooserEvent with the literal Unicode path and original discard/overwrite flags.
It neither calls submit directly nor reads/writes chemical files or fabricates a document.

Cases cover both Open and Save: unchanged loading, directory error, one entry, 32 entries,
and 32 entries with truncation/nine omitted entries. Independent runtime 24236/18503 and
receiving 53389 fail at Open/one-entry after the stable-loading/error controls pass. The old
enabled Open ID 7933386399126555942 becomes unreachable and a new enabled ID 17196170676850145722
replaces it. The old click point remains inside both button rectangles; window height
changes by only one logical unit. This identifies identity instability rather than merely
assuming a large button movement. The failing logs and original 58,678-byte chooser
SHA256 `bae00fc9643e9d6421be1dcb24df16471d3483f4b644fc9ec6942c9ff78a736d`
remain under ignored `target/file-picker-stability/`.

## Small shared correction

The pinned egui 0.29.1 [Ui implementation](../vendor/egui/src/ui.rs#L282) seeds a child UI's
automatic widget IDs from its parent's allocation counter. The conditional loading row,
truncation/omission labels and error label previously appeared in the parent before the
filename/footer. Finishing a scan changed that counter. A footer push_id alone would not
remove the changing automatic-ID seed. ScrollArea's
[auto-shrink implementation](../vendor/egui/src/containers/scroll_area.rs#L897) also explains
why a short/long result list could change its outer height.

One bounded nonshrinking browser area contains loading, omission/truncation warnings and
the result list. Warnings precede entries. The existing 80–280 logical-unit height policy
remains. Filename and action controls follow stable parent slots.

Root's review identified a concrete concern with the initial candidate: putting filename
errors into that scrolled list could hide them. A separate new 5,267-byte test/SHA256
`24ac0399eded24980007343485832a5322335fd1cdc545285afcef27873611d3`
was frozen before execution. It actually wheels to the final painted entry, types an invalid
filename through the focused TextEdit, clicks Open, verifies the literal error, and requires
its full text bounds inside the paint clip. It changes no original 9,055-byte assertion.
Runtime 98973 confirms that the initial candidate hides this error. A temporary control run
against the original 58,678-byte chooser plus only cfg(test) bridges passes 41160; the exact
candidate was then restored before the correction. This is a candidate regression, not a
relabeling of the old click defect.

Errors now use a separate always-allocated two-row scrolling area between filename and
actions. The full message remains available independently of the directory scroll position.
One fixed temporary UI key retains only a message/navigation hash; a changed message or
navigation resets the error offset once. The region's ID stays constant, so no growing
per-message cache is introduced and readers can scroll an unchanged long diagnostic.
Both frozen tests pass together 34280: all 10 directory-completion cases plus painted error.
No widget-ID rewriting, cached click replay, input suppression, delay, sleep or retry is
introduced. Directory scanning, cancellation, path validation, format restrictions,
confirmation decisions and returned action flags are unchanged. This does not modify the
separate confirmation-screen layout or claim every possible window/layout transition.

## Verification and preservation

The exact independent test plus an append-only 79-byte cfg(test) bridge reproduces the RED
before the correction. Initial scoped receiving 77554 passes all 10 mode/state combinations
without changing a test input, assertion or timeout. The test's original metadata logging
remains; it is not production logging. Actual SDK/source pins and before/after identities
are recorded in the ignored baseline/receiving/candidate receipts.

All checks use this checkout's absolute target, incremental 0, dev/test debug 0 and the
existing literal Root `BONDLINE_PYTHON` interpreter.

```bash
cargo test -p bondline-ui --lib files::picker_ --locked
cargo test -p bondline-ui --locked
cargo clippy -p bondline-ui --all-targets --locked -- -D warnings
cargo fmt --all -- --check
```

After Root approved the revised delta, one full default-parallel UI package 51281 exits 0:
225 passed/0 failed/1 ignored across nine suites (library 175/0/1 and 50 integration tests).
It includes both frozen picker tests and every original converter/worker/old UI case.
Relevant all-target warnings-denied clippy 81416 and workspace fmt 20295 both exit 0. The
ignored test is the existing explicit offscreen application-frame capture utility.
No source correction followed these checks; only this verification record was completed.

The final handoff records four intentional files, exact source/log identities, all 15 other
previous conversion inputs unchanged, original chooser test-module preservation and the
unchanged workspace/asset/fixture/consumer identities. Earlier failures remain separate
evidence. Native pointer interaction and cross-platform chooser behavior require
Root's receiving checks; this regression proves the actual egui chooser path only.
