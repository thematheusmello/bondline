# Status

2026-11-13: the persistent full M0–M5 goal is active. Full M0 and M1 are incomplete.
Public repository: https://github.com/thematheusmello/bondline . Continue this repository.

## Integrated checkpoint

Verified application source: c879f4c95964617a86cc2d1e466515c5aeb31f29. Its complete-source
MOL file feature follows codec9e3777e and fixture-checkout correction670178b. This documentation
checkpoint integrates that tested source into main; inspect Git log for the exact doc head.
The current feat/sdf-v2000 branch starts from the same source and has separate in-progress
SDF work. Do not reset, discard or advertise that uncommitted codec as finished.

Hosted application run43508330802 is terminal GREEN:
https://github.com/thematheusmello/bondline/actions/runs/43508330802 .
Mac315 tests (312 workspace plus three bridge), Linux313 and Windows307 pass, along with
three actual window smokes and CDXML/CDX/MOL sanitizer targets. The standalone codec
correction also passes all jobs in run43507660824 (Mac284/Linux282/Windows276). First-run
Windows failures in43504072080 were real Git fixture-byte conversion; the narrow -text
attribute fix leaves parser/oracle/assertions unchanged. See docs/ci.md and the checkout report.

Actual Mac MOL acceptance completed 21:21:39 UTC using source c879f4c and unchanged owned
binary SHA eeb98144f0fe0b7aaba24479d5950ec7de622efe9da985a02995830d3b387927, macOS27.2 arm64,
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

Actual-root cargo xtask ci passed312 workspace plus three bridge tests, rustfmt,
warnings-denied all-target clippy, layers, native licenses/assets and corpus. Log:
target/mol-files-ci-integrated.log. Scoped totals: IO70, commands70, adapters40, UI43.
Runtime closures remain153/164/219 for Mac/Windows/Linux and two bundled assets; no runtime
or lock change. Corpus checks one actual CDXML and one actual MOL input. CDX corpus and
chemistry calculation coverage remain missing; external formula/SMILES are consumer results.

The README/community presentation has original banners, real unedited native photos,
licensing, contribution/security/support and issue/PR guidance. Public GitHub image/render
checks pass; anonymous issue forms redirected to sign-in and remain unverified. Images show
an explicitly identified earlier build. New MOL acceptance did not alter them or the golden.

## In progress and remaining limits

SDF V2000 is the next active M1 codec slice. IO owns new source/tests/original fixture;
verification owns the external oracle and a corpus plan; root owns fuzz/CI/provenance/Git.
Initial missing-module RED and four initial GREEN tests exist, not a final feature pass.
Final hostile/writer/resource cases, independent consumers, ASAN and aggregate remain.
No application SDF command or multi-document flattening is being silently introduced.
Its data must retain empty/multiline/duplicate/DT/raw headers and exact ordered source spans.
The pinned RDKit full supplier loses DT-only data/records; independent raw-field checks must
remain separate from molecular consumer evidence. Never remove valid fields to satisfy it.

MOL saves currently preserve one CTAB/fragment and one page. Changed imported page size,
nondefault native layout, multiple fragments/pages and cross-format source metadata loss
refuse unsupported_content. Another isolated fragment is editable but not currently savable
as MOL; Undo restores eligibility. Explicit lossy export, broad imported graph editing,
query/stereo/3D, legacy isotope mass tables, SDF/V3000/RXN/SMILES/InChI remain M1 work.
The normative M ISO rule and RDKit legacy-delta quirk disagree; record both, do not claim
universal input agreement. General MOL rendering/editing is not delivered by the native
C–C/empty-to-first-fragment workflow. Interactive Windows/Linux remain unverified.

Full M0 inventory/dialog discovery/job addressability, connected construction/selection,
labels/styles, chemistry/CIP/naming, reactions/arrows/text, publishing/Office/printing,
WASM and signed distribution remain open. The576 qualified control rows are not a complete
modern inventory. Full commercial screen/workflow parity is not claimed. No release or
installer publication was requested/performed. See ROADMAP.md and docs/parity-checklist.md.

The native license blocker is resolved without broadening MIT/Apache/BSD. M5 still needs
full source/notice auditing, including windows-permissions0.2.4's missing upstream LICENSE
despite declared MIT. Browser Unicode-bearing dependencies remain unapproved. Golden SHA:
1b650231527f47db5dc7c368644939907d2b038d32f25e77578318f483521bbc. Do not regenerate it.

## Next exact actions and live work

1. Finish the current SDF codec tests and writer boundary review, then frozen-source ASAN,
   fmt/clippy/aggregate. In particular, a valid standalone MOL header starting$$$$ must not
   produce an invalid SDF wrapper. Root/IO coordinate the strict record/field/span contract.
2. Publish that codec checkpoint, pin its original open-licensed fixture for fetched SDF
   corpus checks with independent ordered-field/CTAB consumers, and follow its own hosted run.
   Do not mutate existing RDKit pins or invent a future commit hash. Corpus plan is research
   until agreed/implemented; no new SDF corpus pass is claimed yet.
3. Continue V3000, application CDX/general CDXML, RXN/SMILES/InChI, then M2 drawing/labels/
   styles, M3 chemistry/reactions, M4 publishing/clipboard and M5 depth/WASM/distribution.
   Remaining M0 gaps stay visible while the user-prioritized M1 work advances.

Completed application/codec runs43508330802/43507660824 are terminal; do not restart them.
Main's automatic duplicate run43509919220 at the older670178b may still be live; inspect
that exact run before acting. Its source is already independently GREEN; it is not the
c879f4c application run. No native/controller/CUA job remains. SDF agent Cargo work and
root sanitizer work have separate logs under target/; inspect current handles before restart.
Only root stages/commits/pushes. Development bundles/reports are disposable, never committed.
Future worktrees must keep separate Cargo targets; stale reuse previously tested the wrong
checkout. Keep the full project goal active and continue rather than recreating it.
