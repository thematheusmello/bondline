# SDF existing-value editor — integrated evidence

2026-11-15, branch `feat/multiple-bonds`, parent `778ea47`. Original Bondline presentation
over the existing registered setter; no Professional desktop field-screen parity claim.
The [lifecycle plan](research/sdf-edit-lifecycle.md), [independent App cases](verification-sdf-editor.md),
[bounded text buffer](verification-sdf-draft-text.md) and [registered-worker gates](verification-sdf-editor-lifecycle.md)
retain their separate declared inputs, RED/GREEN, source qualifications and limits.

## Delivered behavior and independent expectations

Records → Inspect record … fields → Field … → Edit opens one bounded temporary draft.
Its text never changes source/history/dirty before explicit Apply. Apply uses the existing
`sdf.field.set_value`, with record ordinal, field occurrence, expected revision and complete
source/close guards. Empty text remains an empty value; line content is not trimmed or
wrapped to make invalid input pass. The existing ASCII, line and command budgets remain.
Omitted query values cannot silently become editable empty strings.

Navigation/Open/Save/close with a changed draft offers Apply, Discard or Cancel. Waiting
actions retain semantics and context, never old command parameters or approval. Apply
must really succeed and its worker retire before a fresh current file/close action runs.
Failed Apply starts no old-byte Save. External source replacement leaves stale text
available for recovery and disables Apply. A cancelled operation that already committed
is truthfully reported as applied; it cannot be treated like a discarded query response.

The unchanged mixed-version input is777 bytes, SHA256
`165e6f5d5cff37dae6315a07843d84733aa42dbbc497b2920da1702bad9526a0`.
The expected second-DUP edit is the literal splice `[0,378)` +
`reviewed\nline two\n` + `[385,777)`:788 bytes, SHA256
`e63cb9adf5cbc62b5b0be94992246c0d10b94cda405e49b9514993e7d9f9c361`.
It preserves all unrelated fields, headers, DT digits, padding and molecular payloads.
These exact expectations predate the editor implementation; no Bondline writer generated them.

## Root checks and corrections

Root reproduced the missing-Edit runtime RED before implementation. The first invocation
selected zero tests through an incorrect filter; it is setup, not behavior evidence.
The frozen positive then passed against the real editor. Broader App tests exposed lost
Previous navigation after Apply:7 passed/1 failed, then8 passed with the unchanged tests
after deferring Previous until the refreshed query, like Next. Finite real-worker gates
separately exposed a missing late-cancellation notice:1 passed/1 failed, then2 passed.
Warnings now use a notice rather than the query-error channel.

The buffer test's deliberately invalid `3..1` range triggered clippy's reversed-empty-
range lint. Only that test expression became `Range { start:3, end:1 }` plus rustfmt;
the same bounds/assertions and exact production prefix remain. A test-only reused-helper
`duplicate_mod` finding received a local, reasoned exemption; test bodies remain exact.
No production lint exemption, global suppression, dependency, SDK, font or golden changed.

An additional ninth actual-App test passed on its first run. It holds an externally
requested native Save before the draft decision: Cancel executes real file cancellation;
Discard adopts the still-current request and writes777 bytes; Apply performs one edit,
obtains a fresh request and writes788 bytes. No extra behavioral RED is invented.
The final independent nine-case file is36,674B, SHA256
`72ac66b03bf74cd91e4c24ad7304ace75898a39f9eaac94d4b672d9963eb9843`;
all earlier31,003/14,341/7,908-byte prefixes remain exact.

Root's earlier full gate passed993/0/1 before that additive case. A later invocation
omitted `BONDLINE_PYTHON` and failed two unchanged CDX consumers with
`ModuleNotFoundError: rdkit`; no code or test was changed in response. The corrected run:

```sh
BONDLINE_PYTHON="$PWD/.cache/oracles/format-python/bin/python" \
  CARGO_INCREMENTAL=0 cargo xtask ci
```

Session44537 exited0: **994 passed,0 failed,1 ignored**,147 result lines including three
macOS AccessKit bridge tests. Rustfmt, warnings-denied all-target clippy, layers, licenses
and corpus pass. Log `target/sdf-editor/root-ci-nine-pinned.log`:118847B, SHA256
`ac370a87068394ed5f73e33bb1834273643bba270674d354f7a91ee75c46eca7`.
The ignored visual-capture test was not run. Runtime closures remain154/165/220 declarations
and two assets. Corpus remains eight pinned files/five bundles; its foreign/general CDX
and general chemistry gaps stay explicit. No new fuzz/Windows/Linux/hosted run occurred.

## Actual native Mac workflow

The user's explicit “Open Bondline on the desktop” authorized opening the new app; no
new availability answer is claimed. The isolated ignored bundle is
`target/sdf-editor-native/Bondline.app`, identifier `org.bondline.sdf-editor-review`.
Its running executable is21,656,392B, SHA256
`be69b1d42682b3800b7bac2a2d12a4d30ca54d377362d34b381b1374604d72ee`, PID19318.
Older preview processes were left untouched. The build log is
`target/sdf-editor/root-native-build.log`; a receipt records the actual copied artifact.

Using the real Mac chooser and controls: open an ignored copy of the777-byte input,
expand Records/record1/Field3:DUP, select Edit, enter `reviewed\nline two`, Apply,
CmdShiftS to `révision-β.sdf`, CmdZ, CmdShiftZ, CmdO/reopen the saved file and inspect
Field3 again. The draft, applied two-line value,788-byte collection, dirty/clean
transitions and reopened value were visibly observed. Undo showed777 bytes and dirty;
Redo restored788 and clean. Final read-only loopback inspection reports revision5,
Undo0/Redo0, clean current788-byte source after reopening. It did not perform the edit.

The initial accessibility-index click on the draft followed by paste made no observed
change; a coordinate click on the visible text line gave focus and native paste worked.
This does not establish general accessibility focus/setter success. Chooser filename
SetValue worked for the Unicode name. No SDK patch was made for this observation.

The actual saved file equals every independently specified788 bytes; the original ignored
input remains777 bytes unchanged. The actual `[0,344)` V3000 and `[473,755)` V2000 spans
pass unchanged external `crates/io/tests/{v3000,mol}_oracle.py` under pinned RDKit2026.09.2.
V3000 reports two neutral carbons, explicit H3/H3, one Single; V2000 reports CC/C2H6/H3/H3.
The initial ad-hoc consumer invocation used the wrong `xtask/oracles/` path and failed
before execution; correcting only the path produced the recorded results. Raw metadata
retention comes from the literal byte comparison, not a lossy RDKit property map.
Ignored `target/sdf-editor-native/verification-receipt.json` stores the actual reads and
token-free final state. Session credentials remain only in private startup output.

Native navigation/close/cancel races, huge-value editing, full accessibility, interactive
Windows/Linux and all reference workflows remain unverified. The default native chemical
preview still honestly refuses its8..256-em guard. No source paper, display H, record list,
font, fixture, oracle or approved golden was altered to manufacture visual success.
This verifies one bounded native editor workflow and its tested lifecycle, not full M1.
