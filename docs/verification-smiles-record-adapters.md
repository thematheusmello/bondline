# SMILES record query adapter verification

2026-11-14. Supplied checkout `bondline-mol-v3000`, initial base `17e2a4a`.
Root owns command implementation, Git and integration. This owner changes only CLI/MCP
adapter tests and this record. No native-window/GUI input or activation, Actions, installs,
dependencies, locks, production code, old oracle or fixture changes.

The agreed command is `smiles.record.inspect`, registry name 25, read-only. Its only
required closed parameters are expected revision and positive ordinal record ID. The
explicit optional `smiles_record` result has twelve fields: captured revision/generation,
record/physical-line identity, full/notation/separator/application-data/ending ranges,
atom/bond counts and exact notation. Ranges are whole-file `{start,end}`; separator/data
ranges may be null. Opaque application bytes, graphs, normalization and model projection
are excluded. The 128 KiB budget applies to the serialized `smiles_record` object only;
existing CLI/MCP/socket envelopes and MCP text duplication have their separate budgets.
This is an original query contract, not evidence of a reference record-browser UI.

## Independent vectors and acceptance scope

The original inline 82-byte source has three meaningful records, mixed CRLF/final EOF,
a leading blank and an ignored line. Record 2 is `[13CH2]=[CH1][OH1]`, with carbon13,
literal H2/H1/H1 and Double/Single bonds. Its physical line is 3, full span20..58,
notation20..38, separator38..40, opaque data40..56 and CRLF56..58. Private metadata includes
non-UTF-8/NUL bytes. Record 3 is charged `[NH4+]` at76..82 on physical line5, with no
separator/data and an empty82..82 EOF ending. Expectations are predeclared literals;
no future query implementation or lossy supplier/property map generates them.

Actual returned nonfirst notation must face the unchanged external `smiles_oracle.py`
with the existing pinned RDKit2026.09.2 settings. It checks every atom's element,
isotope, authored H and charge, plus every endpoint/order. External canonicalization is
not used. An additional stdio case uses the old valid 65,536-byte bracket/leading-zero
lexical vector: exact disclosure and payload size, not external full-grammar conformance.
The original raw source must remain unchanged across query, Save and reopen.

CLI tests cover exact nonfirst selection through Save/reopen, ordinary Inspect omitting
the query, invalid requests and exact saved-source retention. Stdio additionally checks
25-tool discovery, the closed two-field schema, readOnlyHint, exact twelve-field shape,
nullable final-EOF ranges, structured/text agreement, error/Inspect state equality and
the maximum notation-sized disclosure. Authenticated loopback checks the same actual
workflow, unchanged state after removing only the explicit result, and no repaint from
queries/refusals. An already returned response remains historical after a later Open;
that assertion does not inject or claim a concurrent scheduling test. Root separately
owns deferred-capture/concurrency/close-command tests.

No application data appears in automatic state or explicit query. Read-only queries do
not change source/dirty/history/association. Current whole-file Open/Save and every prior
schema assertion remain meaningful; discovery adds only one name/count.

## Actual initial RED

The new CLI test first ran against the actual unchanged 24-command base:

```bash
cargo test --locked -p bondline-cli --test commands executable_explicit_smiles_record_query_selects_nonfirst_notation_and_keeps_metadata_private -- --exact
```

Session `91631` is terminal, exit101:0passed/1failed/17filtered. Compilation and complete
file Open succeed; the actual command returns
`{"error":{"code":"unknown_command","message":"unknown command: smiles.record.inspect"}}`.
Log: `target/smiles-record-adapters-red.log`. This is a runtime missing-command RED,
not a missing symbol, syntax failure or simulated backend. Only generic JSON/current
adapter helpers are used.

At the initial checkpoint the draft was held for the real shared command: no stub or
uncommitted implementation was copied and no draft GREEN was claimed. Root subsequently
ported the real command `0b29ffd` as supplied commit `843a79f`. These commit identities
are supplied by Root, without worker Git operations.


## Actual command acceptance and corrected draft assumptions

The first real-command focused session `47893` returned the exact expected nonfirst record
and passed that workflow. Its second CLI test found a draft error-code assumption:
`WrongContent` has always mapped to `unsupported_content`, whereas three new assertions
had invented `wrong_content`. Existing `Error::code`, SDF query and SMILES file tests were
checked before correcting only those three newly authored strings. No production code,
prior assertion or error API changed. The failed log remains
`target/smiles-record-adapters-focused-cli.log` (1 passed / 1 failed).

Session `85405` then passes the two CLI tests and one stdio test. The new socket test
finds a second draft assumption: malformed closed parameter shapes intentionally receive
an error and close the control connection. Its attempted subsequent Inspect therefore
hits EOF. The existing `Rejected::command` / connection security policy was inspected;
no product change is warranted. The new test now asserts EOF only for the extra-field
and missing-required-field cases, re-authenticates, and keeps every exact state/no-repaint
assertion. Runtime zero/stale/missing targets retain their original connection behavior.
`target/smiles-record-adapters-focused-socket.log` preserves that single failed socket run;
no second failed socket run occurred or is fabricated.

The corrected socket-focused session `77580` passes. There are four new passing tests:
two CLI, one stdio MCP and one authenticated loopback. The real returned isotope/order/H
notation is checked by the unchanged external consumer on each adapter. Stdio also returns
the exact 65,536-byte lexical notation, twelve-field shape and object-only 128 KiB bound.
MCP structured/text agreement is checked separately from that object budget. All full
source bytes survive Save/reopen. Automatic Inspect omits query notation and private data;
only Open, Save and reopen cause the three loopback repaint notifications.

Final owner commands:

```bash
cargo test --locked -p bondline-cli -p bondline-mcp
cargo clippy --locked -p bondline-cli -p bondline-mcp --all-targets -- -D warnings
cargo fmt --all -- --check
```

Session `76645` is terminal, exit 0: **79 passed, 0 failed, 0 ignored** across eight suite
result lines, including empty binary/doctest suites. Relevant all-target warnings-denied
clippy and workspace formatting also exit 0. Logs are
`target/smiles-record-adapters-packages.log`, `target/smiles-record-adapters-clippy.log`
and `target/smiles-record-adapters-fmt.log`. The affected-package result includes the four
new adapter tests; it does not pool Root's separate 174-command-test gate.

Every command uses this checkout's absolute target, incremental 0, dev/test debug 0 and
the existing pinned Root interpreter through `BONDLINE_PYTHON`. Test clients retain the
existing bounded timeouts and authentication helper; no actual token enters evidence.
No live worker, listener, client subprocess or Cargo job remains after completion.

The final ignored handoff records exact six-file/log identities, the received core's
independent freeze/report, and protected source/fixtures/oracles. All four prior adapter
files reconstruct exactly after reversing only newly added tests/schema assertions and
the additive name/count updates. The original pre-command draft and both failed focused
logs are retained, along with the successful corrections; they are not claimed as
production defects or additional feature REDs.

This owner does not run new native GUI/Windows/Linux, full-workspace/xtask/fuzz or hosted
checks. Root's receiving aggregate and command concurrency/lifecycle evidence remain
separate. Returned-response history is checked after a later reopen, without injecting
an actual overlapping wire schedule. No record browser/editor, graph disclosure,
normalization, broader chemistry or completed milestone is claimed. Actions remains
disabled and all own jobs are terminal.
