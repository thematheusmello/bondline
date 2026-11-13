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

## Root integration: fetched CDX corpus and standalone V3000

Fetched-CDX source98bc59a follows30ae30e and is published on feat/cdx-corpus-checks.
Its hosted run43526004010 is terminal GREEN: Mac338/Linux336/Windows328, windows and4ASAN.
The feature now cold-fetches the
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
runtime closures153/164/219 and two assets are unchanged. Its own hosted run is43526004010;
prior local root tests are terminal.

The root feat/m1-integration branch now combines that corpus source with standalone
V3000 codec5caf6ce. Conflict resolution retains both full verification histories, the
fetched corpus, original positive CDX seed/max_len131072 and the fifth V3000 fuzz target.
No codec source changed during integration. Combined source226396e passes its own actual-
root349+3 gate and every hosted43526538340 job: Mac352/Linux350/Windows342, windows and
five ASAN targets. Older isolated successes are not substituted. Main is still c797262;
the following safety correction will be verified before the next main update.

## Current root: save completion safety

Branch fix/save-completion-order follows226396e. Independent review reproduced an older
save relabeling a newer disk format; V3000 Auto could then round to V2000. One Engine save
reservation now lasts from prepare through finish/drop, including the published-result gap.
Another Save fails busy before I/O; edits/queries/Open/capture and other Engines remain usable.
The same review exposed historical close receipts approving after later format/baseline ABA.
Private current-completion acknowledgment now survives the operation's own update but is
permanently cleared by later real changes; UI poll requires it before renewing close.

Actual REDs and retained old tests are in docs/verification.md. Root cargo xtask ci now
passes356 workspace plus3 bridge tests and all gates, including the four-file corpus:
target/save-completion-ci-integrated.log, session96453 terminal0. Six new lifecycle tests,
one cross-executor proof and the exact post-finish UI regression pass. No IO/parser,
runtime/lock, renderer, font or approved golden changed. Corrected native interaction and
this source's own hosted run remain pending. Native processes/CUA are currently stopped.

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
branch feat/mol-v3000-files, local commit3b1fada after5caf6ce; its own target directory.
NEVER share root build outputs or delete an active checkout. Earlier wrong-checkout reuse
was fixed by Cargo's runtime workspace locator; evidence must identify the invoked root.

Version-aware owned IO, shared commands, exact association/default Save, explicit V2000/
V3000 Save As and family-based Open are locally committed. IO107 tests, core
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
That source predates the confirmed save/close correction. Its new exact V2/V3 concurrency
regression currently fails in target/v3000-save-order-red.log and is intentionally uncommitted
until correction integration. No failing code was committed to main or published as verified.
Partial old-source native Open/copy/format-choice/retention passed, then acceptance stopped.
Binary6addbfb7…/source manifestf9a42db0… and exact sequence are in root
target/native-v3000-ui-handoff.md. PID38959/controller38630 exited0; CUA completed.

Root alone stages/commits/pushes. file_verification and native_file_ui froze root correction
tests/UI; no job remains from them. file_io completed primary RXN research and now owns only
the new legacy RXN codec in persistent /Users/sarenaberg/Workspace/doozle/open-source-world/bondline-rxn,
branch feat/rxn-v2000 at3b1fada. It has a separate absolute Cargo target, incremental off and
dev/test debug0 to bound disk use. Do not clean/delete/share any active checkout/target.
This new work is a source-preserving role/container library, not application reactions;
V3000 reaction wrappers remain separate. Primary REAGENT versus RDKit AGENT, default role
folding, mixed MOL versions and consumer EOF/data loss are recorded in target/rxn-next-handoff.md.
Ask file_io for current jobs before starting overlapping work. All completed hosted runs
above are terminal; no completed job was restarted.

## Remaining limits and next exact steps

1. Commit/publish root's locally verified save-order/close-receipt correction and inspect
   its own hosted run. Fast-forward main only after matching GREEN; do not substitute older runs.
2. Bring the correction into sibling3b1fada and turn its exact V2/V3 regression GREEN. Run
   the actual sibling gate; then freeze/rebuild and complete sole-agent native acceptance.
   Keep partial earlier evidence separate. Publish only corrected accepted application source.
3. Integrate its application source with newer root corpus changes,
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
