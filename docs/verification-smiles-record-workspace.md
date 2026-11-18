# Read-only SMILES record workspace

2026-11-15; isolated base `976ef42`. This implements an original inline SMILES inspector
in the real BondlineApp using the shared registered query. Evidence here is actual App
frames/AccessKit/files and finite worker tests, **not native OS input or clipboard execution**.
Root owns integration/Git and any later desktop acceptance. The
[workflow plan](research/smiles-native-workflow-plan.md) and
[typed native work bridge](verification-smiles-ui-work.md) retain their own evidence.

## Implemented behavior

Opening an accepted SMILES file still shows its complete count/byte summary and explicit
unavailable chemical-preview message. It now offers Record number, Inspect record,
Previous record and Next record. Only explicit inspection/navigation discloses notation;
zero-record controls are disabled. Source replacement, including identical-byte Open,
resets the ordinal to1 and clears old text. Out-of-range4 reports a query error rather
than silently inspecting record3. The numeric control is not clamped to the record count;
positive whole-u32/finite guards reject invalid input before a query can use it.

The selected record shows exact literal notation, line/counts and whole-file byte ranges.
Only application-data byte length is shown, never its data bytes. Empty versus absent data
remains distinguishable. No name, canonicalization, inferred chemistry, drawing/import,
selected-record conversion or structure-to-SMILES Copy As is added. Selecting/copying
existing notation is a different workflow from those still-missing capabilities.

Immutable TextEdit permits selection/copy but no typing, paste, cut or local Undo mutation.
Its AccessKit node is ReadOnly without SetValue. Returned text undoers are cleared/stored
each render; hiding/Close/source change clears retained text state and focus. One bounded
scroll area renders one selected reply; the UI never creates a whole-record list or
parses a record during painting. Existing parser/query budgets remain authoritative;
large-value native responsiveness is not established by these small fixtures.

`SmilesRecordsWorkflow` retains only a weak complete-source stamp, close epoch, revision,
ordinal and checked request sequence for currentness. One pending intent, one actual thread
and one self-contained reply stay outside Engine/history/dirty/source. New selection,
source/close change or Close record cancels obsolete work and clears disclosure. Busy,
Cancel inspection, Retry inspection and failure states use the existing two-worker pool.
Registration occurs after the frame Engine guard is released through the real typed
`prepare_ui_smiles`; the key is checked again before installation.

A channel result is not thread retirement: the workflow joins only after `is_finished`.
Pending/actual jobs join the existing File dialog and final-close busy checks. Engine-free
deactivation also drains cancelled finished work on a poisoned Engine. The old plain
workspace adapter remains a static, executor-free view; its old behavior is preserved.
One small named workflow-reference struct connects native rendering, not a new job framework.

## Independent App acceptance

The received five-case source is18624B/SHA256
`8005c5a3a10fe4cdb5b5f9e82a9866aa94625fab2fe71e9aa628a1b5b700094a`.
Its original49-byte vector contains three records, CRLF/LF/final EOF, an ignored non-UTF8
line and NUL/non-UTF8 application data. Explicit record2 must be exactly `[13CH3][OH]`,
line4,2 atoms/1 bond, full21..43/notation21..32/data33..42 spans and9 opaque data bytes.
Literal expectations are independent of a writer or another UI implementation.

- Opening alone exposes neither that notation nor opaque data in paint/current AX strings.
- Explicit nonfirst inspection, standard selection/copy and readonly typing/paste/cut/Undo
  retain exact notation, source, revision/history/dirty and capture stamp. An unchanged
  external RDKit consumer checks the copied isotope/H/Single graph and distinguishes a
  deliberately stripped-isotope negative control. Close record removes text/focus/undo state.
- Actual Previous/Next controls reach first/last records, enforce their bounds, refuse4
  without clamping, and clear disclosure across alternate/identical/ABA Open and empty files.
- Real File controls perform Unicode Save As with no-clobber refusal, explicit replacement,
  known Save and reopen. Every saved49-byte source byte remains exact. The unchanged external
  raw-file consumer checks3 records,5 physical lines, selected spans and `00ff50524956415445`
  opaque data; reopened selected text passes the unchanged molecular consumer again.
- Separate pre-Inspect privacy checks cover every current reachable accessible string.

These are actual BondlineApp frame workflows, not an OS picker/window claim. No independent
acceptance assertion or input was edited by this implementer; revised sources were received
verbatim from their owner with explicit Root approval, as recorded below.

## Finite real-worker lifetime tests

Two private module tests were frozen before production. Their helper runs the real
registered query, publishes its actual result, then holds the UI thread on a finite channel.
All channel/retirement waits have5-second failure bounds; cleanup cancels/releases/joins
outside an Engine guard. No public hook, fake query result or sleep-based ordering proof.

The first test proves published-but-unretired work remains busy, installs no notation,
blocks actual FileWorkflow final-close acknowledgement even after Core activity releases,
and discards late text after a shared Close request. After actual retirement, final close
can commit with complete source retained. The second safely poisons the Engine mutex only
after actual query completion; engine-free deactivation cancels/drains the held thread
without installing text or reacquiring that mutex.

First81847 failed compilation on a nested test-helper path setup error. Moving only that
cfgtest helper declaration to module root preserved assertions; the setup log/diff remain.
Then50878 failed only because the private workflow/RecordJob types did not yet exist.
This is a missing-implementation **compile RED**, not a runtime lifetime failure. The
independent owner separately recorded the initial missing-control runtime RED before UI
implementation. First real lifecycle run83578 passes2/0 with no production correction;
the corrected pre-production lifecycle suffix remains byte-exact.

## Receiving corrections and final checks

First production remained unchanged through every subsequent run:
`smiles_records.rs`23757B/SHA256
`2cfe00d9c1fb047c59c559741dbe1efbf8d3ff34796cd152b22cba9d154fd647`;
`lib.rs`89255B/SHA256
`f15d86f95e5634b8d20424548ef11f7415210869bce942241f17c3622f9608e1`.

The first independent five run99318 passes4 and stops before Replace File: a new test
expected the asynchronous failed no-clobber worker to have already reopened its prompt.
The original target was unchanged. The independent owner added a bounded wait for the
actual prompt and a second exact old-target check, preserving all original expectations.
Run96326 then passes4 and reaches a later path assertion: actual replacement/full49 bytes
already pass, but the new expectation used a lexical `crates/ui/../../target` path rather
than the documented canonical file association. The owner changed only that expectation
to `fs::canonicalize(&output)?.to_str()`, retaining exact Unicode/full-path equality.
Original18393/a76d60aa and intermediate18604/44faa1dd sources, both failures and exact
approved deltas are archived. Neither failure justified a product patch or weaker assertion.

Final independent run31134 exits0:5/0,4.31s compilation/0.38s tests. Whole UI package92222
exits0: **181 passed,0 failed,1 existing ignored** across9 result suites. All-target
warnings-denied UI clippy98174 exits0 (11.19s). Package/owned-file rustfmt checks exit0.
Rust1.94.1, absolute own target, incremental0, dev/test debug0 and pinned BONDLINE_PYTHON
were used. No shared Cargo artifact, Root Fit port, new dependency or SDK patch was used.

```sh
cargo +1.94.1 test --locked -p bondline-ui --lib smiles_records::lifecycle_tests::
cargo +1.94.1 test --locked -p bondline-ui --lib view_tests::smiles_record::
cargo +1.94.1 test --locked -p bondline-ui
cargo +1.94.1 clippy --locked -p bondline-ui --all-targets -- -D warnings
cargo +1.94.1 fmt --package bondline-ui -- --check
```

The nine UI source/input/fixture files are frozen in ignored
`target/smiles-record-ui/source-freeze.json`; six old test files reconstruct their exact
originals after removing only the new App constructor initializer (plus the approved child
bridge in view_tests). MCP work/lib are exact previously verified backing, not new changes
owned by this UI slice. Of63 protected baseline inputs,54 remain exact; the other9 are
those explicit backing/lib/initializer changes. The final ignored handoff pins all sources,
logs, deltas, preserved inputs and terminal jobs.

Root's receiving workspace/independent rerun and native desktop/clipboard/physical-DPI/
Windows/Linux checks are separate. Fractional/nonfinite numeric guards are implemented;
no native numerical-action matrix is claimed. General chemical preview/layout, SMILES record
editing, native conversion/export, broader syntax and full M0–M5 remain unfinished. No Git,
Actions, native input, original data-fixture/oracle/golden/font or dependency mutation occurred here.

Root separately reports receiving missing-control runtime RED33868 before UI publication,
then focused54986 GREEN8/0 (the seven new cases plus one existing matching malformed-input
case) and independent-owner97627 GREEN5/0 on unchanged final acceptance. Root retained its
newer Fit implementation/test files while applying the constructor deltas. These reports
do not replace Root's separate receiving aggregate gate or add native execution evidence.
