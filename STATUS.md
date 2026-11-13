# Status

2026-11-13. The persistent full M0–M5 goal remains active. M0 and M1 are incomplete.
Public repository: https://github.com/thematheusmello/bondline .

## This isolated checkpoint

Branch feat/mol-v3000-files follows standalone codec5caf6ce. The V3000 application slice
now retains decoded version, source/header/frame/history, uses the existing13-command
registry and preserves its associated MOL version by default. Open validates the actual
V2000/V3000 family member. Save As offers both concrete versions. Exact same-version source
passthrough, full-precision supported writing and conservative conversion refusal have
independent IO/engine/CLI/stdio/loopback/UI evidence. New source.format remains mol;
file/receipt format distinguishes mol_v2000/mol_v3000. No runtime/lock/codec changes.

The actual sibling cargo xtask ci passes364 workspace plus3 bridge tests, fmt/clippy,
layers/licenses/assets and this branch's inherited two-file CDXML/MOL corpus. Log:
target/mol-versions-ci-integrated.log. A prior compile attempt stopped on disk-full;
target/mol-versions-ci-disk-full.log is preserved. Only disposable build caches were removed.
All107 IO and43 UI library tests pass;29 adapter tests use the pinned external RDKit.
See docs/verification-v3000-application.md for actual RED/GREEN and precision bounds.

Standalone codec5caf6ce's hosted43519338382 is terminal GREEN: Mac341/Linux339/Windows333,
three actual windows and five10000-run ASAN targets. This is not application evidence.
Newer fetched SDF/CDX corpus features are separate root source; no merged corpus pass is
claimed here. Root integration226396e passed349workspace+3bridge with the four-file corpus;
its own hosted43526538340 is still running at the last check. Fetched CDX98bc59a passes
all jobs in43526004010. Main remains the prior c797262 checkpoint until matching integration.

## Confirmed must-fix and partial native acceptance

Independent read-only review reproduced an existing save-order defect: older Save.complete
publishesV2; a newer same-pathV3 save publishes/applies; older Engine.finish then falsely
relabels the clean associationV2 while disk remainsV3. Next Auto rounds full-precision data.
The target-only probe/report is ROOT target/v3000-integration-review.md. Source is local,
not published as an accepted V3000 application feature. Root is implementing a per-Engine
save reservation from prepare through finish/drop on fix/save-completion-order. A second
outstanding Save must fail busy before writing; other commands remain available.

Native walkthrough deliberately stopped after this finding. Exact earlier binary
6addbfb7603e4e40b174a80d672279a1352d99a7989d0a0411fa11c051376a79;
source manifestf9a42db06b4543314d83a85d9e7c67269a1864085ba848c67247e1c64f4407ab.
Actual Mac Open family, three Save As formats/defaultV3, Unicode exact copy/known Save,
misleading suffix decoding, malformed retention/Cancel and clean Quit passed. These are
partial observations. Full-precision authored/reopen, downgrade refusal, empty→first-fragment,
V2 regression and dirty-close Save/Cancel remain unrun. The native PID38959/controller38630
exited0; CUA completed. No native/controller/listener or Cargo job is live in this checkout.
ROOT target/native-v3000-ui-handoff.md records exact artifacts and the23:24:48UTC stop.

## Continue exactly

1. Preserve both checkouts and their separate Cargo targets. Root alone runs Git. Finish
   root save-order TDD correction and its focused/full gates; bring it into this feature.
2. Run a corrected-source sibling aggregate. Freeze it, then have the sole native agent
   rebuild and complete actual V3000 acceptance. Earlier screenshots are partial old-source
   evidence, not final proof. Publish only the corrected verified source and inspect its
   own hosted run before integrating main.
3. Merge with newer root corpus features, retaining the positive CDX seed/max_len131072 and
   fifth V3000 target. Continue application SDF, wider CDXML/CDX, RXN/SMILES/InChI; then
   M2 drawing/labels/styles, M3 chemistry/reactions, M4 publishing and M5 depth/distribution.

Full graph editing/rendering, query/stereo/3D, SDF application/version dispatch, RXN/SMILES/
InChI, connected drawing, general labels/styles, chemistry, reactions, publishing/Office,
WASM and signed distribution remain open. One-fragment/page/layout/source-loss guards
apply. M0 dialog/job/complete26.1 inventory and interactive Windows/Linux remain incomplete.
Native strict-license blocker is resolved; runtime closures153/164/219 and2assets are
unchanged. Later notice/browser audits remain M5 work. The approved golden remains
1b650231527f47db5dc7c368644939907d2b038d32f25e77578318f483521bbc.
No proprietary binaries/assets, external messages, release, installer or signing action.
