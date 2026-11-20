# Accessible menu activation

A macOS accessibility client can press View → Show Analysis Window while a
different application owns keyboard focus. Bondline displays the panel for
the current drawing or collection. Hide Analysis Window closes it. Object →
Chain constraints → Fixed Lengths also accepts an accessibility press when
the active tool supports that setting.

The original menu button accepted its exact AccessKit Default and closed its
popup. Command replay then required keyboard focus and discarded the selected
action on both drawing and collection surfaces. Replay now recognizes the
specific event index of a current, enabled menu button. At that event the
engine resolves the typed binding against current state. The focus requirement
still applies to physical input. The existing egui submenu separately needed
to open its child popup on activation. [Reviewed submenu patch](../vendor/egui/BONDLINE-PATCH.md)

## Automated regression

`cargo test -p bondline-ui --lib native_menu::` exercises three workflows.

- On a background drawing and an SDF collection, send a Default to the current
  View item, followed in the same frame by a zoom key and physical paper
  press/release. Only the panel state changes. A later Default targeting the
  retired item leaves the state unchanged.
- Activate the open Object submenu and toggle Fixed Lengths. Switch to the
  single-bond tool and send Default to the rendered disabled control. The
  setting, revision, document and history retain their prior values.
- Activate the menu by pointer over paper, then hide it with Enter on the
  focused item. Both operations preserve the document, history and selection.

The unmodified app failed the background action and could not expose the
submenu's child through Default. On macOS, the complete UI library suite
passed 372 cases with zero failures and one ignored test.

## Native receiving check

An isolated background-only macOS build used its own window accessibility
tree. The controller identified it by PID and exact signed executable path,
verified that it was never the frontmost application at each lookup/press,
and sent `AXPress` directly to each named AXButton. It issued no physical
keyboard or mouse events. All button hits were under AXWindow → AXGroup.
The previous build closed the Show popup while leaving the panel closed.
The fixed build showed and hid the panel on both an empty drawing and a
two-record SDF. The native tree exposed the corresponding visible result
text and inspection retained the exact drawing or collection and history.

Object → Chain constraints → Fixed Lengths changed the registered preference
only with the chain tool active. A press on the disabled item produced no
state change. This macOS adapter did not supply the optional AXEnabled value
for the inspected buttons. App frames separately verified the disabled flag.
The isolated process (PID 24038) closed normally. Receipt SHA-256
`473b6c8e9f39ae5945714393de9f4673293082f060723d49a8218bf073d0a27c`
at `target/ax-menu/native-33404hsk/receipt.json`. Windows UI Automation,
Linux AT-SPI and browser accessibility receiving remain unverified here.
