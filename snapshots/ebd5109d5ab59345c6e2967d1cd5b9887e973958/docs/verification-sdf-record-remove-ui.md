# Independent actual-App SDF record-removal acceptance

These four tests exercise the actual `BondlineApp::update_workspace`, current
AccessKit controls, registered background work and `FileWorkflow`. They are input-frame
tests; they do not establish native OS input, clipboard, dialog, Windows/Linux or
Professional screen parity. Record confirmation is original Bondline UI policy.

The frozen design is `docs/research/sdf-record-removal-ui-plan.md`, 7,800 bytes, SHA256
`b25eef759710c1597f0b1f966d0422f81162cc26a93977245788ef662fc999bd`.
Its public format basis is the [official BIOVIA Chemistry 2026 CTfile Formats](https://www.3ds.com/assets/invest/2026-01/biovia_chemistry_2026_ctfile_formats.pdf),
chapter 10, printed 97–99, cached PDF SHA256
`959f2bbf5487e00fa7535ea7ee36f7ce04f39ea72920844641952276de146c94`.
Printed 97's one-or-more-compounds rule supports the sole-record refusal. The shared
codec/command must enforce it independently of the disabled button.

## Frozen inputs and independent expectations

`crates/ui/src/sdf_record_remove_tests.rs` uses the unchanged original mixed fixture:
777 bytes, SHA256 `165e6f5d5cff37dae6315a07843d84733aa42dbbc497b2920da1702bad9526a0`.
Its expected survivor is the literal source suffix `[462,777)`, 315 bytes. The original
V3000 first molecule, duplicate/DT/raw data and delimiter disappear together; the second
MOL, ORDINARY field and delimiter remain exact. No application writer generates the
expected bytes.

The surviving record's ORDINARY value originally occupies `[757,771)`. The independent
draft expectation replaces only that span with `reviewed\nline two\n`: 781 whole-source
bytes, or 319 bytes after removal of the first record. The unchanged raw scanner checks
the actual saved survivor's ranges/ordered value separately from unchanged per-MOL
RDKit checks of CC, C2H6 and the literal single-bond graph. Full-supplier metadata maps
are not a raw fidelity oracle. Preliminary external checks of the declared expected
315/319-byte vectors are explicitly not App-produced-output evidence.

The four finite cases cover:

- Open the complete source, expand Records and require an enabled accessible
  `Remove record 1` button. This is the minimal missing-control test.
- Inspect old fields, request removal and choose `Keep record`: exact state/source/file
  no-op. Request again, `Confirm record removal`, wait for actual work retirement,
  assert the exact 315 bytes, hidden old fields and disabled sole-record removal.
  Undo/Redo restores complete 777/315-byte snapshots. Actual Unicode Save As/reopen uses
  the real picker/workflow, canonical full-path equality and actual-file independent
  consumers; the original input file remains untouched.
- A changed draft on surviving record 2 resolves through Cancel/Discard/Apply before a
  fresh removal confirmation. Apply then remove creates two history steps, with two
  Undos restoring all original bytes; ordinal 2's edited content survives as ordinal 1.
  A 201-byte invalid value cannot apply, show removal confirmation or delete anything.
- Confirmation release followed by Cmd-S in one raw input packet must write the removed
  source through `FileWorkflow`, with one history edit and a completed clean receipt.
  Saving the old 777 bytes cannot satisfy the test.

Controls are locked to `Remove record N`, dialog `Remove SDF record`,
`Confirm record removal` and `Keep record`. No global Delete-key behavior is requested.
Tests use the existing frame/key/button/temporary-directory helpers and unchanged
external oracle support. Asynchronous completion waits on actual state, file bytes and
work retirement with a finite ten-second failure bound; fixed sleeps do not establish
success. Private publication/close/cancellation barriers belong to separate producer
tests and are not fabricated here.

## Baseline RED and source freeze

The initial 1,198-byte test was frozen as SHA256
`5c4076a177ff5131b87b8c1b0902d05f5dabfae40cc40d4b4d14f46a4de3f8e5`.
On unchanged `379fed0`, session 33163 compiled and ran the actual App, then exited 101:
`current actionable Remove record 1 field/control missing`. This is a runtime RED;
the test references no future Rust API. Log: 7,677 bytes, SHA256
`7648be1b512ced3bf60143ab0c806f1ddeaf194dd67884ad6fd0a0c5bbed703b`.

The expanded file is 19,026 bytes, SHA256
`911f583620bbd1188e46d1c1b18dd5f372bd20dd504fc46bc793ad66c1dcd5d9`.
Its original first test function remains byte-exact. Session 22745 then compiled and
ran all four against the same baseline: 0 passed, 4 failed, each at the missing Remove
control. These are intended REDs; no later workflow is claimed to have passed yet.
The only existing-source change is the appended child-module declaration in
`view_tests.rs`. Old assertions, fixtures, oracle scripts, production and dependencies
remain unchanged. The pre-port manifest protects 170 such input identities.

Run with the pinned external interpreter selected through `BONDLINE_PYTHON` and an
absolute checkout-specific `CARGO_TARGET_DIR`:

```sh
CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
  cargo +1.94.1 test -p bondline-ui --lib view_tests::sdf_record_remove --locked
```

At the pre-port freeze, GREEN acceptance was pending a Root-mediated port of the real codec,
command/adapter and UI implementation. The tests do not contain a backend stub or
local byte-splicing implementation. No Git, native input, dependency or Actions mutation
was performed by the independent test worker. Ignored `target/sdf-record-ui/` receipts
retain original and expanded sources, RED logs, expected-vector proof, hashes and
terminal job handles.

## Independent receiving GREEN

Root authorized the exact real backend and producer UI port. The producer's seven-file
source manifest is 1,392 bytes, SHA256
`17aedb0ea61fc39bb66d3f8f2e0ecee4debc4db4efdd5b2799e267ec17539025`;
the six-entry backend manifest is 1,141 bytes, SHA256
`ae51112fc080f92201486d9c9c203855acbdb980a7fbdd726801944ac1089ffd`.
Every received entry was verified before and after copying. The independent 19,026-byte
test file remains exact. Its existing `view_tests` child name stays `sdf_record_remove`;
the producer copy differs only in that private module name. No other parent-test byte
changed. The old editor-lifecycle helper received only the approved Field-input unwrap
and mutation-kind initializer; removing those adaptations reconstructs its original
source and every assertion exactly. The other 160 protected inputs remain unchanged.

Session 32316 compiled those actual sources and passed **4/0/0** with the frozen input,
assertions and event sequences. Immediate `Keep record` completes the no-op and removes
its stale confirmation; no settling frames were added to make the assertion pass.
The other cases verify actual 315-byte and 319-byte source/history results, disabled
sole-record removal, disappearance of old fields, Unicode Save/reopen, unchanged
raw/per-MOL consumer results, invalid Apply refusal and ordered confirm→Save publication.

All-target warnings-denied UI clippy (session 94255) and workspace formatting (41857)
both exit 0; exact logs are recorded in the final receipt. No redundant full UI package run is claimed by
this worker. Producer package evidence belongs to its own record. Native OS input,
dialogs, Windows/Linux workflows, full reference parity and broader record operations
remain outside this four-case acceptance. No test setup correction, oracle update,
dependency change, native input, hosted Actions or Git mutation occurred here.
