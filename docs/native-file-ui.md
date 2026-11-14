# Native structure-file workflow

The single-document File menu exposes Open, Save, Save As and Close Window. The native Mac
application menu/Quit and window close enter the same unsaved Save/Discard/Cancel lifecycle.
That policy now lives in the shared document.close registry command. UI presentation reads
its status/epoch; it no longer owns a private closing flag or close approval.
Cmd/Ctrl O, S, Shift-S and W are wired; Mac O/S/W/Q have native evidence. The exact Save As
shortcut and Windows/Linux gestures remain unverified. Escape cancels the chooser/close prompt.

File operations dispatch registered commands through the shared CommandExecutor off the UI
thread. One UI file job is shown with cancellation; UI/loopback share two worker permits.
Late results cannot replace a newer document. Close after Save requires matching receipt
operation/revision/path/format, trusted presentation ID/currentness, current clean state and
unchanged close epoch/revision; an old receipt never
authorizes discarding newer edits. Published-byte errors keep the window open and state their
outcome. The worker cannot interrupt OS syscalls. This is not an autosave/recovery journal.

The original egui chooser has editable Folder and File name, Go, Up, directory/file rows,
Open/Save/Cancel, dirty-discard and explicit Replace File confirmation. It starts at the user
home directory, falling back to current directory only if home is unavailable. Subsequent
operations reuse the document/last successful directory. Input is bounded to32768 UTF8 bytes,
rejects NUL, and offers explicit CDXML/MOL V2000 encoding. Open lists both extensions; a
recognized filename supplies a hint, while the selected decoder validates actual contents.
The dropdown can override a misleading name. Save As defaults to the associated encoding;
changing it updates the suffix. Missing extensions are added and mismatched save suffixes
refuse. Known Save keeps the authoritative path/encoding even when its extension disagrees.
Failed retries retain the exact attempted basename/path and explicit overwrite decision.
A persistent bounded directory worker sorts entries, follows selected links, rejects
unrepresentable names and reports the2048-entry limit; complete paths can address other files.
Generation checks ignore stale listings. OS directory calls may finish after cancellation.

Folder/name labels have associated accessibility nodes and bounded SetValue handlers. The
reviewed native AccessKit bridge delivers those actions; actual Mac AX entry with a Unicode
filename is verified. General VoiceOver/navigation workflows and other controls remain open.

## MOL execution evidence

Source `c879f4c95964617a86cc2d1e466515c5aeb31f29`, owned executable SHA-256
`eeb98144f0fe0b7aaba24479d5950ec7de622efe9da985a02995830d3b387927`, macOS 27.2 arm64,
Apple M3 Max/Metal. The copy remained unchanged throughout. Completed 2026-11-13 21:21:39 UTC.
All 43 UI tests, the 312-workspace/three-bridge aggregate and hosted checks pass separately.
Actual native observations establish this finite workflow:

- From a fresh clean session, an actual C–C drag followed by Quit, Save, explicit MOL
  selection and AX Unicode filename saved a file and exited successfully. Its own
  CDXML-to-MOL association change correctly renewed the trusted close-Save receipt.
- Actual Open rendered the saved structure; known Save and MOL-default Save As retained
  exact bytes. Malformed Open retained complete state. A second isolated fragment caused
  an explicit layout refusal without replacing saved bytes; Cancel/Undo restored eligibility.
- Opening an original empty MOL, drawing its first fragment and saving retained all three
  supplied headers and the captured frame. Source modified and saved clean are distinct.
- An explicit decoder override opened MOL named associated.cdxml. Known Save and a real
  directory-target failure/restore/Replace retry kept its exact path, encoding and bytes.
  MOL-to-CDXML Save As refused metadata loss and published no destination.
- XML named misnamed.mol first failed MOL decoding, then explicit CDXML retry opened it.
  Actual editing, Quit Cancel and Quit Save preserved the correct associated CDXML encoding.
- Two concurrent 2048-square CPU/GPU captures were still pending after Close Approved.
  The native process waited, published both complete replies, then exited successfully.

Independent RDKit/Expat reads confirm these actual files:

| Artifact | Bytes | Independent result | SHA-256 |
|---|---:|---|---|
| Native MOL and Unicode copy |234|CC / C2H6|494e85ead31dce31ffc9b68774bf68f00910349df591c6f66af74c540ff8cfda|
| Edited original-empty MOL |296|Three retained headers; CC / C2H6|70617705ea99cc7b39be0242c1fd77a6fc6ed36acd64d8b9618a1828fe2e5dd1|
| MOL with .cdxml suffix |282|Exact original ethane fixture|828c1b58b433ed3778bdb0ce7e7d02315f45850507c976a55c82fcebeebf57fc|
| Edited CDXML with .mol suffix |720|Two CC / C2H6 fragments|0e06ff075fa84c98fbf64c833811e304a46ce307a3af9f0c32ef523673598ee4|

Evidence logs: target/native-mol-ui-handoff.md, target/native-mol-ui/verification.json and
target/native-mol-ui-build.log. All three native/controller processes exited successfully;
CUA ended, credentials stayed memory-only and no worker remains. Only the supported C–C
renderer and empty-to-first-fragment authoring were visually verified. General MOL rendering/
editing, interactive Windows/Linux and complete reference/platform-dialog parity remain open.
The approved golden and original README captures were not changed.

## Shared file-control execution evidence

The new frozen executable SHA-256 is
`4a5425b0db81cac37fdf1d6ba8e6417fae44e5f7093dc58e2582d3ce2be25ab2`
on macOS/Apple M3 Max/Metal. Its owned target/dev copy stayed unchanged while the aggregate
rebuilt normal target/debug artifacts. UI36 tests, targeted clippy/fmt and the full aggregate
265 workspace plus3 native bridge tests pass. These are separate from actual native checks:

- A fresh process was confirmed revision0/undo0/redo0/fragments0/clean before and after first
  CUA selection. Remote Open/Cancel and File-menu Open addressed the same inspectable request
  states. Stale Cancel retained the newer picker; local Escape sent its exact shared Cancel.
- CmdShiftS opened Save As, actual AX Folder/Unicode-name entry saved a493-byte one-bond
  document, and a second observed drag plus CmdS saved the associated path at revision2.
  CmdShiftS on that named document opened a new Save As picker instead of ordinary Save.
- Native malformed Open retained exact document/path/history and addressable retry state.
  A disposable destination changed into a directory produced a real Save failure; restoring
  the target and choosing the fixed-path Replace retry saved through the guarded command.
- A third committed fragment was independently confirmed dirty at revision3. Actual CmdQ
  Cancel retained it; CmdQ Save persisted and exited0. A fresh native Open visibly showed
  all three saved bonds.
- Two real2048 CPU/GPU captures were pending when remote Close returned Approved. The process
  remained alive, delivered both complete successful replies and then exited0.

Independent ElementTree/Expat and external RDKit read the actual1125-byte
`target/native-shared-file-controls-λ.cdxml` as three CC/C2H6 fragments, SHA-256
`771fdaf88be2129d633692bfca5a81627885c6bb7022777ca5c1b346016f7347`.
A separate integrating-agent read confirmed the same bytes/hash/chemistry. All native and
controller processes exited0; the sole CUA task completed. Credentials stayed memory-only.
Logs/handoff remain ignored under target/native-file-controls-*; none is a product asset.

The first observed process unexpectedly already had revision3 before recorded authoring;
its cause is unresolved and it is excluded from clean-startup evidence. It was closed before
the independently confirmed fresh process above. A later CUA stale-app guard was resolved by
rechecking the exact unchanged bundle hash and refreshing the UI state/indices. No product
source was changed during acceptance. Interactive Windows/Linux, full dialog-field discovery,
VoiceOver, pending-gesture/Q ordering and broader reference parity remain unverified.

## Shared close execution evidence

The corrected native build SHA-256 is
`68048eaa0d11c2559d7a51dabf2b72fcc95790a54ff244eae5089248838334c3`
on macOS/Apple M3 Max/Metal. All28 UI tests and warnings-denied clippy pass. Core47 and
CLI/MCP/socket27 targeted tests separately exercise conditional policy and worker/reply
barriers; they do not substitute for the following actual native execution.

- A pointer-created committed fragment became dirty. Remote Request displayed the decision
  prompt; remote Cancel dismissed it. Stale Discard and malformed Open retained the revision,
  source/history/path and existing fragment.
- File Close/Cancel, CmdW/Escape, red Close/Cancel and CmdQ/Cancel retained the committed
  document. CmdQ/Save used actual AX Folder/Unicode-name entry, saved and exited successfully.
- Native Open displayed the saved bond. A separately observed later drag was confirmed by
  Inspect as revision2/undo1/fragments2. Quit/Cancel retained that exact state; native
  application-menu Quit/Discard then exited without changing the saved one-fragment file.
- Clean ordinary Save As followed by CmdQ cancelled the obsolete picker and exited. A
  deterministic input regression additionally proves a queued Enter cannot accept Save
  after Approved/Committed; pending actions and start/result failures retain the window.
- Two real2048-pixel CPU/GPU captures were started concurrently. Remote explicit Discard
  returned Approved while the process was still alive and both replies were pending. Both
  complete replies arrived successfully, then the process exited0. This demonstrates real
  shared-work/reply draining; busy-prompt buttons have deterministic input coverage.

Independent ElementTree/Expat and external RDKit read the actual493-byte
native-shared-close-final-λ.cdxml as one fragment, CC/C2H6, SHA-256
`9f72459be089ef815f5292478a80e105e4ccfef1b56f57914067f419735204eb`.
The file stayed unchanged after later Discard. Controller credentials stayed memory-only;
all native/controller processes ended and the sole CUA task completed. Disposable evidence
and full chronology remain under target/native-shared-close-*; none is a shipped asset.

One earlier attempt batched an unobserved drag with Quit and exited cleanly. Its ordering
is ambiguous; the isolated committed edit above is the protection evidence. Immediate
pending-gesture/Q ordering is not verified. An earlier a4303108... build predates the picker
correction and is not substituted for this final hash. CUA observation after exit sometimes
auto-launched a new clean no-control app; those observation-created sessions were closed too.

## Earlier file-workflow execution evidence

Native tests on Mac arm64/Metal create an empty drawing, Save As, edit, ordinary Save,
overwrite/Cancel, close/relaunch/Open, edit reopened data and Undo to the saved clean state.
Dirty Open Cancel retains state; explicit discard loads the chosen document. Malformed Open
after discard approval still retains the prior dirty drawing. Opaque source opens guarded,
refuses drawing and saves byte-for-byte. File Close/red close/CmdW/CmdQ/native Quit all enter
the close policy; Cancel retains, Save persists and exits, Discard exits. External Expat and
RDKit inspect actual saved pointer-created fragments, independently of Bondline's codecs.

The exact executable SHA-256 for that earlier file-workflow checkpoint is
`f331610a4689105b11495016c18513e3a2a69c04921acfeb82c3e25e93961b1d`.
Its final repeated test verifies actual AX Folder/Unicode-name entry, Save/Open, second-fragment
edit, Quit Cancel retaining dirty data, then Quit Save/exit. The two-fragment saved CDXML hash
is `b0272acb0013ed45ee2a09bd68989927245faff73bc9f61a7777b5cf30bab590`;
external XML/RDKit reports four carbons, two bonds and two CC/C2H6 molecules. App/CUA closed.
Detailed RED/GREEN commands and earlier distinct build scopes are in verification.md.
Disposable artifacts, screenshots and the full session record remain ignored under target/.

At that checkpoint24 UI tests included typed input, bounded/stale listings, linked entries, unsupported paths,
SetValue, unsaved/discard/overwrite policy, modal authoring blocking, cancellation and stale
save receipts. Source completeness, retry-error clearing, shortcuts and native lifecycle
failures were observed RED before fixes. Input tests supplement actual native execution.

## Remaining parity and platform scope

This is an original structure-file chooser and single-document app, not a recreation of the
reference's complete File menu, platform dialog or window layout. File New, recent files,
multi-document switching, format selection, Revert and comments remain missing. Save retains
Undo history, unlike G25 printed14's description; exact26.1 semantics need further evidence.
Imported styles/labels/queries and connected construction are not delivered. Native rendering
only supports the declared C-C scene; small viewports can still hit its minimum-label-scale
guard. Full schema mapping is not a promise that every modeled element can be rendered.

Dock UI was unavailable to the verifier. A hide/minimize-action then CmdQ produced a visible
prompt, but pre-Q minimized state was not independently exposed. Do not generalize that into
Dock, logout/shutdown, atomic multi-document approval or Touch Bar hardware validation.
Force Quit/process kill cannot preserve unsaved edits. Interactive Windows/Linux workflows
remain unverified even when hosted window and filesystem tests pass.

## MOL V3000 corrected acceptance

Source1c53312's actual Mac author/save/reopen/refusal/close and concurrent2048 CPU/GPU
reply drain passed at2026-11-14T00:04:57UTC. The [application report](verification-v3000-application.md#corrected-native-acceptance-2026-11-14)
records exact binary/source identities, file hashes, original headers, Decimal/RDKit
checks, rejected harness parameters and complete cleanup. This covers supported elemental
C–C/empty-to-first-fragment workflows; general graph editing and Windows/Linux interaction
remain unverified. No new golden or reference-app asset was created.
