# Status

2026-11-13: the persistent M0–M5 goal is active. Full M0 and M1 remain incomplete.
Public repository: https://github.com/thematheusmello/bondline . Continue this repository.

## Integrated and published checkpoints

Main is at c797262: verified MOL application c879f4c plus the SDF codec/fetched corpus and
byte-preserving metadata checkout. Corrected run43518162847 is terminal GREEN on all jobs:
Mac334/Linux332/Windows324 tests, actual native windows and four parser ASAN targets.
Earlier run43515232724 failed Windows manifest checkout identity. The narrow -text fix
changed no parser, fixture, pin, assertion or license; the failure remains recorded.

Original positive CDX fixture30ae30e passes all jobs in43520991476: Mac335/Linux333/
Windows325, actual windows and four10000-run ASAN targets. The65750-byte original chemical
scene includes a65535-byte opaque property. Independent struct/RDKit checks and mutation
rejections are documented. Foreign/general/application CDX compatibility is not inferred.

Standalone V3000 codec5caf6ce passes all jobs in43519338382: Mac341/Linux339/Windows333,
actual windows and five10000-run ASAN targets. It remains on its published source branch,
with application integration in the persistent sibling checkout below. Neither preceding
run is evidence for the newly fetched CDX corpus or V3000 application changes.

## Current root feature: fetched original CDX corpus

The root feat/cdx-corpus-checks branch follows30ae30e. The feature now cold-fetches the
original CDX from that real public commit, checks its65750 bytes/SHA
63ef1a74133e45de915d5fd25b2ca3321a95dbe5114873a5a59f2f45217000d2 and retains its original
MIT OR Apache-2.0 provenance. All29 record spans/parents/length encodings match independent
Python struct; exact source preservation and the unchanged RDKit ethane guard pass.
Changed opaque bytes, bond order, spans and a different explicit scene fail. An independent
changed-bond consumer confirms C=C/C2H4, without weakening the original scene assertion.

Actual-root cargo xtask ci passes335 workspace plus three bridge tests, fmt, warnings-denied
all-target clippy, layers, licenses/assets and corpus. Log target/cdx-corpus-ci-integrated.log.
There are now four pinned files from three mandatory source bundles: one CDXML, one MOL,
one two-record SDF and one original CDX. Normal/offline results agree. All prior pins/notices
remain byte-identical. Isolated core.autocrlf RED/GREEN proves exact checkout of all three
manifests. Cargo.lock only adds the existing first-party bondline-cdx xtask edge. Native
runtime closures153/164/219 and two assets are unchanged. Its own hosted run follows this
feature's source publication; all local root tests are terminal.

## What works within verified scope

- Thirteen typed commands share one registry across engine, CLI, stdio MCP and authenticated
  opt-in loopback: inspect/close, CDXML and MOL open/save, page size, independent C–C create,
  undo/redo, capture, control discovery and file presentation.
- Source-backed CDXML/profile reopening, standalone bounded CDX/property edits, elemental
  graphs, exact complete-source history, CPU/GPU rendering and PNG have independent evidence.
- MOLV2000 retains source bytes, baseline, headers and frame; unchanged saves are exact.
  Strict elemental fields/writing use published BIOVIA2026, independent raw/Decimal vectors
  and external RDKit. Atomic I/O and deferred command guards retain state/old destinations
  on errors. File association is distinct from suffix and original source format.
- Actual Mac native MOL format choice/Save As, Unicode names, known Save/reopen, retained
  headers, misleading suffixes, malformed retention, retry/Undo, close and concurrent reply
  draining pass at c879f4c. Exact binary eeb98144f0fe0b7aaba24479d5950ec7de622efe9da985a02995830d3b387927,
  macOS27.2 arm64/Apple M3 Max/Metal. All three apps/controllers exited0 and CUA ended.
- SDFV2000 library retains exact records, ordered duplicate/empty/DT/raw data and embedded
  MOL. Independent raw-field and per-record consumers are distinct from RDKit full-supplier
  DT-only data/record loss. The original two-record fixture is fetched from its real pin.
- README has original banners, actual unedited native images, licenses, contributing,
  security, support and issue/PR guidance. Public render/images pass; anonymous issue forms
  redirected to sign-in and remain unverified. Images identify their earlier actual build.

## V3000 application integration and live work

Persistent sibling checkout:
/Users/sarenaberg/Workspace/doozle/open-source-world/bondline-mol-v3000
branch feat/mol-v3000-files, parent5caf6ce; Cargo target is that checkout's own target directory.
NEVER share root build outputs or delete an active checkout. Earlier wrong-checkout reuse
was fixed by Cargo's runtime workspace locator; evidence must identify the invoked root.

Version-aware owned IO, shared commands, exact association/default Save, explicit V2000/
V3000 Save As and family-based Open are frozen for integration review. IO107 tests, core
command tests, adapter29 CLI/stdio/loopback tests and43 UI library tests pass, with scoped
clippy/fmt. V3→V2 refuses before publication; V2→V3 also requires a raw-coordinate precision
proof. Source.format remains mol; file/receipt format records the actual version. No new
command name, runtime dependency, codec source or lock change. Its actual-worktree aggregate
now passes364 workspace plus three bridge tests, fmt, all-target clippy, layers, licenses
and that checkout's inherited two-file corpus. Native V3000 acceptance and hosted application
checks remain pending. Its aggregate first stopped during compilation
because local disk was full (target/mol-versions-ci-disk-full.log). Only the two Bondline
incremental caches were removed; source, fixtures and evidence were preserved. The rerun
completed successfully (session42649, sibling target/mol-versions-ci-integrated.log).

Root alone stages/commits/pushes. file_io delivered frozen IO and is now doing read-only RXN
primary-source research (target/rxn-next-handoff.md). file_verification and native_file_ui
completed their scoped source. native_file_ui is now sole native/CUA owner for the frozen
V3000 acceptance; its build/process handles will be in root target/native-v3000-ui-handoff.md.
Source branches and both worktrees remain recoverable; no completed hosted job was restarted.

## Remaining limits and next exact steps

1. Commit/publish the locally verified fetched-CDX feature; inspect its own hosted run and
   fast-forward main only after matching GREEN. Do not substitute the fixture's older run.
2. Finish the sole native agent's frozen-binary Open/Save/Save As/reopen/close and refusal
   checks. The sibling aggregate42649 has passed and is terminal; do not restart unchanged tests.
   Commit with provenance/control/format/parity/status docs, then run its own hosted checks.
3. Integrate standalone V3000 and its application source with newer root corpus changes,
   preserving both the positive CDX fuzz seed/bound and the fifth V3000 target. Run the actual
   merged checkout gates. Continue application SDF, broader CDXML/CDX, RXN/SMILES/InChI; then
   M2 drawing/labels/styles, M3 chemistry/reactions, M4 publishing/Office and M5 depth/release.

MOL application saving still requires one fragment/page and guarded layout/metadata.
General graph rendering/editing, query/stereo/3D and legacy isotope tables remain incomplete.
Connected construction/selection, labels/styles, chemistry/CIP/naming, reactions/arrows/text,
SVG/PDF/publishing/Office/printing, WASM and signed distribution remain open. The576 qualified
control rows are not a complete26.1 inventory. M0 dialog/job discovery and interactive
Windows/Linux workflows remain unverified. A window smoke is not interactive parity.

Native license blocker is resolved under the unchanged MIT/Apache/BSD policy. M5 still needs
full notice auditing, including windows-permissions0.2.4's missing upstream LICENSE despite
its MIT declaration. Browser Unicode-bearing dependencies remain unapproved. The sole
approved golden SHA is1b650231527f47db5dc7c368644939907d2b038d32f25e77578318f483521bbc;
never regenerate it without new human approval. No proprietary binaries/assets, external
messages, account actions, installer/release/signing publication or unrelated data changed.
