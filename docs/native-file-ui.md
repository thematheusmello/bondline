# Native CDXML workflow

The single-document File menu exposes Open, Save, Save As and Close Window. The native Mac
application menu/Quit and window close enter the same unsaved Save/Discard/Cancel lifecycle.
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

## Independent execution evidence

Native tests on Mac arm64/Metal create an empty drawing, Save As, edit, ordinary Save,
overwrite/Cancel, close/relaunch/Open, edit reopened data and Undo to the saved clean state.
Dirty Open Cancel retains state; explicit discard loads the chosen document. Malformed Open
after discard approval still retains the prior dirty drawing. Opaque source opens guarded,
refuses drawing and saves byte-for-byte. File Close/red close/CmdW/CmdQ/native Quit all enter
the close policy; Cancel retains, Save persists and exits, Discard exits. External Expat and
RDKit inspect actual saved pointer-created fragments, independently of Bondline's codecs.

The exact final executable SHA-256 is
`f331610a4689105b11495016c18513e3a2a69c04921acfeb82c3e25e93961b1d`.
Its final repeated test verifies actual AX Folder/Unicode-name entry, Save/Open, second-fragment
edit, Quit Cancel retaining dirty data, then Quit Save/exit. The two-fragment saved CDXML hash
is `b0272acb0013ed45ee2a09bd68989927245faff73bc9f61a7777b5cf30bab590`;
external XML/RDKit reports four carbons, two bonds and two CC/C2H6 molecules. App/CUA closed.
Detailed RED/GREEN commands and earlier distinct build scopes are in verification.md.
Disposable artifacts, screenshots and the full session record remain ignored under target/.

24 UI tests include typed input, bounded/stale listings, linked entries, unsupported paths,
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
