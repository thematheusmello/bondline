# Ordered Undo and File Save

The actual application-frame regression now passes: primary-Z followed by primary-S
in one input packet undoes Single→Double before capturing Save. Memory and the actual
saved file contain Single/H3, with undo depth0, redo depth1 and a clean file. Independent
whole UI package checks pass144/0/1; the ignored test is the existing opt-in capture.
This is headless evidence, separate from native-window verification.

## Cause and scope

Root's later native inspection exposed a real failure after the earlier navigation
gate: a rapid Undo/Save sequence displayed Saved but retained and wrote Double/H2.
The existing File keyboard/menu prepass could create a pending file request before
the earlier Undo event was dispatched. That request then blocked authoring/history.

The two-file correction moves File controls into the existing ordered workspace event
dispatch and resolves the current central binding at its actual event. It preserves
the shared command registry, FileWorkflow worker, Save reservation, source owner and
completion handling. It adds no command, file format, dependency, camera policy or
chemistry rule. Reverse-order or every possible mixed-input sequence is not claimed
by the new regression.

## Independent regression and RED

`crates/ui/src/view_tests.rs` adds one test after the exact43,346-byte ten-test prefix.
The11-test source is47,960 bytes, SHA-256
`e7dfb9cbc2940c1b855546b20cb34d9b9de90e9aa5fa190f67560e679b911906`.
No earlier body, assertion, event sequence, fixture, oracle or golden changed.

The test creates an original Single structure, associates and reopens native source,
selects150% view, and clicks the real bond through `BondlineApp::update_workspace` to
convert it to Double. It first executes separated Undo and Save as a positive control,
then the same keys in one packet on a fresh equivalent fixture. It waits at most ten
seconds for actual FileWorkflow/executor retirement and checks the actual saved bytes
with the unchanged external Expat/Decimal reader. It also checks complete model,
revision, history, dirty/source state, exact original source bytes and unchanged camera.

Independent35353 exited101 at the intended saved-file assertion: the separated control
passed, but the batched case saved Double/H2 and retained revision3/undo1/redo0. The file
worker had completed. `target/view-undo-save-red-receipt.json` binds this actual RED.
Root's first wrong test filter ran zero tests and is explicitly not RED; its corrected
runtime attempt and subsequent23201 GREEN are separate receiving evidence.

## Independent corrected-source checks

The two Root-owned inputs are frozen by `target/view-ordered-file-port.json`,604 bytes,
SHA-256 `40a738c884b56638b3cc74d722cf49f602c2ffd5c1b9ca97402b5d491b635c83`:

| File | SHA-256 |
|---|---|
|`crates/ui/src/lib.rs`|`cf0bbee73e810a7f7fb0c06ffab0c08b575ce890be9a10c4cd761a3fb361e82e`|
|`crates/ui/src/file_workflow.rs`|`582a6a783223bcca43b5bc9186e20a2beeaccfd6d2bf7c06b27a5ec7365be396`|

All101 combined earlier/follow-up Root inputs were verified; all723 other initial
files remain exact. The frozen eleventh source was not corrected after RED.

| Session | Command | Result | Ignored log |
|---|---|---|---|
|83703|`cargo test -p bondline-ui --lib view_tests --locked`|11/11; exit0|`target/view-ordered-file-eleven.log`|
|84983|`cargo test -p bondline-ui --locked`|144 passed,0 failed,1 ignored; exit0|`target/view-ordered-file-package.log`|
|4251|`cargo fmt --all -- --check`|exit0|`target/view-ordered-file-fmt.log`|
|45305|`cargo clippy -p bondline-ui --all-targets --locked -- -D warnings`|exit0|`target/view-ordered-file-clippy.log`|

Checks used the worktree-local absolute target, incremental/debug information disabled,
and the existing pinned `BONDLINE_PYTHON` interpreter. Unchanged command/CLI suites were
not needlessly repeated. Final source, preserved-file and log hashes are in
`target/view-ordered-file-handoff.json`. All jobs are terminal. This test owner performed
no native interaction, hosted Actions, installations, production edits or Git actions.


## Root integration and actual native verification

Root's corrected matching runtime RED is `target/view-undo-save-root-runtime-red.log`
(exit101); the first `target/view-undo-save-root-red.log` ran zero tests and is not RED.
The unchanged regression23201 passes; complete UI90356 also passes. A bounded separate
source review found no confirmed material regression in current-binding resolution,
text focus, pending-intent barriers or existing cancellation/Save/close handling.
Its ignored report SHA is
`84e557bfd96b8be208ed0c9d7d2324d495b54e70c63e5ade2a9f823c182e777f`.

The actual fixed Mac/Metal binary21,188,872B/SHA
`af8a72ed079d15c32b4c1d68c2f7cf891ba5ab56f34029ae8505dbb903259d93`
natively opens the original952B three-pair A4 scene, enters150% Zoom, and clicks the
first Single edge into Double/H2. The same rapid native CmdZ→CmdS sequence that failed
on the previous build now restores and saves Single/H3. Actual state advances revision2
→3 with Undo0/Redo1 and clean source; camera150%/epoch1 stays exact. The file is byte-for-
byte the original, SHA
`295e90470595acba479b95cf3be45d701e900a463fddeeafeb3e47e483b925f4`.
Unchanged external Expat/Decimal and RDKit2026.09.2 confirm all literal graph data and
CC/C2H6, C=C/C2H4, C#C/C2H2, rather than trusting the Saved label. Native event batching
itself is not inferred from timing; the independent regression proves the exact one-
packet case, while this native repetition proves the actual window/files.

[The current unedited screenshot and provenance](media/workspace-native-view-provenance.json)
show that fixed executable; earlier failed/premature captures remain ignored evidence.
Root aggregate52827 passes961 tests,0 failures,1 existing opt-in ignored capture,146
result lines including3 native AccessKit bridge tests. Full rustfmt, warnings-denied
clippy, layers, licenses154/165/220 plus2 assets and unchanged corpus8files/5bundles pass.
Final log115304B/SHA
`6a680a8baeeb4266fe46b6b1b1658238d98c07b43d556e67e98995b37fb8249e`.
No new hosted, Windows/Linux interaction, parser fuzz or complete M0–M5 claim follows.
