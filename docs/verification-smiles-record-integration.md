# SMILES record workspace integration

2026-11-15, receiving base`d99e719`. The original readonly record inspector is now
implemented in the actual App: explicit ordinal/Inspect/Previous/Next, exact selectable
notation and literal counts/ranges, cancel/retry/hide, preserved whole-file Save As and
reopen. No chemical preview, record editor, conversion/export UI or26.1 screen parity
is claimed. The registry remains28 commands.

## Implementation and independent evidence

`SmilesRecordsWorkflow` uses only existing `smiles.record.inspect` and the typed native
bridge. One Weak source/close/revision/ordinal/request key, one pending intent, one real
thread/channel and one self-contained reply remain outside authored state/history.
Preparation happens after the frame unlock. Source/Open/close/hide clears old text and
focus; cancellation/currentness guards prevent old replies returning after ABA. The job
stays busy through publication and actual thread retirement, including poisoned-engine
cleanup. All native file/close busy gates include it. Ordinals are validated as finite
positive integers; out-of-range selection errors rather than clamping to another record.
Source bytes and opaque application data are never decoded into a name/property screen.

Immutable TextEdit retains select/copy, advertises ReadOnly and clears its own text undo
storage. No source/model/history/Save/ACK authority comes from a query. The original plan,
format limits and public/source-policy distinctions remain in
[the plan](research/smiles-native-workflow-plan.md).

[Five independent actual-App cases](verification-smiles-record-ui.md) cover the unchanged
49-byte mixed-ending/opaque-data source; explicit nonfirst isotope notation and raw spans;
standard copy with unchanged pinned RDKit positive/negative consumers; no editing/text
undo retention; first/last/invalid/zero selection; source/identical-Open ABA; no automatic
paint/AX disclosure; and real Unicode overwrite/known Save/reopen with the unchanged raw
file consumer. Two [finite actual-worker gates](verification-smiles-record-workspace.md)
separately prove publication is not retirement, native close waits, and poisoned-engine
cleanup retires without installing text. These are App/worker tests, not OS interactions.

The initial1,207-byte missing-control test is frozen before implementation. Independent
2405 and receiving Root33868 fail the real `Inspect record` lookup after successful
whole-file Open. Private lifecycle50878 has a separate missing-type compile RED, after
an explicitly recorded helper-path setup correction. No stub makes the tests succeed.

Two later independent test setup corrections preserve all inputs and behavior assertions:
wait for the real async no-clobber failure/Replace File confirmation and recheck old
target bytes; compare the receipt to the full canonical Unicode path rather than a
lexical path containing `..`. Original failures/sources and exact reconstructions are
retained. Product code never changed to satisfy those two mistaken expectations.
Final independent tests18,624B/SHA256
`8005c5a3a10fe4cdb5b5f9e82a9866aa94625fab2fe71e9aa628a1b5b700094a`
are exact after both receiving ports; the initial1,207-byte prefix remains unchanged.

## Receiving checks

The producer source freeze2,602B/SHA256
`396cf30112702c45135206a6902d63fd63823d277a376bcf9c447b25acff6d3d`
pins nine UI inputs and two already integrated MCP backing files. Six old test files
change only the new default workflow initializer. Root's newer Fit module/control/tests
are retained exactly; no whole-file test overwrite loses them. Root's ignored
`target/smiles-record-ui/root-source-port.json` records those proofs and actual hashes.

- Producer lifecycle83578:2/0; five App31134:5/0; whole UI92222:181/0/1 across9 suites;
  warnings-denied all-target clippy98174 and formatting pass.
- Independent receiving97627:unchanged5/0; all-target clippy51652 and formatting pass.
- Root54986:8/0, comprising five new App cases, two lifecycle gates and one existing
  malformed-later-record case matched by the filter. This is not eight new tests.
- Root29041 full pinned-Python `CARGO_INCREMENTAL=0 cargo xtask ci`: **1031 passed,
  0 failed,1 existing ignored**,150 result lines including3 macOS AccessKit cases.
  Rustfmt, all-target warnings-denied clippy, layers, licenses and corpus pass.
  Log`target/smiles-record-ui/root-ci.log`,123441B/SHA256
  `75bb6a65b24224cbc0e3729a8712991a21c3d9923ea0b327950633312f17330c`.

Producer41a03334…7ad60 and independent22ede832…751d3 handoffs retain exact reviewed
source, protected inputs, old-fixture reconstruction and terminal logs. No parser, lock,
dependency, asset, font, oracle, golden or wire-schema changes. Corpus8 files/5 bundles
and runtime license declarations154/165/220+2 assets retain their existing partial scope.
No new fuzz, hosted run or interactive Windows/Linux evidence; Actions stays disabled.

## Actual native observation, limited scope

Root7641 builds the actual Mac app. Ignored bundle
`target/smiles-record-native/Bondline.app`, identifier`org.bondline.smiles-record-review`,
PID53821, executable21,932,120B/SHA256
`f4d8627a75310282ca0b7eadeddd1d031a999f4d7ff8d80a923ac79684e443a7`.
The build receipt pins exact final UI/MCP/Fit sources. Startup readiness is observed;
tokens remain in private ignored stdout, never printed or committed.

Authenticated native loopback opens the exact49-byte copy, SHA256
`fe62db368bfb982d6a0d4309337207a419fc088b9c8e20ae9a0a32ccd61e70b7`.
Actual native AX and screenshot show three records,49 bytes, ordinal1 and Inspect/
Previous/Next controls, with Previous visibly disabled and no notation/application data
automatically displayed. A separate explicit native-process query returns record2's exact
notation/counts/ranges; automatic state still omits the result and UI ordinal stays1.
Revision1, clean source, Undo0/Redo0 and all source bytes remain exact. No native GUI
inspection action was triggered by that automation query.

No OS mouse/keyboard input is injected while the availability clarification is pending.
Native ordinal typing, Inspect/Next/Previous activation, selected text/OS clipboard,
Unicode dialog Save/reopen, cancellation/close races and hardware DPI remain unverified.
The SDF preview app PID16278 remains available separately. Native controls visible and
native registry-query success are narrower evidence than a completed native user journey.
Full M0/M1 and M2–M5 remain incomplete; continue from STATUS.md.
