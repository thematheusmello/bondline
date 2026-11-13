# Status

2026-11-13: the persistent full M0–M5 goal is active. Full M0 and M1 are incomplete.
Public repository: https://github.com/thematheusmello/bondline .

## Recoverable state

Main currently has the native shared-file-control source65b5fb8 and subsequent original
README/community documentation through b2fd61a. Hosted run43496010615 is terminal GREEN;
Mac268/Linux266/Windows260 tests, three actual window smokes and both then-current parser
ASAN targets passed. The polished README has original banners, real untouched native
screenshots, licensing/contribution/security/support and issue/PR guidance. Public GitHub
rendering/images were checked; anonymous issue-form submission remains unverified.

The standalone MOLV2000 codec checkpoint9e3777e is published on feat/mol-v2000. Its first
hosted run43504072080 failed Windows because Git altered pinned fixture line endings;
Mac/Linux and all three ASAN targets passed. Correction670178b preserves fixture checkout
bytes with a narrow -text attribute. Parser, fixture bytes and assertions are unchanged.
An isolated core.autocrlf=true check has actual RED/GREEN evidence. Corrected run43507660824
is live: https://github.com/thematheusmello/bondline/actions/runs/43507660824 . Do not duplicate it.

This application-format feature checkpoint follows670178b on feat/mol-files; Git log
identifies the exact commit. All intentional product Rust is frozen and the actual-root
aggregate is GREEN. Native MOL acceptance is starting in the owned development bundle;
its agent owns app/controller/CUA lifecycle. No reference executable was acquired/inspected.
Source publication triggers a distinct application CI run; it is pending at this checkpoint.

## What is verified locally

- Thirteen typed commands share one registry across engine, CLI, stdio MCP and authenticated
  opt-in native loopback: inspect, close, open/save CDXML, open/save MOL, page.set_size,
  bond.create, undo/redo, render.capture, ui.controls and ui.file.
- MOLV2000 source-preserving parser/writer and strict elemental projection, including
  published valence/H, charge, radical and absolute-isotope handling. Complete source,
  baseline, three headers and coordinate frame survive owned snapshots/history. Unchanged
  saves preserve exact bytes; changed supported graphs retain headers/frame and round only
  within the declared four-decimal V2000 tolerance. External RDKit reads actual output.
- MOL atomic I/O reuses the reviewed CDXML bounded ordinary-file/security/publication path.
  Shared open/save commands preserve errors, no-clobber/explicit overwrite, stale-open
  protection and older-save dirty state. File state/receipts carry authoritative encoding,
  separately from original source format and filename suffix.
- Trusted presentation receipts carry request ID/currentness. Intervening format/baseline
  changes stay invalid after returning to old values. Native close-Save also requires
  encoding, revision, path and close epoch; direct saves cannot manufacture such a receipt.
- Shared File/Edit/Page bindings and guarded file-presentation requests. Headless sessions
  refuse UI presentation while explicit file commands remain available. One finite request
  slot plus two shared capture/file workers; cancellation never pretends published bytes
  rolled back. Shutdown drains registered workers/replies under the engine mutex.
- Independent isolated C–C creation, physical page controls and complete snapshot Undo/Redo.
  Source-preserving CDXML/profile reopening, standalone bounded CDX syntax/property edits,
  CPU/GPU scenes/PNG and one human-approved golden remain delivered at their prior scope.

Actual-root cargo xtask ci passes312 workspace tests plus3 native AccessKit bridge tests,
rustfmt, warnings-denied all-target clippy, layers, native licenses/assets and the corpus.
Log: target/mol-files-ci-integrated.log. Scoped totals IO70/commands70/adapters40/UI43.
Runtime closures remain153/164/219 for Mac/Windows/Linux plus two bundled assets. No runtime
or lock change. The corpus now checks one actual CDXML and one actual MOL input; CDX corpus
and chemistry calculations remain missing. See docs/verification.md for exact RED/GREEN,
independent consumers, sanitizer bounds and previous checkpoints.

## Wired and incomplete

The original native chooser offers CDXML/MOL encoding, extension hints with explicit decoder
override, encoding-aware Save As and authoritative known Save. UI tests43 pass, including
misleading names, malformed retention, format/close ABA and exact failed-retry filename.
Actual MOL app execution is pending here. Previous real Mac CDXML/drawing/close acceptance
uses source65b5fb8 and binary SHA4a5425b0db81cac37fdf1d6ba8e6417fae44e5f7093dc58e2582d3ce2be25ab2;
new native evidence must name its own exact binary. Interactive Windows/Linux remain unverified.

MOL supports one CTAB/fragment and a preserved page. Changed imported page size, nondefault
native page layout, multiple fragments/pages and cross-format source metadata loss refuse
unsupported_content. Appending another isolated fragment is editable but cannot currently
save as MOL; Undo restores eligibility. Explicit lossy export, general imported graph edits,
query/stereo/3D/legacy isotope mass tables, SDF/V3000/RXN/SMILES/InChI remain M1 work.
The documented normative M ISO rule and pinned RDKit legacy-delta quirk disagree; separate
input/output diagnostics retain both facts. Do not claim universal consumer agreement.

Full M0 inventory/dialog discovery/job addressability, connected construction/selection,
labels/styles, chemistry/CIP/naming, reactions/arrows/text, publishing/Office/printing,
WASM and signed distribution remain open. The576 control rows are qualified public-source
coverage, not a complete modern menu inventory. Full commercial screen/workflow parity is
not claimed. No installer/release publication was requested or performed.

The original native license blocker is resolved without broadening MIT/Apache/BSD. M5 still
needs full source/notice auditing, including upstream windows-permissions0.2.4's absent
LICENSE despite declared MIT. The browser Unicode-bearing graph is unapproved. The approved
golden SHA remains1b650231527f47db5dc7c368644939907d2b038d32f25e77578318f483521bbc; do not regenerate it.

## Next exact actions and jobs

1. Complete native MOL acceptance using the frozen source: create/save/reopen, explicit
   format choice/Unicode Save As, byte/header/frame retention, malformed/unsupported refusal,
   known Save, Cancel/Save close and worker drain. Record binary/file hashes and independent
   consumers; fix real failures and rerun affected/full gates before publishing source.
2. Follow corrected codec43507660824 and the forthcoming application source CI. Fast-forward
   main only after the corresponding checks are GREEN. Keep completed jobs terminal; no reruns
   merely because a later session starts. Root alone owns Git integration/publication.
3. Continue M1 with source-preserving SDF records/V3000, application CDX and general CDXML
   mapping/editing, then RXN/SMILES/InChI. file_verification is preparing a bounded SDF grammar/
   independent-consumer handoff under target/, without product changes during this freeze.
4. Proceed through M2 drawing/labels/styles, M3 chemistry/reactions, M4 publishing/clipboard
   and M5 depth/WASM/distribution. Keep remaining M0 inventory/dialog/job gaps visible.

No root Cargo/fuzz job remains live after the passing aggregate. native_file_ui owns the
pending native process/controller/CUA handles; inspect its handoff before touching them.
Other agents' IO/adapter source is frozen and their test processes/sockets have ended.
Development bundles/reports under target/ are disposable and never committed. Future
worktrees must keep separate Cargo targets; stale reused build paths previously validated
the wrong checkout. Continue this repository and goal; do not restart or mark it complete.
