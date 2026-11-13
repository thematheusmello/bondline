# Native CDXML workflow

The single-document File menu exposes Open, Save, Save As and Close Window. The native Mac
application menu/Quit and window close enter the same unsaved Save/Discard/Cancel lifecycle.
That policy now lives in the shared document.close registry command. UI presentation reads
its status/epoch; it no longer owns a private closing flag or close approval.
Cmd/Ctrl O, S, Shift-S and W are wired; Mac O/S/W/Q have native evidence. The exact Save As
shortcut and Windows/Linux gestures remain unverified. Escape cancels the chooser/close prompt.

File operations dispatch registered commands through the shared CommandExecutor off the UI
thread. One UI file job is shown with cancellation; UI/loopback share two worker permits.
Late results cannot replace a newer document. Close after Save requires matching receipt
operation/revision/path, current clean state and unchanged revision; an old receipt never
authorizes discarding newer edits. Published-byte errors keep the window open and state their
outcome. The worker cannot interrupt OS syscalls. This is not an autosave/recovery journal.

The original egui chooser has editable Folder and File name, Go, Up, directory/file rows,
Open/Save/Cancel, dirty-discard and explicit Replace File confirmation. It starts at the user
home directory, falling back to current directory only if home is unavailable. Subsequent
operations reuse the document/last successful directory. Input is bounded to32768 UTF8 bytes,
rejects NUL, and supports only CDXML; missing save extension becomes .cdxml and other extensions
refuse. A persistent bounded directory worker sorts entries, follows selected links, rejects
unrepresentable names and reports the2048-entry limit; complete paths can address other files.
Generation checks ignore stale listings. OS directory calls may finish after cancellation.

Folder/name labels have associated accessibility nodes and bounded SetValue handlers. The
reviewed native AccessKit bridge delivers those actions; actual Mac AX entry with a Unicode
filename is verified. General VoiceOver/navigation workflows and other controls remain open.

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

This is an original CDXML-only chooser and single-document app, not a recreation of the
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
