# Reviewed winit fork

Upstream winit 0.30.13, published archive SHA-256:
`a6755fa58a9f8350bd1e472d4c3fcc25f824ec358933bba33306d0b63df5978d`.
Published VCS revision: `e9809ef54b18499bb4f2cac945719ecc2a61061b`.
Apache-2.0; original source and notices are retained without relabeling. Four keyboard_*.svg
documentation illustrations are omitted because their separate attribution states CC-BY-SA4.0,
outside Bondline's bundled-asset policy. The attribution record stays intact; no runtime source
references these local images. They are not substituted or relabeled.

Only src/platform_impl/macos/app_state.rs changes. The existing application delegate now
implements applicationShouldTerminate: so native Quit requests pass through cancellable
CloseRequested events rather than immediately discarding an unsaved document. Window IDs
use the same object address as WinitWindow::id; non-Winit AppKit windows are excluded.
The existing delegate/class/menu stay intact. No callback swizzling or new dependency.

Before queuing close, minimized/hidden Winit windows are restored and brought forward so
the user can see save/discard/cancel. The two new unsafe calls are standard AppKit actions
on retained main-thread objects, with nil sender permitted by their API. This is a reviewed
extension to the existing OS boundary; no unsafe enters Bondline UI/model/command crates.
If no Winit window remains, termination proceeds. Otherwise NSTerminateCancel keeps the UI
alive and its existing client close approval ends the event loop after any required save.

Sources: Apple NSApplicationDelegate.applicationShouldTerminate(_:) and NSWindow APIs;
the original winit window/event/delegate implementation. See docs/clean-room/sources.md
and docs/verification.md for native RED/GREEN evidence. Current product is single-window.
Atomic multi-document approval, session logout/shutdown and Touch Bar hardware are not
established by the current acceptance workflow; Force Quit/process kill cannot save edits.

Upgrade by diffing against the exact archive, then repeating native Quit/cancel/save,
hidden/minimized visibility and all three platform build/window checks. Do not carry this
patch forward based only on a textual match or remove data-preserving close assertions.
