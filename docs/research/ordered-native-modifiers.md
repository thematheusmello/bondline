# Ordered native modifier input

2026-11-17, base071f34c. This is step3 of
[the native chain dependency plan](native-chain-plan.md), not a chain-tool implementation.
The original identity and catalogue12 delivered/128 disabled remain unchanged.

## Boundary and source

Pinned [egui Event](https://github.com/emilk/egui/blob/fe368bacc4a15697e347607e73a56c0299b3d42d/crates/egui/src/data/input.rs)
has modifier snapshots on buttons/keys, but none on PointerMoved. Pinned
[egui-winit](https://github.com/emilk/egui/blob/fe368bacc4a15697e347607e73a56c0299b3d42d/crates/egui-winit/src/lib.rs)
maps each actual Winit ModifiersChanged into RawInput.modifiers without an event.
That field is the latest accumulated state in this producer, despite the SDK's
start-of-frame field comment. Control down/up between pointer moves is therefore lost.

Both native renderer paths already call the common
[eframe Integration::on_window_event](https://github.com/emilk/egui/blob/fe368bacc4a15697e347607e73a56c0299b3d42d/crates/eframe/src/native/epi_integration.rs).
State::egui_input_mut is public. Extend the existing reviewed egui/eframe forks rather
than add a dependency, fork egui-winit, duplicate its platform mapping, or fabricate keys.
All retain their original MIT OR Apache-2.0 notices and pinned versions.

## Locked change

Add Event::ModifiersChanged(Modifiers), a full reported-state observation in the same
ordered event vector. In the common native wrapper, call upstream on_window_event first;
only for an actual Winit ModifiersChanged append its freshly mapped modifier snapshot.
Return the original EventResponse. Do not deduplicate reports, modify the mapping, alter
RawInput.modifiers, synthesize focus resets or change old gesture/shortcut semantics.
No new unsafe, registry command, model field, native tool binding or file authority.

RawInput::take moves these observations once; append retains order. Context input and
existing event filters retain them without turning them into Key events or applying
later states to earlier moves. The existing per-viewport State owns each queue; no global
side channel. Preserve actual focus ordering. A future drag starts from its PointerButton
snapshot, carries observed state between frames and cancels on focus loss. It must never
initialize earlier movement from a frame's final modifier snapshot. This increment does
not implement that consumer, defaults, chaining, Snaking or WebAssembly modifier parity.

## Verification contract

Before changing either producer, run the actual BondlineApp in an isolated development
wrapper. Its raw_input_hook records only pointer/focus/modifier information, with4096
event/row bounds, to a new local file next to the probe executable. No text, clipboard,
paths from dialogs, commands or document contents are captured. It has no listener.
Use real OS Control input: the baseline must first show focused raw modifier snapshots
true then false, then fail the desired ordered-observation assertion. Missing variants
at compile time are API qualification, not that behavioral RED.

After the patch, repeat with a fresh disposable bundle and keep the same assertion.
Check exact synthetic chronology across pointer events, conflicting final snapshots,
take/append, multiple frames, filters and viewport separation independently. Synthetic
SDK inputs do not establish OS coalescing, cross-platform operation or chain behavior.
Inspect the actual native app and keep both existing user bundles/drafts untouched.
Never open the installed proprietary reference. Public GUIDE-25 chain behavior and
remaining acceptance stay in the parent plan. Actions stays OFF.
