# Fresh document integration — 2026-11-15

This slice adds `document.new`, its native File → New Document binding and source-qualified
Save/Discard/Cancel workflow. The full M0–M5 goal remains active. New is a fresh session
within the current window; style sheets, templates and multiple document windows remain
missing. GitHub Actions stays disabled; no push, publication or dependency change.

## Sources and behavior

[Public Guide25](https://chemistry.beloit.edu/classes/programs/ChemDraw_25_manual.pdf),
printed13/39/77, documents New/untitled/default-style roles and Ctrl+N. Its reviewed PDF
is17,721,447B/SHA2563f1ebc429df07efdd47b6218a148dc346a215043f60bf89ee9a1cb90629bd5b3.
The same pin was rechecked; no reference help prose, template, font, icon or sample copied.
Apple's indexed New/Command-N table is platform convention evidence; its body returned429,
not reference-product Mac execution. Source register entries distinguish these limits.

Default A4, one-window replacement, retained tool/canvas/host and explicit unsaved-data
choices are original Bondline policies. New does not assign Engine::default: it advances
checked revision/generation/view counters in the existing Engine, installs new clean
source-free current/saved Drawing snapshots, clears association/history/IDs/extent/focus
and resets Fit. Old captured saves retain their lease and report publication truthfully;
older generations cannot associate or clean the new session. The typed closed parameter
object requires expected_revision; discard_unsaved defaultsfalse. Dirty/stale/pending-close/
committed-close refusals preserve state. This brings the registry to30 commands.

The native control uses existing ui.file actionnew/request qualification and primaryN.
create_ui_document validates the current Accepted New and executes the same command.
Changed SDF drafts resolve before a fresh New intent. New Save uses the existing actual
Save/job/receipt; only matching current trusted completion and actual thread retirement
can resume New. Cancel/failure/close/stale/poison disarm continuation while preserving
truthful publication/drain. A normal Save retry cannot silently restart New.

## Independent evidence

- [Six command cases](verification-new-document-command.md) cover every existing content
  kind, literal default state, history/IDs/tool/host/camera, actual post-New XML with
  unchanged Expat/RDKit consumers, stale deferred work, old captured saves and close guards.
- Root's three counter cases reject exhausted revision/generation/view counters atomically.
  Three presentation cases cover exact Accepted-New authority, missing/forged/replayed/
  cancelled requests, dirty refusal and source/close ABA; New supplies no Save/close ACK.
- Three actual CLI/stdio-SDK/authenticated-loopback cases discover the closed mutating
  command, preserve source bytes, reject stale/dirty calls, notify correctly and save a
  new independently consumed chemical drawing. These are additional postimplementation
  transport checks, not separately invented REDs.
- [Six independently authored actual-App cases](verification-new-document-ui.md) cover
  reachable menu/shortcut, SDF/SMILES→Drawing, dirty decisions, changed-draft decisions,
  Unicode real-file persistence, raw-event ordering and both input regressions below.
- [Eight finite worker cases](verification-new-document-workspace.md) exercise actual
  registered Save publication versus thread retirement, late cancel, close, replacement,
  poison and cancellation followed by an ordinary retry. No fake completion or new pool.
- [Independent review](research/new-document-review.md) captures exact final sources and
  finds no further material issue within its declared scope. This is not native OS proof.

The original minimal runtime RED was UnknownCommand after opening the complete777B SDF;
actual-App RED was missing New binding/control. Missing private/native Rust API compiler
stages are recorded separately. New-test setup corrections are explicit: supported view/
tool setup before readonly imports; bounded actionable menu wait for pinned egui's hidden
sizing pass; modal prompt disambiguation from existing 'New document' status; explicit
external replacement discard in the stale-source setup. Original old tests/oracles and
all state/byte/history assertions remain; frozen declarations and exact inverses survive.

## Reproduced input defects and fix

First, independent53938 reproduced an old SDF pointer drag becoming an unintended C–C
when Save→New ran at frame-start poll. A poll-only stamp fence fixed that local path,
passing the unchanged fifth test, but independent71635 reproduced the same ghost bond
when the shared registry applied New between UI frames. Both are real product failures,
not setup/timing-oracle corrections. Their logs/declarations are preserved.

The final guard replaces the poll-only flag. Existing DrawingState records the namespace
of the actually presented surface. A different New/Open namespace clears old gesture/
hover/hotspot and fences raw canvas authoring until the new surface is painted. Readonly
surfaces tag their laid-out source; invalid/unsupported Drawing returns do not acknowledge
an unseen canvas. Deliberate typed menu/key ordering remains, and a fresh following-frame
gesture works. The getter is Rust-only, same-Engine-local and unchanged by ordinary edits/
history/view/tool changes; it is not global identity or command authority. Full source/
revision/close CaptureStamp checks on jobs and receipts remain unchanged.

Unchanged final six28,497B/SHA256
fd423e4632d6719db5fb6e1b012a493e47990ddb5b64b56047bb3eb7762c66a4 pass producer52121 and
independent84175. Original26,124-byte five-case prefix and22,237-byte corrected four-case
prefix remain exact; both REDs are separate from intermediate four/five GREENs. A new
namespace API missing-method compile RED precedes Root9173's literal New/Open versus
ordinary-edit/history/navigation test GREEN. No parser/fixture/golden was changed.

## Aggregate verification record

Root focused checks pass: counter3, presentation3, independent command6, native namespace1,
wire3 and delivered-control cases. The first aggregate91017 stopped at warnings-denied
clippy (8-argument helper); passing existing FileJob reduced the signature with exact body
reconstruction, without a waiver. The next aggregate59608 stopped at the old MCP16-control
count after the new17th binding. Both that count and the corresponding core count advance
exactly to17; focused checks pass, with every existing assertion retained.

Producer's first full final-source UI run68304 had two old readonly workflow timeouts.
Both unchanged cases pass individually and the one unchanged default follow-up38580 passes
210/0/1. No source, timeout or assertion changed between those runs; the cause remains
unestablished. That failed run is retained, not relabelled or hidden by the later result.
Independent final UI clippy/fmt pass. Root's final aggregate and actual native evidence are
recorded below; earlier focused results alone do not prove either.

No Cargo.lock/dependency/SDK/font/asset/committed fixture/external oracle/approved golden
change. Corpus remains8 licensed files/5 bundles, with general CDX and chemistry gaps.
No new fuzz, hosted Actions or interactive Windows/Linux execution is claimed. Receipts
under target/new-document and isolated worker targets are disposable, with hashes/commands/
terminal results retained in the committed scoped records. No private token is committed.

## Final receiving gate

Root7556, literal pinned venv interpreter and Rust1.94.1:

```sh
BONDLINE_PYTHON="$PWD/.cache/oracles/format-python/bin/python" CARGO_INCREMENTAL=0 cargo xtask ci
```

Terminal exit0, **1090 passed/0 failed/1 ignored**,156 result lines. Formatting, all-target
warnings-denied clippy, workspace/vendor tests, layers, unchanged licenses and corpus pass.
Log `target/new-document/root-final-ci-2.log`:131044B/SHA256
475cc13c71d1d9a4b9527c16e9f04fbf2efccef051fa625da1a045f3a587723b.
The sole original ignored visual capture remains ignored. The first two aggregate failures
above and unexplained old UI timeouts remain separate evidence; no assertion/timeout was
relaxed. A receipt-generation Python typo happened after the passing gate and was corrected;
it was not a product/test failure or a reason to change source.

## Actual native Mac acceptance

The user explicitly selected **Open Bondline on the desktop**. Root9268 builds the final
`bondline-ui` binary successfully; CUA launches and raises the ignored original development
bundle `target/new-document-native/Bondline.app`, identifier
`org.bondline.new-document-review`, observed PID52347. No control channel is enabled.
Executable22,095,864B/SHA256
e38d0e91e7be6f0bba142bd5498d7d614eeb713fb9f3e03cc892c5f6038d2105.
The build receipt pins final core/UI sources to the reviewed frozen sources.

Actual native pointer/keyboard/chooser/AX and screenshots establish this bounded workflow:

1. Open the unchanged740B mixed-version SDF copy, showing2 records and Saved. Activate
   File → New Document. Untitled is a clean empty A4 drawing,0atoms/0bonds/0fragments.
2. F5 selects100%. Drag a visible C–C, then Cmd+N. Cancel retains the dirty2atom/1bond
   drawing. Repeat Cmd+N and Discard Changes: clean empty Untitled and default Fit return.
3. F5, draw C–C again, Cmd+N and Save. The real chooser writes `before-new-λ.cdxml`;
   only after completion does the window show a clean empty Untitled drawing.
4. Cmd+O selects the actual saved Unicode file. F5 shows the saved Single C–C,2atoms/
   1bond/1fragment on unchanged A4. Hotspot2 changes it to a visible Double and dirty;
   Cmd+Z restores the Single and Saved. Leave this latest app visible.

Actual output436B/SHA256
d08f2cd85e293f52d19093ab7330e01eac7e4e9da80d1f768518023f5626ef81.
Unchanged external `xtask/oracles/native_bond_rdkit.py`745B/SHA256
5f392d65c9c3bda16170cc73f2295b022d337ce7c6378e06a8e7ea3a9bb004e2 exits0 using pinned
RDKit2026.09.2. It verifies2 neutral carbons, one Single, H3/H3, CC and C2H6. Those last
results are external checks, not Bondline analysis features. Separate Expat assertions
verify one Poster page595.2755905511812×841.8897637795276pt, graph references and positions
(210,347)→(282,347). The native740B input still exactly matches its unchanged fixture,
SHA256e5cbdc14329c59bad55148c5de37058de90f651af10966c30cf8b89b3b5efd39.
Ignored build/native receipts retain these identities. This is one real native user
journey, not native proof of race/close timing, every source kind, multiple windows,
styles/templates, reference26.1 defaults, DPI calibration or Windows/Linux interaction.
