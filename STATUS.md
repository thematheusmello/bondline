# Status

2026-11-14. Persistent M0–M5 goal ACTIVE; M0 and M1 remain incomplete.
Repository: https://github.com/thematheusmello/bondline . Continue existing work.
Root owns Git, integration/publication and assignment of the sole native GUI owner.

## Current revisions and gates

Public main: bb6d5b9d2e6dd0d34e9b326dc9bf8061ffaff5a2, whole-SDF session/files.
Matching hosted run43575973213 is GREEN: Mac132390182235:590 (587workspace+3bridge),
Linux132390182293:588, Windows132390182069:580; three actual native-window smokes;
ASAN132390182305:seven10000-run targets. Main advanced only after matching GREEN.
Adapters: Metal/Apple Paravirtual device, Vulkan/llvmpipe CPU, Dx12/Microsoft Basic
Render Driver CPU. Window smoke is not interactive chemical-workflow acceptance.

Root receiving branch: feat/m1-smiles-sdf-integration, core HEAD eaac053.
Atomic history includes46c6f3c (observed Open-help omission/native record),5f92404
(explicit-H writer),e0ca5b5 (fixed-H normalization),d570aeb (bounded SDF query),eaac053
(existing SDF value setter). The next commit records combined fuzz and current scope.
Root receiving gates: writer598, normalization604, query618, setter634, all GREEN;
counts include3Mac bridge tests. Final session73303:631workspace+3bridge,0failed/ignored,
fmt/all-target clippy/layers/unchanged licenses/assets/corpus. Log
 target/sdf-field-set-integrated-ci.log77103B/SHA
 a8ce07f845ed489e6c182d1bfcab36d2d97f81dfade3bd75e718599f54f37ba2.
New source's hosted checks are pending. Keep main atbb6 until this source passes.

## Delivered receiving scope

Twenty shared typed commands: document.inspect/close; CDXML/CDX/MOL/SDF Open/Save;
page.set_size; bond.create/extend; history.undo/redo; render.capture; ui.controls/file;
sdf.record.inspect and sdf.field.set_value. UI/CLI/stdio/authenticated opt-in loopback
use the same registry. Headless engines do not pretend to open native dialogs.

SDF retains complete mixed-version source through current/saved/history/plans; state has
document:null and every-record metadata. The explicit query pages ordered duplicate/DT/
empty values within documented raw/JSON budgets. The setter changes one existing occurrence
off-lock, guards revision/generation/Arc/cancellation/close, commits one history step and
preserves untouched bytes. No-op shares source/index and keeps history/close state. Saved
versus original baselines and the full prepare-to-finish save lease remain distinct.
Actual edited files face unchanged raw/per-MOL consumers. No native field editor, record
operations, chemical preview or all-visible molecular layout is implemented.

SMILES: bounded original nonaromatic notation/source library, restricted noncanonical
explicit-H Fragment writer, named opt-in neutral fixed-H normalization. H/C/N/O/halogen
profile only; source IR and model None semantics stay unchanged. Literal/graph/limits and
unchanged pinned external unsanitized/separate sanitized consumers pass. No application
source backing/import/layout/clipboard, live inference, aromaticity/stereo/perception or
canonicalization follows. RXN is a source-preserving container, not reaction editing.

Combined normal-graph ASAN45229 passes10000, seed703592681/max_len65536/timeout5, on
five original seeds including4096atoms. Largest input4097B, not the configured64KiB limit.
Peer review found a test gap in source-to-normalized comparison; it was fixed before the
final run. Unique isotope markers independently recover every export endpoint despite
DFS order; external fixed-vector interoperability stays separate. Full-graph stable
clippy/fmt pass. See docs/verification-smiles-export-fuzz.md for exact hashes and limits.

Fetched corpus currently remains5 files/4 bundles; broader CDX, SMILES/InChI and chemistry
are missing. Runtime license blocker is resolved without broadening MIT/Apache-2.0/BSD:
154Mac/165Windows/220Linux runtime packages and2assets pass declared-license/hash/notice
checks. windows-permissions0.2.4 lacks an upstream LICENSE despite its MIT declaration;
this remains a release notice-audit question. No dependency/lock/version/asset change.

## Native and reference evidence

Exactbb6 native Mac acceptance now verifies C3/C4 literal-H authoring/history/Unicode
CDXML reopen, mixed777-byte SDF Open/known Save/Unicode.sd Save As/reopen/explicit decoder
choice and failed-second-record retention. Actual files face independent Expat/Decimal/
RDKit/raw checks. Additional690-byte and refusal/capture checks are controller-only after
another application took foreground; input stopped. Native new point-placement precision,
special2x/held/modal/origin/minimum-width and interactive Windows/Linux remain unverified.
The subsequent Open-help string fix lists.sdf/.sd and passes fmt/UI clippy; its new wording
has not been natively rechecked. See docs/verification-native-sdf-and-extension.md.
All owned app/controller/helper processes closed; CUA completion was observed exactly once.
The101-file evidence index was independently rehashed after exact CLI cache recovery.
Do not confuse its archived18-command CLI with current receiving20-command binaries.

Earlier Mac evidence covers pages/history/CDXML/MOLV2000/V3000/guarded CDX and CPU/GPU/PNG.
CDX always remains incomplete: unchanged Save works, faithful preview/edits/new encoding
refuse. Unknown imports never authorize lossy regeneration. General connected editing,
other bonds/rings/selection/transforms/styles, chemical analysis and publishing remain.

Installed reference26.1.0.6801/macOSarm64/tierunknown has six reports/catalogs:14menus,
503physical entries,36palette cells, not completed flows or a parity percentage. Bounded
SDF imports show both original records without an initial browser. Reference data retention/
layout/export are unknown. Raw reference captures/resources stay ignored; original assets
ship. Existing restoration exceptions remain recorded. Only approved golden SHA:
1b650231527f47db5dc7c368644939907d2b038d32f25e77578318f483521bbc.

## Active work and next steps

- reference_map: bondline-rxn/feat/smiles-corpus, base46c6f3c. Fifth mandatory fetched
  SMILES bundle, real publiceabe02f pin/three original licensed seeds, independent graph/
  external checks and all old safety negatives. Initial unknown-format/missing-fifth RED
  recorded; implementation in progress. Expected real default coverage8files/5bundles is
  not yet a pass. No Git/GUI/dependency changes; exact freeze/handoff required.
- sdf_field_review: bondline-mol-v3000/feat/smiles-aromatic-syntax, base3295e8e. Aromatic
  notation/source support only via published grammar and TDD; no chemical perception/model
  conversion. Must explicitly refuse aromatic input in fixed-H normalization. Coordinate
  enum/field changes with corpus and later Root fuzz before integration. In progress.
- native_acceptance and sdf_setter_review: completed read-only reviews, no live jobs.
  Source reviews and exact hashes are recorded in each feature's verification document.

Root Cargo/fuzz jobs are terminal. Query agents before resuming; their jobs can change.
No native controller/helper is live. Foreign GUI ownership was preserved; fresh ownership/
input checks are required before activation/input. Keep targets separate per worktree,
incremental0; Root default debug, sibling debug0. Disk is constrained (~1.9GiB, fluctuating).
Reclaim only inactive owned compiled artifacts, never sources/evidence/fixtures/other projects.
Root fuzz cache is now warm; exact native CLI is additionally archived outside debug outputs.

1. Commit/publish receiving branch and observe its own three-platform/ASAN hosted checks;
   advance main only after terminal matching GREEN. Never substitute previousbb6 results.
2. Review frozen corpus/aromatic slices independently, receive atomically, update all callers/
   fuzz for intentional notation changes and run new receiving/hosted gates.
3. Continue M1 then M2–M5: application SMILES/backing/layout, SDF chemical display/editor,
   RXN/reactions and visible roadmap. Full M0 inventory/dialog addressability, broader
   rendering/platform evidence remain open. Do not pause/block/complete at a checkpoint.

No new golden approval, PR/release/installer/signing, account/TCC or unrelated messages.
