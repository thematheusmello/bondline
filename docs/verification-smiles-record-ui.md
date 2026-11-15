# Original SMILES record UI acceptance

Independent actual-`BondlineApp` frame tests at base `976ef42`, 2026-11-15.
**Current state: five independent receiving tests GREEN; clippy/fmt pass.** No
native window, OS input or clipboard was used. The record browser is original
Bondline UI, not evidence of a Professional26.1 screen or chemical preview.

## Independent input and contract

The existing original query vector is exactly49 bytes:

```text
CC first\r\n\n \xffignored\n[13CH3][OH] \0\xffPRIVATE\nO last
```

It contains three accepted records, five physical lines, an ignored opaque line,
CRLF/LF/final EOF and opaque application data. Explicit record2 is the literal
`[13CH3][OH]`, line4, two atoms/one bond. File-relative ranges are record21..43,
notation21..32, separator32..33, application data33..42 and LF42..43. The nine data
bytes must never enter UI shapes, current accessibility nodes or copied notation.
The UI must not disclose notation automatically when opening a source.

Control labels were locked with Root before additive expectations: Record number,
Inspect record, Previous record, Next record, Close record; Cancel inspection and
Retry inspection appear only for their actual states. The readonly input is
labelled Record2 notation, with literal line/count/range/data-length metadata.
Typing/paste/cut/Undo cannot mutate the notation, source, history or dirty state.
Readonly text remains selectable/copyable and retains no TextEdit undo payload
after rendering, hiding or replacing the source.

The tests use the existing private `view_tests::Fixture`, actual egui frames and
real registered command/file workflows. Only a new child module was appended to
the old harness. No new public test hook, fabricated result or future API type
was introduced.

## Test-first evidence

The first test was frozen before production:1,207 bytes, SHA256
`d9ca1a293b9040a9091db9497b2df7b1bd573cc0b98822df8b59ddc65ca55a6d`.
It opens the complete file, checks three records/nonDrawing/no automatic painted
notation, then requires the actionable Inspect record button.

- Own session `2405`, exit101: **0 passed,1 failed** at
  `current actionable Inspect record field/control missing`. Source decoding and
  preceding assertions passed. This is actual-App runtime RED.
- Three bounded acceptance cases were appended, preserving the complete first
  source prefix. Session `33519` found a new-test borrow setup error (`E0502`):
  calling `warm()` inside an assertion already borrowing the Fixture. It was
  split into a local expression. Source inspection also corrected the existing
  chooser confirmation label to its actual `Replace File`. Neither change
  relaxed an assertion or product behavior; this compiler failure is not a
  behavior RED. The original four-case source remains ignored evidence.
- Corrected four-case freeze:17,533 bytes, SHA256
  `0ec54e31908cb2d9690cc9ddecc408446393723202a90c2c704bb257ee745f13`.
  Own session `15828`, exit101: **0 passed,4 failed**. The first still lacks
  Inspect record; the other three lack Record number. No backend was ported.
- Root then approved one appended fifth guard: before Inspect, selected notation
  must be absent from paint and every reachable AX name/value/description. The
  complete17,533-byte four-case prefix and1,207-byte first prefix remain exact.
  Five-case freeze:18,393 bytes, SHA256
  `a76d60aa8b8f0713bffb0c1b28b9b04016392b189a10739a88c45251fd21e824`.
  Own filtered session `38272` passed this baseline-positive privacy guard. The old
  metadata-only UI already satisfies it; this is not a new-feature GREEN or RED.

The first three frozen additive cases require:

1. Explicit nonfirst notation and exact metadata; current AX ReadOnly/no SetValue,
   real selection/copy, pinned external chemistry consumer, typing/paste/cut/Undo
   protection, hidden-text/focus/undo cleanup and unchanged source/automatic state.
2. First/last navigation limits, invalid ordinal refusal without silently selecting
   another record, source A→B→A and identical-byte Open disclosure clearing, plus
   an actual zero-record file with disabled actions. Root owns deterministic worker
   publication/retirement race tests; this suite does not duplicate their framework.
3. Actual Unicode Save As through the chooser and existing-target confirmation,
   full49-byte output, associated Save, chooser reopen, explicit inspect again and
   unchanged external raw-file/per-record consumers. This is filesystem evidence
   after GREEN, not OS chooser/native clipboard evidence.

## External consumer qualification

The unchanged `smiles_file_raw_oracle.py` independently confirms full framing,
ordered ranges and opaque bytes. The unchanged `smiles_oracle.py` uses pinned
RDKit2026.09.2 with sanitize/removeHs/parseName/allowCXSMILES disabled. It confirms
the selected C isotope13/H3, O/H1 and Single endpoint pair; an isotope-stripped
control is distinguishable. The raw oracle and chemistry consumer have different
jobs; a whole-file RDKit supplier is not raw opaque-data preservation proof.

Bounded pre-port executions of those three fixed vectors passed and are recorded
in `target/smiles-record-ui/vector-oracles.json`. They qualify independent input
expectations only, **not command/UI GREEN**. No consumer, parser, fixture or asset
was changed, and no chemical inference or complete SMILES support is claimed.

## Reproduction and pending checks

Use this worktree's absolute `CARGO_TARGET_DIR`, `CARGO_INCREMENTAL=0`,
`CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0` and the existing pinned
`BONDLINE_PYTHON`, then:

```sh
cargo test -p bondline-ui --lib view_tests::smiles_record --locked
```

Ignored `target/smiles-record-ui/` retains original/corrected frozen sources,
all three baseline logs, raw/notation consumer outputs, source manifests and the
pre-port receipt. Before receiving production,865 other baseline files are exact;
the sole existing-file change is the appended test child-module bridge.

At the pre-port freeze, implementation GREEN, lint, formatting and native
acceptance remained pending. The receiving result is recorded below. No Git,
hosted Actions, new dependency, native input or independently authored production
edit occurred here. All own jobs are terminal.

## Receiving async-confirmation setup correction

The UI owner ran all five original18,393-byte tests against its real first
implementation: session `99318`, exit101, **4 passed,1 failed**. Only the Save As
case failed, looking for Replace File before the asynchronous failed-save response
had reopened that confirmation. Root traced the unchanged chooser/worker path:
submit starts a real `overwrite:false` save; its failed result creates the later
confirmation. Two ordinary frames are not a completion barrier. This is a test
timing setup error, not a producer behavior RED.

Root approved one insertion after the existing old-target byte assertion: wait
through the existing bounded helper for the current Replace File control, then
reassert the exact same old bytes before the existing replacement action. No
input, old assertion or product code changed. No sleep was added. Removing only
that insertion reconstructs the full original18,393-byte/a76 source exactly;
the initial1,207-byte prefix stays exact directly.

Corrected source:18,604 bytes, SHA256
`44faa1ddfa89ea849448db94276a2dcb4fda99b5e6edccfa4e85f5ae9642730f`.
The peer's original1,252-byte log, SHA256
`c2d04a4a97fe56d487921ba85b21342f292db171e037d537cfdb3bf919ad42c0`,
was independently read/rehashed and preserved here alongside its exact source pin.
`replace-wait-freeze.json`, `replace-wait.diff` and
`peer-first-five-original.json` retain this lineage. At this correction checkpoint,
own implementation GREEN awaited the mediated port; no Cargo job was repeated.

## Receiving canonical-path oracle correction

The UI owner reran the exact18,604-byte corrected source in session `96326`:
**4 passed,1 failed**. Actual replacement and complete49-byte output equality
passed. The next assertion compared Engine's canonical file association with the
fixture's lexical `crates/ui/../../target/...` spelling. Existing
`io::files::canonical_save_path` and unchanged SMILES file tests already require
canonical associations; this was a path-oracle setup error, not a product defect.

After Root reviewed that contract, only the expected expression changed to
`fs::canonicalize(&output)?.to_str()`. Full Unicode/path equality, every byte
assertion, input and the earlier bounded wait remain. Reversing that one
substitution reconstructs the exact18,604-byte source. Its original source and
the actual1,917-byte failure log, SHA256
`7fdc9f1bb7d10cea858122b47996dff4755267afc057e19dfa5c9e63239aa526`,
are preserved under ignored evidence before correction.

The current five-case source is18,624 bytes, SHA256
`8005c5a3a10fe4cdb5b5f9e82a9866aa94625fab2fe71e9aa628a1b5b700094a`.
`canonical-path-freeze.json`, `canonical-path.diff` and
`peer-before-canonical-path.json` record the precise delta/receiving provenance.
No implementation or new execution accompanied this correction. Own GREEN then
awaited the mediated source port.

## Independent real-source receiving GREEN

Root authorized the exact frozen UI source and two MCP backing files. Before
copying, the producer manifest was independently read and verified:2,602 bytes,
SHA256 `396cf30112702c45135206a6902d63fd63823d277a376bcf9c447b25acff6d3d`.
All listed sizes/hashes matched. Ten files were copied; the owned18,624-byte tests
were verified exact and never overwritten. The MCP pair also matched current Root
bytes. The unrelated Root Fit patch was explicitly excluded.

All six old fixture files reconstruct their pre-port bytes by removing exactly
one `smiles_records` default initializer. `view_tests.rs` retains the existing child
declaration and old test bodies; removing that initializer reconstructs the prior
bridge exactly. No fixture, oracle, asset, font, dependency, lock or SDK changed.
The before/after entries and six literal reconstruction checks are in
`target/smiles-record-ui/backend-port.json`,5,106 bytes, SHA256
`b18641e427acae8323372afb5570cde2f737de27f8d217511987c0cbae6666c6`.

Own session `97627`, exit0: **5 passed,0 failed,0 ignored**,127 unrelated tests
filtered. Compilation took4.45s and tests0.16s. Every current source assertion and
input was unchanged after port. This reaches real background inspection, literal
metadata, accessible readonly selection/copy, no text undo retention, navigation,
invalid ordinal, source ABA/identical Open, zero records, Unicode Save As/failed
existing-target retry/explicit replacement, exact49-byte Save and chooser reopen.
The unchanged raw-file and per-notation chemistry consumers run on actual saved
bytes/copied notation. These are headless actual-App and filesystem observations.

Own session `51652`, `cargo clippy -p bondline-ui --all-targets --locked --
-D warnings`, exited0 in14.33s. `cargo fmt -p bondline-ui -- --check` exited0.
No lint suppression or expectation change was needed. The producer's full UI
package passed181/0/1; it was not redundantly rerun here. Root owns the receiving
workspace gates and native acceptance.

The final handoff retains every RED/setup correction/GREEN log, exact source and
protected-file hashes, fixture reconstructions and no live handles. No native
window, OS clipboard, platform interaction, Git action or hosted workflow was
performed by this test owner. General chemistry/depiction, editable SMILES records,
conversion/export UI and complete reference screens remain outside this slice.
