# Installed reference UI observations

Observed 2026-11-14 on the user's authorized installed reference application. This is
a bounded first pass, not a complete inventory or a claim that Bondline implements the
observed controls. The [installed feature map](installed-feature-map.md) and
[implementation tickets](installed-implementation-tickets.md) reconcile these observations
with the older, separately qualified public-document catalog.

## Exact target and method

| Item | Observed identity |
|---|---|
| Installed bundle | `/Applications/ChemDraw.app`; `com.revvity.ChemDraw` |
| Version/build | `26.1.0.6801` from bundle metadata; also identified by the writer in the original saved XML |
| Executable SHA-256 | `d2960e9f83de35787190b45acf9ecae5171e7662b7692ea24bea7a716ef14f83` |
| Platform | macOS 27.2, arm64 |
| Runtime selection | Existing PID 50732; original current document window 56131; owned new document window 56198 |
| Product tier | Unknown: no edition claim follows from the bundle/version, menus or file producer string |
| Research tool | Existing cached REA 6.9.0, run using an already installed Node 26.12.1; no installation, MCP registration or client configuration changes |

REA was used for exact PID/window-bound native observations and declared AX scenarios.
Complete canonical Evidence, decoded screenshots, invocation receipts, manual CUA extracts
and the two original saved documents remain ignored under `target/reference-ui/`. They are
research artifacts, not shipped assets or admitted corpus fixtures. CUA observations and
ordinary macOS metadata/screenshot fallbacks are labeled separately; they do not acquire
REA Evidence IDs.

The public [REA native investigation guide](https://github.com/morluto/rea/blob/main/docs/guides/native/native-investigation.md)
explains the selected-window method and leave-as-is behavior. Exact installed-tool source
also matters: cached `bridge/native/ReaNativeUI.swift` lines 73–80 admit only layer-zero
windows. Consequently the separate layer-three Tools palette is rejected with a generic
window-mismatch message. That error does not prove that the palette is absent. Target-only
CoreGraphics metadata and window-only `screencapture -x -l` successfully captured it.
Attached sheets can also fail direct AX-window geometry matching; observing their actual
document parent exposes their AX subtree. Parent screenshots do not substitute for the
separate sheet/palette image.

Some sheet snapshots explicitly set `truncated=true` and retain AX gaps, including
unsupported splitter actions and nonscalar radio-group values. REA uses that flag for
these gaps as well as traversal limits; a successful action is not a complete AX capture.
An initial archive check requiring every snapshot to be untruncated failed and remains
recorded. PNG/build/hash integrity is checked separately. Visible field-group observations
are bounded CUA/OS-image observations, not proof of full accessibility coverage.

## First menu and screen pass

The installed menu bar has fourteen entries, in order: application, File, Edit, View,
Object, Structure, Text, Curves, Colors, Search, Signals, Add-ins, Window and Help. The
ignored `cua-menu-ledger.json` records their native direct ordering, nested functional
labels, and the enabled/disabled state in an empty owned document. It includes the
stationery, template and sample **menu names**; no such library content was imported or
copied. Private Recent/open-document entries and host Services are not catalogued as
product capabilities. CUA AX text did not expose shortcut glyphs or menu checkmarks.

The blank document has a page canvas and scrollbars. Its AX layout area has no descendants
describing drawn atoms/bonds or drawing tools. A separate two-column Tools utility palette
and three horizontal strips have ignored window-only captures. Their geometry and visible
glyphs are evidence; an unnamed glyph alone does not establish a tool role, tooltip,
modifier or workflow. Original Bondline assets must be drawn independently.

The local Save sheet's Format popup was actually opened by REA. It exposes 28 nonseparator
choices, including native XML/binary, explicit V2000 and unqualified MOL/SDF/RXN/RD choices,
SVG/PDF/raster, CML and legacy interchange. Generic labels do not establish their output
version. Popup presence is not encoder correctness. Exact ordered labels are retained in
`12-save-format-scenario-rea.json` and the installed map.

## Original drawing and local persistence

All edits used a new blank owned document, never the preexisting sample or untitled work.
The ordered actions and preconditions are in `cua-flow-ledger.json`.

| Bounded observation | Evidence and limit |
|---|---|
| File New | A separate empty document appeared; REA `02-blank-rea.json` confirms its after-state. The New action itself was CUA, not a REA replay. |
| Documented solid-bond shortcut candidate and drag | CUA sent `x` (G25 p74/KG-002), then drag input `(280,300)` to `(340,300)`. Edited state and Undo Drawing appeared. Stable REA capture 07 shows one isolated line. Input tuples do not establish a coordinate transform, snapping or endpoint rule. |
| Undo/Redo | Undo Drawing returned to an empty, clean document; Redo Drawing restored Edited. Post-draw and post-redo PNG bytes have the same SHA-256 `4ff2e6fdf7fdb61c9f07e415e257bcfce7aa07cb6a60db99338dd5abf224b1ea`. This proves that bounded reference sequence, not general chemistry/history parity. |
| First unassociated Save/Save As | In the existing profile, both initially opened an embedded Save To Signals dialog. No cloud Save, sign-in, notebook creation or account action was performed. Once loaded it offered This Computer, which opened a local native Save sheet. No network trace was collected; automatic initialization/loading is not claimed to transmit zero data. |
| Local CDXML Save As | Native folder and unique filename were verified, XML and LF selected, then REA pressed the local Save button. A real local file appeared and the clean document became associated with it. |
| History after that Save As | Edit still exposed enabled Undo Drawing. This is direct 26.1 evidence for that route; do not generalize to every save mode/platform. |
| Subsequent CDX Save As | The same unchanged original drawing's Save As opened the native local sheet directly. Selecting the binary format removed the text EOL controls. REA pressed Save and a real binary file appeared. |

| Ignored original output | Bytes | SHA-256 |
|---|---:|---|
| `owned-original-bond-20261114.cdxml` | 2866 | `73de4b424e27a01e98be6830d726025fc5f36f8786d02bab594993bac12ed4b5` |
| `owned-original-bond-20261114.cdx` | 1196 | `7b2b71338595ad12bc34f46459f8389ec6a63cb453b1cf29b51543717139a73c` |

Python's independent XML parser sees two nodes and one bond in the XML; bytes contain
111 LF and zero CR. A limited tag scan found no image/embedded/OLE object. Writer style/font
references, print metadata and warning metadata are still present; this is not a complete
asset/license audit. The generated documents are ignored until provenance and any future
redistribution are reviewed. They are not substitutes for the pinned openly licensed corpus.

The decisive REA scenario IDs are
`ev_45c48aa6cb4d82a646ed5c43038c3d38adfb085f80da970a6097f29a5fe3c236`
(format popup),
`ev_7806f73a2e98d94f87aca959b2faa437924f989a59260f2176f26620061b0471`
(local XML save), and
`ev_a85f9c381b08fdf4354348187da9a422c78cd771164e4d0271334b67a2237dc6`
(local binary save). Their complete JSON includes ordered actions, captures and limitations.

The original owned document was closed before reopening either file. XML was reopened
through File Open, its observed cloud/local bridge, the exact local path and the enabled
local Open button. A new clean window 56477 represented that XML path; REA evidence
`ev_8a6a052772697ce4d139fcf8d129f6ebf53b4452be6991b978edc1e58fc820bd`
records its stable after-state. After closing that window, the binary was reopened through
its exact owned Recent entry into new clean window 56515; evidence
`ev_bdb8c4cbe43dd5b44feb39442e55dec95c169cec5b4ccbbc46bd9df81cdc6d17`
records that after-state. These Open/Recent actions were CUA, not REA action replays.
Both source files retained their exact hashes after native reopen and close. No post-reopen
atom edit or selection was tested. Reopened images contain a small red marker at the bond
site whose UI meaning was not established; no across-reopen pixel identity is asserted.
Reference-native opening does not establish Bondline parser support.

## Settings and remaining observations

Document Settings was opened on the owned document only. Its category list is Layout,
Header/Footer, Drawing, Text Captions, Atom Labels, Property Labels, Biopolymer Display,
Reaction Display and Colors. The initial Layout screen distinguishes Pages versus Poster,
page rows/columns, document dimensions, margins, overlap/registration marks, Page Setup
and display units. Drawing exposes chain angle, bond spacing with relative/absolute mode,
fixed length, bold/line widths, margin/hash spacing and separate atom/bond indicator groups.
Atom Labels exposes font-family/size selectors, line spacing, font styles, baseline
normal/subscript/superscript/formula modes, terminal-carbon labels and implicit-hydrogen
visibility. A selected OS font name does not grant permission to bundle that font.
These observed values belong to the selected ACS 1996 document profile; do not adopt them
as current journal requirements without the independent author guidelines.

Settings fields were inspected without editing or applying them. REA pressed Cancel and
observed the sheet disappear with the document still clean:
`ev_3e7e42f306b36a2179b7f40b253bd9f5e82a0495c3322b26a95d06e7cf861c25`.
The exact groups/values and separate sheet-image provenance are recorded in the ignored
`cua-settings-ledger.json` and `os-screenshot-manifest.json`. The other six categories,
tooltip-to-cell associations, contextual menus, modifier sequences, connected structures,
chemistry, multi-document prompts and other platforms remain open. No full milestone or
full parity claim follows from this research pass.

## Final state and continuation

This pass ended at 2026-11-14 01:11 UTC. All owned original/reopened document windows were
closed cleanly. The preexisting application PID remains running; no persistent REA
debugger/binary session was opened and all known CLI jobs are terminal. CUA completion was
issued exactly once after the final UI observation. GUI ownership was released. No explicit
sign-in, license activation, update, cloud submission or settings-apply action was performed.

Do **not** claim uninterrupted preservation of every original window. Cleanup found the
initial locked hotkeys sample window 56131 missing. It was present in metadata 04 and
absent by metadata 10; no intentional Close targeted it, and its disappearance cause is
unestablished. Its exact installed source was restored through its existing Recent entry;
CUA verified the original source URL, Locked state and current/focused document. The
restored window is 56535, with its original 1228×1005 size, at (278,254), whereas the initial
position was (689,147). The position was not restored. Original windows 56129/56130 retain
their exact frames. Four original palette IDs/frames also remain unchanged. Main Toolbar
was invoked twice during investigation; toggle restoration is inferred because checkmarks
were not exposed. Exact initial drawing-tool state was not established.

`24-final-cleanup-receipt.json` is the authoritative final state. `cua-flow-final.json`
extends the earlier flow extract by hash and explicitly supersedes its unverified
preservation assumption; it records reopen, cancellation, cleanup and the restoration
exception. Do not substitute the earlier checkpoint for this final receipt. Complete raw
Evidence and proprietary reference screenshots remain ignored and must not be shipped.

The next separately scoped UI pass is palette button/tooltip/role mapping, subject to
actual documented native interaction support, followed by individual feature flows on a
new owned blank document. Preserve unknown roles and missing behavior until observed.
