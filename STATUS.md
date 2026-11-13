# Status

2026-11-13: persistent full-project goal active. Full M0 is NOT complete.

Public repository: https://github.com/thematheusmello/bondline . Main is the verified
b1e0f4b checkpoint; current feature integration is on feat/cdxml-file-workflow. Do not
restart or discard the branch. Intentional changes are committed in small scopes; inspect
Git log for the final UI hash. No release/installer distribution or reference account action.

## Current integrated scope

- Native Rust egui/wgpu workspace, eight typed commands shared by CLI, stdio MCP and opt-in
  authenticated localhost. Registry: inspect, page.set_size, bond.create, undo, redo,
  render.capture, document.open_cdxml, document.save_cdxml. No raw source XML in results.
- Independent C-C single-bond drag with three explicitly authored H per carbon, terminal
  labels, physical page controls and exact Undo/Redo. No connected graphs/chains/rings yet.
- CPU/GPU canvas/PNG and the single human-approved golden. Its SHA remains
  1b650231527f47db5dc7c368644939907d2b038d32f25e77578318f483521bbc.
  Broader rendering/styles/labels and additional approved scenes remain missing.
- Bounded standalone CDXML parser, exact opaque source, checked attribute edits and native
  elemental writer. Full native profile requires complete input/re-encoding equality; do
  not trust producer name. Pinned v0 fixture must stay editable across encoder upgrades.
- Complete-source filesystem open/save: atomic no-clobber, explicit overwrite, cancellation,
  access metadata and publication/recovery errors. See native-file-io.md for actual limits.
- Native single-document Mac Open/Save/Save As, guarded dirty Open, overwrite confirmation,
  background work, receipt-checked safe close and editable native-profile reopening. Original
  CDXML-only chooser; not full reference File-menu/platform-dialog/window parity.
- Exact dirty baseline survives Undo/Redo and is counted with current/history storage.
  Current+saved essential data may exceed the128MiB threshold. Temporary worker/client
  snapshots and allocator headers are excluded. This is not an RSS cap. Save preserves
  history, an explicit difference from G25 printed14 pending exact26.1 evidence.

## Verification

Final local cargo xtask ci passes193 workspace tests plus3 native AccessKit bridge tests,
fmt, warnings-denied clippy (including that bridge), layers, native licenses/assets and
pinned corpus. Log: target/files-ci-integrated.log. Windows/Linux all-target workspace
source checks pass. Tests are independent Expat/Decimal/Pillow/RDKit, analytic geometry,
approved pixels and fixed-seed history properties; not just our own codec round trips.

Mac native UI final executable SHA:
f331610a4689105b11495016c18513e3a2a69c04921acfeb82c3e25e93961b1d.
Actual AX Folder/Unicode-name, Save/Open, editing, Quit Cancel and Quit Save pass on that
exact rebuilt binary. Broader native phases cover overwrite/discard, malformed retention,
opaque byte-exact copy and red close/CmdW. External XML/RDKit confirms the final saved
file's two CC/C2H6 fragments, SHA b0272acb0013ed45ee2a09bd68989927245faff73bc9f61a7777b5cf30bab590.
See docs/native-file-ui.md and docs/verification.md. App/CUA session closed; no native
listener or file-worker handle remains. Protect startup control tokens; never put them in
reports. target/dev bundle and target/ evidence are disposable, never committed.

Hosted baseline run43451706668 succeeded on all three OSes, actual windows and Linux ASan:
https://github.com/thematheusmello/bondline/actions/runs/43451706668 . It fixed missing Linux
libxkbcommon-x11-0 after first-run RED. Reported adapters: Mac Metal/Apple Paravirtual,
Windows Dx12/Microsoft Basic Render Driver(Cpu), Linux Vulkan/llvmpipe(Cpu). API/window
smoke is not interactive Windows/Linux chemical editing or hardware performance.

Feature library run43458663480 at34d577d passes native Windows seven io unit +six file
integration tests, Ubuntu and ASan, including DACL/exclusive temp/Zone.Identifier/recovery:
https://github.com/thematheusmello/bondline/actions/runs/43458663480 . Its overall result is
unsuccessful: Mac job131999452071 was never acquired after multiple attempts, then cancelled
for hosted arm64 capacity constraints (empty steps/runner). No Mac test failed or ran in
that job. Exact provider annotations are in ignored target/file-io-hosted-macos-*.json.
This run predates the final command/UI/AX integration. Complete feature head ccf8c50 is now
in confirmed active run43463687047:
https://github.com/thematheusmello/bondline/actions/runs/43463687047 . Inspect that exact live
handle before retrying, merging or claiming its success.
Do not restart an unobserved live run or infer success from queue/configuration.

Native Mac io44 tests and actual Linux arm64 Docker17 file tests pass. Linux invalid-byte
paths/FIFO/ACL cases use container-native /tmp with repository read-only, not fake APFS
fixtures. The isolated pinned ASan harness also passed10000 runs, seed3641879538, max_len65536,
timeout5, coverage3656/features10813/corpus669 entries175KiB/reported RSS178MiB; no failure.
It fuzzes parsers/source editing, not filesystem behavior. Locks retained existing versions.

## Provenance and remaining limits

The native license blocker is resolved without broadening MIT/Apache/BSD policy. Declared
runtime closure153/164/219 on Mac/Windows/Linux plus two hash/notice-checked bundled assets.
This is not a completed release source/notice audit: windows-permissions0.2.4 declares MIT
but upstream omits a LICENSE; exact metadata/author/pins and that gap are recorded. Browser
Unicode-bearing candidate remains unapproved. No new unsafe in model/parsers/commands/UI;
reviewed OS boundaries and exact original-source forks are documented in AGENTS and policy.

Public specs, G25,26.x release notes and pinned PhotoCraft/VoiceStudio/REA research remain
in docs/clean-room/sources.md and docs/research. User-authorized artifact/runtime research
uses original Rust/assets; no reference desktop executable was obtained or inspected.
Download center requires login. Observed Signals cloud slides are not desktop evidence.
The573-row inventory lacks complete modern menus/unnamed glyphs/context menus. No full
screen/design/tool parity is claimed. Additional modern inventory audit is delegated to
/root/file_verification; it owns only docs/research/modern-inventory-audit.md.

CDX/MOL/SDF/RXN/SMILES/InChI codecs, chemistry algorithms, connected construction, selection,
styles, templates, general captions/arrows and publishing/Office/WASM/installers remain.
Corpus has two pinned BSD RDKit files: one CDXML syntax/model/source check, one MOL integrity
only. No CDX or differential chemistry calculation coverage. External RDKit's ethane checks
are consumers, not an implemented Bondline formula/SMILES engine. Rendering is limited;
small viewports can hit label-scale guards. Full VoiceOver, Dock/session logout/shutdown,
Touch Bar and interactive Windows/Linux file gestures remain unverified. No autosave journal.

## Next exact actions

1. Inspect live full feature-branch run43463687047, preserving actual runner-capacity failures.
   Merge to main only after required integration evidence; no release publication requested.
2. Read the new inventory audit when ready; reconcile sourced rows and unresolved modern
   controls without claiming completeness from counts or cloud demo imagery.
3. Close remaining M0 architecture/inventory gaps, including named automation for remaining
   UI lifecycle/dialog actions and broader model/transaction requirements. Do not call M0
   complete from a license fix or one file workflow.
4. Continue M1 source-preserving structural edits/CDX/other formats and M2 connected bonds,
   snapping/order/labels with independent tests and actual native workflows. Keep M3-M5 full.

Product implementation agents finished; no confirmed Cargo/native/fuzz job remains. Only
research agent above is active, plus hosted run43463687047. Recheck authoritative agent/job
state on continuation. Do not duplicate live work.
