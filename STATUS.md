# Status

2026-11-13: the persistent full M0–M5 goal is active. Full M0 and M1 are incomplete.
Public repository: https://github.com/thematheusmello/bondline . Continue this repository.

## Integrated checkpoint

Main currently contains the verified MOL application c879f4c with native/hosted acceptance
documentation bac395d. SDF codec source fb86832a0c9b7744fb3ef8f8115c7ed296164505 and its
whitespace-attribute child 2d665de are published on feat/sdf-v2000. The codec now passes
all hosted jobs in run 43512236206: Mac 327/Linux 325/Windows 319, three actual windows and
four parser sanitizer targets. This corpus feature on feat/sdf-corpus follows 2d665de and
is locally verified. Its first hosted run43515232724 passed Mac/Linux/ASAN but failed
Windows because Git changed the pinned manifest to CRLF. A narrow metadata checkout
correction is now independently verified; its new hosted run follows publication.
Fast-forward main only after corrected checks pass; the failed run remains recorded.

Hosted application run 43508330802 is terminal GREEN:
https://github.com/thematheusmello/bondline/actions/runs/43508330802 .
Mac 315 tests (312 workspace plus three bridge), Linux 313 and Windows 307 pass, along with
three actual window smokes and CDXML/CDX/MOL sanitizer targets. The standalone codec
correction also passes all jobs in run 43507660824 (Mac 284/Linux 282/Windows 276). First-run
Windows failures in 43504072080 were real Git fixture-byte conversion; the narrow -text
attribute fix leaves parser/oracle/assertions unchanged. See docs/ci.md and the checkout report.

Actual Mac MOL acceptance completed 21:21:39 UTC using source c879f4c and unchanged owned
binary SHA eeb98144f0fe0b7aaba24479d5950ec7de622efe9da985a02995830d3b387927, macOS 27.2 arm64,
Apple M3 Max/Metal. All three native/controller processes exited successfully; CUA ended;
credentials remained memory-only. No native worker/listener remains. Independent file hashes
and the precise acceptance sequence are in docs/native-file-ui.md and docs/verification.md.

## What works, within the verified scope

- Thirteen typed commands share one registry across engine, CLI, stdio MCP and authenticated
  opt-in native loopback: inspect, close, open/save CDXML, open/save MOL, page.set_size,
  bond.create, undo/redo, render.capture, ui.controls and ui.file.
- MOLV2000 exact source preservation, strict elemental projection and canonical writing,
  including published valence/H, charge, radical and absolute-isotope fields. Owned history
  shares complete original bytes, baseline, three headers and captured drawing frame.
  Unchanged saves preserve every byte; regenerated coordinates have declared four-decimal
  precision. Actual files face separately run RDKit, Decimal and literal-field expectations.
- Atomic MOL I/O reuses the reviewed bounded ordinary-file/security/publication path. Shared
  commands preserve no-clobber/explicit overwrite, failed-import state, stale-open protection
  and older-save dirty state. File state/receipts distinguish associated encoding, filename
  extension and original source format.
- Actual native Mac explicit/default encoding choice, Unicode Save As, reopen/known Save,
  three retained source headers, malformed retention, layout refusal/Undo and exact failed
  overwrite retry. Misleading suffixes use the explicitly selected/associated decoder.
  Source metadata-loss conversion refuses publication. Close Cancel/Save and concurrent
  CPU/GPU reply draining pass on the actual app, not only egui tests.
- Trusted presentation receipts bind request ID and pre-save currentness. Intervening
  format/baseline changes remain invalid after values return. Native close-Save also requires
  encoding/revision/path/close epoch. Direct saves cannot fabricate a trusted UI receipt.
- Independent isolated C–C drawing, physical page controls, complete Undo/Redo, CDXML source/
  profile reopening, standalone bounded CDX syntax/property edits, CPU/GPU scenes/PNG and
  the prior approved golden remain verified only at their documented scope.

Latest actual-root cargo xtask ci passes 331 workspace plus three bridge tests, rustfmt,
warnings-denied all-target clippy, layers, native licenses/assets and corpus. Log:
target/sdf-corpus-ci-integrated.log. Earlier MOL acceptance passed 312 plus three bridge
tests at its own frozen source; target/mol-files-ci-integrated.log retains that evidence.
Runtime closures remain 153/164/219 for Mac/Windows/Linux and two bundled assets; no runtime
or lock change. New corpus tooling passes 331 workspace plus three bridge tests and checks
one CDXML, one MOL and one two-record SDF file from two pinned sources. CDX corpus and
chemistry calculation coverage remain missing; external formula/SMILES are consumer results.

The README/community presentation has original banners, real unedited native photos,
licensing, contribution/security/support and issue/PR guidance. Public GitHub image/render
checks pass; anonymous issue forms redirected to sign-in and remain unverified. Images show
an explicitly identified earlier build. New MOL acceptance did not alter them or the golden.

## In progress and remaining limits

SDF V2000 now passes 12 independent codec tests, final ASAN 10000 iterations and the full
actual-root codec aggregate 324 workspace plus three bridge tests. Source SHA is recorded in
docs/verification.md; log target/sdf-codec-ci-integrated.log. IO/oracle source is frozen.
No runtime dependency, lock, native UI, renderer or golden changed. The codec passes every
hosted check. The fetched SDF corpus now uses its real public pin.
No application SDF command or multi-document flattening is being silently introduced.
Its data must retain empty/multiline/duplicate/DT/raw headers and exact ordered source spans.
The pinned RDKit full supplier loses DT-only data/records; independent raw-field checks must
remain separate from molecular consumer evidence. Never remove valid fields to satisfy it.
The corpus cold fetch attests 690 bytes/f95838cbb55c107c649ce2eb99ac62e191f747ce6994a171e64019f8e6c53d1a;
normal/offline gates agree. Existing RDKit metadata is byte-identical. All manifests/notices/
global cache names are validated before cache/network I/O, with ancestry checks before mkdir.
Actual symlink escape RED and 25 GREEN xtask tests are recorded in docs/verification.md.

MOL saves currently preserve one CTAB/fragment and one page. Changed imported page size,
nondefault native layout, multiple fragments/pages and cross-format source metadata loss
refuse unsupported_content. Another isolated fragment is editable but not currently savable
as MOL; Undo restores eligibility. Explicit lossy export, broad imported graph editing,
query/stereo/3D, legacy isotope mass tables, application SDF/V3000/RXN/SMILES/InChI remain M1 work.
The normative M ISO rule and RDKit legacy-delta quirk disagree; record both, do not claim
universal input agreement. General MOL rendering/editing is not delivered by the native
C–C/empty-to-first-fragment workflow. Interactive Windows/Linux remain unverified.

Full M0 inventory/dialog discovery/job addressability, connected construction/selection,
labels/styles, chemistry/CIP/naming, reactions/arrows/text, publishing/Office/printing,
WASM and signed distribution remain open. The 576 qualified control rows are not a complete
modern inventory. Full commercial screen/workflow parity is not claimed. No release or
installer publication was requested/performed. See ROADMAP.md and docs/parity-checklist.md.

The native license blocker is resolved without broadening MIT/Apache/BSD. M5 still needs
full source/notice auditing, including windows-permissions 0.2.4's missing upstream LICENSE
despite declared MIT. Browser Unicode-bearing dependencies remain unapproved. Golden SHA:
1b650231527f47db5dc7c368644939907d2b038d32f25e77578318f483521bbc. Do not regenerate it.

## Next exact actions and live work

1. Publish the verified corpus metadata checkout correction and follow its new hosted run.
   Corpus run43515232724 is terminal failed on Windows manifest byte identity; the preceding
   codec run43512236206 is GREEN. Do not restart those completed jobs or older MOL runs.
   Main remains at bac395d until the new source's checks pass.
2. Continue the active V3000 codec in the persistent sibling worktree:
   /Users/sarenaberg/Workspace/doozle/open-source-world/bondline-mol-v3000
   branch feat/mol-v3000, base 2d665de, its own absolute Cargo target under that worktree.
   file_io owns Rust/tests/fixtures/fuzz/docs there. native_file_ui delivered its external
   oracle and ended all processes. Source is frozen with14 V3000/96 total IO tests,
   targeted clippy/fmt and final10000-run ASAN GREEN. Root's isolated full gate is still
   running; source is not committed/integrated. Do not discard it or reuse another target.
   Research handoff: root target/v3000-next-handoff.md. Never share Cargo targets or clean
   away an active checkout. Root's corpus evidence excludes those separate artifacts.
3. Finish V3000/application SDF workflows, application CDX/general CDXML, RXN/SMILES/InChI,
   then M2 drawing/labels/styles, M3 chemistry/reactions, M4 publishing/clipboard and M5
   depth/WASM/distribution. Remaining M0 inventory/dialog/job gaps remain visible.

Root corpus jobs are terminal; metadata checkout correction has actual isolated RED/GREEN
evidence in target/sdf-manifest-autocrlf-yoq1rmg2. No native/controller/CUA process remains.
Completed runs43508330802/43507660824/43509919220/43512236206 are GREEN;43515232724 failed
Windows. The next corrected push creates a distinct run, not a retry of passing jobs.
V3000 source is frozen in its sibling checkout; root full-gate handle49107 writes that
checkout's target/v3000-worktree-ci.log and may still run. Original CDX chemical fixture/
expanded-seed checks are separately in progress in root; do not discard their uncommitted
files. No unrelated reference/account/release action was performed.
Only root stages/commits/pushes. Development bundles/reports are disposable, never committed.
Future worktrees must keep separate Cargo targets; stale reuse previously tested the wrong
checkout. Keep the full project goal active and continue rather than recreating it.
